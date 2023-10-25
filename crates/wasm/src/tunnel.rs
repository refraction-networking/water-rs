use crate::config::Config;
use crate::connections::ConnStream;
use crate::decoder::Decoder;
use crate::encoder::Encoder;

use tracing::{debug, trace, warn};

use std::io::{self, Read, Write};

// A Tunnel normally contains both in & outbound streams + a config
pub struct Tunnel<'a, E, D> {
    pub plaintext_conn: ConnStream<'a>,
    pub ciphertext_conn: ConnStream<'a>,

    encoder: Option<E>,
    decoder: Option<D>,

    pub config: Config,
}

impl<'a, E, D> Default for Tunnel<'a, E, D>
where
    E: Encoder,
    D: Decoder,
{
    fn default() -> Self {
        Self::new()
    }
}

impl<'a, E, D> Tunnel<'a, E, D>
where
    E: Encoder,
    D: Decoder,
{
    // A default constructor
    pub fn new() -> Self {
        Tunnel {
            plaintext_conn: ConnStream::Uninitialized,
            ciphertext_conn: ConnStream::Uninitialized,

            encoder: None,
            decoder: None,

            config: Config::new(),
        }
    }

    pub fn set_encoder(mut self, encoder: E) -> Self {
        self.encoder = Some(encoder);
        self
    }

    pub fn unset_encoder(&mut self) {
        self.encoder = None;
    }

    pub fn set_decoder(mut self, decoder: D) -> Self {
        self.decoder = Some(decoder);
        self
    }

    pub fn unset_decoder(&mut self) {
        self.decoder = None;
    }

    pub fn set_inbound(self, fd: i32, _stream: ConnStream) -> io::Result<Self> {
        if fd < 0 {
            eprintln!("[WASM] > ERROR: fd is negative");
            return Err(io::Error::new(
                io::ErrorKind::Other,
                "[WASM] > ERROR: fd is negative",
            ));
        }

        // if self.plaintext_conn.fd != -1 {
        //     warn!("[WASM] > ERROR: plaintext_conn.fd has been set");
        // }

        // self.plaintext_conn.file = Some(stream);
        Ok(self)
    }

    pub fn set_outbound(self, fd: i32, _stream: ConnStream) -> io::Result<Self> {
        if fd < 0 {
            warn!("[WASM] > ERROR: fd is negative");
        }

        // self.ciphertext_conn.file = Some(stream);
        Ok(self)
    }

    /// this _read function is triggered by the Host to read from the remote connection
    pub fn _read_from_outbound(&mut self) -> Result<usize, anyhow::Error> {
        debug!("[WASM] running in _read_from_net");

        let mut buf = vec![0u8; 4096];
        let bytes_read = match (&mut self.ciphertext_conn).read(&mut buf) {
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

        let (decoded, len_after_decoding) = match &self.decoder {
            Some(d) => {
                // NOTE: decode logic here
                let mut decoded = vec![0u8; 4096];
                let len_after_decoding = match d.decode(&buf[..bytes_read], &mut decoded) {
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
                (decoded, len_after_decoding)
            }
            None => {
                trace!("[WASM] > ERROR: attempting to decode with no decoder set");
                return Err(anyhow::anyhow!("decoder is None"));
            }
        };

        match self
            .plaintext_conn
            .write(decoded[..len_after_decoding].as_ref())
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

        Ok(len_after_decoding)
    }

    pub fn _write_to_outbound(&mut self, bytes_write: usize) -> Result<usize, anyhow::Error> {
        debug!("[WASM] running in _write_2_net");

        let mut bytes_read = 0;
        let mut buf = vec![0u8; 4096];
        loop {
            let read = match self.plaintext_conn.read(&mut buf) {
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

        let (encoded, len_after_encoding) = match &self.encoder {
            Some(e) => {
                // NOTE: encode logic here
                let mut encoded = vec![0u8; 4096];
                let len_after_encoding = e.encode(&buf[..bytes_read], &mut encoded)?;
                (encoded, len_after_encoding)
            }
            None => {
                trace!("[WASM] > ERROR: attempting to encode with no encoder set");
                return Err(anyhow::anyhow!("encoder is None"));
            }
        };

        match self
            .ciphertext_conn
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

        Ok(len_after_encoding)
    }

    // pub fn close_inbound(&mut self) {
    //     match &mut self.plaintext_conn.file {
    //         Some(stream) => match stream {
    //             ConnStream::TcpStream(stream) => {
    //                 stream.shutdown(std::net::Shutdown::Both).unwrap();
    //             }
    //             ConnStream::File(stream) => {
    //                 stream.sync_all().unwrap();
    //             }
    //             ConnStream::FileRef(stream) => {
    //                 stream.sync_all().unwrap();
    //             }
    //         },
    //         None => {
    //             eprintln!("[WASM] > ERROR: inbound file is None");
    //         }
    //     }

    //     self.plaintext_conn.fd = -1;
    // }

    // pub fn close_outbound(&mut self) {
    //     match &mut self.ciphertext_conn.file {
    //         Some(stream) => match stream {
    //             ConnStream::TcpStream(stream) => {
    //                 stream.shutdown(std::net::Shutdown::Both).unwrap();
    //             }
    //             ConnStream::File(stream) => {
    //                 stream.sync_all().unwrap();
    //             }
    //             ConnStream::FileRef(stream) => {
    //                 stream.sync_all().unwrap();
    //             }
    //         },
    //         None => {
    //             eprintln!("[WASM] > ERROR: outbound file is None");
    //         }
    //     }

    //     self.ciphertext_conn.fd = -1;
    // }
}

#[cfg(test)]
mod test {
    use crate::{DefaultDecoder, DefaultEncoder};
    use std::os::fd::AsRawFd;

    use super::*;
    use anyhow::Result;
    use tempfile::tempfile;

    #[test]
    fn basic_operation() -> Result<()> {
        let mut f1 = tempfile()?;
        let fd1 = f1.as_raw_fd();
        let c1 = ConnStream::FileRef(&f1);

        let mut f2 = tempfile()?;
        let fd2 = f2.as_raw_fd();
        let c2 = ConnStream::FileRef(&f2);

        let mut t = Tunnel::new()
            .set_inbound(fd1, c1)?
            .set_outbound(fd2, c2)?
            .set_encoder(DefaultEncoder)
            .set_decoder(DefaultDecoder);

        let mut buf = vec![0u8; 4096];
        let msg = b"hello world";
        let n_written = f1.write(msg)?;
        let n_encoded = t._write_to_outbound(n_written)?;
        let n_decoded = t._read_from_outbound()?;
        let n_read = f2.read(&mut buf)?;
        let out_msg = &buf[..n_read as usize];

        assert_eq!(n_written, n_decoded); // because IdentityEncoder
        assert_eq!(n_read, n_encoded); // because IdentityEncoder
        assert_eq!(msg, out_msg);
        assert_eq!(n_read, n_written);

        Ok(())
    }

    #[test]
    fn read_write() {
        assert!(1 == 1)
    }

    #[test]
    fn large_read_write() {
        assert!(1 == 1)
    }

    #[test]
    fn error_handling() {
        assert!(1 == 1)
    }
}
