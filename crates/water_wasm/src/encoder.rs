use super::*;

use tokio::io::AsyncWrite;

// Developer Guide: Logic for packaging

// A trait for a encoder, developers should implement this trait and pass it to _write_to_outbound
pub trait Encoder {
    fn encode(&self, input: &[u8], output: &mut [u8]) -> Result<u32, anyhow::Error>;
}

// A default encoder that does just copy + paste
pub struct DefaultEncoder;

impl Encoder for DefaultEncoder {
    fn encode(&self, input: &[u8], output: &mut [u8]) -> Result<u32, anyhow::Error> {
        let len = input.len();
        output[..len].copy_from_slice(&input[..len]);
        Ok(len as u32)
    }
}

pub trait AsyncEncodeWriter {
    fn poll_write_encrypted<S: AsyncWrite + ?Sized>(&mut self, stream: &mut S) -> Result<u32, anyhow::Error>;
}