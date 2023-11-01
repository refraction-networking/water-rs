use std::io::{self, Read, Write};

// ConnStream can store either a network stream Or a file stream
pub enum ConnStream<'a> {
    Uninitialized,
    TcpStream(std::net::TcpStream),
    File(std::fs::File),
    FileRef(&'a std::fs::File),
}

impl<'a> ConnStream<'a> {
    pub fn as_read(&mut self) -> &mut dyn Read {
        match self {
            ConnStream::TcpStream(stream) => stream,
            ConnStream::File(stream) => stream,
            ConnStream::FileRef(stream) => stream,
            _ => panic!("ConnStream is uninitialized"),
        }
    }
}

impl<'a> Read for ConnStream<'a> {
    fn read(&mut self, buf: &mut [u8]) -> io::Result<usize> {
        match self {
            ConnStream::TcpStream(stream) => stream.read(buf),
            ConnStream::File(stream) => stream.read(buf),
            ConnStream::FileRef(stream) => stream.read(buf),
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "ConnStream is uninitialized",
                ))
            }
        }
    }
}

impl<'a> Write for ConnStream<'a> {
    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        match self {
            ConnStream::TcpStream(stream) => stream.write(buf),
            ConnStream::File(stream) => stream.write(buf),
            ConnStream::FileRef(stream) => stream.write(buf),
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "ConnStream is uninitialized",
                ))
            }
        }
    }

    fn flush(&mut self) -> io::Result<()> {
        match self {
            ConnStream::TcpStream(stream) => stream.flush(),
            ConnStream::File(stream) => stream.flush(),
            ConnStream::FileRef(stream) => stream.flush(),
            _ => {
                return Err(io::Error::new(
                    io::ErrorKind::Other,
                    "ConnStream is uninitialized",
                ))
            }
        }
    }
}
