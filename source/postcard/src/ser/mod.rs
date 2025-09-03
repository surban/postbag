use serde::Serialize;

use crate::error::Result;
use crate::ser::serializer::Serializer;

pub(crate) mod serializer;
pub(crate) mod skippable;

/// Serialize a `T` to a `std::vec::Vec<u8>`.
///
/// ## Example
///
/// ```rust
/// use postcard::to_vec;
///
/// let ser: Vec<u8> = to_vec(&true).unwrap();
/// assert_eq!(ser.as_slice(), &[0x01]);
///
/// let ser: Vec<u8> = to_vec("Hi!").unwrap();
/// assert_eq!(ser.as_slice(), &[0x03, b'H', b'i', b'!']);
/// ```
#[cfg_attr(docsrs, doc(cfg(feature = "use-std")))]

pub fn to_vec<T>(value: &T) -> Result<Vec<u8>>
where
    T: Serialize + ?Sized,
{
    to_io(value, std::vec::Vec::new())
}

/// Serialize a `T` to a [`std::io::Write`],
/// ## Example
///
/// ```rust
/// use postcard::to_io;
/// let mut buf: [u8; 32] = [0; 32];
/// let mut writer: &mut [u8] = &mut buf;
///
/// let ser = to_io(&true, &mut writer).unwrap();
/// to_io("Hi!", ser).unwrap();
/// assert_eq!(&buf[0..5], &[0x01, 0x03, b'H', b'i', b'!']);
/// ```
pub fn to_io<T, W>(value: &T, writer: W) -> Result<W>
where
    T: Serialize + ?Sized,
    W: std::io::Write,
{
    let mut serializer = Serializer::new(writer);
    value.serialize(&mut serializer)?;
    serializer.finalize()
}
