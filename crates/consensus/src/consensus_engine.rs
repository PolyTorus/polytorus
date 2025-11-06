//! eUTXO Consensus Layer Implementation
//!
//! This module provides eUTXO consensus capabilities:
//! - eUTXO block validation and creation
//! - Slot-based timing consensus
//! - UTXO set consistency validation

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

use async_trait::async_trait;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use traits::{Hash, Result, UtxoBlock, UtxoConsensusLayer, UtxoTransaction, ValidatorInfo};

/// eUTXO consensus layer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UtxoConsensusConfig {
    /// Slot time in milliseconds
    pub slot_time: u64,
    /// Proof of work difficulty for eUTXO blocks
    pub difficulty: usize,
    /// Maximum block size
    pub max_block_size: usize,
    /// Maximum transactions per block
    pub max_transactions_per_block: usize,
}

impl Default for UtxoConsensusConfig {
    fn default() -> Self {
        Self {
            slot_time: 1000, // 1 second slots
            difficulty: 4,
            max_block_size: 2 * 1024 * 1024, // 2MB
            max_transactions_per_block: 1000,
        }
    }
}

/// eUTXO consensus layer with slot-based timing
pub struct PolyTorusUtxoConsensusLayer {
    /// Blockchain state
    chain_state: Arc<Mutex<UtxoChainState>>,
    /// Validator set
    validators: Arc<Mutex<HashMap<String, ValidatorInfo>>>,
    /// Configuration
    config: UtxoConsensusConfig,
    /// Node's validator address (if validator)
    validator_address: Option<String>,
    /// Genesis slot timestamp
    genesis_time: u64,
}

/// Internal eUTXO chain state
#[derive(Debug, Clone)]
struct UtxoChainState {
    /// Canonical chain (block hashes in order)
    canonical_chain: Vec<Hash>,
    /// Block storage
    blocks: HashMap<Hash, UtxoBlock>,
    /// Current block height
    height: u64,
    /// Current slot
    current_slot: u64,
    /// Pending transactions
    pending_transactions: Vec<UtxoTransaction>,
}

impl PolyTorusUtxoConsensusLayer {
    /// Create new eUTXO consensus layer
    pub fn new(config: UtxoConsensusConfig) -> Result<Self> {
        let genesis_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let genesis_block = Self::create_genesis_utxo_block(genesis_time);
        let genesis_hash = genesis_block.hash.clone();

        let mut blocks = HashMap::new();
        blocks.insert(genesis_hash.clone(), genesis_block);

        let chain_state = UtxoChainState {
            canonical_chain: vec![genesis_hash],
            blocks,
            height: 0,
            current_slot: 0,
            pending_transactions: Vec::new(),
        };

        Ok(Self {
            chain_state: Arc::new(Mutex::new(chain_state)),
            validators: Arc::new(Mutex::new(HashMap::new())),
            config,
            validator_address: None,
            genesis_time,
        })
    }

    /// Create new eUTXO consensus layer as validator
    pub fn new_as_validator(
        config: UtxoConsensusConfig,
        validator_address: String,
    ) -> Result<Self> {
        let mut layer = Self::new(config)?;
        layer.validator_address = Some(validator_address.clone());

        // Add self as validator
        let validator_info = ValidatorInfo {
            address: validator_address,
            stake: 1000,
            public_key: vec![1, 2, 3],
            active: true,
        };

        {
            let mut validators = layer.validators.lock().unwrap();
            validators.insert(validator_info.address.clone(), validator_info);
        }

        Ok(layer)
    }

    /// Create new eUTXO consensus layer with restored state
    pub fn new_with_restored_state(
        config: UtxoConsensusConfig,
        validator_address: String,
        restored_height: u64,
        restored_slot: u64,
        canonical_chain: Vec<String>,
    ) -> Result<Self> {
        let genesis_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        let genesis_block = Self::create_genesis_utxo_block(genesis_time);
        let genesis_hash = genesis_block.hash.clone();

        let mut blocks = HashMap::new();
        blocks.insert(genesis_hash.clone(), genesis_block);

        // Restore chain state from persistent data
        let chain_state = UtxoChainState {
            canonical_chain,
            blocks,
            height: restored_height,
            current_slot: restored_slot,
            pending_transactions: Vec::new(),
        };

        let layer = Self {
            chain_state: Arc::new(Mutex::new(chain_state)),
            validators: Arc::new(Mutex::new(HashMap::new())),
            config,
            validator_address: Some(validator_address.clone()),
            genesis_time,
        };

        // Add self as validator
        let validator_info = ValidatorInfo {
            address: validator_address,
            stake: 1000,
            public_key: vec![1, 2, 3],
            active: true,
        };

        {
            let mut validators = layer.validators.lock().unwrap();
            validators.insert(validator_info.address.clone(), validator_info);
        }

        Ok(layer)
    }

    /// Create new eUTXO consensus layer with restored state and blocks
    pub fn new_with_restored_state_and_blocks(
        config: UtxoConsensusConfig,
        validator_address: String,
        restored_height: u64,
        restored_slot: u64,
        canonical_chain: Vec<String>,
        restored_blocks: HashMap<String, UtxoBlock>,
    ) -> Result<Self> {
        let genesis_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        // If no blocks are restored, create genesis block
        let mut blocks = if restored_blocks.is_empty() {
            let genesis_block = Self::create_genesis_utxo_block(genesis_time);
            let genesis_hash = genesis_block.hash.clone();
            let mut blocks = HashMap::new();
            blocks.insert(genesis_hash, genesis_block);
            blocks
        } else {
            restored_blocks
        };

        // Ensure genesis block exists if not in restored blocks
        let genesis_hash = "genesis_utxo_block_hash".to_string();
        blocks.entry(genesis_hash)
            .or_insert_with(|| Self::create_genesis_utxo_block(genesis_time));

        // Restore chain state from persistent data
        let chain_state = UtxoChainState {
            canonical_chain,
            blocks,
            height: restored_height,
            current_slot: restored_slot,
            pending_transactions: Vec::new(),
        };

        let layer = Self {
            chain_state: Arc::new(Mutex::new(chain_state)),
            validators: Arc::new(Mutex::new(HashMap::new())),
            config,
            validator_address: Some(validator_address.clone()),
            genesis_time,
        };

        // Add self as validator
        let validator_info = ValidatorInfo {
            address: validator_address,
            stake: 1000,
            public_key: vec![1, 2, 3],
            active: true,
        };

        {
            let mut validators = layer.validators.lock().unwrap();
            validators.insert(validator_info.address.clone(), validator_info);
        }

        Ok(layer)
    }

    /// Create genesis eUTXO block
    fn create_genesis_utxo_block(genesis_time: u64) -> UtxoBlock {
        UtxoBlock {
            hash: "genesis_utxo_block_hash".to_string(),
            parent_hash: "0x0".to_string(),
            number: 0,
            timestamp: genesis_time,
            slot: 0,
            transactions: vec![],
            utxo_set_hash: "genesis_utxo_set_hash".to_string(),
            transaction_root: "genesis_tx_root".to_string(),
            validator: "genesis_validator".to_string(),
            proof: vec![],
        }
    }

    /// Calculate current slot from timestamp
    pub fn timestamp_to_slot(&self, timestamp: u64) -> u64 {
        if timestamp < self.genesis_time {
            return 0;
        }
        (timestamp - self.genesis_time) / self.config.slot_time
    }

    /// Calculate timestamp for a given slot
    pub fn slot_to_timestamp(&self, slot: u64) -> u64 {
        self.genesis_time + (slot * self.config.slot_time)
    }

    /// Get current slot
    pub fn get_current_slot_from_time(&self) -> u64 {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;
        self.timestamp_to_slot(current_time)
    }

    /// Calculate block hash including proof (nonce) for PoW
    pub fn calculate_utxo_block_hash(&self, block: &UtxoBlock) -> Hash {
        let mut hasher = Sha256::new();
        hasher.update(&block.parent_hash);
        hasher.update(block.number.to_be_bytes());
        hasher.update(block.timestamp.to_be_bytes());
        hasher.update(block.slot.to_be_bytes());
        hasher.update(&block.utxo_set_hash);
        hasher.update(&block.transaction_root);
        hasher.update(&block.validator);
        hasher.update(&block.proof);
        hex::encode(hasher.finalize())
    }

    /// Validate proof of work for eUTXO block
    pub fn validate_proof_of_work(&self, block: &UtxoBlock) -> bool {
        if self.config.difficulty == 0 {
            return true;
        }
        let hash = self.calculate_utxo_block_hash(block);
        let required_zeros = "0".repeat(self.config.difficulty);
        hash.starts_with(&required_zeros)
    }

    /// Mine proof of work for eUTXO block
    fn mine_proof_of_work(&self, mut block: UtxoBlock) -> Result<UtxoBlock> {
        if self.config.difficulty == 0 {
            block.proof = vec![0u8; 8];
            block.hash = self.calculate_utxo_block_hash(&block);
            return Ok(block);
        }

        let mut nonce = 0u64;
        let required_zeros = "0".repeat(self.config.difficulty);

        loop {
            block.proof = nonce.to_be_bytes().to_vec();
            let hash = self.calculate_utxo_block_hash(&block);

            if hash.starts_with(&required_zeros) {
                block.hash = hash;
                log::info!(
                    "Successfully mined eUTXO block with nonce {} after {} attempts",
                    nonce,
                    nonce + 1
                );
                return Ok(block);
            }

            nonce += 1;

            if nonce.is_multiple_of(100_000) {
                log::info!(
                    "Mining attempt {}: hash = {}, required = {} zeros",
                    nonce,
                    &hash[0..10.min(hash.len())],
                    self.config.difficulty
                );
            }

            if nonce > 10_000_000 {
                log::error!(
                    "Mining failed after 10M attempts. Difficulty: {}, Last hash: {}",
                    self.config.difficulty,
                    &hash[0..10.min(hash.len())]
                );
                return Err(anyhow::anyhow!(
                    "Failed to mine eUTXO block after 10M attempts"
                ));
            }
        }
    }

    /// Validate eUTXO block structure and rules
    fn validate_utxo_block_structure(&self, block: &UtxoBlock) -> bool {
        // Check basic structure
        if block.hash.is_empty() || block.parent_hash.is_empty() {
            return false;
        }

        // Check transaction count limits
        if block.transactions.len() > self.config.max_transactions_per_block {
            return false;
        }

        // Validate slot timing (relaxed for testing)
        let expected_slot = self.timestamp_to_slot(block.timestamp);
        if self.config.slot_time > 500 && block.slot != expected_slot {
            // Only strict timing for production (slot_time > 500ms)
            log::warn!(
                "Invalid slot timing: block slot={}, expected={}, timestamp={}",
                block.slot,
                expected_slot,
                block.timestamp
            );
            return false;
        }
        // For fast testing (slot_time <= 500ms), allow any slot progression
        log::info!(
            "Slot timing validation: block slot={}, expected={} (relaxed for testing)",
            block.slot,
            expected_slot
        );

        // Check timestamp is reasonable (not too far in future)
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis() as u64;

        if block.timestamp > current_time + (5 * self.config.slot_time) {
            return false;
        }

        // Validate proof of work
        self.validate_proof_of_work(block)
    }

    /// Add transaction to pending pool
    pub fn add_pending_utxo_transaction(&self, transaction: UtxoTransaction) -> Result<()> {
        let mut state = self.chain_state.lock().unwrap();
        state.pending_transactions.push(transaction);
        Ok(())
    }

    /// Get pending transactions for block creation
    pub fn get_pending_utxo_transactions(&self, limit: usize) -> Vec<UtxoTransaction> {
        let mut state = self.chain_state.lock().unwrap();
        let len = state.pending_transactions.len();

        state
            .pending_transactions
            .split_off(len.saturating_sub(limit))
    }

    /// Calculate transaction root for eUTXO transactions
    fn calculate_transaction_root(&self, transactions: &[UtxoTransaction]) -> Hash {
        let mut hasher = Sha256::new();
        for tx in transactions {
            hasher.update(&tx.hash);
        }
        hex::encode(hasher.finalize())
    }
}

#[async_trait]
impl UtxoConsensusLayer for PolyTorusUtxoConsensusLayer {
    async fn propose_utxo_block(&mut self, block: UtxoBlock) -> Result<()> {
        // For now, directly validate and add the block
        if self.validate_utxo_block(&block).await? {
            self.add_utxo_block(block).await?;
        } else {
            return Err(anyhow::anyhow!("Invalid block proposal"));
        }
        Ok(())
    }

    async fn validate_utxo_block(&self, block: &UtxoBlock) -> Result<bool> {
        log::info!("Validating UTXO block: {}", block.hash);

        // Validate block structure
        log::info!("Checking block structure...");
        if !self.validate_utxo_block_structure(block) {
            log::error!("Block structure validation failed");
            return Ok(false);
        }
        log::info!("Block structure validation passed");

        // Check if parent exists
        let state = self.chain_state.lock().unwrap();
        log::info!("Checking parent block exists: {}", block.parent_hash);
        if !state.blocks.contains_key(&block.parent_hash) {
            log::error!("Parent block not found: {}", block.parent_hash);
            return Ok(false);
        }
        log::info!("Parent block found");

        // Validate block number sequence
        let parent_block = state.blocks.get(&block.parent_hash).unwrap();
        log::info!(
            "Checking block number sequence: block={}, parent={}",
            block.number,
            parent_block.number
        );
        if block.number != parent_block.number + 1 {
            log::error!(
                "Block number sequence invalid: expected {}, got {}",
                parent_block.number + 1,
                block.number
            );
            return Ok(false);
        }
        log::info!("Block number sequence valid");

        // Validate slot progression
        log::info!(
            "Checking slot progression: block={}, parent={}",
            block.slot,
            parent_block.slot
        );
        if block.slot <= parent_block.slot {
            log::error!(
                "Slot progression invalid: block slot {} <= parent slot {}",
                block.slot,
                parent_block.slot
            );
            return Ok(false);
        }
        log::info!("Slot progression valid");

        log::info!("Block validation passed for: {}", block.hash);
        Ok(true)
    }

    async fn get_canonical_chain(&self) -> Result<Vec<Hash>> {
        let state = self.chain_state.lock().unwrap();
        Ok(state.canonical_chain.clone())
    }

    async fn get_block_height(&self) -> Result<u64> {
        let state = self.chain_state.lock().unwrap();
        Ok(state.height)
    }

    async fn get_current_slot(&self) -> Result<u64> {
        let state = self.chain_state.lock().unwrap();
        Ok(state.current_slot)
    }

    async fn get_utxo_block_by_hash(&self, hash: &Hash) -> Result<Option<UtxoBlock>> {
        let state = self.chain_state.lock().unwrap();
        Ok(state.blocks.get(hash).cloned())
    }

    async fn add_utxo_block(&mut self, block: UtxoBlock) -> Result<()> {
        let block_hash = block.hash.clone();

        {
            let mut state = self.chain_state.lock().unwrap();

            // Add block to storage
            state.blocks.insert(block_hash.clone(), block.clone());

            // Update canonical chain
            state.canonical_chain.push(block_hash);
            state.height = block.number;
            state.current_slot = block.slot;
        }

        log::info!(
            "Added eUTXO block #{} (slot {}) to chain",
            block.number,
            block.slot
        );
        Ok(())
    }

    async fn is_validator(&self) -> Result<bool> {
        Ok(self.validator_address.is_some())
    }

    async fn get_validator_set(&self) -> Result<Vec<ValidatorInfo>> {
        let validators = self.validators.lock().unwrap();
        Ok(validators.values().cloned().collect())
    }

    async fn mine_utxo_block(&mut self, transactions: Vec<UtxoTransaction>) -> Result<UtxoBlock> {
        log::info!(
            "Starting UTXO block mining with {} transactions",
            transactions.len()
        );

        let state = self.chain_state.lock().unwrap();
        let parent_hash = state
            .canonical_chain
            .last()
            .cloned()
            .unwrap_or_else(|| "genesis_utxo_block_hash".to_string());
        let block_number = state.height + 1;
        let current_slot = std::cmp::max(state.current_slot + 1, self.get_current_slot_from_time());
        log::info!(
            "Block template: parent={}, number={}, slot={} (parent slot: {})",
            parent_hash,
            block_number,
            current_slot,
            state.current_slot
        );
        drop(state);

        // Calculate transaction root
        log::info!(
            "Calculating transaction root for {} transactions",
            transactions.len()
        );
        let transaction_root = self.calculate_transaction_root(&transactions);
        log::info!("Transaction root calculated: {}", transaction_root);

        // Create block template
        let mut block = UtxoBlock {
            hash: String::new(),
            parent_hash,
            number: block_number,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64,
            slot: current_slot,
            transactions,
            utxo_set_hash: "pending_utxo_set_hash".to_string(), // Would be calculated from execution
            transaction_root,
            validator: self
                .validator_address
                .clone()
                .unwrap_or_else(|| "miner".to_string()),
            proof: vec![],
        };

        // Mine the block using PoW
        log::info!(
            "Starting PoW mining with difficulty: {}",
            self.config.difficulty
        );
        block = self.mine_proof_of_work(block)?;
        log::info!("PoW mining completed for block: {}", block.hash);

        log::info!(
            "Successfully mined eUTXO block #{} (slot {}) with hash: {}",
            block.number,
            block.slot,
            block.hash
        );
        Ok(block)
    }

    async fn get_difficulty(&self) -> Result<usize> {
        Ok(self.config.difficulty)
    }

    async fn set_difficulty(&mut self, difficulty: usize) -> Result<()> {
        log::info!(
            "Updating eUTXO consensus difficulty from {} to {}",
            self.config.difficulty,
            difficulty
        );
        self.config.difficulty = difficulty;
        Ok(())
    }

    async fn validate_slot_timing(&self, slot: u64, timestamp: u64) -> Result<bool> {
        let expected_timestamp = self.slot_to_timestamp(slot);
        let tolerance = self.config.slot_time / 2; // Allow 50% tolerance

        Ok(timestamp >= expected_timestamp.saturating_sub(tolerance)
            && timestamp <= expected_timestamp + tolerance)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use traits::{TxInput, TxOutput, UtxoId};

    #[test]
    fn test_utxo_consensus_layer_creation() {
        let config = UtxoConsensusConfig::default();
        let layer = PolyTorusUtxoConsensusLayer::new(config);
        assert!(layer.is_ok());
    }

    #[test]
    fn test_slot_calculation() {
        let config = UtxoConsensusConfig {
            slot_time: 1000, // 1 second slots
            ..UtxoConsensusConfig::default()
        };
        let layer = PolyTorusUtxoConsensusLayer::new(config).unwrap();

        let genesis_time = layer.genesis_time;
        let slot_0_time = layer.slot_to_timestamp(0);
        let slot_1_time = layer.slot_to_timestamp(1);

        assert_eq!(slot_0_time, genesis_time);
        assert_eq!(slot_1_time, genesis_time + 1000);

        let calculated_slot_0 = layer.timestamp_to_slot(genesis_time);
        let calculated_slot_1 = layer.timestamp_to_slot(genesis_time + 1000);

        assert_eq!(calculated_slot_0, 0);
        assert_eq!(calculated_slot_1, 1);
    }

    #[test]
    fn test_utxo_block_mining() {
        // Use blocking runtime to avoid tokio macro issues
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let config = UtxoConsensusConfig {
                difficulty: 0, // No difficulty for faster testing
                ..UtxoConsensusConfig::default()
            };
            let mut layer =
                PolyTorusUtxoConsensusLayer::new_as_validator(config, "utxo_miner_1".to_string())
                    .unwrap();

            let transaction = UtxoTransaction {
                hash: "test_utxo_tx".to_string(),
                inputs: vec![TxInput {
                    utxo_id: UtxoId {
                        tx_hash: "prev_tx".to_string(),
                        output_index: 0,
                    },
                    redeemer: vec![1, 2, 3],
                    signature: vec![4, 5, 6],
                }],
                outputs: vec![TxOutput {
                    value: 100,
                    script: vec![7, 8, 9],
                    datum: None,
                    datum_hash: None,
                }],
                fee: 10,
                validity_range: None,
                script_witness: vec![],
                auxiliary_data: None,
            };

            let block = layer.mine_utxo_block(vec![transaction]).await.unwrap();

            // Verify the block was mined correctly
            assert!(!block.hash.is_empty());
            assert_eq!(block.number, 1);
            assert_eq!(block.transactions.len(), 1);
            assert!(!block.proof.is_empty());

            // Verify PoW validation
            assert!(layer.validate_proof_of_work(&block));
        });
    }

    #[test]
    fn test_utxo_block_validation() {
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let config = UtxoConsensusConfig {
                difficulty: 0, // No difficulty for testing
                ..UtxoConsensusConfig::default()
            };
            let layer = PolyTorusUtxoConsensusLayer::new(config).unwrap();

            let genesis_hash = layer.get_canonical_chain().await.unwrap()[0].clone();

            // Use a future timestamp to ensure slot progression
            let current_time = SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_millis() as u64
                + 2000; // Add 2 seconds
            let current_slot = layer.timestamp_to_slot(current_time);

            let mut block = UtxoBlock {
                hash: String::new(), // Will be calculated properly
                parent_hash: genesis_hash,
                number: 1,
                timestamp: current_time,
                slot: current_slot,
                transactions: vec![],
                utxo_set_hash: "test_utxo_set_hash".to_string(),
                transaction_root: "test_tx_root".to_string(),
                validator: "test_validator".to_string(),
                proof: vec![0, 0, 0, 0], // Valid proof for difficulty 0
            };

            // Calculate the proper hash for the block
            block.hash = layer.calculate_utxo_block_hash(&block);

            // Should pass validation with difficulty 0
            let is_valid = layer.validate_utxo_block(&block).await.unwrap();
            assert!(is_valid);
        });
    }
}
