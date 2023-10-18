// =================== Imports & Modules =====================
use std::{
    io::{self, Read, Write},
    os::fd::IntoRawFd,
    // os::wasi::prelude::FromRawFd,
    os::fd::FromRawFd,
    sync::Mutex,
    vec,
};

use tokio::{
    io::{AsyncRead, AsyncWrite, AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    time,
    time::timeout,
};

use lazy_static::lazy_static;
use serde_json;
use tracing::{info, Level, debug};
use tracing_subscriber;
use bincode::{self};

use std::fs::File;
use std::mem;
use anyhow::{Context, Result};
use serde::{Serialize, Deserialize, de::DeserializeOwned};
use tokio_util::codec::{AnyDelimiterCodec, Framed, FramedParts};
use std::time::Duration;

use water_wasm_crate::*;

pub mod async_socks5_listener;
use async_socks5_listener::*;

// create a mutable global variable stores a pointer to the config
lazy_static! {
    static ref DIALER: Mutex<Dialer> = Mutex::new(Dialer::new());
    // static ref CONN: Mutex<Connection> = Mutex::new(Connection::new());
}

#[export_name = "_init"]
pub fn _init(debug: bool) {
    if debug {
        tracing_subscriber::fmt()
            .with_max_level(Level::INFO)
            .init();
    }
    
    info!("[WASM] running in _init");
}

#[export_name = "_set_inbound"]
pub fn _water_bridging(fd: i32) {
    let mut global_dialer = match DIALER.lock() {
        Ok(dialer) => dialer,
        Err(e) => {
            eprintln!("[WASM] > ERROR: {}", e);
            return;
        }
    };

    global_dialer.file_conn.set_inbound(fd, ConnStream::File(unsafe { std::fs::File::from_raw_fd(fd) }));
}

#[export_name = "_set_outbound"]
pub fn _water_bridging_out(fd: i32) {
    let mut global_dialer = match DIALER.lock() {
        Ok(dialer) => dialer,
        Err(e) => {
            eprintln!("[WASM] > ERROR: {}", e);
            return;
        }
    };

    global_dialer.file_conn.set_outbound(fd, ConnStream::TcpStream(unsafe { std::net::TcpStream::from_raw_fd(fd) }));
}

#[export_name = "_config"]
pub fn _process_config(fd: i32) {
    info!("[WASM] running in _process_config");

    let mut config_file = unsafe { std::fs::File::from_raw_fd(fd) };
    let mut config = String::new();
    match config_file.read_to_string(&mut config) {
        Ok(_) => {
            let config: Config = match serde_json::from_str(&config) {
                Ok(config) => config,
                Err(e) => {
                    eprintln!("[WASM] > _process_config ERROR: {}", e);
                    return;
                }
            };

            let mut global_dialer = match DIALER.lock() {
                Ok(dialer) => dialer,
                Err(e) => {
                    eprintln!("[WASM] > ERROR: {}", e);
                    return;
                }
            };
        
            // global_dialer.file_conn.config = config.clone();
            global_dialer.config = config;
        },
        Err(e) => {
            eprintln!("[WASM] > WASM _process_config falied reading path ERROR: {}", e);
            return;
        }
    };
}

#[export_name = "_write"]
pub fn _write(bytes_write: i64) -> i64 {
    let mut global_dialer = match DIALER.lock() {
        Ok(dialer) => dialer,
        Err(e) => {
            eprintln!("[WASM] > ERROR: {}", e);
            return -1;
        }
    };

    match global_dialer.file_conn._write_2_outbound(&mut DefaultEncoder, bytes_write) {
        Ok(n) => n,
        Err(e) => {
            eprintln!("[WASM] > ERROR in _write: {}", e);
            return -1;
        }
    }
}

#[export_name = "_read"]
pub fn _read() -> i64 {
    let mut global_dialer = match DIALER.lock() {
        Ok(dialer) => dialer,
        Err(e) => {
            eprintln!("[WASM] > ERROR: {}", e);
            return -1;
        }
    };

    match global_dialer.file_conn._read_from_outbound(&mut DefaultDecoder) {
        Ok(n) => n,
        Err(e) => {
            eprintln!("[WASM] > ERROR in _read: {}", e);
            return -1;
        }
    }
}

#[export_name = "_dial"]
pub fn _dial() {
    let mut global_dialer = match DIALER.lock() {
        Ok(dialer) => dialer,
        Err(e) => {
            eprintln!("[WASM] > ERROR: {}", e);
            return;
        }
    };

    match global_dialer.dial() {
        Ok(_) => {},
        Err(e) => {
            eprintln!("[WASM] > ERROR in _dial: {}", e);
            return;
        }
    }
}