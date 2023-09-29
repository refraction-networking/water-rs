use crate::runtime::*;

#[derive(Default, Clone)]
pub struct Host {
    pub preview1_ctx: Option<wasmtime_wasi::WasiCtx>,
    pub wasi_threads: Option<Arc<WasiThreadsCtx<Host>>>,
}

//           UnixSocket          Connection created with Host
//    Write =>  u2w  +----------------+  w2n
//		       ----->|     Decode     |------>
//		Caller       |  WASM Runtime  |  n2w    Destination
//		       <-----| Decode/Encode  |<------
//    Read  =>  w2u  +----------------+
//	                    WATERStream

pub struct WATERStream<Host> {
    // WASM functions for reading & writing

    // the reader in WASM (read from net -- n2w)
    // returns the number of bytes read
    pub reader: Func, 
    
    // the writer in WASM (write to net -- w2n)
    // returns the number of bytes written
    pub writer: Func, 

    pub caller_reader: UnixStream, // the reader in Caller (read from WASM -- w2u)
    pub caller_writer: UnixStream, // the writer in Caller (write to WASM -- u2w)

    pub engine: Engine,
    pub linker: Linker<Host>,
    pub instance: Instance,
    pub store: Store<Host>,
    pub module: Module,
}

impl WATERStream<Host> {

    /// Read from the target address
    pub fn read(&mut self, buf: &mut Vec<u8>) -> Result<(), anyhow::Error> {
        info!("[HOST] WATERStream reading...");

        let mut res = vec![Val::I64(0); self.reader.ty(&self.store).results().len()];
        match self.reader.call(&mut self.store, &[], &mut res) {
            Ok(_) => {},
            Err(e) => return Err(anyhow::Error::msg(format!("{} function failed: {}", READER_FN, e))),
        }

        let nums: i64 = match res.get(0) {
            Some(wasmtime::Val::I64(v)) => {
                *v
            },
            _ => return Err(anyhow::Error::msg(format!("{} function returned unexpected type / no return", READER_FN))),
        };

        // read from WASM's caller_reader
        buf.resize(nums as usize, 0);
        match self.caller_reader.read(&mut buf[..]) {
            Ok(_) => {},
            Err(e) => return Err(anyhow::Error::msg(format!("failed to read from caller_reader: {}", e))),
        }

        Ok(())
    }

    /// Write to the target address
    pub fn write(&mut self, buf: &[u8]) -> Result<(), anyhow::Error> {
        info!("[HOST] WATERStream writing...");

        // write to WASM's caller_writer
        match self.caller_writer.write_all(buf) {
            Ok(_) => {},
            Err(e) => return Err(anyhow::Error::msg(format!("failed to write to caller_writer: {}", e))),
        }

        let params = vec![Val::I64(buf.len() as i64)];
        let mut res = vec![Val::I64(0)];
        match self.writer.call(&mut self.store, &params, &mut res) {
            Ok(_) => {
                match res.get(0) {
                    Some(wasmtime::Val::I64(v)) => {
                        if *v != buf.len() as i64 {
                            return Err(anyhow::Error::msg(format!("WASM write function returned unexpected value: {}", *v)));
                        }
                    },
                    _ => return Err(anyhow::Error::msg("user_write_done function returned unexpected type / no return")),
                };
            },
            Err(e) => return Err(anyhow::Error::msg(format!("{} function failed: {}", WRITER_FN, e))),
        }

        Ok(())
    }

    /// Connect to the target address with running the WASM entry function
    pub fn connect(&mut self, conf: &Config, addr: &str, port: u16) -> Result<(), anyhow::Error> {
        info!("[HOST] WATERStream connecting...");

        // TODO: add addr:port sharing with WASM, for now WASM is using config.json's remote_addr:port
        let fnc = self.instance.get_func(&mut self.store, &conf.entry_fn).unwrap();
        match fnc.call(&mut self.store, &[], &mut []) {
            Ok(_) => {},
            Err(e) => return Err(anyhow::Error::msg(format!("connect function failed: {}", e))),
        }
        

        Ok(())
    }
    
    pub fn init(conf: &Config) -> Result<Self, anyhow::Error> {
        info!("[HOST] WATERStream init...");
        
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

        // constructing 2 pairs of UnixStream for communicating between WASM and Host
        // returns (read_end, write_end) for caller
        let (caller_read_end, water_write_end) = UnixStream::pair()?;
        let (water_read_end, caller_write_end) = UnixStream::pair()?;

        let water_write_file = unsafe { cap_std::fs::File::from_raw_fd(water_write_end.as_raw_fd()) };
        let water_read_file = unsafe { cap_std::fs::File::from_raw_fd(water_read_end.as_raw_fd()) };
        
        // insert file here
        let wasi_water_reader = wasmtime_wasi::sync::file::File::from_cap_std(water_read_file);
        let wasi_water_writer = wasmtime_wasi::sync::file::File::from_cap_std(water_write_file);

        std::mem::forget(water_write_end);
        std::mem::forget(water_read_end);
        
        let ctx = store.data_mut().preview1_ctx.as_mut().unwrap();
        let water_reader_fd = ctx.push_file(Box::new(wasi_water_reader), FileAccessMode::all())?;
        let water_writer_fd = ctx.push_file(Box::new(wasi_water_writer), FileAccessMode::all())?;

        let water_bridging = match instance.get_func(&mut store, WATER_BRIDGING_FN) {
            Some(func) => func,
            None => return Err(anyhow::Error::msg(format!("{} function not found in WASM", WATER_BRIDGING_FN))),
        };

        let params = vec![Val::I32(water_reader_fd as i32), Val::I32(water_writer_fd as i32)];
        match water_bridging.call(&mut store, &params, &mut []) {
            Ok(_) => {},
            Err(e) => return Err(anyhow::Error::msg(format!("{} function failed: {}", WATER_BRIDGING_FN, e))),
        }

        // getting reader & writer func from WASM
        let reader = match instance.get_func(&mut store, READER_FN) {
            Some(func) => func,
            None => return Err(anyhow::Error::msg(format!("{} function not found in WASM", READER_FN))),
        };

        let writer = match instance.get_func(&mut store, WRITER_FN) {
            Some(func) => func,
            None => return Err(anyhow::Error::msg(format!("{} function not found in WASM", WRITER_FN))),
        };

        let runtime = WATERStream {
            reader: reader,
            writer: writer,

            caller_reader: caller_read_end,
            caller_writer: caller_write_end,

            engine: engine,
            linker: linker,
            instance: instance,
            store: store,
            module: module,
        };

        Ok(runtime)
    }

    pub fn _version(&mut self) -> Result<(), anyhow::Error> {
        info!("[HOST] WATERStream calling _version from WASM...");

        let version_fn = match self.instance.get_func(&mut self.store, VERSION_FN) {
            Some(func) => func,
            None => return Err(anyhow::Error::msg("version function not found")),
        };

        // let mut res = vec![Val::I32(0); version_fn.ty(&self.store).results().len()];

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

    pub fn _init(&mut self) -> Result<(), anyhow::Error> {
        info!("[HOST] WATERStream calling _init from WASM...");

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

    pub fn _process_config(&mut self, config: &Config) -> Result<(), anyhow::Error> {
        info!("[HOST] WATERStream calling _process_config from WASM...");

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