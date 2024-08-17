use actix::prelude::*;
use actix_web::{web, HttpRequest, Responder};
use actix_web_actors::ws;
use crate::blockchain::chain::Chain;
use std::sync::{Arc, Mutex};

pub struct P2p {
    chain: Arc<Mutex<Chain>>,
    sockets: Vec<actix::Addr<WebSocketSession>>,
}

pub struct WebSocketSession {
    server: Arc<Mutex<P2p>>
}

impl Actor for WebSocketSession {
    type Context = ws::WebsocketContext<Self>;
}

impl StreamHandler<Result<ws::Message, ws::ProtocolError>> for WebSocketSession {
    fn handle(&mut self, msg: Result<ws::Message, ws::ProtocolError>, ctx: &mut Self::Context) {
        match msg {
            Ok(ws::Message::Ping(msg)) => ctx.pong(&msg),
            Ok(ws::Message::Text(text)) => println!("Received: {}", text),
            _ => (),
        }
    }
}

impl P2p {
    pub fn new(chain: Chain) -> Self {
        P2p {
            chain: Arc::new(Mutex::new(chain)),
            sockets: Vec::new(),
        }
    }

    pub fn connect_socket(&mut self, addr: actix::Addr<WebSocketSession>) {
        self.sockets.push(addr);
        println!("Connected sockets: {}", self.sockets.len());
    }

    pub async fn connect_peers(&self) {
        let peers = std::env::var("PEERS").unwrap_or_default();

        for peer in peers.split(",") {
            if !peer.is_empty() {
                // Connect to peer
                println!("Connecting to peer: {}", peer);
            }
        }
    }
}

pub async fn websocket_route(
    req: actix_web::HttpRequest,
    stream: web::Payload,
    server: web::Data<Arc<Mutex<P2p>>>,
) -> Result<actix_web::HttpResponse, actix_web::Error> {
    let server_clone = server.get_ref().clone();

    ws::start(
        WebSocketSession {
            server: server_clone,
        },
        &req,
        stream,
    )
}