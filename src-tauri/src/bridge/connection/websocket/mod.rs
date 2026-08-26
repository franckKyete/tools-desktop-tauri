mod socket;

use std::{error::Error, future::Future, net::IpAddr, sync::Arc};

use futures::StreamExt;
// use futures::StreamExt;
use warp::Filter;

pub use socket::WebSocket;

use std::io;
use if_addrs::get_if_addrs;

use crate::bridge::connection::Socket;
pub struct WebSocketServer {
    // new_connection: Callback,
    pub url: String,
}

fn get_ip() -> Result<IpAddr, io::Error> {
    if let Ok(interfaces) = get_if_addrs() {
        for interface in interfaces {
            if !interface.is_loopback() && interface.name == "wlan0" {
                return Ok(interface.ip());
            }
        }
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Unable to find the ip",
        ))
    } else {
        Err(io::Error::new(
            io::ErrorKind::NotFound,
            "Unable to find the ip",
        ))
    }
}

impl WebSocketServer {
    pub async fn init<F, Fut>(new_connection: F) -> Result<Self, Box<dyn Error>>
    where
        F: Fn(Socket ) -> Fut + Send + Sync + 'static,
        Fut: Future<Output = ()> + Send + 'static
    {
        let callback = Arc::new(new_connection);
        tokio::spawn(async move {
            let wa_route = warp::path("ws")
                .and(warp::ws())
                .map(move |ws: warp::ws::Ws| {
                    let callback = callback.clone();
                    ws.on_upgrade(async move |sock| {
                        let (tx, rx) = sock.split();
                        let sock = WebSocket::new(rx, tx);
                        tokio::spawn(async move{
                            callback(Socket::WS(sock)).await;
                        });
                    })
                });
            warp::serve(wa_route).run(([0, 0, 0, 0], 3000)).await;
        });
        Result::Ok(Self {
            url: format!("ws://{}:3000/ws", get_ip().unwrap()), // new_connection: Box::new(new_connection),
        })
    }
}
