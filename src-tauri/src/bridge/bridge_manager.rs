use super::bridge::Bridge;

use super::packets::{Advertisement, ConnectionDetails, ErrorPayload, Packet, Payload};
use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::mpsc::UnboundedReceiver;
use tokio::sync::{mpsc, Mutex};

use serde_json::to_string;

use lazy_static::lazy_static;

use crate::bridge::connection::{ConnectionEvent, ConnectionManager, Token};
use crate::bridge::packets::ClipboardPayload;
use crate::bridge::Device;
use crate::event::event_manager::EVENT_MANAGER;
use crate::event::{ClipboardEvent, ConnectedEvent};
use crate::storage::Storage;

pub fn error_packet(context: &str, message: &str, code: &str) -> Packet {
    Packet::new(Payload::Error(ErrorPayload {
        context: String::from(context),
        message: String::from(message),
        code: String::from(code),
    }))
}

pub struct BridgeManager {
    pub bridges: Mutex<HashMap<String, Arc<Mutex<Bridge>>>>,
    capabilities: Vec<String>,
    conn_mngr: Mutex<Option<Arc<Mutex<ConnectionManager>>>>,
}

impl BridgeManager {
    pub fn new() -> Self {
        Self {
            bridges: Mutex::new(HashMap::new()),
            capabilities: vec!["websocket".into(), "text".into()],
            conn_mngr: Mutex::new(None),
        }
    }
    pub async fn setup(self: &Arc<Self>) {
        let bridges = self.load_bridges_from_storage().await;
        *self.bridges.lock().await = bridges;

        *self.conn_mngr.lock().await = Some(ConnectionManager::new().await);

        let this = self.clone();
        let (to_bridge_mngr, from_conn_mngr) = mpsc::unbounded_channel::<ConnectionEvent>();

        tokio::spawn(async move { this.await_connection(from_conn_mngr).await });

        self.conn_mngr
            .lock()
            .await
            .as_ref()
            .unwrap()
            .lock()
            .await
            .run(to_bridge_mngr)
            .await
            .unwrap();
        println!("Are we here");
    }
    async fn load_bridges_from_storage(&self) -> HashMap<String, Arc<Mutex<Bridge>>> {
        let vec_bridges = Storage::get_all::<Bridge>().unwrap();
        let mut bridges: HashMap<String, Arc<Mutex<Bridge>>> = HashMap::new();

        for bridge in vec_bridges.into_iter() {
            bridges.insert(bridge.id.clone(), Arc::new(Mutex::new(bridge)));
        }
        bridges
    }
    async fn get_bridge_from_token(&self, token: Token) -> Option<Arc<Mutex<Bridge>>> {
        let bridges = self.bridges.lock().await;
        for bridge in bridges.values() {
            let b_token = bridge
                .lock()
                .await
                .reconnection_token
                .as_ref()
                .unwrap()
                .clone();
            if *b_token == *token {
                return Some(bridge.clone());
            }
        }
        None
    }

    pub async fn await_connection(&self, mut from_conn_mngr: UnboundedReceiver<ConnectionEvent>) {
        while let Some(event) = from_conn_mngr.recv().await {
            match event {
                ConnectionEvent::NewConnection((device, conn)) => {
                    // self.handle_connection(device, conn).await;
                    let event_manager = EVENT_MANAGER.clone();

                    let bridge = {
                        let mut bridges = self.bridges.lock().await;

                        let id = bridges.len();
                        let id = format!("bridge-{id}");
                        let bridge = Bridge::new(id.clone(), device, conn).await;

                        bridges.insert(id, bridge.clone());
                        bridge
                    };

                    event_manager
                        .on({
                            let bridge = bridge.clone();
                            move |event: ClipboardEvent| {
                                let bridge = bridge.clone();
                                async move {
                                    let bridge = bridge.clone();
                                    let packet =
                                        Packet::new(Payload::Clipboard(ClipboardPayload {
                                            content: event.text,
                                        }));
                                    bridge.lock().await.send(packet).await;
                                }
                            }
                        })
                        .await;

                    event_manager
                        .emit(ConnectedEvent {
                            id: bridge.lock().await.id.clone(),
                        })
                        .await;
                }
                ConnectionEvent::Reconnection(conn) => {
                    let token = conn.lock().await.reconnection_token();
                    let bridge = self.get_bridge_from_token(token).await.unwrap();

                    bridge.lock().await.reconnect(conn).await;
                }
            }
        }
    }

    pub async fn get_bridge(&self, id: &str) -> Option<Arc<Mutex<Bridge>>> {
        self.bridges
            .lock()
            .await
            .get(id)
            .map(|bridge| bridge.clone())
    }

    pub async fn get_advertisement(&self) -> String {
        let advertisement = Advertisement {
            device: Device {
                name: "kyete-desktop".into(),
                device_type: "desktop".into(),
            },
            capabilities: vec![
                "text".to_string(),
                "websocket".to_string(),
                "bluetooth".to_string(),
            ],
            connection_details: ConnectionDetails {
                bluetooth_address: self.get_bluetooth_address().await,
                websocket_url: self.get_ws_url().await,
            },
            protocol_version: "1.0".into(),
        };

        println!("{advertisement:?}");

        to_string(&advertisement).unwrap()
    }
    pub async fn get_bluetooth_address(&self) -> Option<String> {
        self.conn_mngr
            .lock()
            .await
            .as_ref()
            .unwrap()
            .lock()
            .await
            .bluetooth
            .as_ref()
            .map(|bluetooth| bluetooth.address.clone())
    }
    pub async fn get_ws_url(&self) -> Option<String> {
        self.conn_mngr
            .lock()
            .await
            .as_ref()
            .unwrap()
            .lock()
            .await
            .wss
            .as_ref()
            .map(|wss| wss.url.clone())
    }
}

lazy_static! {
    pub static ref BRIDGE_MANAGER: Arc<BridgeManager> = Arc::new(BridgeManager::new());
}
