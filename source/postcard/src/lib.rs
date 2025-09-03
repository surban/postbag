#![warn(missing_docs)]
#![cfg_attr(not(doctest), doc = include_str!("../README.md"))]
#![cfg_attr(docsrs, feature(doc_cfg))]

mod de;

mod error;
pub mod fixint;
mod ser;
mod varint;

pub use de::deserializer::Deserializer;
pub use de::from_bytes;
pub use error::{Error, Result};
pub use ser::serializer::Serializer;

pub use de::from_io;
pub use ser::{to_io, to_vec};

/// Configuration.
#[derive(Debug, Clone, Copy)]
pub struct Cfg {
    /// Encode identifiers.
    pub with_identifiers: bool,
}

impl Cfg {
    /// Default configuration.
    pub const DEFAULT: Cfg = Cfg {
        with_identifiers: true,
    };
}

impl Default for Cfg {
    fn default() -> Self {
        Self::DEFAULT
    }
}

#[cfg(test)]
mod test {

}
