use crate::runtime::{relay::WATERRelayTrait, transport::WATERTransportTrait, *};

pub struct WATERRelay<Host> {
    pub caller_io: Option<UnixStream>, // the pipe for communcating between Host and WASM
    pub cancel_io: Option<UnixStream>, // the UnixStream side for communcating between Host and WASM

    pub core: H2O<Host>, // core WASM runtime (engine, linker, instance, store, module)
}

impl WATERTransportTrait for WATERRelay<Host> {
    fn get_caller_io(&mut self) -> &mut Option<UnixStream> {
        &mut self.caller_io
    }

    fn get_cancel_io(&mut self) -> &mut Option<UnixStream> {
        &mut self.cancel_io
    }

    fn get_core(&mut self) -> &mut H2O<Host> {
        &mut self.core
    }

    fn set_caller_io(&mut self, caller_io: Option<UnixStream>) {
        self.caller_io = caller_io;
    }

    fn set_cancel_io(&mut self, cancel_io: Option<UnixStream>) {
        self.cancel_io = cancel_io;
    }
}

impl WATERRelayTrait for WATERRelay<Host> {
    /// Connect to the target address with running the WASM connect function
    fn associate(&mut self, conf: &WATERConfig) -> Result<(), anyhow::Error> {
        info!("[HOST] WATERRelay v0 associating...");

        let mut store = self
            .core
            .store
            .lock()
            .map_err(|e| anyhow::Error::msg(format!("Failed to lock store: {}", e)))?;

        let _water_associate = match self.core.instance.get_func(&mut *store, ASSOCIATE_FN) {
            Some(func) => func,
            None => {
                return Err(anyhow::Error::msg(format!(
                    "{} function not found in WASM",
                    ASSOCIATE_FN
                )))
            }
        };

        // calling the WATM associate function
        let mut res = vec![Val::I32(0); _water_associate.ty(&*store).results().len()];
        match _water_associate.call(&mut *store, &[], &mut res) {
            Ok(_) => {}
            Err(e) => {
                return Err(anyhow::Error::msg(format!(
                    "{} function failed: {}",
                    ASSOCIATE_FN, e
                )))
            }
        }

        if res[0].unwrap_i32() < 0 {
            return Err(anyhow::Error::msg(format!(
                "{} function failed: {}",
                ASSOCIATE_FN, "connection failed"
            )));
        }

        Ok(())
    }

    fn relay(&mut self, conf: &WATERConfig) -> Result<(), anyhow::Error> {
        info!("[HOST] WATERRelay v0 relaying...");

        // create listener
        match &mut self.core.version {
            Version::V0(v0_conf) => match v0_conf {
                Some(v0_conf) => match v0_conf.lock() {
                    Ok(mut v0_conf) => {
                        v0_conf.create_listener(true)?;
                    }
                    Err(e) => {
                        return Err(anyhow::anyhow!("Failed to lock v0_conf: {}", e))?;
                    }
                },
                None => {
                    return Err(anyhow::anyhow!("v0_conf is None"))?;
                }
            },
            _ => {}
        }

        Ok(())
    }
}

impl WATERRelay<Host> {
    pub fn init(_conf: &WATERConfig, core: H2O<Host>) -> Result<Self, anyhow::Error> {
        info!("[HOST] WATERRelay v0 init...");

        let runtime = WATERRelay {
            caller_io: None,
            cancel_io: None,
            core,
        };

        Ok(runtime)
    }
}
