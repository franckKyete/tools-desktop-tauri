use std::io;
use thiserror::Error;

#[derive(Debug, Error)]
pub enum SocketError {
    #[error("Connection closed")]
    Disconnected,

    #[error("I/O error: {0}")]
    Io(#[from] io::Error),

    #[error("Invalid data received: {0}")]
    InvalidData(String),

    #[error("Socket not ready for operation")]
    NotReady,

    #[error("Unexpected error: {0}")]
    Other(String),
}
