//! Modular blockchain orchestrator
//!
//! This module coordinates the interaction between different layers
//! of the modular blockchain architecture.

use super::consensus::PolyTorusConsensusLayer;
use super::data_availability::PolyTorusDataAvailabilityLayer;
use super::execution::PolyTorusExecutionLayer;
use super::settlement::PolyTorusSettlementLayer;
use super::traits::*;

use crate::blockchain::block::Block;
use crate::blockchain::types::{block_states, network};
use crate::config::DataContext;
use crate::crypto::transaction::Transaction;
use crate::Result;

use std::sync::{Arc, Mutex};
use tokio::sync::{mpsc, Mutex as AsyncMutex};

/// Main modular blockchain orchestrator
pub struct ModularBlockchain {
    /// Execution layer
    execution_layer: Arc<Mutex<PolyTorusExecutionLayer>>,
    /// Settlement layer
    settlement_layer: Arc<PolyTorusSettlementLayer>,
    /// Consensus layer
    consensus_layer: Arc<Mutex<PolyTorusConsensusLayer>>,
    /// Data availability layer
    data_availability_layer: Arc<PolyTorusDataAvailabilityLayer>,
    /// Configuration
    config: ModularConfig,
    /// Event channels for layer communication
    event_tx: mpsc::UnboundedSender<ModularEvent>,
    event_rx: Arc<AsyncMutex<mpsc::UnboundedReceiver<ModularEvent>>>,
}

/// Events for communication between layers
#[derive(Debug, Clone)]
pub enum ModularEvent {
    /// New block proposed
    BlockProposed(Block),
    /// Block validated
    BlockValidated(Block, bool),
    /// Execution completed
    ExecutionCompleted(Hash, ExecutionResult),
    /// Batch ready for settlement
    BatchReady(ExecutionBatch),
    /// Settlement completed
    SettlementCompleted(SettlementResult),
    /// Data stored
    DataStored(Hash, usize),
    /// Challenge submitted
    ChallengeSubmitted(SettlementChallenge),
}

impl ModularBlockchain {
    /// Create a new modular blockchain instance
    pub fn new(config: ModularConfig, data_context: DataContext) -> Result<Self> {
        // Initialize layers
        let execution_layer =
            PolyTorusExecutionLayer::new(data_context.clone(), config.execution.clone())?;

        let consensus_layer = PolyTorusConsensusLayer::new(
            data_context.clone(),
            config.consensus.clone(),
            false, // Not a validator by default
        )?;

        let settlement_layer = PolyTorusSettlementLayer::new(config.settlement.clone())?;

        let data_availability_layer =
            PolyTorusDataAvailabilityLayer::new(config.data_availability.clone())?;

        // Create event channels
        let (event_tx, event_rx) = mpsc::unbounded_channel();

        Ok(Self {
            execution_layer: Arc::new(Mutex::new(execution_layer)),
            settlement_layer: Arc::new(settlement_layer),
            consensus_layer: Arc::new(Mutex::new(consensus_layer)),
            data_availability_layer: Arc::new(data_availability_layer),
            config,
            event_tx,
            event_rx: Arc::new(AsyncMutex::new(event_rx)),
        })
    }

    /// Start the modular blockchain
    pub async fn start(&self) -> Result<()> {
        log::info!("Starting modular blockchain");

        // Start event processing loop
        self.start_event_loop().await?;

        Ok(())
    }

    /// Process a new transaction
    pub async fn process_transaction(
        &self,
        transaction: Transaction,
    ) -> Result<TransactionReceipt> {
        log::debug!("Processing transaction: {}", transaction.id);

        // Execute transaction
        let execution_layer = self.execution_layer.lock().unwrap();
        let receipt = execution_layer.execute_transaction(&transaction)?;

        // Store transaction data
        let tx_data = bincode::serialize(&transaction)?;
        let data_hash = self.data_availability_layer.store_data(&tx_data)?;

        // Send event
        let _ = self
            .event_tx
            .send(ModularEvent::DataStored(data_hash, tx_data.len()));

        Ok(receipt)
    }

    /// Mine a new block
    pub async fn mine_block(&self, transactions: Vec<Transaction>) -> Result<Block> {
        log::info!("Mining new block with {} transactions", transactions.len());

        // Create block through consensus layer
        let mut consensus_layer = self.consensus_layer.lock().unwrap();
        let current_height = consensus_layer.get_block_height()?;

        log::debug!("Current blockchain height: {}", current_height);

        let height = current_height
            .checked_add(1)
            .ok_or_else(|| failure::format_err!("Block height overflow"))?;

        // Get previous block hash
        let canonical_chain = consensus_layer.get_canonical_chain();
        log::debug!("Canonical chain length: {}", canonical_chain.len());

        let prev_hash = if canonical_chain.is_empty() {
            // No blocks in chain yet - this shouldn't happen if genesis was created
            log::warn!("No blocks found in canonical chain");
            String::new()
        } else {
            let hash = canonical_chain.last().cloned().unwrap_or_default();
            log::debug!("Previous block hash: {}", hash);
            hash
        };

        log::debug!(
            "Creating block with height: {}, prev_hash: {}",
            height,
            prev_hash
        );        // Create new block
        let building_block = Block::<block_states::Building, network::Mainnet>::new_building(
            transactions.clone(),
            prev_hash,
            height as i32,
            self.config.consensus.difficulty,
        );// Mine the block
        let mined_block = building_block.mine()?;
        
        // Validate the mined block
        let validated_block = mined_block.validate()?;
        
        // Finalize the validated block
        let block = validated_block.finalize();

        // Validate block with consensus layer
        let is_valid = consensus_layer.validate_block(&block);
        if !is_valid {
            return Err(failure::format_err!("Block validation failed"));
        }

        // Execute block
        let execution_layer = self.execution_layer.lock().unwrap();
        let execution_result = execution_layer.execute_block(&block)?;

        // Store block data
        let block_data = bincode::serialize(&block)?;
        let _block_hash = self.data_availability_layer.store_data(&block_data)?;

        // Add block to consensus layer
        consensus_layer.add_block(block.clone())?;        // Create execution batch for settlement
        let batch = ExecutionBatch {
            batch_id: block.get_hash().to_string(),
            transactions,
            results: vec![execution_result.clone()],
            prev_state_root: execution_layer.get_state_root(),
            new_state_root: execution_result.state_root.clone(),
        };

        // Send events
        let _ = self
            .event_tx
            .send(ModularEvent::BlockProposed(block.clone()));
        let _ = self.event_tx.send(ModularEvent::ExecutionCompleted(
            block.get_hash().to_string(),
            execution_result,
        ));
        let _ = self.event_tx.send(ModularEvent::BatchReady(batch));

        Ok(block)
    }

    /// Submit a settlement challenge
    pub async fn submit_challenge(
        &self,
        challenge: SettlementChallenge,
    ) -> Result<ChallengeResult> {
        log::info!(
            "Processing settlement challenge: {}",
            challenge.challenge_id
        );

        let result = self.settlement_layer.process_challenge(&challenge)?;

        // Send event
        let _ = self
            .event_tx
            .send(ModularEvent::ChallengeSubmitted(challenge));

        Ok(result)
    }

    /// Get blockchain state information
    pub fn get_state_info(&self) -> Result<StateInfo> {
        let execution_layer = self.execution_layer.lock().unwrap();
        let consensus_layer = self.consensus_layer.lock().unwrap();

        Ok(StateInfo {
            execution_state_root: execution_layer.get_state_root(),
            settlement_root: self.settlement_layer.get_settlement_root(),
            block_height: consensus_layer.get_block_height()?,
            canonical_chain_length: consensus_layer.get_canonical_chain().len(),
        })
    }

    /// Start the event processing loop
    async fn start_event_loop(&self) -> Result<()> {
        // Use the existing event receiver
        let event_rx = self.event_rx.clone();
        let settlement_layer = self.settlement_layer.clone();

        tokio::spawn(async move {
            loop {
                let event = {
                    let mut receiver = event_rx.lock().await;
                    receiver.recv().await
                };

                match event {
                    Some(ModularEvent::BatchReady(batch)) => {
                        log::debug!("Processing batch for settlement: {}", batch.batch_id);

                        if let Err(e) = settlement_layer.settle_batch(&batch) {
                            log::error!("Failed to settle batch: {}", e);
                        }
                    }
                    Some(ModularEvent::BlockProposed(block)) => {
                        log::debug!("Block proposed: {}", block.get_hash());
                    }
                    Some(ModularEvent::ExecutionCompleted(hash, result)) => {
                        log::debug!(
                            "Execution completed for {}: gas_used={}",
                            hash,
                            result.gas_used
                        );
                    }
                    Some(ModularEvent::SettlementCompleted(result)) => {
                        log::debug!("Settlement completed: {}", result.settlement_root);
                    }
                    Some(ModularEvent::DataStored(hash, size)) => {
                        log::debug!("Data stored: {} ({} bytes)", hash, size);
                    }
                    Some(ModularEvent::ChallengeSubmitted(challenge)) => {
                        log::debug!("Challenge submitted: {}", challenge.challenge_id);
                    }
                    Some(ModularEvent::BlockValidated(block, is_valid)) => {
                        log::debug!("Block validated: {} - {}", block.get_hash(), is_valid);
                    }
                    None => {
                        log::info!("Event channel closed, stopping event loop");
                        break;
                    }
                }
            }
        });

        Ok(())
    }

    /// Get configuration
    pub fn get_config(&self) -> &ModularConfig {
        &self.config
    }
}

/// Blockchain state information
#[derive(Debug, Clone)]
pub struct StateInfo {
    /// Current execution state root
    pub execution_state_root: Hash,
    /// Current settlement root
    pub settlement_root: Hash,
    /// Current block height
    pub block_height: u64,
    /// Length of canonical chain
    pub canonical_chain_length: usize,
}

/// Builder for modular blockchain
pub struct ModularBlockchainBuilder {
    config: Option<ModularConfig>,
    data_context: Option<DataContext>,
}

impl ModularBlockchainBuilder {
    pub fn new() -> Self {
        Self {
            config: None,
            data_context: None,
        }
    }

    pub fn with_config(mut self, config: ModularConfig) -> Self {
        self.config = Some(config);
        self
    }

    pub fn with_data_context(mut self, data_context: DataContext) -> Self {
        self.data_context = Some(data_context);
        self
    }

    pub fn build(self) -> Result<ModularBlockchain> {
        let config = self.config.unwrap_or_else(|| ModularConfig {
            execution: ExecutionConfig {
                gas_limit: 8_000_000,
                gas_price: 1,
                wasm_config: WasmConfig {
                    max_memory_pages: 256,
                    max_stack_size: 65536,
                    gas_metering: true,
                },
            },
            settlement: SettlementConfig {
                challenge_period: 100,
                batch_size: 100,
                min_validator_stake: 1000,
            },
            consensus: ConsensusConfig {
                block_time: 10000,
                difficulty: 4,
                max_block_size: 1024 * 1024,
            },
            data_availability: DataAvailabilityConfig {
                network_config: NetworkConfig {
                    listen_addr: "0.0.0.0:7000".to_string(),
                    bootstrap_peers: Vec::new(),
                    max_peers: 50,
                },
                retention_period: 86400 * 7,
                max_data_size: 1024 * 1024,
            },
        });

        let data_context = self.data_context.unwrap_or_default();

        ModularBlockchain::new(config, data_context)
    }
}

impl Default for ModularBlockchainBuilder {
    fn default() -> Self {
        Self::new()
    }
}
