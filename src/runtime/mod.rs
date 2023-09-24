pub mod net;
pub mod funcs;

use std::sync::Arc;
use anyhow::Result;

use wasmtime::*;
use wasi_common::WasiCtx;
use wasi_common::WasiFile;
use wasi_common::file::FileAccessMode;

use wasmtime_wasi::sync::WasiCtxBuilder;
use wasi_common::pipe::{ReadPipe, WritePipe};

use wasmtime_wasi_threads::WasiThreadsCtx;

use cap_std::net::{TcpListener, TcpStream};

use crate::Config;
use net::{File, FileName, ListenFile, ConnectFile};
use funcs::{export_tcplistener_create};

use crate::globals::{VERSION_FN, RUNTIME_VERSION_MAJOR, RUNTIME_VERSION, INIT_FN, USER_READ_FN, WRITE_DONE_FN};

pub struct WATERStreamConnector {
    pub config: Config,
    debug: bool,
}

impl WATERStreamConnector {
    pub fn new(conf: Config) -> Result<Self, anyhow::Error> {
        Ok(WATERStreamConnector {
            config: conf,
            debug: false,
        })
    }

    pub fn set_debug(&mut self, debug: bool) {
        self.debug = debug;
    }

    // TODO: add a parameter for target address
    pub fn connect(&mut self) -> Result<WATERStream<Host>, anyhow::Error> {
        let mut runtime = WATERStream::init(&self.config)?;

        // NOTE: After creating the WATERStream, do some initial calls to WASM (e.g. version, init, etc.)
        runtime._version()?;
        runtime._init()?;

        runtime.connect(&self.config)?;
        Ok(runtime)
    }
}

#[derive(Default, Clone)]
pub struct Host {
    preview1_ctx: Option<wasmtime_wasi::WasiCtx>,
    wasi_threads: Option<Arc<WasiThreadsCtx<Host>>>,
}

pub trait WATERStreamOps {
    fn user_write_done(&mut self, n: i32) -> Result<i32, anyhow::Error>;
    fn user_will_read(&mut self) -> Result<i32, anyhow::Error>;
}

pub struct WATERStream<Host> {
    pub engine: Engine,
    pub linker: Linker<Host>,
    pub instance: Instance,
    pub store: Store<Host>,
    pub module: Module,
}

impl WATERStream<Host> {
    pub fn connect(&mut self, conf: &Config) -> Result<(), anyhow::Error>  {
        let fnc = self.instance.get_func(&mut self.store, &conf.entry_fn).unwrap();
        match fnc.call(&mut self.store, &[], &mut []) {
            Ok(_) => {},
            Err(e) => return Err(anyhow::Error::msg(format!("connect function failed: {}", e))),
        }
        
        Ok(())
    }
    
    pub fn init(conf: &Config) -> Result<Self, anyhow::Error> {
        
        let mut wasm_config = wasmtime::Config::new();
        wasm_config.wasm_threads(true);
        
        let engine = Engine::new(&wasm_config)?;
        let mut linker: Linker<Host> = Linker::new(&engine);
        
        let module = Module::from_file(&engine, &conf.filepath)?;
        
        let host = Host::default();
        let mut store = Store::new(&engine, host);
        
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
        export_tcplistener_create(&mut linker);
        
        let instance = linker.instantiate(&mut store, &module)?;

        let runtime = WATERStream { 
            engine: engine,
            linker: linker,
            instance: instance,
            store: store,
            module: module,
        };

        Ok(runtime)
    }

    pub fn _version(&mut self) -> Result<(), anyhow::Error> {
        let version_fn = match self.instance.get_func(&mut self.store, VERSION_FN) {
            Some(func) => func,
            None => return Err(anyhow::Error::msg("version function not found")),
        };

        let mut res = vec![Val::I32(0); version_fn.ty(&self.store).results().len()];
        
        // TODO: add error handling code like this for all other func.call()'s
        match version_fn.call(&mut self.store, &[], &mut res) {
            Ok(_) => {},
            Err(e) => return Err(anyhow::Error::msg(format!("version function failed: {}", e))),
        }
        
        match res.get(0) {
            Some(wasmtime::Val::I32(v)) => {
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
        let init_fn = match self.instance.get_func(&mut self.store, INIT_FN) {
            Some(func) => func,
            None => return Err(anyhow::Error::msg("init function not found")),
        };

        // TODO: check if we need to pass in any arguments / configs later
        init_fn.call(&mut self.store, &[], &mut [])?;

        Ok(())
    }
}

impl WATERStreamOps for WATERStream<Host> {
    fn user_write_done(&mut self, n: i32) -> Result<i32, anyhow::Error> {
        let user_write_done_fn = match self.instance.get_func(&mut self.store, WRITE_DONE_FN) {
            Some(func) => func,
            None => return Err(anyhow::Error::msg("user_write_done function not found")),
        };

        let mut res = vec![Val::I32(0); user_write_done_fn.ty(&self.store).results().len()];
        match user_write_done_fn.call(&mut self.store, &[Val::I32(n)], &mut res) {
            Ok(_) => {},
            Err(e) => return Err(anyhow::Error::msg(format!("user_write_done function failed: {}", e))),
        }

        match res.get(0) {
            Some(wasmtime::Val::I32(v)) => {
                return Ok(*v);
            },
            _ => return Err(anyhow::Error::msg("user_write_done function returned unexpected type / no return")),
        };
    }

    fn user_will_read(&mut self) -> Result<i32, anyhow::Error> {
        let user_will_read_fn = match self.instance.get_func(&mut self.store, USER_READ_FN) {
            Some(func) => func,
            None => return Err(anyhow::Error::msg("user_will_read function not found")),
        };

        let mut res = vec![Val::I32(0); user_will_read_fn.ty(&self.store).results().len()];
        match user_will_read_fn.call(&mut self.store, &[], &mut res) {
            Ok(_) => {},
            Err(e) => return Err(anyhow::Error::msg(format!("user_will_read function failed: {}", e))),
        }

        match res.get(0) {
            Some(wasmtime::Val::I32(v)) => {
                return Ok(*v);
            },
            _ => return Err(anyhow::Error::msg("user_will_read function returned unexpected type / no return")),
        };
    }
}