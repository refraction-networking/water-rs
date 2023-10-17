use crate::runtime::*;

#[derive(Default, Clone)]
pub struct Host {
    pub preview1_ctx: Option<wasmtime_wasi::WasiCtx>,
    pub wasi_threads: Option<Arc<WasiThreadsCtx<Host>>>,
}

pub struct H2O<Host> {
    pub version: Version,

    pub engine: Engine,
    pub linker: Linker<Host>,
    pub instance: Instance,
    pub store: Store<Host>,
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

        let version = module.exports().find_map(|global| {
            info!("[HOST] WATERCore finding exported symbols from WASM bin: {:?}", global.name());
            match Version::from_str(global.name()) {
                Some(v) => {
                    info!("[HOST] WATERCore found version: {:?}", v.as_str());
                    Some(v)
                },
                None => None,
            }
        });

        if version.is_none() {
            return Err(anyhow::Error::msg("WASM module version not found"));
        }
        
        // let path = unsafe { Dir::open_ambient_dir(".", ambient_authority())? };
        
        // store.data_mut().preview1_ctx = Some(WasiCtxBuilder::new().inherit_stdio().preopened_dir(path, ".")?.build());
        store.data_mut().preview1_ctx = Some(WasiCtxBuilder::new().inherit_stdio().build());
        
        wasmtime_wasi::add_to_linker(&mut linker, |h: &mut Host| {
            h.preview1_ctx.as_mut().unwrap()
        })?;
        
        // initializing stuff for multithreading
        #[cfg(feature = "multithread")]
        {
    
            store.data_mut().wasi_threads = Some(Arc::new(WasiThreadsCtx::new(
                module.clone(),
                Arc::new(linker.clone()),
            )?));
            
            wasmtime_wasi_threads::add_to_linker(&mut linker, &store, &module, |h: &mut Host| {
                h.wasi_threads.as_ref().unwrap()
            })?;
        }

        
        // export functions -- version dependent -- has to be done before instantiate
        match &version {
            Some(Version::V0) => {
                v0::funcs::export_tcp_connect(&mut linker);
                v0::funcs::export_tcplistener_create(&mut linker);
            },
            Some(Version::V1) => {
                v1::funcs::export_tcp_connect(&mut linker);
                v1::funcs::export_tcplistener_create(&mut linker);
            },
            _ => {} // add export funcs for other versions here
        }

        // export functions -- version independent
        version_common::funcs::export_config(&mut linker, conf.config_wasm.clone());
        
        let instance = linker.instantiate(&mut store, &module)?;

        Ok(H2O {
            version: version.unwrap(),

            engine: engine,
            linker: linker,
            instance: instance,
            store: store,
            module: module,
        })  
    }

    pub fn _prepare(&mut self, conf: &WATERConfig) -> Result<(), anyhow::Error> {
        // NOTE: version has been checked at the very beginning
        self._init(conf.debug)?;
        self._process_config(&conf)?;
        Ok(())
    }

    pub fn _init(&mut self, debug: bool) -> Result<(), anyhow::Error> {
        info!("[HOST] WATERCore H2O calling _init from WASM...");

        let init_fn = match self.instance.get_func(&mut self.store, INIT_FN) {
            Some(func) => func,
            None => return Err(anyhow::Error::msg("init function not found")),
        };

        // TODO: check if we need to pass in any arguments / configs later
        let params = vec![Val::I32(debug as i32); init_fn.ty(&self.store).params().len()];
        match init_fn.call(&mut self.store, &params, &mut []) {
            Ok(_) => {},
            Err(e) => return Err(anyhow::Error::msg(format!("init function failed: {}", e))),
        }

        Ok(())
    }

    pub fn _process_config(&mut self, config: &WATERConfig) -> Result<(), anyhow::Error> {
        info!("[HOST] WATERCore H2O calling _process_config from WASM...");

        // _required to implement _process_config(i32) in WASM, which will be parsing all the configurations
        let config_fn = match self.instance.get_func(&mut self.store, CONFIG_FN) {
            Some(func) => func,
            None => return Err(anyhow::Error::msg("_process_config function not found in WASM")),
        };

        // open the config file and insert to WASM
        let dir = Dir::open_ambient_dir(".", ambient_authority())?; // Open the root directory
        let wasi_file = dir.open_with(&config.config_wasm, OpenOptions::new().read(true).write(true))?;
        let wasi_file = wasmtime_wasi::sync::file::File::from_cap_std(wasi_file);
        
        let ctx = self.store.data_mut().preview1_ctx.as_mut().unwrap();
        let config_fd = ctx.push_file(Box::new(wasi_file), FileAccessMode::all())? as i32;

        let params = vec![Val::I32(config_fd); config_fn.ty(&self.store).params().len()];
        match config_fn.call(&mut self.store, &params, &mut []) {
            Ok(_) => {},
            Err(e) => return Err(anyhow::Error::msg(format!("_process_config function in WASM failed: {}", e))),
        }

        Ok(())
    }
}