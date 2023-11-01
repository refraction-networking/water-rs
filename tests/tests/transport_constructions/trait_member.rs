use std::io::{self, Read, Result, Write};

pub trait MyCopyTrait {
    fn transform<R, W>(&mut self, reader: &mut R, writer: &mut W) -> Result<u64>
    where
        R: Read + ?Sized,
        W: Write + ?Sized;
}

pub struct TransportWithTrait<R, W, Fe, Fd> {
    reader: R,

    writer: W,

    encode_f: Fe,

    decode_f: Fd,
}

impl<R, W, Fe, Fd> TransportWithTrait<R, W, Fe, Fd>
where
    R: Read,
    W: Write,
    Fe: MyCopyTrait,
    Fd: MyCopyTrait,
{
    pub fn new(reader: R, writer: W, encoder: Fe, decoder: Fd) -> Self {
        TransportWithTrait {
            reader,
            writer,
            encode_f: encoder, //Trfm{forward:true},
            decode_f: decoder, //Trfm{forward:false}
        }
    }
}

struct Trfm {
    forward: bool,
}

impl MyCopyTrait for Trfm {
    fn transform<R, W>(&mut self, reader: &mut R, writer: &mut W) -> Result<u64>
    where
        R: Read + ?Sized,
        W: Write + ?Sized,
    {
        io::copy(reader, writer)
    }
}

impl<R, W, Fe, Fd> Read for TransportWithTrait<R, W, Fe, Fd>
where
    R: Read,
    W: Write,
    Fe: MyCopyTrait,
    Fd: MyCopyTrait,
{
    fn read(&mut self, buf: &mut [u8]) -> Result<usize> {
        let mut b = io::BufWriter::new(buf);
        Ok(io::copy(&mut self.reader, &mut b)? as usize)
    }
}

impl<R, W, Fe, Fd> Write for TransportWithTrait<R, W, Fe, Fd>
where
    R: Read,
    W: Write,
    Fe: MyCopyTrait,
    Fd: MyCopyTrait,
{
    fn write(&mut self, buf: &[u8]) -> Result<usize> {
        let mut b = io::BufReader::new(buf);
        Ok(io::copy(&mut b, &mut self.writer)? as usize)
    }

    fn flush(&mut self) -> io::Result<()> {
        self.writer.flush()
    }
}

#[cfg(test)]
mod test_trait {

    #[test]
    fn build() {
        assert!(1 == 1)
    }
}
