//! This is the core of the runtime, which is responsible for loading the WASM module and
//! initializing the runtime. It also provides the interface for the host to interact with the runtime.

use std::sync::Mutex;

use crate::runtime::*;

/// Host is storing the WasiCtx that we are using, and for the later features will also support the WasiThreadsCtx
#[derive(Default, Clone)]
pub struct Host {
    pub preview1_ctx: Option<wasmtime_wasi::WasiCtx>,
    pub wasi_threads: Option<Arc<WasiThreadsCtx<Host>>>,
}

/// This is the core of the runtime, which stores the necessary components for a WASM runtime and the version of the WATM module.
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
    /// generate a new H2O core instance
    pub fn init_core(conf: &WATERConfig) -> Result<Self, anyhow::Error> {
        info!("[HOST] WATERCore H2O initing...");

        let wasm_config = wasmtime::Config::new();

        #[cfg(feature = "multithread")]
        {
            wasm_config.wasm_threads(true);
        }

        let engine = Engine::new(&wasm_config)?;
        let linker: Linker<Host> = Linker::new(&engine);

        let module = Module::from_file(&engine, &conf.filepath)?;

        let host = Host::default();
        let store = Store::new(&engine, host);

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

        // MUST have a version -- otherwise return error
        if version.is_none() {
            if let Some(e) = error_occured {
                return Err(e);
            }
            return Err(anyhow::Error::msg("WATM module version not found"));
        }

        Self::create_core(conf, linker, store, module, engine, version)
    }

    pub fn create_core(
        conf: &WATERConfig,
        mut linker: Linker<Host>,
        mut store: Store<Host>,
        module: Module,
        engine: Engine,
        version: Option<Version>,
    ) -> Result<Self, anyhow::Error> {
        store.data_mut().preview1_ctx = Some(WasiCtxBuilder::new().inherit_stdio().build());

        if store.data().preview1_ctx.is_none() {
            return Err(anyhow::anyhow!(
                "[HOST] WATERCore Failed to retrieve preview1_ctx from Host"
            ));
        }

        wasmtime_wasi::add_to_linker(&mut linker, |h: &mut Host| h.preview1_ctx.as_mut().unwrap())?;

        /// initialization for WASI-multithread -- currently not completed / used (v1+ feature)
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
            // V0 export functions
            Some(Version::V0(ref config)) => match config {
                Some(v0_conf) => {
                    v0::funcs::export_tcp_connect(&mut linker, Arc::clone(v0_conf))?;
                    v0::funcs::export_accept(&mut linker, Arc::clone(v0_conf))?;
                    v0::funcs::export_defer(&mut linker, Arc::clone(v0_conf))?;
                }
                None => {
                    return Err(anyhow::anyhow!(
                        "v0_conf wasn't initialized / setup correctly"
                    ))?;
                }
            },

            // V1 export functions
            Some(Version::V1) => {
                v1::funcs::export_tcp_connect(&mut linker)?;
                v1::funcs::export_tcplistener_create(&mut linker)?;
            }
            // add export funcs for other versions here
            _ => {
                unimplemented!("This version is not supported yet")
            }
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

    // This function is for migrating the v0 core for listener and relay
    // to handle every new connection is creating a new separate core (as v0 spec)
    pub fn v0_migrate_core(conf: &WATERConfig, core: &H2O<Host>) -> Result<Self, anyhow::Error> {
        info!("[HOST] WATERCore H2O v0_migrating...");

        // reseting the listener accepted_fd or the relay's accepted_fd & dial_fd
        // when migrating from existed listener / relay
        let version = match &core.version {
            Version::V0(v0conf) => {
                match v0conf {
                    Some(og_v0_conf) => match og_v0_conf.lock() {
                        Ok(og_v0_conf) => {
                            let mut new_v0_conf_inner = og_v0_conf.clone();
                            // reset the new cloned v0conf
                            new_v0_conf_inner.reset_listener_or_relay();

                            Version::V0(Some(Arc::new(Mutex::new(new_v0_conf_inner))))
                        }
                        Err(e) => {
                            return Err(anyhow::anyhow!("Failed to lock v0_conf: {}", e))?;
                        }
                    },
                    None => {
                        return Err(anyhow::anyhow!("v0_conf is None"))?;
                    }
                }
            }
            _ => {
                return Err(anyhow::anyhow!("This is not a V0 core"))?;
            }
        };

        // NOTE: Some of the followings can reuse the existing core, leave to later explore
        let wasm_config = wasmtime::Config::new();

        #[cfg(feature = "multithread")]
        {
            wasm_config.wasm_threads(true);
        }

        let engine = Engine::new(&wasm_config)?;
        let linker: Linker<Host> = Linker::new(&engine);

        let module = Module::from_file(&engine, &conf.filepath)?;

        let host = Host::default();
        let store = Store::new(&engine, host);

        Self::create_core(conf, linker, store, module, engine, Some(version))
    }

    pub fn _prepare(&mut self, conf: &WATERConfig) -> Result<(), anyhow::Error> {
        self._init(conf.debug)?;
        self._process_config(conf)?; // This is for now needed only by v1_preview
        Ok(())
    }

    /// This function is called when the host wants to call _init() in WASM
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

    /// This function is called when the host the WATM module to process the configurations,
    /// currently used by v1_preview, will change the behavior later to be
    /// a exported function from Host to WASM to let the WASM module to pull the config.
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
