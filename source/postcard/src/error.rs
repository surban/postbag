use std::fmt::{Display, Formatter};

/// This is the error type used by Postcard
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// deserialize_any is unsupported.
    DeserializeAnyUnsupported,
    /// End of block.
    EndOfBlock,
    /// Found a varint that didn't terminate. Is the usize too big for this platform?
    BadVarint,
    /// Found a bool that wasn't 0 or 1
    BadBool,
    /// Found an invalid unicode char
    BadChar,
    /// Tried to parse invalid utf-8
    BadString,
    /// Found an Option discriminant that wasn't 0 or 1
    BadOption,
    /// Found an enum discriminant that was > `u32::MAX`
    BadEnum,
    /// Bad length of a sequence or map.
    BadLen,
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
            EndOfBlock => write!(f, "end of block"),
            BadVarint => write!(f, "invalid integer"),
            BadBool => write!(f, "invalid bool"),
            BadChar => write!(f, "invalid char"),
            BadString => write!(f, "invalid string"),
            BadOption => write!(f, "invalid option"),
            BadEnum => write!(f, "invalid enum discriminant"),
            BadLen => write!(f, "invalid length"),
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
