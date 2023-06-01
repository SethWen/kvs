use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum KvsError {
    /// IO error
    #[error("{0}")]
    Io(#[from] io::Error),

    /// Serialization or deserialization error.
    #[error("{0}")]
    Serde(#[from] serde_json::Error),

    /// Removing non-existent key error.
    #[error("Key not found")]
    KeyNotFound,

    /// Unexpected command type error.
    /// It indicated a corrupted log or a program bug.
    #[error("unexpected command type")]
    UnexpectedCommandType,
}

// impl From<io::Error> for KvsError {
//     fn from(value: io::Error) -> Self {
//         KvsError::Io(value)
//     }
// }

// impl From<serde_json::Error> for KvsError {
//     fn from(value: serde_json::Error) -> Self {
//         KvsError::Serde(value)
//     }
// }

/// Result type for kvs.
pub type Result<T> = std::result::Result<T, KvsError>;

#[test]
pub fn basic() {}
