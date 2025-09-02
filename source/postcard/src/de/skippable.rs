//! Skippable blocks reader.

use core::mem;

use super::flavors::Flavor;
use crate::{
    varint::{max_of_last_byte, varint_max},
    Error, Result,
};

/// Output with skippable blocks.
#[allow(private_interfaces)]
pub enum SkipStack<F: Flavor> {
    Base(F),
    SkipBlock(SkipBlock<F>),
    Dummy,
}

impl<F: Flavor> SkipStack<F> {
    /// Creates a new skip stack.
    pub fn new(inner: F) -> Self {
        Self::Base(inner)
    }

    /// Obtain the next byte for deserialization
    pub fn pop(&mut self) -> Result<u8> {
        Ok(self.try_take_n(1)?[0])
    }

    /// Returns the number of bytes remaining in the message, if known.
    pub fn size_hint(&self) -> Option<usize> {
        match self {
            Self::Base(base) => base.size_hint(),
            Self::SkipBlock(sb) => sb.size_hint(),
            Self::Dummy => unreachable!(),
        }
    }

    /// Attempt to take the next `ct` bytes from the serialized message.
    pub fn try_take_n(&mut self, ct: usize) -> Result<Vec<u8>> {
        match self {
            Self::Base(base) => Ok(base.try_take_n(ct)?),
            Self::SkipBlock(sb) => sb.try_take_n(ct),
            Self::Dummy => unreachable!(),
        }
    }

    /// Opens a skippable block.
    ///
    /// Must be paired with a call to [`Self::end_skippable`].
    pub fn start_skippable(&mut self) {
        let this = mem::replace(self, Self::Dummy);
        *self = Self::SkipBlock(SkipBlock::new(this));
    }

    /// Finishes a skippable block.
    ///
    /// Remaining contents of the block are skipped if not yet read.
    pub fn end_skippable(&mut self) -> Result<()> {
        let this = mem::replace(self, Self::Dummy);
        match this {
            Self::Base(_) => panic!("no skip block is open"),
            Self::SkipBlock(sb) => *self = sb.finish()?,
            Self::Dummy => unreachable!(),
        }
        Ok(())
    }

    /// Returns the contained base flavor.
    pub fn into_inner(self) -> F {
        match self {
            Self::Base(base) => base,
            Self::SkipBlock(_) => panic!("at least one skip block is still open"),
            Self::Dummy => unreachable!(),
        }
    }

    fn try_take_varint_u16(&mut self) -> Result<u16> {
        let mut out = 0;
        for i in 0..varint_max::<u16>() {
            let val = self.pop()?;
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

struct SkipBlock<F: Flavor> {
    inner: Box<SkipStack<F>>,
    remaining: usize,
    has_next_block: bool,
}

impl<F: Flavor> SkipBlock<F> {
    const MAX_LEN: usize = u16::MAX as usize;

    fn new(inner: SkipStack<F>) -> Self {
        Self {
            inner: Box::new(inner),
            remaining: 0,
            has_next_block: true,
        }
    }

    pub fn size_hint(&self) -> Option<usize> {
        self.inner.size_hint()
    }

    fn update_remaining(&mut self) -> Result<()> {
        if self.remaining > 0 || !self.has_next_block {
            return Ok(());
        }

        self.remaining = self.inner.try_take_varint_u16()?.try_into().unwrap();
        self.has_next_block = self.remaining == Self::MAX_LEN;

        Ok(())
    }

    pub fn try_take_n(&mut self, mut ct: usize) -> Result<Vec<u8>> {
        self.update_remaining()?;

        if self.remaining >= ct {
            let buf = self.inner.try_take_n(ct)?;
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
            buf.extend(&self.inner.try_take_n(n)?);
            self.remaining -= n;
            ct -= n;
        }

        Ok(buf)
    }

    pub fn finish(mut self) -> Result<SkipStack<F>> {
        loop {
            self.update_remaining()?;

            if self.remaining > 0 {
                self.inner.try_take_n(self.remaining)?;
                self.remaining = 0;
            } else {
                break;
            }
        }

        Ok(*self.inner)
    }
}
