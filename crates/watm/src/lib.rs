//! This lib is for a demo and ease of developing the WATM module

pub mod config;
pub mod connections;
pub mod decoder;
pub mod dialer;
pub mod encoder;
pub mod version;
// pub mod net;
// pub mod listener_in_wasm;

pub use config::*;
pub use connections::*;
pub use decoder::*;
pub use dialer::*;
pub use encoder::*;
// pub use net::*;
// pub use listener_in_wasm::*;

// ======= v1 module for a better SS demo =======
// pub mod v1;
// use v1::async_listener_v1::*;

// =================== Imports & Modules =====================
use std::{
    io::{Read, Write},
    os::fd::FromRawFd,
    vec,
};

use bincode::{self};
use tracing::{debug, info};

use anyhow::Result;
use serde::{Deserialize, Serialize};

// =================== WASM Imports =====================
extern "C" {
    /// These functions are exported from the host to the WATM module,
    /// which means host must provide these functions to the WATM module.
    ///
    /// create a listener (specified by returned fd) -- pass ptr + size for the ip:port struct sharing to Host
    // #[link_name = "create_listen"]
    pub fn create_listen(ptr: u32, size: u32) -> i32;

    /// create a TcpStream connection (specified by returned fd) -- pass ptr + size for the ip:port struct sharing to Host
    pub fn connect_tcp(ptr: u32, size: u32) -> i32;
}
