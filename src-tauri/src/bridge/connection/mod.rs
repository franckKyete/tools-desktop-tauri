pub mod bin_data;
pub mod bluetooth;
pub mod errors;
pub mod interfaces;
pub mod websocket;

use std::{any::Any, fmt, future::Future, sync::Arc};

use crate::{
    bridge::{
        Device,
        connection::{
            bin_data::{DataType, Handshake, HandshakeAck, Layout, Reconnection, ReconnectionAck},
            errors::SocketError,
            interfaces::{Reader, TSockect, Writer},
        },
    },
    event::event_emitter::{EventEmitter, TEventEmitter},
};
use bluetooth::{Bluetooth, BluetoothSocket};
use rand::Rng;
use tokio::sync::{Mutex, mpsc::UnboundedSender};
use websocket::{WebSocket, WebSocketServer};

use async_trait::async_trait;

pub enum ConnectionType {
    BT,
    WS,
}
pub enum Socket {
    WS(WebSocket),
    BT(BluetoothSocket),
}

// #[derive(Clone, Debug)]
// pub struct ConnectedEvent {
//     pub conn: Arc<Mutex<Connection>>,
// }
#[derive(Clone, Debug)]
pub struct NewPacketEvent {
    pub packet: String,
}
#[derive(Clone, Debug)]
pub struct DisconnectedEvent {
    pub reason: String,
}

const TOKEN_SIZE: usize = 10;
pub type Token = Arc<[u8; TOKEN_SIZE]>;

pub enum ConnectionEvent {
    NewConnection((Device, Arc<Mutex<Connection>>)),
    Reconnection(Arc<Mutex<Connection>>),
}
pub enum ConnectionInfo {
    Handshake(Device),
    Reconnection(Token),
}

pub struct Connection {
    writer: Arc<dyn Writer>,
    reader: Arc<dyn Reader>,

    conn_type: ConnectionType,

    pub connected: bool,
    reconnection_token: Option<Token>,

    event_emitter: EventEmitter,
    this: Option<Arc<Mutex<Self>>>,
}

impl Connection {
    // type ConnectedEvent = ConnectedEvent;
    pub async fn new(socket: Socket) -> Arc<Mutex<Self>> {
        let (reader, writer, conn_type): (Arc<dyn Reader>, Arc<dyn Writer>, ConnectionType) =
            match socket {
                Socket::BT(socket) => {
                    let (reader, writer) = socket.split().await;
                    (Arc::new(reader), Arc::new(writer), ConnectionType::BT)
                }
                Socket::WS(socket) => {
                    let (reader, writer) = socket.split().await;
                    (Arc::new(reader), Arc::new(writer), ConnectionType::WS)
                }
            };
        let connection = Arc::new(Mutex::new(Connection {
            writer,
            reader,
            conn_type,
            reconnection_token: None,
            connected: false,
            event_emitter: EventEmitter::new(),
            this: None,
        }));

        connection.lock().await.this = Some(connection.clone());

        connection
    }

    pub async fn as_arc(&self) -> Arc<Mutex<Self>> {
        self.this.as_ref().unwrap().clone()
    }

    // async fn connected(&mut self, device: Device) {
    //     self.device = Some(device);
    //     self.emit(ConnectedEvent {
    //         conn: self.as_arc().await,
    //     })
    //     .await;
    // }

    pub async fn send(&self, msg: String) -> Result<(), SocketError> {
        let data = msg.into_bytes();
        self._send(&data, DataType::Packet).await
    }

    pub async fn _send(&self, data: &[u8], r#type: DataType) -> Result<(), SocketError> {
        let mut buf = Vec::with_capacity(data.len() + 1);

        buf.push(r#type as u8);
        buf.extend_from_slice(data);

        self.writer.send(&buf).await
    }
    async fn run_reader(&self) {
        let conn = self.as_arc().await;

        let reader = self.reader.clone();
        tokio::spawn({
            let conn = conn.clone();
            // let reader = reader.clone();
            async move {
                loop {
                    let data = match reader.read().await {
                        Ok(data) => data,
                        Err(e) => match e {
                            SocketError::Io(_)
                            | SocketError::Disconnected
                            | SocketError::Other(_) => {
                                conn.lock().await.connected = false;
                                conn.lock()
                                    .await
                                    .emit(DisconnectedEvent {
                                        reason: String::from(
                                            "Well I don't know, something must have happened",
                                        ),
                                    })
                                    .await;
                                break;
                            }
                            SocketError::InvalidData(_) => continue,
                            SocketError::NotReady => continue,
                        },
                    };
                    // -> type(1) ->
                    //      Disconnection =>
                    //          null
                    //      Packet =>
                    //          size(4) => data(size)

                    // let data: &[u8] = &data;
                    let r#type = data[0];

                    match DataType::from(r#type) {
                        DataType::Disconnection => {}
                        DataType::Packet => {
                            let packet = str::from_utf8(&data[1..]).unwrap().to_string();
                            conn.lock().await.emit(NewPacketEvent { packet }).await;
                        }
                        _ => (),
                    }
                }
            }
        });
    }
    // This should return either a token(reconnection) or a device(handshake)
    async fn await_handshake(&mut self) -> Result<ConnectionInfo, Box<dyn std::error::Error>> {
        let reader = self.reader.clone();
        self.connected = true;
        loop {
            let data = match reader.read().await {
                Ok(data) => data,
                Err(e) => match e {
                    SocketError::Io(_) | SocketError::Disconnected | SocketError::Other(_) => {
                        return Err(anyhow::anyhow!("Error reading socket").into());
                    }
                    SocketError::InvalidData(_) => continue,
                    SocketError::NotReady => continue,
                },
            };
            // -> type(1) ->
            //      Handshake =>
            //          device_type(1) -> device_name_size(2) -> device_name(device_name_size)
            //      HandshakeAck =>
            //          accepted(bool) -> reconnection_token(10)
            //      Disconnection =>
            //          null

            // let data: &[u8] = &data;
            let r#type = data[0];

            match DataType::from(r#type) {
                DataType::Handshake => {
                    let handshake = Handshake::deserialize(&data[1..6]);

                    let device_type = match handshake.device_type {
                        0x01 => "phone",
                        0x02 => "pc",
                        _ => "unknown",
                    }
                    .to_string();

                    let start: usize = 6;

                    let name = &data[start..];
                    let name = str::from_utf8(name).unwrap().to_string();

                    let device = Device { device_type, name };

                    return Ok(ConnectionInfo::Handshake(device));
                }
                //      Reconnection =>
                //          token(10)
                //      ReconnectionAck =>
                //          accepted(bool) -> token(10)
                DataType::Reconnection => {
                    let header = &data[1..11];
                    let reconnection = Reconnection::deserialize(header);

                    let token = Arc::new(reconnection.token);

                    return Ok(ConnectionInfo::Reconnection(token));
                }
                _ => (),
            }
        }
    }

    pub async fn disconnect(&mut self) {
        self.connected = false;
    }

    pub fn reconnection_token(&self) -> Token {
        self.reconnection_token.as_ref().unwrap().clone()
    }
    fn set_reconnection_token(&mut self, token: Token) {
        self.reconnection_token = Some(token);
    }
}

#[async_trait]
impl TEventEmitter for Connection {
    async fn emit<T: Any + Send + Sync + 'static>(&self, event: T) {
        self.event_emitter.emit(event).await
    }
    async fn on<T, F, Fut>(&self, callback: F)
    where
        T: Any + Send + Sync + Clone + 'static,  // Event must be Clone
        F: Fn(T) -> Fut + Send + Sync + 'static, // Callback takes T by value
        Fut: Future<Output = ()> + Send + 'static,
    {
        self.event_emitter.on(callback).await
    }
}

pub struct ConnectionManager {
    pub bluetooth: Option<Bluetooth>,
    pub wss: Option<WebSocketServer>,
    pub tokens: Vec<Token>,
    pub this: Option<Arc<Mutex<Self>>>,
}

impl ConnectionManager {
    pub async fn new() -> Arc<Mutex<Self>> {
        let conn_mngr = Arc::new(Mutex::new(Self {
            bluetooth: None,
            wss: None,
            tokens: vec![],
            this: None,
        }));

        conn_mngr.lock().await.this = Some(conn_mngr.clone());

        conn_mngr
    }
    pub async fn as_arc(&self) -> Arc<Mutex<Self>> {
        self.this.as_ref().unwrap().clone()
    }

    pub fn find_and_remove_token(&mut self, token: &Token) -> bool {
        if let Some(pos) = self.tokens.iter().position(|t| **t == **token) {
            self.tokens.remove(pos);
            true
        } else {
            false
        }
    }

    pub async fn run(
        &mut self,
        to_bridge_mngr: UnboundedSender<ConnectionEvent>,
    ) -> Result<(), Box<dyn std::error::Error>> {
        let this = self.as_arc().await;
        let new_socket_callback = move |socket| {
            let to_bridge_mngr = to_bridge_mngr.clone();
            let this = this.clone();
            async move {
                let connection = Connection::new(socket).await;
                let mut this_lock = this.lock().await;

                // This should return stuff like Reconnection or Handshake, token and device
                let connection_details = connection.lock().await.await_handshake().await.unwrap();
                match connection_details {
                    ConnectionInfo::Handshake(device) => {
                        let token: Token = Arc::new(rand::rng().random());
                        connection
                            .lock()
                            .await
                            .set_reconnection_token(token.clone());

                        this_lock.tokens.push(token.clone());

                        let ack = HandshakeAck {
                            accepted: 0x01,
                            token: *token,
                        };
                        let ack = ack.serialize();

                        connection
                            .lock()
                            .await
                            ._send(&ack, DataType::HandshakeAck)
                            .await
                            .unwrap();

                        to_bridge_mngr
                            .send(ConnectionEvent::NewConnection((device, connection.clone())))
                            .unwrap();
                    }
                    ConnectionInfo::Reconnection(token) => {
                        let mut ack = ReconnectionAck {
                            accepted: 0x00,
                            token: [0u8; TOKEN_SIZE],
                        };

                        let found = this_lock.find_and_remove_token(&token);
                        if found {
                            let token: Token = Arc::new(rand::rng().random());
                            connection
                                .lock()
                                .await
                                .set_reconnection_token(token.clone());
                            this_lock.tokens.push(token.clone());

                            ack.accepted = 0x01;
                            ack.token = *token;

                            to_bridge_mngr
                                .send(ConnectionEvent::Reconnection(connection.clone()))
                                .unwrap();
                        }
                        let ack = ack.serialize();
                        connection
                            .lock()
                            .await
                            ._send(&ack, DataType::ReconnectionAck)
                            .await
                            .unwrap();
                    }
                }

                connection.lock().await.run_reader().await;
            }
        };

        if Bluetooth::available().await? {
            self.bluetooth = Some(Bluetooth::init(new_socket_callback).await?);
        } else {
            self.wss = Some(WebSocketServer::init(new_socket_callback).await?);
        }
        Ok(())
    }
}

impl fmt::Debug for Connection {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("Connection")
            .field(
                "connected",
                &if self.connected {
                    "Some(WebSocket)"
                } else {
                    "None"
                },
            )
            .finish()
    }
}
