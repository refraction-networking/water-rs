// =================== CONSTANTS =====================
pub const CIPHER_METHOD: CipherKind = CipherKind::CHACHA20_POLY1305;
pub const MAX_PACKET_SIZE: usize = 0x3FFF;

// =================== STD Imports ===================
use std::{
    fmt::{self, Debug},
    future::Future,
    io::{self, ErrorKind, Read},
    net::SocketAddr,
    os::fd::FromRawFd,
    pin::Pin,
    slice,
    sync::Mutex,
    task::{Context, Poll},
    vec,
};

// =================== EXTERNAL CRATES ===================
use anyhow::Result;
use bincode;
use byte_string::ByteStr;
use bytes::Buf;
use futures::ready;
use lazy_static::lazy_static;
use pin_project::pin_project;
use serde_json;
use tokio::{
    io::{AsyncRead, AsyncReadExt, AsyncWrite, AsyncWriteExt, ReadBuf},
    net::{TcpListener, TcpStream},
};
use tracing::{debug, info, Level};
use tracing_subscriber;

// =================== MODULES ===================
pub mod water;
pub mod aead;
pub mod socks5;
pub mod crypto_io;
pub mod client;
pub mod utils;

// =================== DEPENDENCIES FROM MODULES ===================
use socks5::*;
use utils::*;
use crypto_io::*;
use client::*;
use water_wasm_crate::*;
use aead::{DecryptedReader, EncryptedWriter};

// =================== SHADOWSOCKS_CRYPTO ===================
use shadowsocks_crypto::{
    v1::Cipher,
    v1::random_iv_or_salt,
    CipherKind,
};

// Export version info
#[export_name = "V1"]
pub static V1: i32 = 0;

// create a mutable global variable stores a pointer to the config
lazy_static! {
    pub static ref CONN: Mutex<Connection> = Mutex::new(Connection::new());
    pub static ref DIALER: Mutex<Dialer> = Mutex::new(Dialer::new());
}