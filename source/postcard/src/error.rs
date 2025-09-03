use std::fmt::{Display, Formatter};

/// This is the error type used by Postcard
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// This is a feature that postcard will never implement
    WontImplement,
    /// This is a feature that postcard intends to support, but does not yet
    NotYetImplemented,
    /// The serialize buffer is full
    SerializeBufferFull,
    /// The length of a sequence must be known
    SerializeSeqLengthUnknown,
    /// Hit the end of buffer, expected more data
    DeserializeUnexpectedEnd,
    /// Found a varint that didn't terminate. Is the usize too big for this platform?
    DeserializeBadVarint,
    /// Found a bool that wasn't 0 or 1
    DeserializeBadBool,
    /// Found an invalid unicode char
    DeserializeBadChar,
    /// Tried to parse invalid utf-8
    DeserializeBadUtf8,
    /// Found an Option discriminant that wasn't 0 or 1
    DeserializeBadOption,
    /// Found an enum discriminant that was > `u32::MAX`
    DeserializeBadEnum,
    /// The original data was not well encoded
    DeserializeBadEncoding,
    /// Bad CRC while deserializing
    DeserializeBadCrc,
    /// Serde Serialization Error
    SerdeSerCustom(String),
    /// Serde Deserialization Error
    SerdeDeCustom(String),
    /// Error while processing `collect_str` during serialization
    CollectStrError,
    /// I/O error.
    Io(std::io::Error),
}

impl From<std::io::Error> for Error {
    fn from(err: std::io::Error) -> Self {
        Self::Io(err)
    }
}

impl Display for Error {
    fn fmt(&self, f: &mut Formatter<'_>) -> core::fmt::Result {
        use Error::*;
        match self {
            WontImplement => write!(f, "This is a feature that PostCard will never implement"),
            NotYetImplemented => {
                write!(
                    f,
                    "This is a feature that Postcard intends to support, but does not yet"
                )
            }
            SerializeBufferFull => write!(f, "The serialize buffer is full"),
            SerializeSeqLengthUnknown => write!(f, "The length of a sequence must be known"),
            DeserializeUnexpectedEnd => write!(f, "Hit the end of buffer, expected more data"),
            DeserializeBadVarint => {
                write!(
                    f,
                    "Found a varint that didn't terminate. Is the usize too big for this platform?"
                )
            }
            DeserializeBadBool => write!(f, "Found a bool that wasn't 0 or 1"),
            DeserializeBadChar => write!(f, "Found an invalid unicode char"),
            DeserializeBadUtf8 => write!(f, "Tried to parse invalid utf-8"),
            DeserializeBadOption => write!(f, "Found an Option discriminant that wasn't 0 or 1"),
            DeserializeBadEnum => {
                write!(f, "Found an enum discriminant that was > u32::max_value()")
            }
            DeserializeBadEncoding => write!(f, "The original data was not well encoded"),
            DeserializeBadCrc => write!(f, "Bad CRC while deserializing"),
            SerdeSerCustom(msg) => write!(f, "Serde Serialization Error: {msg}"),
            SerdeDeCustom(msg) => write!(f, "Serde Deserialization Error: {msg}"),
            CollectStrError => write!(
                f,
                "Error while processing `collect_str` during serialization"
            ),
            Io(err) => write!(f, "IO error: {err}"),
        }
    }
}

impl serde::ser::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Error::SerdeSerCustom(msg.to_string())
    }
}

impl serde::de::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Error::SerdeDeCustom(msg.to_string())
    }
}

impl serde::ser::StdError for Error {}

/// This is the Result type used by Postcard.
pub type Result<T> = ::core::result::Result<T, Error>;
