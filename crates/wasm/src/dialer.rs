use super::*;

use anyhow::Ok;
use std::os::fd::AsRawFd;

pub struct Dialer {
    pub file_conn: Connection,
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
            file_conn: Connection::new(),
            config: Config::new(),
        }
    }

    pub fn dial(&mut self) -> Result<(), anyhow::Error> {
        info!("[WASM] running in dial func...");

        let addr = self.config.dst_addr()?;

        let outbound = crate::net::TcpStream::connect(addr)?;

        self.file_conn
            .set_outbound(outbound.as_raw_fd(), ConnStream::TcpStream(outbound));

        Ok(())
    }
}
