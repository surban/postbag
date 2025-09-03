#![warn(missing_docs)]
#![cfg_attr(not(doctest), doc = include_str!("../README.md"))]
#![cfg_attr(docsrs, feature(doc_cfg))]

mod cfg;
mod de;
mod error;
pub mod fixint;
mod ser;
mod varint;

pub use cfg::{Cfg, Config, DefaultCfg};
pub use de::{from_io, from_io_with_cfg, from_slice, from_slice_with_cfg};
pub use error::{Error, Result};
pub use ser::{to_io, to_io_with_cfg, to_vec, to_vec_with_cfg};
