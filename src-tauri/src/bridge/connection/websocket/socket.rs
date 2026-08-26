use async_trait::async_trait;
use futures::stream::{SplitSink, SplitStream};
use log::error;
use tokio::sync::Mutex;
use warp::filters::ws::{Message, WebSocket as WarpWebSocket};

use crate::bridge::connection::{self, errors::SocketError, interfaces::TSockect};

pub struct WebSocket {
    rx: Mutex<SplitStream<WarpWebSocket>>,
    tx: Mutex<SplitSink<WarpWebSocket, Message>>,
}
impl WebSocket {
    pub fn new(rx: SplitStream<WarpWebSocket>, tx: SplitSink<WarpWebSocket, Message>) -> Self {
        Self {
            tx: tx.into(),
            rx: rx.into(),
        }
    }
}

pub struct Reader {
    rx: SplitStream<WarpWebSocket>,
}
pub struct Writer {
    tx: SplitSink<WarpWebSocket, Message>,
}

#[async_trait]
impl connection::interfaces::Reader for Reader {
    async fn read(&self) -> Result<Vec<u8>, SocketError> {
        todo!()
    }
}
#[async_trait]
impl connection::interfaces::Writer for Writer {
    
    async fn send(&self, msg: &[u8]) -> Result<(), SocketError> {
        // match self.tx.lock().await.send(Message::text(msg)).await {
        //     Ok(_) => Ok(()),
        //     Err(e) => {
        //         error!("Failed to write to the socket : {}", e);
        //         Err(SocketError::Other("WebSocket error".to_string() ))
        //     }
        // }
        Ok(())
    }
}

impl TSockect for WebSocket {
    type Reader = Reader;
    type Writer = Writer;
    async fn split(self) -> (Self::Reader, Self::Writer) {
        todo!()
    }

    // async fn read(&self) -> Result<Vec<u8>, SocketError> {
    //     //     if let Some(result) = self.rx.lock().await.next().await {
    //     //         match result {
    //     //             Ok(msg) => {
    //     //                 // handle valid message
    //     //                 if msg.is_text() {
    //     //                     match msg.to_str() {
    //     //                         Ok(text) => Ok(String::from(text)),
    //     //                         Err(_) => {
    //     //                             error!("Failed to get string msg");
    //     //                             Err(SocketError::InvalidData("No string message".to_string()))
    //     //                         }
    //     //                     }
    //     //                 } else if msg.is_binary() {
    //     //                     match String::from_utf8(msg.as_bytes().to_vec()) {
    //     //                         Ok(text) => Ok(text),
    //     //                         Err(e) => {
    //     //                             error!("Unable to parse message as a utf8 string : {e}");
    //     //                             Err(SocketError::InvalidData("Unable to parse as utf8 : {e}".to_string()))
    //     //                         }
    //     //                     }
    //     //                 } else {
    //     //                     panic!("I've no idea what to do");
    //     //                 }
    //     //             }
    //     //             Err(e) => {
    //     //                 error!("WebSocket error: {}", e);
    //     //                 // handle the error (e.g., disconnect client)
    //     //                 Err(SocketError::Other("WebSocket error".to_string()))
    //     //             }
    //     //         }
    //     //     } else {
    //     //         Err(SocketError::Disconnected)
    //     //     }
    //     Ok(vec![])
    // }
}
