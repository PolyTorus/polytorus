use actix::prelude::*;
use actix_web::{web, HttpRequest, Responder};
use actix_web_actors::ws;
use crate::blockchain::chain::{self, Chain};
use std::sync::{Arc, Mutex};

#[derive(Debug, Clone)]
pub struct P2p {
    chain: Arc<Mutex<Chain>>,
    sockets: Vec<actix::Addr<WebSocketSession>>,
}

#[derive(Debug, Clone)]
pub struct WebSocketSession {
    server: Arc<Mutex<P2p>>
}

impl Actor for WebSocketSession {
    type Context = ws::WebsocketContext<Self>;
}

impl WebSocketSession {
    pub fn message_handler(&self, text: String, ctx: &mut ws::WebsocketContext<Self>) {
        match serde_json::from_str::<Chain>(&text) {
            Ok(received_chain) => {
                println!("Received chain: {:?}", received_chain);
                let server = self.server.lock().unwrap();
                server.chain.lock().unwrap().replace_chain(&received_chain);
            }
            Err(e) => {
                println!("Error: {}", e);
            }
        }
    }
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

#[derive(Message)]
#[rtype(result = "()")]
struct SendChain(String);

impl Handler<SendChain> for WebSocketSession {
    type Result = ();

    fn handle(&mut self, msg: SendChain, ctx: &mut Self::Context) {
        ctx.text(msg.0);
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
        let addr_clone = addr.clone();
        self.sockets.push(addr);
        println!("Connected sockets: {}", self.sockets.len());

        let chain = self.chain.lock().unwrap().clone();
        let chain_json = serde_json::to_string(&chain).unwrap();

        addr_clone.do_send(SendChain(chain_json));
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

    pub fn broadcast_chain(&self) {
        let chain = self.chain.lock().unwrap().clone();
        let chain_json = serde_json::to_string(&chain).unwrap();

        for socket in &self.sockets {
            socket.do_send(SendChain(chain_json.clone()));
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