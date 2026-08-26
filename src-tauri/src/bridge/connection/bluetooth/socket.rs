use std::io;

use async_trait::async_trait;
use log::error;
use tokio::net::{
    UnixStream,
    unix::{OwnedReadHalf, OwnedWriteHalf},
};

use crate::bridge::connection::{
    self,
    bin_data::{Header, Layout},
    errors::SocketError,
    interfaces::TSockect,
};

pub struct BluetoothSocket {
    // socket: UnixStream,
    reader: OwnedReadHalf,
    writer: OwnedWriteHalf,
}
impl BluetoothSocket {
    pub fn new(socket: UnixStream) -> Self {
        let (reader, writer) = socket.into_split();
        Self { reader, writer }
    }
}

const MAGIC: u16 = 0x424D;
const SUPPORTED_VERSION: u8 = 0x02;

// Protocol v2
// MAGIC(2) -> version(1) -> length(4)

pub struct Reader {
    reader: OwnedReadHalf,
}
#[async_trait]
impl connection::interfaces::Reader for Reader {
    async fn read(&self) -> Result<Vec<u8>, SocketError> {
        let reader = &self.reader;

        // --- 1. Wait for readability ---
        reader.readable().await.map_err(SocketError::Io)?;

        // --- 2. Read header (7 bytes) ---
        let mut header = [0u8; 7];
        match reader.try_read(&mut header) {
            Ok(0) => {
                error!("Socket disconnected");
                return Err(SocketError::Disconnected);
            }
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                return Err(SocketError::NotReady);
            }
            Err(e) => return Err(SocketError::Io(e)),
            _ => (),
        }
        let header = Header::deserialize(&header);

        // --- 3. Parse and validate header ---
        if header.magic != MAGIC {
            return Err(SocketError::InvalidData(format!("Invalid magic ",)));
        }

        if header.version != SUPPORTED_VERSION {
            return Err(SocketError::InvalidData(format!(
                "Unsupported version: {}",
                header.version
            )));
        }

        // --- 4. Read payload ---
        let mut data = vec![0u8; header.length as usize];
        let mut total = 0;
        while total < data.len() {
            match reader.try_read(&mut data[total..]) {
                Ok(0) => {
                    error!("Connection closed during payload read");
                    return Err(SocketError::Disconnected);
                }
                Ok(n) => total += n,
                Err(e) if e.kind() == io::ErrorKind::WouldBlock => {
                    return Err(SocketError::NotReady);
                }
                Err(e) => return Err(SocketError::Io(e)),
            }
        }
        Ok(data)
    }
}

pub struct Writer {
    writer: OwnedWriteHalf,
}

#[async_trait]
impl connection::interfaces::Writer for Writer {
    async fn send(&self, data: &[u8]) -> Result<(), SocketError> {
        let writer = &self.writer;

        let length = data.len() as u32;
        // let data: &[u8] = &data;

        let header = Header {
            magic: MAGIC,
            version: SUPPORTED_VERSION,
            length,
        };
        let header = header.serialize();

        // --- Build header (big-endian) ---
        let mut buf = Vec::with_capacity(header.len() + data.len());

        buf.extend_from_slice(&header);
        buf.extend_from_slice(data);

        // --- Wait until writable ---
        writer.writable().await.map_err(SocketError::Io)?;

        // --- Try writing once ---
        match writer.try_write(&buf) {
            Ok(0) => {
                error!("Socket disconnected while writing");
                Err(SocketError::Disconnected)
            }
            Ok(n) if n < buf.len() => {
                error!("Partial write: wrote {n} of {} bytes", buf.len());
                Err(SocketError::Other("Incomplete write".to_string()))
            }
            Ok(_) => Ok(()),
            Err(e) if e.kind() == io::ErrorKind::WouldBlock => Err(SocketError::NotReady),
            Err(e) => Err(SocketError::Io(e)),
        }
    }
}

impl TSockect for BluetoothSocket {
    type Reader = Reader;
    type Writer = Writer;

    async fn split(self) -> (Self::Reader, Self::Writer) {
        let reader = Reader {
            reader: self.reader,
        };
        let writer = Writer {
            writer: self.writer,
        };
        (reader, writer)
    }
}
