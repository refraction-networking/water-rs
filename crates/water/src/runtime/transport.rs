use std::thread::JoinHandle;

use crate::runtime::*;

pub trait WATERTransportTrait: Send {
    // ============================ all version ============================
    fn read(&mut self, buf: &mut Vec<u8>) -> Result<i64, anyhow::Error> {
        info!("[HOST] WATERTransport v0 reading...");

        let caller_io = self.get_caller_io();

        // read from WASM's caller_reader
        match caller_io {
            Some(ref mut caller_io) => match caller_io.read(buf) {
                Ok(n) => Ok(n as i64),
                Err(e) => Err(anyhow::Error::msg(format!(
                    "failed to read from caller_reader: {}",
                    e
                ))),
            },
            None => Err(anyhow::Error::msg(format!(
                "read function failed: {}",
                "caller_io is None"
            ))),
        }
    }

    fn write(&mut self, buf: &[u8]) -> Result<(), anyhow::Error> {
        info!("[HOST] WATERTransport v0 writing...");

        let caller_io = self.get_caller_io();

        // write to WASM's caller_writer
        match caller_io {
            Some(ref mut caller_io) => match caller_io.write_all(buf) {
                Ok(_) => Ok(()),
                Err(e) => Err(anyhow::Error::msg(format!(
                    "failed to write to caller_writer: {}",
                    e
                ))),
            },
            None => Err(anyhow::Error::msg(format!(
                "write function failed: {}",
                "caller_io is None"
            ))),
        }
    }

    // ============================ v0 only ============================
    // Methods to provide access to the shared state, not implemented by default
    fn get_caller_io(&mut self) -> &mut Option<UnixStream> {
        unimplemented!("get_caller_io not implemented")
    }
    fn get_cancel_io(&mut self) -> &mut Option<UnixStream> {
        unimplemented!("get_cancel_io not implemented")
    }
    fn get_core(&mut self) -> &mut H2O<Host> {
        unimplemented!("get_core not implemented")
    }

    fn set_caller_io(&mut self, caller_io: Option<UnixStream>) {
        unimplemented!("set_caller_io not implemented")
    }
    fn set_cancel_io(&mut self, cancel_io: Option<UnixStream>) {
        unimplemented!("set_cancel_io not implemented")
    }

    fn cancel_with(&mut self, _conf: &WATERConfig) -> Result<(), anyhow::Error> {
        info!("[HOST] WATERTransport v0 cancel_with...");

        let (caller_io, water_io) = UnixStream::pair()?;

        self.set_cancel_io(Some(caller_io));

        let water_io_file = unsafe { cap_std::fs::File::from_raw_fd(water_io.as_raw_fd()) };

        // insert file here
        let water_io_file = wasmtime_wasi::sync::file::File::from_cap_std(water_io_file);

        std::mem::forget(water_io); // forget the water_io, so that it won't be closed

        let core = self.get_core();

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

        let _water_cancel_with = match core.instance.get_func(&mut *store, CANCEL_FN) {
            Some(func) => func,
            None => {
                return Err(anyhow::Error::msg(format!(
                    "{} function not found in WASM",
                    CANCEL_FN
                )))
            }
        };

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

    fn cancel(&mut self, _conf: &WATERConfig) -> Result<(), anyhow::Error> {
        info!("[HOST] WATERTransport v0 cancel...");

        let cancel_io = self.get_cancel_io();

        match cancel_io {
            Some(ref mut cancel_io) => {
                // write anything to cancel
                match cancel_io.write_all(&[0]) {
                    Ok(_) => Ok(()),
                    Err(e) => Err(anyhow::Error::msg(format!(
                        "failed to write to cancel_io: {}",
                        e
                    ))),
                }
            }
            None => Err(anyhow::Error::msg(format!(
                "cancel function failed: {}",
                "cancel_io is None"
            ))),
        }
    }

    fn run_entry_fn(
        &mut self,
        conf: &WATERConfig,
    ) -> Result<JoinHandle<Result<(), anyhow::Error>>, anyhow::Error> {
        info!(
            "[HOST] WATERTransport v0 running entry_fn {}...",
            conf.entry_fn
        );

        let core = self.get_core();

        let store = Arc::clone(&core.store);
        let entry_fn = {
            let mut store = store
                .lock()
                .map_err(|e| anyhow::Error::msg(format!("Failed to lock store: {}", e)))?;
            match core
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

        // run the entry_fn in a thread -- Host will still have the ability to control it (e.g. with cancel)
        let handle = std::thread::spawn(move || {
            let mut store = store
                .lock()
                .map_err(|e| anyhow::Error::msg(format!("Failed to lock store: {}", e)))?;
            let mut res = vec![Val::I32(0); entry_fn.ty(&mut *store).results().len()];
            match entry_fn.call(&mut *store, &[], &mut res) {
                Ok(_) => Ok(()),
                Err(e) => Err(anyhow::Error::msg(format!("function failed: {}", e))),
            }
        });

        Ok(handle)
    }

    // fn read(&mut self, _buf: &mut Vec<u8>) -> Result<i64, anyhow::Error> {
    //     Err(anyhow::anyhow!("Method not supported"))
    // }

    // fn write(&mut self, _buf: &[u8]) -> Result<(), anyhow::Error> {
    //     Err(anyhow::anyhow!("Method not supported"))
    // }

    // // v0 only
    // fn cancel_with(&mut self, _conf: &WATERConfig) -> Result<(), anyhow::Error> {
    //     Err(anyhow::anyhow!("Method not supported"))
    // }

    // // v0 only
    // fn cancel(&mut self, _conf: &WATERConfig) -> Result<(), anyhow::Error> {
    //     Err(anyhow::anyhow!("Method not supported"))
    // }

    // // v0 only
    // fn run_entry_fn(
    //     &mut self,
    //     _conf: &WATERConfig,
    // ) -> Result<std::thread::JoinHandle<Result<(), anyhow::Error>>, anyhow::Error> {
    //     Err(anyhow::anyhow!("Method not supported"))
    // }

}
