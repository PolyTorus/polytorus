//! Enhanced P2P network implementation for blockchain nodes
//!
//! This module provides a complete P2P networking layer for blockchain communication
//! with features like peer discovery, message broadcasting, transaction propagation,
//! network resilience, network management, and message prioritization.

use crate::blockchain::block::{Block, FinalizedBlock};
use crate::crypto::transaction::Transaction;
use crate::network::{
    network_manager::{NetworkManager, NetworkManagerConfig, PeerInfo as NetPeerInfo},
    message_priority::{PriorityMessageQueue, MessagePriority, PrioritizedMessage},
};
use crate::Result;

use bincode;
use failure::format_err;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, HashSet, VecDeque},
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
/// Maximum peers to maintain connections with
const MAX_PEERS: usize = 50;
/// Ping interval in seconds
const PING_INTERVAL: u64 = 30;
/// Peer timeout in seconds
const PEER_TIMEOUT: u64 = 120;

/// Network events that can be sent to the application layer
#[derive(Debug, Clone)]
pub enum NetworkEvent {
    /// New peer connected
    PeerConnected(PeerId),
    /// Peer disconnected
    PeerDisconnected(PeerId),
    /// New block received
    BlockReceived(Box<FinalizedBlock>, PeerId),
    /// New transaction received
    TransactionReceived(Box<Transaction>, PeerId),
    /// Block request received
    BlockRequest(String, PeerId),
    /// Transaction request received
    TransactionRequest(String, PeerId),
    /// Peer information received
    PeerInfo(PeerId, i32),
    /// Peer discovery update
    PeerDiscovery(Vec<PeerInfo>),
    /// Network health status update
    NetworkHealthUpdate(crate::network::network_manager::NetworkTopology),
    /// Peer health status changed
    PeerHealthChanged(PeerId, crate::network::network_manager::NodeHealth),
    /// Message queue statistics update
    MessageQueueStats(crate::network::message_priority::QueueStats),
}

/// Network commands that can be sent to the network layer
#[derive(Debug, Clone)]
pub enum NetworkCommand {
    /// Broadcast a block
    BroadcastBlock(Box<FinalizedBlock>),
    /// Broadcast a transaction
    BroadcastTransaction(Transaction),
    /// Broadcast with priority
    BroadcastPriority(P2PMessage, MessagePriority),
    /// Request a block by hash from a specific peer
    RequestBlock(String, PeerId),
    /// Request a transaction by hash from a specific peer
    RequestTransaction(String, PeerId),
    /// Connect to a specific peer
    ConnectPeer(SocketAddr),
    /// Disconnect from a peer
    DisconnectPeer(PeerId),
    /// Get list of connected peers
    GetPeers,
    /// Send a direct message to a peer
    SendDirectMessage(PeerId, P2PMessage),
    /// Send priority message to a peer
    SendPriorityMessage(PeerId, P2PMessage, MessagePriority),
    /// Request peer list from all connected peers
    RequestPeerDiscovery,
    /// Update our best block height
    UpdateHeight(i32),
    /// Get network health information
    GetNetworkHealth,
    /// Get peer information
    GetPeerInfo(PeerId),
    /// Add peer to blacklist
    BlacklistPeer(PeerId, String),
    /// Remove peer from blacklist
    UnblacklistPeer(PeerId),
    /// Get message queue statistics
    GetMessageQueueStats,
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
    /// Error message
    Error { message: String },
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
#[derive(Debug)]
struct PeerConnection {
    peer_id: PeerId,
    address: SocketAddr,
    best_height: i32,
    last_ping: Instant,
    last_pong: Instant,
    connected_at: Instant,
    message_tx: mpsc::UnboundedSender<P2PMessage>,
    message_queue: VecDeque<P2PMessage>,
    is_active: bool,
    ping_nonce: Option<u64>,
}

impl PeerConnection {
    fn new(
        peer_id: PeerId,
        address: SocketAddr,
        message_tx: mpsc::UnboundedSender<P2PMessage>,
    ) -> Self {
        let now = Instant::now();
        Self {
            peer_id,
            address,
            best_height: 0,
            last_ping: now,
            last_pong: now,
            connected_at: now,
            message_tx,
            message_queue: VecDeque::new(),
            is_active: true,
            ping_nonce: None,
        }
    }

    fn is_stale(&self) -> bool {
        let is_stale = self.last_pong.elapsed() > Duration::from_secs(PEER_TIMEOUT);
        if is_stale {
            log::debug!(
                "Peer {} is stale (last pong: {:?} ago)",
                self.peer_id,
                self.last_pong.elapsed()
            );
        }
        is_stale
    }

    fn queue_message(&mut self, message: P2PMessage) {
        if self.message_queue.len() < 1000 {
            // Prevent memory overflow
            self.message_queue.push_back(message);
        }
    }

    fn send_queued_messages(&mut self) -> Result<()> {
        while let Some(message) = self.message_queue.pop_front() {
            if self.message_tx.send(message).is_err() {
                return Err(format_err!("Failed to send queued message"));
            }
        }
        Ok(())
    }
}

/// Enhanced P2P network node for blockchain communication
pub struct EnhancedP2PNode {
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
    /// Transaction pool for mempool synchronization
    transaction_pool: Arc<Mutex<HashMap<String, Transaction>>>,
    /// Block cache for block synchronization
    block_cache: Arc<Mutex<HashMap<String, FinalizedBlock>>>,
    /// Network statistics
    stats: Arc<Mutex<NetworkStats>>,
    /// Network manager for health monitoring and topology optimization
    network_manager: Arc<Mutex<NetworkManager>>,
    /// Priority message queue for message prioritization and rate limiting
    message_queue: Arc<Mutex<PriorityMessageQueue>>,
}

/// Network statistics
#[derive(Debug, Default, Clone)]
pub struct NetworkStats {
    pub total_connections: u64,
    pub active_connections: u64,
    pub messages_sent: u64,
    pub messages_received: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub blocks_propagated: u64,
    pub transactions_propagated: u64,
}

impl EnhancedP2PNode {
    /// Creates a new enhanced P2P node
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
        for addr in bootstrap_peers.clone() {
            known_peers.insert(addr);
        }

        // Initialize network manager
        let network_manager = NetworkManager::new(
            NetworkManagerConfig::default(),
            bootstrap_peers,
        );
        
        // Initialize priority message queue
        let message_queue = PriorityMessageQueue::new(
            crate::network::message_priority::RateLimitConfig::default(),
        );

        log::info!("Created enhanced P2P node with peer ID: {}", peer_id);

        Ok((
            Self {
                peer_id,
                listen_addr,
                event_tx,
                command_rx,
                peers: Arc::new(Mutex::new(HashMap::new())),
                known_peers: Arc::new(Mutex::new(known_peers)),
                best_height: Arc::new(Mutex::new(0)),
                transaction_pool: Arc::new(Mutex::new(HashMap::new())),
                block_cache: Arc::new(Mutex::new(HashMap::new())),
                stats: Arc::new(Mutex::new(NetworkStats::default())),
                network_manager: Arc::new(Mutex::new(network_manager)),
                message_queue: Arc::new(Mutex::new(message_queue)),
            },
            event_rx,
            command_tx,
        ))
    }

    /// Runs the enhanced P2P node
    pub async fn run(&mut self) -> Result<()> {
        log::info!("Starting enhanced P2P node on {}", self.listen_addr);

        // Start listening for incoming connections
        let listener = TcpListener::bind(self.listen_addr).await?;
        log::info!("Enhanced P2P node listening on {}", self.listen_addr);

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
                            self.handle_incoming_connection(stream, addr).await;
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
        // Start network manager (simplified approach - no background task for now)
        // In a production system, this would need a proper async approach
        
        // Start message queue processing (simplified)
        let message_queue_clone = self.message_queue.clone();
        let peers_clone = self.peers.clone();
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_millis(100));
            loop {
                interval.tick().await;
                
                // Try to process one message at a time to avoid holding locks across await
                let message_opt = {
                    if let Ok(mut queue) = message_queue_clone.try_lock() {
                        queue.dequeue()
                    } else {
                        None
                    }
                };
                
                if let Some(mut message) = message_opt {
                    // Process the message outside the lock
                    if let Ok(peers) = peers_clone.try_lock() {
                        if let Some(target_peer) = message.target_peer {
                            if let Some(connection) = peers.get(&target_peer) {
                                if connection.is_active {
                                    log::debug!("Sending priority message {} to peer {}", 
                                               message.id, target_peer);
                                }
                            }
                        }
                    }
                    message.increment_retry();
                }
            }
        });

        // Ping task
        let peers_ping = self.peers.clone();
        let stats_ping = self.stats.clone();
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(PING_INTERVAL));
            loop {
                interval.tick().await;
                let mut peers_guard = peers_ping.lock().unwrap();
                let mut to_ping = Vec::new();

                for (peer_id, connection) in peers_guard.iter_mut() {
                    if connection.is_active
                        && connection.last_ping.elapsed() > Duration::from_secs(PING_INTERVAL)
                    {
                        let nonce = SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_nanos() as u64;

                        connection.ping_nonce = Some(nonce);
                        connection.last_ping = Instant::now();

                        let ping_msg = P2PMessage::Ping {
                            nonce,
                            timestamp: SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap()
                                .as_secs(),
                        };

                        to_ping.push((*peer_id, ping_msg));
                    }
                }

                for (peer_id, ping_msg) in to_ping {
                    if let Some(connection) = peers_guard.get(&peer_id) {
                        if let Err(e) = connection.message_tx.send(ping_msg) {
                            log::debug!("Failed to send ping to {}: {}", peer_id, e);
                        } else {
                            stats_ping.lock().unwrap().messages_sent += 1;
                        }
                    }
                }
            }
        });

        // Cleanup task for stale connections
        let peers_cleanup = self.peers.clone();
        let event_tx_cleanup = self.event_tx.clone();
        let stats_cleanup = self.stats.clone();
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(60));
            loop {
                interval.tick().await;
                let mut to_remove = Vec::new();
                {
                    let peers_guard = peers_cleanup.lock().unwrap();
                    for (peer_id, connection) in peers_guard.iter() {
                        if connection.is_stale() {
                            to_remove.push(*peer_id);
                        }
                    }
                }

                for peer_id in to_remove {
                    peers_cleanup.lock().unwrap().remove(&peer_id);
                    let _ = event_tx_cleanup.send(NetworkEvent::PeerDisconnected(peer_id));
                    stats_cleanup.lock().unwrap().active_connections -= 1;
                    log::info!("Removed stale peer: {}", peer_id);
                }
            }
        });

        // Peer discovery task
        let known_peers_discovery = self.known_peers.clone();
        let peers_discovery = self.peers.clone();
        let event_tx_discovery = self.event_tx.clone();
        let peer_id_discovery = self.peer_id;
        let best_height_discovery = self.best_height.clone();
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(300)); // Every 5 minutes
            loop {
                interval.tick().await;

                // Try to connect to new peers from known peers list
                let known_addrs: Vec<SocketAddr> = {
                    let known = known_peers_discovery.lock().unwrap();
                    known.iter().cloned().collect()
                };

                let current_peer_count = peers_discovery.lock().unwrap().len();
                if current_peer_count < MAX_PEERS / 2 {
                    for addr in known_addrs.iter().take(3) {
                        // Try 3 new connections at a time
                        let peers_clone = peers_discovery.clone();
                        let event_tx_clone = event_tx_discovery.clone();
                        let addr_clone = *addr;
                        let peer_id_clone = peer_id_discovery;
                        let best_height_clone = best_height_discovery.clone();

                        tokio::spawn(async move {
                            if let Err(e) = Self::connect_to_peer(
                                addr_clone,
                                peers_clone,
                                event_tx_clone,
                                peer_id_clone,
                                best_height_clone,
                            )
                            .await
                            {
                                log::debug!(
                                    "Failed to connect to discovered peer {}: {}",
                                    addr_clone,
                                    e
                                );
                            }
                        });
                    }
                }
            }
        });

        // Message queue processing task
        let peers_queue = self.peers.clone();
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_millis(100)); // Process queue every 100ms
            loop {
                interval.tick().await;
                let mut peers_to_process = Vec::new();

                // Collect peers that have queued messages
                {
                    let peers_guard = peers_queue.lock().unwrap();
                    for (peer_id, connection) in peers_guard.iter() {
                        if !connection.message_queue.is_empty() {
                            peers_to_process.push(*peer_id);
                        }
                    }
                }

                // Process queued messages for each peer
                for peer_id in peers_to_process {
                    if let Some(connection) = peers_queue.lock().unwrap().get_mut(&peer_id) {
                        if let Err(e) = connection.send_queued_messages() {
                            log::debug!(
                                "Failed to send queued messages for peer {}: {}",
                                peer_id,
                                e
                            );
                        }
                    }
                }
            }
        });
    }

    /// Connect to bootstrap peers
    async fn connect_to_bootstrap_peers(&self) {
        let known_peers = self.known_peers.lock().unwrap().clone();
        log::info!("Connecting to {} bootstrap peers", known_peers.len());

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
                } else {
                    log::info!("Successfully connected to bootstrap peer {}", addr);
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

        // Check if we're already connected to this address
        {
            let peers_guard = peers.lock().unwrap();
            for connection in peers_guard.values() {
                if connection.address == addr {
                    log::debug!("Already connected to {}", addr);
                    return Ok(());
                }
            }
        }

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
    async fn handle_incoming_connection(&self, stream: TcpStream, addr: SocketAddr) {
        let peers = self.peers.clone();
        let event_tx = self.event_tx.clone();
        let our_peer_id = self.peer_id;
        let stats = self.stats.clone();

        tokio::spawn(async move {
            stats.lock().unwrap().total_connections += 1;

            if let Err(e) =
                Self::handle_peer_connection(stream, addr, peers, event_tx, our_peer_id, None).await
            {
                log::error!("Error handling incoming connection from {}: {}", addr, e);
            }
        });
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
        let (message_tx, mut message_rx) = mpsc::unbounded_channel();

        // Send initial message if provided (outgoing connection)
        if let Some(msg) = initial_message {
            Self::send_message(&mut stream, &msg).await?;
        }

        let mut peer_id_opt: Option<PeerId> = None;
        let mut connection_established = false;

        loop {
            tokio::select! {
                // Read message from peer
                result = Self::read_message(&mut stream) => {
                    match result {
                        Ok(message) => {
                            match Self::handle_peer_message(
                                message,
                                &mut peer_id_opt,
                                &mut connection_established,
                                addr,
                                &peers,
                                &event_tx,
                                our_peer_id,
                                &mut stream,
                                &message_tx,
                            ).await {
                                Ok(true) => continue,
                                Ok(false) => break,
                                Err(e) => {
                                    log::error!("Error handling peer message from {}: {}", addr, e);
                                    break;
                                }
                            }
                        }
                        Err(e) => {
                            log::debug!("Connection to {} closed: {}", addr, e);
                            break;
                        }
                    }
                }
                // Send message to peer
                message = message_rx.recv() => {
                    match message {
                        Some(msg) => {
                            if let Err(e) = Self::send_message(&mut stream, &msg).await {
                                log::error!("Failed to send message to {}: {}", addr, e);
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
            log::info!("Peer {} disconnected", peer_id);
        }

        Ok(())
    }

    /// Handle a message from a peer
    async fn handle_peer_message(
        message: P2PMessage,
        peer_id_opt: &mut Option<PeerId>,
        connection_established: &mut bool,
        addr: SocketAddr,
        peers: &Arc<Mutex<HashMap<PeerId, PeerConnection>>>,
        event_tx: &mpsc::UnboundedSender<NetworkEvent>,
        our_peer_id: PeerId,
        stream: &mut TcpStream,
        message_tx: &mpsc::UnboundedSender<P2PMessage>,
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
                    let error = P2PMessage::Error {
                        message: format!(
                            "Protocol version mismatch: expected {}, got {}",
                            PROTOCOL_VERSION, protocol_version
                        ),
                    };
                    Self::send_message(stream, &error).await?;
                    return Ok(false);
                }

                // Check if we already have this peer
                let already_connected = {
                    let peers_guard = peers.lock().unwrap();
                    peers_guard.contains_key(&peer_id)
                };

                if already_connected {
                    log::debug!("Already connected to peer {}", peer_id);
                    let error = P2PMessage::Error {
                        message: "Already connected".to_string(),
                    };
                    Self::send_message(stream, &error).await?;
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
                let mut connection = PeerConnection::new(peer_id, addr, message_tx.clone());
                connection.best_height = best_height;
                connection.is_active = true;

                peers.lock().unwrap().insert(peer_id, connection);
                let _ = event_tx.send(NetworkEvent::PeerConnected(peer_id));
                let _ = event_tx.send(NetworkEvent::PeerInfo(peer_id, best_height));

                *connection_established = true;
                log::info!(
                    "Peer {} connected from {} (height: {})",
                    peer_id,
                    addr,
                    best_height
                );
            }
            P2PMessage::HandshakeAck { peer_id, accepted } => {
                if !accepted {
                    log::warn!("Handshake rejected by {}", peer_id);
                    return Ok(false);
                }
                *connection_established = true;
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
                nonce,
                timestamp: _,
            } => {
                if let Some(peer_id) = peer_id_opt {
                    if let Some(connection) = peers.lock().unwrap().get_mut(peer_id) {
                        // Verify nonce matches
                        if connection.ping_nonce == Some(nonce) {
                            connection.last_pong = Instant::now();
                            connection.ping_nonce = None;
                        }
                    }
                }
            }
            P2PMessage::BlockData { block } => {
                if let Some(peer_id) = peer_id_opt {
                    // Create a simplified FinalizedBlock from Block
                    // In a real implementation, you'd handle the conversion more carefully
                    let finalized_block = Box::new(*block.clone());
                    let _ = event_tx.send(NetworkEvent::BlockReceived(finalized_block, *peer_id));
                }
            }
            P2PMessage::TransactionData { transaction } => {
                if let Some(peer_id) = peer_id_opt {
                    let _ = event_tx.send(NetworkEvent::TransactionReceived(transaction, *peer_id));
                }
            }
            P2PMessage::BlockRequest { block_hash } => {
                if let Some(peer_id) = peer_id_opt {
                    // Also queue the request for potential retry
                    if let Some(connection) = peers.lock().unwrap().get_mut(peer_id) {
                        connection.queue_message(P2PMessage::BlockRequest {
                            block_hash: block_hash.clone(),
                        });
                        log::debug!("Queued block request for peer {}", connection.peer_id);
                    }
                    let _ = event_tx.send(NetworkEvent::BlockRequest(block_hash, *peer_id));
                }
            }
            P2PMessage::TransactionRequest { tx_hash } => {
                if let Some(peer_id) = peer_id_opt {
                    // Also queue the request for potential retry
                    if let Some(connection) = peers.lock().unwrap().get_mut(peer_id) {
                        connection.queue_message(P2PMessage::TransactionRequest {
                            tx_hash: tx_hash.clone(),
                        });
                        log::debug!("Queued transaction request for peer {}", connection.peer_id);
                    }
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
            P2PMessage::PeerList { peers: peer_list } => {
                let _ = event_tx.send(NetworkEvent::PeerDiscovery(peer_list));
            }
            P2PMessage::Error { message } => {
                log::warn!("Received error from peer: {}", message);
                if !*connection_established {
                    return Ok(false);
                }
            }
            _ => {
                log::debug!("Received unhandled message: {:?}", message);
            }
        }

        Ok(true)
    }

    /// Send a message to a peer
    async fn send_message(stream: &mut TcpStream, message: &P2PMessage) -> Result<()> {
        let data =
            bincode::serialize(message).map_err(|e| format_err!("Serialization failed: {}", e))?;
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
        // Read length prefix with timeout
        let mut len_bytes = [0u8; 4];
        timeout(Duration::from_secs(30), stream.read_exact(&mut len_bytes)).await??;
        let len = u32::from_be_bytes(len_bytes) as usize;

        if len > MAX_MESSAGE_SIZE {
            return Err(format_err!("Message too large: {}", len));
        }

        if len == 0 {
            return Err(format_err!("Empty message"));
        }

        // Read data with timeout
        let mut data = vec![0u8; len];
        timeout(Duration::from_secs(30), stream.read_exact(&mut data)).await??;

        // Deserialize with error handling
        let message = bincode::deserialize(&data)
            .map_err(|e| format_err!("Deserialization failed: {}", e))?;
        Ok(message)
    }

    /// Handle commands from application
    async fn handle_command(&mut self, command: NetworkCommand) -> Result<()> {
        match command {
            NetworkCommand::BroadcastBlock(block) => {
                self.broadcast_block(block).await?;
            }
            NetworkCommand::BroadcastTransaction(transaction) => {
                self.broadcast_transaction(transaction).await?;
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
                    } else {
                        log::info!("Successfully connected to peer {}", addr);
                    }
                });
            }
            NetworkCommand::DisconnectPeer(peer_id) => {
                if let Some(_connection) = self.peers.lock().unwrap().remove(&peer_id) {
                    let _ = self.event_tx.send(NetworkEvent::PeerDisconnected(peer_id));
                    log::info!("Disconnected from peer {}", peer_id);
                }
            }
            NetworkCommand::GetPeers => {
                self.print_peer_info().await;
            }
            NetworkCommand::SendDirectMessage(peer_id, message) => {
                self.send_to_peer(peer_id, message).await?;
            }
            NetworkCommand::RequestPeerDiscovery => {
                self.request_peer_discovery().await?;
            }
            NetworkCommand::UpdateHeight(height) => {
                *self.best_height.lock().unwrap() = height;
                self.broadcast_status_update(height).await?;
            }
            NetworkCommand::BroadcastPriority(message, priority) => {
                self.broadcast_priority_message(message, priority).await?;
            }
            NetworkCommand::SendPriorityMessage(peer_id, message, priority) => {
                self.send_priority_message(message, priority, Some(peer_id)).await?;
            }
            NetworkCommand::GetNetworkHealth => {
                match self.get_network_health().await {
                    Ok(health) => {
                        let _ = self.event_tx.send(NetworkEvent::NetworkHealthUpdate(health));
                    }
                    Err(e) => log::error!("Failed to get network health: {}", e),
                }
            }
            NetworkCommand::GetPeerInfo(peer_id) => {
                match self.get_peer_info(peer_id).await {
                    Ok(Some(info)) => {
                        let _ = self.event_tx.send(NetworkEvent::PeerHealthChanged(peer_id, info.health));
                    }
                    Ok(None) => log::debug!("Peer {} not found", peer_id),
                    Err(e) => log::error!("Failed to get peer info for {}: {}", peer_id, e),
                }
            }
            NetworkCommand::BlacklistPeer(peer_id, reason) => {
                if let Err(e) = self.blacklist_peer(peer_id, reason).await {
                    log::error!("Failed to blacklist peer {}: {}", peer_id, e);
                }
            }
            NetworkCommand::UnblacklistPeer(peer_id) => {
                if let Err(e) = self.unblacklist_peer(peer_id).await {
                    log::error!("Failed to unblacklist peer {}: {}", peer_id, e);
                }
            }
            NetworkCommand::GetMessageQueueStats => {
                match self.get_message_queue_stats().await {
                    Ok(stats) => {
                        let _ = self.event_tx.send(NetworkEvent::MessageQueueStats(stats));
                    }
                    Err(e) => log::error!("Failed to get message queue stats: {}", e),
                }
            }
        }

        Ok(())
    }

    /// Broadcast a block to all connected peers
    async fn broadcast_block(&self, block: Box<FinalizedBlock>) -> Result<()> {
        let block_hash = format!("{:?}", block.get_hash());
        let block_height = block.get_height();

        // First announce the block
        let announcement = P2PMessage::BlockAnnouncement {
            block_hash: block_hash.clone(),
            block_height,
        };
        self.broadcast_message(announcement).await?;

        // Cache the block for potential requests
        self.block_cache
            .lock()
            .unwrap()
            .insert(block_hash.clone(), *block.clone());

        // Send full block data to select peers (flood control)
        let connected_peers: Vec<PeerId> = self.peers.lock().unwrap().keys().cloned().collect();
        let target_peers = std::cmp::min(connected_peers.len(), 5); // Send to max 5 peers initially

        for peer_id in connected_peers.into_iter().take(target_peers) {
            // Send block data directly
            let block_data = P2PMessage::BlockData {
                block: block.clone(),
            };
            if let Err(e) = self.send_to_peer(peer_id, block_data).await {
                log::debug!("Failed to send block to {}: {}", peer_id, e);
            }
        }

        self.stats.lock().unwrap().blocks_propagated += 1;
        log::info!(
            "Broadcasted block {} (height: {}) to network",
            block_hash,
            block_height
        );
        Ok(())
    }

    /// Broadcast a transaction to all connected peers
    async fn broadcast_transaction(&self, transaction: Transaction) -> Result<()> {
        let tx_hash = format!("{:?}", transaction.hash());

        // Cache transaction for potential requests
        self.transaction_pool
            .lock()
            .unwrap()
            .insert(tx_hash.clone(), transaction.clone());

        // Announce transaction
        let announcement = P2PMessage::TransactionAnnouncement {
            tx_hash: tx_hash.clone(),
        };
        self.broadcast_message(announcement).await?;

        // Send transaction data to a subset of peers
        let message = P2PMessage::TransactionData {
            transaction: Box::new(transaction),
        };
        self.broadcast_message(message).await?;

        self.stats.lock().unwrap().transactions_propagated += 1;
        log::debug!("Broadcasted transaction {} to network", tx_hash);
        Ok(())
    }

    /// Broadcast a message to all connected peers
    async fn broadcast_message(&self, message: P2PMessage) -> Result<()> {
        let peers = self.peers.lock().unwrap();
        let mut failed_peers = Vec::new();

        for (peer_id, connection) in peers.iter() {
            if connection.is_active {
                if let Err(e) = connection.message_tx.send(message.clone()) {
                    log::debug!("Failed to send message to {}: {}", peer_id, e);
                    failed_peers.push(*peer_id);
                } else {
                    self.stats.lock().unwrap().messages_sent += 1;
                }
            }
        }

        // Mark failed peers as inactive
        drop(peers);
        if !failed_peers.is_empty() {
            let mut peers = self.peers.lock().unwrap();
            for peer_id in failed_peers {
                if let Some(connection) = peers.get_mut(&peer_id) {
                    connection.is_active = false;
                }
            }
        }

        Ok(())
    }

    /// Send a message to a specific peer
    async fn send_to_peer(&self, peer_id: PeerId, message: P2PMessage) -> Result<()> {
        let peers = self.peers.lock().unwrap();
        if let Some(connection) = peers.get(&peer_id) {
            if connection.is_active {
                connection
                    .message_tx
                    .send(message)
                    .map_err(|e| format_err!("Failed to send to peer {}: {}", peer_id, e))?;
                self.stats.lock().unwrap().messages_sent += 1;
            } else {
                return Err(format_err!("Peer {} is not active", peer_id));
            }
        } else {
            return Err(format_err!("Peer {} not connected", peer_id));
        }
        Ok(())
    }

    /// Request peer discovery from connected peers
    async fn request_peer_discovery(&self) -> Result<()> {
        let request = P2PMessage::PeerList { peers: vec![] }; // Empty list means request
        self.broadcast_message(request).await?;
        Ok(())
    }

    /// Broadcast status update
    async fn broadcast_status_update(&self, height: i32) -> Result<()> {
        let status = P2PMessage::StatusUpdate {
            best_height: height,
        };
        self.broadcast_message(status).await?;
        log::debug!("Broadcasted status update: height {}", height);
        Ok(())
    }

    /// Print peer information
    async fn print_peer_info(&self) {
        let peers = self.peers.lock().unwrap();
        let stats = self.stats.lock().unwrap();

        log::info!("=== P2P Network Status ===");
        log::info!("Connected peers: {}", peers.len());
        log::info!("Total connections: {}", stats.total_connections);
        log::info!("Messages sent: {}", stats.messages_sent);
        log::info!("Messages received: {}", stats.messages_received);
        log::info!("Blocks propagated: {}", stats.blocks_propagated);
        log::info!("Transactions propagated: {}", stats.transactions_propagated);

        for (peer_id, connection) in peers.iter() {
            log::info!(
                "  {} at {} (height: {}, active: {}, connected: {:?})",
                peer_id,
                connection.address,
                connection.best_height,
                connection.is_active,
                connection.connected_at.elapsed()
            );
        }
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

    /// Get network statistics
    pub fn get_stats(&self) -> NetworkStats {
        self.stats.lock().unwrap().clone()
    }

    /// Add a known peer for discovery
    pub fn add_known_peer(&self, addr: SocketAddr) {
        self.known_peers.lock().unwrap().insert(addr);
    }

    /// Remove a known peer
    pub fn remove_known_peer(&self, addr: SocketAddr) {
        self.known_peers.lock().unwrap().remove(&addr);
    }

    /// Send a message with priority through the message queue
    async fn send_priority_message(
        &self,
        message: P2PMessage,
        priority: MessagePriority,
        target_peer: Option<PeerId>,
    ) -> Result<()> {
        // Serialize message to bytes
        let message_data = bincode::serialize(&message)
            .map_err(|e| format_err!("Failed to serialize message: {}", e))?;
        
        let message_id = format!("{:?}_{}", message, SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos());
        
        let prioritized_message = PrioritizedMessage::new(
            message_id,
            priority,
            message_data,
            target_peer,
        );

        if let Ok(mut queue) = self.message_queue.lock() {
            queue.enqueue(prioritized_message)?;
        }

        Ok(())
    }

    /// Send broadcast message with priority
    async fn broadcast_priority_message(
        &self,
        message: P2PMessage,
        priority: MessagePriority,
    ) -> Result<()> {
        let peer_ids: Vec<PeerId> = {
            let peers = self.peers.lock().unwrap();
            peers.keys().cloned().collect()
        };
        
        for peer_id in peer_ids {
            self.send_priority_message(message.clone(), priority, Some(peer_id)).await?;
        }
        Ok(())
    }

    /// Get network health information
    #[allow(clippy::await_holding_lock)]
    pub async fn get_network_health(&self) -> Result<crate::network::network_manager::NetworkTopology> {
        let topology = {
            let manager = self.network_manager.lock()
                .map_err(|_| format_err!("Failed to access network manager"))?;
            manager.get_network_topology().await
        };
        Ok(topology)
    }

    /// Get peer information
    #[allow(clippy::await_holding_lock)]
    pub async fn get_peer_info(&self, peer_id: PeerId) -> Result<Option<NetPeerInfo>> {
        let result = {
            let manager = self.network_manager.lock()
                .map_err(|_| format_err!("Failed to access network manager"))?;
            manager.get_peer_info(peer_id).await
        };
        result
    }

    /// Add peer to blacklist
    #[allow(clippy::await_holding_lock)]
    pub async fn blacklist_peer(&self, peer_id: PeerId, reason: String) -> Result<()> {
        let result = {
            let manager = self.network_manager.lock()
                .map_err(|_| format_err!("Failed to access network manager"))?;
            manager.blacklist_peer(peer_id, reason).await
        };
        result
    }

    /// Remove peer from blacklist
    #[allow(clippy::await_holding_lock)]
    pub async fn unblacklist_peer(&self, peer_id: PeerId) -> Result<()> {
        let result = {
            let manager = self.network_manager.lock()
                .map_err(|_| format_err!("Failed to access network manager"))?;
            manager.unblacklist_peer(peer_id).await
        };
        result
    }

    /// Get message queue statistics
    #[allow(clippy::await_holding_lock)]
    pub async fn get_message_queue_stats(&self) -> Result<crate::network::message_priority::QueueStats> {
        let stats = {
            let queue = self.message_queue.lock()
                .map_err(|_| format_err!("Failed to access message queue"))?;
            queue.get_stats().await
        };
        Ok(stats)
    }
}
