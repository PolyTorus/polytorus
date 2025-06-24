//! Type-safe block implementation with compile-time guarantees and Verkle tree support

use std::{marker::PhantomData, time::SystemTime};

use bincode::serialize;
use log::info;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

use crate::{
    blockchain::types::{block_states, network, BlockState, NetworkConfig},
    crypto::{
        transaction::*,
        verkle_tree::{VerklePoint, VerkleProof, VerkleTree},
    },
    Result,
};

#[cfg(test)]
pub const TEST_DIFFICULTY: usize = 1;

/// Difficulty adjustment parameters
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct DifficultyAdjustmentConfig {
    /// Base difficulty
    pub base_difficulty: usize,
    /// Minimum difficulty
    pub min_difficulty: usize,
    /// Maximum difficulty
    pub max_difficulty: usize,
    /// Adjustment factor strength (0.0-1.0)
    pub adjustment_factor: f64,
    /// Tolerance percentage from target block time (%)
    pub tolerance_percentage: f64,
}

impl Default for DifficultyAdjustmentConfig {
    fn default() -> Self {
        Self {
            base_difficulty: 4,
            min_difficulty: 1,
            max_difficulty: 32,
            adjustment_factor: 0.25,
            tolerance_percentage: 20.0,
        }
    }
}

/// Mining statistics information
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct MiningStats {
    /// Average mining time
    pub avg_mining_time: u128,
    /// Recent block times
    pub recent_block_times: Vec<u128>,
    /// Total mining attempts
    pub total_attempts: u64,
    /// Successful mining count
    pub successful_mines: u64,
}

impl Default for MiningStats {
    fn default() -> Self {
        Self {
            avg_mining_time: 0,
            recent_block_times: Vec::with_capacity(10),
            total_attempts: 0,
            successful_mines: 0,
        }
    }
}

impl MiningStats {
    /// Record new mining time
    pub fn record_mining_time(&mut self, mining_time: u128) {
        self.recent_block_times.push(mining_time);
        if self.recent_block_times.len() > 10 {
            self.recent_block_times.remove(0);
        }
        self.update_average();
        self.successful_mines += 1;
    }

    /// Record mining attempt
    pub fn record_attempt(&mut self) {
        self.total_attempts += 1;
    }

    /// Update average time
    fn update_average(&mut self) {
        if !self.recent_block_times.is_empty() {
            self.avg_mining_time = self.recent_block_times.iter().sum::<u128>()
                / self.recent_block_times.len() as u128;
        }
    }

    /// Calculate success rate
    pub fn success_rate(&self) -> f64 {
        if self.total_attempts == 0 {
            0.0
        } else {
            self.successful_mines as f64 / self.total_attempts as f64
        }
    }
}

/// Test parameters for creating finalized blocks
#[cfg(test)]
#[derive(Clone)]
pub struct TestFinalizedParams {
    pub prev_block_hash: String,
    pub hash: String,
    pub nonce: i32,
    pub height: i32,
    pub difficulty: usize,
    pub difficulty_config: DifficultyAdjustmentConfig,
    pub mining_stats: MiningStats,
}

/// Type-safe block with state tracking and Verkle tree support
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
    /// Difficulty adjustment configuration
    difficulty_config: DifficultyAdjustmentConfig,
    /// Mining statistics
    mining_stats: MiningStats,
    /// Verkle tree for transaction commitments
    #[serde(skip)]
    verkle_tree: Option<VerkleTree>,
    /// Root commitment of the Verkle tree (serializable)
    verkle_root_commitment: Option<Vec<u8>>,
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
            difficulty_config: DifficultyAdjustmentConfig::default(),
            mining_stats: MiningStats::default(),
            verkle_tree: None,
            verkle_root_commitment: None,
            _state: PhantomData,
            _network: PhantomData,
        }
    }

    /// Create a new block with custom difficulty configuration
    pub fn new_building_with_config(
        transactions: Vec<Transaction>,
        prev_block_hash: String,
        height: i32,
        difficulty: usize,
        difficulty_config: DifficultyAdjustmentConfig,
        mining_stats: MiningStats,
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
            difficulty_config,
            mining_stats,
            verkle_tree: None,
            verkle_root_commitment: None,
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

    /// Get difficulty configuration
    pub fn get_difficulty_config(&self) -> &DifficultyAdjustmentConfig {
        &self.difficulty_config
    }

    /// Update difficulty configuration
    pub fn update_difficulty_config(&mut self, config: DifficultyAdjustmentConfig) {
        self.difficulty_config = config;
    }

    /// Get mining statistics
    pub fn get_mining_stats(&self) -> &MiningStats {
        &self.mining_stats
    }

    /// Update mining statistics
    pub fn update_mining_stats(&mut self, stats: MiningStats) {
        self.mining_stats = stats;
    }

    /// Calculate dynamic difficulty based on current difficulty
    pub fn calculate_dynamic_difficulty(
        &self,
        recent_blocks: &[&Block<block_states::Finalized, N>],
    ) -> usize {
        if recent_blocks.is_empty() {
            return self.difficulty_config.base_difficulty;
        }

        // Collect recent block times
        let mut block_times = Vec::new();
        for i in 1..recent_blocks.len() {
            let time_diff = recent_blocks[i].timestamp - recent_blocks[i - 1].timestamp;
            block_times.push(time_diff);
        }

        if block_times.is_empty() {
            return self.difficulty_config.base_difficulty;
        }

        // Calculate average block time
        let avg_time = block_times.iter().sum::<u128>() / block_times.len() as u128;
        let target_time = N::DESIRED_BLOCK_TIME;

        // Compare with target time
        let time_ratio = avg_time as f64 / target_time as f64;
        let tolerance = self.difficulty_config.tolerance_percentage / 100.0;

        let mut new_difficulty = self.difficulty as f64;

        if time_ratio < (1.0 - tolerance) {
            // If block time is too short, increase difficulty
            new_difficulty *= 1.0 + (self.difficulty_config.adjustment_factor * (1.0 - time_ratio));
        } else if time_ratio > (1.0 + tolerance) {
            // If block time is too long, decrease difficulty
            new_difficulty *= 1.0 - (self.difficulty_config.adjustment_factor * (time_ratio - 1.0));
        }

        // Apply min/max difficulty limits
        let adjusted_difficulty = new_difficulty.round() as usize;
        adjusted_difficulty
            .max(self.difficulty_config.min_difficulty)
            .min(self.difficulty_config.max_difficulty)
    }
}
impl<N: NetworkConfig> BuildingBlock<N> {
    /// Mine the block using Proof-of-Work
    pub fn mine(mut self) -> Result<MinedBlock<N>> {
        info!("Mining the block with difficulty {}", self.difficulty);
        let start_time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis();

        // Update mining statistics
        self.mining_stats.record_attempt();

        while !self.validate_pow()? {
            self.nonce += 1;
            if self.nonce % 10000 == 0 {
                self.mining_stats.record_attempt();
                info!(
                    "Mining attempt: {}, nonce: {}",
                    self.mining_stats.total_attempts, self.nonce
                );
            }
        }

        let end_time = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis();
        let mining_time = end_time.saturating_sub(start_time);
        self.mining_stats.record_mining_time(mining_time);

        let data = self.prepare_hash_data()?;
        let mut hasher = Sha256::new();
        hasher.update(&data[..]);
        self.hash = hex::encode(hasher.finalize());

        info!(
            "Block mined successfully! Mining time: {}ms, Nonce: {}, Hash: {}",
            mining_time,
            self.nonce,
            &self.hash[..8]
        );

        Ok(Block {
            timestamp: self.timestamp,
            transactions: self.transactions,
            prev_block_hash: self.prev_block_hash,
            hash: self.hash,
            nonce: self.nonce,
            height: self.height,
            difficulty: self.difficulty,
            difficulty_config: self.difficulty_config,
            mining_stats: self.mining_stats,
            verkle_tree: self.verkle_tree,
            verkle_root_commitment: self.verkle_root_commitment,
            _state: PhantomData,
            _network: PhantomData,
        })
    }

    /// Mine with custom difficulty
    pub fn mine_with_difficulty(mut self, custom_difficulty: usize) -> Result<MinedBlock<N>> {
        self.difficulty = custom_difficulty
            .max(self.difficulty_config.min_difficulty)
            .min(self.difficulty_config.max_difficulty);
        self.mine()
    }

    /// Mine with adaptive difficulty based on recent blocks
    pub fn mine_adaptive(
        mut self,
        recent_blocks: &[&Block<block_states::Finalized, N>],
    ) -> Result<MinedBlock<N>> {
        let adaptive_difficulty = self.calculate_dynamic_difficulty(recent_blocks);
        self.difficulty = adaptive_difficulty;
        info!("Using adaptive difficulty: {}", self.difficulty);
        self.mine()
    }
}

impl<N: NetworkConfig> MinedBlock<N> {
    /// Validate the block completely
    pub fn validate(mut self) -> Result<ValidatedBlock<N>> {
        // Validate proof of work
        if !self.validate_pow()? {
            return Err(anyhow::anyhow!("Invalid proof of work"));
        }

        // Basic transaction validation (more comprehensive validation would require UTXO set)
        if self.transactions.is_empty() {
            return Err(anyhow::anyhow!(
                "Block must contain at least one transaction"
            ));
        }

        // Check that the first transaction is coinbase
        if !self.transactions[0].is_coinbase() {
            return Err(anyhow::anyhow!("First transaction must be coinbase"));
        }

        // Check that only the first transaction is coinbase
        for tx in &self.transactions[1..] {
            if tx.is_coinbase() {
                return Err(anyhow::anyhow!("Only first transaction can be coinbase"));
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
            difficulty_config: self.difficulty_config,
            mining_stats: self.mining_stats,
            verkle_tree: self.verkle_tree,
            verkle_root_commitment: self.verkle_root_commitment,
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
            difficulty_config: self.difficulty_config,
            mining_stats: self.mining_stats,
            verkle_tree: self.verkle_tree,
            verkle_root_commitment: self.verkle_root_commitment,
            _state: PhantomData,
            _network: PhantomData,
        }
    }
}

impl<S: BlockState, N: NetworkConfig> Block<S, N> {
    /// Validate proof of work
    fn validate_pow(&mut self) -> Result<bool> {
        let data = self.prepare_hash_data()?;
        let mut hasher = Sha256::new();
        hasher.update(&data[..]);
        let hash_str = hex::encode(hasher.finalize());
        let prefix = "0".repeat(self.difficulty);
        Ok(hash_str.starts_with(&prefix))
    }
    /// Hash all transactions using Verkle tree
    fn hash_transactions(&mut self) -> Result<Vec<u8>> {
        let root_commitment = self.get_verkle_root_commitment()?;

        // Use Blake3 to hash the root commitment for block hashing
        let hash = blake3::hash(&root_commitment);
        Ok(hash.as_bytes().to_vec())
    }

    fn prepare_hash_data(&mut self) -> Result<Vec<u8>> {
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
            // Dynamic difficulty adjustment based on block timing
            // If no recent blocks are available, use initial difficulty
            N::INITIAL_DIFFICULTY
        };

        Self::new_building(transactions, prev_block_hash, height, difficulty)
    }

    /// Create a new block with network-specific parameters and previous blocks
    pub fn new_with_network_config_and_history(
        transactions: Vec<Transaction>,
        prev_block_hash: String,
        height: i32,
        recent_blocks: &[&Block<block_states::Finalized, N>],
    ) -> Self {
        let difficulty = if height == 0 || recent_blocks.is_empty() {
            N::INITIAL_DIFFICULTY
        } else {
            // Calculate dynamic difficulty based on recent blocks timing
            Self::calculate_difficulty_from_history(recent_blocks)
        };

        Self::new_building(transactions, prev_block_hash, height, difficulty)
    }

    /// Calculate difficulty based on block history
    fn calculate_difficulty_from_history(
        recent_blocks: &[&Block<block_states::Finalized, N>],
    ) -> usize {
        if recent_blocks.len() < 2 {
            return N::INITIAL_DIFFICULTY;
        }

        // Calculate average block time from recent blocks
        let mut total_time_diff = 0u128;
        let mut block_count = 0;

        for i in 1..recent_blocks.len() {
            let time_diff = recent_blocks[i].timestamp - recent_blocks[i - 1].timestamp;
            total_time_diff += time_diff;
            block_count += 1;
        }

        if block_count == 0 {
            return N::INITIAL_DIFFICULTY;
        }

        let avg_block_time = total_time_diff / block_count as u128;
        let target_time = N::DESIRED_BLOCK_TIME;

        // Adjust difficulty based on timing
        let current_difficulty = recent_blocks.last().unwrap().difficulty;

        if avg_block_time < target_time / 2 {
            // Blocks are coming too fast - increase difficulty significantly
            (current_difficulty * 2).min(32)
        } else if avg_block_time < target_time * 4 / 5 {
            // Blocks are somewhat fast - increase difficulty moderately
            (current_difficulty + 1).min(32)
        } else if avg_block_time > target_time * 2 {
            // Blocks are coming too slow - decrease difficulty significantly
            (current_difficulty / 2).max(1)
        } else if avg_block_time > target_time * 5 / 4 {
            // Blocks are somewhat slow - decrease difficulty moderately
            if current_difficulty > 1 {
                current_difficulty - 1
            } else {
                1
            }
        } else {
            // Block timing is acceptable - maintain current difficulty
            current_difficulty
        }
    }

    /// Create genesis block
    pub fn new_genesis(coinbase: Transaction) -> FinalizedBlock<N> {
        let building_block = Self::new_with_network_config(vec![coinbase], String::new(), 0);

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
    /// Basic difficulty adjustment
    pub fn adjust_difficulty(&self, current_timestamp: u128) -> usize {
        let time_diff = current_timestamp - self.timestamp;
        let mut new_difficulty = self.difficulty;

        if time_diff < N::DESIRED_BLOCK_TIME {
            new_difficulty += 1;
        } else if time_diff > N::DESIRED_BLOCK_TIME && new_difficulty > 1 {
            new_difficulty -= 1;
        }
        new_difficulty
    }

    /// Advanced difficulty adjustment (considering multiple block history)
    pub fn adjust_difficulty_advanced(
        &self,
        recent_blocks: &[&Block<block_states::Finalized, N>],
    ) -> usize {
        if recent_blocks.len() < 2 {
            return self.difficulty_config.base_difficulty;
        }

        // Calculate time variance
        let mut block_times = Vec::new();
        for i in 1..recent_blocks.len() {
            let time_diff = recent_blocks[i].timestamp - recent_blocks[i - 1].timestamp;
            block_times.push(time_diff);
        }

        if block_times.is_empty() {
            return self.difficulty_config.base_difficulty;
        }

        // Calculate average time and variance
        let avg_time = block_times.iter().sum::<u128>() / block_times.len() as u128;
        let variance = block_times
            .iter()
            .map(|&time| {
                let diff = time as f64 - avg_time as f64;
                diff * diff
            })
            .sum::<f64>()
            / block_times.len() as f64;

        let target_time = N::DESIRED_BLOCK_TIME as f64;
        let time_ratio = avg_time as f64 / target_time;

        // Adjustment considering variance
        let stability_factor = 1.0 + (variance.sqrt() / target_time).min(0.5);
        let adjustment = self.difficulty_config.adjustment_factor * stability_factor;

        let mut new_difficulty = self.difficulty as f64;

        if time_ratio < 0.8 {
            // If very fast, increase significantly
            new_difficulty *= 1.0 + adjustment;
        } else if time_ratio < 0.9 {
            // If somewhat fast, increase slightly
            new_difficulty *= 1.0 + (adjustment * 0.5);
        } else if time_ratio > 1.2 {
            // If very slow, decrease significantly
            new_difficulty *= 1.0 - adjustment;
        } else if time_ratio > 1.1 {
            // If somewhat slow, decrease slightly
            new_difficulty *= 1.0 - (adjustment * 0.5);
        }

        let result = new_difficulty.round() as usize;
        result
            .max(self.difficulty_config.min_difficulty)
            .min(self.difficulty_config.max_difficulty)
    }

    /// Calculate mining efficiency
    pub fn calculate_mining_efficiency(&self) -> f64 {
        if self.mining_stats.total_attempts == 0 {
            return 0.0;
        }

        let success_rate = self.mining_stats.success_rate();
        let avg_time = self.mining_stats.avg_mining_time as f64;
        let target_time = N::DESIRED_BLOCK_TIME as f64;

        // Efficiency = success rate * (target time / actual time)
        let time_efficiency = if avg_time > 0.0 {
            target_time / avg_time
        } else {
            0.0
        };

        let efficiency = success_rate * time_efficiency;
        efficiency.min(2.0) // Limit maximum efficiency to 200%
    }

    /// Calculate recommended difficulty value for the entire network
    pub fn recommend_network_difficulty(
        &self,
        network_hash_rate: f64,
        target_hash_rate: f64,
    ) -> usize {
        let hash_rate_ratio = network_hash_rate / target_hash_rate;
        let current_difficulty = self.difficulty as f64;

        let recommended = current_difficulty * hash_rate_ratio;

        (recommended.round() as usize)
            .max(self.difficulty_config.min_difficulty)
            .min(self.difficulty_config.max_difficulty)
    }
    /// Test helper to create a finalized block (should only be used in tests)
    #[cfg(test)]
    pub fn new_test_finalized(transactions: Vec<Transaction>, params: TestFinalizedParams) -> Self {
        Block {
            timestamp: SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_millis(),
            transactions,
            prev_block_hash: params.prev_block_hash,
            hash: params.hash,
            nonce: params.nonce,
            height: params.height,
            difficulty: params.difficulty,
            difficulty_config: params.difficulty_config,
            mining_stats: params.mining_stats,
            verkle_tree: None,
            verkle_root_commitment: None,
            _state: PhantomData,
            _network: PhantomData,
        }
    }
}

impl<S: BlockState, N: NetworkConfig> Block<S, N> {
    /// Get or build the Verkle tree for this block
    pub fn get_or_build_verkle_tree(&mut self) -> Result<&VerkleTree> {
        if self.verkle_tree.is_none() {
            let mut tree = VerkleTree::new();

            // Insert all transactions into the Verkle tree
            for (i, tx) in self.transactions.iter().enumerate() {
                let key = format!("tx_{:08x}", i);
                let value = bincode::serialize(tx)?;
                tree.insert(key.as_bytes(), &value).map_err(|e| {
                    anyhow::anyhow!("Failed to insert transaction into Verkle tree: {}", e)
                })?;
            }

            // Store the root commitment
            let root_commitment = tree.get_root_commitment();
            self.verkle_root_commitment = Some(bincode::serialize(&root_commitment)?);
            self.verkle_tree = Some(tree);
        }

        Ok(self.verkle_tree.as_ref().unwrap())
    }

    /// Get the Verkle tree root commitment
    pub fn get_verkle_root_commitment(&mut self) -> Result<Vec<u8>> {
        if self.verkle_root_commitment.is_none() {
            self.get_or_build_verkle_tree()?;
        }
        Ok(self.verkle_root_commitment.clone().unwrap_or_default())
    }

    /// Generate a Verkle proof for a transaction
    pub fn generate_transaction_proof(&mut self, tx_index: usize) -> Result<VerkleProof> {
        let tree = self.get_or_build_verkle_tree()?;
        let key = format!("tx_{:08x}", tx_index);
        tree.generate_proof(key.as_bytes())
            .map_err(|e| anyhow::anyhow!("Failed to generate proof: {}", e))
    }
    /// Verify a Verkle proof against this block's commitment
    pub fn verify_transaction_proof(&self, proof: &VerkleProof) -> bool {
        if let Some(ref commitment_bytes) = self.verkle_root_commitment {
            if let Ok(expected_commitment) = bincode::deserialize::<VerklePoint>(commitment_bytes) {
                return proof.root_commitment.0 == expected_commitment.0;
            }
        }
        false
    }
}

#[cfg(test)]
mod verkle_integration_tests {
    use super::*;
    fn create_test_transaction(from: &str, to: &str, amount: i64) -> Transaction {
        Transaction::new_coinbase(
            to.to_string(),
            format!("transfer {} from {} to {}", amount, from, to),
        )
        .unwrap()
    }

    #[test]
    fn test_verkle_tree_in_block_creation() {
        // Create test transactions
        let tx1 = create_test_transaction("alice", "bob", 100);
        let tx2 = create_test_transaction("bob", "charlie", 50);
        let tx3 = create_test_transaction("charlie", "dave", 25);
        let transactions = vec![tx1, tx2, tx3];

        // Create a building block
        let mut block = Block::<block_states::Building, network::Mainnet>::new_building(
            transactions.clone(),
            "prev_hash".to_string(),
            1,
            4,
        );

        // Build the Verkle tree
        let tree = block.get_or_build_verkle_tree().unwrap();
        // Verify the tree is not empty
        use ark_std::Zero;
        assert!(!tree.get_root_commitment().0.is_zero());

        // Get the root commitment
        let root_commitment = block.get_verkle_root_commitment().unwrap();
        assert!(!root_commitment.is_empty());
    }

    #[test]
    fn test_verkle_proof_generation_and_verification() {
        // Create test transactions
        let tx1 = create_test_transaction("alice", "bob", 100);
        let tx2 = create_test_transaction("bob", "charlie", 50);
        let transactions = vec![tx1.clone(), tx2.clone()];

        // Create a building block
        let mut block = Block::<block_states::Building, network::Mainnet>::new_building(
            transactions,
            "prev_hash".to_string(),
            1,
            4,
        );

        // Generate proof for the first transaction
        let proof = block.generate_transaction_proof(0).unwrap();

        // Verify the proof
        assert!(block.verify_transaction_proof(&proof));

        // Generate proof for the second transaction
        let proof2 = block.generate_transaction_proof(1).unwrap();

        // Verify the second proof
        assert!(block.verify_transaction_proof(&proof2));

        // Verify that the proofs are different
        assert_ne!(proof.key, proof2.key);
    }

    #[test]
    fn test_verkle_tree_with_empty_transactions() {
        // Create a block with no transactions
        let mut block = Block::<block_states::Building, network::Mainnet>::new_building(
            vec![],
            "prev_hash".to_string(),
            1,
            4,
        );

        // Build the Verkle tree
        let tree = block.get_or_build_verkle_tree().unwrap();
        // Verify the tree has identity root for empty tree
        let root_commitment = tree.get_root_commitment();
        use crate::crypto::verkle_tree::VerklePoint;
        // The root should be the identity element for empty tree
        assert_eq!(root_commitment.0, VerklePoint::identity().0);
    }

    #[test]
    fn test_verkle_tree_deterministic_commitment() {
        // Create same transactions in two different blocks
        let tx1 = create_test_transaction("alice", "bob", 100);
        let tx2 = create_test_transaction("bob", "charlie", 50);

        let transactions = vec![tx1.clone(), tx2.clone()];
        // Create two identical blocks
        let mut block1 = Block::<block_states::Building, network::Mainnet>::new_building(
            transactions.clone(),
            "prev_hash".to_string(),
            1,
            4,
        );

        let mut block2 = Block::<block_states::Building, network::Mainnet>::new_building(
            transactions,
            "prev_hash".to_string(),
            1,
            4,
        );

        // Get commitments from both blocks
        let commitment1 = block1.get_verkle_root_commitment().unwrap();
        let commitment2 = block2.get_verkle_root_commitment().unwrap();

        // Commitments should be identical for identical transaction sets
        assert_eq!(commitment1, commitment2);
    }

    #[test]
    fn test_verkle_proof_size_efficiency() {
        // Create test transactions
        let tx1 = create_test_transaction("alice", "bob", 100);
        let tx2 = create_test_transaction("bob", "charlie", 50);
        let tx3 = create_test_transaction("charlie", "dave", 25);

        let transactions = vec![tx1, tx2, tx3];
        // Create a building block
        let mut block = Block::<block_states::Building, network::Mainnet>::new_building(
            transactions,
            "prev_hash".to_string(),
            1,
            4,
        );

        // Generate proof for a transaction
        let proof = block.generate_transaction_proof(0).unwrap();

        // Check proof size (should be reasonably small)
        let proof_size = proof.size();
        assert!(proof_size > 0);
        println!("Verkle proof size: {} bytes", proof_size);

        // Proof should be reasonably compact (less than 10KB for small trees)
        assert!(proof_size < 10_000);
    }
}
