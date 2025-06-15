//! Unified Modular Blockchain Architecture for PolyTorus
//!
//! This module implements a truly modular blockchain design where different layers
//! (execution, settlement, consensus, data availability) are separated and
//! can be independently developed, tested, and deployed. The architecture supports
//! pluggable implementations, sophisticated configuration management, and
//! event-driven communication between layers.

use std::fs;
use std::path::Path;

use crate::Result;

// Core modular components
pub mod consensus;
pub mod data_availability;
pub mod diamond_io_layer;
pub mod eutxo_processor;
pub mod execution;
pub mod network;
pub mod settlement;
pub mod storage;
pub mod traits;
pub mod transaction_processor;

// Unified modular architecture
pub mod config_manager;
pub mod layer_factory;
pub mod message_bus;
pub mod unified_orchestrator;

#[cfg(kani)]
pub mod kani_verification;

// Re-export main types and traits
// Supporting modular components exports
pub use config_manager::{
    create_config_templates,
    ConfigTemplate,
    ModularConfigManager,
    UseCase,
    ValidationResult,
};
pub use consensus::PolyTorusConsensusLayer;
pub use data_availability::PolyTorusDataAvailabilityLayer;
pub use diamond_io_layer::{
    DiamondIOLayer,
    DiamondIOLayerConfig,
    DiamondIOLayerFactory,
    DiamondIOMessage,
    DiamondIOStats,
};
pub use eutxo_processor::{
    EUtxoProcessor,
    EUtxoProcessorConfig,
    UtxoState,
    UtxoStats,
};
pub use execution::PolyTorusExecutionLayer;
pub use layer_factory::{
    create_default_enhanced_config,
    EnhancedModularConfig,
    GlobalConfig,
    LayerConfig,
    LayerImplementation,
    ModularLayerFactory,
    PerformanceMode,
};
pub use message_bus::{
    HealthStatus,
    LayerInfo,
    LayerType,
    MessageBuilder,
    MessagePayload,
    MessagePriority,
    MessageType,
    ModularMessage,
    ModularMessageBus,
};
pub use network::{
    ModularNetwork,
    ModularNetworkConfig,
    ModularNetworkStats,
};
pub use settlement::PolyTorusSettlementLayer;
pub use storage::{
    BlockMetadata,
    ModularStorage,
    StorageConfig,
    StorageLayer,
    StorageLayerBuilder,
    StorageStats,
};
pub use traits::*;
// Re-export configuration types for external use
pub use traits::{
    ConsensusConfig,
    DataAvailabilityConfig,
    ExecutionConfig,
    ModularConfig,
    NetworkConfig,
    SettlementConfig,
    WasmConfig,
};
pub use transaction_processor::{
    ModularTransactionProcessor,
    ProcessorAccountState,
    TransactionProcessorConfig,
    TransactionResult,
};
// Main unified orchestrator exports
pub use unified_orchestrator::{
    AlertSeverity,
    ExecutionEventResult,
    LayerMetrics,
    LayerStatus,
    OrchestratorMetrics,
    OrchestratorState,
    UnifiedEvent,
    UnifiedModularOrchestrator,
    UnifiedOrchestratorBuilder,
};

#[cfg(test)]
mod tests;

/// Create a default modular blockchain configuration
pub fn default_modular_config() -> ModularConfig {
    ModularConfig {
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
            challenge_period: 100, // 100 blocks
            batch_size: 100,
            min_validator_stake: 1000,
        },
        consensus: ConsensusConfig {
            block_time: 10000, // 10 seconds
            difficulty: 4,
            max_block_size: 1024 * 1024, // 1MB
        },
        data_availability: DataAvailabilityConfig {
            network_config: NetworkConfig {
                listen_addr: "0.0.0.0:7000".to_string(),
                bootstrap_peers: Vec::new(),
                max_peers: 50,
            },
            retention_period: 86400 * 7, // 7 days
            max_data_size: 1024 * 1024,  // 1MB
        },
    }
}

/// Load modular blockchain configuration from a TOML file
pub fn load_modular_config_from_file<P: AsRef<Path>>(path: P) -> Result<ModularConfig> {
    let config_str = fs::read_to_string(path)?;
    let config: ModularConfig = toml::from_str(&config_str)?;
    Ok(config)
}
