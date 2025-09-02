//! # Serialization Flavors
//!
//! "Flavors" in `postcard` are used as modifiers to the serialization or deserialization
//! process. Flavors typically modify one or both of the following:
//!
//! 1. The output medium of the serialization, e.g. whether the data is serialized to a `[u8]` slice, or a `heapless::Vec`.
//! 2. The format of the serialization, such as encoding the serialized output in a COBS format, performing CRC32 checksumming while serializing, etc.
//!
//! Flavors are implemented using the [`Flavor`] trait, which acts as a "middleware" for receiving the bytes as serialized by `serde`.
//! Multiple flavors may be combined to obtain a desired combination of behavior and storage.
//! When flavors are combined, it is expected that the storage flavor (such as `Slice` or `HVec`) is the innermost flavor.
//!
//! Custom flavors may be defined by users of the `postcard` crate, however some commonly useful flavors have been provided in
//! this module. If you think your custom flavor would be useful to others, PRs adding flavors are very welcome!
//!
//! ## Usability
//!
//! Flavors may not always be convenient to use directly, as they may expose some implementation details of how the
//! inner workings of the flavor behaves. It is typical to provide a convenience method for using a flavor, to prevent
//! the user from having to specify generic parameters, setting correct initialization values, or handling the output of
//! the flavor correctly. See `postcard::to_vec()` for an example of this.
//!
//! It is recommended to use the [`serialize_with_flavor()`](../fn.serialize_with_flavor.html) method for serialization. See it's documentation for information
//! regarding usage and generic type parameters.
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
//! The downside of passing serialization through multiple steps is that it is typically slower than
//! performing each step serially. Said simply, "cobs encoding while serializing" is often slower
//! than "serialize then cobs encode", due to the ability to handle longer "runs" of data in each
//! stage. The downside is that if these stages can not be performed in-place on the buffer, you
//! will need additional buffers for each stage.
//!
//! ## Examples
//!
//! ### Using a single flavor
//!
//! In the first example, we use the `Slice` flavor, to store the serialized output into a mutable `[u8]` slice.
//! No other modification is made to the serialization process.
//!
//! ### Using combined flavors
//!
//! In the second example, we mix `Slice` with `Cobs`, to cobs encode the output while
//! the data is serialized. Notice how `Slice` (the storage flavor) is the innermost flavor used.
//!

use crate::error::Result;

/// The serialization Flavor trait
///
/// This is used as the primary way to encode serialized data into some kind of buffer,
/// or modify that data in a middleware style pattern.
///
/// See the module level docs for an example of how flavors are used.
pub trait Flavor {
    /// The `Output` type is what this storage "resolves" to when the serialization is complete,
    /// such as a slice or a Vec of some sort.
    type Output;

    /// Override this method when you want to customize processing
    /// multiple bytes at once, such as copying a slice to the output,
    /// rather than iterating over one byte at a time.
    #[inline]
    fn try_extend(&mut self, data: &[u8]) -> Result<()> {
        data.iter().try_for_each(|d| self.try_push(*d))
    }

    /// Push a single byte to be modified and/or stored.
    fn try_push(&mut self, data: u8) -> Result<()>;

    /// Finalize the serialization process.
    fn finalize(self) -> Result<Self::Output>;
}

/// Support for the [`std::io`] traits
#[cfg(feature = "use-std")]
pub mod io {

    use super::Flavor;
    use crate::{Error, Result};

    /// Wrapper over a [`std::io::Write`] that implements the flavor trait
    pub struct WriteFlavor<T> {
        writer: T,
    }

    impl<T> WriteFlavor<T>
    where
        T: std::io::Write,
    {
        /// Create a new [`Self`] flavor from a given [`std::io::Write`]
        pub fn new(writer: T) -> Self {
            Self { writer }
        }
    }

    impl<T> Flavor for WriteFlavor<T>
    where
        T: std::io::Write,
    {
        type Output = T;

        #[inline(always)]
        fn try_push(&mut self, data: u8) -> Result<()> {
            self.writer
                .write_all(&[data])
                .map_err(|_| Error::SerializeBufferFull)?;
            Ok(())
        }

        #[inline(always)]
        fn try_extend(&mut self, b: &[u8]) -> Result<()> {
            self.writer
                .write_all(b)
                .map_err(|_| Error::SerializeBufferFull)?;
            Ok(())
        }

        fn finalize(mut self) -> Result<Self::Output> {
            self.writer
                .flush()
                .map_err(|_| Error::SerializeBufferFull)?;
            Ok(self.writer)
        }
    }
}
