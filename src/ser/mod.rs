use serde::Serialize;

use crate::{cfg::Cfg, error::Result, ser::serializer::Serializer};

pub(crate) mod serializer;
pub(crate) mod skippable;

/// Serialize a value of type `T` to a [`std::io::Write`].
///
/// The `CFG` parameter controls the serialization format and can be either:
/// - [`Full`](crate::cfg::Full): Serializes struct field identifiers and enum variant identifiers as strings
/// - [`Slim`](crate::cfg::Slim): Serializes without identifiers, using indices for enum variants
///
/// # Example
///
/// ```rust
/// use serde::{Serialize, Deserialize};
/// use postbag::{serialize, cfg::Full};
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
/// serialize::<Full, _, _>(&mut buffer, &person).unwrap();
/// println!("Serialized {} bytes", buffer.len());
/// ```
pub fn serialize<CFG, W, T>(writer: W, value: &T) -> Result<()>
where
    CFG: Cfg,
    W: std::io::Write,
    T: Serialize + ?Sized,
{
    let mut serializer = Serializer::<W, CFG>::new(writer);
    value.serialize(&mut serializer)?;
    serializer.finalize();
    Ok(())
}

/// Serialize a value using the [`Full`](crate::cfg::Full) configuration.
///
/// This is a convenience function equivalent to `serialize::<Full, _, _>(writer, value)`.
/// It serializes struct field identifiers and enum variant identifiers as strings.
///
/// # Example
///
/// ```rust
/// use serde::{Serialize, Deserialize};
/// use postbag::serialize_full;
///
/// #[derive(Serialize, Deserialize)]
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
/// ```
pub fn serialize_full<W, T>(writer: W, value: &T) -> Result<()>
where
    W: std::io::Write,
    T: Serialize + ?Sized,
{
    serialize::<crate::cfg::Full, W, T>(writer, value)
}

/// Serialize a value using the [`Slim`](crate::cfg::Slim) configuration.
///
/// This is a convenience function equivalent to `serialize::<Slim, _, _>(writer, value)`.
/// It serializes without identifiers, using indices for enum variants.
///
/// # Example
///
/// ```rust
/// use serde::{Serialize, Deserialize};
/// use postbag::serialize_slim;
///
/// #[derive(Serialize, Deserialize)]
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
/// ```
pub fn serialize_slim<W, T>(writer: W, value: &T) -> Result<()>
where
    W: std::io::Write,
    T: Serialize + ?Sized,
{
    serialize::<crate::cfg::Slim, W, T>(writer, value)
}

/// Serialize a value using the [`Full`](crate::cfg::Full) configuration and return a `Vec<u8>`.
///
/// This is a convenience function that creates a new `Vec<u8>` and calls `serialize_full` on it.
/// It serializes struct field identifiers and enum variant identifiers as strings.
///
/// # Example
///
/// ```rust
/// use serde::{Serialize, Deserialize};
/// use postbag::to_full_vec;
///
/// #[derive(Serialize, Deserialize)]
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
/// println!("Serialized {} bytes", bytes.len());
/// ```
pub fn to_full_vec<T>(value: &T) -> Result<Vec<u8>>
where
    T: Serialize + ?Sized,
{
    let mut buffer = Vec::new();
    serialize_full(&mut buffer, value)?;
    Ok(buffer)
}

/// Serialize a value using the [`Slim`](crate::cfg::Slim) configuration and return a `Vec<u8>`.
///
/// This is a convenience function that creates a new `Vec<u8>` and calls `serialize_slim` on it.
/// It serializes without identifiers, using indices for enum variants.
///
/// # Example
///
/// ```rust
/// use serde::{Serialize, Deserialize};
/// use postbag::to_slim_vec;
///
/// #[derive(Serialize, Deserialize)]
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
/// println!("Serialized {} bytes", bytes.len());
/// ```
pub fn to_slim_vec<T>(value: &T) -> Result<Vec<u8>>
where
    T: Serialize + ?Sized,
{
    let mut buffer = Vec::new();
    serialize_slim(&mut buffer, value)?;
    Ok(buffer)
}
