use std::fmt::{Display, Formatter};

/// Error of serializaton and deserialization operations.
#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    /// deserialize_any is unsupported
    DeserializeAnyUnsupported,
    /// End of block
    EndOfBlock,
    /// Found a varint that didn't terminate
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
    /// Bad length of a sequence or map
    BadLen,
    /// Bad identifier
    BadIdentifier,
    /// Overflow of target usize
    UsizeOverflow,
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
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        use Error::*;
        match self {
            DeserializeAnyUnsupported => write!(f, "deserialize_any is unsupproted"),
            EndOfBlock => write!(f, "end of block"),
            BadVarint => write!(f, "invalid integer"),
            BadBool => write!(f, "invalid bool"),
            BadChar => write!(f, "invalid char"),
            BadString => write!(f, "invalid string"),
            BadOption => write!(f, "invalid option"),
            BadIdentifier => write!(f, "invalid identifier"),
            BadEnum => write!(f, "invalid enum discriminant"),
            BadLen => write!(f, "invalid length"),
            UsizeOverflow => write!(f, "usize overflow"),
            Custom(msg) => write!(f, "serde error: {msg}"),
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

impl std::error::Error for Error {}

/// Result type of serialization and deserialization operations.
pub type Result<T> = std::result::Result<T, Error>;
