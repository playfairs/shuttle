use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
pub enum ShuttleError {
    #[error("IO error: {0}")]
    Io(#[from] io::Error),

    #[error("QUIC error: {0}")]
    QuicError(String),

    #[error("Protocol error: {0}")]
    ProtocolError(String),

    #[error("Transfer error: {0}")]
    TransferError(String),

    #[error("File not found: {0}")]
    FileNotFound(String),

    #[error("Bind error: {0}")]
    BindError(String),

    #[error("Connection error: {0}")]
    ConnectionError(String),

    #[error("File exists: {0}")]
    FileExists(String),

    #[error("Invalid configuration: {0}")]
    InvalidConfig(String),

    #[error("Serialization error: {0}")]
    SerializationError(String),

    #[error("TLS error: {0}")]
    TlsError(String),
}

impl From<quinn::ConnectError> for ShuttleError {
    fn from(e: quinn::ConnectError) -> Self {
        ShuttleError::QuicError(e.to_string())
    }
}

impl From<quinn::ConnectionError> for ShuttleError {
    fn from(e: quinn::ConnectionError) -> Self {
        ShuttleError::QuicError(e.to_string())
    }
}

impl From<rustls::Error> for ShuttleError {
    fn from(e: rustls::Error) -> Self {
        ShuttleError::TlsError(e.to_string())
    }
}

impl From<bincode::Error> for ShuttleError {
    fn from(e: bincode::Error) -> Self {
        ShuttleError::SerializationError(e.to_string())
    }
}

pub type Result<T> = std::result::Result<T, ShuttleError>;
