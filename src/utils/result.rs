use vented::result::VentedError;
use std::fmt;
use std::io;
use std::error::Error;


pub type SnekcloudResult<T> = Result<T, SnekcloudError>;

#[derive(Debug)]
pub enum SnekcloudError {
    Vented(VentedError),
    IoError(io::Error),
    Base64DecodeError(base64::DecodeError),
    TomlDeserializeError(toml::de::Error),
    TomlSerializeError(toml::ser::Error),
    InvalidKey,
}

impl fmt::Display for SnekcloudError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Vented(v) => write!(f, "Vented Error: {}", v),
            Self::IoError(e) => write!(f, "IO Error: {}", e),
            Self::Base64DecodeError(e) => write!(f, "Base 64 Decode error: {}", e),
            Self::InvalidKey => write!(f, "Invalid Key!"),
            Self::TomlDeserializeError(e) => write!(f, "Toml Deserialization Error: {}", e),
            Self::TomlSerializeError(e) => write!(f, "Toml Serialization Error: {}", e),
        }
    }
}

impl Error for SnekcloudError {}

impl From<VentedError> for SnekcloudError {
    fn from(error: VentedError) -> Self {
        Self::Vented(error)
    }
}

impl From<io::Error> for SnekcloudError {
    fn from(error: io::Error) -> Self {
        Self::IoError(error)
    }
}

impl From<base64::DecodeError> for SnekcloudError {
    fn from(error: base64::DecodeError) -> Self {
        Self::Base64DecodeError(error)
    }
}

impl From<toml::ser::Error> for SnekcloudError {
    fn from(other: toml::ser::Error) -> Self {
        Self::TomlSerializeError(other)
    }
}

impl From<toml::de::Error> for SnekcloudError {
    fn from(other: toml::de::Error) -> Self {
        Self::TomlDeserializeError(other)
    }
}