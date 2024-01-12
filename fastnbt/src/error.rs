//! Contains the Error and Result type used by the deserializer.
use std::fmt::Display;

/// Various errors that can occur during deserialization.
#[derive(Debug, Clone)]
#[non_exhaustive]
pub enum Error {
    InvalidTag(u8),
    InvalidSize(i32),
    NoRootCompound,
    NonUnicodeString(Vec<u8>),
    UnexpectedTag {
        tag: crate::Tag,
        expected: &'static str,
    },
    UnexpectedList {
        elem_tag: crate::Tag,
        size: i32,
        expected: &'static str,
    },

    UnexpectedEof,
    IoError(std::sync::Arc<std::io::Error>),
    Other(&'static str),
    Custom(String),
    ExpectedListFoundCompount(Option<crate::Tag>),
    BadArrayLength(i32),
}

/// Convenience type for Result.
pub type Result<T> = std::result::Result<T, Error>;

impl std::error::Error for Error {}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Error::InvalidTag(tag) => write!(f, "invalid nbt tag value: {}", tag),
            Error::InvalidSize(size) => write!(f, "invalid nbt list/array size: {}", size),
            Error::NoRootCompound => write!(f, "invalid nbt: no root compound"),
            Error::NonUnicodeString(data) => write!(f, "invalid nbt string: nonunicode: {}", String::from_utf8_lossy(data)),
            Error::UnexpectedEof => write!(f, "eof: unexpectedly ran out of input"),
            Error::IoError(e) => write!(f, "io error: {}", e),
            Error::Other(s) => f.write_str(s),
            Error::Custom(s) => f.write_str(s),

            Error::UnexpectedTag { tag, expected } => {
                write!(f, "expected {}, found {:?}", expected, tag)
            }
            Error::UnexpectedList { elem_tag, size, expected } => {
                write!(f, "expected {}, found [{:?}; {}]", expected, elem_tag, size)
            }
            Error::ExpectedListFoundCompount(current_tag) => {
                write!(f, "expected to be in list, but was in compound {:?}", current_tag)
            }
            Error::BadArrayLength(int) => write!(f, "Couldn't convert array length {} to usize", int),
        }
    }
}

impl serde::de::Error for Error {
    fn custom<T: Display>(msg: T) -> Self {
        Error::Custom(msg.to_string())
    }
}
impl From<std::io::Error> for Error {
    fn from(e: std::io::Error) -> Self {
        Error::IoError(std::sync::Arc::new(e))
    }
}

impl Error {
    pub(crate) fn invalid_tag(tag: u8) -> Error {
        Error::InvalidTag(tag)
    }
    pub(crate) fn invalid_size(size: i32) -> Error {
        Error::InvalidSize(size)
    }
    pub(crate) fn no_root_compound() -> Error {
        Error::NoRootCompound
    }
    pub(crate) fn nonunicode_string(data: &[u8]) -> Error {
        Error::NonUnicodeString(data.to_owned())
    }
    pub(crate) fn unexpected_eof() -> Error {
        Error::UnexpectedEof
    }
    #[deprecated]
    pub(crate) fn bespoke(msg: &'static str) -> Error {
        Error::Other(msg)
    }
    #[deprecated]
    pub(crate) fn custom(msg: String) -> Error {
        Error::Custom(msg)
    }
}
