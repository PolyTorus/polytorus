//! Modular blockchain architecture for PolyTorus
//!
//! This module implements a modular blockchain design where different layers
//! (execution, settlement, consensus, data availability) are separated and
//! can be independently developed, tested, and deployed.

use crate::Result;
use std::fs;
use std::path::Path;

pub mod consensus;
pub mod data_availability;
pub mod execution;
pub mod network;
pub mod orchestrator;
pub mod settlement;
pub mod storage;
pub mod traits;
pub mod transaction_processor;

// Re-export main types and traits
pub use consensus::PolyTorusConsensusLayer;
pub use data_availability::PolyTorusDataAvailabilityLayer;
pub use execution::PolyTorusExecutionLayer;
pub use network::{ModularNetwork, ModularNetworkConfig, ModularNetworkStats};
pub use orchestrator::{ModularBlockchain, ModularBlockchainBuilder, ModularEvent, StateInfo};
pub use settlement::PolyTorusSettlementLayer;
pub use storage::{ModularStorage, StorageConfig, StorageLayer, StorageLayerBuilder, StorageStats, BlockMetadata};
pub use transaction_processor::{ModularTransactionProcessor, TransactionProcessorConfig, ProcessorAccountState, TransactionResult};
pub use traits::*;

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
