use std::error::Error;
use std::fmt;
use std::io;
use vented::utils::result::VentedError;

pub type SnekcloudResult<T> = Result<T, SnekcloudError>;

#[derive(Debug)]
pub enum SnekcloudError {
    Vented(VentedError),
    IoError(io::Error),
    Base64DecodeError(base64::DecodeError),
    TomlDeserializeError(toml::de::Error),
    TomlSerializeError(toml::ser::Error),
    JsonError(serde_json::error::Error),
    InvalidKey,
    ConfigError(config::ConfigError),
    GlobPatternError(glob::PatternError),
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
            Self::ConfigError(e) => write!(f, "Config Error: {}", e),
            Self::GlobPatternError(e) => write!(f, "Glob Error {}", e),
            Self::JsonError(e) => write!(f, "JSON Error: {}", e),
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
    fn from(error: toml::ser::Error) -> Self {
        Self::TomlSerializeError(error)
    }
}

impl From<toml::de::Error> for SnekcloudError {
    fn from(error: toml::de::Error) -> Self {
        Self::TomlDeserializeError(error)
    }
}

impl From<config::ConfigError> for SnekcloudError {
    fn from(error: config::ConfigError) -> Self {
        Self::ConfigError(error)
    }
}

impl From<glob::PatternError> for SnekcloudError {
    fn from(error: glob::PatternError) -> Self {
        Self::GlobPatternError(error)
    }
}

impl From<serde_json::error::Error> for SnekcloudError {
    fn from(error: serde_json::error::Error) -> Self {
        Self::JsonError(error)
    }
}
