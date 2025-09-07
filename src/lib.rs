#![forbid(unsafe_code)]
#![warn(missing_docs)]

//! Postbag is a [serde] codec.

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

const ID_LEN: usize = 64;
const ID_LEN_NAME: usize = ID_LEN + 1;
const ID_COUNT: usize = 60;

pub use cfg::{Cfg, Config, Full, Slim};
pub use de::deserialize;
pub use error::{Error, Result};
pub use ser::serialize;
