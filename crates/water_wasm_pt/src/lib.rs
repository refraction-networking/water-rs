
// lib.rs
// export all modules
pub mod config;
pub mod dialer;
pub mod connections;
pub mod version;
pub mod encoder;
pub mod decoder;
// pub mod net;
// pub mod listener_in_wasm;

pub use config::*;
pub use dialer::*;
pub use connections::*;
pub use encoder::*;
pub use decoder::*;
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

use tracing::{info, debug};
use bincode::{self};

use anyhow::Result;
use serde::{Serialize, Deserialize};


// TODO: move these to speicific implementations, shouldn't be in the crate lib
// =================== WASM Imports =====================
extern "C" {
    // #[link_name = "create-listen"]
    pub fn create_listen(ptr: u32, size: u32) -> i32;
    pub fn connect_tcp(ptr: u32, size: u32) -> i32;
}