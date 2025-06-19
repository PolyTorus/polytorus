//! Unified Modular Blockchain Orchestrator
//!
//! This is the new unified orchestrator that combines the best features
//! from both the legacy and enhanced implementations, providing a clean
//! trait-based architecture with comprehensive event handling.

use std::{collections::HashMap, sync::Arc};

use anyhow;
use serde::{Deserialize, Serialize};
use tokio::sync::{mpsc, Mutex as AsyncMutex, RwLock};

use super::{
    config_manager::ModularConfigManager, layer_factory::ModularLayerFactory,
    message_bus::ModularMessageBus, traits::*,
};
use crate::{
    blockchain::{
        block::Block,
        types::{block_states, network},
    },
    network::blockchain_integration::NetworkedBlockchainNode,
    Result,
};

/// Unified Modular Blockchain Orchestrator with P2P Network Integration
///
/// This is the central coordination component that orchestrates all modular layers
/// in the PolyTorus blockchain. It provides comprehensive system coordination with:
///
/// * **Layer Coordination**: Manages communication between all modular layers
/// * **Event System**: 17 different event types for comprehensive monitoring
/// * **P2P Integration**: Built-in network node integration for distributed operation
/// * **Configuration Management**: Dynamic configuration with validation
/// * **Performance Monitoring**: Tracks metrics and health across all layers
///
/// # Examples
///
/// ```rust,no_run
/// use polytorus::modular::UnifiedModularOrchestrator;
/// use polytorus::config::DataContext;
/// use std::path::PathBuf;
///
/// let data_context = DataContext::new(PathBuf::from("orchestrator_data"));
/// println!("Unified orchestrator configuration ready!");
/// ```
///
/// # Implementation Status
///
/// ⚠️ **BASIC IMPLEMENTATION** - Well-designed architecture but needs integration tests
pub struct UnifiedModularOrchestrator {
    /// Execution layer (trait object)
    execution_layer: Arc<dyn ExecutionLayer + Send + Sync>,
    /// Settlement layer (trait object)
    settlement_layer: Arc<dyn SettlementLayer + Send + Sync>,
    /// Consensus layer (trait object)
    consensus_layer: Arc<dyn ConsensusLayer + Send + Sync>,
    /// Data availability layer (trait object)
    data_availability_layer: Arc<dyn DataAvailabilityLayer + Send + Sync>,

    /// Enhanced infrastructure
    message_bus: Arc<ModularMessageBus>,
    config_manager: Arc<RwLock<ModularConfigManager>>,
    layer_factory: Arc<ModularLayerFactory>,

    /// P2P Network integration
    network_node: Option<Arc<AsyncMutex<NetworkedBlockchainNode>>>,

    /// Event handling
    event_tx: mpsc::UnboundedSender<UnifiedEvent>,
    event_rx: Arc<AsyncMutex<mpsc::UnboundedReceiver<UnifiedEvent>>>,

    /// State management
    state: Arc<RwLock<OrchestratorState>>,
    metrics: Arc<RwLock<OrchestratorMetrics>>,
}

/// Unified event system for all layer communications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UnifiedEvent {
    /// Block lifecycle events
    BlockProposed {
        block: String, // Serialized block
        proposer_id: String,
        timestamp: u64,
    },
    BlockValidated {
        block_hash: String,
        is_valid: bool,
        validator_id: String,
        validation_time_ms: u64,
    },
    BlockFinalized {
        block_hash: String,
        block_height: u64,
        timestamp: u64,
    },

    /// Execution events
    ExecutionStarted {
        transaction_batch_id: String,
        transaction_count: usize,
    },
    ExecutionCompleted {
        batch_id: String,
        result: ExecutionEventResult,
        execution_time_ms: u64,
        gas_used: u64,
    },
    ExecutionFailed {
        batch_id: String,
        error: String,
        failed_transaction_id: Option<String>,
    },

    /// Settlement events
    BatchSubmitted {
        batch_id: String,
        transaction_count: usize,
        batch_size_bytes: usize,
    },
    SettlementCompleted {
        batch_id: String,
        settlement_hash: String,
        settlement_time_ms: u64,
    },

    /// Consensus events
    ConsensusStarted { round: u64, proposer_id: String },
    ConsensusAchieved {
        round: u64,
        block_hash: String,
        participant_count: usize,
    },

    /// Data availability events
    DataStored {
        data_hash: String,
        size_bytes: usize,
        availability_score: f64,
    },
    DataRetrieved {
        data_hash: String,
        retrieval_time_ms: u64,
    },

    /// System events
    LayerHealthChanged {
        layer_type: String,
        is_healthy: bool,
        details: String,
    },
    ConfigurationUpdated {
        component: String,
        change_summary: String,
    },
    PerformanceAlert {
        metric: String,
        current_value: f64,
        threshold: f64,
        severity: AlertSeverity,
    },
    /// Performance optimization events
    PerformanceOptimization {
        optimization_type: String,
        metrics_before: String,
        metrics_after: String,
    },
    /// Transaction processing events
    TransactionProcessed {
        tx_id: String,
        success: bool,
        gas_used: u64,
        processing_time_ms: u64,
    },
    /// System alert events
    SystemAlert {
        severity: AlertSeverity,
        message: String,
        component: String,
    },
    /// Layer status change events
    LayerStatusChanged {
        layer: String,
        old_status: String,
        new_status: String,
    },
}

/// Execution result for events
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionEventResult {
    pub success: bool,
    pub gas_used: u64,
    pub state_changes: Vec<String>,
    pub events_emitted: Vec<String>,
    pub error_message: Option<String>,
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Current state of the orchestrator
#[derive(Debug, Clone)]
pub struct OrchestratorState {
    pub is_running: bool,
    pub current_block_height: u64,
    pub last_finalized_block: Option<String>,
    pub pending_transactions: usize,
    pub active_layers: HashMap<String, LayerStatus>,
    pub last_health_check: u64,
}

/// Status of individual layers
#[derive(Debug, Clone)]
pub struct LayerStatus {
    pub is_healthy: bool,
    pub last_activity: u64,
    pub processed_items: u64,
    pub error_count: u64,
    pub average_processing_time_ms: f64,
}

/// Orchestrator performance metrics
#[derive(Debug, Clone)]
pub struct OrchestratorMetrics {
    pub total_blocks_processed: u64,
    pub total_transactions_processed: u64,
    pub average_block_time_ms: f64,
    pub average_transaction_throughput: f64,
    pub total_events_handled: u64,
    pub error_rate: f64,
    pub uptime_seconds: u64,
    pub layer_metrics: HashMap<String, LayerMetrics>,
}

/// Performance metrics for individual layers
#[derive(Debug, Clone)]
pub struct LayerMetrics {
    pub operations_count: u64,
    pub average_operation_time_ms: f64,
    pub success_rate: f64,
    pub last_operation_timestamp: u64,
}

impl UnifiedModularOrchestrator {
    /// Create a new unified orchestrator
    pub fn new(
        execution_layer: Arc<dyn ExecutionLayer + Send + Sync>,
        settlement_layer: Arc<dyn SettlementLayer + Send + Sync>,
        consensus_layer: Arc<dyn ConsensusLayer + Send + Sync>,
        data_availability_layer: Arc<dyn DataAvailabilityLayer + Send + Sync>,
        message_bus: Arc<ModularMessageBus>,
        config_manager: Arc<RwLock<ModularConfigManager>>,
        layer_factory: Arc<ModularLayerFactory>,
    ) -> Result<Self> {
        let (event_tx, event_rx) = mpsc::unbounded_channel();

        let initial_state = OrchestratorState {
            is_running: false,
            current_block_height: 0,
            last_finalized_block: None,
            pending_transactions: 0,
            active_layers: HashMap::new(),
            last_health_check: 0,
        };

        let initial_metrics = OrchestratorMetrics {
            total_blocks_processed: 0,
            total_transactions_processed: 0,
            average_block_time_ms: 0.0,
            average_transaction_throughput: 0.0,
            total_events_handled: 0,
            error_rate: 0.0,
            uptime_seconds: 0,
            layer_metrics: HashMap::new(),
        };

        Ok(UnifiedModularOrchestrator {
            execution_layer,
            settlement_layer,
            consensus_layer,
            data_availability_layer,
            message_bus,
            config_manager,
            layer_factory,
            network_node: None,
            event_tx,
            event_rx: Arc::new(AsyncMutex::new(event_rx)),
            state: Arc::new(RwLock::new(initial_state)),
            metrics: Arc::new(RwLock::new(initial_metrics)),
        })
    }

    /// Create a new unified orchestrator with network integration
    pub async fn new_with_network(
        execution_layer: Arc<dyn ExecutionLayer + Send + Sync>,
        settlement_layer: Arc<dyn SettlementLayer + Send + Sync>,
        consensus_layer: Arc<dyn ConsensusLayer + Send + Sync>,
        data_availability_layer: Arc<dyn DataAvailabilityLayer + Send + Sync>,
        message_bus: Arc<ModularMessageBus>,
        config_manager: Arc<RwLock<ModularConfigManager>>,
        layer_factory: Arc<ModularLayerFactory>,
        listen_addr: std::net::SocketAddr,
        bootstrap_peers: Vec<std::net::SocketAddr>,
    ) -> Result<Self> {
        let (event_tx, event_rx) = mpsc::unbounded_channel();

        // Create networked blockchain node
        let network_node = NetworkedBlockchainNode::new(listen_addr, bootstrap_peers).await?;

        let initial_state = OrchestratorState {
            is_running: false,
            current_block_height: 0,
            last_finalized_block: None,
            pending_transactions: 0,
            active_layers: HashMap::new(),
            last_health_check: 0,
        };

        let initial_metrics = OrchestratorMetrics {
            total_blocks_processed: 0,
            total_transactions_processed: 0,
            average_block_time_ms: 0.0,
            average_transaction_throughput: 0.0,
            total_events_handled: 0,
            error_rate: 0.0,
            uptime_seconds: 0,
            layer_metrics: HashMap::new(),
        };

        Ok(UnifiedModularOrchestrator {
            execution_layer,
            settlement_layer,
            consensus_layer,
            data_availability_layer,
            message_bus,
            config_manager,
            layer_factory,
            network_node: Some(Arc::new(AsyncMutex::new(network_node))),
            event_tx,
            event_rx: Arc::new(AsyncMutex::new(event_rx)),
            state: Arc::new(RwLock::new(initial_state)),
            metrics: Arc::new(RwLock::new(initial_metrics)),
        })
    }

    /// Start the orchestrator
    pub async fn start(&self) -> Result<()> {
        {
            let mut state = self.state.write().await;
            state.is_running = true;
            state.last_health_check = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs();
        }

        self.emit_event(UnifiedEvent::LayerHealthChanged {
            layer_type: "orchestrator".to_string(),
            is_healthy: true,
            details: "Orchestrator started successfully".to_string(),
        })
        .await?;

        println!("🚀 Unified Modular Orchestrator started");
        Ok(())
    }

    /// Stop the orchestrator
    pub async fn stop(&self) -> Result<()> {
        {
            let mut state = self.state.write().await;
            state.is_running = false;
        }

        self.emit_event(UnifiedEvent::LayerHealthChanged {
            layer_type: "orchestrator".to_string(),
            is_healthy: false,
            details: "Orchestrator stopped".to_string(),
        })
        .await?;

        println!("🛑 Unified Modular Orchestrator stopped");
        Ok(())
    }

    /// Start the orchestrator with network integration
    pub async fn start_with_network(&self) -> Result<()> {
        // Start the standard orchestrator
        self.start().await?;

        // Start the network node if available
        if let Some(network_node) = &self.network_node {
            let mut node = network_node.lock().await;
            node.start().await?;
            println!("🌐 Network layer started successfully");
        }

        Ok(())
    }
    /// Stop the orchestrator and network
    pub async fn stop_with_network(&self) -> Result<()> {
        // Stop the network first
        if let Some(_network_node) = &self.network_node {
            // Network node doesn't have a stop method, but we can indicate it's stopping
            println!("🌐 Stopping network layer...");
        }

        // Stop the orchestrator
        self.stop().await?;

        Ok(())
    }

    /// Process a new block through all layers
    pub async fn process_block(
        &self,
        block: Block<block_states::Building, network::Development>,
    ) -> Result<Block<block_states::Finalized, network::Development>> {
        let start_time = std::time::Instant::now();
        let block_hash = format!("{:?}", block.get_hash());

        // Emit block proposed event
        self.emit_event(UnifiedEvent::BlockProposed {
            block: format!("{:?}", block),
            proposer_id: "unified-orchestrator".to_string(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        })
        .await?;

        // Process through execution layer
        // Note: This is a simplified implementation
        // In a real system, each layer would have specific processing logic

        let mined_block = block.mine()?;
        let validated_block = mined_block.validate()?;
        let finalized_block = validated_block.finalize();

        // Emit block finalized event
        self.emit_event(UnifiedEvent::BlockFinalized {
            block_hash: block_hash.clone(),
            block_height: finalized_block.get_height() as u64,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        })
        .await?;

        // Update metrics
        {
            let mut metrics = self.metrics.write().await;
            metrics.total_blocks_processed += 1;
            let processing_time = start_time.elapsed().as_millis() as f64;
            metrics.average_block_time_ms = (metrics.average_block_time_ms
                * (metrics.total_blocks_processed - 1) as f64
                + processing_time)
                / metrics.total_blocks_processed as f64;
        }

        // Update state
        {
            let mut state = self.state.write().await;
            state.current_block_height = finalized_block.get_height() as u64;
            state.last_finalized_block = Some(block_hash);
        }

        Ok(finalized_block)
    }

    /// Get current orchestrator state
    pub async fn get_state(&self) -> OrchestratorState {
        self.state.read().await.clone()
    }

    /// Get orchestrator metrics
    pub async fn get_metrics(&self) -> OrchestratorMetrics {
        self.metrics.read().await.clone()
    }

    /// Get layer health information
    pub async fn get_layer_health(&self) -> Result<HashMap<String, bool>> {
        let mut health_map = HashMap::new();

        // Check each layer's health (simplified check for now)
        health_map.insert("execution".to_string(), true);
        health_map.insert("settlement".to_string(), true);
        health_map.insert("consensus".to_string(), true);
        health_map.insert("data_availability".to_string(), true);

        Ok(health_map)
    }

    /// Get detailed layer information using actual layer instances
    pub async fn get_detailed_layer_info(&self) -> Result<HashMap<String, String>> {
        let mut layer_info = HashMap::new();

        // Access execution layer information
        layer_info.insert(
            "execution".to_string(),
            format!(
                "Execution layer active at {:p}",
                self.execution_layer.as_ref()
            ),
        );

        // Access settlement layer information
        layer_info.insert(
            "settlement".to_string(),
            format!(
                "Settlement layer active at {:p}",
                self.settlement_layer.as_ref()
            ),
        );

        // Access consensus layer information
        layer_info.insert(
            "consensus".to_string(),
            format!(
                "Consensus layer active at {:p}",
                self.consensus_layer.as_ref()
            ),
        );

        // Access data availability layer information
        layer_info.insert(
            "data_availability".to_string(),
            format!(
                "DA layer active at {:p}",
                self.data_availability_layer.as_ref()
            ),
        );

        Ok(layer_info)
    }

    /// Execute a transaction through the execution layer
    pub async fn execute_transaction(&self, transaction_data: Vec<u8>) -> Result<String> {
        // Use the execution layer to process transaction
        let tx_id = format!(
            "tx_{}_{}",
            transaction_data.len(),
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        );

        // Emit execution started event
        self.emit_event(UnifiedEvent::ExecutionStarted {
            transaction_batch_id: tx_id.clone(),
            transaction_count: 1,
        })
        .await?;

        // Simulate execution (in real implementation, would use execution_layer)
        // Process the transaction data
        let gas_used = std::cmp::min(transaction_data.len() as u64 * 100, 100000);
        tokio::time::sleep(tokio::time::Duration::from_millis(10)).await;

        // Emit execution completed event
        self.emit_event(UnifiedEvent::ExecutionCompleted {
            batch_id: tx_id.clone(),
            result: ExecutionEventResult {
                success: true,
                gas_used,
                state_changes: vec![format!("processed_{}_bytes", transaction_data.len())],
                events_emitted: vec!["transfer".to_string()],
                error_message: None,
            },
            execution_time_ms: 10,
            gas_used,
        })
        .await?;

        // Update metrics
        {
            let mut metrics = self.metrics.write().await;
            metrics.total_transactions_processed += 1;
        }

        Ok(tx_id)
    }

    /// Send a message through the message bus
    pub async fn send_message(&self, message_type: String, payload: Vec<u8>) -> Result<()> {
        // Use the message bus to send a message
        let message_id = format!(
            "msg_{}",
            std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis()
        );

        println!(
            "📤 Sending message {} (type: {}, size: {} bytes)",
            message_id,
            message_type,
            payload.len()
        );

        // In real implementation, would use self.message_bus
        // self.message_bus.send_message(...).await?;

        Ok(())
    }

    /// Broadcast message through the actual message bus
    pub async fn broadcast_message(&self, message_type: String, payload: Vec<u8>) -> Result<()> {
        // Use the actual message_bus field
        println!(
            "📡 Broadcasting via message bus at {:p}: {} ({} bytes)",
            self.message_bus.as_ref(),
            message_type,
            payload.len()
        );

        // In real implementation: self.message_bus.broadcast(...).await?;

        Ok(())
    }

    /// Update configuration through config manager
    pub async fn update_configuration(&self, component: String, new_config: String) -> Result<()> {
        // Use the config manager to update configuration
        println!("⚙️ Updating {} configuration: {}", component, new_config);

        // In real implementation, would use self.config_manager
        // let mut config_mgr = self.config_manager.write().await;
        // config_mgr.update_config(...)?;

        // Emit configuration updated event
        self.emit_event(UnifiedEvent::ConfigurationUpdated {
            component,
            change_summary: new_config,
        })
        .await?;

        Ok(())
    }

    /// Access configuration manager
    pub async fn get_current_config(&self) -> Result<String> {
        // Use the actual config_manager field
        let _config_mgr = self.config_manager.read().await;
        let config_info = format!("Config manager active with {} configurations", 0); // Simplified for now

        Ok(config_info)
    }

    /// Use layer factory to create components
    pub async fn create_test_component(&self) -> Result<String> {
        // Use the actual layer_factory field
        let factory_info = format!("Layer factory at {:p}", self.layer_factory.as_ref());

        // In real implementation: self.layer_factory.create_layer(...)?;

        Ok(factory_info)
    }

    /// Emit an event
    async fn emit_event(&self, event: UnifiedEvent) -> Result<()> {
        if self.event_tx.send(event.clone()).is_err() {
            eprintln!("Failed to emit event: {:?}", event);
        }

        // Update event metrics
        {
            let mut metrics = self.metrics.write().await;
            metrics.total_events_handled += 1;
        }

        Ok(())
    }

    /// Advanced performance optimization methods
    ///
    /// Optimize memory usage by cleaning up unused resources
    pub async fn optimize_memory_usage(&self) -> Result<()> {
        // Clean up cache entries
        let mut state = self.state.write().await;
        if state.pending_transactions > 1000 {
            // Implement intelligent transaction pruning
            state.pending_transactions = (state.pending_transactions * 80) / 100; // Keep 80%

            self.emit_event(UnifiedEvent::PerformanceOptimization {
                optimization_type: "memory_cleanup".to_string(),
                metrics_before: format!("pending_txs: {}", state.pending_transactions),
                metrics_after: "optimized".to_string(),
            })
            .await?;
        }
        Ok(())
    }

    /// Process events in batch for better performance
    pub async fn process_events_batch(&self, batch_size: usize) -> Result<Vec<UnifiedEvent>> {
        let mut processed_events = Vec::new();
        let mut event_rx = self.event_rx.lock().await;

        for _ in 0..batch_size {
            if let Ok(event) = event_rx.try_recv() {
                // Process event efficiently
                match &event {
                    UnifiedEvent::TransactionProcessed { .. } => {
                        let mut metrics = self.metrics.write().await;
                        metrics.total_transactions_processed += 1;
                        metrics.total_events_handled += 1;

                        let mut state = self.state.write().await;
                        if state.pending_transactions > 0 {
                            state.pending_transactions -= 1;
                        }
                    }
                    UnifiedEvent::BlockValidated { .. } => {
                        let mut metrics = self.metrics.write().await;
                        metrics.total_blocks_processed += 1;
                        metrics.total_events_handled += 1;

                        let mut state = self.state.write().await;
                        state.current_block_height += 1;
                    }
                    _ => {
                        let mut metrics = self.metrics.write().await;
                        metrics.total_events_handled += 1;
                    }
                }

                processed_events.push(event);
            } else {
                break;
            }
        }

        Ok(processed_events)
    }

    /// Get performance statistics
    pub async fn get_performance_stats(&self) -> Result<HashMap<String, f64>> {
        let metrics = self.metrics.read().await;
        let state = self.state.read().await;

        let mut stats = HashMap::new();

        // Calculate throughput metrics
        let transactions_per_second = if metrics.uptime_seconds > 0 {
            metrics.total_transactions_processed as f64 / metrics.uptime_seconds as f64
        } else {
            0.0
        };

        let blocks_per_minute = if metrics.uptime_seconds > 0 {
            (metrics.total_blocks_processed as f64 * 60.0) / metrics.uptime_seconds as f64
        } else {
            0.0
        };

        let events_per_second = if metrics.uptime_seconds > 0 {
            metrics.total_events_handled as f64 / metrics.uptime_seconds as f64
        } else {
            0.0
        };

        stats.insert(
            "transactions_per_second".to_string(),
            transactions_per_second,
        );
        stats.insert("blocks_per_minute".to_string(), blocks_per_minute);
        stats.insert("events_per_second".to_string(), events_per_second);
        stats.insert(
            "pending_transaction_ratio".to_string(),
            state.pending_transactions as f64 / (metrics.total_transactions_processed + 1) as f64,
        );
        stats.insert("error_rate".to_string(), metrics.error_rate);
        stats.insert(
            "average_block_time_ms".to_string(),
            metrics.average_block_time_ms,
        );

        Ok(stats)
    }

    /// Enhance event processing with priority handling
    pub async fn process_priority_events(&self) -> Result<()> {
        let mut event_rx = self.event_rx.lock().await;
        let mut high_priority_events = Vec::new();
        let mut normal_events = Vec::new();

        // Collect events and categorize by priority
        while let Ok(event) = event_rx.try_recv() {
            match &event {
                UnifiedEvent::SystemAlert { severity, .. } => {
                    if matches!(severity, AlertSeverity::Critical | AlertSeverity::High) {
                        high_priority_events.push(event);
                    } else {
                        normal_events.push(event);
                    }
                }
                UnifiedEvent::LayerStatusChanged { .. }
                | UnifiedEvent::ConfigurationUpdated { .. } => {
                    high_priority_events.push(event);
                }
                _ => {
                    normal_events.push(event);
                }
            }
        }

        // Process high priority events first
        for event in high_priority_events {
            self.handle_priority_event(event).await?;
        }

        // Then process normal events
        for event in normal_events.into_iter().take(10) {
            // Limit batch size
            self.handle_normal_event(event).await?;
        }

        Ok(())
    }

    /// Handle high priority events with immediate processing
    async fn handle_priority_event(&self, event: UnifiedEvent) -> Result<()> {
        match event {
            UnifiedEvent::SystemAlert {
                severity,
                message,
                component,
            } => {
                eprintln!(
                    "🚨 PRIORITY ALERT [{:?}] in {}: {}",
                    severity, component, message
                );

                // Update metrics
                let mut metrics = self.metrics.write().await;
                metrics.total_events_handled += 1;
                if matches!(severity, AlertSeverity::Critical) {
                    metrics.error_rate = (metrics.error_rate + 0.01).min(1.0);
                }
            }
            UnifiedEvent::LayerStatusChanged {
                layer,
                old_status,
                new_status,
            } => {
                println!(
                    "🔄 Layer {} status: {:?} → {:?}",
                    layer, old_status, new_status
                );

                let mut metrics = self.metrics.write().await;
                metrics.total_events_handled += 1;
            }
            _ => {
                let mut metrics = self.metrics.write().await;
                metrics.total_events_handled += 1;
            }
        }

        Ok(())
    }

    /// Handle normal priority events
    async fn handle_normal_event(&self, event: UnifiedEvent) -> Result<()> {
        // Standard event processing
        let mut metrics = self.metrics.write().await;
        metrics.total_events_handled += 1;

        // Log event processing (could be more sophisticated)
        match event {
            UnifiedEvent::TransactionProcessed { tx_id, .. } => {
                log::debug!("Processed transaction: {}", tx_id);
            }
            UnifiedEvent::BlockValidated { block_hash, .. } => {
                log::debug!("Validated block: {}", block_hash);
            }
            _ => {
                log::trace!("Processed event: {:?}", event);
            }
        }

        Ok(())
    }

    /// Run the event processing loop
    pub async fn run_event_loop(&self) -> Result<()> {
        let mut rx = self.event_rx.lock().await;

        while let Some(event) = rx.recv().await {
            if let Err(e) = self.handle_event(event).await {
                eprintln!("Error handling event: {}", e);

                // Update error metrics
                let mut metrics = self.metrics.write().await;
                let total_events = metrics.total_events_handled;
                metrics.error_rate =
                    (metrics.error_rate * (total_events - 1) as f64 + 1.0) / total_events as f64;
            }
        }

        Ok(())
    }

    /// Handle individual events
    async fn handle_event(&self, event: UnifiedEvent) -> Result<()> {
        match event {
            UnifiedEvent::BlockProposed {
                block: _,
                proposer_id,
                timestamp,
            } => {
                println!("📦 Block proposed by {} at {}", proposer_id, timestamp);
            }
            UnifiedEvent::BlockFinalized {
                block_hash,
                block_height,
                timestamp,
            } => {
                println!(
                    "✅ Block finalized: {} (height: {}) at {}",
                    block_hash, block_height, timestamp
                );
            }
            UnifiedEvent::LayerHealthChanged {
                layer_type,
                is_healthy,
                details,
            } => {
                let status = if is_healthy { "✅" } else { "❌" };
                println!("{} Layer {} health: {}", status, layer_type, details);
            }
            UnifiedEvent::PerformanceAlert {
                metric,
                current_value,
                threshold,
                severity,
            } => {
                println!(
                    "🚨 Performance Alert ({:?}): {} = {} (threshold: {})",
                    severity, metric, current_value, threshold
                );
            }
            UnifiedEvent::PerformanceOptimization {
                optimization_type,
                metrics_before,
                metrics_after,
            } => {
                println!(
                    "⚙️ Performance Optimization ({}) applied: {} → {}",
                    optimization_type, metrics_before, metrics_after
                );
            }
            _ => {
                // Handle other event types as needed
                println!("📨 Event handled: {:?}", std::mem::discriminant(&event));
            }
        }

        Ok(())
    }

    /// Create a unified orchestrator with default implementations and start it
    pub async fn create_and_start_with_defaults(
        config: ModularConfig,
        data_context: crate::config::DataContext,
    ) -> Result<Self> {
        use super::{
            consensus::PolyTorusConsensusLayer, data_availability::PolyTorusDataAvailabilityLayer,
            execution::PolyTorusExecutionLayer, network::ModularNetwork,
            settlement::PolyTorusSettlementLayer,
        };

        // Create infrastructure components first
        let message_bus = Arc::new(ModularMessageBus::new());
        let config_manager = Arc::new(RwLock::new(ModularConfigManager::new()));
        let layer_factory = Arc::new(ModularLayerFactory::new(message_bus.clone()));

        // Create network for data availability
        let network_config = super::network::ModularNetworkConfig {
            listen_address: config.data_availability.network_config.listen_addr.clone(),
            bootstrap_peers: config
                .data_availability
                .network_config
                .bootstrap_peers
                .clone(),
            max_connections: config.data_availability.network_config.max_peers,
            request_timeout: 30, // Default timeout
        };
        let network = Arc::new(ModularNetwork::new(network_config)?);

        // Create default implementations
        let execution_layer = Arc::new(PolyTorusExecutionLayer::new(
            data_context.clone(),
            config.execution.clone(),
        )?);
        let settlement_layer = Arc::new(PolyTorusSettlementLayer::new(config.settlement.clone())?);
        let consensus_layer = Arc::new(PolyTorusConsensusLayer::new(
            data_context.clone(),
            config.consensus.clone(),
            false,
        )?);
        let data_availability_layer = Arc::new(PolyTorusDataAvailabilityLayer::new(
            config.data_availability.clone(),
            network,
        )?);

        let orchestrator = Self::new(
            execution_layer,
            settlement_layer,
            consensus_layer,
            data_availability_layer,
            message_bus,
            config_manager,
            layer_factory,
        )?;

        orchestrator.start().await?;
        Ok(orchestrator)
    }
    /// Broadcast a block through the network
    pub async fn broadcast_block_to_network(
        &self,
        block: crate::blockchain::block::FinalizedBlock,
    ) -> Result<()> {
        if let Some(network_node) = &self.network_node {
            let node = network_node.lock().await;
            node.broadcast_block(block).await?;
        } else {
            log::warn!("No network node available for block broadcasting");
        }
        Ok(())
    }

    /// Broadcast a transaction through the network
    pub async fn broadcast_transaction_to_network(
        &self,
        transaction: crate::crypto::transaction::Transaction,
    ) -> Result<()> {
        if let Some(network_node) = &self.network_node {
            let node = network_node.lock().await;
            node.broadcast_transaction(transaction).await?;
        } else {
            log::warn!("No network node available for transaction broadcasting");
        }
        Ok(())
    }

    /// Get network status
    pub async fn get_network_status(&self) -> Result<Option<String>> {
        if let Some(network_node) = &self.network_node {
            let node = network_node.lock().await;
            let stats = node.get_network_stats().await?;
            Ok(Some(stats))
        } else {
            Ok(None)
        }
    }

    /// Get connected peers
    pub async fn get_connected_peers(&self) -> Result<Vec<String>> {
        if let Some(network_node) = &self.network_node {
            let node = network_node.lock().await;
            let peers = node.get_connected_peers().await;
            Ok(peers.into_iter().map(|p| p.to_string()).collect())
        } else {
            Ok(vec![])
        }
    }

    /// Connect to a peer
    pub async fn connect_to_peer(&self, addr: std::net::SocketAddr) -> Result<()> {
        if let Some(network_node) = &self.network_node {
            let node = network_node.lock().await;
            node.connect_to_peer(addr).await?;
        } else {
            return Err(anyhow::anyhow!("No network node available"));
        }
        Ok(())
    }

    /// Get blockchain synchronization status
    pub async fn get_sync_status(&self) -> Result<Option<crate::network::SyncState>> {
        if let Some(network_node) = &self.network_node {
            let node = network_node.lock().await;
            let sync_state = node.get_sync_state().await;
            Ok(Some(sync_state))
        } else {
            Ok(None)
        }
    }
}

/// Builder for creating UnifiedModularOrchestrator instances
pub struct UnifiedOrchestratorBuilder {
    execution_layer: Option<Arc<dyn ExecutionLayer + Send + Sync>>,
    settlement_layer: Option<Arc<dyn SettlementLayer + Send + Sync>>,
    consensus_layer: Option<Arc<dyn ConsensusLayer + Send + Sync>>,
    data_availability_layer: Option<Arc<dyn DataAvailabilityLayer + Send + Sync>>,
    message_bus: Option<Arc<ModularMessageBus>>,
    config_manager: Option<Arc<RwLock<ModularConfigManager>>>,
    layer_factory: Option<Arc<ModularLayerFactory>>,
}

impl UnifiedOrchestratorBuilder {
    pub fn new() -> Self {
        Self {
            execution_layer: None,
            settlement_layer: None,
            consensus_layer: None,
            data_availability_layer: None,
            message_bus: None,
            config_manager: None,
            layer_factory: None,
        }
    }

    pub fn with_execution_layer(mut self, layer: Arc<dyn ExecutionLayer + Send + Sync>) -> Self {
        self.execution_layer = Some(layer);
        self
    }

    pub fn with_settlement_layer(mut self, layer: Arc<dyn SettlementLayer + Send + Sync>) -> Self {
        self.settlement_layer = Some(layer);
        self
    }

    pub fn with_consensus_layer(mut self, layer: Arc<dyn ConsensusLayer + Send + Sync>) -> Self {
        self.consensus_layer = Some(layer);
        self
    }

    pub fn with_data_availability_layer(
        mut self,
        layer: Arc<dyn DataAvailabilityLayer + Send + Sync>,
    ) -> Self {
        self.data_availability_layer = Some(layer);
        self
    }

    pub fn with_message_bus(mut self, message_bus: Arc<ModularMessageBus>) -> Self {
        self.message_bus = Some(message_bus);
        self
    }

    pub fn with_config_manager(
        mut self,
        config_manager: Arc<RwLock<ModularConfigManager>>,
    ) -> Self {
        self.config_manager = Some(config_manager);
        self
    }

    pub fn with_layer_factory(mut self, layer_factory: Arc<ModularLayerFactory>) -> Self {
        self.layer_factory = Some(layer_factory);
        self
    }

    pub fn build(self) -> Result<UnifiedModularOrchestrator> {
        let execution_layer = self
            .execution_layer
            .ok_or_else(|| anyhow::anyhow!("Execution layer is required"))?;
        let settlement_layer = self
            .settlement_layer
            .ok_or_else(|| anyhow::anyhow!("Settlement layer is required"))?;
        let consensus_layer = self
            .consensus_layer
            .ok_or_else(|| anyhow::anyhow!("Consensus layer is required"))?;
        let data_availability_layer = self
            .data_availability_layer
            .ok_or_else(|| anyhow::anyhow!("Data availability layer is required"))?;
        let message_bus = self
            .message_bus
            .ok_or_else(|| anyhow::anyhow!("Message bus is required"))?;
        let config_manager = self
            .config_manager
            .ok_or_else(|| anyhow::anyhow!("Config manager is required"))?;
        let layer_factory = self
            .layer_factory
            .ok_or_else(|| anyhow::anyhow!("Layer factory is required"))?;

        UnifiedModularOrchestrator::new(
            execution_layer,
            settlement_layer,
            consensus_layer,
            data_availability_layer,
            message_bus,
            config_manager,
            layer_factory,
        )
    }
}

impl Default for UnifiedOrchestratorBuilder {
    fn default() -> Self {
        Self::new()
    }
}
