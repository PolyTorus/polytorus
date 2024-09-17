use crate::blockchain::chain::Chain;
use std::sync::Arc;
use futures::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use serde_json::json;

type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;

#[derive(Debug, Clone)]
pub struct P2p {
    chain: Arc<Mutex<Chain>>,
    sockets: Arc<Mutex<Vec<Arc<Mutex<WsStream>>>>>,
}

impl P2p {
    pub fn new(chain: Chain) -> Self {
        P2p {
            chain: Arc::new(Mutex::new(chain)),
            sockets: Arc::new(Mutex::new(Vec::new()))        
        }
    }

    pub async fn listen(&self) -> Result<(), Box<dyn std::error::Error>> {
        let p2p_port = std::env::var("P2P_PORT").unwrap_or_else(|_| "5001".to_string());
        let addr = format!("127.0.0.1:{}", p2p_port);
        let listener = TcpListener::bind(&addr).await?;
        println!("Listening on: {}", addr);

        self.connect_peers().await?;
        
        while let Ok((stream, _)) = listener.accept().await {
            let ws_stream = tokio_tungstenite::accept_async(tokio_tungstenite::MaybeTlsStream::Plain(stream)).await.expect("Failed to accept");
            self.connect_socket(ws_stream).await;
        }

        Ok(())
    }

    pub async fn connect_peers(&self) -> Result<(), Box<dyn std::error::Error>>{
        let peers = std::env::var("PEERS").unwrap_or_default();

        for peer in peers.split(",") {
            if !peer.is_empty() {
                let (ws_stream, _) = connect_async(peer).await?;
                self.connect_socket(ws_stream).await;
            }
        }
        Ok(())
    }

    pub async fn connect_socket(&self, ws_stream: WsStream) {
        let chain = self.chain.clone();
        let sockets = self.sockets.clone();
        let ws_stream = Arc::new(Mutex::new(ws_stream));

        {
            let mut sockets = sockets.lock().await;
            sockets.push(ws_stream.clone());
            println!("Socket connected");
        }

        self.send_chain(ws_stream.clone()).await;
        self.message_handler(ws_stream.clone(), chain.clone()).await;
    }

    async fn send_chain(&self, ws_stream: Arc<Mutex<WsStream>>) {
        let chain_json = {
            let chain = self.chain.lock().await;
            json!(&*chain).to_string()
        };
        
        let mut ws_stream = ws_stream.lock().await;
        if let Err(e) = ws_stream.send(tokio_tungstenite::tungstenite::Message::Text(chain_json)).await {
            eprintln!("Failed to send message: {}", e);
        }
    }

    pub async fn sync_chain(&self) {
        let sockets = self.sockets.lock().await;
        for socket in sockets.iter() {
            self.send_chain(socket.clone()).await;
        }
    }

    async fn message_handler(&self, ws_stream: Arc<Mutex<WsStream>>, chain: Arc<Mutex<Chain>>) {
        tokio::spawn(async move {
            loop {
                let msg = {
                    let mut ws_stream = ws_stream.lock().await;
                    ws_stream.next().await
                };

                match msg {
                    Some(Ok(tokio_tungstenite::tungstenite::Message::Text(text))) => {
                        if let Ok(received_chain) = serde_json::from_str::<Vec<serde_json::Value>>(&text) {
                            println!("Received chain: {:?}", received_chain);
                            let mut current_chain = chain.lock().await;
                            if let Ok(new_chain) = serde_json::from_value::<Chain>(serde_json::Value::Array(received_chain)) {
                                current_chain.replace_chain(&new_chain);
                            } else {
                                eprintln!("Failed to parse received chain");
                            }
                        }
                    }
                    None => break,
                    _ => {}
                }
            }
        });
    }
}

pub async fn run_p2p(chain: Chain) -> Result<(), Box<dyn std::error::Error>> {
    let p2p = P2p::new(chain);
    p2p.listen().await
}