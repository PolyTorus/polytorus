//! Configuration module
//!
//! This module provides configuration management for the PolyTorus blockchain,
//! including network settings, execution parameters, and environment variable support.

pub mod enhanced_config;

// Re-export commonly used types
use std::path::PathBuf;

pub use enhanced_config::{
    CompleteConfig,
    ConfigManager,
    ConsensusConfig,
    ExecutionConfig,
    LoggingConfig,
    NetworkConfig,
    StorageConfig,
};

// Legacy compatibility - maintain existing DataContext structure
use crate::Result;

/// Data context for legacy compatibility
#[derive(Debug, Clone)]
pub struct DataContext {
    pub data_dir: PathBuf,
    pub wallet_dir: PathBuf,
    pub blockchain_dir: PathBuf,
    pub contracts_db_path: String,
}

impl Default for DataContext {
    fn default() -> Self {
        let data_dir = PathBuf::from("./data");
        Self {
            wallet_dir: data_dir.join("wallets"),
            blockchain_dir: data_dir.join("blockchain"),
            contracts_db_path: data_dir.join("contracts").join("db").to_string_lossy().to_string(),
            data_dir,
        }
    }
}

impl DataContext {
    pub fn new(data_dir: PathBuf) -> Self {
        Self {
            wallet_dir: data_dir.join("wallets"),
            blockchain_dir: data_dir.join("blockchain"),
            contracts_db_path: data_dir.join("contracts").join("db").to_string_lossy().to_string(),
            data_dir,
        }
    }

    pub fn ensure_directories(&self) -> Result<()> {
        std::fs::create_dir_all(&self.data_dir)?;
        std::fs::create_dir_all(&self.wallet_dir)?;
        std::fs::create_dir_all(&self.blockchain_dir)?;
        std::fs::create_dir_all(PathBuf::from(&self.contracts_db_path).parent().unwrap())?;
        Ok(())
    }

    pub fn data_dir(&self) -> &PathBuf {
        &self.data_dir
    }

    pub fn wallets_dir(&self) -> &PathBuf {
        &self.wallet_dir
    }

    pub fn blockchain_dir(&self) -> &PathBuf {
        &self.blockchain_dir
    }
}

/// Configuration builder for easy setup
pub struct ConfigBuilder {
    config: CompleteConfig,
}

impl ConfigBuilder {
    pub fn new() -> Self {
        Self {
            config: CompleteConfig::default(),
        }
    }

    pub fn with_network_listen_addr(mut self, addr: String) -> Self {
        self.config.network.listen_addr = addr;
        self
    }

    pub fn with_bootstrap_peers(mut self, peers: Vec<String>) -> Self {
        self.config.network.bootstrap_peers = peers;
        self
    }

    pub fn with_data_dir(mut self, dir: String) -> Self {
        self.config.storage.data_dir = dir;
        self
    }

    pub fn with_log_level(mut self, level: String) -> Self {
        self.config.logging.level = level;
        self
    }

    pub fn build(self) -> CompleteConfig {
        self.config
    }
}

impl Default for ConfigBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for CompleteConfig {
    fn default() -> Self {
        use enhanced_config::*;

        CompleteConfig {
            execution: ExecutionConfig {
                gas_limit: 8000000,
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
                max_block_size: 1048576,
            },
            data_availability: DataAvailabilityConfig {
                retention_period: 604800,
                max_data_size: 1048576,
                network_config: DaNetworkConfig {
                    listen_addr: "0.0.0.0:7000".to_string(),
                    bootstrap_peers: vec![],
                    max_peers: 50,
                },
            },
            network: NetworkConfig {
                listen_addr: "0.0.0.0:8000".to_string(),
                bootstrap_peers: vec![],
                max_peers: 50,
                connection_timeout: 10,
                ping_interval: 30,
                peer_timeout: 120,
                enable_discovery: true,
                discovery_interval: 300,
                max_message_size: 10485760,
                bandwidth_limit: None,
            },
            logging: LoggingConfig {
                level: "INFO".to_string(),
                output: "console".to_string(),
                file_path: None,
                max_file_size: 104857600,
                rotation_count: 5,
            },
            storage: StorageConfig {
                data_dir: "./data".to_string(),
                max_cache_size: 1073741824,
                sync_interval: 60,
                compression: true,
                backup_interval: Some(3600),
            },
        }
    }
}
