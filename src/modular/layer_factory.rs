//! Modular Layer Factory
//!
//! This module provides a factory system for creating and configuring
//! different implementations of blockchain layers in a pluggable manner.

use std::collections::HashMap;
use std::sync::Arc;

use serde::{
    Deserialize,
    Serialize,
};

use super::consensus::PolyTorusConsensusLayer;
use super::data_availability::PolyTorusDataAvailabilityLayer;
use super::execution::PolyTorusExecutionLayer;
use super::message_bus::{
    HealthStatus,
    LayerInfo,
    LayerType,
    ModularMessageBus,
};
use super::settlement::PolyTorusSettlementLayer;
use super::traits::*;
use crate::config::DataContext;
use crate::Result;

/// Factory for creating modular blockchain layers
pub struct ModularLayerFactory {
    /// Configuration for each layer type
    layer_configs: HashMap<LayerType, LayerConfig>,
    /// Message bus for inter-layer communication
    message_bus: Arc<ModularMessageBus>,
    /// Registry of available layer implementations
    implementation_registry: HashMap<String, LayerImplementation>,
}

/// Layer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LayerConfig {
    /// Implementation name to use
    pub implementation: String,
    /// Layer-specific configuration
    pub config: serde_json::Value,
    /// Whether the layer is enabled
    pub enabled: bool,
    /// Priority level for the layer
    pub priority: u8,
    /// Dependencies on other layers
    pub dependencies: Vec<LayerType>,
}

/// Layer implementation descriptor
#[derive(Clone)]
pub struct LayerImplementation {
    /// Name of the implementation
    pub name: String,
    /// Description
    pub description: String,
    /// Version
    pub version: String,
    /// Supported capabilities
    pub capabilities: Vec<String>,
    /// Factory function for creating the layer
    pub factory: LayerFactoryFunction,
}

/// Factory function type for creating layers
pub type LayerFactoryFunction = Arc<
    dyn Fn(&LayerConfig, &DataContext) -> Result<Box<dyn std::any::Any + Send + Sync>>
        + Send
        + Sync,
>;

/// Enhanced modular configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedModularConfig {
    /// Layer configurations
    pub layers: HashMap<LayerType, LayerConfig>,
    /// Global configuration
    pub global: GlobalConfig,
    /// Plugin configuration
    pub plugins: HashMap<String, serde_json::Value>,
}

/// Global configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GlobalConfig {
    /// Network mode (mainnet, testnet, devnet)
    pub network_mode: String,
    /// Logging level
    pub log_level: String,
    /// Performance mode
    pub performance_mode: PerformanceMode,
    /// Feature flags
    pub features: HashMap<String, bool>,
}

/// Performance mode settings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PerformanceMode {
    Development,
    Testing,
    Production,
    HighThroughput,
    LowLatency,
}

impl ModularLayerFactory {
    /// Create a new layer factory
    pub fn new(message_bus: Arc<ModularMessageBus>) -> Self {
        let mut factory = Self {
            layer_configs: HashMap::new(),
            message_bus,
            implementation_registry: HashMap::new(),
        };

        // Register default implementations
        factory.register_default_implementations();
        factory
    }

    /// Register default layer implementations
    fn register_default_implementations(&mut self) {
        // Register PolyTorus Execution Layer
        self.register_implementation(LayerImplementation {
            name: "polytorus-execution".to_string(),
            description: "Default PolyTorus execution layer with WASM support".to_string(),
            version: "1.0.0".to_string(),
            capabilities: vec![
                "wasm-execution".to_string(),
                "gas-metering".to_string(),
                "smart-contracts".to_string(),
                "eutxo".to_string(),
            ],
            factory: Arc::new(|config, data_context| {
                let execution_config: ExecutionConfig =
                    serde_json::from_value(config.config.clone())
                        .map_err(|e| failure::format_err!("Invalid execution config: {}", e))?;

                let layer = PolyTorusExecutionLayer::new(data_context.clone(), execution_config)?;
                Ok(Box::new(layer) as Box<dyn std::any::Any + Send + Sync>)
            }),
        });

        // Register PolyTorus Consensus Layer
        self.register_implementation(LayerImplementation {
            name: "polytorus-consensus".to_string(),
            description: "Default PolyTorus consensus layer with PoW".to_string(),
            version: "1.0.0".to_string(),
            capabilities: vec![
                "proof-of-work".to_string(),
                "block-validation".to_string(),
                "chain-management".to_string(),
            ],
            factory: Arc::new(|config, data_context| {
                let consensus_config: ConsensusConfig =
                    serde_json::from_value(config.config.clone())
                        .map_err(|e| failure::format_err!("Invalid consensus config: {}", e))?;

                let layer = PolyTorusConsensusLayer::new(
                    data_context.clone(),
                    consensus_config,
                    false, // Default to non-validator
                )?;
                Ok(Box::new(layer) as Box<dyn std::any::Any + Send + Sync>)
            }),
        });

        // Register PolyTorus Settlement Layer
        self.register_implementation(LayerImplementation {
            name: "polytorus-settlement".to_string(),
            description: "Default PolyTorus settlement layer with optimistic rollups".to_string(),
            version: "1.0.0".to_string(),
            capabilities: vec![
                "batch-settlement".to_string(),
                "fraud-proofs".to_string(),
                "challenge-resolution".to_string(),
            ],
            factory: Arc::new(|config, _data_context| {
                let settlement_config: SettlementConfig =
                    serde_json::from_value(config.config.clone())
                        .map_err(|e| failure::format_err!("Invalid settlement config: {}", e))?;

                let layer = PolyTorusSettlementLayer::new(settlement_config)?;
                Ok(Box::new(layer) as Box<dyn std::any::Any + Send + Sync>)
            }),
        });

        // Register PolyTorus Data Availability Layer
        self.register_implementation(LayerImplementation {
            name: "polytorus-data-availability".to_string(),
            description: "Default PolyTorus data availability layer with P2P storage".to_string(),
            version: "1.0.0".to_string(),
            capabilities: vec![
                "p2p-storage".to_string(),
                "data-sampling".to_string(),
                "availability-proofs".to_string(),
            ],
            factory: Arc::new(|config, _data_context| {
                let da_config: DataAvailabilityConfig =
                    serde_json::from_value(config.config.clone())
                        .map_err(|e| failure::format_err!("Invalid DA config: {}", e))?;

                // Create network for DA layer
                let network_config = super::network::ModularNetworkConfig::default();
                let network = Arc::new(super::network::ModularNetwork::new(network_config)?);

                let layer = PolyTorusDataAvailabilityLayer::new(da_config, network)?;
                Ok(Box::new(layer) as Box<dyn std::any::Any + Send + Sync>)
            }),
        });
    }

    /// Register a new layer implementation
    pub fn register_implementation(&mut self, implementation: LayerImplementation) {
        log::info!(
            "Registering layer implementation: {} v{}",
            implementation.name,
            implementation.version
        );
        self.implementation_registry
            .insert(implementation.name.clone(), implementation);
    }

    /// Configure a layer
    pub fn configure_layer(&mut self, layer_type: LayerType, config: LayerConfig) {
        self.layer_configs.insert(layer_type, config);
    }

    /// Create an execution layer
    pub async fn create_execution_layer(
        &self,
        data_context: &DataContext,
    ) -> Result<Arc<dyn ExecutionLayer>> {
        let config = self
            .layer_configs
            .get(&LayerType::Execution)
            .ok_or_else(|| failure::format_err!("Execution layer not configured"))?;

        let implementation = self
            .implementation_registry
            .get(&config.implementation)
            .ok_or_else(|| {
                failure::format_err!("Implementation not found: {}", config.implementation)
            })?;

        let layer_any = (implementation.factory)(config, data_context)?;

        // Try to downcast to the execution layer
        let layer = layer_any
            .downcast::<PolyTorusExecutionLayer>()
            .map_err(|_| failure::format_err!("Failed to downcast to execution layer"))?;

        // Register with message bus
        let layer_info = LayerInfo {
            layer_type: LayerType::Execution,
            layer_id: format!("{}-{}", implementation.name, uuid::Uuid::new_v4()),
            capabilities: implementation.capabilities.clone(),
            health_status: HealthStatus::Healthy,
            message_handler: None, // Could add message handler here
        };

        self.message_bus.register_layer(layer_info).await?;

        Ok(Arc::new(*layer) as Arc<dyn ExecutionLayer>)
    }

    /// Create a consensus layer
    pub async fn create_consensus_layer(
        &self,
        data_context: &DataContext,
    ) -> Result<Arc<dyn ConsensusLayer>> {
        let config = self
            .layer_configs
            .get(&LayerType::Consensus)
            .ok_or_else(|| failure::format_err!("Consensus layer not configured"))?;

        let implementation = self
            .implementation_registry
            .get(&config.implementation)
            .ok_or_else(|| {
                failure::format_err!("Implementation not found: {}", config.implementation)
            })?;

        let layer_any = (implementation.factory)(config, data_context)?;

        let layer = layer_any
            .downcast::<PolyTorusConsensusLayer>()
            .map_err(|_| failure::format_err!("Failed to downcast to consensus layer"))?;

        // Register with message bus
        let layer_info = LayerInfo {
            layer_type: LayerType::Consensus,
            layer_id: format!("{}-{}", implementation.name, uuid::Uuid::new_v4()),
            capabilities: implementation.capabilities.clone(),
            health_status: HealthStatus::Healthy,
            message_handler: None,
        };

        self.message_bus.register_layer(layer_info).await?;

        Ok(Arc::new(*layer) as Arc<dyn ConsensusLayer>)
    }

    /// Create a settlement layer
    pub async fn create_settlement_layer(&self) -> Result<Arc<dyn SettlementLayer>> {
        let config = self
            .layer_configs
            .get(&LayerType::Settlement)
            .ok_or_else(|| failure::format_err!("Settlement layer not configured"))?;

        let implementation = self
            .implementation_registry
            .get(&config.implementation)
            .ok_or_else(|| {
                failure::format_err!("Implementation not found: {}", config.implementation)
            })?;

        // For settlement layer, we don't need data_context
        let data_context = DataContext::default();
        let layer_any = (implementation.factory)(config, &data_context)?;

        let layer = layer_any
            .downcast::<PolyTorusSettlementLayer>()
            .map_err(|_| failure::format_err!("Failed to downcast to settlement layer"))?;

        // Register with message bus
        let layer_info = LayerInfo {
            layer_type: LayerType::Settlement,
            layer_id: format!("{}-{}", implementation.name, uuid::Uuid::new_v4()),
            capabilities: implementation.capabilities.clone(),
            health_status: HealthStatus::Healthy,
            message_handler: None,
        };

        self.message_bus.register_layer(layer_info).await?;

        Ok(Arc::new(*layer) as Arc<dyn SettlementLayer>)
    }

    /// Create a data availability layer
    pub async fn create_data_availability_layer(&self) -> Result<Arc<dyn DataAvailabilityLayer>> {
        let config = self
            .layer_configs
            .get(&LayerType::DataAvailability)
            .ok_or_else(|| failure::format_err!("Data availability layer not configured"))?;

        let implementation = self
            .implementation_registry
            .get(&config.implementation)
            .ok_or_else(|| {
                failure::format_err!("Implementation not found: {}", config.implementation)
            })?;

        let data_context = DataContext::default();
        let layer_any = (implementation.factory)(config, &data_context)?;

        let layer = layer_any
            .downcast::<PolyTorusDataAvailabilityLayer>()
            .map_err(|_| failure::format_err!("Failed to downcast to data availability layer"))?;

        // Register with message bus
        let layer_info = LayerInfo {
            layer_type: LayerType::DataAvailability,
            layer_id: format!("{}-{}", implementation.name, uuid::Uuid::new_v4()),
            capabilities: implementation.capabilities.clone(),
            health_status: HealthStatus::Healthy,
            message_handler: None,
        };

        self.message_bus.register_layer(layer_info).await?;

        Ok(Arc::new(*layer) as Arc<dyn DataAvailabilityLayer>)
    }

    /// Get available implementations for a layer type
    pub fn get_available_implementations(
        &self,
        layer_type: &LayerType,
    ) -> Vec<&LayerImplementation> {
        self.implementation_registry
            .values()
            .filter(|impl_| {
                // Filter implementations based on capabilities or layer type
                match layer_type {
                    LayerType::Execution => {
                        impl_.capabilities.contains(&"wasm-execution".to_string())
                    }
                    LayerType::Consensus => {
                        impl_.capabilities.contains(&"block-validation".to_string())
                    }
                    LayerType::Settlement => {
                        impl_.capabilities.contains(&"batch-settlement".to_string())
                    }
                    LayerType::DataAvailability => {
                        impl_.capabilities.contains(&"p2p-storage".to_string())
                    }
                    _ => false,
                }
            })
            .collect()
    }

    /// Validate layer configuration
    pub fn validate_configuration(
        &self,
        layer_type: &LayerType,
        config: &LayerConfig,
    ) -> Result<()> {
        // Check if implementation exists
        if !self
            .implementation_registry
            .contains_key(&config.implementation)
        {
            return Err(failure::format_err!(
                "Implementation not found: {}",
                config.implementation
            ));
        }

        // Check dependencies
        for dependency in &config.dependencies {
            if !self.layer_configs.contains_key(dependency) {
                return Err(failure::format_err!(
                    "Dependency layer not configured: {:?}",
                    dependency
                ));
            }
        }

        log::debug!("Configuration validated for layer {:?}", layer_type);
        Ok(())
    }

    /// Load configuration from enhanced config
    pub fn load_configuration(&mut self, config: &EnhancedModularConfig) -> Result<()> {
        for (layer_type, layer_config) in &config.layers {
            // Validate configuration
            self.validate_configuration(layer_type, layer_config)?;

            // Configure layer
            self.configure_layer(layer_type.clone(), layer_config.clone());
        }

        log::info!("Loaded configuration for {} layers", config.layers.len());
        Ok(())
    }
}

/// Helper function to create default enhanced configuration
pub fn create_default_enhanced_config() -> EnhancedModularConfig {
    let mut layers = HashMap::new();

    // Execution layer config
    layers.insert(
        LayerType::Execution,
        LayerConfig {
            implementation: "polytorus-execution".to_string(),
            config: serde_json::to_value(ExecutionConfig {
                gas_limit: 8_000_000,
                gas_price: 1,
                wasm_config: WasmConfig {
                    max_memory_pages: 256,
                    max_stack_size: 65536,
                    gas_metering: true,
                },
            })
            .unwrap(),
            enabled: true,
            priority: 1,
            dependencies: vec![],
        },
    );

    // Consensus layer config
    layers.insert(
        LayerType::Consensus,
        LayerConfig {
            implementation: "polytorus-consensus".to_string(),
            config: serde_json::to_value(ConsensusConfig {
                block_time: 10000,
                difficulty: 4,
                max_block_size: 1024 * 1024,
            })
            .unwrap(),
            enabled: true,
            priority: 1,
            dependencies: vec![],
        },
    );

    // Settlement layer config
    layers.insert(
        LayerType::Settlement,
        LayerConfig {
            implementation: "polytorus-settlement".to_string(),
            config: serde_json::to_value(SettlementConfig {
                challenge_period: 100,
                batch_size: 100,
                min_validator_stake: 1000,
            })
            .unwrap(),
            enabled: true,
            priority: 2,
            dependencies: vec![LayerType::Execution],
        },
    );

    // Data availability layer config
    layers.insert(
        LayerType::DataAvailability,
        LayerConfig {
            implementation: "polytorus-data-availability".to_string(),
            config: serde_json::to_value(DataAvailabilityConfig {
                network_config: NetworkConfig {
                    listen_addr: "0.0.0.0:7000".to_string(),
                    bootstrap_peers: Vec::new(),
                    max_peers: 50,
                },
                retention_period: 86400 * 7,
                max_data_size: 1024 * 1024,
            })
            .unwrap(),
            enabled: true,
            priority: 3,
            dependencies: vec![],
        },
    );

    EnhancedModularConfig {
        layers,
        global: GlobalConfig {
            network_mode: "devnet".to_string(),
            log_level: "info".to_string(),
            performance_mode: PerformanceMode::Development,
            features: HashMap::new(),
        },
        plugins: HashMap::new(),
    }
}
