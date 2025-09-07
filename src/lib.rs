#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![doc = include_str!("../README.md")]

pub mod cfg;
mod de;
mod error;
pub mod fixint;
mod ser;
mod varint;

const FALSE: u8 = 0;
const TRUE: u8 = 1;

const NONE: u8 = 0;
const SOME: u8 = 1;

const SPECIAL_LEN: usize = 125;
const UNKNOWN_LEN: usize = 0;

const ID_LEN: usize = 64;
const ID_LEN_NAME: usize = ID_LEN + 1;
const ID_COUNT: usize = 60;

pub use de::deserialize;
pub use error::{Error, Result};
pub use ser::serialize;

/// Serialize with identifiers.
///
/// Struct field identifiers and enum variant identifiers are serialized
/// as strings or using numerical identifier encoding.
pub type Full = cfg::StaticCfg<true>;

/// Serialize without identifiers.
///
/// Struct field identifiers are not serialized.
/// Enum variants are serialized using their index.
pub type Slim = cfg::StaticCfg<false>;
