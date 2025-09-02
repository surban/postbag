//! Skippable blocks writer.

use core::mem;

use super::flavors::Flavor;
use crate::{
    error::Result,
    varint::{varint_max, varint_u16},
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

    /// Push a single byte to be modified and/or stored.
    pub fn try_push(&mut self, data: u8) -> Result<()> {
        self.try_extend(&[data])
    }

    /// Pushes multiple byte to be modified and/or stored.
    pub fn try_extend(&mut self, data: &[u8]) -> Result<()> {
        match self {
            Self::Base(base) => base.try_extend(data),
            Self::SkipBlock(sb) => sb.try_extend(data),
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
}

struct SkipBlock<F: Flavor> {
    inner: Box<SkipStack<F>>,
    buf: Vec<u8>,
}

impl<F: Flavor> SkipBlock<F> {
    const MAX_LEN: usize = u16::MAX as usize;

    fn new(inner: SkipStack<F>) -> Self {
        Self {
            inner: Box::new(inner),
            buf: Vec::new(),
        }
    }

    pub fn try_extend(&mut self, data: &[u8]) -> Result<()> {
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
        self.inner.try_extend(len_buf)?;

        self.inner.try_extend(&self.buf)
    }

    pub fn finish(mut self) -> Result<SkipStack<F>> {
        assert_ne!(self.buf.len(), Self::MAX_LEN);

        self.flush_buf()?;
        Ok(*self.inner)
    }
}
