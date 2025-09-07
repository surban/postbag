use deserializer::Deserializer;
use serde::de::DeserializeOwned;

use crate::{cfg::Cfg, error::Result};

pub(crate) mod deserializer;
mod skippable;

/// Deserialize a value of type `T` from a [`std::io::Read`].
pub fn deserialize<CFG, T, R>(read: R) -> Result<T>
where
    CFG: Cfg,
    T: DeserializeOwned,
    R: std::io::Read,
{
    let mut deserializer = Deserializer::<R, CFG>::new(read);
    let t = T::deserialize(&mut deserializer)?;
    deserializer.finalize();
    Ok(t)
}
