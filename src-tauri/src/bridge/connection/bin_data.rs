#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DataType {
    Handshake = 0x01,
    Reconnection = 0x02,
    HandshakeAck = 0x03,
    ReconnectionAck = 0x04,
    Disconnection = 0x05,
    Packet = 0x06,
    FileChunk = 0x07,
    AutomergeSync = 0x08,
    ChunkAck = 0x09,
    Unknown,
}

impl From<u8> for DataType {
    fn from(value: u8) -> Self {
        match value {
            0x00 => Self::Handshake,
            0x01 => Self::Handshake,
            0x02 => Self::Reconnection,
            0x03 => Self::HandshakeAck,
            0x04 => Self::ReconnectionAck,
            0x05 => Self::Disconnection,
            0x06 => Self::Packet,
            0x07 => Self::FileChunk,
            0x08 => Self::AutomergeSync,
            0x09 => Self::ChunkAck,
            _ => Self::Unknown,
        }
    }
}

use bytemuck::{Pod, Zeroable, bytes_of, from_bytes};

pub trait Layout: Copy + Clone + Pod + Zeroable {
    fn serialize(&self) -> &[u8] {
        let bytes: &[u8] = bytes_of(self);
        bytes
    }
    fn deserialize(header: &[u8]) -> Self {
        let header: &Self = from_bytes(header);
        *header
    }
}

#[derive(Copy, Clone, Pod, Zeroable, Debug)]
#[repr(C, packed)]
pub struct Header {
    pub magic: u16,
    pub version: u8,
    pub length: u32,
}
impl Layout for Header {}

#[derive(Copy, Clone, Pod, Zeroable, Debug)]
#[repr(C, packed)]
pub struct Handshake {
    pub device_type: u8,
    pub device_name_size: u32,
}
impl Layout for Handshake {}

#[derive(Copy, Clone, Pod, Zeroable, Debug)]
#[repr(C, packed)]
pub struct HandshakeAck {
    pub accepted: u8,
    pub token: [u8; 10],
}
impl Layout for HandshakeAck {}

#[derive(Copy, Clone, Pod, Zeroable, Debug)]
#[repr(C, packed)]
pub struct Reconnection {
    pub token: [u8; 10],
}
impl Layout for Reconnection {}

#[derive(Copy, Clone, Pod, Zeroable, Debug)]
#[repr(C, packed)]
pub struct ReconnectionAck {
    pub accepted: u8,
    pub token: [u8; 10],
}
impl Layout for ReconnectionAck {}

pub const CHUNK_FLAG_EOF: u8 = 0x01;
pub const CHUNK_FLAG_ACK_REQ: u8 = 0x02;
pub const CHUNK_FLAG_RESUMED: u8 = 0x04;

pub const CHUNK_STATUS_OK: u8 = 0x00;
pub const CHUNK_STATUS_CRC_FAIL: u8 = 0x01;
pub const CHUNK_STATUS_IO_ERROR: u8 = 0x02;

#[derive(Copy, Clone, Pod, Zeroable, Debug, PartialEq, Eq)]
#[repr(C, packed)]
pub struct FileChunkHeader {
    pub transfer_id: [u8; 16],
    pub chunk_index: u32,
    pub total_chunks: u32,
    pub byte_offset: u64,
    pub payload_len: u32,
    pub flags: u8,
    pub checksum: u32,
}
impl Layout for FileChunkHeader {}

#[derive(Copy, Clone, Pod, Zeroable, Debug, PartialEq, Eq)]
#[repr(C, packed)]
pub struct ChunkAckHeader {
    pub transfer_id: [u8; 16],
    pub cumulative_chunk_index: u32,
    pub window_credit: u32,
    pub status: u8,
    pub nack_chunk_index: u32,
}
impl Layout for ChunkAckHeader {}

