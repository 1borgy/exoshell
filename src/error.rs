use std::io;
use std::result;
use std::time::SystemTimeError;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum Error {
    #[error("serialization failed: {0}")]
    Serialization(#[from] ron::Error),
    #[error("deserialization failed: {0}")]
    Deserialization(#[from] ron::de::SpannedError),
    #[error("io error: {0}")]
    Io(io::ErrorKind),
    #[error("path error: {0}")]
    Path(String),
    #[error("invalid system time: {0}")]
    SystemTime(#[from] SystemTimeError),
}
pub type Result<T, E = Error> = result::Result<T, E>;

impl From<io::Error> for Error {
    fn from(value: io::Error) -> Self {
        Self::Io(value.kind())
    }
}
