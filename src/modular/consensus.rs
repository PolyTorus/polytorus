//! Modular consensus layer implementation
//!
//! This module implements the consensus layer for the modular blockchain,
//! handling block validation and chain management.

use super::traits::*;
use crate::blockchain::block::Block;
use crate::blockchain::blockchain::Blockchain;
use crate::config::DataContext;
use crate::Result;

use std::sync::{Arc, Mutex};

/// Consensus layer implementation using Proof of Work
pub struct PolyTorusConsensusLayer {
    /// Underlying blockchain
    blockchain: Arc<Mutex<Blockchain>>,
    /// Validator information
    validators: Arc<Mutex<Vec<ValidatorInfo>>>,
    /// Current validator status
    is_validator: bool,
    /// Consensus configuration
    config: ConsensusConfig,
}

impl PolyTorusConsensusLayer {
    /// Create a new consensus layer
    pub fn new(
        data_context: DataContext,
        config: ConsensusConfig,
        is_validator: bool,
    ) -> Result<Self> {
        let blockchain = match Blockchain::new_with_context(data_context.clone()) {
            Ok(bc) => {
                // Check if blockchain is empty (no genesis block)
                if bc.tip.is_empty() {
                    let genesis_address = "genesis_address".to_string();
                    Blockchain::create_blockchain_with_context(genesis_address, data_context)?
                } else {
                    bc
                }
            }
            Err(_) => {
                // Create genesis blockchain if it doesn't exist
                let genesis_address = "genesis_address".to_string();
                Blockchain::create_blockchain_with_context(genesis_address, data_context)?
            }
        };

        Ok(Self {
            blockchain: Arc::new(Mutex::new(blockchain)),
            validators: Arc::new(Mutex::new(Vec::new())),
            is_validator,
            config,
        })
    }

    /// Validate block structure and proof of work
    fn validate_block_structure(&self, block: &Block) -> bool {
        // Check basic block structure
        if block.get_hash().is_empty() {
            return false;
        }

        // Validate proof of work
        self.validate_proof_of_work(block)
    }

    /// Validate proof of work
    fn validate_proof_of_work(&self, block: &Block) -> bool {
        // Recreate the hash and check if it meets difficulty requirement
        let hash = block.get_hash();
        let difficulty_target = "0".repeat(self.config.difficulty);

        hash.starts_with(&difficulty_target)
    }

    /// Check if block height is valid
    fn validate_block_height(&self, block: &Block) -> Result<bool> {
        let blockchain = self.blockchain.lock().unwrap();
        let current_height = blockchain.get_best_height()?;

        // Block height should be current height + 1
        Ok(block.get_height() == current_height + 1)
    }

    /// Validate block against parent
    fn validate_block_parent(&self, block: &Block) -> Result<bool> {
        let blockchain = self.blockchain.lock().unwrap();

        // Get the current tip (last block)
        if blockchain.tip.is_empty() {
            // Genesis block case
            return Ok(block.get_prev_hash().is_empty());
        }

        // Check if previous hash matches current tip
        Ok(block.get_prev_hash() == blockchain.tip)
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
    fn propose_block(&self, block: Block) -> Result<()> {
        if !self.is_validator {
            return Err(failure::format_err!("Node is not a validator"));
        }

        // Validate the proposed block
        if !self.validate_block(&block) {
            return Err(failure::format_err!("Invalid block proposed"));
        }

        // Add block to the chain
        let mut blockchain = self.blockchain.lock().unwrap();
        blockchain.add_block(block)?;

        Ok(())
    }

    fn validate_block(&self, block: &Block) -> bool {
        // Basic structure validation
        if !self.validate_block_structure(block) {
            return false;
        }

        // Height validation
        if let Ok(valid_height) = self.validate_block_height(block) {
            if !valid_height {
                return false;
            }
        } else {
            return false;
        }

        // Parent validation
        if let Ok(valid_parent) = self.validate_block_parent(block) {
            if !valid_parent {
                return false;
            }
        } else {
            return false;
        }

        // Transaction validation would go here
        // For now, we assume all transactions are valid

        true
    }

    fn get_canonical_chain(&self) -> Vec<Hash> {
        let blockchain = self.blockchain.lock().unwrap();
        blockchain.get_block_hashs()
    }

    fn get_block_height(&self) -> Result<u64> {
        let blockchain = self.blockchain.lock().unwrap();
        let height = blockchain.get_best_height()?;

        // Handle the case where blockchain is empty (height = -1)
        if height < 0 {
            Ok(0) // Genesis blockchain starts at height 0
        } else {
            Ok(height as u64)
        }
    }

    fn get_block_by_hash(&self, hash: &Hash) -> Result<Block> {
        let blockchain = self.blockchain.lock().unwrap();
        blockchain.get_block(hash)
    }

    fn add_block(&mut self, block: Block) -> Result<()> {
        // Validate before adding
        if !self.validate_block(&block) {
            return Err(failure::format_err!("Block validation failed"));
        }

        let mut blockchain = self.blockchain.lock().unwrap();
        blockchain.add_block(block)?;
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

    pub fn as_validator(mut self) -> Self {
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
