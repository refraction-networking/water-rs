// =================== CONSTANTS =====================
pub const CIPHER_METHOD: CipherKind = CipherKind::CHACHA20_POLY1305;
pub const MAX_PACKET_SIZE: usize = 0x3FFF;

// =================== STD Imports ===================
use std::{
    fmt::{self, Debug},
    future::Future,
    io::{self, ErrorKind, Read},
    net::{IpAddr, SocketAddr},
    os::fd::FromRawFd,
    pin::Pin,
    slice,
    str::FromStr,
    sync::Mutex,
    task::{Context, Poll},
    vec,
};

// =================== EXTERNAL CRATES ===================
use anyhow::Result;
use byte_string::ByteStr;
use bytes::Buf;
use futures::ready;
use lazy_static::lazy_static;
use pin_project::pin_project;
use tokio::{
    io::{copy_bidirectional, AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, ReadBuf},
    net::{TcpListener, TcpStream},
};
use tracing::{debug, info, Level};

// =================== MODULES ===================
pub mod aead;
pub mod client;
pub mod config;
pub mod crypto_io;
pub mod socks5;
pub mod utils;
pub mod water;

// =================== DEPENDENCIES FROM MODULES ===================
use aead::{DecryptedReader, EncryptedWriter};
use client::*;
use config::*;
use crypto_io::*;
use socks5::*;
use utils::*;
use water_watm::*;

// =================== SHADOWSOCKS_CRYPTO ===================
use shadowsocks_crypto::{v1::random_iv_or_salt, v1::Cipher, CipherKind};

// Export version info
#[export_name = "_water_v1"]
pub static V1: i32 = 0;

// create a mutable global variable stores a pointer to the config
lazy_static! {
    pub static ref CONN: Mutex<Connection<SSConfig>> = Mutex::new(Connection::new(SSConfig::new()));
    pub static ref DIALER: Mutex<Dialer> = Mutex::new(Dialer::new());
}
