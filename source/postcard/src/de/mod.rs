use serde::de::DeserializeOwned;

pub(crate) mod deserializer;
mod skippable;

use crate::{Cfg, cfg::DefaultCfg, error::Result};
use deserializer::Deserializer;

/// Deserialize a message of type `T` from a byte slice. The unused portion (if any)
/// of the byte slice is not returned.
pub fn from_slice<T>(s: &[u8]) -> Result<T>
where
    T: DeserializeOwned,
{
    let (t, _) = from_io(s)?;
    Ok(t)
}

/// Deserialize a message of type `T` from a byte slice. The unused portion (if any)
/// of the byte slice is not returned.
pub fn from_slice_with_cfg<T, CFG>(s: &[u8]) -> Result<T>
where
    T: DeserializeOwned,
    CFG: Cfg,
{
    let (t, _) = from_io_with_cfg::<T, _, CFG>(s)?;
    Ok(t)
}

/// Deserialize a message of type `T` from a [`std::io::Read`].
pub fn from_io<T, R>(read: R) -> Result<(T, R)>
where
    T: DeserializeOwned,
    R: std::io::Read,
{
    from_io_with_cfg::<T, R, DefaultCfg>(read)
}

/// Deserialize a message of type `T` from a [`std::io::Read`].
pub fn from_io_with_cfg<T, R, CFG>(read: R) -> Result<(T, R)>
where
    T: DeserializeOwned,
    R: std::io::Read,
    CFG: Cfg,
{
    let mut deserializer = Deserializer::<R, CFG>::new(read);
    let t = T::deserialize(&mut deserializer)?;
    Ok((t, deserializer.finalize()))
}
