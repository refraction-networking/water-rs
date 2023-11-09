// use crate::runtime::{listener::WATERListenerTrait, v0::transport::WATERTransportTraitV0, *, transport::WATERTransportTrait};
use crate::runtime::{*, listener::WATERListenerTrait, transport::WATERTransportTrait};

pub struct WATERListener<Host> {
    pub caller_io: Option<UnixStream>, // the pipe for communcating between Host and WASM
    pub cancel_io: Option<UnixStream>, // the UnixStream side for communcating between Host and WASM

    pub core: H2O<Host>, // core WASM runtime (engine, linker, instance, store, module)
}

// impl WATERTransportTrait for WATERListener<Host> {}

impl WATERTransportTrait for WATERListener<Host> {
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

impl WATERListenerTrait for WATERListener<Host> {
    /// Connect to the target address with running the WASM connect function
    fn listen(&mut self, conf: &WATERConfig)
            -> Result<(), anyhow::Error> {
        info!("[HOST] WATERListener v0 create listener...");

        Ok(())
    }

    fn accept(&mut self, conf: &WATERConfig)
            -> Result<(), anyhow::Error> {
        info!("[HOST] WATERListener v0 accepting...");

        let (caller_io, water_io) = UnixStream::pair()?;
        self.caller_io = Some(caller_io);

        // push the WATM end of the Unixpipe to WATM
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

        let _water_accept = match self.core.instance.get_func(&mut *store, ACCEPT_FN) {
            Some(func) => func,
            None => {
                return Err(anyhow::Error::msg(format!(
                    "{} function not found in WASM",
                    ACCEPT_FN
                )))
            }
        };

        // calling the WASM dial function
        let params: Vec<Val> = vec![Val::I32(water_io_fd as i32)];
        let mut res = vec![Val::I32(0); _water_accept.ty(&*store).results().len()];
        match _water_accept.call(&mut *store, &params, &mut res) {
            Ok(_) => {}
            Err(e) => {
                return Err(anyhow::Error::msg(format!(
                    "{} function failed: {}",
                    ACCEPT_FN, e
                )))
            }
        }

        if res[0].unwrap_i32() < 0 {
            return Err(anyhow::Error::msg(format!(
                "{} function failed: {}",
                ACCEPT_FN, "connection failed"
            )));
        }

        Ok(())
    }
}

impl WATERListener<Host> {
    pub fn init(_conf: &WATERConfig, core: H2O<Host>) -> Result<Self, anyhow::Error> {
        info!("[HOST] WATERListener v0 init...");

        let runtime = WATERListener {
            caller_io: None,
            cancel_io: None,
            core,
        };

        Ok(runtime)
    }
}
