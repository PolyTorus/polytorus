//! Modular Architecture Traits for PolyTorus
//!
//! This module defines the core interfaces for a modular blockchain architecture
//! where different layers can be independently developed, tested, and deployed.

use serde::{
    Deserialize,
    Serialize,
};

use crate::blockchain::block::Block;
use crate::crypto::transaction::Transaction;
use crate::Result;

/// Hash type for blockchain data
pub type Hash = String;

/// Execution result from processing a block
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// New state root after execution
    pub state_root: Hash,
    /// Gas used for execution
    pub gas_used: u64,
    /// Transaction receipts
    pub receipts: Vec<TransactionReceipt>,
    /// Events emitted during execution
    pub events: Vec<Event>,
}

/// Receipt for a single transaction execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionReceipt {
    /// Transaction hash
    pub tx_hash: Hash,
    /// Execution status
    pub success: bool,
    /// Gas used
    pub gas_used: u64,
    /// Events emitted
    pub events: Vec<Event>,
}

/// Event emitted during execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Event {
    /// Contract address that emitted the event
    pub contract: String,
    /// Event data
    pub data: Vec<u8>,
    /// Event topics
    pub topics: Vec<Hash>,
}

/// Batch of executions for settlement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionBatch {
    /// Batch identifier
    pub batch_id: Hash,
    /// Transactions in the batch
    pub transactions: Vec<Transaction>,
    /// Execution results
    pub results: Vec<ExecutionResult>,
    /// Previous state root
    pub prev_state_root: Hash,
    /// New state root
    pub new_state_root: Hash,
}

/// Result of settlement process
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettlementResult {
    /// Settlement root hash
    pub settlement_root: Hash,
    /// Settled batches
    pub settled_batches: Vec<Hash>,
    /// Settlement timestamp
    pub timestamp: u64,
}

/// Fraud proof for challenging invalid execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FraudProof {
    /// Disputed execution batch
    pub batch_id: Hash,
    /// Proof data
    pub proof_data: Vec<u8>,
    /// Expected state root
    pub expected_state_root: Hash,
    /// Actual state root
    pub actual_state_root: Hash,
}

/// Execution proof for state verification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionProof {
    /// State transition proof
    pub state_proof: Vec<u8>,
    /// Execution trace
    pub execution_trace: Vec<u8>,
    /// Input state root
    pub input_state_root: Hash,
    /// Output state root
    pub output_state_root: Hash,
}

/// Execution Layer Interface
///
/// Responsible for transaction execution and state transitions
pub trait ExecutionLayer: Send + Sync {
    /// Execute a block and return the execution result
    fn execute_block(&self, block: &Block) -> Result<ExecutionResult>;

    /// Get the current state root
    fn get_state_root(&self) -> Hash;

    /// Verify an execution proof
    fn verify_execution(&self, proof: &ExecutionProof) -> bool;

    /// Execute a single transaction
    fn execute_transaction(&self, tx: &Transaction) -> Result<TransactionReceipt>;

    /// Get account state
    fn get_account_state(&self, address: &str) -> Result<AccountState>;

    /// Begin a new execution context
    fn begin_execution(&mut self) -> Result<()>;

    /// Commit the current execution context
    fn commit_execution(&mut self) -> Result<Hash>;

    /// Rollback the current execution context
    fn rollback_execution(&mut self) -> Result<()>;
}

/// Settlement Layer Interface
///
/// Responsible for finalizing state transitions and handling disputes
pub trait SettlementLayer: Send + Sync {
    /// Settle a batch of executions
    fn settle_batch(&self, batch: &ExecutionBatch) -> Result<SettlementResult>;

    /// Verify a fraud proof
    fn verify_fraud_proof(&self, proof: &FraudProof) -> bool;

    /// Get the current settlement root
    fn get_settlement_root(&self) -> Hash;

    /// Process a settlement challenge
    fn process_challenge(&self, challenge: &SettlementChallenge) -> Result<ChallengeResult>;

    /// Get settlement history
    fn get_settlement_history(&self, limit: usize) -> Result<Vec<SettlementResult>>;
}

/// Consensus Layer Interface
///
/// Responsible for block ordering and validator management
pub trait ConsensusLayer: Send + Sync {
    /// Propose a new block
    fn propose_block(&self, block: Block) -> Result<()>;

    /// Validate a proposed block
    fn validate_block(&self, block: &Block) -> bool;

    /// Get the canonical chain
    fn get_canonical_chain(&self) -> Vec<Hash>;

    /// Get the current block height
    fn get_block_height(&self) -> Result<u64>;

    /// Get block by hash
    fn get_block_by_hash(&self, hash: &Hash) -> Result<Block>;

    /// Add a block to the chain
    fn add_block(&mut self, block: Block) -> Result<()>;

    /// Check if this node is a validator
    fn is_validator(&self) -> bool;

    /// Get validator set
    fn get_validator_set(&self) -> Vec<ValidatorInfo>;
}

/// Data Availability Layer Interface
///
/// Responsible for data storage and distribution
pub trait DataAvailabilityLayer: Send + Sync {
    /// Store data and return its hash
    fn store_data(&self, data: &[u8]) -> Result<Hash>;

    /// Retrieve data by hash
    fn retrieve_data(&self, hash: &Hash) -> Result<Vec<u8>>;

    /// Verify data availability
    fn verify_availability(&self, hash: &Hash) -> bool;

    /// Broadcast data to the network
    fn broadcast_data(&self, hash: &Hash, data: &[u8]) -> Result<()>;

    /// Request data from peers
    fn request_data(&self, hash: &Hash) -> Result<()>;

    /// Get data availability proof
    fn get_availability_proof(&self, hash: &Hash) -> Result<AvailabilityProof>;
}

/// Layer message trait for inter-layer communication
pub trait LayerMessage: Clone + Send + Sync {
    /// Get the message type for routing
    fn message_type(&self) -> String;
}

/// Core layer trait for modular architecture
#[async_trait::async_trait]
pub trait Layer: Clone + Send + Sync {
    /// Configuration type for this layer
    type Config: Clone + Send + Sync;
    /// Message type for this layer
    type Message: LayerMessage;

    /// Start the layer
    async fn start(&mut self) -> anyhow::Result<()>;

    /// Stop the layer
    async fn stop(&mut self) -> anyhow::Result<()>;

    /// Process a message
    async fn process_message(&mut self, message: Self::Message) -> anyhow::Result<()>;

    /// Get the layer type identifier
    fn get_layer_type(&self) -> String;
}

/// Account state information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccountState {
    /// Account balance
    pub balance: u64,
    /// Account nonce
    pub nonce: u64,
    /// Contract code hash (if this is a contract account)
    pub code_hash: Option<Hash>,
    /// Storage root (if this is a contract account)
    pub storage_root: Option<Hash>,
}

/// Settlement challenge information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettlementChallenge {
    /// Challenge ID
    pub challenge_id: Hash,
    /// Challenged batch
    pub batch_id: Hash,
    /// Challenge proof
    pub proof: FraudProof,
    /// Challenger address
    pub challenger: String,
    /// Challenge timestamp
    pub timestamp: u64,
}

/// Result of processing a settlement challenge
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ChallengeResult {
    /// Challenge ID
    pub challenge_id: Hash,
    /// Whether the challenge was successful
    pub successful: bool,
    /// Penalty applied (if any)
    pub penalty: Option<u64>,
    /// Resolution timestamp
    pub timestamp: u64,
}

/// Validator information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorInfo {
    /// Validator address
    pub address: String,
    /// Validator stake
    pub stake: u64,
    /// Validator public key
    pub public_key: Vec<u8>,
    /// Whether the validator is active
    pub active: bool,
}

/// Data availability proof
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AvailabilityProof {
    /// Data hash
    pub data_hash: Hash,
    /// Merkle proof
    pub merkle_proof: Vec<Hash>,
    /// Root hash
    pub root_hash: Hash,
    /// Proof timestamp
    pub timestamp: u64,
}

/// Modular blockchain configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModularConfig {
    /// Execution layer configuration
    pub execution: ExecutionConfig,
    /// Settlement layer configuration
    pub settlement: SettlementConfig,
    /// Consensus layer configuration
    pub consensus: ConsensusConfig,
    /// Data availability layer configuration
    pub data_availability: DataAvailabilityConfig,
}

/// Execution layer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionConfig {
    /// Gas limit per block
    pub gas_limit: u64,
    /// Gas price
    pub gas_price: u64,
    /// WASM engine settings
    pub wasm_config: WasmConfig,
}

/// Settlement layer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettlementConfig {
    /// Challenge period in blocks
    pub challenge_period: u64,
    /// Settlement batch size
    pub batch_size: usize,
    /// Minimum stake for validators
    pub min_validator_stake: u64,
}

/// Consensus layer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusConfig {
    /// Block time in milliseconds
    pub block_time: u64,
    /// Proof of work difficulty
    pub difficulty: usize,
    /// Maximum block size
    pub max_block_size: usize,
}

/// Data availability layer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataAvailabilityConfig {
    /// P2P network configuration
    pub network_config: NetworkConfig,
    /// Data retention period
    pub retention_period: u64,
    /// Maximum data size
    pub max_data_size: usize,
}

/// Network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Listen address
    pub listen_addr: String,
    /// Bootstrap peers
    pub bootstrap_peers: Vec<String>,
    /// Maximum number of peers
    pub max_peers: usize,
}

/// WASM execution configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmConfig {
    /// Maximum memory pages
    pub max_memory_pages: u32,
    /// Maximum stack size
    pub max_stack_size: u32,
    /// Gas metering enabled
    pub gas_metering: bool,
}
