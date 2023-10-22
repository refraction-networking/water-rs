// =================== MODULES ===================
pub mod core;
pub mod listener;
pub mod net;
pub mod runner;
pub mod stream;
pub mod v0;
pub mod v1;
pub mod version;
pub mod version_common;

// =================== STD Imports ===================
use std::{
    io::{Read, Write},
    os::unix::io::{AsRawFd, FromRawFd},
    path::Path,
    sync::Arc,
};

// =================== EXTERNAL CRATES ===================
use anyhow::{Context, Result};
use cap_std::{
    ambient_authority,
    fs::OpenOptions,
    net::{TcpListener, TcpStream},
    os::unix::net::UnixStream,
};
use tracing::{debug, info};
use wasi_common::{file::FileAccessMode, WasiCtx, WasiFile};
use wasmtime::*;
use wasmtime_wasi::sync::{Dir, WasiCtxBuilder};
use wasmtime_wasi_threads::WasiThreadsCtx;

// =================== CURRENT CRATE IMPORTS ===================
use crate::{
    config::{WATERConfig, WaterBinType},
    globals::{CONFIG_FN, DIAL_FN, INIT_FN, READER_FN, WATER_BRIDGING_FN, WRITER_FN},
};

// =================== MODULES' DEPENDENCIES ===================
use self::core::{Host, H2O};
use self::listener::WATERListener;
use self::net::{ConnectFile, File, ListenFile};
use self::runner::WATERRunner;
use self::stream::WATERStream;
use self::version::Version;

// =================== WATERClient Definition ===================
pub enum WATERClientType {
    Dialer(WATERStream<Host>),
    Listener(WATERListener<Host>),
    Runner(WATERRunner<Host>), // This is a customized runner -- not like any stream
}

pub struct WATERClient {
    debug: bool,

    pub config: WATERConfig,
    pub stream: WATERClientType,
}

impl WATERClient {
    pub fn new(conf: WATERConfig) -> Result<Self, anyhow::Error> {
        // client_type: 0 -> Dialer, 1 -> Listener, 2 -> Runner
        let water = match conf.client_type {
            WaterBinType::Dial => {
                let stream = WATERStream::init(&conf)?;
                WATERClientType::Dialer(stream)
            }
            WaterBinType::Listen => {
                let stream = WATERListener::init(&conf)?;
                WATERClientType::Listener(stream)
            }
            WaterBinType::Runner => {
                let runner = WATERRunner::init(&conf)?;
                WATERClientType::Runner(runner)
            }
            _ => {
                return Err(anyhow::anyhow!("Invalid client type"));
            }
        };

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
            }
            _ => {
                return Err(anyhow::anyhow!("This client is not a Runner"));
            }
        }
        Ok(())
    }

    pub fn connect(&mut self, addr: &str, port: u16) -> Result<(), anyhow::Error> {
        info!("[HOST] WATERClient connecting ...");

        match &mut self.stream {
            WATERClientType::Dialer(dialer) => {
                dialer.connect(&self.config, addr, port)?;
            }
            _ => {
                return Err(anyhow::anyhow!("This client is not a listener"));
            }
        }
        Ok(())
    }

    pub fn listen(&mut self, addr: &str, port: u16) -> Result<(), anyhow::Error> {
        info!("[HOST] WATERClient listening ...");

        match &mut self.stream {
            WATERClientType::Listener(listener) => {
                listener.listen(&self.config, addr, port)?;
            }
            _ => {
                return Err(anyhow::anyhow!("This client is not a listener"));
            }
        }
        Ok(())
    }

    pub fn read(&mut self, buf: &mut Vec<u8>) -> Result<i64, anyhow::Error> {
        let read_bytes = match self.stream {
            WATERClientType::Dialer(ref mut dialer) => dialer.read(buf)?,
            WATERClientType::Listener(ref mut listener) => listener.read(buf)?,
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
            }
            WATERClientType::Listener(ref mut listener) => {
                listener.write(buf)?;
            }
            _ => {
                return Err(anyhow::anyhow!("This client is not supporting write"));
            }
        }
        Ok(())
    }
}
