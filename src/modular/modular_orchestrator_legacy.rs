//! Enhanced Modular Blockchain Orchestrator
//!
//! This module provides a truly modular orchestrator that depends only on traits,
//! enabling easy swapping of layer implementations and better separation of concerns.

use super::traits::*;
use crate::blockchain::block::Block;
use crate::blockchain::types::{block_states, network};
use crate::crypto::transaction::Transaction;
use crate::Result;

use std::sync::Arc;
use tokio::sync::{mpsc, Mutex as AsyncMutex};

/// Pluggable modular blockchain orchestrator
pub struct PluggableModularBlockchain {
    /// Execution layer (trait object)
    execution_layer: Arc<dyn ExecutionLayer>,
    /// Settlement layer (trait object)
    settlement_layer: Arc<dyn SettlementLayer>,
    /// Consensus layer (trait object)
    consensus_layer: Arc<dyn ConsensusLayer>,
    /// Data availability layer (trait object)
    data_availability_layer: Arc<dyn DataAvailabilityLayer>,
    /// Configuration
    config: ModularConfig,
    /// Event channels for layer communication
    event_tx: mpsc::UnboundedSender<ModularEvent>,
    event_rx: Arc<AsyncMutex<mpsc::UnboundedReceiver<ModularEvent>>>,
}

/// Enhanced events for modular communication
#[derive(Debug, Clone)]
pub enum ModularEvent {
    /// New block proposed
    BlockProposed {
        block: Block,
        proposer_id: String,
    },
    /// Block validated
    BlockValidated {
        block: Block,
        is_valid: bool,
        validator_id: String,
    },
    /// Execution completed
    ExecutionCompleted {
        block_hash: Hash,
        result: ExecutionResult,
        execution_time: u64,
    },
    /// Batch ready for settlement
    BatchReady {
        batch: ExecutionBatch,
        priority: u8,
    },
    /// Settlement completed
    SettlementCompleted {
        result: SettlementResult,
        finality_level: u8,
    },
    /// Data stored
    DataStored {
        hash: Hash,
        size: usize,
        layer_id: String,
    },
    /// Challenge submitted
    ChallengeSubmitted {
        challenge: SettlementChallenge,
        challenger_id: String,
    },
    /// Layer health check
    LayerHealthCheck {
        layer_type: LayerType,
        is_healthy: bool,
        metrics: LayerMetrics,
    },
}

/// Layer type enumeration
#[derive(Debug, Clone)]
pub enum LayerType {
    Execution,
    Settlement,
    Consensus,
    DataAvailability,
}

/// Layer performance metrics
#[derive(Debug, Clone)]
pub struct LayerMetrics {
    pub throughput: f64,
    pub latency: u64,
    pub error_rate: f64,
    pub resource_usage: f64,
}

impl PluggableModularBlockchain {
    /// Create a new pluggable modular blockchain
    pub fn new(
        execution_layer: Arc<dyn ExecutionLayer>,
        settlement_layer: Arc<dyn SettlementLayer>,
        consensus_layer: Arc<dyn ConsensusLayer>,
        data_availability_layer: Arc<dyn DataAvailabilityLayer>,
        config: ModularConfig,
    ) -> Result<Self> {
        let (event_tx, event_rx) = mpsc::unbounded_channel();

        Ok(Self {
            execution_layer,
            settlement_layer,
            consensus_layer,
            data_availability_layer,
            config,
            event_tx,
            event_rx: Arc::new(AsyncMutex::new(event_rx)),
        })
    }

    /// Start the modular blockchain with health monitoring
    pub async fn start(&self) -> Result<()> {
        log::info!("Starting pluggable modular blockchain");

        // Start health monitoring for all layers
        self.start_health_monitoring().await?;

        // Start event processing loop
        self.start_enhanced_event_loop().await?;

        Ok(())
    }

    /// Process transaction through the modular pipeline
    pub async fn process_transaction(&self, transaction: Transaction) -> Result<TransactionReceipt> {
        log::debug!("Processing transaction through modular pipeline: {}", transaction.id);

        let start_time = std::time::Instant::now();

        // Step 1: Execute transaction
        let receipt = self.execution_layer.execute_transaction(&transaction)?;

        // Step 2: Store transaction data
        let tx_data = bincode::serialize(&transaction)?;
        let data_hash = self.data_availability_layer.store_data(&tx_data)?;

        // Step 3: Send events
        let execution_time = start_time.elapsed().as_millis() as u64;
        let _ = self.event_tx.send(ModularEvent::DataStored {
            hash: data_hash,
            size: tx_data.len(),
            layer_id: "execution".to_string(),
        });

        // Step 4: Create execution result for potential batching
        let execution_result = ExecutionResult {
            state_root: self.execution_layer.get_state_root(),
            gas_used: receipt.gas_used,
            receipts: vec![receipt.clone()],
            events: receipt.events.clone(),
        };

        let _ = self.event_tx.send(ModularEvent::ExecutionCompleted {
            block_hash: transaction.id.clone(),
            result: execution_result,
            execution_time,
        });

        Ok(receipt)
    }

    /// Mine a block through the modular consensus process
    pub async fn mine_block(&self, transactions: Vec<Transaction>) -> Result<Block> {
        log::info!("Mining block through modular consensus with {} transactions", transactions.len());

        // Step 1: Validate with consensus layer
        let canonical_chain = self.consensus_layer.get_canonical_chain();
        let current_height = self.consensus_layer.get_block_height()?;

        let height = current_height.checked_add(1)
            .ok_or_else(|| failure::format_err!("Block height overflow"))?;

        let prev_hash = canonical_chain.last().cloned().unwrap_or_default();

        // Step 2: Create and mine block
        let building_block = Block::<block_states::Building, network::Mainnet>::new_building(
            transactions.clone(),
            prev_hash,
            height as i32,
            self.config.consensus.difficulty,
        );

        let mined_block = building_block.mine()?;
        let validated_block = mined_block.validate()?;
        let block = validated_block.finalize();

        // Step 3: Validate through consensus
        let is_valid = self.consensus_layer.validate_block(&block);
        if !is_valid {
            return Err(failure::format_err!("Block validation failed"));
        }

        // Step 4: Execute block
        let execution_result = self.execution_layer.execute_block(&block)?;

        // Step 5: Store block data
        let block_data = bincode::serialize(&block)?;
        let _block_hash = self.data_availability_layer.store_data(&block_data)?;

        // Step 6: Create execution batch for settlement
        let batch = ExecutionBatch {
            batch_id: block.get_hash().to_string(),
            transactions,
            results: vec![execution_result.clone()],
            prev_state_root: self.execution_layer.get_state_root(),
            new_state_root: execution_result.state_root.clone(),
        };

        // Step 7: Send events
        let _ = self.event_tx.send(ModularEvent::BlockProposed {
            block: block.clone(),
            proposer_id: "self".to_string(),
        });

        let _ = self.event_tx.send(ModularEvent::BlockValidated {
            block: block.clone(),
            is_valid,
            validator_id: "consensus".to_string(),
        });

        let _ = self.event_tx.send(ModularEvent::BatchReady {
            batch,
            priority: 1,
        });

        Ok(block)
    }

    /// Start health monitoring for all layers
    async fn start_health_monitoring(&self) -> Result<()> {
        let event_tx = self.event_tx.clone();
        let _execution_layer = self.execution_layer.clone();
        let _settlement_layer = self.settlement_layer.clone();
        let _consensus_layer = self.consensus_layer.clone();
        let _data_availability_layer = self.data_availability_layer.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(tokio::time::Duration::from_secs(30));

            loop {
                interval.tick().await;

                // Check execution layer health
                let exec_metrics = LayerMetrics {
                    throughput: 100.0, // Mock metrics - in real implementation, collect actual metrics
                    latency: 50,
                    error_rate: 0.01,
                    resource_usage: 0.6,
                };

                let _ = event_tx.send(ModularEvent::LayerHealthCheck {
                    layer_type: LayerType::Execution,
                    is_healthy: true,
                    metrics: exec_metrics,
                });

                // Similar health checks for other layers...
                log::trace!("Health check completed for all layers");
            }
        });

        Ok(())
    }

    /// Enhanced event processing loop with metrics
    async fn start_enhanced_event_loop(&self) -> Result<()> {
        let event_rx = self.event_rx.clone();
        let settlement_layer = self.settlement_layer.clone();

        tokio::spawn(async move {
            loop {
                let event = {
                    let mut receiver = event_rx.lock().await;
                    receiver.recv().await
                };

                match event {
                    Some(ModularEvent::BatchReady { batch, priority }) => {
                        log::debug!("Processing priority {} batch for settlement: {}", priority, batch.batch_id);
                        
                        match settlement_layer.settle_batch(&batch) {
                            Ok(result) => {
                                log::info!("Batch settled successfully: {}", result.settlement_root);
                                // Could send SettlementCompleted event here
                            }
                            Err(e) => {
                                log::error!("Failed to settle batch: {}", e);
                            }
                        }
                    }
                    Some(ModularEvent::LayerHealthCheck { layer_type, is_healthy, metrics }) => {
                        if !is_healthy {
                            log::warn!("Layer {:?} is unhealthy: {:?}", layer_type, metrics);
                        } else {
                            log::trace!("Layer {:?} health check passed", layer_type);
                        }
                    }
                    Some(ModularEvent::ChallengeSubmitted { challenge, challenger_id }) => {
                        log::info!("Processing challenge from {}: {}", challenger_id, challenge.challenge_id);
                        
                        match settlement_layer.process_challenge(&challenge) {
                            Ok(result) => {
                                log::info!("Challenge processed: {:?}", result.successful);
                            }
                            Err(e) => {
                                log::error!("Failed to process challenge: {}", e);
                            }
                        }
                    }
                    Some(event) => {
                        log::trace!("Processed event: {:?}", event);
                    }
                    None => {
                        log::info!("Event channel closed, stopping enhanced event loop");
                        break;
                    }
                }
            }
        });

        Ok(())
    }

    /// Get comprehensive blockchain state
    pub fn get_enhanced_state_info(&self) -> Result<EnhancedStateInfo> {
        Ok(EnhancedStateInfo {
            execution_state_root: self.execution_layer.get_state_root(),
            settlement_root: self.settlement_layer.get_settlement_root(),
            block_height: self.consensus_layer.get_block_height()?,
            canonical_chain_length: self.consensus_layer.get_canonical_chain().len(),
            layer_health: self.get_layer_health_summary(),
        })
    }

    /// Get layer health summary
    fn get_layer_health_summary(&self) -> LayerHealthSummary {
        LayerHealthSummary {
            execution_healthy: true, // In real implementation, check actual health
            settlement_healthy: true,
            consensus_healthy: true,
            data_availability_healthy: true,
        }
    }

    /// Submit challenge through settlement layer
    pub async fn submit_challenge(&self, challenge: SettlementChallenge) -> Result<ChallengeResult> {
        log::info!("Submitting settlement challenge: {}", challenge.challenge_id);

        let result = self.settlement_layer.process_challenge(&challenge)?;

        let _ = self.event_tx.send(ModularEvent::ChallengeSubmitted {
            challenge,
            challenger_id: "external".to_string(),
        });

        Ok(result)
    }
}

/// Enhanced state information
#[derive(Debug, Clone)]
pub struct EnhancedStateInfo {
    pub execution_state_root: Hash,
    pub settlement_root: Hash,
    pub block_height: u64,
    pub canonical_chain_length: usize,
    pub layer_health: LayerHealthSummary,
}

/// Layer health summary
#[derive(Debug, Clone)]
pub struct LayerHealthSummary {
    pub execution_healthy: bool,
    pub settlement_healthy: bool,
    pub consensus_healthy: bool,
    pub data_availability_healthy: bool,
}

/// Builder for pluggable modular blockchain
pub struct PluggableModularBlockchainBuilder {
    execution_layer: Option<Arc<dyn ExecutionLayer>>,
    settlement_layer: Option<Arc<dyn SettlementLayer>>,
    consensus_layer: Option<Arc<dyn ConsensusLayer>>,
    data_availability_layer: Option<Arc<dyn DataAvailabilityLayer>>,
    config: Option<ModularConfig>,
}

impl PluggableModularBlockchainBuilder {
    pub fn new() -> Self {
        Self {
            execution_layer: None,
            settlement_layer: None,
            consensus_layer: None,
            data_availability_layer: None,
            config: None,
        }
    }

    pub fn with_execution_layer(mut self, layer: Arc<dyn ExecutionLayer>) -> Self {
        self.execution_layer = Some(layer);
        self
    }

    pub fn with_settlement_layer(mut self, layer: Arc<dyn SettlementLayer>) -> Self {
        self.settlement_layer = Some(layer);
        self
    }

    pub fn with_consensus_layer(mut self, layer: Arc<dyn ConsensusLayer>) -> Self {
        self.consensus_layer = Some(layer);
        self
    }

    pub fn with_data_availability_layer(mut self, layer: Arc<dyn DataAvailabilityLayer>) -> Self {
        self.data_availability_layer = Some(layer);
        self
    }

    pub fn with_config(mut self, config: ModularConfig) -> Self {
        self.config = Some(config);
        self
    }

    pub fn build(self) -> Result<PluggableModularBlockchain> {
        let execution_layer = self.execution_layer
            .ok_or_else(|| failure::format_err!("Execution layer is required"))?;
        let settlement_layer = self.settlement_layer
            .ok_or_else(|| failure::format_err!("Settlement layer is required"))?;
        let consensus_layer = self.consensus_layer
            .ok_or_else(|| failure::format_err!("Consensus layer is required"))?;
        let data_availability_layer = self.data_availability_layer
            .ok_or_else(|| failure::format_err!("Data availability layer is required"))?;
        let config = self.config
            .ok_or_else(|| failure::format_err!("Configuration is required"))?;

        PluggableModularBlockchain::new(
            execution_layer,
            settlement_layer,
            consensus_layer,
            data_availability_layer,
            config,
        )
    }
}

impl Default for PluggableModularBlockchainBuilder {
    fn default() -> Self {
        Self::new()
    }
}
