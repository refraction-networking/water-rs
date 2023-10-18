use serde::{Deserialize, Serialize};
use std::mem;

#[repr(C)]
pub struct WASMSharedConfig {
    //     pub key: u64, // a pointer to a key string's byte-view
    //     pub size: u64, // size for the key
}

impl WASMSharedConfig {
    pub fn to_bytes(&self) -> Vec<u8> {
        let size = mem::size_of::<WASMSharedConfig>();
        let ptr = self as *const Self;

        let bytes_slice = unsafe { std::slice::from_raw_parts(ptr as *const u8, size) };
        let bytes = bytes_slice.to_vec();
        bytes
    }

    // pub fn from_bytes(bytes: Vec<u8>) -> Option<Self> {
    //     let size = mem::size_of::<Data>();
    //     if bytes.len() != size {
    //         return None;
    //     }
    //     let mut struct_bytes = bytes;
    //     let ptr = struct_bytes.as_mut_ptr() as *mut Self;
    //     let struct_ref = unsafe { Box::from_raw(ptr) };
    //     Some(*struct_ref)
    // }
}

#[derive(Serialize, Deserialize)]
#[repr(C)]
pub struct StreamConfig {
    pub addr: String,
    pub port: u32,
    pub name: String,
}

impl StreamConfig {
    pub fn to_bytes(&self) -> Vec<u8> {
        let size = mem::size_of::<StreamConfig>();
        let ptr = self as *const Self;

        let bytes_slice = unsafe { std::slice::from_raw_parts(ptr as *const u8, size) };
        let bytes = bytes_slice.to_vec();
        bytes
    }
}
