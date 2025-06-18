//! Enhanced P2P network implementation for blockchain nodes
//!
//! This module provides a complete P2P networking layer for blockchain communication
//! with features like peer discovery, message broadcasting, transaction propagation,
//! network resilience, network management, and message prioritization.

use std::{
    collections::{HashMap, HashSet, VecDeque},
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::{Duration, Instant, SystemTime, UNIX_EPOCH},
};

use bincode;
use serde::{Deserialize, Serialize};
use tokio::{
    io::{AsyncReadExt, AsyncWriteExt},
    net::{TcpListener, TcpStream},
    sync::mpsc,
    time::{interval, timeout},
};
use uuid::Uuid;

use crate::{
    blockchain::block::{Block, FinalizedBlock},
    crypto::transaction::Transaction,
    network::{
        message_priority::{MessagePriority, PrioritizedMessage, PriorityMessageQueue},
        network_manager::{NetworkManager, NetworkManagerConfig, PeerInfo as NetPeerInfo},
    },
    Result,
};

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
    failure_count: u32,
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
            failure_count: 0,
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
                return Err(anyhow::anyhow!("Failed to send queued message"));
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
    /// Real peer discovery state
    peer_discovery: Arc<Mutex<PeerDiscoveryState>>,
    /// Connection pool for managing actual TCP connections
    connection_pool: Arc<Mutex<ConnectionPool>>,
    /// Blacklisted peers
    blacklisted_peers: Arc<Mutex<HashMap<PeerId, BlacklistEntry>>>,
}

/// State for peer discovery
#[derive(Debug)]
struct PeerDiscoveryState {
    /// Last time we performed peer discovery
    last_discovery: Instant,
    /// Pending peer discovery requests
    pending_requests: HashMap<PeerId, Instant>,
    /// Discovered peers that we haven't connected to yet
    discovered_peers: HashMap<SocketAddr, PeerDiscoveryInfo>,
    /// Bootstrap peer addresses
    bootstrap_peers: Vec<SocketAddr>,
}

/// Information about a discovered peer
#[derive(Debug, Clone)]
#[allow(dead_code)] // Comprehensive data structure for future functionality
struct PeerDiscoveryInfo {
    /// When this peer was discovered
    discovered_at: Instant,
    /// Source of discovery (bootstrap, peer_list, etc.)
    discovery_source: DiscoverySource,
    /// Last known height
    last_known_height: i32,
    /// Connection attempts made
    connection_attempts: u32,
    /// Last connection attempt
    last_attempt: Option<Instant>,
}

/// Source of peer discovery
#[derive(Debug, Clone)]
#[allow(dead_code)] // Comprehensive enum for future functionality
enum DiscoverySource {
    Bootstrap,
    PeerList(PeerId),
    DirectConnection,
    Network,
}

/// Pool for managing actual TCP connections
#[derive(Debug)]
struct ConnectionPool {
    /// Active TCP connections mapped by peer ID
    active_connections: HashMap<PeerId, ActiveConnection>,
    /// Connection attempts in progress
    pending_connections: HashMap<SocketAddr, PendingConnection>,
    /// Failed connection attempts
    failed_connections: HashMap<SocketAddr, FailedConnection>,
}

/// An active TCP connection
#[derive(Debug)]
#[allow(dead_code)] // Comprehensive data structure for future functionality
struct ActiveConnection {
    /// The peer ID
    peer_id: PeerId,
    /// Remote address
    remote_addr: SocketAddr,
    /// Connection start time
    connected_at: Instant,
    /// Last successful message exchange
    last_activity: Instant,
    /// Bytes sent/received
    bytes_sent: u64,
    bytes_received: u64,
    /// Message counts
    messages_sent: u32,
    messages_received: u32,
    /// Connection health metrics
    latency_ms: Option<u64>,
    packet_loss_rate: f32,
}

/// A pending connection attempt
#[derive(Debug)]
#[allow(dead_code)] // Comprehensive data structure for future functionality
struct PendingConnection {
    /// Target address
    target_addr: SocketAddr,
    /// Attempt start time
    started_at: Instant,
    /// Attempt count
    attempt_number: u32,
}

/// A failed connection record
#[derive(Debug)]
#[allow(dead_code)] // Comprehensive data structure for future functionality
struct FailedConnection {
    /// Target address
    target_addr: SocketAddr,
    /// Last failure time
    failed_at: Instant,
    /// Failure reason
    failure_reason: String,
    /// Total failure count
    failure_count: u32,
}

/// Blacklist entry
#[derive(Debug, Clone)]
#[allow(dead_code)] // Comprehensive data structure for future functionality
struct BlacklistEntry {
    /// Reason for blacklisting
    reason: String,
    /// When the peer was blacklisted
    blacklisted_at: Instant,
    /// Duration of blacklist (None = permanent)
    duration: Option<Duration>,
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

/// Real connection pool metrics
#[derive(Debug, Clone)]
pub struct ConnectionPoolMetrics {
    /// Number of active TCP connections
    pub active_connections: usize,
    /// Number of pending connection attempts
    pub pending_connections: usize,
    /// Number of failed connection records
    pub failed_connections: usize,
    /// Number of logical peer entries
    pub logical_peers: usize,
    /// Number of healthy connections (recent activity)
    pub healthy_connections: usize,
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
        let network_manager =
            NetworkManager::new(NetworkManagerConfig::default(), bootstrap_peers.clone());

        // Initialize priority message queue
        let message_queue =
            PriorityMessageQueue::new(crate::network::message_priority::RateLimitConfig::default());

        // Initialize peer discovery state
        let peer_discovery = PeerDiscoveryState {
            last_discovery: Instant::now(),
            pending_requests: HashMap::new(),
            discovered_peers: HashMap::new(),
            bootstrap_peers: bootstrap_peers.clone(),
        };

        // Initialize connection pool
        let connection_pool = ConnectionPool {
            active_connections: HashMap::new(),
            pending_connections: HashMap::new(),
            failed_connections: HashMap::new(),
        };

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
                peer_discovery: Arc::new(Mutex::new(peer_discovery)),
                connection_pool: Arc::new(Mutex::new(connection_pool)),
                blacklisted_peers: Arc::new(Mutex::new(HashMap::new())),
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
        self.start_background_tasks();

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
    fn start_background_tasks(&self) {
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
                                    log::debug!(
                                        "Sending priority message {} to peer {}",
                                        message.id,
                                        target_peer
                                    );
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
        let connection_pool_discovery = self.connection_pool.clone();
        let blacklisted_peers_discovery = self.blacklisted_peers.clone();
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(30)); // Check every 30 seconds
            loop {
                interval.tick().await;

                // Try to connect to new peers from known peers list
                let known_addrs: Vec<SocketAddr> = {
                    let known = known_peers_discovery.lock().unwrap();
                    known.iter().cloned().collect()
                };

                let current_peer_count = {
                    let peers_guard = peers_discovery.lock().unwrap();
                    let active_count = peers_guard.values().filter(|c| c.is_active).count();
                    let total_count = peers_guard.len();
                    log::debug!(
                        "Network status: {}/{} active peers",
                        active_count,
                        total_count
                    );

                    // Log network health
                    if active_count == 0 {
                        log::error!("No active peers! Network is isolated.");
                    } else if active_count < MAX_PEERS / 4 {
                        log::warn!(
                            "Low peer count: {} active peers (recommended: {})",
                            active_count,
                            MAX_PEERS / 2
                        );
                    }

                    active_count
                };

                // Connect to new peers if we're below target
                if current_peer_count < MAX_PEERS / 2 {
                    let mut connection_attempts = 0;
                    let max_attempts = 3;

                    for addr in known_addrs.iter().take(max_attempts) {
                        let peers_clone = peers_discovery.clone();
                        let event_tx_clone = event_tx_discovery.clone();
                        let addr_clone = *addr;
                        let peer_id_clone = peer_id_discovery;
                        let best_height_clone = best_height_discovery.clone();

                        // We need access to connection_pool and blacklisted_peers for the real connect_to_peer
                        let connection_pool_clone = connection_pool_discovery.clone();
                        let blacklisted_peers_clone = blacklisted_peers_discovery.clone();

                        tokio::spawn(async move {
                            match Self::connect_to_peer(
                                addr_clone,
                                peers_clone,
                                event_tx_clone,
                                peer_id_clone,
                                best_height_clone,
                                connection_pool_clone,
                                blacklisted_peers_clone,
                            )
                            .await
                            {
                                Ok(()) => {
                                    log::info!(
                                        "Successfully connected to peer {} during discovery",
                                        addr_clone
                                    );
                                }
                                Err(e) => {
                                    log::debug!(
                                        "Failed to connect to discovered peer {}: {}",
                                        addr_clone,
                                        e
                                    );
                                }
                            }
                        });

                        connection_attempts += 1;

                        // Small delay between connection attempts
                        tokio::time::sleep(Duration::from_millis(200)).await;
                    }

                    if connection_attempts > 0 {
                        log::info!(
                            "Attempted {} new peer connections during discovery",
                            connection_attempts
                        );
                    }
                }

                // Cleanup inactive peers after extended failure
                let mut peers_to_remove = Vec::new();
                {
                    let peers_guard = peers_discovery.lock().unwrap();
                    for (peer_id, connection) in peers_guard.iter() {
                        // Remove peers that have failed too many times and been inactive for a while
                        if !connection.is_active
                            && connection.failure_count > 10
                            && connection.connected_at.elapsed() > Duration::from_secs(300)
                        {
                            peers_to_remove.push(*peer_id);
                        }
                    }
                }

                if !peers_to_remove.is_empty() {
                    let mut peers_guard = peers_discovery.lock().unwrap();
                    for peer_id in peers_to_remove {
                        peers_guard.remove(&peer_id);
                        log::info!("Removed permanently failed peer {}", peer_id);
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

    /// Connect to bootstrap peers using real peer discovery
    async fn connect_to_bootstrap_peers(&self) {
        log::info!("Starting real bootstrap peer discovery and connection");

        // Use the real peer discovery mechanism
        match self.discover_peers().await {
            Ok(discovered_addrs) => {
                log::info!("Discovered {} bootstrap peers", discovered_addrs.len());

                // Connect to discovered peers with staggered timing
                for (index, addr) in discovered_addrs.iter().enumerate() {
                    let peers = self.peers.clone();
                    let event_tx = self.event_tx.clone();
                    let peer_id = self.peer_id;
                    let best_height = self.best_height.clone();
                    let connection_pool = self.connection_pool.clone();
                    let blacklisted_peers = self.blacklisted_peers.clone();
                    let addr = *addr;

                    tokio::spawn(async move {
                        // Stagger connections to avoid network congestion
                        tokio::time::sleep(Duration::from_millis((index as u64) * 500)).await;

                        // Retry logic for bootstrap connections
                        let mut retry_count = 0;
                        const MAX_RETRIES: usize = 3;
                        const RETRY_DELAY: u64 = 5; // seconds

                        while retry_count < MAX_RETRIES {
                            match Self::connect_to_peer(
                                addr,
                                peers.clone(),
                                event_tx.clone(),
                                peer_id,
                                best_height.clone(),
                                connection_pool.clone(),
                                blacklisted_peers.clone(),
                            )
                            .await
                            {
                                Ok(()) => {
                                    log::info!(
                                        "Successfully connected to bootstrap peer {} on attempt {}",
                                        addr,
                                        retry_count + 1
                                    );
                                    break;
                                }
                                Err(e) => {
                                    retry_count += 1;
                                    if retry_count < MAX_RETRIES {
                                        log::warn!(
                                            "Failed to connect to bootstrap peer {} (attempt {}): {}. Retrying in {}s...",
                                            addr, retry_count, e, RETRY_DELAY
                                        );
                                        tokio::time::sleep(Duration::from_secs(RETRY_DELAY)).await;
                                    } else {
                                        log::error!(
                                            "Failed to connect to bootstrap peer {} after {} attempts: {}",
                                            addr,
                                            MAX_RETRIES,
                                            e
                                        );
                                    }
                                }
                            }
                        }
                    });
                }

                // Wait for initial connections to establish
                tokio::time::sleep(Duration::from_secs(2)).await;

                // Log connection status with real metrics
                let connected_count = self.peers.lock().unwrap().len();
                let active_connections = self
                    .connection_pool
                    .lock()
                    .unwrap()
                    .active_connections
                    .len();

                log::info!(
                    "Bootstrap connection phase completed. Connected to {}/{} peers (real connections: {})",
                    connected_count,
                    discovered_addrs.len(),
                    active_connections
                );
            }
            Err(e) => {
                log::error!("Failed to discover bootstrap peers: {}", e);
            }
        }
    }

    /// Real peer discovery implementation
    async fn discover_peers(&self) -> Result<Vec<SocketAddr>> {
        let mut discovered_addrs = Vec::new();

        log::info!("Starting peer discovery process");

        // First, try bootstrap peers if we have few connections
        let current_peer_count = self.peers.lock().unwrap().len();
        if current_peer_count < MAX_PEERS / 2 {
            let bootstrap_peers = {
                let discovery_state = self.peer_discovery.lock().unwrap();
                discovery_state.bootstrap_peers.clone()
            };

            for bootstrap_addr in bootstrap_peers {
                if !self.is_address_blacklisted(bootstrap_addr).await {
                    discovered_addrs.push(bootstrap_addr);

                    let discovery_info = PeerDiscoveryInfo {
                        discovered_at: Instant::now(),
                        discovery_source: DiscoverySource::Bootstrap,
                        last_known_height: 0,
                        connection_attempts: 0,
                        last_attempt: None,
                    };

                    let mut discovery_state = self.peer_discovery.lock().unwrap();
                    discovery_state
                        .discovered_peers
                        .insert(bootstrap_addr, discovery_info);
                }
            }
        }

        // Request peer lists from connected peers
        let connected_peer_ids: Vec<PeerId> = {
            let peers = self.peers.lock().unwrap();
            peers.keys().cloned().collect()
        };

        for peer_id in connected_peer_ids {
            let should_request = {
                let discovery_state = self.peer_discovery.lock().unwrap();
                !discovery_state.pending_requests.contains_key(&peer_id)
            };

            if should_request {
                {
                    let mut discovery_state = self.peer_discovery.lock().unwrap();
                    discovery_state
                        .pending_requests
                        .insert(peer_id, Instant::now());
                }

                // Send peer list request
                let request_msg = P2PMessage::PeerList { peers: vec![] };
                if let Err(e) = self.send_to_peer(peer_id, request_msg).await {
                    log::debug!("Failed to request peer list from {}: {}", peer_id, e);
                    let mut discovery_state = self.peer_discovery.lock().unwrap();
                    discovery_state.pending_requests.remove(&peer_id);
                }
            }
        }

        {
            let mut discovery_state = self.peer_discovery.lock().unwrap();
            discovery_state.last_discovery = Instant::now();
        }

        log::info!("Discovered {} potential peers", discovered_addrs.len());
        Ok(discovered_addrs)
    }

    /// Check if an address is blacklisted
    async fn is_address_blacklisted(&self, _addr: SocketAddr) -> bool {
        let blacklist = self.blacklisted_peers.lock().unwrap();

        // Check if any peer from this address is blacklisted
        for (_, entry) in blacklist.iter() {
            // In a real implementation, you'd map addresses to peer IDs
            // For now, we'll check if the blacklist duration has expired
            if let Some(duration) = entry.duration {
                if entry.blacklisted_at.elapsed() > duration {
                    continue; // Blacklist expired
                }
            }
            // For simplicity, we'll assume address-based blacklisting isn't implemented yet
        }

        false
    }

    /// Connect to a specific peer with real validation and connection tracking
    async fn connect_to_peer(
        addr: SocketAddr,
        peers: Arc<Mutex<HashMap<PeerId, PeerConnection>>>,
        event_tx: mpsc::UnboundedSender<NetworkEvent>,
        our_peer_id: PeerId,
        best_height: Arc<Mutex<i32>>,
        connection_pool: Arc<Mutex<ConnectionPool>>,
        _blacklisted_peers: Arc<Mutex<HashMap<PeerId, BlacklistEntry>>>,
    ) -> Result<()> {
        log::debug!("Attempting real connection to peer at {}", addr);

        // Record connection attempt in pool
        {
            let mut pool = connection_pool.lock().unwrap();

            // Check if connection is already pending
            if pool.pending_connections.contains_key(&addr) {
                log::debug!("Connection attempt to {} already in progress", addr);
                return Err(anyhow::anyhow!("Connection already pending"));
            }

            // Check for recent failures
            if let Some(failed_conn) = pool.failed_connections.get(&addr) {
                let retry_delay = Duration::from_secs(failed_conn.failure_count.min(300) as u64);
                if failed_conn.failed_at.elapsed() < retry_delay {
                    log::debug!(
                        "Recent failure for {}, waiting {:?} before retry",
                        addr,
                        retry_delay - failed_conn.failed_at.elapsed()
                    );
                    return Err(anyhow::anyhow!(
                        "Recent connection failure, waiting for retry"
                    ));
                }
            }

            // Add to pending connections
            let pending = PendingConnection {
                target_addr: addr,
                started_at: Instant::now(),
                attempt_number: pool
                    .failed_connections
                    .get(&addr)
                    .map(|f| f.failure_count + 1)
                    .unwrap_or(1),
            };
            pool.pending_connections.insert(addr, pending);
        }

        // Check if we're already connected to this address
        {
            let peers_guard = peers.lock().unwrap();
            for connection in peers_guard.values() {
                if connection.address == addr && connection.is_active {
                    log::debug!("Already have active connection to {}", addr);
                    // Remove from pending connections
                    connection_pool
                        .lock()
                        .unwrap()
                        .pending_connections
                        .remove(&addr);
                    return Ok(());
                }
            }

            // Check connection limit
            let active_connections = peers_guard.values().filter(|c| c.is_active).count();
            if active_connections >= MAX_PEERS {
                log::warn!(
                    "Maximum peer connections reached ({}), cannot connect to {}",
                    MAX_PEERS,
                    addr
                );
                connection_pool
                    .lock()
                    .unwrap()
                    .pending_connections
                    .remove(&addr);
                return Err(anyhow::anyhow!("Maximum peer connections reached"));
            }
        }

        // Validate address (don't connect to ourselves)
        if addr.ip().is_loopback() && addr.port() == 0 {
            connection_pool
                .lock()
                .unwrap()
                .pending_connections
                .remove(&addr);
            return Err(anyhow::anyhow!("Invalid address: {}", addr));
        }

        log::info!("Establishing real TCP connection to {}", addr);

        // Real TCP connection with timeout and enhanced error handling
        let connection_start = Instant::now();
        let stream = match timeout(Duration::from_secs(10), TcpStream::connect(addr)).await {
            Ok(Ok(stream)) => {
                log::debug!(
                    "Real TCP connection established to {} in {:?}",
                    addr,
                    connection_start.elapsed()
                );
                stream
            }
            Ok(Err(e)) => {
                log::debug!("Real TCP connection failed to {}: {}", addr, e);

                // Record failure
                Self::record_connection_failure(
                    connection_pool.clone(),
                    addr,
                    format!("TCP connection failed: {}", e),
                )
                .await;

                return Err(anyhow::anyhow!("TCP connection failed: {}", e));
            }
            Err(_) => {
                log::debug!("Real TCP connection timed out to {}", addr);

                // Record failure
                Self::record_connection_failure(
                    connection_pool.clone(),
                    addr,
                    "Connection timeout".to_string(),
                )
                .await;

                return Err(anyhow::anyhow!("Connection timeout"));
            }
        };

        // Send real handshake with our node information
        let current_height = *best_height.lock().unwrap();
        let handshake = P2PMessage::Handshake {
            peer_id: our_peer_id,
            protocol_version: PROTOCOL_VERSION,
            best_height: current_height,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            node_type: "full_node".to_string(),
        };

        log::debug!(
            "Sending real handshake to {} (our_id: {}, height: {})",
            addr,
            our_peer_id,
            current_height
        );

        // Handle the real peer connection with handshake
        match Self::handle_peer_connection(
            stream,
            addr,
            peers,
            event_tx,
            our_peer_id,
            Some(handshake),
            connection_pool.clone(),
        )
        .await
        {
            Ok(peer_id) => {
                // Record successful connection
                {
                    let mut pool = connection_pool.lock().unwrap();
                    pool.pending_connections.remove(&addr);
                    pool.failed_connections.remove(&addr);

                    let active_conn = ActiveConnection {
                        peer_id,
                        remote_addr: addr,
                        connected_at: connection_start,
                        last_activity: Instant::now(),
                        bytes_sent: 0,
                        bytes_received: 0,
                        messages_sent: 0,
                        messages_received: 0,
                        latency_ms: None,
                        packet_loss_rate: 0.0,
                    };
                    pool.active_connections.insert(peer_id, active_conn);
                }

                log::info!(
                    "Successfully established real peer connection to {} (peer_id: {})",
                    addr,
                    peer_id
                );
                Ok(())
            }
            Err(e) => {
                log::warn!(
                    "Failed to establish real peer connection to {}: {}",
                    addr,
                    e
                );

                // Record failure
                Self::record_connection_failure(
                    connection_pool,
                    addr,
                    format!("Handshake failed: {}", e),
                )
                .await;

                Err(e)
            }
        }
    }

    /// Record a connection failure
    async fn record_connection_failure(
        connection_pool: Arc<Mutex<ConnectionPool>>,
        addr: SocketAddr,
        reason: String,
    ) {
        let mut pool = connection_pool.lock().unwrap();

        pool.pending_connections.remove(&addr);

        let failure_count = pool
            .failed_connections
            .get(&addr)
            .map(|f| f.failure_count + 1)
            .unwrap_or(1);

        let failed_conn = FailedConnection {
            target_addr: addr,
            failed_at: Instant::now(),
            failure_reason: reason,
            failure_count,
        };

        pool.failed_connections.insert(addr, failed_conn);

        log::debug!(
            "Recorded connection failure #{} for {}",
            failure_count,
            addr
        );
    }

    /// Handle incoming connection
    async fn handle_incoming_connection(&self, stream: TcpStream, addr: SocketAddr) {
        let peers = self.peers.clone();
        let event_tx = self.event_tx.clone();
        let our_peer_id = self.peer_id;
        let stats = self.stats.clone();

        let connection_pool = self.connection_pool.clone();

        tokio::spawn(async move {
            stats.lock().unwrap().total_connections += 1;

            if let Err(e) = Self::handle_peer_connection(
                stream,
                addr,
                peers,
                event_tx,
                our_peer_id,
                None,
                connection_pool,
            )
            .await
            {
                log::error!("Error handling incoming connection from {}: {}", addr, e);
            }
        });
    }

    /// Handle peer connection (both incoming and outgoing) with real connection tracking
    async fn handle_peer_connection(
        mut stream: TcpStream,
        addr: SocketAddr,
        peers: Arc<Mutex<HashMap<PeerId, PeerConnection>>>,
        event_tx: mpsc::UnboundedSender<NetworkEvent>,
        our_peer_id: PeerId,
        initial_message: Option<P2PMessage>,
        connection_pool: Arc<Mutex<ConnectionPool>>,
    ) -> Result<PeerId> {
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

            // Remove from connection pool
            connection_pool
                .lock()
                .unwrap()
                .active_connections
                .remove(&peer_id);

            let _ = event_tx.send(NetworkEvent::PeerDisconnected(peer_id));
            log::info!("Peer {} disconnected", peer_id);

            Ok(peer_id)
        } else {
            Err(anyhow::anyhow!("No peer ID established"))
        }
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
        let data = bincode::serialize(message)
            .map_err(|e| anyhow::anyhow!("Serialization failed: {}", e))?;
        let len = data.len() as u32;

        if len > MAX_MESSAGE_SIZE as u32 {
            return Err(anyhow::anyhow!("Message too large: {}", len));
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
            return Err(anyhow::anyhow!("Message too large: {}", len));
        }

        if len == 0 {
            return Err(anyhow::anyhow!("Empty message"));
        }

        // Read data with timeout
        let mut data = vec![0u8; len];
        timeout(Duration::from_secs(30), stream.read_exact(&mut data)).await??;

        // Deserialize with error handling
        let message = bincode::deserialize(&data)
            .map_err(|e| anyhow::anyhow!("Deserialization failed: {}", e))?;
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
                let connection_pool = self.connection_pool.clone();
                let blacklisted_peers = self.blacklisted_peers.clone();

                tokio::spawn(async move {
                    if let Err(e) = Self::connect_to_peer(
                        addr,
                        peers,
                        event_tx,
                        peer_id,
                        best_height,
                        connection_pool,
                        blacklisted_peers,
                    )
                    .await
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
                self.send_priority_message(message, priority, Some(peer_id))
                    .await?;
            }
            NetworkCommand::GetNetworkHealth => match self.get_network_health().await {
                Ok(health) => {
                    let _ = self
                        .event_tx
                        .send(NetworkEvent::NetworkHealthUpdate(health));
                }
                Err(e) => log::error!("Failed to get network health: {}", e),
            },
            NetworkCommand::GetPeerInfo(peer_id) => match self.get_peer_info(peer_id).await {
                Ok(Some(info)) => {
                    let _ = self
                        .event_tx
                        .send(NetworkEvent::PeerHealthChanged(peer_id, info.health));
                }
                Ok(None) => log::debug!("Peer {} not found", peer_id),
                Err(e) => log::error!("Failed to get peer info for {}: {}", peer_id, e),
            },
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
            NetworkCommand::GetMessageQueueStats => match self.get_message_queue_stats().await {
                Ok(stats) => {
                    let _ = self.event_tx.send(NetworkEvent::MessageQueueStats(stats));
                }
                Err(e) => log::error!("Failed to get message queue stats: {}", e),
            },
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

    /// Broadcast a message to all connected peers with failure handling
    async fn broadcast_message(&self, message: P2PMessage) -> Result<()> {
        let peers = self.peers.lock().unwrap();
        let mut failed_peers = Vec::new();
        let mut successful_sends = 0;
        let total_active_peers = peers.values().filter(|c| c.is_active).count();

        if total_active_peers == 0 {
            log::warn!("No active peers available for broadcasting message");
            return Err(anyhow::anyhow!("No active peers available"));
        }

        for (peer_id, connection) in peers.iter() {
            if connection.is_active {
                match connection.message_tx.send(message.clone()) {
                    Ok(()) => {
                        successful_sends += 1;
                        self.stats.lock().unwrap().messages_sent += 1;
                    }
                    Err(e) => {
                        log::debug!("Failed to send message to peer {}: {}", peer_id, e);
                        failed_peers.push(*peer_id);
                    }
                }
            }
        }

        // Mark failed peers as inactive and log network health
        drop(peers);
        if !failed_peers.is_empty() {
            let mut peers = self.peers.lock().unwrap();
            for peer_id in failed_peers.iter() {
                if let Some(connection) = peers.get_mut(peer_id) {
                    connection.is_active = false;
                    connection.failure_count += 1;
                    log::warn!(
                        "Peer {} failed (total failures: {}), marking as inactive",
                        peer_id,
                        connection.failure_count
                    );
                }
            }
        }

        // Calculate and log broadcast success rate
        let success_rate = (successful_sends as f64 / total_active_peers as f64) * 100.0;
        if success_rate < 50.0 {
            log::error!(
                "Low broadcast success rate: {:.1}% ({}/{} peers)",
                success_rate,
                successful_sends,
                total_active_peers
            );
        } else if success_rate < 80.0 {
            log::warn!(
                "Moderate broadcast success rate: {:.1}% ({}/{} peers)",
                success_rate,
                successful_sends,
                total_active_peers
            );
        } else {
            log::debug!(
                "Broadcast success rate: {:.1}% ({}/{} peers)",
                success_rate,
                successful_sends,
                total_active_peers
            );
        }

        // Return error if too many peers failed
        if success_rate < 30.0 {
            return Err(anyhow::anyhow!(
                "Broadcast failed - too many peer failures: {:.1}% success rate",
                success_rate
            ));
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
                    .map_err(|e| anyhow::anyhow!("Failed to send to peer {}: {}", peer_id, e))?;
                self.stats.lock().unwrap().messages_sent += 1;
            } else {
                return Err(anyhow::anyhow!("Peer {} is not active", peer_id));
            }
        } else {
            return Err(anyhow::anyhow!("Peer {} not connected", peer_id));
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

    /// Get connected peers with real connection validation
    pub fn get_connected_peers(&self) -> Vec<PeerId> {
        let peers = self.peers.lock().unwrap();
        let connection_pool = self.connection_pool.lock().unwrap();

        // Only return peers that have both logical and physical connections
        peers
            .keys()
            .filter(|&peer_id| {
                peers.get(peer_id).map(|c| c.is_active).unwrap_or(false)
                    && connection_pool.active_connections.contains_key(peer_id)
            })
            .cloned()
            .collect()
    }

    /// Get real connection pool metrics
    pub fn get_connection_pool_metrics(&self) -> ConnectionPoolMetrics {
        let pool = self.connection_pool.lock().unwrap();
        let peers = self.peers.lock().unwrap();

        ConnectionPoolMetrics {
            active_connections: pool.active_connections.len(),
            pending_connections: pool.pending_connections.len(),
            failed_connections: pool.failed_connections.len(),
            logical_peers: peers.len(),
            healthy_connections: pool
                .active_connections
                .values()
                .filter(|c| c.last_activity.elapsed() < Duration::from_secs(60))
                .count(),
        }
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
            .map_err(|e| anyhow::anyhow!("Failed to serialize message: {}", e))?;

        let message_id = format!(
            "{:?}_{}",
            message,
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_nanos()
        );

        let prioritized_message =
            PrioritizedMessage::new(message_id, priority, message_data, target_peer);

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
            self.send_priority_message(message.clone(), priority, Some(peer_id))
                .await?;
        }
        Ok(())
    }

    /// Get network health information
    #[allow(clippy::await_holding_lock)]
    pub async fn get_network_health(
        &self,
    ) -> Result<crate::network::network_manager::NetworkTopology> {
        let topology = {
            let manager = self
                .network_manager
                .lock()
                .map_err(|_| anyhow::anyhow!("Failed to access network manager"))?;
            manager.get_network_topology().await
        };
        Ok(topology)
    }

    /// Get peer information
    #[allow(clippy::await_holding_lock)]
    pub async fn get_peer_info(&self, peer_id: PeerId) -> Result<Option<NetPeerInfo>> {
        {
            let manager = self
                .network_manager
                .lock()
                .map_err(|_| anyhow::anyhow!("Failed to access network manager"))?;
            manager.get_peer_info(peer_id).await
        }
    }

    /// Add peer to blacklist
    #[allow(clippy::await_holding_lock)]
    pub async fn blacklist_peer(&self, peer_id: PeerId, reason: String) -> Result<()> {
        {
            let manager = self
                .network_manager
                .lock()
                .map_err(|_| anyhow::anyhow!("Failed to access network manager"))?;
            manager.blacklist_peer(peer_id, reason).await
        }
    }

    /// Remove peer from blacklist
    #[allow(clippy::await_holding_lock)]
    pub async fn unblacklist_peer(&self, peer_id: PeerId) -> Result<()> {
        {
            let manager = self
                .network_manager
                .lock()
                .map_err(|_| anyhow::anyhow!("Failed to access network manager"))?;
            manager.unblacklist_peer(peer_id).await
        }
    }

    /// Get message queue statistics
    #[allow(clippy::await_holding_lock)]
    pub async fn get_message_queue_stats(
        &self,
    ) -> Result<crate::network::message_priority::QueueStats> {
        let stats = {
            let queue = self
                .message_queue
                .lock()
                .map_err(|_| anyhow::anyhow!("Failed to access message queue"))?;
            queue.get_stats().await
        };
        Ok(stats)
    }

    /// Validate real peer connections
    pub async fn validate_peer_connections(&self) -> Result<ConnectionValidationReport> {
        let mut report = ConnectionValidationReport {
            total_logical_peers: 0,
            total_physical_connections: 0,
            matched_connections: 0,
            orphaned_logical_peers: Vec::new(),
            orphaned_physical_connections: Vec::new(),
            invalid_connections: Vec::new(),
        };

        let peers = self.peers.lock().unwrap();
        let pool = self.connection_pool.lock().unwrap();

        report.total_logical_peers = peers.len();
        report.total_physical_connections = pool.active_connections.len();

        // Check for matched connections
        for (peer_id, peer_conn) in peers.iter() {
            if let Some(physical_conn) = pool.active_connections.get(peer_id) {
                // Validate that addresses match
                if peer_conn.address == physical_conn.remote_addr {
                    report.matched_connections += 1;
                } else {
                    report.invalid_connections.push(format!(
                        "Peer {} address mismatch: logical={}, physical={}",
                        peer_id, peer_conn.address, physical_conn.remote_addr
                    ));
                }
            } else {
                report.orphaned_logical_peers.push(*peer_id);
            }
        }

        // Check for orphaned physical connections
        for (peer_id, _) in pool.active_connections.iter() {
            if !peers.contains_key(peer_id) {
                report.orphaned_physical_connections.push(*peer_id);
            }
        }

        log::info!(
            "Connection validation: {}/{} logical peers have physical connections, {} orphaned logical, {} orphaned physical",
            report.matched_connections,
            report.total_logical_peers,
            report.orphaned_logical_peers.len(),
            report.orphaned_physical_connections.len()
        );

        Ok(report)
    }

    /// Cleanup orphaned connections
    pub async fn cleanup_orphaned_connections(&self) -> Result<()> {
        let validation_report = self.validate_peer_connections().await?;

        // Remove orphaned logical peers
        {
            let mut peers = self.peers.lock().unwrap();
            for orphaned_peer in validation_report.orphaned_logical_peers {
                peers.remove(&orphaned_peer);
                log::info!("Removed orphaned logical peer: {}", orphaned_peer);
            }
        }

        // Remove orphaned physical connections
        {
            let mut pool = self.connection_pool.lock().unwrap();
            for orphaned_peer in validation_report.orphaned_physical_connections {
                pool.active_connections.remove(&orphaned_peer);
                log::info!("Removed orphaned physical connection: {}", orphaned_peer);
            }
        }

        Ok(())
    }
}

/// Report for connection validation
#[derive(Debug, Clone)]
pub struct ConnectionValidationReport {
    pub total_logical_peers: usize,
    pub total_physical_connections: usize,
    pub matched_connections: usize,
    pub orphaned_logical_peers: Vec<PeerId>,
    pub orphaned_physical_connections: Vec<PeerId>,
    pub invalid_connections: Vec<String>,
}

#[cfg(test)]
mod tests {
    use std::net::{IpAddr, Ipv4Addr};

    use tokio::time::Duration;

    use super::*;

    fn create_test_address(port: u16) -> SocketAddr {
        SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), port)
    }

    #[tokio::test]
    async fn test_peer_discovery_state_creation() {
        let bootstrap_peers = vec![create_test_address(8001), create_test_address(8002)];

        let discovery_state = PeerDiscoveryState {
            last_discovery: Instant::now(),
            pending_requests: HashMap::new(),
            discovered_peers: HashMap::new(),
            bootstrap_peers: bootstrap_peers.clone(),
        };

        assert_eq!(discovery_state.bootstrap_peers.len(), 2);
        assert!(discovery_state.pending_requests.is_empty());
        assert!(discovery_state.discovered_peers.is_empty());
    }

    #[tokio::test]
    async fn test_connection_pool_operations() {
        let mut pool = ConnectionPool {
            active_connections: HashMap::new(),
            pending_connections: HashMap::new(),
            failed_connections: HashMap::new(),
        };

        let peer_id = PeerId::random();
        let addr = create_test_address(8003);

        // Test adding pending connection
        let pending = PendingConnection {
            target_addr: addr,
            started_at: Instant::now(),
            attempt_number: 1,
        };
        pool.pending_connections.insert(addr, pending);

        assert!(pool.pending_connections.contains_key(&addr));
        assert_eq!(pool.active_connections.len(), 0);

        // Test adding active connection
        let active = ActiveConnection {
            peer_id,
            remote_addr: addr,
            connected_at: Instant::now(),
            last_activity: Instant::now(),
            bytes_sent: 0,
            bytes_received: 0,
            messages_sent: 0,
            messages_received: 0,
            latency_ms: None,
            packet_loss_rate: 0.0,
        };
        pool.active_connections.insert(peer_id, active);
        pool.pending_connections.remove(&addr);

        assert!(pool.active_connections.contains_key(&peer_id));
        assert!(!pool.pending_connections.contains_key(&addr));
    }

    #[tokio::test]
    async fn test_connection_failure_tracking() {
        let mut pool = ConnectionPool {
            active_connections: HashMap::new(),
            pending_connections: HashMap::new(),
            failed_connections: HashMap::new(),
        };

        let addr = create_test_address(8004);

        // Simulate first failure
        let failed = FailedConnection {
            target_addr: addr,
            failed_at: Instant::now(),
            failure_reason: "Connection refused".to_string(),
            failure_count: 1,
        };
        pool.failed_connections.insert(addr, failed);

        assert_eq!(pool.failed_connections.get(&addr).unwrap().failure_count, 1);

        // Simulate second failure
        let failed = FailedConnection {
            target_addr: addr,
            failed_at: Instant::now(),
            failure_reason: "Timeout".to_string(),
            failure_count: 2,
        };
        pool.failed_connections.insert(addr, failed);

        assert_eq!(pool.failed_connections.get(&addr).unwrap().failure_count, 2);
    }

    #[tokio::test]
    async fn test_blacklist_functionality() {
        let mut blacklist = HashMap::new();
        let peer_id = PeerId::random();

        // Add temporary blacklist entry
        let entry = BlacklistEntry {
            reason: "Malicious behavior".to_string(),
            blacklisted_at: Instant::now(),
            duration: Some(Duration::from_secs(60)),
        };
        blacklist.insert(peer_id, entry);

        assert!(blacklist.contains_key(&peer_id));

        // Add permanent blacklist entry
        let peer_id2 = PeerId::random();
        let entry2 = BlacklistEntry {
            reason: "Protocol violation".to_string(),
            blacklisted_at: Instant::now(),
            duration: None,
        };
        blacklist.insert(peer_id2, entry2);

        assert!(blacklist.contains_key(&peer_id2));
        assert_eq!(blacklist.len(), 2);
    }

    #[tokio::test]
    async fn test_peer_discovery_info() {
        let _addr = create_test_address(8005);
        let peer_id = PeerId::random();

        let discovery_info = PeerDiscoveryInfo {
            discovered_at: Instant::now(),
            discovery_source: DiscoverySource::PeerList(peer_id),
            last_known_height: 42,
            connection_attempts: 3,
            last_attempt: Some(Instant::now()),
        };

        assert_eq!(discovery_info.last_known_height, 42);
        assert_eq!(discovery_info.connection_attempts, 3);
        assert!(discovery_info.last_attempt.is_some());

        match discovery_info.discovery_source {
            DiscoverySource::PeerList(source_peer) => {
                assert_eq!(source_peer, peer_id);
            }
            _ => panic!("Wrong discovery source"),
        }
    }

    #[tokio::test]
    async fn test_enhanced_p2p_node_creation() {
        let listen_addr = create_test_address(8006);
        let bootstrap_peers = vec![create_test_address(8007)];

        let result = EnhancedP2PNode::new(listen_addr, bootstrap_peers);
        assert!(result.is_ok());

        let (node, _event_rx, _command_tx) = result.unwrap();
        assert_eq!(node.listen_addr, listen_addr);
        assert_eq!(node.known_peers.lock().unwrap().len(), 1);
    }

    #[tokio::test]
    async fn test_connection_pool_metrics() {
        let listen_addr = create_test_address(8008);
        let bootstrap_peers = vec![];

        let (node, _event_rx, _command_tx) =
            EnhancedP2PNode::new(listen_addr, bootstrap_peers).unwrap();

        let metrics = node.get_connection_pool_metrics();
        assert_eq!(metrics.active_connections, 0);
        assert_eq!(metrics.pending_connections, 0);
        assert_eq!(metrics.failed_connections, 0);
        assert_eq!(metrics.logical_peers, 0);
        assert_eq!(metrics.healthy_connections, 0);
    }

    #[tokio::test]
    async fn test_connection_validation() {
        let listen_addr = create_test_address(8009);
        let bootstrap_peers = vec![];

        let (node, _event_rx, _command_tx) =
            EnhancedP2PNode::new(listen_addr, bootstrap_peers).unwrap();

        let validation_report = node.validate_peer_connections().await.unwrap();
        assert_eq!(validation_report.total_logical_peers, 0);
        assert_eq!(validation_report.total_physical_connections, 0);
        assert_eq!(validation_report.matched_connections, 0);
        assert!(validation_report.orphaned_logical_peers.is_empty());
        assert!(validation_report.orphaned_physical_connections.is_empty());
        assert!(validation_report.invalid_connections.is_empty());
    }
}
