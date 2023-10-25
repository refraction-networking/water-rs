use tokio::io::AsyncWrite;

use std::io;

/// The `Encoder` trait implements the core logic for encapsulating stream data into an opaque
/// format for transmission. This can include encryption, compression, or other transformations as
/// well as things like Handshakes as the encoder is given the ability to drive the connection
/// negotiation once called.
pub trait Encoder {
    fn encode(&self, input: &[u8], output: &mut [u8]) -> io::Result<usize>;
}
/// The default encoder implemting a straight copy from input to output.
pub struct DefaultEncoder;

impl Encoder for DefaultEncoder {
    fn encode(&self, input: &[u8], output: &mut [u8]) -> io::Result<usize> {
        let mut len = input.len();
        if len < output.len() {
            len = output.len();
        }

        output[..len].copy_from_slice(&input[..len]);
        Ok(len)
    }
}

pub trait AsyncEncodeWriter {
    fn poll_write_encrypted<S: AsyncWrite + ?Sized>(
        &mut self,
        stream: &mut S,
    ) -> Result<u32, anyhow::Error>;
}
