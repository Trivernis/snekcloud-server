use vented::result::VentedError;
use std::fmt;
use std::error::Error;

pub type SnekcloudResult<T> = Result<T, SnekcloudError>;

#[derive(Debug)]
pub enum SnekcloudError {
    Vented(VentedError),
}

impl fmt::Display for SnekcloudError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Vented(v) => write!(f, "Vented Error: {}", v)
        }
    }
}

impl Error for SnekcloudError {}

impl From<VentedError> for SnekcloudError {
    fn from(error: VentedError) -> Self {
        Self::Vented(error)
    }
}