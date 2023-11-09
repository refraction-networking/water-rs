use std::sync::Mutex;

use tracing::Instrument;

use crate::runtime::*;

#[derive(Default, Clone)]
pub struct Host {
    pub preview1_ctx: Option<wasmtime_wasi::WasiCtx>,
    pub wasi_threads: Option<Arc<WasiThreadsCtx<Host>>>,
}

#[derive(Clone)]
pub struct H2O<Host> {
    pub version: Version,

    pub engine: Engine,
    pub linker: Linker<Host>,
    pub instance: Instance,
    pub store: Arc<Mutex<Store<Host>>>,
    pub module: Module,
}

impl H2O<Host> {
    pub fn init(conf: &WATERConfig) -> Result<Self, anyhow::Error> {
        info!("[HOST] WATERCore H2O initing...");

        let mut wasm_config = wasmtime::Config::new();
        wasm_config.wasm_threads(true);

        let engine = Engine::new(&wasm_config)?;
        let mut linker: Linker<Host> = Linker::new(&engine);

        let module = Module::from_file(&engine, &conf.filepath)?;

        let host = Host::default();
        let mut store = Store::new(&engine, host);

        let mut error_occured = None;

        // Get the version global from WATM
        let version = module.exports().find_map(|global| {
            match Version::parse(global.name()) {
                Some(mut v) => {
                    info!("[HOST] WATERCore found version: {:?}", v.as_str());
                    match v {
                        Version::V0(_) => match v.config_v0(conf) {
                            Ok(v) => Some(v),
                            Err(e) => {
                                info!("[HOST] WATERCore failed to configure for V0: {}", e);
                                error_occured = Some(e);
                                None
                            }
                        },
                        _ => Some(v), // for now only V0 needs to be configured
                    }
                }
                None => None,
            }
        });

        if version.is_none() {
            if let Some(e) = error_occured {
                return Err(e);
            }
            return Err(anyhow::Error::msg("WATM module version not found"));
        }

        store.data_mut().preview1_ctx = Some(WasiCtxBuilder::new().inherit_stdio().build());

        if store.data().preview1_ctx.is_none() {
            return Err(anyhow::anyhow!(
                "[HOST] WATERCore Failed to retrieve preview1_ctx from Host"
            ));
        }

        wasmtime_wasi::add_to_linker(&mut linker, |h: &mut Host| h.preview1_ctx.as_mut().unwrap())?;

        // initializing stuff for multithreading -- currently not used yet (v1+ feature)
        #[cfg(feature = "multithread")]
        {
            store.data_mut().wasi_threads = Some(Arc::new(WasiThreadsCtx::new(
                module.clone(),
                Arc::new(linker.clone()),
            )?));

            wasmtime_wasi_threads::add_to_linker(&mut linker, &store, &module, |h: &mut Host| {
                h.wasi_threads
                    .as_ref()
                    .context("Failed to get ref of wasi_threads from Host")?
            })?;
        }

        // export functions -- version dependent -- has to be done before instantiate
        match &version {
            Some(Version::V0(ref config)) => match config {
                Some(v0_conf) => {
                    let v0_conf = Arc::new(Mutex::new(v0_conf.clone()));
                    v0::funcs::export_tcp_connect(&mut linker, Arc::clone(&v0_conf))?;
                    v0::funcs::export_accept(&mut linker, Arc::clone(&v0_conf))?;
                    v0::funcs::export_defer(&mut linker, Arc::clone(&v0_conf))?;

                    // if client_type is Listen, then create a listener with the same config
                    match conf.client_type {
                        WaterBinType::Listen => {
                            match v0_conf.lock() {
                                Ok(mut v0_conf) => {
                                    v0_conf.create_listener()?;
                                }
                                Err(e) => {
                                    return Err(anyhow::anyhow!(
                                        "Failed to lock v0_conf: {}",
                                        e
                                    ))?;
                                }
                            }
                        }
                        _ => {}
                    }
                }
                None => {
                    return Err(anyhow::anyhow!(
                        "v0_conf wasn't initialized / setup correctly"
                    ))?;
                }
            },
            Some(Version::V1) => {
                v1::funcs::export_tcp_connect(&mut linker)?;
                v1::funcs::export_tcplistener_create(&mut linker)?;
            }
            _ => {
                unimplemented!("This version is not supported yet")
            } // add export funcs for other versions here
        }

        // export functions -- version independent
        {
            version_common::funcs::export_config(&mut linker, conf.config_wasm.clone())?;
        }

        let instance = linker.instantiate(&mut store, &module)?;

        Ok(H2O {
            version: match version {
                Some(v) => v,
                None => {
                    return Err(anyhow::anyhow!("Version is None"));
                }
            },

            engine,
            linker,
            instance,
            store: Arc::new(Mutex::new(store)),
            module,
        })
    }

    pub fn _prepare(&mut self, conf: &WATERConfig) -> Result<(), anyhow::Error> {
        self._init(conf.debug)?;
        self._process_config(conf)?; // This is for now needed only by v1_preview
        Ok(())
    }

    pub fn _init(&mut self, debug: bool) -> Result<(), anyhow::Error> {
        info!("[HOST] WATERCore calling _init from WASM...");

        let store_lock_result = self.store.lock();

        let mut store = match store_lock_result {
            Ok(store) => store,
            Err(e) => return Err(anyhow::Error::msg(format!("Failed to lock store: {}", e))),
        };

        let init_fn = match self.instance.get_func(&mut *store, INIT_FN) {
            Some(func) => func,
            None => return Err(anyhow::Error::msg("init function not found")),
        };

        // check if we need to pass in any arguments / configs later
        let params = vec![Val::I32(debug as i32); init_fn.ty(&*store).params().len()];
        let mut res = vec![Val::I64(0); init_fn.ty(&*store).results().len()];
        match init_fn.call(&mut *store, &params, &mut res) {
            Ok(_) => {}
            Err(e) => return Err(anyhow::Error::msg(format!("init function failed: {}", e))),
        }

        Ok(())
    }

    pub fn _process_config(&mut self, config: &WATERConfig) -> Result<(), anyhow::Error> {
        info!("[HOST] WATERCore calling _process_config from WASM...");

        let store_lock_result = self.store.lock();

        let mut store = match store_lock_result {
            Ok(store) => store,
            Err(e) => return Err(anyhow::Error::msg(format!("Failed to lock store: {}", e))),
        };

        // _required to implement _process_config(i32) in WASM, which will be parsing all the configurations
        let config_fn = match self.instance.get_func(&mut *store, CONFIG_FN) {
            Some(func) => func,
            None => {
                // Currently not going to return error, where V0 don't need config;
                // NOTE: remove this function for v1_preview as well, where config will be pulled from WASM
                info!("config function not found -- skipping");
                return Ok(());
            }
        };

        // Obtain the directory path and file name from config_wasm
        let full_path = Path::new(&config.config_wasm);
        let parent_dir = full_path
            .parent()
            .ok_or_else(|| anyhow::anyhow!("config_wasm does not have a parent directory"))?; // Assumes config_wasm has a parent directory
        let file_name = full_path
            .file_name()
            .and_then(|os_str| os_str.to_str())
            .ok_or_else(|| anyhow::anyhow!("file_name is not valid UTF-8"))?; // Assumes file_name is valid UTF-8

        // Open the parent directory
        let dir = Dir::open_ambient_dir(parent_dir, ambient_authority())?;

        let wasi_file = dir.open_with(file_name, OpenOptions::new().read(true).write(true))?;

        let wasi_file = wasmtime_wasi::sync::file::File::from_cap_std(wasi_file);

        let ctx = store
            .data_mut()
            .preview1_ctx
            .as_mut()
            .ok_or(anyhow::anyhow!("preview1_ctx in Store is None"))?;

        // push the config file into WATM
        let config_fd = ctx.push_file(Box::new(wasi_file), FileAccessMode::all())? as i32;

        let params = vec![Val::I32(config_fd); config_fn.ty(&*store).params().len()];
        match config_fn.call(&mut *store, &params, &mut []) {
            Ok(_) => {}
            Err(e) => {
                return Err(anyhow::Error::msg(format!(
                    "_process_config function in WASM failed: {}",
                    e
                )))
            }
        }

        Ok(())
    }
}
