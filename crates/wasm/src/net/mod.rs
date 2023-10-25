//! Networking primitives for TCP/UDP communication.
//!
//! This module provides networking functionality for TCP/UDP communication supported by the
//! external wasm host. We re-use a bunch of code from the rust std library, allowing us to
//! effectively provide a matching interface around the watr C API.

mod std_sys_net;

use crate::config::StreamConfigV1;
use std_sys_net::*;

use std::{
    io,
    net::{SocketAddr, ToSocketAddrs},
    os::fd::FromRawFd,
};

/// Network based functions guaranteed to be available from the WASM host. These functions are part
/// of the WATER C API specification and any change to this API requires a major version bump.
pub mod c {
    extern "C" {
        pub fn create_listen(ptr: u32, size: u32) -> i32;
        pub fn connect_tcp(ptr: u32, size: u32) -> i32;
        // pub fn bind_udp(ptr: u32, size: u32) -> i32;
    }
}

pub struct TcpStream {}

impl TcpStream {
    pub fn connect<A: ToSocketAddrs>(addr: A) -> io::Result<std::net::TcpStream> {
        each_addr(addr, connect)
    }

    // pub fn connect_timeout(addr: &SocketAddr, timeout: Duration) -> io::Result<TcpStream> {
    //     Err(io::Error::new(io::ErrorKind::Other, "not implemented"))
    // }
}

fn connect(addr: io::Result<&SocketAddr>) -> io::Result<std::net::TcpStream> {
    let a = addr?;
    match a {
        SocketAddr::V4(_) => {}
        SocketAddr::V6(_) => {}
    }

    let stream = StreamConfigV1::init(a.ip().to_string(), a.port(), "CONNECT_REMOTE".to_string());

    let encoded: Vec<u8> = bincode::serialize(&stream).expect("Failed to serialize");

    let address = encoded.as_ptr() as u32;
    let size = encoded.len() as u32;

    let stream = unsafe {
        // connect_tcp_unix(len, xxxx)
        let fd = cvt_r(|| c::connect_tcp(address, size))?;
        std::net::TcpStream::from_raw_fd(fd)
    };
    Ok(stream)
}

pub struct UdpSocket {}

impl UdpSocket {
    pub fn bind<A: ToSocketAddrs>(addr: A) -> io::Result<std::net::UdpSocket> {
        each_addr(addr, bind)
    }
}

fn bind(_addr: io::Result<&SocketAddr>) -> io::Result<std::net::UdpSocket> {
    Err(io::Error::new(io::ErrorKind::Other, "not implemented"))
}
