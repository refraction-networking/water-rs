//! lib.rs
//! export all modules
//!

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

// TODO: move these to speicific implementations, shouldn't be in the crate lib

// =================== WASM Imports =====================
extern "C" {
    /// These functions are imported from the host to the WASM module.
    /// host must provide these functions to the WASM module.
    // #[link_name = "create-listen"]
    pub fn create_listen(ptr: u32, size: u32) -> i32;
    pub fn connect_tcp(ptr: u32, size: u32) -> i32;
}
