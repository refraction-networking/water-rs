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

use crate::globals::{VERSION_FN, RUNTIME_VERSION_MAJOR, RUNTIME_VERSION, INIT_FN};

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

    pub fn connect_context(&mut self) -> Result<WATERStream<Host>, anyhow::Error> {
        let mut runtime = WATERStream::init(&self.config)?;

        // NOTE: After creating the WATERStream, do some initial calls to WASM (e.g. version, init, etc.)
        runtime._version()?;
        runtime._init()?;

        // runtime.connect(conf)?;
        Ok(runtime)
    }
}


#[derive(Default, Clone)]
pub struct Host {
    preview1_ctx: Option<wasmtime_wasi::WasiCtx>,
    wasi_threads: Option<Arc<WasiThreadsCtx<Host>>>,
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

        fnc.call(&mut self.store, &[], &mut []);

        Ok(())
    }

    // pub fn linkDialFuns() {

    // }

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
        
        // export functions
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
        
        let mut res = &mut [ Val::I32(0) ];
        version_fn.call(&mut self.store, &[], res)?;
        
        match res[0] {
            wasmtime::Val::I32(v) => { 
                if v != RUNTIME_VERSION_MAJOR { 
                    return Err(anyhow::Error::msg(format!("WASI module version {} is not compatible with runtime version {}", v, RUNTIME_VERSION))); 
                }
            },
            _ => return Err(anyhow::Error::msg("version function returned unexpected type")),
        };

        Ok(())
    }

    pub fn _init(&mut self) -> Result<(), anyhow::Error> {
        let version_fn = match self.instance.get_func(&mut self.store, INIT_FN) {
            Some(func) => func,
            None => return Err(anyhow::Error::msg("init function not found")),
        };

        // TODO: check if we need to pass in any arguments / configs later
        version_fn.call(&mut self.store, &[], &mut [])?;

        Ok(())
    }
}
