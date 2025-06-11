//! Enhanced Modular Configuration System
//!
//! This module provides a sophisticated configuration system for the modular blockchain,
//! supporting layer-specific configurations, environment variables, and runtime updates.

use super::layer_factory::{EnhancedModularConfig, LayerConfig, PerformanceMode};
use super::message_bus::LayerType;
use super::traits::*;
use crate::Result;

use serde::{Deserialize, Serialize};
use std::env;
use std::path::Path;

/// Configuration manager for the modular blockchain
pub struct ModularConfigManager {
    /// Current configuration
    config: EnhancedModularConfig,
    /// Configuration file path
    config_path: Option<String>,
    /// Environment prefix for variables
    env_prefix: String,
    /// Watchers for configuration changes
    change_watchers: Vec<Box<dyn Fn(&EnhancedModularConfig) + Send + Sync>>,
}

/// Configuration validation result
#[derive(Debug)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub errors: Vec<String>,
    pub warnings: Vec<String>,
}

/// Configuration template for different use cases
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConfigTemplate {
    pub name: String,
    pub description: String,
    pub use_case: UseCase,
    pub config: EnhancedModularConfig,
}

/// Use case enumeration for configuration templates
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum UseCase {
    Development,
    Testing,
    Mainnet,
    Testnet,
    HighThroughput,
    LowLatency,
    CustomExperiment,
}

impl Default for ModularConfigManager {
    fn default() -> Self {
        Self::new()
    }
}

impl ModularConfigManager {
    /// Create a new configuration manager
    pub fn new() -> Self {
        Self {
            config: create_default_config(),
            config_path: None,
            env_prefix: "POLYTORUS".to_string(),
            change_watchers: Vec::new(),
        }
    }

    /// Create with specific configuration
    pub fn with_config(config: EnhancedModularConfig) -> Self {
        Self {
            config,
            config_path: None,
            env_prefix: "POLYTORUS".to_string(),
            change_watchers: Vec::new(),
        }
    }

    /// Load configuration from file
    pub fn load_from_file<P: AsRef<Path>>(path: P) -> Result<Self> {
        let content = std::fs::read_to_string(&path)
            .map_err(|e| failure::format_err!("Failed to read config file: {}", e))?;

        let config: EnhancedModularConfig = toml::from_str(&content)
            .map_err(|e| failure::format_err!("Failed to parse config file: {}", e))?;

        let mut manager = Self::with_config(config);
        manager.config_path = Some(path.as_ref().to_string_lossy().to_string());

        // Apply environment variable overrides
        manager.apply_env_overrides()?;

        log::info!(
            "Loaded configuration from: {}",
            manager.config_path.as_ref().unwrap()
        );
        Ok(manager)
    }

    /// Apply environment variable overrides
    fn apply_env_overrides(&mut self) -> Result<()> {
        // Override global configuration
        if let Ok(network_mode) = env::var(format!("{}_NETWORK_MODE", self.env_prefix)) {
            self.config.global.network_mode = network_mode;
        }

        if let Ok(log_level) = env::var(format!("{}_LOG_LEVEL", self.env_prefix)) {
            self.config.global.log_level = log_level;
        }

        if let Ok(performance_mode) = env::var(format!("{}_PERFORMANCE_MODE", self.env_prefix)) {
            self.config.global.performance_mode = match performance_mode.as_str() {
                "development" => PerformanceMode::Development,
                "testing" => PerformanceMode::Testing,
                "production" => PerformanceMode::Production,
                "high-throughput" => PerformanceMode::HighThroughput,
                "low-latency" => PerformanceMode::LowLatency,
                _ => self.config.global.performance_mode.clone(),
            };
        }

        // Override layer-specific configurations
        self.apply_execution_layer_overrides()?;
        self.apply_consensus_layer_overrides()?;
        self.apply_settlement_layer_overrides()?;
        self.apply_data_availability_overrides()?;

        log::debug!("Applied environment variable overrides");
        Ok(())
    }

    /// Apply execution layer environment overrides
    fn apply_execution_layer_overrides(&mut self) -> Result<()> {
        if let Some(layer_config) = self.config.layers.get_mut(&LayerType::Execution) {
            let mut exec_config: ExecutionConfig =
                serde_json::from_value(layer_config.config.clone())?;

            if let Ok(gas_limit) = env::var(format!("{}_EXECUTION_GAS_LIMIT", self.env_prefix)) {
                if let Ok(limit) = gas_limit.parse::<u64>() {
                    exec_config.gas_limit = limit;
                }
            }

            if let Ok(gas_price) = env::var(format!("{}_EXECUTION_GAS_PRICE", self.env_prefix)) {
                if let Ok(price) = gas_price.parse::<u64>() {
                    exec_config.gas_price = price;
                }
            }

            layer_config.config = serde_json::to_value(exec_config)?;
        }
        Ok(())
    }

    /// Apply consensus layer environment overrides
    fn apply_consensus_layer_overrides(&mut self) -> Result<()> {
        if let Some(layer_config) = self.config.layers.get_mut(&LayerType::Consensus) {
            let mut consensus_config: ConsensusConfig =
                serde_json::from_value(layer_config.config.clone())?;

            if let Ok(difficulty) = env::var(format!("{}_CONSENSUS_DIFFICULTY", self.env_prefix)) {
                if let Ok(diff) = difficulty.parse::<usize>() {
                    consensus_config.difficulty = diff;
                }
            }

            if let Ok(block_time) = env::var(format!("{}_CONSENSUS_BLOCK_TIME", self.env_prefix)) {
                if let Ok(time) = block_time.parse::<u64>() {
                    consensus_config.block_time = time;
                }
            }

            layer_config.config = serde_json::to_value(consensus_config)?;
        }
        Ok(())
    }

    /// Apply settlement layer environment overrides
    fn apply_settlement_layer_overrides(&mut self) -> Result<()> {
        if let Some(layer_config) = self.config.layers.get_mut(&LayerType::Settlement) {
            let mut settlement_config: SettlementConfig =
                serde_json::from_value(layer_config.config.clone())?;

            if let Ok(challenge_period) =
                env::var(format!("{}_SETTLEMENT_CHALLENGE_PERIOD", self.env_prefix))
            {
                if let Ok(period) = challenge_period.parse::<u64>() {
                    settlement_config.challenge_period = period;
                }
            }

            layer_config.config = serde_json::to_value(settlement_config)?;
        }
        Ok(())
    }

    /// Apply data availability layer environment overrides
    fn apply_data_availability_overrides(&mut self) -> Result<()> {
        if let Some(layer_config) = self.config.layers.get_mut(&LayerType::DataAvailability) {
            let mut da_config: DataAvailabilityConfig =
                serde_json::from_value(layer_config.config.clone())?;

            if let Ok(listen_addr) = env::var(format!("{}_DA_LISTEN_ADDR", self.env_prefix)) {
                da_config.network_config.listen_addr = listen_addr;
            }

            if let Ok(max_peers) = env::var(format!("{}_DA_MAX_PEERS", self.env_prefix)) {
                if let Ok(peers) = max_peers.parse::<usize>() {
                    da_config.network_config.max_peers = peers;
                }
            }

            layer_config.config = serde_json::to_value(da_config)?;
        }
        Ok(())
    }

    /// Validate the current configuration
    pub fn validate(&self) -> ValidationResult {
        let mut result = ValidationResult {
            is_valid: true,
            errors: Vec::new(),
            warnings: Vec::new(),
        };

        // Validate global configuration
        self.validate_global_config(&mut result);

        // Validate each layer configuration
        for (layer_type, layer_config) in &self.config.layers {
            self.validate_layer_config(layer_type, layer_config, &mut result);
        }

        // Check dependencies
        self.validate_dependencies(&mut result);

        result
    }

    /// Validate global configuration
    fn validate_global_config(&self, result: &mut ValidationResult) {
        // Validate network mode
        let valid_network_modes = ["mainnet", "testnet", "devnet"];
        if !valid_network_modes.contains(&self.config.global.network_mode.as_str()) {
            result.errors.push(format!(
                "Invalid network mode: {}",
                self.config.global.network_mode
            ));
            result.is_valid = false;
        }

        // Validate log level
        let valid_log_levels = ["trace", "debug", "info", "warn", "error"];
        if !valid_log_levels.contains(&self.config.global.log_level.as_str()) {
            result.warnings.push(format!(
                "Unknown log level: {}",
                self.config.global.log_level
            ));
        }
    }

    /// Validate layer configuration
    fn validate_layer_config(
        &self,
        layer_type: &LayerType,
        _layer_config: &LayerConfig,
        result: &mut ValidationResult,
    ) {
        // Add layer-specific validation logic here
        match layer_type {
            LayerType::Execution => {
                // Validate execution layer configuration
                if let Ok(exec_config) = self.get_execution_config() {
                    if exec_config.gas_limit == 0 {
                        result
                            .errors
                            .push("Execution gas limit cannot be zero".to_string());
                        result.is_valid = false;
                    }
                    if exec_config.wasm_config.max_memory_pages == 0 {
                        result
                            .warnings
                            .push("WASM memory pages set to zero may cause issues".to_string());
                    }
                }
            }
            LayerType::Consensus => {
                // Validate consensus layer configuration
                if let Ok(consensus_config) = self.get_consensus_config() {
                    if consensus_config.difficulty == 0 {
                        result
                            .warnings
                            .push("Consensus difficulty is zero (very easy mining)".to_string());
                    }
                    if consensus_config.block_time < 1000 {
                        result
                            .warnings
                            .push("Block time is very low, may cause instability".to_string());
                    }
                }
            }
            LayerType::Settlement => {
                // Validate settlement layer configuration
                if let Ok(settlement_config) = self.get_settlement_config() {
                    if settlement_config.challenge_period == 0 {
                        result
                            .errors
                            .push("Settlement challenge period cannot be zero".to_string());
                        result.is_valid = false;
                    }
                }
            }
            LayerType::DataAvailability => {
                // Validate data availability layer configuration
                if let Ok(da_config) = self.get_data_availability_config() {
                    if da_config.network_config.max_peers == 0 {
                        result
                            .warnings
                            .push("Data availability max peers is zero".to_string());
                    }
                }
            }
            _ => {
                // Other layer types
            }
        }
    }

    /// Validate layer dependencies
    fn validate_dependencies(&self, result: &mut ValidationResult) {
        for (layer_type, layer_config) in &self.config.layers {
            for dependency in &layer_config.dependencies {
                if !self.config.layers.contains_key(dependency) {
                    result.errors.push(format!(
                        "Layer {:?} depends on {:?} which is not configured",
                        layer_type, dependency
                    ));
                    result.is_valid = false;
                }
            }
        }
    }

    /// Get the current configuration
    pub fn get_config(&self) -> &EnhancedModularConfig {
        &self.config
    }

    /// Get execution layer configuration
    pub fn get_execution_config(&self) -> Result<ExecutionConfig> {
        let layer_config = self
            .config
            .layers
            .get(&LayerType::Execution)
            .ok_or_else(|| failure::format_err!("Execution layer not configured"))?;

        serde_json::from_value(layer_config.config.clone())
            .map_err(|e| failure::format_err!("Invalid execution config: {}", e))
    }

    /// Get consensus layer configuration
    pub fn get_consensus_config(&self) -> Result<ConsensusConfig> {
        let layer_config = self
            .config
            .layers
            .get(&LayerType::Consensus)
            .ok_or_else(|| failure::format_err!("Consensus layer not configured"))?;

        serde_json::from_value(layer_config.config.clone())
            .map_err(|e| failure::format_err!("Invalid consensus config: {}", e))
    }

    /// Get settlement layer configuration
    pub fn get_settlement_config(&self) -> Result<SettlementConfig> {
        let layer_config = self
            .config
            .layers
            .get(&LayerType::Settlement)
            .ok_or_else(|| failure::format_err!("Settlement layer not configured"))?;

        serde_json::from_value(layer_config.config.clone())
            .map_err(|e| failure::format_err!("Invalid settlement config: {}", e))
    }

    /// Get data availability layer configuration
    pub fn get_data_availability_config(&self) -> Result<DataAvailabilityConfig> {
        let layer_config = self
            .config
            .layers
            .get(&LayerType::DataAvailability)
            .ok_or_else(|| failure::format_err!("Data availability layer not configured"))?;

        serde_json::from_value(layer_config.config.clone())
            .map_err(|e| failure::format_err!("Invalid data availability config: {}", e))
    }

    /// Save configuration to file
    pub fn save_to_file<P: AsRef<Path>>(&self, path: P) -> Result<()> {
        let content = toml::to_string_pretty(&self.config)
            .map_err(|e| failure::format_err!("Failed to serialize config: {}", e))?;

        std::fs::write(&path, content)
            .map_err(|e| failure::format_err!("Failed to write config file: {}", e))?;

        log::info!("Saved configuration to: {}", path.as_ref().display());
        Ok(())
    }

    /// Update configuration at runtime
    pub fn update_config(&mut self, new_config: EnhancedModularConfig) -> Result<()> {
        // Validate new configuration
        let temp_manager = Self::with_config(new_config.clone());
        let validation = temp_manager.validate();

        if !validation.is_valid {
            return Err(failure::format_err!(
                "Invalid configuration: {:?}",
                validation.errors
            ));
        }

        // Apply the new configuration
        self.config = new_config;

        // Notify watchers
        for watcher in &self.change_watchers {
            watcher(&self.config);
        }

        log::info!("Configuration updated successfully");
        Ok(())
    }

    /// Add a configuration change watcher
    pub fn add_change_watcher<F>(&mut self, watcher: F)
    where
        F: Fn(&EnhancedModularConfig) + Send + Sync + 'static,
    {
        self.change_watchers.push(Box::new(watcher));
    }
}

/// Create a default configuration
fn create_default_config() -> EnhancedModularConfig {
    super::layer_factory::create_default_enhanced_config()
}

/// Create configuration templates for different use cases
pub fn create_config_templates() -> Vec<ConfigTemplate> {
    vec![
        ConfigTemplate {
            name: "Development".to_string(),
            description: "Configuration optimized for development and testing".to_string(),
            use_case: UseCase::Development,
            config: create_development_config(),
        },
        ConfigTemplate {
            name: "High Throughput".to_string(),
            description: "Configuration optimized for maximum transaction throughput".to_string(),
            use_case: UseCase::HighThroughput,
            config: create_high_throughput_config(),
        },
        ConfigTemplate {
            name: "Low Latency".to_string(),
            description: "Configuration optimized for minimal latency".to_string(),
            use_case: UseCase::LowLatency,
            config: create_low_latency_config(),
        },
    ]
}

/// Create development-optimized configuration
fn create_development_config() -> EnhancedModularConfig {
    let mut config = create_default_config();

    // Development-specific settings
    config.global.performance_mode = PerformanceMode::Development;
    config.global.log_level = "debug".to_string();

    // Lower difficulty for faster mining
    if let Some(consensus_config) = config.layers.get_mut(&LayerType::Consensus) {
        let mut consensus: ConsensusConfig =
            serde_json::from_value(consensus_config.config.clone()).unwrap();
        consensus.difficulty = 1;
        consensus.block_time = 5000; // 5 seconds
        consensus_config.config = serde_json::to_value(consensus).unwrap();
    }

    config
}

/// Create high throughput configuration
fn create_high_throughput_config() -> EnhancedModularConfig {
    let mut config = create_default_config();

    config.global.performance_mode = PerformanceMode::HighThroughput;

    // Higher gas limits and batch sizes
    if let Some(execution_config) = config.layers.get_mut(&LayerType::Execution) {
        let mut exec: ExecutionConfig =
            serde_json::from_value(execution_config.config.clone()).unwrap();
        exec.gas_limit = 20_000_000; // Higher gas limit
        execution_config.config = serde_json::to_value(exec).unwrap();
    }

    if let Some(settlement_config) = config.layers.get_mut(&LayerType::Settlement) {
        let mut settlement: SettlementConfig =
            serde_json::from_value(settlement_config.config.clone()).unwrap();
        settlement.batch_size = 500; // Larger batch size
        settlement_config.config = serde_json::to_value(settlement).unwrap();
    }

    config
}

/// Create low latency configuration
fn create_low_latency_config() -> EnhancedModularConfig {
    let mut config = create_default_config();

    config.global.performance_mode = PerformanceMode::LowLatency;

    // Faster block times and smaller batches
    if let Some(consensus_config) = config.layers.get_mut(&LayerType::Consensus) {
        let mut consensus: ConsensusConfig =
            serde_json::from_value(consensus_config.config.clone()).unwrap();
        consensus.block_time = 3000; // 3 seconds
        consensus_config.config = serde_json::to_value(consensus).unwrap();
    }

    if let Some(settlement_config) = config.layers.get_mut(&LayerType::Settlement) {
        let mut settlement: SettlementConfig =
            serde_json::from_value(settlement_config.config.clone()).unwrap();
        settlement.batch_size = 50; // Smaller batch size for faster processing
        settlement.challenge_period = 50; // Shorter challenge period
        settlement_config.config = serde_json::to_value(settlement).unwrap();
    }

    config
}
