
use anyhow::{Ok, anyhow};

pub struct Dialer {
    pub file_conn: Connection,
    pub config: Config,
}

impl Dialer {
    pub fn new() -> Self {
        Dialer {
            file_conn: Connection::new(),
            config: Config::new(),
        }
    }

    // v1 dial, where WASM has the ability to specify ip:port
    #[cfg(feature = "v1")]
    fn dial_v1(&mut self) -> Result<(), anyhow::Error> {
        info!("[WASM] running in dial func...");
    
        let mut fd: i32 = -1;
        
        // FIXME: hardcoded the filename for now, make it a config later
        fd = self.tcp_connect()?;
    
        if fd < 0 {
            eprintln!("failed to create connection to remote");
            return Err(anyhow!("failed to create connection to remote"));
        }
    
        self.file_conn.set_outbound(fd,  ConnStream::TcpStream(unsafe { std::net::TcpStream::from_raw_fd(fd) }));

        Ok(())
    }

    #[cfg(feature = "v1")]
    fn tcp_connect(&self) -> Result<i32, anyhow::Error> {
        let stream = StreamConfigV1::init(self.config.remote_address.clone(), self.config.remote_port, "CONNECT_REMOTE".to_string());
        
        let encoded: Vec<u8> = bincode::serialize(&stream).expect("Failed to serialize");
        
        let address = encoded.as_ptr() as u32;
        let size = encoded.len() as u32;
    
        let mut fd = -1;
        unsafe {
            fd = connect_tcp(address, size);
        };
    
        if fd < 0 {
            return Err(anyhow!("failed to create listener"));
        }
    
        Ok(fd)
    }
}