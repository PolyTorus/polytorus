use crate::blockchain::chain::Chain;
use std::sync::Arc;
use serde::{Deserialize, Serialize};
use tokio::time::Duration as TokioDuration;
use futures::{SinkExt, StreamExt};
use tokio::net::TcpListener;
use tokio::sync::Mutex;
use tokio::net::TcpStream;
use tokio_tungstenite::{connect_async, MaybeTlsStream, WebSocketStream};
use crate::wallet::{transaction::Transaction, transaction_pool::Pool};

type WsStream = WebSocketStream<MaybeTlsStream<TcpStream>>;


#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
enum MessageType {
    CHAIN,
    TRANSACTION,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
struct Message {
    type_: MessageType,
    #[serde(skip_serializing_if = "Option::is_none")]
    chain: Option<Chain>,
    #[serde(skip_serializing_if = "Option::is_none")]
    transaction: Option<Transaction>,
}

#[derive(Debug, Clone)]
pub struct P2p {
    chain: Arc<Mutex<Chain>>,
    transaction_pool: Arc<Mutex<Pool>>,
    sockets: Arc<Mutex<Vec<Arc<Mutex<WsStream>>>>>,
}

impl P2p {
    pub fn new(chain: Chain, transaction_pool: Pool ) -> Self {
        P2p {
            chain: Arc::new(Mutex::new(chain)),
            transaction_pool: Arc::new(Mutex::new(transaction_pool)),
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
        let max_retries = 5;
        let retry_delay = TokioDuration::from_secs(5);

        for peer in peers.split(",") {
            if !peer.is_empty() {
                let mut retries = 0;
                while retries < max_retries {
                    match connect_async(peer).await {
                        Ok((ws_stream, _)) => {
                            self.connect_socket(ws_stream).await;
                            break;
                        }
                        Err(e) => {
                            eprintln!("Failed to connect to peer: {}. Retrying in {} seconds. Error: {}", peer, retry_delay.as_secs(), e);
                            retries += 1;
                            if retries < max_retries {
                                tokio::time::sleep(retry_delay).await;
                            }
                        }
                    }
                }
                // let (ws_stream, _) = connect_async(peer).await?;
                // self.connect_socket(ws_stream).await;
                if retries == max_retries {
                    eprint!("Failed to connect to peer: {}", peer);
                }
            }
        }
        Ok(())
    }

    pub async fn connect_socket(&self, ws_stream: WsStream) {
        // let chain = self.chain.clone();
        let sockets = self.sockets.clone();
        let ws_stream = Arc::new(Mutex::new(ws_stream));

        {
            let mut sockets = sockets.lock().await;
            sockets.push(ws_stream.clone());
            println!("Socket connected");
        }

        self.send_chain(ws_stream.clone()).await;
        self.message_handler(ws_stream.clone()).await;
    }

    async fn send_chain(&self, ws_stream: Arc<Mutex<WsStream>>) {
        let chain = {
            let chain = self.chain.lock().await;
            chain.clone()
        };

        let message = Message {
            type_: MessageType::CHAIN,
            chain: Some(chain),
            transaction: None,
        };
        
        let json = serde_json::to_string(&message).unwrap();
        
        let mut ws_stream = ws_stream.lock().await;
        if let Err(e) = ws_stream.send(tokio_tungstenite::tungstenite::Message::Text(json)).await {
            eprintln!("Failed to send message: {}", e);
        }
    }

    async fn send_transaction(&self, ws_stream: Arc<Mutex<WsStream>>, transaction: Transaction) {
        let message = Message {
            type_: MessageType::TRANSACTION,
            chain: None,
            transaction: Some(transaction),
        };

        let json = serde_json::to_string(&message).unwrap();

        let mut ws_stream = ws_stream.lock().await;
        if let Err(e) = ws_stream.send(tokio_tungstenite::tungstenite::Message::Text(json)).await {
            eprintln!("Failed to send transaction message: {}", e);
        }
    }

    pub async fn sync_chain(&self) {
        let sockets = self.sockets.lock().await;
        for socket in sockets.iter() {
            self.send_chain(socket.clone()).await;
        }
    }

    async fn message_handler(&self, ws_stream: Arc<Mutex<WsStream>>) {
        let blockchain = self.chain.clone();
        let transaction_pool = self.transaction_pool.clone();

        tokio::spawn(async move {
            loop {
                let msg = {
                    let mut ws_stream = ws_stream.lock().await;
                    ws_stream.next().await
                };

                match msg {
                    Some(Ok(tokio_tungstenite::tungstenite::Message::Text(text))) => {
                        if let Ok(message) = serde_json::from_str::<Message>(&text) {
                            match message.type_ {
                                MessageType::CHAIN => {
                                    if let Some(chain) = message.chain {
                                        let mut blockchain = blockchain.lock().await;
                                        blockchain.replace_chain(&chain);
                                    }
                                },
                                MessageType::TRANSACTION => {
                                    if let Some(transaction) = message.transaction {
                                        let mut transaction_pool = transaction_pool.lock().await;
                                        transaction_pool.update_or_add_transaction(transaction);
                                    }
                                },
                            }
                            println!("Received message type: {:?}", message.type_);
                        }
                    }
                    None => break,
                    _ => {}
                }
            }
        });
    }

    pub async fn broadcast_transaction(&self, transaction: Transaction) {
        let sockets = self.sockets.lock().await;
        for socket in sockets.iter() {
            self.send_transaction(socket.clone(), transaction.clone()).await;
        }
    }
}

pub async fn run_p2p(chain: Chain, transaction_pool: Pool) -> Result<(), Box<dyn std::error::Error>> {
    let p2p = P2p::new(chain, transaction_pool);
    p2p.listen().await
}