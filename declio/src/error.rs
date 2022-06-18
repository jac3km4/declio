use std::fmt;

/// Encoding and decoding errors.
// pub struct Error {
//     message: String,
//     source: Option<Box<dyn std::error::Error + Send + Sync + 'static>>,
// }

pub enum Error {
    TagError(Box<dyn std::error::Error + Send + Sync>),
    FieldError(&'static str, Box<dyn std::error::Error + Send + Sync>),
    RemainingBytes(usize),
    UnexpectedLength { expected: usize, received: usize },
    Custom(String),
    Other(Box<dyn std::error::Error + Send + Sync>),
}

impl Error {
    pub fn new<S>(message: S) -> Self
    where
        S: ToString,
    {
        Self::Custom(message.to_string())
    }

    /// Creates a new `Error` with the given error value as the source.
    pub fn wrap<E>(error: E) -> Self
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        Self::Other(error.into())
    }
}

impl fmt::Debug for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // defer to Display
        write!(f, "{}", self)
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Error::TagError(err) => write!(f, "error at enum tag: {err}"),
            Error::FieldError(field, err) => write!(f, "error at {field}: {err}"),
            Error::RemainingBytes(bytes) => write!(f, "{bytes} bytes left in the input"),
            Error::UnexpectedLength { expected, received } => {
                write!(f, "unexpected slice length {received}, expected {expected}")
            }
            Error::Custom(msg) => write!(f, "{msg}"),
            Error::Other(other) => write!(f, "{}", other),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Error::TagError(inner) => Some(inner.as_ref()),
            Error::FieldError(_, inner) => Some(inner.as_ref()),
            Error::RemainingBytes(_) => None,
            Error::UnexpectedLength { .. } => None,
            Error::Custom(_) => None,
            Error::Other(inner) => Some(inner.as_ref()),
        }
    }
}

macro_rules! convert_error {
    ($($t:ty,)*) => {$(
        impl From<$t> for Error {
            fn from(error: $t) -> Self {
                Self::wrap(error)
            }
        }
    )*}
}

convert_error! {
    std::convert::Infallible,
    std::array::TryFromSliceError,
    std::char::CharTryFromError,
    std::char::DecodeUtf16Error,
    std::io::Error,
    std::num::TryFromIntError,
    std::str::Utf8Error,
    std::string::FromUtf8Error,
    std::string::FromUtf16Error,
}
