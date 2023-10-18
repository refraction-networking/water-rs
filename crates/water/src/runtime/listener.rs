use crate::runtime::*;

pub struct WATERListener<Host> {
    // WASM functions for reading & writing

    // the reader in WASM (read from net -- n2w)
    // returns the number of bytes read
    pub reader: Func,

    // the writer in WASM (write to net -- w2n)
    // returns the number of bytes written
    pub writer: Func,

    pub caller_reader: UnixStream, // the reader in Caller (read from WASM -- w2u)
    pub caller_writer: UnixStream, // the writer in Caller (write to WASM -- u2w)

    pub core: H2O<Host>, // core WASM runtime (engine, linker, instance, store, module)
}

impl WATERListener<Host> {
    /// Read from the target address
    pub fn read(&mut self, buf: &mut Vec<u8>) -> Result<i64, anyhow::Error> {
        info!("[HOST] WATERStream reading...");

        let mut res = vec![Val::I64(0); self.reader.ty(&self.core.store).results().len()];
        match self.reader.call(&mut self.core.store, &[], &mut res) {
            Ok(_) => {}
            Err(e) => {
                return Err(anyhow::Error::msg(format!(
                    "{} function failed: {}",
                    READER_FN, e
                )))
            }
        }

        let nums: i64 = match res.get(0) {
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
        match self.caller_reader.read(&mut buf[..]) {
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

    /// Write to the target address
    pub fn write(&mut self, buf: &[u8]) -> Result<(), anyhow::Error> {
        info!("[HOST] WATERStream writing...");

        // write to WASM's caller_writer
        match self.caller_writer.write_all(buf) {
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
        match self.writer.call(&mut self.core.store, &params, &mut res) {
            Ok(_) => {
                match res.get(0) {
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

    /// Listening at the addr:port with running the WASM listen function
    pub fn listen(
        &mut self,
        conf: &WATERConfig,
        addr: &str,
        port: u16,
    ) -> Result<(), anyhow::Error> {
        info!("[HOST] WATERStream listening...");

        // TODO: add addr:port sharing with WASM, for now WASM is using config.json's remote_addr:port
        let fnc = self
            .core
            .instance
            .get_func(&mut self.core.store, &conf.entry_fn)
            .unwrap();
        match fnc.call(&mut self.core.store, &[], &mut []) {
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

    pub fn init(conf: &WATERConfig) -> Result<Self, anyhow::Error> {
        info!("[HOST] WATERStream init...");

        let mut core = H2O::init(conf)?;
        core._prepare(conf)?;

        // constructing 2 pairs of UnixStream for communicating between WASM and Host
        // returns (read_end, write_end) for caller
        let (caller_read_end, water_write_end) = UnixStream::pair()?;
        let (water_read_end, caller_write_end) = UnixStream::pair()?;

        let water_write_file =
            unsafe { cap_std::fs::File::from_raw_fd(water_write_end.as_raw_fd()) };
        let water_read_file = unsafe { cap_std::fs::File::from_raw_fd(water_read_end.as_raw_fd()) };

        // insert file here
        let wasi_water_reader = wasmtime_wasi::sync::file::File::from_cap_std(water_read_file);
        let wasi_water_writer = wasmtime_wasi::sync::file::File::from_cap_std(water_write_file);

        std::mem::forget(water_write_end);
        std::mem::forget(water_read_end);

        let ctx = core.store.data_mut().preview1_ctx.as_mut().unwrap();
        let water_reader_fd = ctx.push_file(Box::new(wasi_water_reader), FileAccessMode::all())?;
        let water_writer_fd = ctx.push_file(Box::new(wasi_water_writer), FileAccessMode::all())?;

        let water_bridging = match core.instance.get_func(&mut core.store, WATER_BRIDGING_FN) {
            Some(func) => func,
            None => {
                return Err(anyhow::Error::msg(format!(
                    "{} function not found in WASM",
                    WATER_BRIDGING_FN
                )))
            }
        };

        let params = vec![
            Val::I32(water_reader_fd as i32),
            Val::I32(water_writer_fd as i32),
        ];
        match water_bridging.call(&mut core.store, &params, &mut []) {
            Ok(_) => {}
            Err(e) => {
                return Err(anyhow::Error::msg(format!(
                    "{} function failed: {}",
                    WATER_BRIDGING_FN, e
                )))
            }
        }

        // getting reader & writer func from WASM
        let reader = match core.instance.get_func(&mut core.store, READER_FN) {
            Some(func) => func,
            None => {
                return Err(anyhow::Error::msg(format!(
                    "{} function not found in WASM",
                    READER_FN
                )))
            }
        };

        let writer = match core.instance.get_func(&mut core.store, WRITER_FN) {
            Some(func) => func,
            None => {
                return Err(anyhow::Error::msg(format!(
                    "{} function not found in WASM",
                    WRITER_FN
                )))
            }
        };

        let runtime = WATERListener {
            reader: reader,
            writer: writer,

            caller_reader: caller_read_end,
            caller_writer: caller_write_end,

            core: core,
        };

        Ok(runtime)
    }
}
