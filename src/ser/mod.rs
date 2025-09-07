use serde::Serialize;

use crate::{cfg::Cfg, error::Result, ser::serializer::Serializer};

pub(crate) mod serializer;
pub(crate) mod skippable;

/// Serialize a value of type `T` to a [`std::io::Write`].
///
/// The `CFG` parameter controls the serialization format and can be either:
/// - [`Full`](crate::Full): Serializes struct field identifiers and enum variant identifiers as strings
/// - [`Slim`](crate::Slim): Serializes without identifiers, using indices for enum variants
///
/// # Example
///
/// ```rust
/// use serde::{Serialize, Deserialize};
/// use postbag::{serialize, Full};
///
/// #[derive(Serialize, Deserialize, Debug, PartialEq)]
/// struct Person {
///     name: String,
///     age: u32,
/// }
///
/// let person = Person {
///     name: "Alice".to_string(),
///     age: 30,
/// };
///
/// let mut buffer = Vec::new();
/// serialize::<Full, _, _>(&person, &mut buffer).unwrap();
/// println!("Serialized {} bytes", buffer.len());
/// ```
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
