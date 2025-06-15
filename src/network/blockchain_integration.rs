//! Blockchain Network Integration
//!
//! This module integrates the blockchain with the P2P network layer,
//! handling block propagation, transaction broadcasting, and network consensus.

use crate::blockchain::block::FinalizedBlock;
use crate::crypto::transaction::Transaction;
use crate::network::p2p_enhanced::{EnhancedP2PNode, NetworkCommand, NetworkEvent, PeerId};
use crate::Result;

use failure::format_err;
use std::collections::{HashMap, VecDeque};
use std::sync::{Arc, Mutex};
use std::time::{Duration, SystemTime, UNIX_EPOCH};
use tokio::sync::{mpsc, RwLock};
use tokio::time::interval;

/// Network-integrated blockchain node
pub struct NetworkedBlockchainNode {
    /// P2P network node
    p2p_node: Arc<RwLock<EnhancedP2PNode>>,
    /// Network event receiver
    network_events: Arc<Mutex<mpsc::UnboundedReceiver<NetworkEvent>>>,
    /// Network command sender
    network_commands: mpsc::UnboundedSender<NetworkCommand>,
    /// Blockchain state
    blockchain_state: Arc<RwLock<BlockchainState>>,
    /// Transaction pool (mempool)
    mempool: Arc<RwLock<TransactionPool>>,
    /// Block cache for synchronization
    block_cache: Arc<RwLock<BlockCache>>,
    /// Synchronization state
    sync_state: Arc<RwLock<SyncState>>,
    /// Event handlers
    event_handlers: Arc<RwLock<Vec<EventHandler>>>,
}

/// Blockchain state
#[derive(Debug, Clone)]
pub struct BlockchainState {
    pub current_height: i32,
    pub best_block_hash: Option<String>,
    pub pending_blocks: VecDeque<FinalizedBlock>,
    pub is_syncing: bool,
    pub last_update: u64,
}

/// Transaction pool (mempool)
#[derive(Debug)]
pub struct TransactionPool {
    pub transactions: HashMap<String, Transaction>,
    pub pending_count: usize,
    pub max_size: usize,
    pub last_cleanup: u64,
}

/// Block cache for synchronization
#[derive(Debug)]
pub struct BlockCache {
    pub blocks: HashMap<String, FinalizedBlock>,
    pub requested_blocks: HashMap<String, (PeerId, u64)>, // block_hash -> (requester, timestamp)
    pub max_size: usize,
}

/// Synchronization state
#[derive(Debug, Clone)]
pub struct SyncState {
    pub is_syncing: bool,
    pub target_height: Option<i32>,
    pub sync_peer: Option<PeerId>,
    pub last_sync_request: u64,
    pub blocks_behind: i32,
}

/// Event handler type
pub type EventHandler = Box<dyn Fn(&NetworkEvent) -> Result<()> + Send + Sync>;

/// Network synchronization events
#[derive(Debug, Clone)]
pub enum SyncEvent {
    SyncStarted {
        target_height: i32,
        peer: PeerId,
    },
    SyncProgress {
        current_height: i32,
        target_height: i32,
    },
    SyncCompleted {
        final_height: i32,
    },
    SyncFailed {
        error: String,
    },
}

impl NetworkedBlockchainNode {
    /// Create a new networked blockchain node
    pub async fn new(
        listen_addr: std::net::SocketAddr,
        bootstrap_peers: Vec<std::net::SocketAddr>,
    ) -> Result<Self> {
        let (p2p_node, network_events, network_commands) =
            EnhancedP2PNode::new(listen_addr, bootstrap_peers)?;

        let blockchain_state = BlockchainState {
            current_height: 0,
            best_block_hash: None,
            pending_blocks: VecDeque::new(),
            is_syncing: false,
            last_update: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        let mempool = TransactionPool {
            transactions: HashMap::new(),
            pending_count: 0,
            max_size: 10000,
            last_cleanup: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        let block_cache = BlockCache {
            blocks: HashMap::new(),
            requested_blocks: HashMap::new(),
            max_size: 1000,
        };

        let sync_state = SyncState {
            is_syncing: false,
            target_height: None,
            sync_peer: None,
            last_sync_request: 0,
            blocks_behind: 0,
        };

        Ok(NetworkedBlockchainNode {
            p2p_node: Arc::new(RwLock::new(p2p_node)),
            network_events: Arc::new(Mutex::new(network_events)),
            network_commands,
            blockchain_state: Arc::new(RwLock::new(blockchain_state)),
            mempool: Arc::new(RwLock::new(mempool)),
            block_cache: Arc::new(RwLock::new(block_cache)),
            sync_state: Arc::new(RwLock::new(sync_state)),
            event_handlers: Arc::new(RwLock::new(Vec::new())),
        })
    }

    /// Start the networked blockchain node
    pub async fn start(&mut self) -> Result<()> {
        log::info!("Starting networked blockchain node...");

        // Start event processing
        self.start_event_processing().await;

        // Start background tasks
        self.start_background_tasks().await;

        log::info!("Networked blockchain node started successfully");
        Ok(())
    }

    /// Start event processing
    async fn start_event_processing(&self) {
        let network_events = self.network_events.clone();
        let blockchain_state = self.blockchain_state.clone();
        let mempool = self.mempool.clone();
        let block_cache = self.block_cache.clone();
        let sync_state = self.sync_state.clone();
        let network_commands = self.network_commands.clone();
        let event_handlers = self.event_handlers.clone();

        tokio::spawn(async move {
            loop {
                let event_opt = {
                    let mut events = network_events.lock().unwrap();
                    events.try_recv().ok()
                };

                if let Some(event) = event_opt {
                    // Call registered event handlers
                    {
                        let handlers = event_handlers.read().await;
                        for handler in handlers.iter() {
                            if let Err(e) = handler(&event) {
                                log::error!("Event handler error: {}", e);
                            }
                        }
                    }

                    // Process the event
                    if let Err(e) = Self::process_network_event(
                        event,
                        blockchain_state.clone(),
                        mempool.clone(),
                        block_cache.clone(),
                        sync_state.clone(),
                        network_commands.clone(),
                    )
                    .await
                    {
                        log::error!("Error processing network event: {}", e);
                    }
                } else {
                    // Sleep briefly if no events
                    tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;
                }
            }
        });
    }

    /// Process network events
    async fn process_network_event(
        event: NetworkEvent,
        blockchain_state: Arc<RwLock<BlockchainState>>,
        mempool: Arc<RwLock<TransactionPool>>,
        block_cache: Arc<RwLock<BlockCache>>,
        sync_state: Arc<RwLock<SyncState>>,
        network_commands: mpsc::UnboundedSender<NetworkCommand>,
    ) -> Result<()> {
        match event {
            NetworkEvent::PeerConnected(peer_id) => {
                log::info!("New peer connected: {}", peer_id);

                // Send our status to the new peer
                let current_height = blockchain_state.read().await.current_height;
                let _ = network_commands.send(NetworkCommand::UpdateHeight(current_height));
            }

            NetworkEvent::PeerDisconnected(peer_id) => {
                log::info!("Peer disconnected: {}", peer_id);

                // If this was our sync peer, find a new one
                let mut sync = sync_state.write().await;
                if sync.sync_peer == Some(peer_id) {
                    sync.sync_peer = None;
                    sync.is_syncing = false;
                }
            }

            NetworkEvent::BlockReceived(block, peer_id) => {
                log::debug!(
                    "Received block from {}: height {}",
                    peer_id,
                    block.get_height()
                );

                // Process the received block
                Self::process_received_block(
                    *block,
                    peer_id,
                    blockchain_state.clone(),
                    block_cache.clone(),
                    sync_state.clone(),
                    network_commands.clone(),
                )
                .await?;
            }

            NetworkEvent::TransactionReceived(transaction, peer_id) => {
                log::debug!("Received transaction from {}", peer_id);

                // Add to mempool if valid
                Self::process_received_transaction(*transaction, mempool.clone()).await?;
            }

            NetworkEvent::BlockRequest(block_hash, peer_id) => {
                log::debug!("Block request from {}: {}", peer_id, block_hash);

                // Look for the block in cache and send it
                let cache = block_cache.read().await;
                if let Some(block) = cache.blocks.get(&block_hash) {
                    let _ = network_commands
                        .send(NetworkCommand::BroadcastBlock(Box::new(block.clone())));
                }
            }

            NetworkEvent::TransactionRequest(tx_hash, peer_id) => {
                log::debug!("Transaction request from {}: {}", peer_id, tx_hash);

                // Look for the transaction in mempool and send it
                let pool = mempool.read().await;
                if let Some(tx) = pool.transactions.get(&tx_hash) {
                    let _ = network_commands.send(NetworkCommand::BroadcastTransaction(tx.clone()));
                }
            }

            NetworkEvent::PeerInfo(peer_id, height) => {
                log::debug!("Peer {} info: height {}", peer_id, height);

                // Check if we need to sync
                let current_height = blockchain_state.read().await.current_height;
                if height > current_height + 1 {
                    log::info!("Peer {} is ahead ({}), starting sync", peer_id, height);
                    Self::start_sync(
                        peer_id,
                        height,
                        sync_state.clone(),
                        network_commands.clone(),
                    )
                    .await?;
                }
            }

            NetworkEvent::PeerDiscovery(peers) => {
                log::debug!("Discovered {} peers", peers.len());

                // Connect to new peers if we don't have enough connections
                for peer_info in peers.iter().take(3) {
                    // Limit new connections
                    let _ = network_commands.send(NetworkCommand::ConnectPeer(peer_info.address));
                }
            }

            // Handle new network management events
            NetworkEvent::NetworkHealthUpdate(topology) => {
                log::info!("Network health update: {} total nodes, {} healthy peers", 
                          topology.total_nodes, topology.healthy_peers);
            }

            NetworkEvent::PeerHealthChanged(peer_id, health) => {
                log::debug!("Peer {} health changed to {:?}", peer_id, health);
            }

            NetworkEvent::MessageQueueStats(stats) => {
                log::debug!("Message queue stats: {} total messages in queues", 
                          stats.critical_queue_size + stats.high_queue_size + 
                          stats.normal_queue_size + stats.low_queue_size);
            }
        }

        Ok(())
    }

    /// Process received block
    async fn process_received_block(
        block: FinalizedBlock,
        _peer_id: PeerId,
        blockchain_state: Arc<RwLock<BlockchainState>>,
        block_cache: Arc<RwLock<BlockCache>>,
        sync_state: Arc<RwLock<SyncState>>,
        _network_commands: mpsc::UnboundedSender<NetworkCommand>,
    ) -> Result<()> {
        let block_height = block.get_height();
        let block_hash = format!("{:?}", block.get_hash());

        // Add to cache
        {
            let mut cache = block_cache.write().await;
            cache.blocks.insert(block_hash.clone(), block.clone());

            // Clean up cache if too large
            if cache.blocks.len() > cache.max_size {
                // Remove oldest blocks (simplified - in practice you'd use LRU)
                let keys_to_remove: Vec<String> = cache.blocks.keys().take(100).cloned().collect();
                for key in keys_to_remove {
                    cache.blocks.remove(&key);
                }
            }
        }

        // Update blockchain state
        {
            let mut state = blockchain_state.write().await;

            // Check if this block extends our chain
            if block_height == state.current_height + 1 {
                state.current_height = block_height;
                state.best_block_hash = Some(block_hash.clone());
                state.last_update = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();

                log::info!("Extended blockchain to height {}", block_height);
            } else if block_height > state.current_height {
                // Add to pending blocks for potential reorganization
                state.pending_blocks.push_back(block);
                log::debug!(
                    "Added block {} to pending (current height: {})",
                    block_height,
                    state.current_height
                );
            }
        }

        // Update sync progress
        {
            let mut sync = sync_state.write().await;
            if sync.is_syncing {
                if let Some(target) = sync.target_height {
                    if block_height >= target {
                        sync.is_syncing = false;
                        sync.target_height = None;
                        sync.sync_peer = None;
                        log::info!("Synchronization completed at height {}", block_height);
                    }
                }
            }
        }

        Ok(())
    }

    /// Process received transaction
    async fn process_received_transaction(
        transaction: Transaction,
        mempool: Arc<RwLock<TransactionPool>>,
    ) -> Result<()> {
        let tx_hash = format!("{:?}", transaction.hash());

        let mut pool = mempool.write().await;

        // Check if we already have this transaction
        if pool.transactions.contains_key(&tx_hash) {
            return Ok(());
        }

        // Check mempool size limit
        if pool.transactions.len() >= pool.max_size {
            log::warn!("Mempool full, dropping transaction {}", tx_hash);
            return Ok(());
        }

        // Add transaction to mempool (simplified validation)
        pool.transactions.insert(tx_hash.clone(), transaction);
        pool.pending_count += 1;

        log::debug!(
            "Added transaction {} to mempool (total: {})",
            tx_hash,
            pool.transactions.len()
        );
        Ok(())
    }

    /// Start synchronization with a peer
    async fn start_sync(
        peer_id: PeerId,
        target_height: i32,
        sync_state: Arc<RwLock<SyncState>>,
        network_commands: mpsc::UnboundedSender<NetworkCommand>,
    ) -> Result<()> {
        let mut sync = sync_state.write().await;

        if sync.is_syncing {
            return Ok(()); // Already syncing
        }

        sync.is_syncing = true;
        sync.target_height = Some(target_height);
        sync.sync_peer = Some(peer_id);
        sync.last_sync_request = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Request blocks starting from our current height + 1
        // In practice, you'd implement a more sophisticated sync protocol
        let _ = network_commands.send(NetworkCommand::RequestBlock(
            "next_block_hash".to_string(), // Placeholder
            peer_id,
        ));

        log::info!(
            "Started synchronization with {} (target height: {})",
            peer_id,
            target_height
        );
        Ok(())
    }

    /// Start background tasks
    async fn start_background_tasks(&self) {
        let mempool = self.mempool.clone();
        let blockchain_state = self.blockchain_state.clone();
        let sync_state = self.sync_state.clone();
        let network_commands = self.network_commands.clone();

        // Mempool cleanup task
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(60));
            loop {
                interval.tick().await;
                Self::cleanup_mempool(mempool.clone()).await;
            }
        });

        // Sync monitoring task
        let sync_state_monitor = sync_state.clone();
        let network_commands_monitor = network_commands.clone();
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(30));
            loop {
                interval.tick().await;
                Self::monitor_sync_progress(
                    sync_state_monitor.clone(),
                    network_commands_monitor.clone(),
                )
                .await;
            }
        });

        // Status broadcasting task
        let blockchain_state_broadcast = blockchain_state.clone();
        let network_commands_broadcast = network_commands.clone();
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(10));
            loop {
                interval.tick().await;
                let height = blockchain_state_broadcast.read().await.current_height;
                let _ = network_commands_broadcast.send(NetworkCommand::UpdateHeight(height));
            }
        });
    }

    /// Cleanup mempool
    async fn cleanup_mempool(mempool: Arc<RwLock<TransactionPool>>) {
        let mut pool = mempool.write().await;
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Remove old transactions (simplified - in practice you'd check transaction age)
        if pool.transactions.len() > pool.max_size / 2 {
            let keys_to_remove: Vec<String> = pool.transactions.keys().take(100).cloned().collect();
            for key in keys_to_remove {
                pool.transactions.remove(&key);
            }
            pool.pending_count = pool.transactions.len();
            log::debug!(
                "Cleaned up mempool, {} transactions remaining",
                pool.transactions.len()
            );
        }

        pool.last_cleanup = now;
    }

    /// Monitor sync progress
    async fn monitor_sync_progress(
        sync_state: Arc<RwLock<SyncState>>,
        _network_commands: mpsc::UnboundedSender<NetworkCommand>,
    ) {
        let sync = sync_state.read().await;
        if sync.is_syncing {
            let now = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
            if now - sync.last_sync_request > 60 {
                // 1 minute timeout
                log::warn!("Sync timeout, may need to restart synchronization");
            }
        }
    }

    /// Public API methods
    /// Broadcast a block to the network
    pub async fn broadcast_block(&self, block: FinalizedBlock) -> Result<()> {
        // Update our state first
        {
            let mut state = self.blockchain_state.write().await;
            state.current_height = block.get_height();
            state.best_block_hash = Some(format!("{:?}", block.get_hash()));
            state.last_update = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs();
        }

        // Broadcast to network
        self.network_commands
            .send(NetworkCommand::BroadcastBlock(Box::new(block)))
            .map_err(|e| format_err!("Failed to broadcast block: {}", e))?;

        Ok(())
    }

    /// Broadcast a transaction to the network
    pub async fn broadcast_transaction(&self, transaction: Transaction) -> Result<()> {
        // Add to our mempool first
        {
            let tx_hash = format!("{:?}", transaction.hash());
            let mut pool = self.mempool.write().await;

            if !pool.transactions.contains_key(&tx_hash) && pool.transactions.len() < pool.max_size
            {
                pool.transactions.insert(tx_hash, transaction.clone());
                pool.pending_count += 1;
            }
        }

        // Broadcast to network
        self.network_commands
            .send(NetworkCommand::BroadcastTransaction(transaction))
            .map_err(|e| format_err!("Failed to broadcast transaction: {}", e))?;

        Ok(())
    }

    /// Get current blockchain state
    pub async fn get_blockchain_state(&self) -> BlockchainState {
        self.blockchain_state.read().await.clone()
    }

    /// Get mempool transactions
    pub async fn get_mempool_transactions(&self) -> Vec<Transaction> {
        let pool = self.mempool.read().await;
        pool.transactions.values().cloned().collect()
    }

    /// Get sync state
    pub async fn get_sync_state(&self) -> SyncState {
        self.sync_state.read().await.clone()
    }

    /// Connect to a peer
    pub async fn connect_to_peer(&self, addr: std::net::SocketAddr) -> Result<()> {
        self.network_commands
            .send(NetworkCommand::ConnectPeer(addr))
            .map_err(|e| format_err!("Failed to connect to peer: {}", e))?;
        Ok(())
    }

    /// Get connected peers
    pub async fn get_connected_peers(&self) -> Vec<PeerId> {
        let p2p = self.p2p_node.read().await;
        p2p.get_connected_peers()
    }

    /// Add an event handler
    pub async fn add_event_handler<F>(&self, handler: F)
    where
        F: Fn(&NetworkEvent) -> Result<()> + Send + Sync + 'static,
    {
        let mut handlers = self.event_handlers.write().await;
        handlers.push(Box::new(handler));
    }

    /// Get network statistics
    pub async fn get_network_stats(&self) -> Result<String> {
        let p2p = self.p2p_node.read().await;
        let stats = p2p.get_stats();

        Ok(format!(
            "Connected Peers: {}\nMessages Sent: {}\nMessages Received: {}\nBlocks Propagated: {}\nTransactions Propagated: {}",
            p2p.get_connected_peers().len(),
            stats.messages_sent,
            stats.messages_received,
            stats.blocks_propagated,
            stats.transactions_propagated
        ))
    }
}
