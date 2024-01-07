use super::*;

// ConnStream can store either a network stream Or a file stream
pub enum ConnStream {
    TcpStream(std::net::TcpStream),
    File(std::fs::File),
}

impl ConnStream {
    pub fn as_read(&mut self) -> &mut dyn Read {
        match self {
            ConnStream::TcpStream(stream) => stream,
            ConnStream::File(stream) => stream,
        }
    }
}

// ConnFile is the struct for a connection -- either for in / outbound
pub struct ConnFile {
    pub fd: i32,
    pub file: Option<ConnStream>,
}

impl Default for ConnFile {
    fn default() -> Self {
        Self::new()
    }
}

impl ConnFile {
    // A default constructor for ConnFile
    pub fn new() -> Self {
        ConnFile { fd: -1, file: None }
    }

    pub fn read(&mut self, buf: &mut [u8]) -> Result<i64, anyhow::Error> {
        match &mut self.file {
            Some(stream) => {
                let bytes_read = match stream {
                    ConnStream::TcpStream(stream) => {
                        stream.read(buf).map_err(anyhow::Error::from)?
                    }
                    ConnStream::File(stream) => stream.read(buf).map_err(anyhow::Error::from)?,
                };
                Ok(bytes_read as i64)
            }
            None => {
                eprintln!("[WASM] > ERROR: ConnFile's file is None");
                Err(anyhow::anyhow!("ConnFile's file is None"))
            }
        }
    }

    pub fn write(&mut self, buf: &[u8]) -> Result<(), anyhow::Error> {
        match &mut self.file {
            Some(stream) => match stream {
                ConnStream::TcpStream(stream) => stream.write_all(buf).map_err(anyhow::Error::from),
                ConnStream::File(stream) => stream.write_all(buf).map_err(anyhow::Error::from),
            },
            None => Err(anyhow::anyhow!("[WASM] > ERROR: ConnFile's file is None")),
        }
    }
}

// A Connection normally contains both in & outbound streams + a config
pub struct Connection<T> {
    pub inbound_conn: ConnFile,
    pub outbound_conn: ConnFile,

    pub config: T,
}

impl<T: Default> Default for Connection<T> {
    fn default() -> Self {
        Self {
            inbound_conn: ConnFile::new(),
            outbound_conn: ConnFile::new(),
            config: T::default(),
        }
    }
}

impl<T> Connection<T> {
    // A default constructor
    pub fn new(config: T) -> Self {
        Connection {
            inbound_conn: ConnFile::new(),
            outbound_conn: ConnFile::new(),
            config,
        }
    }

    pub fn set_inbound(&mut self, fd: i32, stream: ConnStream) {
        if fd < 0 {
            eprintln!("[WASM] > ERROR: fd is negative");
            return;
        }

        if self.inbound_conn.fd != -1 {
            eprintln!("[WASM] > ERROR: inbound_conn.fd has been set");
            return;
        }

        self.inbound_conn.fd = fd;
        self.inbound_conn.file = Some(stream);
    }

    pub fn set_outbound(&mut self, fd: i32, stream: ConnStream) {
        if fd < 0 {
            eprintln!("[WASM] > ERROR: fd is negative");
            return;
        }

        if self.outbound_conn.fd != -1 {
            eprintln!("[WASM] > ERROR: outbound_conn.fd has been set");
            return;
        }

        self.outbound_conn.fd = fd;
        self.outbound_conn.file = Some(stream);
    }

    // pub fn decoder_read_from_outbound<D: AsyncDecodeReader>(&mut self, decoder: &mut D, buf: &mut [u8]) -> Result<i64, anyhow::Error> {
    //     debug!("[WASM] running in decoder_read_from_outbound");

    //     // match self.outbound_conn.file.as_mut().unwrap() {
    //     //     ConnStream::TcpStream(stream) => {
    //     //         decoder.read_decrypted(stream);
    //     //     },
    //     //     ConnStream::File(stream) => {
    //     //         decoder.read_decrypted(stream);
    //     //     },
    //     // }
    //     Ok(decoder.poll_read_decrypted(self.outbound_conn.file.as_mut().unwrap().as_read(), buf)? as i64)
    // }

    /// this _read function is triggered by the Host to read from the remote connection
    pub fn _read_from_outbound<D: Decoder>(
        &mut self,
        decoder: &mut D,
    ) -> Result<i64, anyhow::Error> {
        debug!("[WASM] running in _read_from_net");

        let mut buf = vec![0u8; 4096];
        let bytes_read: i64 = match self.outbound_conn.read(&mut buf) {
            Ok(n) => n,
            Err(e) => {
                // eprintln!("[WASM] > ERROR in _read when reading from outbound: {:?}", e);
                // return -1; // Or another sentinel value to indicate error}
                return Err(anyhow::anyhow!(
                    "[WASM] > ERROR in _read when reading from outbound: {:?}",
                    e
                ));
            }
        };

        // NOTE: decode logic here
        let mut decoded = vec![0u8; 4096];
        let len_after_decoding = match decoder.decode(&buf[..bytes_read as usize], &mut decoded) {
            Ok(n) => n,
            Err(e) => {
                // eprintln!("[WASM] > ERROR in _write when encoding: {:?}", e);
                // return -1; // Or another sentinel value to indicate error
                return Err(anyhow::anyhow!(
                    "[WASM] > ERROR in _write when encoding: {:?}",
                    e
                ));
            }
        };

        match self
            .inbound_conn
            .write(decoded[..len_after_decoding as usize].as_ref())
        {
            Ok(_) => {}
            Err(e) => {
                // eprintln!("[WASM] > ERROR in _read when writing to inbound: {:?}", e);
                // return -1; // Or another sentinel value to indicate error
                return Err(anyhow::anyhow!(
                    "[WASM] > ERROR in _read when writing to inbound: {:?}",
                    e
                ));
            }
        }

        Ok(len_after_decoding as i64)
    }

    pub fn _write_2_outbound<E: Encoder>(
        &mut self,
        encoder: &mut E,
        bytes_write: i64,
    ) -> Result<i64, anyhow::Error> {
        debug!("[WASM] running in _write_2_net");

        let mut bytes_read: i64 = 0;
        let mut buf = vec![0u8; 4096];
        loop {
            let read = match self.inbound_conn.read(&mut buf) {
                Ok(n) => n,
                Err(e) => {
                    // eprintln!("[WASM] > ERROR in _read when reading from inbound: {:?}", e);
                    // return -1; // Or another sentinel value to indicate error
                    return Err(anyhow::anyhow!(
                        "[WASM] > ERROR in _read when reading from inbound: {:?}",
                        e
                    ));
                }
            };

            bytes_read += read;

            if read == 0 || bytes_read == bytes_write {
                break;
            }
        }

        // NOTE: encode logic here
        let mut encoded = vec![0u8; 4096];
        let len_after_encoding = match encoder.encode(&buf[..bytes_read as usize], &mut encoded) {
            Ok(n) => n,
            Err(e) => {
                // eprintln!("[WASM] > ERROR in _write when encoding: {:?}", e);
                // return -1; // Or another sentinel value to indicate error
                return Err(anyhow::anyhow!(
                    "[WASM] > ERROR in _write when encoding: {:?}",
                    e
                ));
            }
        };

        match self
            .outbound_conn
            .write(encoded[..len_after_encoding as usize].as_ref())
        {
            Ok(_) => {}
            Err(e) => {
                // eprintln!("[WASM] > ERROR in _read when writing to outbound: {:?}", e);
                // return -1; // Or another sentinel value to indicate error
                return Err(anyhow::anyhow!(
                    "[WASM] > ERROR in _read when writing to outbound: {:?}",
                    e
                ));
            }
        }

        Ok(len_after_encoding as i64)
    }

    pub fn close_inbound(&mut self) {
        match &mut self.inbound_conn.file {
            Some(stream) => match stream {
                ConnStream::TcpStream(stream) => {
                    stream.shutdown(std::net::Shutdown::Both).unwrap();
                }
                ConnStream::File(stream) => {
                    stream.sync_all().unwrap();
                }
            },
            None => {
                eprintln!("[WASM] > ERROR: ConnFile's file is None");
            }
        }

        self.inbound_conn.fd = -1;
    }

    pub fn close_outbound(&mut self) {
        match &mut self.outbound_conn.file {
            Some(stream) => match stream {
                ConnStream::TcpStream(stream) => {
                    stream.shutdown(std::net::Shutdown::Both).unwrap();
                }
                ConnStream::File(stream) => {
                    stream.sync_all().unwrap();
                }
            },
            None => {
                eprintln!("[WASM] > ERROR: ConnFile's file is None");
            }
        }

        self.outbound_conn.fd = -1;
    }
}
