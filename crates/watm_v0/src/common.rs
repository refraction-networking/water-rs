//! The common code for v0 development of WATM module

use std::os::fd::FromRawFd;
use tokio::net::TcpStream;

// WASI Imports
extern "C" {
    /// obtain a connection (specified by returned fd) accepted by the host
    pub fn host_accept() -> i32;
    /// obtain a connection (specified by returned fd) dialed by the host
    pub fn host_dial() -> i32;
    /// call when exiting
    pub fn host_defer();
    /// obtain a configuration file (specified by returned fd) from the host
    #[allow(dead_code)]
    pub fn pull_config() -> i32;
}

/// enumerated constants for Role (i32)
///  0: unknown
///  1: dialer
///  2: listener
///  3: relay
#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum Role {
    Unknown = 0,
    Dialer = 1,
    Listener = 2,
    Relay = 3,
}

pub struct AsyncFdConn {
    fd: i32,
    /// used to hold the std tcp stream, will be upgraded to tokio stream later
    temp_stream: Option<std::net::TcpStream>,
    stream: Option<TcpStream>,
}

impl Default for AsyncFdConn {
    fn default() -> Self {
        Self::new()
    }
}

impl AsyncFdConn {
    pub fn new() -> Self {
        AsyncFdConn {
            fd: -1,
            temp_stream: None,
            stream: None,
        }
    }

    pub fn wrap(&mut self, fd: i32) -> Result<(), String> {
        if self.fd > 0 {
            return Err("already wrapped".to_string());
        }
        if fd < 0 {
            return Err("invalid fd".to_string());
        }
        self.fd = fd;
        println!("wrap: fd = {}", fd);
        let stdstream = unsafe { std::net::TcpStream::from_raw_fd(fd) };

        self.temp_stream = Some(stdstream);

        Ok(())
    }

    /// upgrade the std stream to tokio stream where to explicitly make it non-blocking for async
    pub fn tokio_upgrade(&mut self) -> Result<(), String> {
        if self.fd < 0 {
            return Err("invalid fd".to_string());
        }
        let stdstream = self.temp_stream.take().unwrap();
        stdstream
            .set_nonblocking(true)
            .expect("Failed to set non-blocking");
        self.stream =
            Some(TcpStream::from_std(stdstream).expect("Failed to convert to tokio stream"));
        Ok(())
    }

    pub fn close(&mut self) {
        if self.fd < 0 {
            return;
        }
        let stream = self.stream.take().unwrap();
        drop(stream);
        self.fd = -1;
    }

    pub fn stream(&mut self) -> Option<&mut TcpStream> {
        self.stream.as_mut()
    }
}
