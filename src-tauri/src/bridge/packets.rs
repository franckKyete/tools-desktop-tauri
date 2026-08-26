use serde::{Deserialize, Serialize};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::bridge::Device;



#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ClipboardPayload {
    // pub format: String,
    pub content: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FileRequestPayload {
    pub file_name: String,
    pub file_size: u32,
    pub mime_type: String,
    pub transfer_mode: String,
    pub transfer_id: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct FileResponsePayload {
    pub transfer_id: String,
    pub accepted: bool,
    pub message: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ErrorPayload {
    pub code: String,
    pub context: String,
    pub message: String,
}

// struct InnerData{
//     z : String
// }
//
// struct Data{
//     x : i64,
//     y : u32,
//     v : Vec<InnerData>
// }

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ConnectionDetails {
    pub bluetooth_address: Option<String>,
    pub websocket_url: Option<String>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Advertisement {
    pub device: Device,
    pub connection_details: ConnectionDetails,
    pub capabilities: Vec<String>,
    pub protocol_version: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TextPayload {
    pub content: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
#[serde(tag = "type", content = "payload")]
pub enum Payload {
    Ping,
    Pong,
    Clipboard(ClipboardPayload),
    FileRequest(FileRequestPayload),
    FileResponse(FileResponsePayload),
    Error(ErrorPayload),
    Text(TextPayload),
}
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Packet {
    pub timestamp: u64,
    #[serde(flatten)]
    pub payload: Payload,
}

impl Packet {
    pub fn new(payload: Payload) -> Self {
        Self {
            payload,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .expect("We traveled back in time")
                .as_secs(),
        }
    }
}


