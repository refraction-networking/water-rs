pub mod net;
pub mod funcs;
pub mod stream;
pub mod core;
pub mod listener;
pub mod runner;

use std::sync::Arc;
use anyhow::{Context, Result};

use cap_std::os::unix::net::UnixStream;
use tracing_subscriber::fmt::format;
use wasmtime::*;
use wasi_common::WasiCtx;
use wasi_common::WasiFile;
use wasi_common::file::FileAccessMode;

use wasmtime_wasi::sync::WasiCtxBuilder;
use wasi_common::pipe::{ReadPipe, WritePipe};

use wasmtime_wasi_threads::WasiThreadsCtx;

use wasmtime_wasi::sync::Dir;
use cap_std::ambient_authority;
use cap_std::fs::OpenOptions;
use std::os::unix::io::{AsRawFd, FromRawFd};
// use system_interface::io::io_ext::IoExt;
use std::io::{Read, Write};

use tracing::{info, trace};

use cap_std::net::{TcpListener, TcpStream};

use crate::config::Config;
use crate::config::wasm_shared_config::WASMSharedConfig;
use crate::globals::READER_FN;
use crate::globals::WRITER_FN;

use net::{File, FileName, ListenFile, ConnectFile};
use funcs::{export_tcplistener_create, export_tcp_connect};

use crate::globals::{VERSION_FN, RUNTIME_VERSION_MAJOR, RUNTIME_VERSION, INIT_FN, USER_READ_FN, WRITE_DONE_FN, CONFIG_FN, WATER_BRIDGING_FN};

use stream::{WATERStream};
use listener::{WATERListener};
use runner::{WATERRunner};
// use core::WATERCore;
use core::{H2O, Host};


pub enum WATERClientType {
    Dialer(WATERStream<Host>),
    Listener(WATERListener<Host>),
    Runner(WATERRunner<Host>), // This is a customized runner -- not like any stream
}

pub struct WATERClient {
    debug: bool,
    
    pub config: Config,
    pub stream: WATERClientType,
}

impl WATERClient {
    pub fn new(conf: Config) -> Result<Self, anyhow::Error> {
        // client_type: 0 -> Dialer, 1 -> Listener, 2 -> Runner
        let mut water: WATERClientType;
        if conf.client_type == 0 {
            let mut stream = WATERStream::init(&conf)?;
            water = WATERClientType::Dialer(stream);
        } else if conf.client_type == 1 {
            let mut stream = WATERListener::init(&conf)?;
            water = WATERClientType::Listener(stream);
        } else if conf.client_type == 2 {
            let mut runner = WATERRunner::init(&conf)?;
            water = WATERClientType::Runner(runner);
        } else {
            return Err(anyhow::anyhow!("Invalid client type"));
        }

        Ok(WATERClient {
            config: conf,
            debug: false,
            stream: water,
        })
    }

    pub fn set_debug(&mut self, debug: bool) {
        self.debug = debug;
    }

    pub fn execute(&mut self) -> Result<(), anyhow::Error> {
        info!("[HOST] WATERClient Executing ...");
        
        match &mut self.stream {
            WATERClientType::Runner(runner) => {
                runner.run(&self.config)?;
            },
            _ => {
                return Err(anyhow::anyhow!("This client is not a Runner"));
            }
        }
        Ok(())
    }
    
    pub fn connect(&mut self, addr: &str, port: u16) -> Result<(), anyhow::Error> {
        info!("[HOST] WATERClient connecting ...");
        // NOTE: After creating the WATERStream, do some initial calls to WASM (e.g. version, init, etc.)
        match &mut self.stream {
            WATERClientType::Dialer(dialer) => {
                dialer.connect(&self.config, addr, port)?;
            },
            _ => {
                return Err(anyhow::anyhow!("This client is not a listener"));
            }
        }
        Ok(())
    }
    
    pub fn listen(&mut self, addr: &str, port: u16) -> Result<(), anyhow::Error> {
        info!("[HOST] WATERClient listening ...");
        // NOTE: After creating the WATERStream, do some initial calls to WASM (e.g. version, init, etc.)
        match &mut self.stream {
            WATERClientType::Listener(listener) => {
                listener.listen(&self.config, addr, port)?;
            },
            _ => {
                return Err(anyhow::anyhow!("This client is not a listener"));
            }
        }
        Ok(())
    }
    
    pub fn read(&mut self, buf: &mut Vec<u8>) -> Result<i64, anyhow::Error> {
        let read_bytes = match self.stream {
            WATERClientType::Dialer(ref mut dialer) => {
                dialer.read(buf)?
            },
            WATERClientType::Listener(ref mut listener) => {
                listener.read(buf)?
            },
            _ => {
                return Err(anyhow::anyhow!("This client is not supporting read"));
            }
        };
        
        Ok(read_bytes)
    }
    
    pub fn write(&mut self, buf: &[u8]) -> Result<(), anyhow::Error> {
        match self.stream {
            WATERClientType::Dialer(ref mut dialer) => {
                dialer.write(buf)?;
            },
            WATERClientType::Listener(ref mut listener) => {
                listener.write(buf)?;
            },
            _ => {
                return Err(anyhow::anyhow!("This client is not supporting write"));
            }
        }
        Ok(())
    }
}