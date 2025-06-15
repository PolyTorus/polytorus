//! P2P network implementation for blockchain nodes
//!
//! This module provides a modern P2P networking layer for blockchain communication
//! with features like peer discovery, message broadcasting, and network resilience.

use crate::blockchain::block::{Block, FinalizedBlock};
use crate::crypto::transaction::Transaction;
use crate::Result;

use failure::format_err;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet},
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::mpsc,
    time::{interval, timeout},
};
use uuid::Uuid;

/// Maximum message size (10MB)
const MAX_MESSAGE_SIZE: usize = 10 * 1024 * 1024;

/// Protocol version for compatibility
const PROTOCOL_VERSION: u32 = 1;

/// Network events that can be sent to the application layer
#[derive(Debug, Clone)]
pub enum NetworkEvent {
    /// New peer connected
    PeerConnected(PeerId),
    /// Peer disconnected
    PeerDisconnected(PeerId),
    /// New block received
    BlockReceived(Box<FinalizedBlock>),
    /// New transaction received
    TransactionReceived(Box<Transaction>),
    /// Block request received
    BlockRequest(String, PeerId),
    /// Transaction request received
    TransactionRequest(String, PeerId),
    /// Peer information received
    PeerInfo(PeerId, i32), // peer_id, best_height
}

/// Network commands that can be sent to the network layer
#[derive(Debug, Clone)]
pub enum NetworkCommand {
    /// Broadcast a block
    BroadcastBlock(Box<FinalizedBlock>),
    /// Broadcast a transaction
    BroadcastTransaction(Transaction),
    /// Request a block by hash
    RequestBlock(String, PeerId),
    /// Request a transaction by hash
    RequestTransaction(String, PeerId),
    /// Connect to a specific peer
    ConnectPeer(SocketAddr),
    /// Get list of connected peers
    GetPeers,
    /// Send a direct message to a peer
    SendDirectMessage(PeerId, P2PMessage),
}

/// Peer identifier
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct PeerId(pub Uuid);

impl PeerId {
    pub fn random() -> Self {
        Self(Uuid::new_v4())
    }
}

impl std::fmt::Display for PeerId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

/// P2P protocol messages
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum P2PMessage {
    /// Handshake message with peer info
    Handshake {
        peer_id: PeerId,
        protocol_version: u32,
        best_height: i32,
        timestamp: u64,
        node_type: String,
    },
    /// Handshake acknowledgment
    HandshakeAck { peer_id: PeerId, accepted: bool },
    /// Ping message for connectivity check
    Ping { nonce: u64, timestamp: u64 },
    /// Pong response to ping
    Pong { nonce: u64, timestamp: u64 },
    /// Block announcement
    BlockAnnouncement {
        block_hash: String,
        block_height: i32,
    },
    /// Block data
    BlockData { block: Box<Block> },
    /// Transaction announcement
    TransactionAnnouncement { tx_hash: String },
    /// Transaction data
    TransactionData { transaction: Box<Transaction> },
    /// Request for block data
    BlockRequest { block_hash: String },
    /// Request for transaction data
    TransactionRequest { tx_hash: String },
    /// Peer list sharing
    PeerList { peers: Vec<PeerInfo> },
    /// Status update
    StatusUpdate { best_height: i32 },
}

/// Information about a peer
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub peer_id: PeerId,
    pub address: SocketAddr,
    pub last_seen: u64,
    pub best_height: i32,
    pub node_type: String,
}

/// Connection state for a peer
#[derive(Debug, Clone)]
struct PeerConnection {
    _peer_id: PeerId,
    address: SocketAddr,
    best_height: i32,
    _last_ping: Instant,
    last_pong: Instant,
    _connected_at: Instant,
    message_tx: mpsc::UnboundedSender<P2PMessage>,
}

/// P2P network node for blockchain communication
pub struct P2PNode {
    /// Our peer ID
    peer_id: PeerId,
    /// Address we're listening on
    listen_addr: SocketAddr,
    /// Event sender to application
    event_tx: mpsc::UnboundedSender<NetworkEvent>,
    /// Command receiver from application
    command_rx: mpsc::UnboundedReceiver<NetworkCommand>,
    /// Connected peers
    peers: Arc<Mutex<HashMap<PeerId, PeerConnection>>>,
    /// Known peer addresses for discovery
    known_peers: Arc<Mutex<HashSet<SocketAddr>>>,
    /// Our current blockchain height
    best_height: Arc<Mutex<i32>>,
}

impl P2PNode {
    /// Creates a new P2P node
    pub fn new(
        listen_addr: SocketAddr,
        bootstrap_peers: Vec<SocketAddr>,
    ) -> Result<(
        Self,
        mpsc::UnboundedReceiver<NetworkEvent>,
        mpsc::UnboundedSender<NetworkCommand>,
    )> {
        let peer_id = PeerId::random();
        let (event_tx, event_rx) = mpsc::unbounded_channel();
        let (command_tx, command_rx) = mpsc::unbounded_channel();

        let mut known_peers = HashSet::new();
        for addr in bootstrap_peers {
            known_peers.insert(addr);
        }

        log::info!("Created P2P node with peer ID: {}", peer_id);

        Ok((
            Self {
                peer_id,
                listen_addr,
                event_tx,
                command_rx,
                peers: Arc::new(Mutex::new(HashMap::new())),
                known_peers: Arc::new(Mutex::new(known_peers)),
                best_height: Arc::new(Mutex::new(0)),
            },
            event_rx,
            command_tx,
        ))
    }

    /// Runs the P2P node
    pub async fn run(&mut self) -> Result<()> {
        log::info!("Starting P2P node on {}", self.listen_addr);

        // Start listening for incoming connections
        let listener = TcpListener::bind(self.listen_addr).await?;
        log::info!("P2P node listening on {}", self.listen_addr);

        // Start background tasks
        self.start_background_tasks().await;

        // Start connecting to bootstrap peers
        self.connect_to_bootstrap_peers().await;

        // Main event loop
        loop {
            tokio::select! {
                // Accept incoming connections
                result = listener.accept() => {
                    match result {
                        Ok((stream, addr)) => {
                            log::debug!("Incoming connection from {}", addr);
                            let peers = self.peers.clone();
                            let event_tx = self.event_tx.clone();
                            let peer_id = self.peer_id;
                            let best_height = self.best_height.clone();

                            tokio::spawn(async move {
                                if let Err(e) = Self::handle_incoming_connection(
                                    stream, addr, peers, event_tx, peer_id, best_height
                                ).await {
                                    log::error!("Error handling incoming connection: {}", e);
                                }
                            });
                        }
                        Err(e) => {
                            log::error!("Error accepting connection: {}", e);
                        }
                    }
                }
                // Handle commands from application
                command = self.command_rx.recv() => {
                    match command {
                        Some(cmd) => {
                            if let Err(e) = self.handle_command(cmd).await {
                                log::error!("Error handling command: {}", e);
                            }
                        }
                        None => break,
                    }
                }
            }
        }

        Ok(())
    }

    /// Start background tasks
    async fn start_background_tasks(&self) {
        let peers = self.peers.clone();
        let _event_tx = self.event_tx.clone();

        // Ping task
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(30));
            loop {
                interval.tick().await;
                let peers_guard = peers.lock().unwrap();
                for (peer_id, connection) in peers_guard.iter() {
                    let nonce = SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_nanos() as u64;

                    let ping_msg = P2PMessage::Ping {
                        nonce,
                        timestamp: SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_secs(),
                    };

                    if let Err(e) = connection.message_tx.send(ping_msg) {
                        log::debug!("Failed to send ping to {}: {}", peer_id, e);
                    }
                }
            }
        });

        // Cleanup task for stale connections
        let peers_cleanup = self.peers.clone();
        let event_tx_cleanup = self.event_tx.clone();
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(60));
            loop {
                interval.tick().await;
                let mut to_remove = Vec::new();
                {
                    let peers_guard = peers_cleanup.lock().unwrap();
                    let now = Instant::now();
                    for (peer_id, connection) in peers_guard.iter() {
                        // Remove peers that haven't responded to ping in 2 minutes
                        if now.duration_since(connection.last_pong) > Duration::from_secs(120) {
                            to_remove.push(*peer_id);
                        }
                    }
                }

                for peer_id in to_remove {
                    peers_cleanup.lock().unwrap().remove(&peer_id);
                    let _ = event_tx_cleanup.send(NetworkEvent::PeerDisconnected(peer_id));
                    log::info!("Removed stale peer: {}", peer_id);
                }
            }
        });
    }

    /// Connect to bootstrap peers
    async fn connect_to_bootstrap_peers(&self) {
        let known_peers = self.known_peers.lock().unwrap().clone();
        for addr in known_peers {
            let peers = self.peers.clone();
            let event_tx = self.event_tx.clone();
            let peer_id = self.peer_id;
            let best_height = self.best_height.clone();

            tokio::spawn(async move {
                if let Err(e) =
                    Self::connect_to_peer(addr, peers, event_tx, peer_id, best_height).await
                {
                    log::warn!("Failed to connect to bootstrap peer {}: {}", addr, e);
                }
            });
        }
    }

    /// Connect to a specific peer
    async fn connect_to_peer(
        addr: SocketAddr,
        peers: Arc<Mutex<HashMap<PeerId, PeerConnection>>>,
        event_tx: mpsc::UnboundedSender<NetworkEvent>,
        our_peer_id: PeerId,
        best_height: Arc<Mutex<i32>>,
    ) -> Result<()> {
        log::debug!("Connecting to peer at {}", addr);

        let stream = timeout(Duration::from_secs(10), TcpStream::connect(addr)).await??;

        // Send handshake
        let handshake = P2PMessage::Handshake {
            peer_id: our_peer_id,
            protocol_version: PROTOCOL_VERSION,
            best_height: *best_height.lock().unwrap(),
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            node_type: "full_node".to_string(),
        };

        Self::handle_peer_connection(stream, addr, peers, event_tx, our_peer_id, Some(handshake))
            .await
    }

    /// Handle incoming connection
    async fn handle_incoming_connection(
        stream: TcpStream,
        addr: SocketAddr,
        peers: Arc<Mutex<HashMap<PeerId, PeerConnection>>>,
        event_tx: mpsc::UnboundedSender<NetworkEvent>,
        our_peer_id: PeerId,
        _best_height: Arc<Mutex<i32>>,
    ) -> Result<()> {
        Self::handle_peer_connection(stream, addr, peers, event_tx, our_peer_id, None).await
    }

    /// Handle peer connection (both incoming and outgoing)
    async fn handle_peer_connection(
        mut stream: TcpStream,
        addr: SocketAddr,
        peers: Arc<Mutex<HashMap<PeerId, PeerConnection>>>,
        event_tx: mpsc::UnboundedSender<NetworkEvent>,
        our_peer_id: PeerId,
        initial_message: Option<P2PMessage>,
    ) -> Result<()> {
        let (_message_tx, mut message_rx) = mpsc::unbounded_channel();

        // Send initial message if provided (outgoing connection)
        if let Some(msg) = initial_message {
            Self::send_message(&mut stream, &msg).await?;
        }

        // Read messages from peer
        let mut peer_id_opt = None;

        loop {
            tokio::select! {
                // Read message from peer
                result = Self::read_message(&mut stream) => {
                    match result {
                        Ok(message) => {
                            match Self::handle_peer_message(
                                message,
                                &mut peer_id_opt,
                                addr,
                                &peers,
                                &event_tx,
                                our_peer_id,
                                &mut stream,
                            ).await {
                                Ok(true) => continue,
                                Ok(false) => break,
                                Err(e) => {
                                    log::error!("Error handling peer message: {}", e);
                                    break;
                                }
                            }
                        }
                        Err(e) => {
                            log::debug!("Connection closed: {}", e);
                            break;
                        }
                    }
                }
                // Send message to peer
                message = message_rx.recv() => {
                    match message {
                        Some(msg) => {
                            if let Err(e) = Self::send_message(&mut stream, &msg).await {
                                log::error!("Failed to send message: {}", e);
                                break;
                            }
                        }
                        None => break,
                    }
                }
            }
        }

        // Clean up on disconnect
        if let Some(peer_id) = peer_id_opt {
            peers.lock().unwrap().remove(&peer_id);
            let _ = event_tx.send(NetworkEvent::PeerDisconnected(peer_id));
        }

        Ok(())
    }

    /// Handle a message from a peer
    async fn handle_peer_message(
        message: P2PMessage,
        peer_id_opt: &mut Option<PeerId>,
        addr: SocketAddr,
        peers: &Arc<Mutex<HashMap<PeerId, PeerConnection>>>,
        event_tx: &mpsc::UnboundedSender<NetworkEvent>,
        our_peer_id: PeerId,
        stream: &mut TcpStream,
    ) -> Result<bool> {
        match message {
            P2PMessage::Handshake {
                peer_id,
                protocol_version,
                best_height,
                timestamp: _,
                node_type: _,
            } => {
                if protocol_version != PROTOCOL_VERSION {
                    log::warn!(
                        "Protocol version mismatch with {}: {} vs {}",
                        peer_id,
                        protocol_version,
                        PROTOCOL_VERSION
                    );
                    return Ok(false);
                }

                *peer_id_opt = Some(peer_id);

                // Send handshake ack
                let ack = P2PMessage::HandshakeAck {
                    peer_id: our_peer_id,
                    accepted: true,
                };
                Self::send_message(stream, &ack).await?;

                // Add to peers
                let (message_tx, _) = mpsc::unbounded_channel();
                let connection = PeerConnection {
                    _peer_id: peer_id,
                    address: addr,
                    best_height,
                    _last_ping: Instant::now(),
                    last_pong: Instant::now(),
                    _connected_at: Instant::now(),
                    message_tx,
                };

                peers.lock().unwrap().insert(peer_id, connection);
                let _ = event_tx.send(NetworkEvent::PeerConnected(peer_id));
                let _ = event_tx.send(NetworkEvent::PeerInfo(peer_id, best_height));

                log::info!("Peer {} connected from {}", peer_id, addr);
            }
            P2PMessage::HandshakeAck { peer_id, accepted } => {
                if !accepted {
                    log::warn!("Handshake rejected by {}", peer_id);
                    return Ok(false);
                }
                log::debug!("Handshake accepted by {}", peer_id);
            }
            P2PMessage::Ping {
                nonce,
                timestamp: _,
            } => {
                let pong = P2PMessage::Pong {
                    nonce,
                    timestamp: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                };
                Self::send_message(stream, &pong).await?;
            }
            P2PMessage::Pong {
                nonce: _,
                timestamp: _,
            } => {
                if let Some(peer_id) = peer_id_opt {
                    if let Some(connection) = peers.lock().unwrap().get_mut(peer_id) {
                        connection.last_pong = Instant::now();
                    }
                }
            }
            P2PMessage::BlockData { block } => {
                let _ = event_tx.send(NetworkEvent::BlockReceived(block));
            }
            P2PMessage::TransactionData { transaction } => {
                let _ = event_tx.send(NetworkEvent::TransactionReceived(transaction));
            }
            P2PMessage::BlockRequest { block_hash } => {
                if let Some(peer_id) = peer_id_opt {
                    let _ = event_tx.send(NetworkEvent::BlockRequest(block_hash, *peer_id));
                }
            }
            P2PMessage::TransactionRequest { tx_hash } => {
                if let Some(peer_id) = peer_id_opt {
                    let _ = event_tx.send(NetworkEvent::TransactionRequest(tx_hash, *peer_id));
                }
            }
            P2PMessage::StatusUpdate { best_height } => {
                if let Some(peer_id) = peer_id_opt {
                    if let Some(connection) = peers.lock().unwrap().get_mut(peer_id) {
                        connection.best_height = best_height;
                    }
                    let _ = event_tx.send(NetworkEvent::PeerInfo(*peer_id, best_height));
                }
            }
            _ => {
                log::debug!("Received message: {:?}", message);
            }
        }

        Ok(true)
    }

    /// Send a message to a peer
    async fn send_message(stream: &mut TcpStream, message: &P2PMessage) -> Result<()> {
        let data = bincode::serialize(message)?;
        let len = data.len() as u32;

        if len > MAX_MESSAGE_SIZE as u32 {
            return Err(format_err!("Message too large: {}", len));
        }

        // Send length prefix
        stream.write_all(&len.to_be_bytes()).await?;
        // Send data
        stream.write_all(&data).await?;
        stream.flush().await?;

        Ok(())
    }

    /// Read a message from a peer
    async fn read_message(stream: &mut TcpStream) -> Result<P2PMessage> {
        // Read length prefix
        let mut len_bytes = [0u8; 4];
        stream.read_exact(&mut len_bytes).await?;
        let len = u32::from_be_bytes(len_bytes) as usize;

        if len > MAX_MESSAGE_SIZE {
            return Err(format_err!("Message too large: {}", len));
        }

        // Read data
        let mut data = vec![0u8; len];
        stream.read_exact(&mut data).await?;

        // Deserialize
        let message = bincode::deserialize(&data)?;
        Ok(message)
    }
    /// Handle commands from application
    async fn handle_command(&mut self, command: NetworkCommand) -> Result<()> {
        match command {
            NetworkCommand::BroadcastBlock(block) => {
                let message = P2PMessage::BlockData { block };
                self.broadcast_message(message).await?;
            }
            NetworkCommand::BroadcastTransaction(transaction) => {
                let message = P2PMessage::TransactionData {
                    transaction: Box::new(transaction),
                };
                self.broadcast_message(message).await?;
            }
            NetworkCommand::RequestBlock(hash, peer_id) => {
                let message = P2PMessage::BlockRequest { block_hash: hash };
                self.send_to_peer(peer_id, message).await?;
            }
            NetworkCommand::RequestTransaction(hash, peer_id) => {
                let message = P2PMessage::TransactionRequest { tx_hash: hash };
                self.send_to_peer(peer_id, message).await?;
            }
            NetworkCommand::ConnectPeer(addr) => {
                let peers = self.peers.clone();
                let event_tx = self.event_tx.clone();
                let peer_id = self.peer_id;
                let best_height = self.best_height.clone();

                tokio::spawn(async move {
                    if let Err(e) =
                        Self::connect_to_peer(addr, peers, event_tx, peer_id, best_height).await
                    {
                        log::error!("Failed to connect to peer {}: {}", addr, e);
                    }
                });
            }
            NetworkCommand::GetPeers => {
                let peers = self.peers.lock().unwrap();
                log::info!("Connected peers: {}", peers.len());
                for (peer_id, connection) in peers.iter() {
                    log::info!(
                        "  {} at {} (height: {})",
                        peer_id,
                        connection.address,
                        connection.best_height
                    );
                }
            }
            NetworkCommand::SendDirectMessage(peer_id, message) => {
                self.send_to_peer(peer_id, message).await?;
            }
        }

        Ok(())
    }

    /// Broadcast a message to all connected peers
    async fn broadcast_message(&self, message: P2PMessage) -> Result<()> {
        let peers = self.peers.lock().unwrap();
        for (peer_id, connection) in peers.iter() {
            if let Err(e) = connection.message_tx.send(message.clone()) {
                log::debug!("Failed to send message to {}: {}", peer_id, e);
            }
        }
        Ok(())
    }

    /// Send a message to a specific peer
    async fn send_to_peer(&self, peer_id: PeerId, message: P2PMessage) -> Result<()> {
        let peers = self.peers.lock().unwrap();
        if let Some(connection) = peers.get(&peer_id) {
            connection
                .message_tx
                .send(message)
                .map_err(|e| format_err!("Failed to send to peer {}: {}", peer_id, e))?;
        } else {
            return Err(format_err!("Peer {} not connected", peer_id));
        }
        Ok(())
    }

    /// Get connected peers
    pub fn get_connected_peers(&self) -> Vec<PeerId> {
        self.peers.lock().unwrap().keys().cloned().collect()
    }

    /// Get peer heights
    pub fn get_peer_heights(&self) -> HashMap<PeerId, i32> {
        self.peers
            .lock()
            .unwrap()
            .iter()
            .map(|(id, conn)| (*id, conn.best_height))
            .collect()
    }

    /// Update our best height
    pub fn update_best_height(&self, height: i32) {
        *self.best_height.lock().unwrap() = height;
    }
}
