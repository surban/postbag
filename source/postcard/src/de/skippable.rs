//! Skippable blocks reader.

use std::io::Read;
use std::mem;

use crate::{
    Error, Result,
    varint::{max_of_last_byte, varint_max},
};

/// Reader that allows blocks to be (partially) skipped.
pub struct SkipRead<R>(SkipStack<R>);

impl<R: Read> SkipRead<R> {
    /// Creates a new skip stack.
    pub fn new(inner: R) -> Self {
        SkipRead(SkipStack::Base(inner))
    }

    /// Read one byte.
    pub fn pop(&mut self) -> Result<u8> {
        Ok(self.read(1)?[0])
    }

    /// Read `cnt` bytes.
    pub fn read(&mut self, cnt: usize) -> Result<Vec<u8>> {
        self.0.read(cnt)
    }

    /// Opens a skippable block.
    ///
    /// Must be paired with a call to [`Self::end_skippable`].
    pub fn start_skippable(&mut self) {
        let this = mem::replace(&mut self.0, SkipStack::Dummy);
        self.0 = SkipStack::SkipBlock(SkipBlock::new(this));
    }

    /// Finishes a skippable block.
    ///
    /// Remaining contents of the block are skipped if not yet read.
    pub fn end_skippable(&mut self) -> Result<()> {
        match mem::replace(&mut self.0, SkipStack::Dummy) {
            SkipStack::Base(_) => panic!("no skip block is open"),
            SkipStack::SkipBlock(sb) => self.0 = sb.finish()?,
            SkipStack::Dummy => unreachable!(),
        }
        Ok(())
    }

    /// Returns the reader.
    pub fn into_inner(self) -> R {
        match self.0 {
            SkipStack::Base(base) => base,
            SkipStack::SkipBlock(_) => panic!("at least one skip block is still open"),
            SkipStack::Dummy => unreachable!(),
        }
    }
}

enum SkipStack<R> {
    Base(R),
    SkipBlock(SkipBlock<R>),
    Dummy,
}

impl<R: Read> SkipStack<R> {
    pub fn read(&mut self, ct: usize) -> Result<Vec<u8>> {
        match self {
            Self::Base(base) => {
                let mut buf = vec![0; ct];
                base.read_exact(&mut buf)?;
                Ok(buf)
            }
            Self::SkipBlock(sb) => sb.read(ct),
            Self::Dummy => unreachable!(),
        }
    }

    fn try_take_varint_u16(&mut self) -> Result<u16> {
        let mut out = 0;
        for i in 0..varint_max::<u16>() {
            let val = self.read(1)?[0];
            let carry = (val & 0x7F) as u16;
            out |= carry << (7 * i);

            if (val & 0x80) == 0 {
                if i == varint_max::<u16>() - 1 && val > max_of_last_byte::<u16>() {
                    return Err(Error::DeserializeBadVarint);
                } else {
                    return Ok(out);
                }
            }
        }
        Err(Error::DeserializeBadVarint)
    }
}

struct SkipBlock<R> {
    inner: Box<SkipStack<R>>,
    remaining: usize,
    has_next_block: bool,
}

impl<R: Read> SkipBlock<R> {
    const MAX_LEN: usize = u16::MAX as usize;

    fn new(inner: SkipStack<R>) -> Self {
        Self {
            inner: Box::new(inner),
            remaining: 0,
            has_next_block: true,
        }
    }

    fn update_remaining(&mut self) -> Result<()> {
        if self.remaining > 0 || !self.has_next_block {
            return Ok(());
        }

        self.remaining = self.inner.try_take_varint_u16()?.try_into().unwrap();
        self.has_next_block = self.remaining == Self::MAX_LEN;

        Ok(())
    }

    fn read(&mut self, mut ct: usize) -> Result<Vec<u8>> {
        self.update_remaining()?;

        if self.remaining >= ct {
            let buf = self.inner.read(ct)?;
            self.remaining -= ct;
            return Ok(buf);
        }

        let mut buf = Vec::with_capacity(ct);
        while ct > 0 {
            self.update_remaining()?;

            if self.remaining == 0 {
                return Err(Error::DeserializeUnexpectedEnd);
            }

            let n = ct.min(self.remaining);
            buf.extend(&self.inner.read(n)?);
            self.remaining -= n;
            ct -= n;
        }

        Ok(buf)
    }

    fn finish(mut self) -> Result<SkipStack<R>> {
        loop {
            self.update_remaining()?;

            if self.remaining > 0 {
                self.inner.read(self.remaining)?;
                self.remaining = 0;
            } else {
                break;
            }
        }

        Ok(*self.inner)
    }
}
