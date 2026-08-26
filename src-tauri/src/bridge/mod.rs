use serde::{Deserialize, Serialize};

pub mod bridge;
pub mod packets;
pub mod connection;
pub mod bridge_manager;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Device {
    pub name: String,
    pub device_type: String,
}


