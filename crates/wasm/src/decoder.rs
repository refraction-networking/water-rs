use tokio::io::AsyncRead;

use std::io;

/// The `Decoder` trait implements the core logic for decapsulating stream data into a caller
/// usable format.
pub trait Decoder {
    fn decode(&self, input: &[u8], output: &mut [u8]) -> io::Result<usize>;
}

/// The default decoder implemting a straight copy from input to output.
pub struct DefaultDecoder;

impl Decoder for DefaultDecoder {
    fn decode(&self, input: &[u8], output: &mut [u8]) -> io::Result<usize> {
        let mut len = input.len();
        if len < output.len() {
            len = output.len();
        }

        output[..len].copy_from_slice(&input[..len]);
        Ok(len)
    }
}

pub trait AsyncDecodeReader {
    fn poll_read_decrypted<S: AsyncRead + ?Sized>(
        &mut self,
        stream: &mut S,
        buf: &mut [u8],
    ) -> Result<u32, anyhow::Error>;
}
