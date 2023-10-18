use super::*;
use bytes::{BufMut, BytesMut};
use std::task::{self};

pub enum ProxyClientStreamWriteState {
    Connect(Address),
    Connecting(BytesMut),
    Connected,
}

pub enum ProxyClientStreamReadState {
    Established,
}

#[pin_project]
pub struct ProxyClientStream<S> {
    #[pin]
    pub stream: CryptoStream<S>,
    pub writer_state: ProxyClientStreamWriteState,
    pub reader_state: ProxyClientStreamReadState,
}

impl<S> AsyncRead for ProxyClientStream<S>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    #[inline]
    fn poll_read(
        self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
        buf: &mut ReadBuf<'_>,
    ) -> Poll<io::Result<()>> {
        #[allow(unused_mut)]
        let mut this = self.project();
        return this.stream.poll_read_decrypted(cx, buf).map_err(Into::into);
    }
}

impl<S> AsyncWrite for ProxyClientStream<S>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    fn poll_write(
        self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
        buf: &[u8],
    ) -> Poll<Result<usize, io::Error>> {
        let this = self.project();

        loop {
            match this.writer_state {
                ProxyClientStreamWriteState::Connect(ref addr) => {
                    let buffer = make_first_packet_buffer(this.stream.method, addr, buf);
                    *(this.writer_state) = ProxyClientStreamWriteState::Connecting(buffer);
                }
                ProxyClientStreamWriteState::Connecting(ref buffer) => {
                    let n = ready!(this.stream.poll_write_encrypted(cx, buffer))?;

                    // In general, poll_write_encrypted should perform like write_all.
                    debug_assert!(n == buffer.len());

                    *(this.writer_state) = ProxyClientStreamWriteState::Connected;

                    return Ok(buf.len()).into();
                }
                ProxyClientStreamWriteState::Connected => {
                    return this
                        .stream
                        .poll_write_encrypted(cx, buf)
                        .map_err(Into::into);
                }
            }
        }
    }

    #[inline]
    fn poll_flush(self: Pin<&mut Self>, cx: &mut task::Context<'_>) -> Poll<Result<(), io::Error>> {
        self.project().stream.poll_flush(cx).map_err(Into::into)
    }

    #[inline]
    fn poll_shutdown(
        self: Pin<&mut Self>,
        cx: &mut task::Context<'_>,
    ) -> Poll<Result<(), io::Error>> {
        self.project().stream.poll_shutdown(cx).map_err(Into::into)
    }
}

impl<S> ProxyClientStream<S>
where
    S: AsyncRead + AsyncWrite + Unpin,
{
    pub fn from_stream<A>(
        stream: S,
        addr: A,
        method: CipherKind,
        key: &[u8],
    ) -> ProxyClientStream<S>
    where
        A: Into<Address>,
    {
        let addr = addr.into();
        let stream = CryptoStream::from_stream_with_identity(stream, method, key);

        let reader_state = ProxyClientStreamReadState::Established;

        ProxyClientStream {
            stream,
            writer_state: ProxyClientStreamWriteState::Connect(addr),
            reader_state,
        }
    }
}

#[inline]
pub fn make_first_packet_buffer(method: CipherKind, addr: &Address, buf: &[u8]) -> BytesMut {
    // Target Address should be sent with the first packet together,
    // which would prevent from being detected.

    info!("[making first packet] {:?}", buf);

    let addr_length = addr.serialized_len();
    let mut buffer = BytesMut::new();

    let header_length = addr_length + buf.len();
    buffer.reserve(header_length);

    // STREAM / AEAD / AEAD2022 protocol, append the Address before payload
    addr.write_to_buf(&mut buffer);

    buffer.put_slice(buf);

    let slice: &[u8] = &buffer;

    info!("[after making first packet] {:?}", slice);

    buffer
}
