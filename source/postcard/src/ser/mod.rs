use serde::Serialize;

use crate::ser::serializer::Serializer;
use crate::{Cfg, cfg::DefaultCfg, error::Result};

pub(crate) mod serializer;
pub(crate) mod skippable;

/// Serialize a `T` to a `std::vec::Vec<u8>` with a configurable serializer configuration.
pub fn to_vec_with_cfg<T, CFG>(value: &T) -> Result<Vec<u8>>
where
    T: Serialize + ?Sized,
    CFG: Cfg,
{
    to_io_with_cfg::<T, Vec<u8>, CFG>(value, std::vec::Vec::new())
}

/// Serialize a `T` to a `std::vec::Vec<u8>`.
pub fn to_vec<T>(value: &T) -> Result<Vec<u8>>
where
    T: Serialize + ?Sized,
{
    to_vec_with_cfg::<T, DefaultCfg>(value)
}

/// Serialize a `T` to a [`std::io::Write`] with a configurable serializer configuration.
pub fn to_io_with_cfg<T, W, CFG>(value: &T, writer: W) -> Result<W>
where
    T: Serialize + ?Sized,
    W: std::io::Write,
    CFG: Cfg,
{
    let mut serializer = Serializer::<W, CFG>::new(writer);
    value.serialize(&mut serializer)?;
    serializer.finalize()
}

/// Serialize a `T` to a [`std::io::Write`],
pub fn to_io<T, W>(value: &T, writer: W) -> Result<W>
where
    T: Serialize + ?Sized,
    W: std::io::Write,
{
    to_io_with_cfg::<T, W, DefaultCfg>(value, writer)
}
