//! High-level network manager for blockchain P2P communication
//!
//! This module provides a simplified interface for blockchain applications
//! to interact with the libp2p network layer.

use crate::blockchain::block::Block;
use crate::blockchain::blockchain::Blockchain;
use crate::crypto::transaction::Transaction;
use crate::network::network_config::NetworkConfig;
use crate::network::p2p::{NetworkCommand, NetworkEvent, P2PNode, PeerId};
use crate::Result;

use failure::format_err;
use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::{
    sync::mpsc,
    time::{interval, timeout},
};

/// Network manager for blockchain nodes
pub struct NetworkManager {
    /// P2P node communication channels
    command_tx: mpsc::UnboundedSender<NetworkCommand>,
    event_rx: mpsc::UnboundedReceiver<NetworkEvent>,
    /// Network configuration
    _config: NetworkConfig,
    /// Connected peers with their status
    peers: Arc<Mutex<HashMap<PeerId, PeerStatus>>>,
    /// Pending block requests
    pending_block_requests: Arc<Mutex<HashMap<String, Instant>>>,
    /// Pending transaction requests
    pending_tx_requests: Arc<Mutex<HashMap<String, Instant>>>,
    /// Blockchain reference for validation
    blockchain: Option<Arc<Mutex<Blockchain>>>,
    /// Event handlers
    block_handler: Option<Box<dyn Fn(Block) + Send + Sync>>,
    transaction_handler: Option<Box<dyn Fn(Transaction) + Send + Sync>>,
}

/// Status of a connected peer
#[derive(Debug, Clone)]
pub struct PeerStatus {
    /// Peer ID
    pub peer_id: PeerId,
    /// Best known blockchain height
    pub best_height: i32,
    /// Connection timestamp
    pub connected_at: Instant,
    /// Last activity timestamp
    pub last_activity: Instant,
    /// Number of successful interactions
    pub successful_interactions: u64,
    /// Number of failed interactions
    pub failed_interactions: u64,
}

impl NetworkManager {
    /// Create a new network manager
    pub async fn new(config: NetworkConfig) -> Result<Self> {
        // Create P2P node
        let listen_addr: std::net::SocketAddr = config.get_listen_address()
            .parse()
            .map_err(|e| format_err!("Invalid listen address: {}", e))?;
        
        let bootstrap_addrs: Vec<std::net::SocketAddr> = config.get_bootstrap_addresses()
            .iter()
            .filter_map(|addr| addr.parse().ok())
            .collect();

        let (mut p2p_node, event_rx, command_tx) = P2PNode::new(listen_addr, bootstrap_addrs)?;

        // Start P2P node in background
        tokio::spawn(async move {
            if let Err(e) = p2p_node.run().await {
                log::error!("P2P node error: {}", e);
            }
        });

        Ok(Self {
            command_tx,
            event_rx,
            _config: config,
            peers: Arc::new(Mutex::new(HashMap::new())),
            pending_block_requests: Arc::new(Mutex::new(HashMap::new())),
            pending_tx_requests: Arc::new(Mutex::new(HashMap::new())),
            blockchain: None,
            block_handler: None,
            transaction_handler: None,
        })
    }

    /// Set blockchain reference for validation
    pub fn set_blockchain(&mut self, blockchain: Arc<Mutex<Blockchain>>) {
        self.blockchain = Some(blockchain);
    }

    /// Set block handler
    pub fn set_block_handler<F>(&mut self, handler: F)
    where
        F: Fn(Block) + Send + Sync + 'static,
    {
        self.block_handler = Some(Box::new(handler));
    }

    /// Set transaction handler
    pub fn set_transaction_handler<F>(&mut self, handler: F)
    where
        F: Fn(Transaction) + Send + Sync + 'static,
    {
        self.transaction_handler = Some(Box::new(handler));
    }

    /// Start the network manager
    pub async fn start(&mut self) -> Result<()> {
        log::info!("Starting network manager");

        // Start periodic tasks
        let peers_cleanup = self.peers.clone();
        let pending_blocks_cleanup = self.pending_block_requests.clone();
        let pending_tx_cleanup = self.pending_tx_requests.clone();

        // Cleanup task for stale requests and inactive peers
        tokio::spawn(async move {
            let mut cleanup_interval = interval(Duration::from_secs(60));
            loop {
                cleanup_interval.tick().await;

                let now = Instant::now();

                // Cleanup stale block requests (older than 30 seconds)
                {
                    let mut pending_blocks = pending_blocks_cleanup.lock().unwrap();
                    pending_blocks.retain(|_, timestamp| now.duration_since(*timestamp) < Duration::from_secs(30));
                }

                // Cleanup stale transaction requests
                {
                    let mut pending_tx = pending_tx_cleanup.lock().unwrap();
                    pending_tx.retain(|_, timestamp| now.duration_since(*timestamp) < Duration::from_secs(30));
                }

                // Update peer activity
                {
                    let mut peers = peers_cleanup.lock().unwrap();
                    for peer_status in peers.values_mut() {
                        // Mark peers as inactive if no activity for 5 minutes
                        if now.duration_since(peer_status.last_activity) > Duration::from_secs(300) {
                            log::debug!("Peer {} is inactive", peer_status.peer_id);
                        }
                    }
                }
            }
        });

        // Main event loop
        loop {
            match timeout(Duration::from_millis(100), self.event_rx.recv()).await {
                Ok(Some(event)) => {
                    if let Err(e) = self.handle_network_event(event).await {
                        log::error!("Error handling network event: {}", e);
                    }
                }
                Ok(None) => {
                    log::warn!("Network event channel closed");
                    break;
                }
                Err(_) => {
                    // Timeout - continue loop
                }
            }
        }

        Ok(())
    }

    /// Handle network events
    async fn handle_network_event(&mut self, event: NetworkEvent) -> Result<()> {
        match event {
            NetworkEvent::PeerConnected(peer_id) => {
                log::info!("Peer connected: {}", peer_id);
                let mut peers = self.peers.lock().unwrap();
                peers.insert(
                    peer_id,
                    PeerStatus {
                        peer_id,
                        best_height: -1,
                        connected_at: Instant::now(),
                        last_activity: Instant::now(),
                        successful_interactions: 0,
                        failed_interactions: 0,
                    },
                );
            }
            NetworkEvent::PeerDisconnected(peer_id) => {
                log::info!("Peer disconnected: {}", peer_id);
                self.peers.lock().unwrap().remove(&peer_id);
            }
            NetworkEvent::BlockReceived(block) => {
                log::debug!("Received block: {}", block.get_hash());
                
                // Try to add block to blockchain if available
                if let Some(blockchain) = &self.blockchain {
                    let mut blockchain = blockchain.lock().unwrap();
                    if let Err(e) = blockchain.add_block(block.clone()) {
                        log::warn!("Failed to add block {}: {}", block.get_hash(), e);
                        return Ok(());
                    }
                }

                // Call block handler if set
                if let Some(handler) = &self.block_handler {
                    handler(block);
                }
            }
            NetworkEvent::TransactionReceived(transaction) => {
                log::debug!("Received transaction: {}", transaction.id);
                
                // Basic transaction validation would require previous transactions
                // For now, we accept all transactions - validation should be done
                // at the blockchain level when adding to mempool
                
                // Call transaction handler if set
                if let Some(handler) = &self.transaction_handler {
                    handler(transaction);
                }
            }
            NetworkEvent::BlockRequest(hash, peer_id) => {
                log::debug!("Block request from {}: {}", peer_id, hash);
                // This would be handled by the application layer
                // The application should call send_block_response
            }
            NetworkEvent::TransactionRequest(hash, peer_id) => {
                log::debug!("Transaction request from {}: {}", peer_id, hash);
                // This would be handled by the application layer
                // The application should call send_transaction_response
            }
            NetworkEvent::PeerInfo(peer_id, height) => {
                log::debug!("Peer {} has height {}", peer_id, height);
                if let Some(peer_status) = self.peers.lock().unwrap().get_mut(&peer_id) {
                    peer_status.best_height = height;
                    peer_status.last_activity = Instant::now();
                }
            }
        }

        Ok(())
    }

    /// Broadcast a block to all connected peers
    pub async fn broadcast_block(&self, block: Block) -> Result<()> {
        log::debug!("Broadcasting block: {}", block.get_hash());
        self.command_tx
            .send(NetworkCommand::BroadcastBlock(block))
            .map_err(|e| format_err!("Failed to send broadcast command: {}", e))?;
        Ok(())
    }

    /// Broadcast a transaction to all connected peers
    pub async fn broadcast_transaction(&self, transaction: Transaction) -> Result<()> {
        log::debug!("Broadcasting transaction: {}", transaction.id);
        self.command_tx
            .send(NetworkCommand::BroadcastTransaction(transaction))
            .map_err(|e| format_err!("Failed to send broadcast command: {}", e))?;
        Ok(())
    }

    /// Request a block from a specific peer
    pub async fn request_block(&self, hash: &str, peer_id: PeerId) -> Result<()> {
        log::debug!("Requesting block {} from {}", hash, peer_id);
        
        // Track the request
        self.pending_block_requests
            .lock()
            .unwrap()
            .insert(hash.to_string(), Instant::now());

        self.command_tx
            .send(NetworkCommand::RequestBlock(hash.to_string(), peer_id))
            .map_err(|e| format_err!("Failed to send request command: {}", e))?;
        Ok(())
    }

    /// Request a transaction from a specific peer
    pub async fn request_transaction(&self, hash: &str, peer_id: PeerId) -> Result<()> {
        log::debug!("Requesting transaction {} from {}", hash, peer_id);
        
        // Track the request
        self.pending_tx_requests
            .lock()
            .unwrap()
            .insert(hash.to_string(), Instant::now());

        self.command_tx
            .send(NetworkCommand::RequestTransaction(hash.to_string(), peer_id))
            .map_err(|e| format_err!("Failed to send request command: {}", e))?;
        Ok(())
    }

    /// Connect to a specific peer
    pub async fn connect_peer(&self, address: &str) -> Result<()> {
        log::info!("Connecting to peer: {}", address);
        let socket_addr: std::net::SocketAddr = address
            .parse()
            .map_err(|e| format_err!("Invalid peer address: {}", e))?;
        
        self.command_tx
            .send(NetworkCommand::ConnectPeer(socket_addr))
            .map_err(|e| format_err!("Failed to send connect command: {}", e))?;
        Ok(())
    }

    /// Get list of connected peers
    pub fn get_connected_peers(&self) -> Vec<PeerStatus> {
        self.peers.lock().unwrap().values().cloned().collect()
    }

    /// Get the best peer for requesting blocks (highest blockchain height)
    pub fn get_best_peer(&self) -> Option<PeerId> {
        self.peers
            .lock()
            .unwrap()
            .values()
            .max_by_key(|peer| peer.best_height)
            .map(|peer| peer.peer_id)
    }

    /// Sync blockchain with peers
    pub async fn sync_blockchain(&self) -> Result<()> {
        let best_peer = self.get_best_peer();
        
        if let Some(peer_id) = best_peer {
            let peer_height = self.peers
                .lock()
                .unwrap()
                .get(&peer_id)
                .map(|p| p.best_height)
                .unwrap_or(-1);

            let current_height = if let Some(blockchain) = &self.blockchain {
                blockchain.lock().unwrap().get_best_height().unwrap_or(-1)
            } else {
                -1
            };

            if peer_height > current_height {
                log::info!(
                    "Starting blockchain sync: local height {}, peer height {}",
                    current_height,
                    peer_height
                );
                
                // Request blocks starting from our current height
                // This is a simplified version - in practice, you'd implement
                // a more sophisticated block download strategy
                for height in (current_height + 1)..=peer_height {
                    // Request block at specific height
                    // This would require extending the protocol
                    log::debug!("Would request block at height {}", height);
                }
            }
        }

        Ok(())
    }

    /// Get network statistics
    pub fn get_network_stats(&self) -> NetworkStats {
        let peers = self.peers.lock().unwrap();
        let pending_blocks = self.pending_block_requests.lock().unwrap().len();
        let pending_transactions = self.pending_tx_requests.lock().unwrap().len();

        let total_successful_interactions: u64 = peers.values().map(|p| p.successful_interactions).sum();
        let total_failed_interactions: u64 = peers.values().map(|p| p.failed_interactions).sum();

        NetworkStats {
            connected_peers: peers.len(),
            pending_block_requests: pending_blocks,
            pending_transaction_requests: pending_transactions,
            total_successful_interactions,
            total_failed_interactions,
            best_height: peers.values().map(|p| p.best_height).max().unwrap_or(-1),
        }
    }
}

/// Network statistics
#[derive(Debug, Clone)]
pub struct NetworkStats {
    /// Number of connected peers
    pub connected_peers: usize,
    /// Number of pending block requests
    pub pending_block_requests: usize,
    /// Number of pending transaction requests
    pub pending_transaction_requests: usize,
    /// Total successful interactions
    pub total_successful_interactions: u64,
    /// Total failed interactions
    pub total_failed_interactions: u64,
    /// Best known blockchain height
    pub best_height: i32,
}

impl std::fmt::Display for NetworkStats {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "Network Stats: {} peers, {} pending blocks, {} pending txs, best height: {}",
            self.connected_peers,
            self.pending_block_requests,
            self.pending_transaction_requests,
            self.best_height
        )
    }
}
