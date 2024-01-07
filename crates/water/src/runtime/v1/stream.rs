//! This file contains the v1_preview WATERStream implementation,
//! it implements the WATERStreamTrait and WATERTransportTrait.

use crate::runtime::{stream::WATERStreamTrait, transport::WATERTransportTrait, *};

/// This file contains the WATERStream implementation
/// which is a TcpStream liked definition with utilizing WASM

///           UnixSocket          Connection created with Host
///    Write =>  u2w  +----------------+  w2n
///            -----> |  WATERStream   |------>
///    Caller        |  WASM Runtime  |  n2w    Destination
///           <----- | Decode/Encode  |<------
///    Read  => w2u  +----------------+
///                     WATERStream

pub struct WATERStream<Host> {
    // WASM functions for reading & writing

    // the reader in WASM (read from net -- n2w)
    // returns the number of bytes read
    pub reader: Func,

    // the writer in WASM (write to net -- w2n)
    // returns the number of bytes written
    pub writer: Func,

    pub caller_io: UnixStream, // the pipe for communcating between Host and WASM

    pub core: H2O<Host>, // core WASM runtime (engine, linker, instance, store, module)
}

impl WATERTransportTrait for WATERStream<Host> {
    /// Read from the target address thru the WATM module
    fn read(&mut self, buf: &mut Vec<u8>) -> Result<i64, anyhow::Error> {
        debug!("[HOST] WATERStream v1_preview reading...");

        let store_lock_result = self.core.store.lock();

        let mut store = match store_lock_result {
            Ok(store) => store,
            Err(e) => return Err(anyhow::Error::msg(format!("Failed to lock store: {}", e))),
        };

        let mut res = vec![Val::I64(0); self.reader.ty(&*store).results().len()];
        match self.reader.call(&mut *store, &[], &mut res) {
            Ok(_) => {}
            Err(e) => {
                return Err(anyhow::Error::msg(format!(
                    "{} function failed: {}",
                    READER_FN, e
                )))
            }
        }

        let nums: i64 = match res.first() {
            Some(wasmtime::Val::I64(v)) => *v,
            _ => {
                return Err(anyhow::Error::msg(format!(
                    "{} function returned unexpected type / no return",
                    READER_FN
                )))
            }
        };

        // read from WASM's caller_reader
        buf.resize(nums as usize, 0);
        match self.caller_io.read(&mut buf[..]) {
            Ok(_) => {}
            Err(e) => {
                return Err(anyhow::Error::msg(format!(
                    "failed to read from caller_reader: {}",
                    e
                )))
            }
        }

        Ok(nums)
    }

    /// Write to the target address thru the WATM module
    fn write(&mut self, buf: &[u8]) -> Result<(), anyhow::Error> {
        debug!("[HOST] WATERStream v1_preview writing...");

        let store_lock_result = self.core.store.lock();

        let mut store = match store_lock_result {
            Ok(store) => store,
            Err(e) => return Err(anyhow::Error::msg(format!("Failed to lock store: {}", e))),
        };

        // write to WASM's caller_writer
        match self.caller_io.write_all(buf) {
            Ok(_) => {}
            Err(e) => {
                return Err(anyhow::Error::msg(format!(
                    "failed to write to caller_writer: {}",
                    e
                )))
            }
        }

        let params = vec![Val::I64(buf.len() as i64)];
        let mut res = vec![Val::I64(0)];
        match self.writer.call(&mut *store, &params, &mut res) {
            Ok(_) => {
                match res.first() {
                    Some(wasmtime::Val::I64(v)) => {
                        if *v != buf.len() as i64 {
                            return Err(anyhow::Error::msg(format!(
                                "WASM write function returned unexpected value: {}",
                                *v
                            )));
                        }
                    }
                    _ => {
                        return Err(anyhow::Error::msg(
                            "user_write_done function returned unexpected type / no return",
                        ))
                    }
                };
            }
            Err(e) => {
                return Err(anyhow::Error::msg(format!(
                    "{} function failed: {}",
                    WRITER_FN, e
                )))
            }
        }

        Ok(())
    }
}

impl WATERStreamTrait for WATERStream<Host> {
    /// Connect to the target address with running the WATM connect function
    fn connect(&mut self, _conf: &WATERConfig) -> Result<(), anyhow::Error> {
        info!("[HOST] WATERStream v1_preview connecting...");

        let store_lock_result = self.core.store.lock();

        let mut store = match store_lock_result {
            Ok(store) => store,
            Err(e) => return Err(anyhow::Error::msg(format!("Failed to lock store: {}", e))),
        };

        let fnc = match self.core.instance.get_func(&mut *store, DIAL_FN) {
            Some(func) => func,
            None => {
                return Err(anyhow::Error::msg(format!(
                    "{} function not found in WASM",
                    DIAL_FN
                )))
            }
        };

        match fnc.call(&mut *store, &[], &mut []) {
            Ok(_) => {}
            Err(e) => {
                return Err(anyhow::Error::msg(format!(
                    "connect function failed: {}",
                    e
                )))
            }
        }

        Ok(())
    }
}

impl WATERStream<Host> {
    pub fn init(_conf: &WATERConfig, core: H2O<Host>) -> Result<Self, anyhow::Error> {
        info!("[HOST] WATERStream v1_preview...");

        // constructing a pair of UnixStream for communicating between WASM and Host
        let (caller_io, water_io) = UnixStream::pair()?;

        let water_io_file = unsafe { cap_std::fs::File::from_raw_fd(water_io.as_raw_fd()) };

        // insert file here
        let water_io_file = wasmtime_wasi::sync::file::File::from_cap_std(water_io_file);

        std::mem::forget(water_io); // forget the water_io, so that it won't be closed

        let reader;
        let writer;

        {
            let mut store = core
                .store
                .lock()
                .map_err(|e| anyhow::Error::msg(format!("Failed to lock store: {}", e)))?;

            let ctx = store
                .data_mut()
                .preview1_ctx
                .as_mut()
                .context("Failed to retrieve preview1_ctx from Host")?;
            let water_io_fd = ctx.push_file(Box::new(water_io_file), FileAccessMode::all())?;

            let water_bridging = match core.instance.get_func(&mut *store, WATER_BRIDGING_FN) {
                Some(func) => func,
                None => {
                    return Err(anyhow::Error::msg(format!(
                        "{} function not found in WASM",
                        WATER_BRIDGING_FN
                    )))
                }
            };

            let params: Vec<Val> = vec![Val::I32(water_io_fd as i32)];
            match water_bridging.call(&mut *store, &params, &mut []) {
                Ok(_) => {}
                Err(e) => {
                    return Err(anyhow::Error::msg(format!(
                        "{} function failed: {}",
                        WATER_BRIDGING_FN, e
                    )))
                }
            }

            // getting reader & writer func from WASM
            reader = match core.instance.get_func(&mut *store, READER_FN) {
                Some(func) => func,
                None => {
                    return Err(anyhow::Error::msg(format!(
                        "{} function not found in WASM",
                        READER_FN
                    )))
                }
            };

            writer = match core.instance.get_func(&mut *store, WRITER_FN) {
                Some(func) => func,
                None => {
                    return Err(anyhow::Error::msg(format!(
                        "{} function not found in WASM",
                        WRITER_FN
                    )))
                }
            };
        }

        let runtime = WATERStream {
            reader,
            writer,

            caller_io,

            core,
        };

        Ok(runtime)
    }
}
