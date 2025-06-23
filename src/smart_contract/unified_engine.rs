//! Unified Smart Contract Engine
//!
//! This module provides a unified interface for all smart contract execution engines,
//! eliminating duplication between WASM and Diamond IO contract engines.

use std::collections::HashMap;

use anyhow::Result;
use serde::{Deserialize, Serialize};

/// Unified smart contract execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedContractResult {
    pub success: bool,
    pub return_data: Vec<u8>,
    pub gas_used: u64,
    pub events: Vec<ContractEvent>,
    pub execution_time_ms: u64,
    pub error_message: Option<String>,
}

/// Unified contract event structure
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractEvent {
    pub contract_address: String,
    pub event_type: String,
    pub topics: Vec<String>,
    pub data: Vec<u8>,
    pub timestamp: u64,
}

/// Unified contract metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedContractMetadata {
    pub address: String,
    pub name: String,
    pub description: String,
    pub contract_type: ContractType,
    pub deployment_tx: String,
    pub deployment_time: u64,
    pub owner: String,
    pub is_active: bool,
}

/// Types of smart contracts supported
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContractType {
    /// Traditional WASM-based contracts
    Wasm {
        bytecode: Vec<u8>,
        abi: Option<String>,
    },
    /// Privacy-enhanced contracts using obfuscated circuits
    PrivacyEnhanced {
        circuit_id: String,
        obfuscated: bool,
    },
    /// Built-in contracts (ERC20, Governance, etc.)
    BuiltIn {
        contract_name: String,
        parameters: HashMap<String, String>,
    },
}

/// Unified contract execution request
#[derive(Debug, Clone)]
pub struct UnifiedContractExecution {
    pub contract_address: String,
    pub function_name: String,
    pub input_data: Vec<u8>,
    pub caller: String,
    pub value: u64,
    pub gas_limit: u64,
}

/// Unified gas configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnifiedGasConfig {
    /// Base gas cost for any contract call
    pub base_cost: u64,
    /// Gas cost per byte of input data
    pub data_cost_per_byte: u64,
    /// Gas cost for storage operations
    pub storage_cost: u64,
    /// Gas cost for memory allocation
    pub memory_cost_per_kb: u64,
    /// Gas cost for computational operations
    pub computation_multiplier: f64,
}

impl Default for UnifiedGasConfig {
    fn default() -> Self {
        Self {
            base_cost: 21000,
            data_cost_per_byte: 4,
            storage_cost: 20000,
            memory_cost_per_kb: 3,
            computation_multiplier: 1.0,
        }
    }
}

/// Trait for unified contract state storage
pub trait ContractStateStorage: Send + Sync {
    /// Store contract metadata
    fn store_contract_metadata(&self, metadata: &UnifiedContractMetadata) -> Result<()>;

    /// Get contract metadata
    fn get_contract_metadata(&self, address: &str) -> Result<Option<UnifiedContractMetadata>>;

    /// Set contract state key-value pair
    fn set_contract_state(&self, contract: &str, key: &str, value: &[u8]) -> Result<()>;

    /// Get contract state value
    fn get_contract_state(&self, contract: &str, key: &str) -> Result<Option<Vec<u8>>>;

    /// Delete contract state
    fn delete_contract_state(&self, contract: &str, key: &str) -> Result<()>;

    /// List all contracts
    fn list_contracts(&self) -> Result<Vec<String>>;

    /// Store execution history
    fn store_execution(&self, execution: &ContractExecutionRecord) -> Result<()>;

    /// Get execution history
    fn get_execution_history(&self, contract: &str) -> Result<Vec<ContractExecutionRecord>>;
}

/// Contract execution record for history tracking
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractExecutionRecord {
    pub execution_id: String,
    pub contract_address: String,
    pub function_name: String,
    pub caller: String,
    pub timestamp: u64,
    pub gas_used: u64,
    pub success: bool,
    pub error_message: Option<String>,
}

/// Unified smart contract execution engine trait
pub trait UnifiedContractEngine: Send + Sync {
    /// Deploy a new contract
    fn deploy_contract(
        &mut self,
        metadata: UnifiedContractMetadata,
        init_data: Vec<u8>,
    ) -> Result<String>;

    /// Execute contract function
    fn execute_contract(
        &mut self,
        execution: UnifiedContractExecution,
    ) -> Result<UnifiedContractResult>;

    /// Get contract metadata
    fn get_contract(&self, address: &str) -> Result<Option<UnifiedContractMetadata>>;

    /// Get contract state
    fn get_contract_state(&self, contract: &str, key: &str) -> Result<Option<Vec<u8>>>;

    /// List all deployed contracts
    fn list_contracts(&self) -> Result<Vec<String>>;

    /// Calculate gas cost for execution
    fn estimate_gas(&self, execution: &UnifiedContractExecution) -> Result<u64>;

    /// Get execution history
    fn get_execution_history(&self, contract: &str) -> Result<Vec<ContractExecutionRecord>>;

    /// Engine-specific information
    fn engine_info(&self) -> EngineInfo;
}

/// Information about the contract engine
#[derive(Debug, Clone)]
pub struct EngineInfo {
    pub name: String,
    pub version: String,
    pub supported_contract_types: Vec<String>,
    pub features: Vec<String>,
}

/// Unified gas manager for all contract types
#[derive(Clone)]
pub struct UnifiedGasManager {
    config: UnifiedGasConfig,
}

impl UnifiedGasManager {
    pub fn new(config: UnifiedGasConfig) -> Self {
        Self { config }
    }

    /// Calculate base gas cost for contract execution
    pub fn calculate_base_gas(&self, execution: &UnifiedContractExecution) -> u64 {
        let mut gas = self.config.base_cost;

        // Add gas for input data
        gas += execution.input_data.len() as u64 * self.config.data_cost_per_byte;

        gas
    }

    /// Calculate storage gas cost
    pub fn calculate_storage_gas(&self, key_size: usize, value_size: usize) -> u64 {
        self.config.storage_cost + (key_size + value_size) as u64 * self.config.data_cost_per_byte
    }

    /// Calculate memory gas cost
    pub fn calculate_memory_gas(&self, memory_kb: u64) -> u64 {
        memory_kb * self.config.memory_cost_per_kb
    }

    /// Calculate computation gas cost based on execution time
    pub fn calculate_computation_gas(&self, execution_time_ms: u64) -> u64 {
        (execution_time_ms as f64 * self.config.computation_multiplier) as u64
    }

    /// Get gas configuration
    pub fn config(&self) -> &UnifiedGasConfig {
        &self.config
    }

    /// Update gas configuration
    pub fn update_config(&mut self, config: UnifiedGasConfig) {
        self.config = config;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_unified_gas_manager() {
        let gas_manager = UnifiedGasManager::new(UnifiedGasConfig::default());

        let execution = UnifiedContractExecution {
            contract_address: "test_contract".to_string(),
            function_name: "test_function".to_string(),
            input_data: vec![0; 100], // 100 bytes
            caller: "test_caller".to_string(),
            value: 0,
            gas_limit: 1000000,
        };

        let base_gas = gas_manager.calculate_base_gas(&execution);
        assert_eq!(base_gas, 21000 + 100 * 4); // base + data cost

        let storage_gas = gas_manager.calculate_storage_gas(32, 64);
        assert_eq!(storage_gas, 20000 + 96 * 4); // storage + key/value cost

        let memory_gas = gas_manager.calculate_memory_gas(10);
        assert_eq!(memory_gas, 10 * 3); // memory cost
    }

    #[test]
    fn test_contract_metadata_serialization() {
        let metadata = UnifiedContractMetadata {
            address: "0x1234".to_string(),
            name: "Test Contract".to_string(),
            description: "A test contract".to_string(),
            contract_type: ContractType::Wasm {
                bytecode: vec![1, 2, 3, 4],
                abi: Some("test_abi".to_string()),
            },
            deployment_tx: "0xabcd".to_string(),
            deployment_time: 1234567890,
            owner: "0x5678".to_string(),
            is_active: true,
        };

        let serialized = serde_json::to_string(&metadata).unwrap();
        let deserialized: UnifiedContractMetadata = serde_json::from_str(&serialized).unwrap();

        assert_eq!(metadata.address, deserialized.address);
        assert_eq!(metadata.name, deserialized.name);
        assert_eq!(metadata.is_active, deserialized.is_active);
    }

    #[test]
    fn test_contract_event() {
        let event = ContractEvent {
            contract_address: "0x1234".to_string(),
            event_type: "Transfer".to_string(),
            topics: vec!["from".to_string(), "to".to_string()],
            data: vec![1, 2, 3, 4],
            timestamp: 1234567890,
        };

        let serialized = serde_json::to_string(&event).unwrap();
        let deserialized: ContractEvent = serde_json::from_str(&serialized).unwrap();

        assert_eq!(event.contract_address, deserialized.contract_address);
        assert_eq!(event.event_type, deserialized.event_type);
        assert_eq!(event.topics.len(), deserialized.topics.len());
    }
}
