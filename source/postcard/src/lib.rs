#![warn(missing_docs)]
#![cfg_attr(not(doctest), doc = include_str!("../README.md"))]
#![cfg_attr(docsrs, feature(doc_cfg))]

mod de;

mod cfg;
mod error;
pub mod fixint;
mod ser;
mod varint;

pub use de::deserializer::Deserializer;
pub use de::from_bytes;
pub use error::{Error, Result};
pub use ser::serializer::Serializer;

pub use cfg::Cfg;
pub use de::from_io;
pub use ser::{to_io, to_vec};
