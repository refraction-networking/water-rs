use super::*;

use std::task::{self};

lazy_static! {
    pub static ref ENC_CIPHER: Mutex<Cipher> = Mutex::new(Cipher::new(CipherKind::NONE, &[], &[]));
    pub static ref DEC_CIPHER: Mutex<DecryptedReader> =
        Mutex::new(DecryptedReader::new(CipherKind::NONE, &[]));
}

/// AEAD Protocol Error
#[derive(thiserror::Error, Debug)]
pub enum ProtocolError {
    #[error(transparent)]
    IoError(#[from] io::Error),
    #[error("header too short, expecting {0} bytes, but found {1} bytes")]
    HeaderTooShort(usize, usize),
    #[error("decrypt data failed")]
    DecryptDataError,
    #[error("decrypt length failed")]
    DecryptLengthError,
    #[error("buffer size too large ({0:#x}), AEAD encryption protocol requires buffer to be smaller than 0x3FFF, the higher two bits must be set to zero")]
    DataTooLong(usize),
}

/// AEAD Protocol result
pub type ProtocolResult<T> = Result<T, ProtocolError>;

impl From<ProtocolError> for io::Error {
    fn from(e: ProtocolError) -> io::Error {
        match e {
            ProtocolError::IoError(err) => err,
            _ => io::Error::new(ErrorKind::Other, e),
        }
    }
}

/// The type of TCP stream
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum StreamType {
    /// Client -> Server
    Client,
    /// Server -> Client
    Server,
}

/// Cryptographic reader trait
pub trait CryptoRead {
    fn poll_read_decrypted(
        self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<ProtocolResult<()>>;
}

/// Cryptographic writer trait
pub trait CryptoWrite {
    fn poll_write_encrypted(
        self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
        buf: &[u8],
    ) -> Poll<ProtocolResult<usize>>;
}

/// A bidirectional stream for read/write encrypted data in shadowsocks' tunnel
pub struct CryptoStream<S> {
    pub stream: S,
    pub dec: DecryptedReader,
    pub enc: EncryptedWriter,
    pub method: CipherKind,
    pub has_handshaked: bool,
}

impl<S> CryptoRead for CryptoStream<S>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    /// Attempt to read decrypted data from `stream`
    #[inline]
    fn poll_read_decrypted(
        mut self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<ProtocolResult<()>> {
        let CryptoStream {
            ref mut dec,
            // ref mut enc,
            ref mut stream,
            ref mut has_handshaked,
            ..
        } = *self;
        ready!(dec.poll_read_decrypted(cx, stream, buf))?;

        if !*has_handshaked && dec.handshaked() {
            *has_handshaked = true;
        }

        Ok(()).into()
    }
}

impl<S> CryptoWrite for CryptoStream<S>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    /// Attempt to write encrypted data to `stream`
    #[inline]
    fn poll_write_encrypted(
        mut self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
        buf: &[u8],
    ) -> Poll<ProtocolResult<usize>> {
        let CryptoStream {
            ref mut enc,
            ref mut stream,
            ..
        } = *self;
        enc.poll_write_encrypted(cx, stream, buf)
            .map_err(Into::into)
    }
}

impl<S> CryptoStream<S>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    /// Polls `flush` on the underlying stream
    #[inline]
    pub fn poll_flush(&mut self, cx: &mut task::Context<'_>) -> Poll<ProtocolResult<()>> {
        Pin::new(&mut self.stream)
            .poll_flush(cx)
            .map_err(Into::into)
    }

    /// Polls `shutdown` on the underlying stream
    #[inline]
    pub fn poll_shutdown(&mut self, cx: &mut task::Context<'_>) -> Poll<ProtocolResult<()>> {
        Pin::new(&mut self.stream)
            .poll_shutdown(cx)
            .map_err(Into::into)
    }
}

impl<S: AsyncRead + AsyncWrite> CryptoStream<S> {
    pub fn new(stream: S, method: CipherKind, key: &[u8], nonce: &[u8]) -> CryptoStream<S> {
        CryptoStream {
            stream,
            dec: DecryptedReader::new(method, key),
            enc: EncryptedWriter::new(method, key, nonce),
            method,
            has_handshaked: false,
        }
    }

    /// Create a new CryptoStream with the underlying stream connection
    pub fn from_stream_with_identity(stream: S, method: CipherKind, key: &[u8]) -> CryptoStream<S> {
        let prev_len = method.salt_len();

        let iv = {
            let mut local_salt = vec![0u8; prev_len];
            generate_nonce(method, &mut local_salt, true);
            info!("generated AEAD cipher salt {:?}", ByteStr::new(&local_salt));
            local_salt
        };

        CryptoStream {
            stream,
            dec: DecryptedReader::new(method, key),
            enc: EncryptedWriter::new(method, key, &iv),
            method,
            has_handshaked: false,
        }
    }
}

/// Generate nonce (IV or SALT)
pub fn generate_nonce(_method: CipherKind, nonce: &mut [u8], _unique: bool) {
    if nonce.is_empty() {
        return;
    }

    random_iv_or_salt(nonce);

    // loop {
    //     random_iv_or_salt(nonce);

    //     // Salt already exists, generate a new one. FIXME: ignore replay attack for now
    //     if unique && false {
    //         continue;
    //     }

    //     break;
    // }
}

pub fn create_cipher(key: &[u8], nonce: &[u8], kind: CipherKind) -> Cipher {
    Cipher::new(kind, key, nonce)
}
