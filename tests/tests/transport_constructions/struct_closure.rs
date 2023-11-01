use std::io::{self, Read, Result, Write};

// first shot
// type CopyFnMut = dyn FnMut(&mut dyn Read, &mut dyn Write) -> Result<u64>;

// second shot
// type CopyFnMut = dyn for<'a, 'b> FnMut(&'a mut (dyn Read + 'a), &'b mut (dyn Write + 'b)) -> Result<u64>;

type CopyFnMut =
    dyn for<'a, 'b> FnMut(&'a mut dyn Read, &'b mut dyn Write) -> Result<u64> + 'static;

// ----------------------------- struct wrapping a closure -----------------------------

pub trait TransformSized {
    fn transform<R, W>(&mut self, reader: &mut R, writer: &mut W) -> Result<u64>
    where
        R: Read,
        W: Write;
}

pub trait Transform {
    fn transform<R, W>(&mut self, reader: &mut R, writer: &mut W) -> Result<u64>
    where
        R: Read + ?Sized,
        W: Write + ?Sized;
}

struct Transformer {
    encode: Box<CopyFnMut>,
}

impl Transformer {
    pub fn new() -> Self {
        Transformer::with_fn(|r, w| io::copy(r, w))
    }

    pub fn with_fn<F>(f: F) -> Self
    where
        // // first shot
        // F: FnMut(&mut dyn Read, &mut dyn Write) -> Result<u64> + 'static,

        // // second shot
        // F: for<'a, 'b> FnMut(&'a mut (dyn Read + 'a), &'b mut (dyn Write + 'b)) -> Result<u64> + 'static,

        // copied from error message
        F: for<'a, 'b> FnMut(&'a mut dyn std::io::Read, &'b mut dyn std::io::Write) -> Result<u64>
            + 'static,
    {
        Transformer {
            encode: Box::new(f),
        }
    }

    // pub fn set_fn<F>(mut self, f: F) -> Self
    // where
    //     F: FnMut(&mut dyn Read, &mut dyn Write) -> Result<u64> + 'static,
    // {
    //     self.encode = Box::new(f);
    //     self
    // }
}

impl TransformSized for Transformer {
    fn transform<R, W>(&mut self, reader: &mut R, writer: &mut W) -> Result<u64>
    where
        R: Read,
        W: Write,
    {
        let cp = &mut self.encode;
        cp(reader, writer)
    }
}

#[test]
fn build() {
    let mut t = Transformer::new();
    let mut r = b"hello".as_ref();
    let mut w = Vec::new();

    let res = t.transform(&mut r, &mut w);
    assert!(res.is_ok());
    assert!(w.len() > 0);
}
