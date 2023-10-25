use std::io::{self, Read, Result, Write};

type CopyFnMut = dyn FnMut(&mut dyn Read, &mut dyn Write) -> Result<u64>;
// type CopyFnMut = dyn for<'a, 'b> FnMut(&'a mut (dyn Read + 'a), &'b mut (dyn Write + 'b)) -> Result<u64>;

struct TransportWithFnMut<R, W> {
    reader: R,

    writer: W,

    encode: Box<CopyFnMut>,

    decode: Box<CopyFnMut>,
}
impl<R, W> TransportWithFnMut<R, W>
where
    R: Read,
    W: Write,
{
    pub fn new(reader: R, writer: W) -> Self {
        TransportWithFnMut {
            reader,
            writer,
            encode: Box::new(|r, w| io::copy(r, w)),
            decode: Box::new(|r, w| io::copy(r, w)),
        }
    }

    pub fn with_encode<F>(mut self, f: F) -> Self
    where
        F: FnMut(&mut dyn Read, &mut dyn Write) -> Result<u64> + 'static,
    {
        self.encode = Box::new(f);
        self
    }

    pub fn with_decode<F>(mut self, f: F) -> Self
    where
        F: FnMut(&mut dyn Read, &mut dyn Write) -> Result<u64> + 'static,
    {
        self.decode = Box::new(f);
        self
    }
}

impl<R, W> Read for TransportWithFnMut<R, W>
where
    R: Read,
    W: Write,
{
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let mut b = io::BufWriter::new(buf);

        let cp = &mut self.encode;

        Ok(cp(&mut self.reader, &mut b)? as usize)
    }
}

impl<R, W> Write for TransportWithFnMut<R, W>
where
    R: Read,
    W: Write,
{
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let mut b = io::BufReader::new(buf);
        let cp = &mut self.decode;
        Ok(cp(&mut b, &mut self.writer)? as usize)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

#[test]
fn build() {
    assert!(1 == 1)
}
