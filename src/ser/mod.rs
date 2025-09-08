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

/// Serialize a value using the [`Full`](crate::cfg::Full) configuration.
///
/// This is a convenience function equivalent to `serialize::<Full, _, _>(value, writer)`.
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
/// serialize_full(&person, &mut buffer).unwrap();
/// ```
pub fn serialize_full<T, W>(value: &T, writer: W) -> Result<()>
where
    T: Serialize + ?Sized,
    W: std::io::Write,
{
    serialize::<crate::cfg::Full, T, W>(value, writer)
}

/// Serialize a value using the [`Slim`](crate::cfg::Slim) configuration.
///
/// This is a convenience function equivalent to `serialize::<Slim, _, _>(value, writer)`.
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
/// serialize_slim(&person, &mut buffer).unwrap();
/// ```
pub fn serialize_slim<T, W>(value: &T, writer: W) -> Result<()>
where
    T: Serialize + ?Sized,
    W: std::io::Write,
{
    serialize::<crate::cfg::Slim, T, W>(value, writer)
}
