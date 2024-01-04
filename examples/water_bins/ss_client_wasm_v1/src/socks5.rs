use super::*;

use bytes::{BufMut, BytesMut};
use std::fmt::Formatter;
use std::net::{SocketAddrV4, SocketAddrV6};

#[rustfmt::skip]
pub mod consts {
    pub const SOCKS5_ADDR_TYPE_IPV4:                   u8 = 0x01;
    pub const SOCKS5_ADDR_TYPE_DOMAIN_NAME:            u8 = 0x03;
    pub const SOCKS5_ADDR_TYPE_IPV6:                   u8 = 0x04;

    pub const SOCKS5_VERSION:                          u8 = 0x05;
    pub const SOCKS5_REPLY_SUCCEEDED:                  u8 = 0x00;
}

/// SOCKS5 address type
#[derive(Clone, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Address {
    /// Socket address (IP Address)
    SocketAddress(SocketAddr),
    /// Domain name address
    DomainNameAddress(String, u16),
}

impl Address {
    /// Writes to buffer
    #[inline]
    pub fn write_to_buf<B: BufMut>(&self, buf: &mut B) {
        write_address(self, buf)
    }

    /// Get required buffer size for serializing
    #[inline]
    pub fn serialized_len(&self) -> usize {
        get_addr_len(self)
    }
}

impl Debug for Address {
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            Address::SocketAddress(ref addr) => write!(f, "{addr}"),
            Address::DomainNameAddress(ref addr, ref port) => write!(f, "{addr}:{port}"),
        }
    }
}

impl fmt::Display for Address {
    #[inline]
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match *self {
            Address::SocketAddress(ref addr) => write!(f, "{addr}"),
            Address::DomainNameAddress(ref addr, ref port) => write!(f, "{addr}:{port}"),
        }
    }
}

#[inline]
pub fn get_addr_len(atyp: &Address) -> usize {
    match *atyp {
        Address::SocketAddress(SocketAddr::V4(..)) => 1 + 4 + 2,
        Address::SocketAddress(SocketAddr::V6(..)) => 1 + 8 * 2 + 2,
        Address::DomainNameAddress(ref dmname, _) => 1 + 1 + dmname.len() + 2,
    }
}

pub fn write_address<B: BufMut>(addr: &Address, buf: &mut B) {
    match *addr {
        Address::SocketAddress(ref addr) => write_socket_address(addr, buf),
        Address::DomainNameAddress(ref dnaddr, ref port) => {
            write_domain_name_address(dnaddr, *port, buf)
        }
    }
}

pub fn write_domain_name_address<B: BufMut>(dnaddr: &str, port: u16, buf: &mut B) {
    assert!(dnaddr.len() <= u8::MAX as usize);

    buf.put_u8(consts::SOCKS5_ADDR_TYPE_DOMAIN_NAME);
    assert!(
        dnaddr.len() <= u8::MAX as usize,
        "domain name length must be smaller than 256"
    );
    buf.put_u8(dnaddr.len() as u8);
    buf.put_slice(dnaddr[..].as_bytes());
    buf.put_u16(port);
}

pub fn write_socket_address<B: BufMut>(addr: &SocketAddr, buf: &mut B) {
    match *addr {
        SocketAddr::V4(ref addr) => write_ipv4_address(addr, buf),
        SocketAddr::V6(ref addr) => write_ipv6_address(addr, buf),
    }
}

pub fn write_ipv4_address<B: BufMut>(addr: &SocketAddrV4, buf: &mut B) {
    buf.put_u8(consts::SOCKS5_ADDR_TYPE_IPV4); // Address type
    buf.put_slice(&addr.ip().octets()); // Ipv4 bytes
    buf.put_u16(addr.port()); // Port
}

pub fn write_ipv6_address<B: BufMut>(addr: &SocketAddrV6, buf: &mut B) {
    buf.put_u8(consts::SOCKS5_ADDR_TYPE_IPV6); // Address type
    for seg in &addr.ip().segments() {
        buf.put_u16(*seg); // Ipv6 bytes
    }
    buf.put_u16(addr.port()); // Port
}

pub struct Socks5Handler {
    pub stream: TcpStream,
    pub buffer: BytesMut,
}

impl Socks5Handler {
    pub fn new(stream: TcpStream) -> Self {
        Self {
            stream,
            buffer: BytesMut::with_capacity(512),
        }
    }

    pub async fn socks5_greet(&mut self) -> Result<(), std::io::Error> {
        // Read the SOCKS5 greeting
        self.stream
            .read_buf(&mut self.buffer)
            .await
            .expect("Failed to read from stream");

        info!(
            "SOCKS5 greeting: Received {} bytes: {:?}",
            self.buffer.len(),
            self.buffer.to_vec()
        );

        if self.buffer.len() < 2 || self.buffer[0] != 0x05 {
            eprintln!("Not a SOCKS5 request");
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Not a SOCKS5 request",
            ));
        }

        let nmethods = self.buffer[1] as usize;
        if self.buffer.len() < 2 + nmethods {
            eprintln!("Incomplete SOCKS5 greeting");
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Incomplete SOCKS5 greeting",
            ));
        }

        // For simplicity, always use "NO AUTHENTICATION REQUIRED"
        self.stream
            .write_all(&[0x05, 0x00])
            .await
            .expect("Failed to write to stream");

        self.buffer.clear();

        Ok(())
    }

    pub async fn socks5_get_target(&mut self) -> Result<Address, std::io::Error> {
        // Read the actual request
        self.stream
            .read_buf(&mut self.buffer)
            .await
            .expect("Failed to read from stream");

        info!(
            "Actual SOCKS5 request: Received {} bytes: {:?}",
            self.buffer.len(),
            self.buffer.to_vec()
        );

        if self.buffer.len() < 7 || self.buffer[0] != 0x05 || self.buffer[1] != 0x01 {
            println!("Invalid SOCKS5 request");
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "Invalid SOCKS5 request",
            ));
        }

        // Extract address and port
        let target_addr: Address = match self.buffer[3] {
            0x01 => {
                // IPv4
                if self.buffer.len() < 10 {
                    eprintln!("Incomplete request for IPv4 address");
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "Incomplete request for IPv4 address",
                    ));
                }
                let addr = std::net::Ipv4Addr::new(
                    self.buffer[4],
                    self.buffer[5],
                    self.buffer[6],
                    self.buffer[7],
                );
                let port = (&self.buffer[8..10]).get_u16();
                Address::SocketAddress(SocketAddr::from((addr, port)))
            }
            0x03 => {
                // Domain name
                let domain_length = self.buffer[4] as usize;
                if self.buffer.len() < domain_length + 5 {
                    eprintln!("Incomplete request for domain name");
                    return Err(io::Error::new(
                        io::ErrorKind::InvalidInput,
                        "Incomplete request for domain name",
                    ));
                }
                let domain = std::str::from_utf8(&self.buffer[5..5 + domain_length])
                    .expect("Invalid domain string");

                let port = (&self.buffer[5 + domain_length..5 + domain_length + 2]).get_u16();

                info!("Requested Domain:port: {}:{}", domain, port);

                Address::DomainNameAddress(domain.to_string(), port)
            }
            _ => {
                eprintln!("Address type not supported");
                return Err(io::Error::new(
                    io::ErrorKind::InvalidInput,
                    "Address type not supported",
                ));
            }
        };

        Ok(target_addr)
    }

    pub async fn socks5_response(&mut self, buf: &mut BytesMut) {
        // Send the response header
        self.stream
            .write_all(buf)
            .await
            .expect("Failed to write back to client's stream");
        info!("Responsed header to SOCKS5 client: {:?}", buf.to_vec());
    }
}
