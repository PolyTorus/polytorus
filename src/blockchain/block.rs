//! Type-safe block implementation with compile-time guarantees

use crate::blockchain::types::{block_states, network, BlockState, NetworkConfig};
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
    /// Difficulty adjustment configuration
    difficulty_config: DifficultyAdjustmentConfig,
    /// Mining statistics
    mining_stats: MiningStats,
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
        let mining_time = end_time - start_time;
        self.mining_stats.record_mining_time(mining_time);

        let data = self.prepare_hash_data()?;
        let mut hasher = Sha256::new();
        hasher.input(&data[..]);
        self.hash = hasher.result_str();

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
            difficulty_config: self.difficulty_config,
            mining_stats: self.mining_stats,
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
impl<N: NetworkConfig> Block<block_states::Building, N> {    /// Create a new block with network-specific parameters
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
        let difficulty = if height == 0 {
            N::INITIAL_DIFFICULTY
        } else if recent_blocks.is_empty() {
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
            if current_difficulty > 1 { current_difficulty - 1 } else { 1 }
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
            _state: PhantomData,
            _network: PhantomData,
        }
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
