
use async_trait::async_trait;

use crate::bridge::connection::errors::SocketError;

#[async_trait]
pub trait Reader: Send + Sync {
    async fn read(&self) -> Result<Vec<u8>, SocketError>;
}
#[async_trait]
pub trait Writer: Send + Sync {
    async fn send(&self, msg: &[u8]) -> Result<(), SocketError>;
}

pub trait TSockect: Send + Sync {
    type Reader: Reader;
    type Writer: Writer;
    async fn split(self) -> (Self::Reader, Self::Writer);
}


