//! # Fixed Size Integers
//!
//! In some cases, the use of variably length encoded data may not be
//! preferable. These modules, for use with `#[serde(with = "postcard::fixint")]`
//! "opt out" of variable length encoding.
//!
//! Disables varint serialization/deserialization for the specified integer
//! field. The integer will always be serialized in the same way as a fixed
//! size array.
//! 
//! Support explicitly not provided for `usize` or `isize`, as
//! these types would not be portable between systems of different
//! pointer widths.
//!
//! ```rust
//! # use serde::Serialize;
//! #[derive(Serialize)]
//! pub struct DefinitelyFixInt {
//!     #[serde(with = "postcard::fixint")]
//!     x: u16,
//! }
//! ```

use serde::Deserializer;
use serde::{Deserialize, Serialize, Serializer};

/// Serialize the integer value as a fixed-size array.
pub fn serialize<S, T>(val: &T, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: Copy,
    LE<T>: Serialize,
{
    LE(*val).serialize(serializer)
}

/// Deserialize the integer value from a fixed-size array.
pub fn deserialize<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    LE<T>: Deserialize<'de>,
{
    LE::<T>::deserialize(deserializer).map(|x| x.0)
}

#[doc(hidden)]
pub struct LE<T>(T);

macro_rules! impl_fixint {
    ($( $int:ty ),*) => {
        $(
            impl Serialize for LE<$int> {

                fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where
                    S: Serializer,
                {
                    self.0.to_le_bytes().serialize(serializer)
                }
            }

            impl<'de> Deserialize<'de> for LE<$int> {

                fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
                where
                    D: serde::Deserializer<'de>,
                {
                    <_ as Deserialize>::deserialize(deserializer)
                        .map(<$int>::from_le_bytes)
                        .map(Self)
                }
            }
        )*
    };
}

impl_fixint![i16, i32, i64, i128, u16, u32, u64, u128];
