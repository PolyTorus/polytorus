//! Diamond IO Layer Implementation
//!
//! This layer provides Diamond IO cryptographic operations integration.

use std::{collections::HashMap, sync::Arc};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use tracing::{error, info, warn};

use crate::{
    diamond_io_integration_new::{PrivacyEngineConfig, PrivacyEngineIntegration},
    modular::{
        message_bus::MessageBus,
        traits::{Layer, LayerMessage},
    },
};

/// Diamond IO Layer message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiamondIOMessage {
    CircuitCreation {
        circuit_id: String,
        description: String,
    },
    DataEncryption {
        data: Vec<bool>,
        requester: String,
    },
    DataDecryption {
        encrypted_data: Vec<u8>,
        requester: String,
    },
    ConfigUpdate {
        config: PrivacyEngineConfig,
    },
}

impl LayerMessage for DiamondIOMessage {
    fn message_type(&self) -> String {
        match self {
            DiamondIOMessage::CircuitCreation { .. } => "CircuitCreation".to_string(),
            DiamondIOMessage::DataEncryption { .. } => "DataEncryption".to_string(),
            DiamondIOMessage::DataDecryption { .. } => "DataDecryption".to_string(),
            DiamondIOMessage::ConfigUpdate { .. } => "ConfigUpdate".to_string(),
        }
    }
}

/// Diamond IO Layer configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiamondIOLayerConfig {
    pub diamond_config: PrivacyEngineConfig,
    pub max_concurrent_operations: usize,
    pub enable_encryption: bool,
    pub enable_decryption: bool,
}

impl Default for DiamondIOLayerConfig {
    fn default() -> Self {
        Self {
            diamond_config: PrivacyEngineConfig::testing(),
            max_concurrent_operations: 10,
            enable_encryption: true,
            enable_decryption: true,
        }
    }
}

/// Statistics for Diamond IO operations
#[derive(Debug, Clone, Default)]
pub struct DiamondIOStats {
    pub circuits_created: u64,
    pub data_encrypted: u64,
    pub data_decrypted: u64,
    pub total_operations: u64,
    pub failed_operations: u64,
}

/// Diamond IO Layer implementation
pub struct DiamondIOLayer {
    config: DiamondIOLayerConfig,
    integration: Arc<RwLock<Option<PrivacyEngineIntegration>>>,
    message_bus: Arc<MessageBus>,
    stats: Arc<RwLock<DiamondIOStats>>,
    active_operations: Arc<RwLock<HashMap<String, tokio::task::JoinHandle<()>>>>,
}

impl DiamondIOLayer {
    /// Create a new Diamond IO layer
    pub fn new(config: DiamondIOLayerConfig, message_bus: Arc<MessageBus>) -> Self {
        Self {
            config,
            integration: Arc::new(RwLock::new(None)),
            message_bus,
            stats: Arc::new(RwLock::new(DiamondIOStats::default())),
            active_operations: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    /// Initialize the Diamond IO integration
    pub async fn initialize(&self) -> Result<()> {
        let integration = PrivacyEngineIntegration::new(self.config.diamond_config.clone())?;
        let mut integration_guard = self.integration.write().await;
        *integration_guard = Some(integration);
        info!("Diamond IO Layer initialized");
        Ok(())
    }

    /// Create a demo circuit
    pub async fn create_demo_circuit(&self, circuit_id: String, description: String) -> Result<()> {
        let integration_guard = self.integration.read().await;
        if let Some(ref integration) = *integration_guard {
            let _circuit = integration.create_demo_circuit();

            // Update stats
            let mut stats = self.stats.write().await;
            stats.circuits_created += 1;
            stats.total_operations += 1;

            info!("Created demo circuit: {} - {}", circuit_id, description);
            Ok(())
        } else {
            error!("Diamond IO integration not initialized");
            Err(anyhow::anyhow!("Diamond IO integration not initialized"))
        }
    }

    /// Encrypt data
    pub async fn encrypt_data(&self, data: Vec<bool>, _requester: String) -> Result<Vec<u8>> {
        if !self.config.enable_encryption {
            return Err(anyhow::anyhow!("Encryption is disabled"));
        }

        let integration_guard = self.integration.read().await;
        if let Some(ref integration) = *integration_guard {
            match integration.encrypt_data(&data) {
                Ok(encrypted) => {
                    // Update stats
                    let mut stats = self.stats.write().await;
                    stats.data_encrypted += 1;
                    stats.total_operations += 1;

                    info!("Encrypted data of size: {}", data.len());
                    Ok(encrypted)
                }
                Err(e) => {
                    let mut stats = self.stats.write().await;
                    stats.failed_operations += 1;
                    error!("Failed to encrypt data: {}", e);
                    Err(e)
                }
            }
        } else {
            error!("Diamond IO integration not initialized");
            Err(anyhow::anyhow!("Diamond IO integration not initialized"))
        }
    }

    /// Update configuration
    pub async fn update_config(&mut self, config: PrivacyEngineConfig) -> Result<()> {
        self.config.diamond_config = config.clone();

        // Reinitialize the integration with new config
        let integration = PrivacyEngineIntegration::new(config)?;
        let mut integration_guard = self.integration.write().await;
        *integration_guard = Some(integration);

        info!("Updated Diamond IO configuration");
        Ok(())
    }

    /// Get layer statistics
    pub async fn get_stats(&self) -> DiamondIOStats {
        let stats = self.stats.read().await;
        stats.clone()
    }

    /// Handle Diamond IO messages
    async fn handle_message(&self, message: DiamondIOMessage) -> Result<()> {
        match message {
            DiamondIOMessage::CircuitCreation {
                circuit_id,
                description,
            } => {
                self.create_demo_circuit(circuit_id, description).await?;
            }
            DiamondIOMessage::DataEncryption { data, requester } => {
                let _ = self.encrypt_data(data, requester).await?;
            }
            DiamondIOMessage::DataDecryption {
                encrypted_data: _,
                requester: _,
            } => {
                // Decryption not implemented in current integration
                warn!("Decryption not yet implemented");
            }
            DiamondIOMessage::ConfigUpdate { config } => {
                // Note: This would require &mut self, so we'll log it for now
                info!("Config update requested: {:?}", config);
            }
        }
        Ok(())
    }

    /// Get current configuration
    pub fn get_config(&self) -> &DiamondIOLayerConfig {
        &self.config
    }

    /// Clean up completed operations
    pub async fn cleanup_operations(&self) {
        let mut operations = self.active_operations.write().await;
        operations.retain(|_, handle| !handle.is_finished());
    }
}

#[async_trait::async_trait]
impl Layer for DiamondIOLayer {
    type Config = DiamondIOLayerConfig;
    type Message = DiamondIOMessage;

    async fn start(&mut self) -> Result<()> {
        info!("Starting Diamond IO Layer");

        // Initialize the integration
        self.initialize().await?;

        info!("Diamond IO Layer started successfully");
        Ok(())
    }

    async fn stop(&mut self) -> Result<()> {
        info!("Stopping Diamond IO Layer");

        // Cancel all active operations
        let mut operations = self.active_operations.write().await;
        for (_, handle) in operations.drain() {
            handle.abort();
        }

        // Clear integration
        let mut integration_guard = self.integration.write().await;
        *integration_guard = None;

        info!("Diamond IO Layer stopped");
        Ok(())
    }

    async fn process_message(&mut self, message: Self::Message) -> Result<()> {
        self.handle_message(message).await
    }

    fn get_layer_type(&self) -> String {
        "diamond_io".to_string()
    }
}

// Need to implement Clone for the Layer trait
impl Clone for DiamondIOLayer {
    fn clone(&self) -> Self {
        Self {
            config: self.config.clone(),
            integration: self.integration.clone(),
            message_bus: self.message_bus.clone(),
            stats: self.stats.clone(),
            active_operations: self.active_operations.clone(),
        }
    }
}

/// Diamond IO Layer factory
pub struct DiamondIOLayerFactory;

impl DiamondIOLayerFactory {
    pub fn create(config: DiamondIOLayerConfig, message_bus: Arc<MessageBus>) -> DiamondIOLayer {
        DiamondIOLayer::new(config, message_bus)
    }
}

#[cfg(test)]
mod tests {
    use tokio;

    use super::*;

    #[tokio::test]
    async fn test_diamond_io_layer_creation() {
        let config = DiamondIOLayerConfig::default();
        let message_bus = Arc::new(MessageBus::new());
        let layer = DiamondIOLayer::new(config, message_bus);

        assert_eq!(layer.get_layer_type(), "diamond_io");
    }

    #[tokio::test]
    async fn test_layer_initialization() {
        let config = DiamondIOLayerConfig::default();
        let message_bus = Arc::new(MessageBus::new());
        let layer = DiamondIOLayer::new(config, message_bus);

        let result = layer.initialize().await;
        assert!(result.is_ok());
    }
}
