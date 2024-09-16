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

        let chain_json = {
            let chain = chain.lock().await;
            json!(&*chain).to_string()
        };
        
        {
            let mut ws_stream = ws_stream.lock().await;
            ws_stream.send(tokio_tungstenite::tungstenite::Message::Text(chain_json)).await.unwrap();
        }

        let chain_for_message = chain.clone();
        let ws_stream_clone = ws_stream.clone();
        tokio::spawn(async move {
            loop {
                let msg = {
                    let mut ws_stream = ws_stream_clone.lock().await;
                    ws_stream.next().await
                };

                match msg {
                    Some(Ok(tokio_tungstenite::tungstenite::Message::Text(text))) => {
                        if let Ok(received_chain) = serde_json::from_str::<Chain>(&text) {
                            println!("Received chain: {:?}", received_chain);
                            let mut current_chain = chain_for_message.lock().await;
                            current_chain.replace_chain(&received_chain);
                        }
                    }
                    None => break,
                    _ => {}
                }
            }
        });
    }

    pub async fn broadcast_chain(&self) {
        let chain_json = {
            let chain = self.chain.lock().await;
            json!(&*chain).to_string()
        };
        
        let sockets = self.sockets.lock().await;
        for socket in sockets.iter() {
            let mut socket = socket.lock().await;
            if let Err(e) = socket.send(tokio_tungstenite::tungstenite::Message::Text(chain_json.clone())).await {
                eprintln!("Failed to send message: {}", e);
            }
        }
    }
}
