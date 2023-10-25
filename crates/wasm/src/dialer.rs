use crate::{Config, ConnStream, Decoder, Encoder, Tunnel};

use tracing::info;

use std::io;
use std::os::fd::AsRawFd;

pub struct Dialer {
    pub config: Config,
}

impl Default for Dialer {
    fn default() -> Self {
        Self::new()
    }
}

impl Dialer {
    pub fn new() -> Self {
        Dialer {
            config: Config::new(),
        }
    }

    pub fn with_config(config: Config) -> Self {
        Dialer { config }
    }

    pub fn set_config(mut self, config: Config) -> Self {
        self.config = config;
        self
    }

    pub fn dial<E, D>(&mut self) -> io::Result<Tunnel<E, D>>
    where
        E: Encoder,
        D: Decoder,
    {
        info!("[WASM] running in dial func...");

        let addr = self.config.dst_addr().map_err(|e| {
            eprintln!("[WASM] > ERROR: {}", e);
            std::io::Error::new(std::io::ErrorKind::Other, "failed to get dst addr")
        })?;

        let outbound = crate::net::TcpStream::connect(addr)?;

        Tunnel::new().set_outbound(outbound.as_raw_fd(), ConnStream::TcpStream(outbound))
    }
}
