pub mod net;
pub mod funcs;
pub mod stream;

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

use crate::Config;
use crate::config::sharedconfig::WASMSharedConfig;
use crate::globals::READER_FN;
use crate::globals::WRITER_FN;

use net::{File, FileName, ListenFile, ConnectFile};
use funcs::{export_tcplistener_create, export_tcp_connect};

use crate::globals::{VERSION_FN, RUNTIME_VERSION_MAJOR, RUNTIME_VERSION, INIT_FN, USER_READ_FN, WRITE_DONE_FN, CONFIG_FN, WATER_BRIDGING_FN};

use stream::{WATERStream, Host};

pub struct WATERClient {
    debug: bool,
    
    pub config: Config,
    pub stream: WATERStream<Host>,
}

impl WATERClient {
    pub fn new(conf: Config) -> Result<Self, anyhow::Error> {
        let mut water = WATERStream::init(&conf)?;
        water._version()?;
        water._init()?;
        water._process_config(&conf)?;

        Ok(WATERClient {
            config: conf,
            debug: false,
            stream: water,
        })
    }

    pub fn set_debug(&mut self, debug: bool) {
        self.debug = debug;
    }

    pub fn connect(&mut self, addr: &str, port: u16) -> Result<(), anyhow::Error> {
        // NOTE: After creating the WATERStream, do some initial calls to WASM (e.g. version, init, etc.)
        self.stream.connect(&self.config, addr, port)?;
        Ok(())
    }
}