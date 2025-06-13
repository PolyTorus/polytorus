use crate::diamond_io_integration::DiamondIOConfig;
use crate::diamond_smart_contracts::{DiamondContractEngine, DiamondContract, ContractExecution};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;
use anyhow::Result;
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiamondIOMessage {
    ContractDeployment {
        contract_id: String,
        owner: String,
        circuit_description: String,
    },
    ContractExecution {
        contract_id: String,
        inputs: Vec<bool>,
        executor: String,
    },
    ObfuscationRequest {
        contract_id: String,
    },
    EncryptionRequest {
        data: Vec<bool>,
        requester: String,
    },
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiamondIOLayerConfig {
    pub diamond_config: DiamondIOConfig,
    pub max_concurrent_executions: usize,
    pub obfuscation_enabled: bool,
    pub encryption_enabled: bool,
    pub gas_limit_per_execution: u64,
}

impl Default for DiamondIOLayerConfig {
    fn default() -> Self {
        Self {
            diamond_config: DiamondIOConfig::default(),
            max_concurrent_executions: 10,
            obfuscation_enabled: true,
            encryption_enabled: true,
            gas_limit_per_execution: 1_000_000,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiamondIOLayerStats {
    pub total_contracts: usize,
    pub obfuscated_contracts: usize,
    pub total_executions: u64,
    pub successful_executions: u64,
    pub failed_executions: u64,
    pub total_gas_used: u64,
    pub average_execution_time_ms: u64,
    pub active_executions: usize,
}

impl Default for DiamondIOLayerStats {
    fn default() -> Self {
        Self {
            total_contracts: 0,
            obfuscated_contracts: 0,
            total_executions: 0,
            successful_executions: 0,
            failed_executions: 0,
            total_gas_used: 0,
            average_execution_time_ms: 0,
            active_executions: 0,
        }
    }
}

pub struct PolyTorusDiamondIOLayer {
    config: DiamondIOLayerConfig,
    contract_engine: RwLock<DiamondContractEngine>,
    stats: RwLock<DiamondIOLayerStats>,
    message_handlers: HashMap<String, Box<dyn Fn(&DiamondIOMessage) -> Result<()> + Send + Sync>>,
}

impl std::fmt::Debug for PolyTorusDiamondIOLayer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("PolyTorusDiamondIOLayer")
            .field("config", &self.config)
            .field("contract_engine", &"<RwLock<DiamondContractEngine>>")
            .field("stats", &self.stats)
            .field("message_handlers", &format!("{} handlers", self.message_handlers.len()))
            .finish()
    }
}

impl PolyTorusDiamondIOLayer {
    pub fn new(config: DiamondIOLayerConfig) -> Result<Self> {
        let contract_engine = DiamondContractEngine::new(config.diamond_config.clone())?;
        
        Ok(Self {
            config,
            contract_engine: RwLock::new(contract_engine),
            stats: RwLock::new(DiamondIOLayerStats::default()),
            message_handlers: HashMap::new(),
        })
    }

    pub async fn deploy_contract(
        &self,
        contract_id: String,
        name: String,
        description: String,
        owner: String,
        circuit_description: &str,
    ) -> Result<String> {
        info!("Deploying Diamond contract: {} by {}", name, owner);
        
        let mut engine = self.contract_engine.write().await;
        let result = engine.deploy_contract(
            contract_id.clone(),
            name,
            description,
            owner,
            circuit_description,
        ).await;

        if result.is_ok() {
            let mut stats = self.stats.write().await;
            stats.total_contracts += 1;
        }

        result
    }

    pub async fn obfuscate_contract(&self, contract_id: &str) -> Result<()> {
        info!("Obfuscating contract: {}", contract_id);
        
        if !self.config.obfuscation_enabled {
            return Err(anyhow::anyhow!("Obfuscation is disabled"));
        }

        let mut engine = self.contract_engine.write().await;
        let result = engine.obfuscate_contract(contract_id).await;

        if result.is_ok() {
            let mut stats = self.stats.write().await;
            stats.obfuscated_contracts += 1;
        }

        result
    }

    pub async fn execute_contract(
        &self,
        contract_id: &str,
        inputs: Vec<bool>,
        executor: String,
    ) -> Result<Vec<bool>> {
        info!("Executing contract: {} by {}", contract_id, executor);
        
        // Check concurrent execution limit
        {
            let stats = self.stats.read().await;
            if stats.active_executions >= self.config.max_concurrent_executions {
                return Err(anyhow::anyhow!("Maximum concurrent executions reached"));
            }
        }

        // Increment active executions
        {
            let mut stats = self.stats.write().await;
            stats.active_executions += 1;
            stats.total_executions += 1;
        }

        let start_time = std::time::Instant::now();
        let mut engine = self.contract_engine.write().await;
        let result = engine.execute_contract(contract_id, inputs, executor).await;
        let execution_time = start_time.elapsed().as_millis() as u64;

        // Update stats
        {
            let mut stats = self.stats.write().await;
            stats.active_executions -= 1;
            
            match &result {
                Ok(_) => {
                    stats.successful_executions += 1;
                    // Update average execution time
                    let total_time = stats.average_execution_time_ms * (stats.successful_executions - 1) + execution_time;
                    stats.average_execution_time_ms = total_time / stats.successful_executions;
                }
                Err(_) => {
                    stats.failed_executions += 1;
                }
            }
        }

        result
    }

    pub async fn get_contract(&self, contract_id: &str) -> Option<DiamondContract> {
        let engine = self.contract_engine.read().await;
        engine.get_contract(contract_id).cloned()
    }

    pub async fn list_contracts(&self) -> Vec<DiamondContract> {
        let engine = self.contract_engine.read().await;
        engine.list_contracts().into_iter().cloned().collect()
    }

    pub async fn get_execution_history(&self, contract_id: &str) -> Vec<ContractExecution> {
        let engine = self.contract_engine.read().await;
        engine.get_execution_history(contract_id).into_iter().cloned().collect()
    }

    pub async fn get_stats(&self) -> DiamondIOLayerStats {
        self.stats.read().await.clone()
    }

    pub async fn encrypt_data(&self, data: Vec<bool>) -> Result<String> {
        if !self.config.encryption_enabled {
            return Err(anyhow::anyhow!("Encryption is disabled"));
        }

        let engine = self.contract_engine.read().await;
        let encrypted = engine.encrypt_data(&data)?;
        
        Ok(encrypted)
    }
}

// Simple trait definitions for Diamond IO Layer
pub trait DiamondLayerTrait {
    fn start_layer(&mut self) -> impl std::future::Future<Output = Result<()>> + Send;
    fn stop_layer(&mut self) -> impl std::future::Future<Output = Result<()>> + Send;
    fn health_check(&self) -> impl std::future::Future<Output = Result<bool>> + Send;
    fn layer_type(&self) -> &'static str;
}

impl DiamondLayerTrait for PolyTorusDiamondIOLayer {
    fn start_layer(&mut self) -> impl std::future::Future<Output = Result<()>> + Send {
        async move {
            info!("Starting Diamond IO Layer");
            info!("Diamond IO Layer started successfully");
            Ok(())
        }
    }

    fn stop_layer(&mut self) -> impl std::future::Future<Output = Result<()>> + Send {
        async move {
            info!("Stopping Diamond IO Layer");
            info!("Diamond IO Layer stopped");
            Ok(())
        }
    }

    fn health_check(&self) -> impl std::future::Future<Output = Result<bool>> + Send {
        async move {
            let stats = self.get_stats().await;
            let failure_rate = if stats.total_executions > 0 {
                stats.failed_executions as f64 / stats.total_executions as f64
            } else {
                0.0
            };
            Ok(failure_rate < 0.5)
        }
    }

    fn layer_type(&self) -> &'static str {
        "diamond_io"
    }
}

// Builder pattern for Diamond IO Layer
#[derive(Debug)]
pub struct DiamondIOLayerBuilder {
    config: DiamondIOLayerConfig,
}

impl DiamondIOLayerBuilder {
    pub fn new() -> Self {
        Self {
            config: DiamondIOLayerConfig::default(),
        }
    }

    pub fn with_diamond_config(mut self, config: DiamondIOConfig) -> Self {
        self.config.diamond_config = config;
        self
    }

    pub fn with_max_concurrent_executions(mut self, max: usize) -> Self {
        self.config.max_concurrent_executions = max;
        self
    }

    pub fn with_obfuscation_enabled(mut self, enabled: bool) -> Self {
        self.config.obfuscation_enabled = enabled;
        self
    }

    pub fn with_encryption_enabled(mut self, enabled: bool) -> Self {
        self.config.encryption_enabled = enabled;
        self
    }

    pub fn with_gas_limit(mut self, limit: u64) -> Self {
        self.config.gas_limit_per_execution = limit;
        self
    }

    pub fn build(self) -> Result<PolyTorusDiamondIOLayer> {
        PolyTorusDiamondIOLayer::new(self.config)
    }
}

impl Default for DiamondIOLayerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_diamond_io_layer_creation() {
        let layer = DiamondIOLayerBuilder::new()
            .with_max_concurrent_executions(5)
            .build()
            .unwrap();

        assert_eq!(layer.config.max_concurrent_executions, 5);
    }

    #[tokio::test]
    async fn test_contract_deployment_and_execution() {
        // Create a test configuration with appropriate input size
        let mut test_config = DiamondIOConfig::dummy();
        test_config.input_size = 2; // Set input size to 2 for this test
        
        let layer = DiamondIOLayerBuilder::new()
            .with_diamond_config(test_config)
            .build()
            .unwrap();

        // Deploy a contract
        let contract_id = layer.deploy_contract(
            "test_and".to_string(),
            "Test AND Gate".to_string(),
            "and_gate".to_string(),
            "alice".to_string(),
            "and_gate",
        ).await.unwrap();

        // Execute the contract with 2 inputs as configured
        let result = layer.execute_contract(
            &contract_id,
            vec![true, false],
            "bob".to_string(),
        ).await.unwrap();

        assert_eq!(result, vec![false]);

        // Check stats
        let stats = layer.get_stats().await;
        assert_eq!(stats.total_contracts, 1);
        assert_eq!(stats.successful_executions, 1);
    }

    #[tokio::test]
    async fn test_health_check() {
        let layer = DiamondIOLayerBuilder::new().build().unwrap();
        let is_healthy = layer.health_check().await.unwrap();
        assert!(is_healthy);
    }
}
