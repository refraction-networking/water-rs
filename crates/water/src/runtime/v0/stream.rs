use std::thread::JoinHandle;

use crate::runtime::{stream::WATERStreamTrait, *};

/// This file contains the WATERStream implementation
/// which is a TcpStream liked definition with utilizing WASM

//           UnixSocket          Connection created with Host
//    Write =>  u2w  +----------------+  w2n
//		       ----->|  WATERStream   |------>
//		Caller       |  WASM Runtime  |  n2w    Destination
//		       <-----| Decode/Encode  |<------
//    Read  =>  w2u  +----------------+
//	                    WATERStream

pub struct WATERStream<Host> {
    pub caller_io: Option<UnixStream>, // the pipe for communcating between Host and WASM
    pub cancel_io: Option<UnixStream>, // the UnixStream side for communcating between Host and WASM

    pub core: H2O<Host>, // core WASM runtime (engine, linker, instance, store, module)
}

impl WATERStreamTrait for WATERStream<Host> {
    /// Connect to the target address with running the WASM connect function
    fn connect(
        &mut self,
        conf: &WATERConfig,
        _addr: &str,
        _port: u16,
    ) -> Result<(), anyhow::Error> {
        info!("[HOST] WATERStream v0 connecting...");

        let (caller_io, water_io) = UnixStream::pair()?;
        self.caller_io = Some(caller_io);

        let water_io_file = unsafe { cap_std::fs::File::from_raw_fd(water_io.as_raw_fd()) };

        // insert file here
        let water_io_file = wasmtime_wasi::sync::file::File::from_cap_std(water_io_file);

        std::mem::forget(water_io); // forget the water_io, so that it won't be closed

        let mut store = self
            .core
            .store
            .lock()
            .map_err(|e| anyhow::Error::msg(format!("Failed to lock store: {}", e)))?;

        let ctx = store
            .data_mut()
            .preview1_ctx
            .as_mut()
            .context("Failed to retrieve preview1_ctx from Host")?;

        let water_io_fd = ctx.push_file(Box::new(water_io_file), FileAccessMode::all())?;

        let _water_dial = match self.core.instance.get_func(&mut *store, DIAL_FN) {
            Some(func) => func,
            None => {
                return Err(anyhow::Error::msg(format!(
                    "{} function not found in WASM",
                    DIAL_FN
                )))
            }
        };

        // let params = vec![Val::I32(water_reader_fd as i32), Val::I32(water_writer_fd as i32)];
        let params: Vec<Val> = vec![Val::I32(water_io_fd as i32)];
        let mut res = vec![Val::I32(0); _water_dial.ty(&*store).results().len()];
        match _water_dial.call(&mut *store, &params, &mut res) {
            Ok(_) => {}
            Err(e) => {
                return Err(anyhow::Error::msg(format!(
                    "{} function failed: {}",
                    DIAL_FN, e
                )))
            }
        }

        if res[0].unwrap_i32() < 0 {
            return Err(anyhow::Error::msg(format!(
                "{} function failed: {}",
                DIAL_FN, "connection failed"
            )));
        }

        Ok(())
    }

    fn cancel_with(&mut self, conf: &WATERConfig) -> Result<(), anyhow::Error> {
        info!("[HOST] WATERStream v0 cancel_with...");

        let (caller_io, water_io) = UnixStream::pair()?;
        self.cancel_io = Some(caller_io);

        let water_io_file = unsafe { cap_std::fs::File::from_raw_fd(water_io.as_raw_fd()) };

        // insert file here
        let water_io_file = wasmtime_wasi::sync::file::File::from_cap_std(water_io_file);

        std::mem::forget(water_io); // forget the water_io, so that it won't be closed

        let mut store = self
            .core
            .store
            .lock()
            .map_err(|e| anyhow::Error::msg(format!("Failed to lock store: {}", e)))?;

        let ctx = store
            .data_mut()
            .preview1_ctx
            .as_mut()
            .context("Failed to retrieve preview1_ctx from Host")?;

        let water_io_fd = ctx.push_file(Box::new(water_io_file), FileAccessMode::all())?;

        let _water_cancel_with = match self.core.instance.get_func(&mut *store, CANCEL_FN) {
            Some(func) => func,
            None => {
                return Err(anyhow::Error::msg(format!(
                    "{} function not found in WASM",
                    DIAL_FN
                )))
            }
        };

        // let params = vec![Val::I32(water_reader_fd as i32), Val::I32(water_writer_fd as i32)];
        let params: Vec<Val> = vec![Val::I32(water_io_fd as i32)];
        let mut res = vec![Val::I32(0); _water_cancel_with.ty(&*store).results().len()];
        match _water_cancel_with.call(&mut *store, &params, &mut res) {
            Ok(_) => {}
            Err(e) => {
                return Err(anyhow::Error::msg(format!(
                    "{} function failed: {}",
                    CANCEL_FN, e
                )))
            }
        }

        if res[0].unwrap_i32() != 0 {
            return Err(anyhow::Error::msg(format!(
                "{} function failed: {}",
                CANCEL_FN, "connection failed"
            )));
        }

        Ok(())
    }

    fn cancel(&mut self, conf: &WATERConfig) -> Result<(), anyhow::Error> {
        info!("[HOST] WATERStream v0 cancel...");

        match self.cancel_io {
            Some(ref mut cancel_io) => {
                // write anything to cancel
                match cancel_io.write_all(&[0]) {
                    Ok(_) => Ok(()),
                    Err(e) => {
                        return Err(anyhow::Error::msg(format!(
                            "failed to write to cancel_io: {}",
                            e
                        )))
                    }
                }
            }
            None => {
                return Err(anyhow::Error::msg(format!(
                    "cancel function failed: {}",
                    "cancel_io is None"
                )))
            }
        }
    }

    fn run_entry_fn(
        &mut self,
        conf: &WATERConfig,
    ) -> Result<JoinHandle<Result<(), anyhow::Error>>, anyhow::Error> {
        info!(
            "[HOST] WATERStream v0 running entry_fn {}...",
            conf.entry_fn
        );

        let store = Arc::clone(&self.core.store);
        let entry_fn = {
            let mut store = store.lock().unwrap();
            match self
                .core
                .instance
                .get_func(&mut *store, conf.entry_fn.as_str())
            {
                Some(func) => func,
                None => {
                    return Err(anyhow::Error::msg(format!(
                        "{} function not found in WASM",
                        conf.entry_fn
                    )))
                }
            }
        };

        let handle = std::thread::spawn(move || {
            let mut store = store.lock().unwrap();
            let mut res = vec![Val::I32(0); entry_fn.ty(&mut *store).results().len()];
            match entry_fn.call(&mut *store, &[], &mut res) {
                Ok(_) => Ok(()),
                Err(e) => return Err(anyhow::Error::msg(format!("function failed: {}", e))),
            }
        });

        Ok(handle)
    }

    fn read(&mut self, buf: &mut Vec<u8>) -> Result<i64, anyhow::Error> {
        info!("[HOST] WATERStream reading...");

        // read from WASM's caller_reader
        match self.caller_io {
            Some(ref mut caller_io) => match caller_io.read(buf) {
                Ok(n) => Ok(n as i64),
                Err(e) => {
                    return Err(anyhow::Error::msg(format!(
                        "failed to read from caller_reader: {}",
                        e
                    )))
                }
            },
            None => {
                return Err(anyhow::Error::msg(format!(
                    "read function failed: {}",
                    "caller_io is None"
                )))
            }
        }
    }

    fn write(&mut self, buf: &[u8]) -> Result<(), anyhow::Error> {
        info!("[HOST] WATERStream writing...");

        // write to WASM's caller_writer
        match self.caller_io {
            Some(ref mut caller_io) => match caller_io.write_all(buf) {
                Ok(_) => Ok(()),
                Err(e) => {
                    return Err(anyhow::Error::msg(format!(
                        "failed to write to caller_writer: {}",
                        e
                    )))
                }
            },
            None => {
                return Err(anyhow::Error::msg(format!(
                    "write function failed: {}",
                    "caller_io is None"
                )))
            }
        }
    }
}

impl WATERStream<Host> {
    pub fn init(conf: &WATERConfig, mut core: H2O<Host>) -> Result<Self, anyhow::Error> {
        info!("[HOST] WATERStream v0_init...");
        
        let runtime = WATERStream {
            caller_io: None,
            cancel_io: None,
            core,
        };

        Ok(runtime)
    }
}
