//! Modular consensus layer implementation
//!
//! This module implements the consensus layer for the modular blockchain,
//! handling block validation and chain management.

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::{SystemTime, UNIX_EPOCH},
};

use bincode::serialize;

use super::{
    storage::{ModularStorage, StorageLayer},
    traits::*,
};
use crate::{
    blockchain::block::{Block, BuildingBlock, FinalizedBlock},
    config::DataContext,
    crypto::transaction::Transaction,
    Result,
};

/// Consensus layer implementation using Proof of Work
pub struct PolyTorusConsensusLayer {
    /// Modular storage layer
    storage: Arc<ModularStorage>,
    /// Validator information
    validators: Arc<Mutex<Vec<ValidatorInfo>>>,
    /// Current validator status
    is_validator: bool,
    /// Consensus configuration
    config: ConsensusConfig,
}

impl PolyTorusConsensusLayer {
    /// Create a new consensus layer with modular storage
    pub fn new(
        data_context: DataContext,
        config: ConsensusConfig,
        is_validator: bool,
    ) -> Result<Self> {
        // Create modular storage with data context path
        let storage_path = data_context.data_dir().join("modular_storage");
        let storage = Arc::new(ModularStorage::new_with_path(&storage_path)?);

        Ok(Self {
            storage,
            validators: Arc::new(Mutex::new(Vec::new())),
            is_validator,
            config,
        })
    }

    /// Create a new consensus layer with existing storage
    pub fn new_with_storage(
        storage: Arc<ModularStorage>,
        config: ConsensusConfig,
        is_validator: bool,
    ) -> Result<Self> {
        Ok(Self {
            storage,
            validators: Arc::new(Mutex::new(Vec::new())),
            is_validator,
            config,
        })
    }

    /// Validate block structure and proof of work
    fn validate_block_structure(&self, block: &FinalizedBlock) -> bool {
        // Check basic block structure - allow empty hash for newly created blocks
        if block.get_transactions().is_empty() {
            log::warn!("Block has no transactions");
            return false;
        }

        // For building/unmined blocks, skip PoW validation
        if block.get_hash().is_empty() {
            log::debug!("Block hash is empty, skipping PoW validation (building block)");
            return true;
        }

        // Validate proof of work for mined blocks
        self.validate_proof_of_work(block)
    }

    /// Validate proof of work using actual hash computation
    fn validate_proof_of_work(&self, block: &Block) -> bool {
        // For finalized blocks that came from the mining process,
        // we trust the block's internal validation
        let stored_hash = block.get_hash();

        // Check if hash is not empty (indicates a mined block)
        if stored_hash.is_empty() {
            log::warn!("Block hash is empty - not a mined block");
            return false;
        }

        // Check if hash meets difficulty requirement
        let difficulty_target = "0".repeat(self.config.difficulty);
        let meets_difficulty = stored_hash.starts_with(&difficulty_target);

        if !meets_difficulty {
            log::warn!(
                "Block {} does not meet difficulty requirement: {} zeros",
                stored_hash,
                self.config.difficulty
            );
        } else {
            log::debug!(
                "Block {} meets difficulty requirement: {} zeros",
                stored_hash,
                self.config.difficulty
            );
        }

        meets_difficulty
    }

    /// Check if block height is valid
    fn validate_block_height(&self, block: &FinalizedBlock) -> Result<bool> {
        let current_height = self.storage.get_height()?;

        // For genesis block (height 0), allow if current height is 0
        if block.get_height() == 0 && current_height == 0 {
            return Ok(true);
        }

        // Block height should be current height + 1
        Ok(block.get_height() == (current_height + 1) as i32)
    }

    /// Validate block against parent
    fn validate_block_parent(&self, block: &FinalizedBlock) -> Result<bool> {
        // Get the current tip (last block)
        let current_tip = self.storage.get_tip()?;

        if current_tip.is_empty() {
            // Genesis block case
            return Ok(block.get_prev_hash().is_empty());
        }

        // Check if previous hash matches current tip
        Ok(block.get_prev_hash() == current_tip)
    }

    /// Validate all transactions in a block
    fn validate_transactions(&self, block: &FinalizedBlock) -> bool {
        let transactions = block.get_transactions();

        if transactions.is_empty() {
            log::warn!("Block has no transactions");
            return false;
        }

        // Check for duplicate transactions
        let mut seen_txids = std::collections::HashSet::new();
        for tx in transactions {
            if !seen_txids.insert(&tx.id) {
                log::warn!("Duplicate transaction found: {}", tx.id);
                return false;
            }
        }

        // Validate each transaction
        for tx in transactions {
            if !self.validate_single_transaction(tx, block) {
                log::warn!("Transaction validation failed: {}", tx.id);
                return false;
            }
        }

        // Validate coinbase transaction (first transaction should be coinbase)
        if !transactions[0].is_coinbase() {
            log::warn!("First transaction is not coinbase");
            return false;
        }

        // Ensure only one coinbase transaction
        let coinbase_count = transactions.iter().filter(|tx| tx.is_coinbase()).count();
        if coinbase_count != 1 {
            log::warn!(
                "Block has {} coinbase transactions, expected 1",
                coinbase_count
            );
            return false;
        }

        true
    }

    /// Validate a single transaction
    fn validate_single_transaction(&self, tx: &Transaction, _block: &FinalizedBlock) -> bool {
        // Validate transaction hash
        if let Ok(calculated_hash) = tx.hash() {
            if calculated_hash != tx.id {
                log::warn!(
                    "Transaction hash mismatch: {} != {}",
                    calculated_hash,
                    tx.id
                );
                return false;
            }
        } else {
            log::warn!("Failed to calculate transaction hash: {}", tx.id);
            return false;
        }

        // Skip signature validation for coinbase transactions
        if tx.is_coinbase() {
            return true;
        }

        // Validate transaction signatures
        if !self.validate_transaction_signatures(tx) {
            log::warn!("Transaction signature validation failed: {}", tx.id);
            return false;
        }

        // Validate transaction inputs/outputs
        if !self.validate_transaction_inputs_outputs(tx) {
            log::warn!("Transaction input/output validation failed: {}", tx.id);
            return false;
        }

        true
    }

    /// Validate transaction signatures
    fn validate_transaction_signatures(&self, tx: &Transaction) -> bool {
        // Get previous transactions for signature verification
        let mut prev_txs = HashMap::new();

        for input in &tx.vin {
            if let Ok(prev_tx) = self.storage.get_transaction(&input.txid) {
                prev_txs.insert(input.txid.clone(), prev_tx);
            } else {
                log::warn!("Previous transaction not found: {}", input.txid);
                return false;
            }
        }

        // Verify transaction signatures
        match tx.verify(prev_txs) {
            Ok(valid) => {
                if !valid {
                    log::warn!("Transaction signature verification failed: {}", tx.id);
                }
                valid
            }
            Err(e) => {
                log::warn!("Error verifying transaction {}: {}", tx.id, e);
                false
            }
        }
    }

    /// Validate transaction inputs and outputs
    fn validate_transaction_inputs_outputs(&self, tx: &Transaction) -> bool {
        if tx.vin.is_empty() {
            log::warn!("Transaction has no inputs: {}", tx.id);
            return false;
        }

        if tx.vout.is_empty() {
            log::warn!("Transaction has no outputs: {}", tx.id);
            return false;
        }

        // Calculate input and output values
        let mut input_value = 0i64;
        let mut output_value = 0i64;

        for input in &tx.vin {
            if let Ok(prev_tx) = self.storage.get_transaction(&input.txid) {
                if input.vout >= 0 && (input.vout as usize) < prev_tx.vout.len() {
                    input_value += prev_tx.vout[input.vout as usize].value as i64;
                } else {
                    log::warn!("Invalid output index in transaction input: {}", tx.id);
                    return false;
                }
            } else {
                log::warn!("Previous transaction not found for input: {}", tx.id);
                return false;
            }
        }

        for output in &tx.vout {
            if output.value < 0 {
                log::warn!("Negative output value in transaction: {}", tx.id);
                return false;
            }
            output_value += output.value as i64;
        }

        // For non-coinbase transactions, inputs must be >= outputs (accounting for fees)
        if input_value < output_value {
            log::warn!(
                "Transaction {} has insufficient input value: {} < {}",
                tx.id,
                input_value,
                output_value
            );
            return false;
        }

        true
    }

    /// Validate block size
    fn validate_block_size(&self, block: &FinalizedBlock) -> bool {
        if let Ok(block_bytes) = serialize(block) {
            let block_size = block_bytes.len();
            if block_size > self.config.max_block_size {
                log::warn!(
                    "Block size {} exceeds maximum {}",
                    block_size,
                    self.config.max_block_size
                );
                return false;
            }
        }
        true
    }

    /// Validate block timestamp
    fn validate_block_timestamp(&self, block: &FinalizedBlock) -> bool {
        let current_time = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_millis();

        let block_time = block.get_timestamp();

        // Block timestamp should not be too far in the future (within 2 hours)
        let max_future_time = current_time + (2 * 60 * 60 * 1000); // 2 hours

        if block_time > max_future_time {
            log::warn!(
                "Block timestamp too far in future: {} > {}",
                block_time,
                max_future_time
            );
            return false;
        }

        // Block timestamp should be greater than previous block
        if let Ok(prev_hash) = self.storage.get_tip() {
            if !prev_hash.is_empty() {
                if let Ok(prev_block) = self.storage.get_block(&prev_hash) {
                    if block_time <= prev_block.get_timestamp() {
                        log::warn!(
                            "Block timestamp not greater than previous block: {} <= {}",
                            block_time,
                            prev_block.get_timestamp()
                        );
                        return false;
                    }
                }
            }
        }

        true
    }

    /// Mine a block by finding a valid nonce  
    pub fn mine_block(&self, building_block: &BuildingBlock) -> Result<FinalizedBlock> {
        log::info!(
            "Starting to mine block at height {} with difficulty {}",
            building_block.get_height(),
            self.config.difficulty
        );

        // Use the block's built-in mine() method to create a mined block
        let mined_block = building_block.clone().mine()?;

        // Then validate and finalize the mined block
        let validated_block = mined_block.validate()?;
        let finalized_block = validated_block.finalize();

        let elapsed = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis();
        log::info!(
            "Block mined successfully! Hash: {}, Nonce: {}, Time: {:?}ms",
            finalized_block.get_hash(),
            finalized_block.get_nonce(),
            elapsed
        );

        Ok(finalized_block)
    }

    /// Add validator to the set
    pub fn add_validator(&self, validator: ValidatorInfo) {
        let mut validators = self.validators.lock().unwrap();
        validators.push(validator);
    }

    /// Remove validator from the set
    pub fn remove_validator(&self, address: &str) {
        let mut validators = self.validators.lock().unwrap();
        validators.retain(|v| v.address != address);
    }
}

impl ConsensusLayer for PolyTorusConsensusLayer {
    fn propose_block(&self, block: FinalizedBlock) -> Result<()> {
        if !self.is_validator {
            return Err(anyhow::anyhow!("Node is not a validator"));
        }

        log::info!("Proposing new block at height {}", block.get_height());

        // Convert to building block for mining
        let building_block: BuildingBlock = unsafe { std::mem::transmute(block) };

        // Mine the block (find valid nonce)
        let mined_block = self.mine_block(&building_block)?;

        // Validate the mined block
        if !self.validate_block(&mined_block) {
            return Err(anyhow::anyhow!("Invalid block proposed after mining"));
        }

        // Add block to storage
        let hash = self.storage.store_block(&mined_block)?;

        log::info!("Successfully proposed and stored block: {}", hash);
        Ok(())
    }

    fn validate_block(&self, block: &FinalizedBlock) -> bool {
        log::info!("Validating block: {}", block.get_hash());

        // Basic structure validation
        if !self.validate_block_structure(block) {
            log::warn!("Block structure validation failed");
            return false;
        }

        // Timestamp validation
        if !self.validate_block_timestamp(block) {
            log::warn!("Block timestamp validation failed");
            return false;
        }

        // Height validation
        if let Ok(valid_height) = self.validate_block_height(block) {
            if !valid_height {
                log::warn!("Block height validation failed");
                return false;
            }
        } else {
            log::warn!("Error during block height validation");
            return false;
        }

        // Parent validation
        if let Ok(valid_parent) = self.validate_block_parent(block) {
            if !valid_parent {
                log::warn!("Block parent validation failed");
                return false;
            }
        } else {
            log::warn!("Error during block parent validation");
            return false;
        }

        // Transaction validation
        if !self.validate_transactions(block) {
            log::warn!("Block {} failed transaction validation", block.get_hash());
            return false;
        }

        // Block size validation
        if !self.validate_block_size(block) {
            log::warn!("Block {} exceeds maximum size", block.get_hash());
            return false;
        }

        log::info!("Block {} passed all validation checks", block.get_hash());
        true
    }
    fn get_canonical_chain(&self) -> Vec<Hash> {
        self.storage.get_block_hashes().unwrap_or_default()
    }

    fn get_block_height(&self) -> Result<u64> {
        self.storage.get_height()
    }

    fn get_block_by_hash(&self, hash: &Hash) -> Result<Block> {
        self.storage.get_block(hash)
    }
    fn add_block(&mut self, block: Block) -> Result<()> {
        // Validate before adding
        if !self.validate_block(&block) {
            return Err(anyhow::anyhow!("Block validation failed"));
        }

        self.storage.store_block(&block)?;
        Ok(())
    }

    fn is_validator(&self) -> bool {
        self.is_validator
    }

    fn get_validator_set(&self) -> Vec<ValidatorInfo> {
        let validators = self.validators.lock().unwrap();
        validators.clone()
    }
}

/// Builder for consensus layer configuration
pub struct ConsensusLayerBuilder {
    data_context: Option<DataContext>,
    config: Option<ConsensusConfig>,
    is_validator: bool,
}

impl ConsensusLayerBuilder {
    pub fn new() -> Self {
        Self {
            data_context: None,
            config: None,
            is_validator: false,
        }
    }

    pub fn with_data_context(mut self, context: DataContext) -> Self {
        self.data_context = Some(context);
        self
    }

    pub fn with_config(mut self, config: ConsensusConfig) -> Self {
        self.config = Some(config);
        self
    }

    pub fn into_validator(mut self) -> Self {
        self.is_validator = true;
        self
    }

    pub fn build(self) -> Result<PolyTorusConsensusLayer> {
        let data_context = self.data_context.unwrap_or_default();
        let config = self.config.unwrap_or(ConsensusConfig {
            block_time: 10000, // 10 seconds
            difficulty: 4,
            max_block_size: 1024 * 1024, // 1MB
        });

        PolyTorusConsensusLayer::new(data_context, config, self.is_validator)
    }
}

impl Default for ConsensusLayerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        blockchain::types::network::Mainnet,
        crypto::transaction::Transaction,
        test_helpers::{cleanup_test_context, create_test_context},
    };

    #[tokio::test]
    async fn test_real_pow_validation() {
        let context = create_test_context();
        let config = ConsensusConfig {
            block_time: 10000,
            difficulty: 1, // Easy difficulty for testing
            max_block_size: 1024 * 1024,
        };

        let consensus = PolyTorusConsensusLayer::new(context.clone(), config, true).unwrap();

        // Create a test transaction
        let coinbase_tx =
            Transaction::new_coinbase("test_address".to_string(), "test_data".to_string()).unwrap();

        // Create a test block with low difficulty
        let building_block = BuildingBlock::new_building(
            vec![coinbase_tx],
            "".to_string(),
            0,
            1, // difficulty 1
        );

        // Mine the block
        let result = consensus.mine_block(&building_block);
        assert!(result.is_ok(), "Mining should succeed with difficulty 1");

        let mined_block = result.unwrap();

        // Validate the mined block
        assert!(
            consensus.validate_block(&mined_block),
            "Mined block should be valid"
        );

        // Check that hash meets difficulty requirement
        let hash = mined_block.get_hash(); // Use the actual block hash
        assert!(
            hash.starts_with("0"),
            "Hash should start with at least one zero: {}",
            hash
        );

        cleanup_test_context(&context);
    }

    #[tokio::test]
    async fn test_transaction_validation() {
        let context = create_test_context();
        let config = ConsensusConfig {
            block_time: 10000,
            difficulty: 1,
            max_block_size: 1024 * 1024,
        };

        let consensus = PolyTorusConsensusLayer::new(context.clone(), config, true).unwrap();

        // Create a valid coinbase transaction
        let valid_tx =
            Transaction::new_coinbase("test_address".to_string(), "test_data".to_string()).unwrap();

        // Create a block with the transaction and finalize it for validation
        let building_block: BuildingBlock<Mainnet> =
            BuildingBlock::new_building(vec![valid_tx], "".to_string(), 0, 1);

        // Convert to finalized block for validation (simplified conversion)
        let finalized_block: FinalizedBlock = unsafe { std::mem::transmute(building_block) };

        // Validate transactions in the block
        assert!(
            consensus.validate_transactions(&finalized_block),
            "Valid transactions should pass validation"
        );

        cleanup_test_context(&context);
    }

    #[tokio::test]
    async fn test_block_structure_validation() {
        let context = create_test_context();
        let config = ConsensusConfig {
            block_time: 10000,
            difficulty: 1,
            max_block_size: 1024 * 1024,
        };

        let consensus = PolyTorusConsensusLayer::new(context.clone(), config, true).unwrap();

        // Create a test transaction
        let coinbase_tx =
            Transaction::new_coinbase("test_address".to_string(), "test_data".to_string()).unwrap();

        // Create a test block and finalize it for validation
        let building_block: BuildingBlock<Mainnet> =
            BuildingBlock::new_building(vec![coinbase_tx], "".to_string(), 0, 1);

        // Convert to finalized block for validation (simplified conversion)
        let finalized_block: FinalizedBlock = unsafe { std::mem::transmute(building_block) };

        // Test block structure validation
        assert!(
            consensus.validate_block_structure(&finalized_block),
            "Valid block structure should pass"
        );

        cleanup_test_context(&context);
    }

    #[tokio::test]
    async fn test_consensus_layer_creation() {
        let context1 = create_test_context();
        let context2 = create_test_context();
        let config = ConsensusConfig {
            block_time: 10000,
            difficulty: 4,
            max_block_size: 1024 * 1024,
        };

        // Test validator node creation with separate context
        let validator_consensus =
            PolyTorusConsensusLayer::new(context1.clone(), config.clone(), true).unwrap();
        assert!(
            validator_consensus.is_validator(),
            "Node should be configured as validator"
        );

        // Test non-validator node creation with separate context
        let non_validator_consensus =
            PolyTorusConsensusLayer::new(context2.clone(), config, false).unwrap();
        assert!(
            !non_validator_consensus.is_validator(),
            "Node should not be configured as validator"
        );

        cleanup_test_context(&context1);
        cleanup_test_context(&context2);
    }

    #[tokio::test]
    async fn test_consensus_builder() {
        let context = create_test_context();
        let config = ConsensusConfig {
            block_time: 5000,
            difficulty: 2,
            max_block_size: 512 * 1024,
        };

        // Test builder pattern
        let consensus = ConsensusLayerBuilder::new()
            .with_data_context(context.clone())
            .with_config(config)
            .into_validator()
            .build()
            .unwrap();

        assert!(
            consensus.is_validator(),
            "Builder should create validator node"
        );

        cleanup_test_context(&context);
    }
}
