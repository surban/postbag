use serde::Serialize;

use crate::{Cfg, error::Result, ser::serializer::Serializer};

pub(crate) mod serializer;
pub(crate) mod skippable;

/// Serialize a `T` to a [`std::io::Write`].
pub fn serialize<CFG, T, W>(value: &T, writer: W) -> Result<()>
where
    CFG: Cfg,
    T: Serialize + ?Sized,
    W: std::io::Write,
{
    let mut serializer = Serializer::<W, CFG>::new(writer);
    value.serialize(&mut serializer)?;
    serializer.finalize();
    Ok(())
}
