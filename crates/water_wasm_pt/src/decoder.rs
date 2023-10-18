use super::*;

use tokio::io::AsyncRead;

// Developer Guide: Logic for packaging

// A trait for a decoder, developers should implement this trait and pass it to _read_from_outbound
pub trait Decoder {
    fn decode(&self, input: &[u8], output: &mut [u8]) -> Result<u32, anyhow::Error>;
}

// A default decoder that does just copy + paste
pub struct DefaultDecoder;

impl Decoder for DefaultDecoder {
    fn decode(&self, input: &[u8], output: &mut [u8]) -> Result<u32, anyhow::Error> {
        let len = input.len();
        output[..len].copy_from_slice(&input[..len]);
        Ok(len as u32)
    }
}

pub trait AsyncDecodeReader {
    fn poll_read_decrypted<S: AsyncRead + ?Sized>(&mut self, stream: &mut S, buf: &mut [u8]) -> Result<u32, anyhow::Error>;
}