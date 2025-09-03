//! Skippable blocks writer.

use std::io::Result;
use std::io::Write;
use std::mem;

use crate::varint::{varint_max, varint_u16};

/// Writer that allows block to be (partially) skipped during reading.
pub struct SkipWrite<W>(SkipStack<W>);

impl<W: Write> SkipWrite<W> {
    /// Creates a new skip writer.
    pub fn new(inner: W) -> Self {
        Self(SkipStack::Base(inner))
    }

    /// Write bytes.
    pub fn write(&mut self, data: &[u8]) -> Result<()> {
        self.0.write(data)
    }

    /// Opens a skippable block.
    ///
    /// Must be paired with a call to [`Self::end_skippable`].
    pub fn start_skippable(&mut self) {
        let this = mem::replace(&mut self.0, SkipStack::Dummy);
        self.0 = SkipStack::SkipBlock(SkipBlock::new(this));
    }

    /// Finishes a skippable block.
    pub fn end_skippable(&mut self) -> Result<()> {
        match mem::replace(&mut self.0, SkipStack::Dummy) {
            SkipStack::Base(_) => panic!("no skip block is open"),
            SkipStack::SkipBlock(sb) => self.0 = sb.finish()?,
            SkipStack::Dummy => unreachable!(),
        }
        Ok(())
    }

    /// Returns the contained writer after flushing it.
    pub fn into_inner(self) -> Result<W> {
        match self.0 {
            SkipStack::Base(mut inner) => {
                inner.flush()?;
                Ok(inner)
            }
            SkipStack::SkipBlock(_) => panic!("at least one skip block is still open"),
            SkipStack::Dummy => unreachable!(),
        }
    }
}

enum SkipStack<W> {
    Base(W),
    SkipBlock(SkipBlock<W>),
    Dummy,
}

impl<W: Write> SkipStack<W> {
    fn write(&mut self, data: &[u8]) -> Result<()> {
        match self {
            Self::Base(inner) => inner.write_all(data),
            Self::SkipBlock(sb) => sb.write(data),
            Self::Dummy => unreachable!(),
        }
    }
}

struct SkipBlock<W> {
    inner: Box<SkipStack<W>>,
    buf: Vec<u8>,
}

impl<W: Write> SkipBlock<W> {
    const MAX_LEN: usize = u16::MAX as usize;

    fn new(inner: SkipStack<W>) -> Self {
        Self {
            inner: Box::new(inner),
            buf: Vec::new(),
        }
    }

    fn write(&mut self, data: &[u8]) -> Result<()> {
        self.buf.extend_from_slice(data);
        self.flush_buf_if_required()?;

        Ok(())
    }

    fn flush_buf_if_required(&mut self) -> Result<()> {
        while self.buf.len() >= Self::MAX_LEN {
            let rem = self.buf.split_off(Self::MAX_LEN);
            self.flush_buf()?;
            self.buf = rem;
        }

        Ok(())
    }

    fn flush_buf(&mut self) -> Result<()> {
        let mut len_buf = [0; varint_max::<u16>()];
        let len_buf = varint_u16(self.buf.len().try_into().unwrap(), &mut len_buf);
        self.inner.write(len_buf)?;

        self.inner.write(&self.buf)
    }

    fn finish(mut self) -> Result<SkipStack<W>> {
        assert_ne!(self.buf.len(), Self::MAX_LEN);

        self.flush_buf()?;
        Ok(*self.inner)
    }
}
