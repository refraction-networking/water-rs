// =================== MODULES ===================
pub mod client;
pub mod core;
pub mod listener;
pub mod net;
pub mod relay;
pub mod runner;
pub mod stream;
pub mod transport;
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
    globals::{
        ACCEPT_FN, ASSOCIATE_FN, CANCEL_FN, CONFIG_FN, DIAL_FN, INIT_FN, READER_FN,
        WATER_BRIDGING_FN, WRITER_FN,
    },
};

// =================== MODULES' DEPENDENCIES ===================
use self::core::{Host, H2O};
use self::net::{ConnectFile, File, ListenFile};
use self::runner::WATERRunner;
use self::version::Version;
