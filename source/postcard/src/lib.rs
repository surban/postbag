//! Remoc codec.

mod cfg;
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

pub use cfg::{Cfg, Config, DefaultCfg};
pub use de::{from_io, from_io_with_cfg, from_slice, from_slice_with_cfg};
pub use error::{Error, Result};
pub use ser::{to_io, to_io_with_cfg, to_vec, to_vec_with_cfg};
