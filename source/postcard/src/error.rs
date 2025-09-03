use std::fmt::{Display, Formatter};

/// This is the error type used by Postcard
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// This is a feature that postcard will never implement
    DeserializeAnyUnsupported,
    /// The length of a sequence must be known
    SerializeSeqLengthUnknown,
    /// Hit the end of a skip block, expected more data
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
    /// Overflow of target usize.
    UsizeOverflow,
    /// Error while processing `collect_str` during serialization
    CollectStrError,
    /// Serde custom error
    Custom(String),
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
            DeserializeAnyUnsupported => write!(f, "deserialize_any is unsupproted"),
            SerializeSeqLengthUnknown => write!(f, "The length of a sequence must be known"),
            DeserializeUnexpectedEnd => write!(f, "unexpected end of skip block"),
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
            UsizeOverflow => write!(f, "usize overflow"),
            Custom(msg) => write!(f, "serde error: {msg}"),
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
        Error::Custom(msg.to_string())
    }
}

impl serde::de::Error for Error {
    fn custom<T>(msg: T) -> Self
    where
        T: Display,
    {
        Error::Custom(msg.to_string())
    }
}

impl serde::ser::StdError for Error {}

/// This is the Result type used by Postcard.
pub type Result<T> = ::core::result::Result<T, Error>;
