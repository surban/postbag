#![warn(missing_docs)]
#![cfg_attr(not(doctest), doc = include_str!("../README.md"))]
#![cfg_attr(docsrs, feature(doc_cfg))]

mod cfg;
mod de;
mod error;
pub mod fixint;
mod ser;
mod varint;

pub use cfg::Cfg;
pub use de::deserializer::Deserializer;
pub use de::from_io;
pub use de::from_slice;
pub use error::{Error, Result};
pub use ser::serializer::Serializer;
pub use ser::{to_io, to_vec};
