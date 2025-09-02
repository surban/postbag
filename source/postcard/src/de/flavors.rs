//! # Deserialization Flavors
//!
//! "Flavors" in `postcard` are used as modifiers to the serialization or deserialization
//! process. Flavors typically modify one or both of the following:
//!
//! 1. The source medium of the deserialization, e.g. whether the data is serialized from a `[u8]` slice, or some other container
//! 2. The format of the deserialization, such as if the original data is encoded in a COBS format, contains a CRC32 checksum
//!    appended to the message, etc.
//!
//! Flavors are implemented using the [`Flavor`] trait, which acts as a "middleware" for retrieving the bytes before they
//! are passed to `serde` for deserialization
//!
//! Multiple flavors may be combined to obtain a desired combination of behavior and storage.
//! When flavors are combined, it is expected that the storage flavor (such as [`Slice`]) is the innermost flavor.
//!
//! Custom flavors may be defined by users of the `postcard` crate, however some commonly useful flavors have been provided in
//! this module. If you think your custom flavor would be useful to others, PRs adding flavors are very welcome!
//!
//! ## Usability
//!
//! Flavors may not always be convenient to use directly, as they may expose some implementation details of how the
//! inner workings of the flavor behaves. It is typical to provide a convenience method for using a flavor, to prevent
//! the user from having to specify generic parameters, setting correct initialization values, or handling the output of
//! the flavor correctly. See `postcard::from_bytes()` for an example of this.
//!
//! ## When to use (multiple) flavors
//!
//! Combining flavors are nice for convenience, as they perform potentially multiple steps of
//! serialization at one time.
//!
//! This can often be more memory efficient, as intermediate buffers are not typically required.
//!
//! ## When NOT to use (multiple) flavors
//!
//! The downside of passing deserialization through multiple steps is that it is typically slower than
//! performing each step serially. Said simply, "cobs decoding while deserializing" is often slower
//! than "cobs decode then deserialize", due to the ability to handle longer "runs" of data in each
//! stage. The downside is that if these stages can not be performed in-place on the buffer, you
//! will need additional buffers for each stage.
//!
//! Additionally, deserializating flavors can be more restrictive or difficult to work with than
//! serialization flavors, as deserialization may require that the deserialized types borrow some
//! portion of the original message.
//!
//! ## Examples
//!
//! ### Using a single flavor
//!
//! In the first example, we use the `Slice` flavor, to retrieve the serialized output from a `[u8]` slice.
//! No other modification is made to the serialization process.
//!
//! ```rust
//! use postcard::{
//!     Deserializer,
//! };
//! use serde::Deserialize;
//! use std::io::Read;
//!
//! #[derive(Deserialize, Debug, PartialEq)]
//! struct Tup(u8, u8, u8);
//!
//! let msg = [0x04, 0x00, 0x04, 0x01, 0x02, 0x03];
//! let mut deserializer = Deserializer::from_bytes(&msg);
//! let t = Tup::deserialize(&mut deserializer).unwrap();
//! assert_eq!(t, Tup(4, 0, 4));
//! let mut reader = deserializer.finalize().unwrap();
//! let mut remainder = Vec::new();
//! reader.read_to_end(&mut remainder).unwrap();
//! assert_eq!(&remainder, &[1, 2, 3]);
//! ```

use crate::Result;

/// The deserialization Flavor trait
///
/// This is used as the primary way to decode serialized data from some kind of buffer,
/// or modify that data in a middleware style pattern.
///
/// See the module level docs for an example of how flavors are used.
pub trait Flavor {
    /// The remaining data of this flavor after deserializing has completed.
    ///
    /// Typically, this includes the remaining buffer that was not used for
    /// deserialization, and in cases of more complex flavors, any additional
    /// information that was decoded or otherwise calculated during
    /// the deserialization process.
    type Remainder;

    /// Obtain the next byte for deserialization
    fn pop(&mut self) -> Result<u8>;

    /// Returns the number of bytes remaining in the message, if known.
    ///
    /// # Implementation notes
    ///
    /// It is not enforced that this number is exactly correct.
    /// A flavor may yield less or more bytes than the what is hinted at by
    /// this function.
    ///
    /// `size_hint()` is primarily intended to be used for optimizations such as
    /// reserving space for deserialized items, but must not be trusted to
    /// e.g., omit bounds checks in unsafe code. An incorrect implementation of
    /// `size_hint()` should not lead to memory safety violations.
    ///
    /// That said, the implementation should provide a correct estimation,
    /// because otherwise it would be a violation of the trait’s protocol.
    ///
    /// The default implementation returns `None` which is correct for any flavor.
    fn size_hint(&self) -> Option<usize> {
        None
    }

    /// Attempt to take the next `ct` bytes from the serialized message.
    ///
    /// This variant borrows the data from the input for zero-copy deserialization. If zero-copy
    /// deserialization is not necessary, prefer to use `try_take_n_temp` instead.
    fn try_take_n(&mut self, ct: usize) -> Result<Vec<u8>>;

    // /// Attempt to take the next `ct` bytes from the serialized message.
    // ///
    // /// This variant does not guarantee that the returned value is borrowed from the input, so it
    // /// cannot be used for zero-copy deserialization, but it also avoids needing to potentially
    // /// allocate a data in a temporary buffer.
    // ///
    // /// This variant should be used instead of `try_take_n`
    // /// if zero-copy deserialization is not necessary.
    // ///
    // /// It is only necessary to implement this method if the flavor requires storing data in a
    // /// temporary buffer in order to implement the borrow semantics, e.g. the `std::io::Read`
    // /// flavor.
    // fn try_take_n_temp<'a>(&'a mut self, ct: usize) -> Result<&'a [u8]>
    // where
    //     'de: 'a,
    // {
    //     self.try_take_n(ct)
    // }

    /// Complete the deserialization process.
    ///
    /// This is typically called separately, after the `serde` deserialization
    /// has completed.
    fn finalize(self) -> Result<Self::Remainder>;
}

/// Support for [`std::io`] or `embedded-io` traits
#[cfg(any(
    feature = "embedded-io-04",
    feature = "embedded-io-06",
    feature = "use-std"
))]
pub mod io {
    /// Support for [`std::io`] traits
    #[allow(clippy::module_inception)]
    #[cfg(feature = "use-std")]
    pub mod io {
        use super::super::Flavor;
        use crate::{Error, Result};

        /// Wrapper over a [`std::io::Read`] and a sliding buffer to implement the [Flavor] trait
        pub struct IOReader<T>
        where
            T: std::io::Read,
        {
            reader: T,
        }

        impl<T> IOReader<T>
        where
            T: std::io::Read,
        {
            /// Create a new [`IOReader`] from a reader and a buffer.
            ///
            /// `buff` must have enough space to hold all data read during the deserialisation.
            pub fn new(reader: T) -> Self {
                Self { reader }
            }
        }

        impl<T> Flavor for IOReader<T>
        where
            T: std::io::Read,
        {
            type Remainder = T;

            #[inline]
            fn pop(&mut self) -> Result<u8> {
                let mut val = [0; 1];
                self.reader
                    .read_exact(&mut val)
                    .map_err(|_| Error::DeserializeUnexpectedEnd)?;
                Ok(val[0])
            }

            #[inline]
            fn size_hint(&self) -> Option<usize> {
                None
            }

            #[inline]
            fn try_take_n(&mut self, ct: usize) -> Result<Vec<u8>> {
                let mut buf = vec![0; ct];
                self.reader
                    .read_exact(&mut buf)
                    .map_err(|_| Error::DeserializeUnexpectedEnd)?;
                Ok(buf)
            }

            /// Return the remaining (unused) bytes in the Deserializer
            fn finalize(self) -> Result<T> {
                Ok(self.reader)
            }
        }

        #[cfg(test)]
        mod tests {
            use super::*;

            #[test]
            fn test_pop() {
                let mut reader = IOReader::new(&[0xAA, 0xBB, 0xCC][..]);

                assert_eq!(reader.pop(), Ok(0xAA));
                assert_eq!(reader.pop(), Ok(0xBB));
                assert_eq!(reader.pop(), Ok(0xCC));
                assert_eq!(reader.pop(), Err(Error::DeserializeUnexpectedEnd));
            }

            #[test]
            fn test_try_take_n() {
                let mut reader = IOReader::new(&[0xAA, 0xBB, 0xCC, 0xDD, 0xEE][..]);

                assert_eq!(reader.try_take_n(2).as_deref(), Ok(&[0xAA, 0xBB][..]));
                assert_eq!(reader.try_take_n(2).as_deref(), Ok(&[0xCC, 0xDD][..]));
                assert_eq!(reader.try_take_n(2), Err(Error::DeserializeUnexpectedEnd));
            }
        }
    }
}
