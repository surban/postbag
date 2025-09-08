use deserializer::Deserializer;
use serde::de::DeserializeOwned;

use crate::{cfg::Cfg, error::Result};

pub(crate) mod deserializer;
mod skippable;

/// Deserialize a value of type `T` from a [`std::io::Read`].
///
/// The `CFG` parameter controls the deserialization format and must match the configuration
/// used during serialization. It can be either:
/// - [`Full`](crate::cfg::Full): Expects struct field identifiers and enum variant identifiers as strings
/// - [`Slim`](crate::cfg::Slim): Expects serialized data without identifiers, using indices for enum variants
///
/// # Example
///
/// This example demonstrates a complete round-trip serialization and deserialization:
///
/// ```rust
/// use serde::{Serialize, Deserialize};
/// use postbag::{serialize, deserialize, cfg::Full};
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

/// Deserialize a value using the [`Full`](crate::cfg::Full) configuration.
///
/// This is a convenience function equivalent to `deserialize::<Full, _, _>(reader)`.
/// It expects struct field identifiers and enum variant identifiers as strings.
///
/// # Example
///
/// ```rust
/// use serde::{Serialize, Deserialize};
/// use postbag::{serialize_full, deserialize_full};
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
/// serialize_full(&person, &mut buffer).unwrap();
///
/// let deserialized: Person = deserialize_full(buffer.as_slice()).unwrap();
/// assert_eq!(person, deserialized);
/// ```
pub fn deserialize_full<T, R>(reader: R) -> Result<T>
where
    T: DeserializeOwned,
    R: std::io::Read,
{
    deserialize::<crate::cfg::Full, T, R>(reader)
}

/// Deserialize a value using the [`Slim`](crate::cfg::Slim) configuration.
///
/// This is a convenience function equivalent to `deserialize::<Slim, _, _>(reader)`.
/// It expects serialized data without identifiers, using indices for enum variants.
///
/// # Example
///
/// ```rust
/// use serde::{Serialize, Deserialize};
/// use postbag::{serialize_slim, deserialize_slim};
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
/// serialize_slim(&person, &mut buffer).unwrap();
///
/// let deserialized: Person = deserialize_slim(buffer.as_slice()).unwrap();
/// assert_eq!(person, deserialized);
/// ```
pub fn deserialize_slim<T, R>(reader: R) -> Result<T>
where
    T: DeserializeOwned,
    R: std::io::Read,
{
    deserialize::<crate::cfg::Slim, T, R>(reader)
}
