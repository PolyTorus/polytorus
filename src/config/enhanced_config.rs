//! Enhanced configuration management with network settings
//!
//! This module provides comprehensive configuration management including
//! network settings, environment variable overrides, and dynamic updates.

use crate::Result;
use failure::format_err;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::env;
use std::fs;
use std::net::SocketAddr;
use std::path::Path;

/// Complete configuration structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CompleteConfig {
    pub execution: ExecutionConfig,
    pub settlement: SettlementConfig,
    pub consensus: ConsensusConfig,
    pub data_availability: DataAvailabilityConfig,
    pub network: NetworkConfig,
    pub logging: LoggingConfig,
    pub storage: StorageConfig,
}

/// Execution layer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionConfig {
    pub gas_limit: u64,
    pub gas_price: u64,
    pub wasm_config: WasmConfig,
}

/// WASM execution configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WasmConfig {
    pub max_memory_pages: u32,
    pub max_stack_size: u32,
    pub gas_metering: bool,
}

/// Settlement layer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SettlementConfig {
    pub challenge_period: u32,
    pub batch_size: u32,
    pub min_validator_stake: u64,
}

/// Consensus configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConsensusConfig {
    pub block_time: u64,
    pub difficulty: u32,
    pub max_block_size: u64,
}

/// Data availability configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataAvailabilityConfig {
    pub retention_period: u64,
    pub max_data_size: u64,
    pub network_config: DaNetworkConfig,
}

/// Data availability network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DaNetworkConfig {
    pub listen_addr: String,
    pub bootstrap_peers: Vec<String>,
    pub max_peers: u32,
}

/// Enhanced network configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    pub listen_addr: String,
    pub bootstrap_peers: Vec<String>,
    pub max_peers: u32,
    pub connection_timeout: u64,
    pub ping_interval: u64,
    pub peer_timeout: u64,
    pub enable_discovery: bool,
    pub discovery_interval: u64,
    pub max_message_size: u64,
    pub bandwidth_limit: Option<u64>,
}

/// Logging configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LoggingConfig {
    pub level: String,
    pub output: String,
    pub file_path: Option<String>,
    pub max_file_size: u64,
    pub rotation_count: u32,
}

/// Storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    pub data_dir: String,
    pub max_cache_size: u64,
    pub sync_interval: u64,
    pub compression: bool,
    pub backup_interval: Option<u64>,
}

/// Configuration manager with environment variable support
pub struct ConfigManager {
    config: CompleteConfig,
    config_file_path: String,
    env_prefix: String,
}

impl ConfigManager {
    /// Create a new configuration manager
    pub fn new(config_file_path: String) -> Result<Self> {
        let config = if Path::new(&config_file_path).exists() {
            Self::load_from_file(&config_file_path)?
        } else {
            Self::default_config()
        };

        let mut manager = ConfigManager {
            config,
            config_file_path,
            env_prefix: "POLYTORUS_".to_string(),
        };

        // Apply environment variable overrides
        manager.apply_env_overrides()?;

        Ok(manager)
    }

    /// Load configuration from file
    fn load_from_file(path: &str) -> Result<CompleteConfig> {
        let contents = fs::read_to_string(path)
            .map_err(|e| format_err!("Failed to read config file {}: {}", path, e))?;

        toml::from_str(&contents)
            .map_err(|e| format_err!("Failed to parse config file {}: {}", path, e))
    }

    /// Get default configuration
    fn default_config() -> CompleteConfig {
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
                block_time: 10000, // 10 seconds
                difficulty: 4,
                max_block_size: 1048576, // 1MB
            },
            data_availability: DataAvailabilityConfig {
                retention_period: 604800, // 7 days
                max_data_size: 1048576,   // 1MB
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
                max_message_size: 10485760, // 10MB
                bandwidth_limit: None,
            },
            logging: LoggingConfig {
                level: "INFO".to_string(),
                output: "console".to_string(),
                file_path: None,
                max_file_size: 104857600, // 100MB
                rotation_count: 5,
            },
            storage: StorageConfig {
                data_dir: "./data".to_string(),
                max_cache_size: 1073741824, // 1GB
                sync_interval: 60,
                compression: true,
                backup_interval: Some(3600), // 1 hour
            },
        }
    }

    /// Apply environment variable overrides
    fn apply_env_overrides(&mut self) -> Result<()> {
        // Network configuration overrides
        if let Ok(listen_addr) = env::var(format!("{}NETWORK_LISTEN_ADDR", self.env_prefix)) {
            self.config.network.listen_addr = listen_addr;
        }

        if let Ok(bootstrap_peers) = env::var(format!("{}NETWORK_BOOTSTRAP_PEERS", self.env_prefix)) {
            self.config.network.bootstrap_peers = 
                bootstrap_peers.split(',').map(|s| s.trim().to_string()).collect();
        }

        if let Ok(max_peers) = env::var(format!("{}NETWORK_MAX_PEERS", self.env_prefix)) {
            self.config.network.max_peers = max_peers.parse()
                .map_err(|e| format_err!("Invalid NETWORK_MAX_PEERS value: {}", e))?;
        }

        // Consensus configuration overrides
        if let Ok(block_time) = env::var(format!("{}CONSENSUS_BLOCK_TIME", self.env_prefix)) {
            self.config.consensus.block_time = block_time.parse()
                .map_err(|e| format_err!("Invalid CONSENSUS_BLOCK_TIME value: {}", e))?;
        }

        if let Ok(difficulty) = env::var(format!("{}CONSENSUS_DIFFICULTY", self.env_prefix)) {
            self.config.consensus.difficulty = difficulty.parse()
                .map_err(|e| format_err!("Invalid CONSENSUS_DIFFICULTY value: {}", e))?;
        }

        // Storage configuration overrides
        if let Ok(data_dir) = env::var(format!("{}STORAGE_DATA_DIR", self.env_prefix)) {
            self.config.storage.data_dir = data_dir;
        }

        // Logging configuration overrides
        if let Ok(log_level) = env::var(format!("{}LOG_LEVEL", self.env_prefix)) {
            self.config.logging.level = log_level;
        }

        if let Ok(log_file) = env::var(format!("{}LOG_FILE", self.env_prefix)) {
            self.config.logging.file_path = Some(log_file);
        }

        Ok(())
    }

    /// Get current configuration
    pub fn get_config(&self) -> &CompleteConfig {
        &self.config
    }

    /// Get mutable configuration
    pub fn get_config_mut(&mut self) -> &mut CompleteConfig {
        &mut self.config
    }

    /// Save configuration to file
    pub fn save(&self) -> Result<()> {
        let toml_string = toml::to_string_pretty(&self.config)
            .map_err(|e| format_err!("Failed to serialize config: {}", e))?;

        fs::write(&self.config_file_path, toml_string)
            .map_err(|e| format_err!("Failed to write config file {}: {}", self.config_file_path, e))?;

        Ok(())
    }

    /// Update network configuration
    pub fn update_network_config(&mut self, network_config: NetworkConfig) -> Result<()> {
        self.config.network = network_config;
        self.save()
    }

    /// Update consensus configuration
    pub fn update_consensus_config(&mut self, consensus_config: ConsensusConfig) -> Result<()> {
        self.config.consensus = consensus_config;
        self.save()
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<()> {
        // Validate network configuration
        let _listen_addr: SocketAddr = self.config.network.listen_addr.parse()
            .map_err(|e| format_err!("Invalid listen address: {}", e))?;

        for peer_addr in &self.config.network.bootstrap_peers {
            let _addr: SocketAddr = peer_addr.parse()
                .map_err(|e| format_err!("Invalid bootstrap peer address {}: {}", peer_addr, e))?;
        }

        // Validate storage configuration
        if self.config.storage.data_dir.is_empty() {
            return Err(format_err!("Data directory cannot be empty"));
        }

        // Validate consensus configuration
        if self.config.consensus.block_time == 0 {
            return Err(format_err!("Block time cannot be zero"));
        }

        if self.config.consensus.max_block_size == 0 {
            return Err(format_err!("Max block size cannot be zero"));
        }

        // Validate execution configuration
        if self.config.execution.gas_limit == 0 {
            return Err(format_err!("Gas limit cannot be zero"));
        }

        Ok(())
    }

    /// Get network configuration as parsed socket addresses
    pub fn get_network_addresses(&self) -> Result<(SocketAddr, Vec<SocketAddr>)> {
        let listen_addr = self.config.network.listen_addr.parse()
            .map_err(|e| format_err!("Invalid listen address: {}", e))?;

        let mut bootstrap_addrs = Vec::new();
        for peer_addr in &self.config.network.bootstrap_peers {
            let addr = peer_addr.parse()
                .map_err(|e| format_err!("Invalid bootstrap peer address {}: {}", peer_addr, e))?;
            bootstrap_addrs.push(addr);
        }

        Ok((listen_addr, bootstrap_addrs))
    }

    /// Get configuration summary
    pub fn get_summary(&self) -> HashMap<String, String> {
        let mut summary = HashMap::new();

        summary.insert("network_listen_addr".to_string(), self.config.network.listen_addr.clone());
        summary.insert("network_bootstrap_peers".to_string(), 
            format!("{}", self.config.network.bootstrap_peers.len()));
        summary.insert("network_max_peers".to_string(), 
            self.config.network.max_peers.to_string());
        
        summary.insert("consensus_block_time".to_string(), 
            self.config.consensus.block_time.to_string());
        summary.insert("consensus_difficulty".to_string(), 
            self.config.consensus.difficulty.to_string());
        
        summary.insert("execution_gas_limit".to_string(), 
            self.config.execution.gas_limit.to_string());
        
        summary.insert("storage_data_dir".to_string(), 
            self.config.storage.data_dir.clone());
        
        summary.insert("logging_level".to_string(), 
            self.config.logging.level.clone());

        summary
    }

    /// Set environment prefix for variable overrides
    pub fn set_env_prefix(&mut self, prefix: String) {
        self.env_prefix = prefix;
    }

    /// Get all available environment variable names
    pub fn get_env_variable_names(&self) -> Vec<String> {
        vec![
            format!("{}NETWORK_LISTEN_ADDR", self.env_prefix),
            format!("{}NETWORK_BOOTSTRAP_PEERS", self.env_prefix),
            format!("{}NETWORK_MAX_PEERS", self.env_prefix),
            format!("{}CONSENSUS_BLOCK_TIME", self.env_prefix),
            format!("{}CONSENSUS_DIFFICULTY", self.env_prefix),
            format!("{}STORAGE_DATA_DIR", self.env_prefix),
            format!("{}LOG_LEVEL", self.env_prefix),
            format!("{}LOG_FILE", self.env_prefix),
        ]
    }
}

impl Default for ConfigManager {
    fn default() -> Self {
        Self::new("config/polytorus.toml".to_string()).unwrap_or_else(|_| {
            ConfigManager {
                config: Self::default_config(),
                config_file_path: "config/polytorus.toml".to_string(),
                env_prefix: "POLYTORUS_".to_string(),
            }
        })
    }
}
