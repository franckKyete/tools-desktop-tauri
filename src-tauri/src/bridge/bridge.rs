use crate::bridge::connection::{self, Connection, Token};
use crate::event::event_emitter::TEventEmitter;
use crate::event::event_manager::EVENT_MANAGER;
use crate::event::{DisconnectedEvent, MessageUpdateEvent, ReconnectedEvent};
use crate::storage::storable::Storable;
use crate::storage::Storage;

use super::bridge_manager::error_packet;
use super::{
    packets::{Packet, Payload},
    Device,
};
use std::sync::Arc;

use anyhow::anyhow;
use async_trait::async_trait;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use tokio::sync::Mutex;

// use serde::{Deserialize, Serialize};
use serde_json::{from_str, to_string};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct SavedPacket {
    pub packet: Packet,
    pub sent: bool,
    pub outgoing: bool,
    pub store_id: Option<i64>,
}

impl SavedPacket {
    pub fn outgoing(packet: Packet) -> Self {
        Self {
            sent: false,
            outgoing: true,
            packet,
            store_id: None,
        }
    }
    pub fn incoming(packet: Packet) -> Self {
        Self {
            sent: false,
            outgoing: false,
            packet,
            store_id: None,
        }
    }
}
#[async_trait]
impl Storable for SavedPacket {
    fn get_name() -> String {
        "SavedPacket".to_string()
    }

    fn get_store_id(&self) -> Option<i64> {
        self.store_id
    }
    fn serialize(&self) -> Vec<u8> {
        let data = postcard::to_allocvec(self).unwrap();
        data
    }
    fn deserialize(
        _: i64,
        object: &Vec<u8>,
    ) -> Result<Box<Self>, Box<dyn std::error::Error>> {
        let saved_packet = postcard::from_bytes(object);

        match saved_packet {
            Ok(saved_packet) => Ok(Box::new(saved_packet)),
            Err(e) => Err(anyhow!("Deserialize error : {}", e.to_string()).into()),
        }
    }
    async fn save(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        let id = Storage::store(self)?;
        self.store_id = Some(id);
        Ok(())
    }
}

fn serialize_packet_ids<S>(packets: &Vec<SavedPacket>, serializer: S) -> Result<S::Ok, S::Error>
where
    S: Serializer,
{
    let ids: Vec<i64> = packets.iter().map(|p| p.store_id.unwrap()).collect();
    ids.serialize(serializer)
}

fn deserialize_packet_ids<'de, D>(deserializer: D) -> Result<Vec<SavedPacket>, D::Error>
where
    D: Deserializer<'de>,
{
    // Deserialize as Vec<i64> first
    let _ids = Vec::<i64>::deserialize(deserializer)?;

    // ↓ Option A: ignore and return empty
    Ok(Vec::new())

    // ↓ Option B: create placeholder SavedPacket values
    // Ok(ids.into_iter().map(|id| SavedPacket::new_placeholder(id)).collect())
}

#[derive(Serialize, Deserialize, Debug)]
pub struct Bridge {
    pub id: String,
    pub reconnection_token: Option<Token>,
    pub device: Device,
    store_id: Option<i64>,

    #[serde(
        serialize_with = "serialize_packet_ids",
        deserialize_with = "deserialize_packet_ids"
    )]
    pub packets: Vec<SavedPacket>,

    #[serde(skip)]
    pub conn: Option<Arc<Mutex<Connection>>>,
    #[serde(skip)]
    pub connected: bool,
    #[serde(skip)]
    this: Option<Arc<Mutex<Self>>>,
}
#[async_trait]
impl Storable for Bridge {
    fn get_store_id(&self) -> Option<i64> {
        self.store_id
    }

    fn get_name() -> String {
        "Bridge".to_string()
    }
    fn serialize(&self) -> Vec<u8> {
        let data = postcard::to_allocvec(self).unwrap();
        data
    }
    fn deserialize(
        _: i64,
        object: &Vec<u8>,
    ) -> Result<Box<Self>, Box<dyn std::error::Error>> {
        let saved_bridge = postcard::from_bytes(object);

        match saved_bridge {
            Ok(saved_bridge) => Ok(Box::new(saved_bridge)),
            Err(e) => Err(anyhow!("Deserialize error : {}", e.to_string()).into()),
        }
    }
    async fn save(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        for packet in self.packets.iter_mut() {
            packet.save().await.unwrap();
        }
        let id = Storage::store(self)?;

        self.store_id = Some(id);
        Ok(())
    }
}
impl Bridge {
    pub async fn new(id: String, device: Device, conn: Arc<Mutex<Connection>>) -> Arc<Mutex<Self>> {
        let bridge = Arc::new(Mutex::new(Self {
            id,
            store_id: None,
            device: device,
            packets: vec![],
            reconnection_token: None,
            conn: None,
            this: None,
            connected: false,
        }));

        bridge.lock().await.this = Some(bridge.clone());

        bridge.lock().await.connect(conn).await;

        bridge
    }
    // Darkside Fields

    pub fn as_arc(&self) -> Arc<Mutex<Self>> {
        self.this.as_ref().unwrap().clone()
    }
    pub async fn disconnect(&mut self) {
        match &self.conn {
            Some(_) => (),
            None => {
                return;
            }
        };
        self.conn.as_mut().unwrap().lock().await.disconnect().await;
        self.connected = false;
        self.conn = None;
        EVENT_MANAGER
            .emit(DisconnectedEvent {
                id: self.id.clone(),
            })
            .await;
    }
    pub async fn reconnect(&mut self, conn: Arc<Mutex<Connection>>) {
        self.disconnect().await;
        // self.reconnection_token = token;
        self.connect(conn).await;
        EVENT_MANAGER
            .emit(ReconnectedEvent {
                id: self.id.clone(),
            })
            .await;
        self.resend_pending_messages().await;
    }
    pub async fn connect(&mut self, conn: Arc<Mutex<Connection>>) {
        self.conn = Some(conn);
        let bridge = self.as_arc();

        match &self.conn {
            Some(_) => (),
            None => {
                return;
            }
        };
        let conn = &self.conn.as_mut().unwrap().lock().await;
        conn.on({
            let bridge = bridge.clone();
            move |event: connection::NewPacketEvent| {
                let bridge = bridge.clone();
                async move {
                    bridge.lock().await.handle_message(event.packet).await;
                }
            }
        })
        .await;
        conn.on({
            let bridge = bridge.clone();
            move |_: connection::DisconnectedEvent| {
                let bridge = bridge.clone();
                async move {
                    bridge.lock().await.disconnect().await;
                }
            }
        })
        .await;
        // Start listening to new messages

        self.reconnection_token = Some(conn.reconnection_token());
        self.connected = true;
    }

    pub async fn handle_message(&mut self, msg: String) {
        if let Ok(packet) = from_str::<Packet>(&msg) {
            self.packets.push(SavedPacket::incoming(packet.clone()));
            match packet.payload {
                Payload::Ping => {
                    let pong = Packet::new(Payload::Pong);
                    self.send(pong).await;
                    println!("Ping");
                }
                Payload::Text(_) => {
                    EVENT_MANAGER
                        .clone()
                        .emit(MessageUpdateEvent {
                            id: self.id.clone(),
                        })
                        .await;
                }
                _ => {
                    println!("Not handled");
                }
            }
        } else {
            let err = error_packet("None", "Unknown data format received", "invalid-format");
            self.send(err).await;
        }
    }
    pub async fn send(&mut self, packet: Packet) {
        let event_manager = EVENT_MANAGER.clone();

        self.packets.push(SavedPacket::outgoing(packet.clone()));

        let is_msg = match &packet.payload {
            Payload::Text(_) | Payload::Clipboard(_) => true,
            _ => false,
        };

        if is_msg {
            event_manager
                .emit(MessageUpdateEvent {
                    id: self.id.clone(),
                })
                .await;
        }

        if let Some(conn) = &self.conn {
            let packet = to_string(&packet).unwrap();
            conn.lock().await.send(packet).await.unwrap();
        } else {
            return;
        }

        let saved_packet = self.packets.last_mut().unwrap();
        saved_packet.sent = true;
        if is_msg {
            event_manager
                .emit(MessageUpdateEvent {
                    id: self.id.clone(),
                })
                .await;
        }
    }

    async fn resend_pending_messages(&mut self) {
        let conn = if let Some(conn) = &self.conn {
            conn
        } else {
            return;
        };

        let event_manager = EVENT_MANAGER.clone();

        for saved_packet in self.packets.iter_mut() {
            if !saved_packet.outgoing || saved_packet.sent {
                continue;
            }
            match &saved_packet.packet.payload {
                Payload::Text(_) | Payload::Clipboard(_) => {
                    let packet = saved_packet.packet.clone();

                    let packet = to_string(&packet).unwrap();
                    conn.lock().await.send(packet).await.unwrap();

                    saved_packet.sent = true;
                }
                _ => (),
            }
        }
        event_manager
            .emit(MessageUpdateEvent {
                id: self.id.clone(),
            })
            .await;
    }
}
