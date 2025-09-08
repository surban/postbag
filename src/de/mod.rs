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
/// serialize::<Full, _, _>(&mut buffer, &original).unwrap();
///
/// let deserialized: Person = deserialize::<Full, _, _>(buffer.as_slice()).unwrap();
/// assert_eq!(original, deserialized);
/// ```
pub fn deserialize<CFG, R, T>(read: R) -> Result<T>
where
    CFG: Cfg,
    R: std::io::Read,
    T: DeserializeOwned,
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
/// serialize_full(&mut buffer, &person).unwrap();
///
/// let deserialized: Person = deserialize_full(buffer.as_slice()).unwrap();
/// assert_eq!(person, deserialized);
/// ```
pub fn deserialize_full<R, T>(reader: R) -> Result<T>
where
    R: std::io::Read,
    T: DeserializeOwned,
{
    deserialize::<crate::cfg::Full, R, T>(reader)
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
/// serialize_slim(&mut buffer, &person).unwrap();
///
/// let deserialized: Person = deserialize_slim(buffer.as_slice()).unwrap();
/// assert_eq!(person, deserialized);
/// ```
pub fn deserialize_slim<R, T>(reader: R) -> Result<T>
where
    R: std::io::Read,
    T: DeserializeOwned,
{
    deserialize::<crate::cfg::Slim, R, T>(reader)
}

/// Deserialize a value from a byte slice using the [`Full`](crate::cfg::Full) configuration.
///
/// This is a convenience function that calls `deserialize_full` with the provided byte slice.
/// It deserializes data that includes struct field identifiers and enum variant identifiers as strings.
///
/// # Example
///
/// ```rust
/// use serde::{Serialize, Deserialize};
/// use postbag::{to_full_vec, from_full_slice};
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
/// let bytes = to_full_vec(&person).unwrap();
/// let deserialized: Person = from_full_slice(&bytes).unwrap();
/// assert_eq!(person, deserialized);
/// ```
pub fn from_full_slice<T>(slice: &[u8]) -> Result<T>
where
    T: DeserializeOwned,
{
    deserialize_full(slice)
}

/// Deserialize a value from a byte slice using the [`Slim`](crate::cfg::Slim) configuration.
///
/// This is a convenience function that calls `deserialize_slim` with the provided byte slice.
/// It deserializes data without identifiers, using indices for enum variants.
///
/// # Example
///
/// ```rust
/// use serde::{Serialize, Deserialize};
/// use postbag::{to_slim_vec, from_slim_slice};
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
/// let bytes = to_slim_vec(&person).unwrap();
/// let deserialized: Person = from_slim_slice(&bytes).unwrap();
/// assert_eq!(person, deserialized);
/// ```
pub fn from_slim_slice<T>(slice: &[u8]) -> Result<T>
where
    T: DeserializeOwned,
{
    deserialize_slim(slice)
}
