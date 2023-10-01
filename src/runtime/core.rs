use crate::runtime::*;

#[derive(Default, Clone)]
pub struct Host {
    pub preview1_ctx: Option<wasmtime_wasi::WasiCtx>,
    pub wasi_threads: Option<Arc<WasiThreadsCtx<Host>>>,
}

pub struct H2O<Host> {
    pub engine: Engine,
    pub linker: Linker<Host>,
    pub instance: Instance,
    pub store: Store<Host>,
    pub module: Module,
}

impl H2O<Host> {
    pub fn init(conf: &Config) -> Result<Self, anyhow::Error> {
        info!("[HOST] WATERCore H2O initing...");
        
        let mut wasm_config = wasmtime::Config::new();
        wasm_config.wasm_threads(true);
        
        let engine = Engine::new(&wasm_config)?;
        let mut linker: Linker<Host> = Linker::new(&engine);
        
        let module = Module::from_file(&engine, &conf.filepath)?;
        
        let host = Host::default();
        let mut store = Store::new(&engine, host);

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
        
        // export functions -- link connect functions
        export_tcp_connect(&mut linker);
        export_tcplistener_create(&mut linker);
        
        let instance = linker.instantiate(&mut store, &module)?;

        Ok(H2O {
            engine: engine,
            linker: linker,
            instance: instance,
            store: store,
            module: module,
        })  
    }

    pub fn _prepare(&mut self, conf: &Config) -> Result<(), anyhow::Error> {
        self._version()?;
        self._init()?;
        self._process_config(&conf)?;
        Ok(())
    }

    pub fn _init(&mut self) -> Result<(), anyhow::Error> {
        info!("[HOST] WATERCore H2O calling _init from WASM...");

        let init_fn = match self.instance.get_func(&mut self.store, INIT_FN) {
            Some(func) => func,
            None => return Err(anyhow::Error::msg("init function not found")),
        };

        // TODO: check if we need to pass in any arguments / configs later
        match init_fn.call(&mut self.store, &[], &mut []) {
            Ok(_) => {},
            Err(e) => return Err(anyhow::Error::msg(format!("init function failed: {}", e))),
        }

        Ok(())
    }

    pub fn _version(&mut self) -> Result<(), anyhow::Error> {
        info!("[HOST] WATERCore H2O calling _version from WASM...");

        let version_fn = match self.instance.get_func(&mut self.store, VERSION_FN) {
            Some(func) => func,
            None => return Err(anyhow::Error::msg("version function not found")),
        };

        // let mut res = vec![Val::I32(0); version_fn.ty(&self.core.store).results().len()];

        // NOTE: below is the most generic way to setup the res vector, to avoid panic from calling
        let mut res: Vec<Val> = version_fn
            .ty(&self.store)
            .results()
            .map(|ty| // map each type to a default value
                match ty {
                    // i32 and i64 are the only integer types in WebAssembly as of 2021
                    ValType::I32 => Val::I32(0),
                    ValType::I64 => Val::I64(0),
                    ValType::F32 => Val::F32(0),
                    ValType::F64 => Val::F64(0),
                    _            => panic!("Unsupported type"),
                }
            )
            .collect(); // collect the default values into a Vec
        
        // TODO: add error handling code like this for all other func.call()'s
        match version_fn.call(&mut self.store, &[], &mut res) {
            Ok(_) => {},
            Err(e) => return Err(anyhow::Error::msg(format!("version function failed: {}", e))),
        }
        
        match res.get(0) {
            Some(Val::I32(v)) => {
                if *v != RUNTIME_VERSION_MAJOR {
                    return Err(anyhow::Error::msg(format!("WASI module version {} is not compatible with runtime version {}", v, RUNTIME_VERSION)));
                }
            },
            Some(_) => return Err(anyhow::Error::msg("version function returned unexpected type")),
            None => return Err(anyhow::Error::msg("version function has no return")),
        };

        Ok(())
    }

    pub fn _process_config(&mut self, config: &Config) -> Result<(), anyhow::Error> {
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
        
        // TODO: might be better to ask WASM for the fd? Or if it is fixed in the pipeline, then 3 is fine
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