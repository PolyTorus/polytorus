//! State Synchronization for Modular Blockchain
//!
//! This module implements comprehensive state synchronization between nodes,
//! including block synchronization, state verification, and fork resolution.

use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, RwLock},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::{sync::mpsc, time::interval};

use crate::{
    blockchain::block::FinalizedBlock,
    crypto::transaction::Transaction,
    modular::{
        peer_discovery::NodeId,
        storage::{ModularStorage, StorageLayer},
    },
};

/// Synchronization state of the node
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum SyncState {
    Synced,
    Syncing {
        current_height: u64,
        target_height: u64,
        peer_id: NodeId,
    },
    Behind {
        current_height: u64,
        best_known_height: u64,
    },
    Ahead {
        current_height: u64,
        peer_height: u64,
    },
    Forked {
        fork_height: u64,
        our_hash: String,
        their_hash: String,
    },
}

/// Synchronization request types
#[derive(Debug, Clone, Serialize, Deserialize)]
#[allow(clippy::enum_variant_names)]
pub enum SyncRequest {
    GetBlockHeaders {
        start_height: u64,
        count: u32,
        skip: u32,
        reverse: bool,
    },
    GetBlocks {
        hashes: Vec<String>,
    },
    GetBlockBodies {
        hashes: Vec<String>,
    },
    GetState {
        block_hash: String,
        keys: Vec<String>,
    },
    GetChainInfo,
}

/// Synchronization response types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SyncResponse {
    BlockHeaders {
        headers: Vec<BlockHeader>,
    },
    Blocks {
        blocks: Vec<FinalizedBlock>,
    },
    BlockBodies {
        bodies: Vec<BlockBody>,
    },
    State {
        entries: Vec<StateEntry>,
    },
    ChainInfo {
        best_height: u64,
        best_hash: String,
        total_difficulty: u64,
        genesis_hash: String,
    },
    Error {
        message: String,
    },
}

/// Lightweight block header for synchronization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockHeader {
    pub height: u64,
    pub hash: String,
    pub previous_hash: String,
    pub timestamp: u64,
    pub difficulty: u32,
    pub nonce: u64,
    pub transaction_count: usize,
    pub state_root: String,
}

/// Block body for synchronization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockBody {
    pub hash: String,
    pub transactions: Vec<Transaction>,
}

/// State entry for synchronization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateEntry {
    pub key: String,
    pub value: Vec<u8>,
    pub proof: Option<Vec<u8>>,
}

/// Synchronization events
#[derive(Debug, Clone)]
pub enum SyncEvent {
    SyncStarted {
        target_height: u64,
        peer_id: NodeId,
    },
    SyncProgress {
        current_height: u64,
        target_height: u64,
        percentage: f64,
    },
    SyncCompleted {
        final_height: u64,
        blocks_synced: u64,
    },
    SyncFailed {
        error: String,
        peer_id: NodeId,
    },
    ForkDetected {
        fork_height: u64,
        our_hash: String,
        their_hash: String,
    },
    ForkResolved {
        winning_branch: String,
        discarded_blocks: u64,
    },
    StateVerified {
        block_height: u64,
        verified: bool,
    },
}

/// Synchronization configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SyncConfig {
    pub max_blocks_per_request: u32,
    pub max_headers_per_request: u32,
    pub sync_timeout: Duration,
    pub verification_batch_size: usize,
    pub max_fork_depth: u64,
    pub state_sync_enabled: bool,
    pub fast_sync_enabled: bool,
    pub checkpoint_interval: u64,
}

impl Default for SyncConfig {
    fn default() -> Self {
        Self {
            max_blocks_per_request: 128,
            max_headers_per_request: 512,
            sync_timeout: Duration::from_secs(30),
            verification_batch_size: 10,
            max_fork_depth: 100,
            state_sync_enabled: true,
            fast_sync_enabled: true,
            checkpoint_interval: 1000,
        }
    }
}

/// Peer synchronization information
#[derive(Debug, Clone)]
pub struct PeerSyncInfo {
    node_id: NodeId,
    best_height: u64,
    best_hash: String,
    total_difficulty: u64,
    last_update: u64,
    is_syncing: bool,
    sync_quality: f64,
}

impl PeerSyncInfo {
    /// Get the best hash
    pub fn best_hash(&self) -> &str {
        &self.best_hash
    }

    /// Get the last update timestamp
    pub fn last_update(&self) -> u64 {
        self.last_update
    }
}

/// State synchronization manager
pub struct StateSynchronizer {
    config: SyncConfig,
    storage: Arc<ModularStorage>,
    current_state: Arc<RwLock<SyncState>>,
    peer_info: Arc<RwLock<HashMap<NodeId, PeerSyncInfo>>>,
    sync_queue: Arc<RwLock<VecDeque<SyncRequest>>>,
    event_tx: mpsc::UnboundedSender<SyncEvent>,
    pending_blocks: Arc<RwLock<HashMap<String, FinalizedBlock>>>,
    verified_checkpoints: Arc<RwLock<HashMap<u64, String>>>,
}

impl StateSynchronizer {
    /// Create a new state synchronizer
    pub fn new(
        config: SyncConfig,
        storage: Arc<ModularStorage>,
    ) -> (Self, mpsc::UnboundedReceiver<SyncEvent>) {
        let (event_tx, event_rx) = mpsc::unbounded_channel();

        let synchronizer = Self {
            config,
            storage,
            current_state: Arc::new(RwLock::new(SyncState::Synced)),
            peer_info: Arc::new(RwLock::new(HashMap::new())),
            sync_queue: Arc::new(RwLock::new(VecDeque::new())),
            event_tx,
            pending_blocks: Arc::new(RwLock::new(HashMap::new())),
            verified_checkpoints: Arc::new(RwLock::new(HashMap::new())),
        };

        (synchronizer, event_rx)
    }

    /// Start the synchronization process
    pub async fn start(&self) -> Result<()> {
        self.start_sync_loop().await;
        self.start_verification_loop().await;
        Ok(())
    }

    /// Update peer information
    pub async fn update_peer_info(
        &self,
        node_id: NodeId,
        best_height: u64,
        best_hash: String,
        total_difficulty: u64,
    ) -> Result<()> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let peer_info = PeerSyncInfo {
            node_id,
            best_height,
            best_hash,
            total_difficulty,
            last_update: now,
            is_syncing: false,
            sync_quality: 1.0,
        };

        {
            let mut peers = self.peer_info.write().unwrap();
            peers.insert(node_id, peer_info);
        }

        // Check if we need to sync
        self.evaluate_sync_status().await?;
        Ok(())
    }

    /// Evaluate whether we need to synchronize
    async fn evaluate_sync_status(&self) -> Result<()> {
        let our_height = self.get_current_height().await?;
        let best_peer = self.find_best_peer().await;

        if let Some(peer) = best_peer {
            let height_difference = peer.best_height.saturating_sub(our_height);

            match height_difference {
                0 => {
                    *self.current_state.write().unwrap() = SyncState::Synced;
                }
                1..=5 => {
                    *self.current_state.write().unwrap() = SyncState::Behind {
                        current_height: our_height,
                        best_known_height: peer.best_height,
                    };
                }
                _ => {
                    // Start synchronization
                    self.start_sync_with_peer(peer).await?;
                }
            }
        }

        Ok(())
    }

    /// Find the best peer to sync from
    async fn find_best_peer(&self) -> Option<PeerSyncInfo> {
        let peers = self.peer_info.read().unwrap();

        peers
            .values()
            .filter(|peer| !peer.is_syncing)
            .max_by_key(|peer| {
                // Score based on height, difficulty, and quality
                (
                    peer.best_height,
                    peer.total_difficulty,
                    (peer.sync_quality * 1000.0) as u64,
                )
            })
            .cloned()
    }

    /// Start synchronization with a specific peer
    async fn start_sync_with_peer(&self, peer: PeerSyncInfo) -> Result<()> {
        let our_height = self.get_current_height().await?;

        *self.current_state.write().unwrap() = SyncState::Syncing {
            current_height: our_height,
            target_height: peer.best_height,
            peer_id: peer.node_id,
        };

        // Mark peer as syncing
        {
            let mut peers = self.peer_info.write().unwrap();
            if let Some(peer_info) = peers.get_mut(&peer.node_id) {
                peer_info.is_syncing = true;
            }
        }

        // Emit sync started event
        let _ = self.event_tx.send(SyncEvent::SyncStarted {
            target_height: peer.best_height,
            peer_id: peer.node_id,
        });

        // Add sync requests to queue
        self.queue_sync_requests(our_height, peer.best_height)
            .await?;

        log::info!(
            "Started sync with peer {} from height {} to {}",
            peer.node_id,
            our_height,
            peer.best_height
        );

        Ok(())
    }

    /// Queue synchronization requests
    async fn queue_sync_requests(&self, start_height: u64, target_height: u64) -> Result<()> {
        let mut current_height = start_height + 1;
        let max_blocks = self.config.max_blocks_per_request as u64;

        while current_height <= target_height {
            let count = std::cmp::min(max_blocks, target_height - current_height + 1) as u32;

            let request = SyncRequest::GetBlockHeaders {
                start_height: current_height,
                count,
                skip: 0,
                reverse: false,
            };

            {
                let mut queue = self.sync_queue.write().unwrap();
                queue.push_back(request);
            }

            current_height += count as u64;
        }

        Ok(())
    }

    /// Process a synchronization response
    pub async fn process_sync_response(
        &self,
        response: SyncResponse,
        from_peer: NodeId,
    ) -> Result<()> {
        match response {
            SyncResponse::BlockHeaders { headers } => {
                self.process_block_headers(headers, from_peer).await?;
            }
            SyncResponse::Blocks { blocks } => {
                self.process_blocks(blocks, from_peer).await?;
            }
            SyncResponse::BlockBodies { bodies } => {
                self.process_block_bodies(bodies, from_peer).await?;
            }
            SyncResponse::State { entries } => {
                self.process_state_entries(entries, from_peer).await?;
            }
            SyncResponse::ChainInfo {
                best_height,
                best_hash,
                total_difficulty,
                ..
            } => {
                let _ = self
                    .update_peer_info(from_peer, best_height, best_hash, total_difficulty)
                    .await;
            }
            SyncResponse::Error { message } => {
                log::error!("Sync error from peer {}: {}", from_peer, message);
                self.handle_sync_error(from_peer, message).await?;
            }
        }

        Ok(())
    }

    /// Process block headers
    async fn process_block_headers(
        &self,
        headers: Vec<BlockHeader>,
        from_peer: NodeId,
    ) -> Result<()> {
        log::debug!(
            "Processing {} block headers from peer {}",
            headers.len(),
            from_peer
        );

        for header in headers {
            // Validate header chain continuity
            if !self.validate_header_chain(&header).await? {
                log::warn!("Invalid header chain from peer {}", from_peer);
                continue;
            }

            // Request block bodies for these headers
            let request = SyncRequest::GetBlocks {
                hashes: vec![header.hash.clone()],
            };

            {
                let mut queue = self.sync_queue.write().unwrap();
                queue.push_back(request);
            }
        }

        Ok(())
    }

    /// Process full blocks
    async fn process_blocks(&self, blocks: Vec<FinalizedBlock>, from_peer: NodeId) -> Result<()> {
        log::debug!("Processing {} blocks from peer {}", blocks.len(), from_peer);

        for block in blocks {
            // Validate block
            if !self.validate_block(&block).await? {
                log::warn!("Invalid block {} from peer {}", block.get_hash(), from_peer);
                continue;
            }

            // Store block temporarily
            {
                let mut pending = self.pending_blocks.write().unwrap();
                pending.insert(block.get_hash().to_string(), block.clone());
            }

            // Try to add to chain
            self.try_add_block_to_chain(block).await?;
        }

        // Update sync progress
        self.update_sync_progress().await?;

        Ok(())
    }

    /// Process block bodies
    async fn process_block_bodies(&self, bodies: Vec<BlockBody>, _from_peer: NodeId) -> Result<()> {
        for body in bodies {
            // Store block body for later assembly
            log::debug!("Received block body for hash: {}", body.hash);
            // Implementation would store bodies and match with headers
        }
        Ok(())
    }

    /// Process state entries
    async fn process_state_entries(
        &self,
        entries: Vec<StateEntry>,
        _from_peer: NodeId,
    ) -> Result<()> {
        for entry in entries {
            // Verify state proof if provided
            if let Some(_proof) = &entry.proof {
                // Implement Merkle proof verification
                log::debug!("Verifying state proof for key: {}", entry.key);
            }

            // Store state entry
            log::debug!("Storing state entry: {}", entry.key);
        }
        Ok(())
    }

    /// Validate header chain continuity
    async fn validate_header_chain(&self, header: &BlockHeader) -> Result<bool> {
        if header.height == 0 {
            return Ok(true); // Genesis block
        }

        // Check if we have the previous block
        let previous_block = self
            .storage
            .get_block_by_hash(&header.previous_hash)
            .await?;
        if let Some(prev_block) = previous_block {
            // Validate height sequence
            if header.height != (prev_block.get_height() + 1) as u64 {
                return Ok(false);
            }

            // Validate timestamp
            if header.timestamp <= prev_block.get_timestamp() as u64 {
                return Ok(false);
            }
        }

        Ok(true)
    }

    /// Validate a full block
    async fn validate_block(&self, block: &FinalizedBlock) -> Result<bool> {
        // Basic validation
        if block.get_transactions().is_empty() {
            return Ok(false);
        }

        // Validate block structure
        if block.get_hash().is_empty() {
            return Ok(false);
        }

        // Additional validation logic would go here
        Ok(true)
    }

    /// Try to add block to the main chain
    async fn try_add_block_to_chain(&self, block: FinalizedBlock) -> Result<()> {
        let current_height = self.get_current_height().await?;
        let block_height = block.get_height() as u64;

        if block_height == (current_height + 1) as u64 {
            // Next block in sequence - add directly
            self.add_block_to_chain(block).await?;
        } else if block_height > (current_height + 1) as u64 {
            // Future block - keep in pending
            log::debug!(
                "Keeping future block {} at height {}",
                block.get_hash(),
                block_height
            );
        } else {
            // Past block - might be a fork
            self.handle_potential_fork(block).await?;
        }

        Ok(())
    }

    /// Add block to the main chain
    async fn add_block_to_chain(&self, block: FinalizedBlock) -> Result<()> {
        // Store the block
        self.storage.store_block(&block)?;

        // Update chain state
        self.storage
            .update_best_block(block.get_hash(), block.get_height() as u64)
            .await?;

        log::info!(
            "Added block {} at height {}",
            block.get_hash(),
            block.get_height()
        );
        Ok(())
    }

    /// Handle potential fork
    async fn handle_potential_fork(&self, block: FinalizedBlock) -> Result<()> {
        let our_hash = self
            .get_block_hash_at_height(block.get_height() as u64)
            .await?;

        if let Some(hash) = our_hash {
            if hash != block.get_hash() {
                // Fork detected
                let _ = self.event_tx.send(SyncEvent::ForkDetected {
                    fork_height: block.get_height() as u64,
                    our_hash: hash,
                    their_hash: block.get_hash().to_string(),
                });

                // Implement fork resolution logic
                self.resolve_fork(block).await?;
            }
        }

        Ok(())
    }

    /// Resolve a fork by comparing chain difficulty
    async fn resolve_fork(&self, their_block: FinalizedBlock) -> Result<()> {
        // Simplified fork resolution - in practice, would compare total difficulty
        log::info!("Resolving fork at height {}", their_block.get_height());

        // For now, keep our chain (implement proper resolution logic)
        let _ = self.event_tx.send(SyncEvent::ForkResolved {
            winning_branch: "ours".to_string(),
            discarded_blocks: 0,
        });

        Ok(())
    }

    /// Update synchronization progress
    async fn update_sync_progress(&self) -> Result<()> {
        let (target_height, peer_id) = {
            let state = self.current_state.read().unwrap();
            if let SyncState::Syncing {
                target_height,
                peer_id,
                ..
            } = *state
            {
                (target_height, peer_id)
            } else {
                return Ok(());
            }
        };

        let new_height = self.get_current_height().await?;
        let progress = (new_height as f64 / target_height as f64) * 100.0;

        let _ = self.event_tx.send(SyncEvent::SyncProgress {
            current_height: new_height,
            target_height,
            percentage: progress,
        });

        // Check if sync is complete
        if new_height >= target_height {
            self.complete_sync(new_height).await?;
        } else {
            // Update current height in sync state
            *self.current_state.write().unwrap() = SyncState::Syncing {
                current_height: new_height,
                target_height,
                peer_id,
            };
        }

        Ok(())
    }

    /// Complete synchronization
    async fn complete_sync(&self, final_height: u64) -> Result<()> {
        let initial_height = match *self.current_state.read().unwrap() {
            SyncState::Syncing { current_height, .. } => current_height,
            _ => 0,
        };

        *self.current_state.write().unwrap() = SyncState::Synced;

        let _ = self.event_tx.send(SyncEvent::SyncCompleted {
            final_height,
            blocks_synced: final_height.saturating_sub(initial_height),
        });

        // Clean up pending blocks
        {
            let mut pending = self.pending_blocks.write().unwrap();
            pending.clear();
        }

        log::info!("Synchronization completed at height {}", final_height);
        Ok(())
    }

    /// Handle synchronization error
    async fn handle_sync_error(&self, peer_id: NodeId, error: String) -> Result<()> {
        // Mark peer as unreliable
        {
            let mut peers = self.peer_info.write().unwrap();
            if let Some(peer) = peers.get_mut(&peer_id) {
                peer.sync_quality *= 0.5; // Reduce quality score
                peer.is_syncing = false;
            }
        }

        let _ = self.event_tx.send(SyncEvent::SyncFailed { error, peer_id });

        // Try to find another peer for sync
        self.evaluate_sync_status().await?;

        Ok(())
    }

    /// Start the synchronization loop
    async fn start_sync_loop(&self) {
        let sync_queue = Arc::clone(&self.sync_queue);

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_millis(100));

            loop {
                interval.tick().await;

                // Process sync queue
                let request = {
                    let mut queue = sync_queue.write().unwrap();
                    queue.pop_front()
                };

                if let Some(request) = request {
                    log::debug!("Processing sync request: {:?}", request);
                    // Send request to appropriate peer
                    // Implementation would send via network layer
                }
            }
        });
    }

    /// Start the verification loop
    async fn start_verification_loop(&self) {
        let pending_blocks = Arc::clone(&self.pending_blocks);
        let event_tx = self.event_tx.clone();

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(5));

            loop {
                interval.tick().await;

                // Verify pending blocks
                let blocks_to_verify: Vec<_> = {
                    let pending = pending_blocks.read().unwrap();
                    pending.values().cloned().collect()
                };

                for block in blocks_to_verify {
                    // Simplified verification
                    let verified = !block.get_hash().is_empty();

                    let _ = event_tx.send(SyncEvent::StateVerified {
                        block_height: block.get_height() as u64,
                        verified,
                    });
                }
            }
        });
    }

    /// Get current blockchain height
    async fn get_current_height(&self) -> Result<u64> {
        self.storage.get_latest_block_height().await
    }

    /// Get block hash at specific height
    async fn get_block_hash_at_height(&self, height: u64) -> Result<Option<String>> {
        if let Some(block) = self.storage.get_block_by_height(height).await? {
            Ok(Some(block.get_hash().to_string()))
        } else {
            Ok(None)
        }
    }

    /// Get current synchronization state
    pub fn get_sync_state(&self) -> SyncState {
        self.current_state.read().unwrap().clone()
    }

    /// Get peer synchronization information
    pub fn get_peer_sync_info(&self) -> Vec<PeerSyncInfo> {
        self.peer_info.read().unwrap().values().cloned().collect()
    }

    /// Get verified checkpoints
    pub fn get_verified_checkpoints(&self) -> Vec<(u64, String)> {
        self.verified_checkpoints
            .read()
            .unwrap()
            .iter()
            .map(|(height, hash)| (*height, hash.clone()))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    #[tokio::test]
    async fn test_sync_state_creation() {
        let temp_dir = TempDir::new().unwrap();
        let storage = Arc::new(ModularStorage::new_with_path(temp_dir.path()).unwrap());
        let config = SyncConfig::default();

        let (synchronizer, _event_rx) = StateSynchronizer::new(config, storage);

        assert_eq!(synchronizer.get_sync_state(), SyncState::Synced);
    }

    #[tokio::test]
    async fn test_peer_info_update() {
        let temp_dir = TempDir::new().unwrap();
        let storage = Arc::new(ModularStorage::new_with_path(temp_dir.path()).unwrap());
        let config = SyncConfig::default();

        let (synchronizer, _event_rx) = StateSynchronizer::new(config, storage);

        let node_id = NodeId::random();
        let _ = synchronizer
            .update_peer_info(node_id, 100, "test_hash".to_string(), 1000)
            .await;

        let peer_info = synchronizer.get_peer_sync_info();
        assert_eq!(peer_info.len(), 1);
        assert_eq!(peer_info[0].node_id, node_id);
        assert_eq!(peer_info[0].best_height, 100);
    }

    #[tokio::test]
    async fn test_sync_request_queueing() {
        let temp_dir = TempDir::new().unwrap();
        let storage = Arc::new(ModularStorage::new_with_path(temp_dir.path()).unwrap());
        let config = SyncConfig::default();

        let (synchronizer, _event_rx) = StateSynchronizer::new(config, storage);

        synchronizer.queue_sync_requests(0, 10).await.unwrap();

        let queue_len = {
            let queue = synchronizer.sync_queue.read().unwrap();
            queue.len()
        };

        // Should have requests for blocks 1-10
        assert!(queue_len > 0);
    }
}
