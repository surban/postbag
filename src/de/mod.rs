use deserializer::Deserializer;
use serde::de::DeserializeOwned;

use crate::{cfg::Cfg, error::Result};

pub(crate) mod deserializer;
mod skippable;

/// Deserialize a value of type `T` from a [`std::io::Read`].
///
/// The `CFG` parameter controls the deserialization format and must match the configuration
/// used during serialization. It can be either:
/// - [`Full`](crate::Full): Expects struct field identifiers and enum variant identifiers as strings
/// - [`Slim`](crate::Slim): Expects serialized data without identifiers, using indices for enum variants
///
/// # Example
///
/// This example demonstrates a complete round-trip serialization and deserialization:
///
/// ```rust
/// use serde::{Serialize, Deserialize};
/// use postbag::{serialize, deserialize, Full};
///
/// #[derive(Serialize, Deserialize, Debug, PartialEq)]
/// struct Person {
///     name: String,
///     age: u32,
/// }
///
/// let original = Person {
///     name: "Alice".to_string(),
///     age: 30,
/// };
///
/// let mut buffer = Vec::new();
/// serialize::<Full, _, _>(&original, &mut buffer).unwrap();
///
/// let deserialized: Person = deserialize::<Full, _, _>(buffer.as_slice()).unwrap();
/// assert_eq!(original, deserialized);
/// ```
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
