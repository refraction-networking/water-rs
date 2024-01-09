//! This module defines the `WASMSharedConfig` struct, which is used to pass infos between the host and the WATM module.
//!
//! This is a temporary workaround for the constraint that the WATM module can only take in parameters with primitive types.
//!
//! We are designing a better way to do this, which can be uniform across all Host languages (Go).
//!
//! Currently only used for v1 implementations, where v1 grant the ability for the WATM module to dial and listen on specific ip + ports.

use serde::{Deserialize, Serialize};
use std::mem;

/// This struct is used to pass infos between the host and the WATM module. Only addr, port, and name are used for creating the connection for now.
#[derive(Serialize, Deserialize)]
#[repr(C)]
pub struct StreamConfig {
    /// ip address
    pub addr: String,

    /// port
    pub port: u32,

    /// a name for the stream
    pub name: String,
}

impl StreamConfig {
    /// Convert the struct to a byte array -- the way of sharing data between the host and the WATM module
    /// is using memory sharing where the WATM module will pass in a pointer to the memory location of the byte array to the Host helper function.
    ///
    /// Then the Host helper function will offset the pointer into WASM's memory addresses to retreive the byte array
    /// and convert it back to the struct.
    pub fn to_bytes(&self) -> Vec<u8> {
        let size = mem::size_of::<StreamConfig>();
        let ptr = self as *const Self;

        let bytes_slice = unsafe { std::slice::from_raw_parts(ptr as *const u8, size) };
        bytes_slice.to_vec()
    }
}
