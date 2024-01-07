use super::*;

use anyhow::{anyhow, Ok};

pub struct Dialer {
    pub file_conn: Connection<Config>,
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
            file_conn: Connection::default(),
            config: Config::new(),
        }
    }

    pub fn dial(&mut self) -> Result<i32, anyhow::Error> {
        info!("[WASM] running in dial func...");

        let fd: i32 = self.tcp_connect()?;

        if fd < 0 {
            eprintln!("failed to create connection to remote");
            return Err(anyhow!("failed to create connection to remote"));
        }

        self.file_conn.set_outbound(
            fd,
            ConnStream::TcpStream(unsafe { std::net::TcpStream::from_raw_fd(fd) }),
        );

        Ok(fd)
    }

    fn tcp_connect(&self) -> Result<i32, anyhow::Error> {
        let stream = StreamConfigV1::init(
            self.config.remote_address.clone(),
            self.config.remote_port,
            "CONNECT_REMOTE".to_string(),
        );

        let encoded: Vec<u8> = bincode::serialize(&stream).expect("Failed to serialize");

        let address = encoded.as_ptr() as u32;
        let size = encoded.len() as u32;

        let fd = unsafe {
            // connect_tcp_unix(len, xxxx)
            connect_tcp(address, size)
        };

        if fd < 0 {
            return Err(anyhow!("failed to connect to remote"));
        }

        Ok(fd)
    }
}
