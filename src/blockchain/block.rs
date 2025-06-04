//! Type-safe block implementation with compile-time guarantees

use crate::blockchain::types::{
    block_states, network, BlockState, NetworkConfig,
};
use crate::crypto::transaction::*;
use crate::Result;
use bincode::serialize;
use crypto::digest::Digest;
use crypto::sha2::Sha256;
use failure::format_err;
use merkle_cbt::merkle_tree::Merge;
use merkle_cbt::merkle_tree::CBMT;
use serde::{Deserialize, Serialize};
use std::marker::PhantomData;
use std::time::SystemTime;

#[cfg(test)]
pub const TEST_DIFFICULTY: usize = 1;

/// Type-safe block with state tracking
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block<S = block_states::Finalized, N = network::Mainnet> 
where 
    S: BlockState,
    N: NetworkConfig,
{
    timestamp: u128,
    transactions: Vec<Transaction>,
    prev_block_hash: String,
    hash: String,
    nonce: i32,
    height: i32,
    difficulty: usize,
    #[serde(skip)]
    _state: PhantomData<S>,
    #[serde(skip)]
    _network: PhantomData<N>,
}

/// Type alias for building blocks
pub type BuildingBlock<N = network::Mainnet> = Block<block_states::Building, N>;

/// Type alias for mined blocks
pub type MinedBlock<N = network::Mainnet> = Block<block_states::Mined, N>;

/// Type alias for validated blocks
pub type ValidatedBlock<N = network::Mainnet> = Block<block_states::Validated, N>;

/// Type alias for finalized blocks
pub type FinalizedBlock<N = network::Mainnet> = Block<block_states::Finalized, N>;

/// Block builder with type-level guarantees
pub struct BlockBuilder<S: BlockState, N: NetworkConfig> {
    #[allow(dead_code)]
    block: Block<S, N>,
}

/// Proof-of-Work validator
pub struct ProofOfWorkValidator<N: NetworkConfig> {
    _network: PhantomData<N>,
}

/// Transaction validator
pub struct TransactionValidator<N: NetworkConfig> {
    _network: PhantomData<N>,
}

impl<S: BlockState, N: NetworkConfig> Block<S, N> {
    /// Create a new block in building state
    pub fn new_building(
        transactions: Vec<Transaction>,
        prev_block_hash: String,
        height: i32,
        difficulty: usize,
    ) -> BuildingBlock<N> {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis();
        
        Block {
            timestamp,
            transactions,
            prev_block_hash,
            hash: String::new(),
            nonce: 0,
            height,
            difficulty,
            _state: PhantomData,
            _network: PhantomData,
        }
    }
    
    pub fn get_hash(&self) -> &str {
        &self.hash
    }

    pub fn get_prev_hash(&self) -> &str {
        &self.prev_block_hash
    }

    pub fn get_transactions(&self) -> &[Transaction] {
        &self.transactions
    }

    pub fn get_height(&self) -> i32 {
        self.height
    }

    pub fn get_timestamp(&self) -> u128 {
        self.timestamp
    }
    
    pub fn get_difficulty(&self) -> usize {
        self.difficulty
    }
    
    pub fn get_nonce(&self) -> i32 {
        self.nonce
    }
}impl<N: NetworkConfig> BuildingBlock<N> {
    /// Mine the block using Proof-of-Work
    pub fn mine(mut self) -> Result<MinedBlock<N>> {
        info!("Mining the block");
        while !self.validate_pow()? {
            self.nonce += 1;
        }
        let data = self.prepare_hash_data()?;
        let mut hasher = Sha256::new();
        hasher.input(&data[..]);
        self.hash = hasher.result_str();
        
        Ok(Block {
            timestamp: self.timestamp,
            transactions: self.transactions,
            prev_block_hash: self.prev_block_hash,
            hash: self.hash,
            nonce: self.nonce,
            height: self.height,
            difficulty: self.difficulty,
            _state: PhantomData,
            _network: PhantomData,
        })
    }
}

impl<N: NetworkConfig> MinedBlock<N> {
    /// Validate the block completely
    pub fn validate(self) -> Result<ValidatedBlock<N>> {
        // Validate proof of work
        if !self.validate_pow()? {
            return Err(format_err!("Invalid proof of work"));
        }
        
        // Basic transaction validation (more comprehensive validation would require UTXO set)
        if self.transactions.is_empty() {
            return Err(format_err!("Block must contain at least one transaction"));
        }
        
        // Check that the first transaction is coinbase
        if !self.transactions[0].is_coinbase() {
            return Err(format_err!("First transaction must be coinbase"));
        }
        
        // Check that only the first transaction is coinbase
        for tx in &self.transactions[1..] {
            if tx.is_coinbase() {
                return Err(format_err!("Only first transaction can be coinbase"));
            }
        }
        
        Ok(Block {
            timestamp: self.timestamp,
            transactions: self.transactions,
            prev_block_hash: self.prev_block_hash,
            hash: self.hash,
            nonce: self.nonce,
            height: self.height,
            difficulty: self.difficulty,
            _state: PhantomData,
            _network: PhantomData,
        })
    }
}

impl<N: NetworkConfig> ValidatedBlock<N> {
    /// Finalize the block for blockchain inclusion
    pub fn finalize(self) -> FinalizedBlock<N> {
        Block {
            timestamp: self.timestamp,
            transactions: self.transactions,
            prev_block_hash: self.prev_block_hash,
            hash: self.hash,
            nonce: self.nonce,
            height: self.height,
            difficulty: self.difficulty,
            _state: PhantomData,
            _network: PhantomData,
        }
    }
}

impl<S: BlockState, N: NetworkConfig> Block<S, N> {
    /// Validate proof of work
    fn validate_pow(&self) -> Result<bool> {
        let data = self.prepare_hash_data()?;
        let mut hasher = Sha256::new();
        hasher.input(&data[..]);
        let hash_str = hasher.result_str();
        let prefix = "0".repeat(self.difficulty);
        Ok(hash_str.starts_with(&prefix))
    }
    
    /// Hash all transactions using Merkle tree
    fn hash_transactions(&self) -> Result<Vec<u8>> {
        let mut transactions = Vec::new();
        for tx in &self.transactions {
            transactions.push(tx.hash()?.as_bytes().to_owned());
        }
        let tree = CBMT::<Vec<u8>, MergeVu8>::build_merkle_tree(transactions);
        Ok(tree.root())
    }

    fn prepare_hash_data(&self) -> Result<Vec<u8>> {
        let content = (
            self.prev_block_hash.clone(),
            self.hash_transactions()?,
            self.timestamp,
            self.difficulty,
            self.nonce,
        );
        let bytes = serialize(&content)?;
        Ok(bytes)
    }
}

/// Network-specific block creation
impl<N: NetworkConfig> Block<block_states::Building, N> {
    /// Create a new block with network-specific parameters
    pub fn new_with_network_config(
        transactions: Vec<Transaction>,
        prev_block_hash: String,
        height: i32,
    ) -> Self {
        let difficulty = if height == 0 {
            N::INITIAL_DIFFICULTY
        } else {
            N::INITIAL_DIFFICULTY // This would be calculated based on previous blocks
        };
        
        Self::new_building(transactions, prev_block_hash, height, difficulty)
    }
    
    /// Create genesis block
    pub fn new_genesis(coinbase: Transaction) -> FinalizedBlock<N> {
        let building_block = Self::new_with_network_config(
            vec![coinbase], 
            String::new(), 
            0
        );
        
        building_block
            .mine()
            .unwrap()
            .validate()
            .unwrap()
            .finalize()
    }
}

/// Difficulty adjustment with type safety
impl<N: NetworkConfig> Block<block_states::Finalized, N> {
    pub fn adjust_difficulty(&self, current_timestamp: u128) -> usize {
        let time_diff = current_timestamp - self.timestamp;
        let mut new_difficulty = self.difficulty;

        if time_diff < N::DESIRED_BLOCK_TIME {
            new_difficulty += 1;
        } else if time_diff > N::DESIRED_BLOCK_TIME && new_difficulty > 1 {
            new_difficulty -= 1;
        }        new_difficulty
    }
}

struct MergeVu8 {}

impl Merge for MergeVu8 {
    type Item = Vec<u8>;
    fn merge(left: &Self::Item, right: &Self::Item) -> Self::Item {
        let mut hasher = Sha256::new();
        let mut data: Vec<u8> = left.clone();
        data.append(&mut right.clone());
        hasher.input(&data);
        let mut re: [u8; 32] = [0; 32];
        hasher.result(&mut re);
        re.to_vec()
    }
}
