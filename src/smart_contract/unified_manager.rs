//! Unified Smart Contract Manager
//!
//! This module provides a single entry point for all smart contract operations,
//! routing requests to appropriate engines based on contract type.

use std::{collections::HashMap, sync::Arc};

use anyhow::Result;
use tokio::sync::RwLock;

use super::{
    privacy_engine::PrivacyContractEngine,
    unified_engine::{
        ContractExecutionRecord, ContractStateStorage, ContractType, EngineInfo,
        UnifiedContractEngine, UnifiedContractExecution, UnifiedContractMetadata,
        UnifiedContractResult, UnifiedGasManager,
    },
    unified_storage::UnifiedContractStorage,
    wasm_engine::WasmContractEngine,
};
use crate::diamond_io_integration::PrivacyEngineConfig;

/// Unified smart contract manager that routes operations to appropriate engines
pub struct UnifiedContractManager {
    storage: Arc<dyn ContractStateStorage>,
    gas_manager: UnifiedGasManager,
    wasm_engine: Arc<RwLock<WasmContractEngine>>,
    privacy_engine: Arc<RwLock<PrivacyContractEngine>>,
    active_contracts: Arc<RwLock<HashMap<String, ContractType>>>,
}

impl UnifiedContractManager {
    /// Create a new unified contract manager
    pub fn new(
        storage: Arc<dyn ContractStateStorage>,
        gas_manager: UnifiedGasManager,
        privacy_config: PrivacyEngineConfig,
    ) -> Result<Self> {
        let wasm_engine = Arc::new(RwLock::new(WasmContractEngine::new(
            Arc::clone(&storage),
            gas_manager.clone(),
        )?));

        let privacy_engine = Arc::new(RwLock::new(PrivacyContractEngine::new(
            Arc::clone(&storage),
            gas_manager.clone(),
            privacy_config,
        )?));

        Ok(Self {
            storage,
            gas_manager,
            wasm_engine,
            privacy_engine,
            active_contracts: Arc::new(RwLock::new(HashMap::new())),
        })
    }

    /// Create a manager with default configuration
    pub fn with_defaults(storage_path: &str) -> Result<Self> {
        let storage = Arc::new(UnifiedContractStorage::new(storage_path)?);
        let gas_manager = UnifiedGasManager::new(Default::default());
        let privacy_config = PrivacyEngineConfig::dummy(); // Safe default

        Self::new(storage, gas_manager, privacy_config)
    }

    /// Create an in-memory manager for testing
    pub fn in_memory() -> Result<Self> {
        let storage = Arc::new(super::unified_storage::SyncInMemoryContractStorage::new());
        let gas_manager = UnifiedGasManager::new(Default::default());
        let privacy_config = PrivacyEngineConfig::dummy();

        Self::new(storage, gas_manager, privacy_config)
    }

    /// Deploy a contract using the appropriate engine
    pub async fn deploy_contract(
        &self,
        metadata: UnifiedContractMetadata,
        init_data: Vec<u8>,
    ) -> Result<String> {
        let contract_type = metadata.contract_type.clone();
        let contract_address = metadata.address.clone();

        let result = match &contract_type {
            ContractType::Wasm { .. } | ContractType::BuiltIn { .. } => {
                let mut engine = self.wasm_engine.write().await;
                engine.deploy_contract(metadata, init_data)
            }
            ContractType::PrivacyEnhanced { .. } => {
                let mut engine = self.privacy_engine.write().await;
                engine.deploy_contract(metadata, init_data)
            }
        };

        // Cache the contract type for routing
        if result.is_ok() {
            let mut contracts = self.active_contracts.write().await;
            contracts.insert(contract_address.clone(), contract_type);
        }

        result
    }

    /// Execute a contract function using the appropriate engine
    pub async fn execute_contract(
        &self,
        execution: UnifiedContractExecution,
    ) -> Result<UnifiedContractResult> {
        // Determine the engine to use
        let contract_type = self.get_contract_type(&execution.contract_address).await?;

        match contract_type {
            ContractType::Wasm { .. } | ContractType::BuiltIn { .. } => {
                let mut engine = self.wasm_engine.write().await;
                engine.execute_contract(execution)
            }
            ContractType::PrivacyEnhanced { .. } => {
                let mut engine = self.privacy_engine.write().await;
                engine.execute_contract(execution)
            }
        }
    }

    /// Get contract metadata
    pub async fn get_contract(&self, address: &str) -> Result<Option<UnifiedContractMetadata>> {
        self.storage.get_contract_metadata(address)
    }

    /// Get contract state
    pub async fn get_contract_state(&self, contract: &str, key: &str) -> Result<Option<Vec<u8>>> {
        self.storage.get_contract_state(contract, key)
    }

    /// List all contracts
    pub async fn list_contracts(&self) -> Result<Vec<String>> {
        self.storage.list_contracts()
    }

    /// List contracts by type
    pub async fn list_contracts_by_type(&self, contract_type: &str) -> Result<Vec<String>> {
        let all_contracts = self.storage.list_contracts()?;
        let mut filtered_contracts = Vec::new();

        for address in all_contracts {
            if let Ok(Some(metadata)) = self.storage.get_contract_metadata(&address) {
                let matches = matches!(
                    (&metadata.contract_type, contract_type),
                    (ContractType::Wasm { .. }, "wasm")
                        | (ContractType::BuiltIn { .. }, "builtin")
                        | (ContractType::PrivacyEnhanced { .. }, "privacy")
                );

                if matches {
                    filtered_contracts.push(address);
                }
            }
        }

        Ok(filtered_contracts)
    }

    /// Estimate gas for contract execution
    pub async fn estimate_gas(&self, execution: &UnifiedContractExecution) -> Result<u64> {
        // Try to determine contract type for accurate estimation
        if let Ok(contract_type) = self.get_contract_type(&execution.contract_address).await {
            match contract_type {
                ContractType::Wasm { .. } | ContractType::BuiltIn { .. } => {
                    let engine = self.wasm_engine.read().await;
                    engine.estimate_gas(execution)
                }
                ContractType::PrivacyEnhanced { .. } => {
                    let engine = self.privacy_engine.read().await;
                    engine.estimate_gas(execution)
                }
            }
        } else {
            // Fallback to base gas calculation
            Ok(self.gas_manager.calculate_base_gas(execution))
        }
    }

    /// Get execution history for a contract
    pub async fn get_execution_history(
        &self,
        contract: &str,
    ) -> Result<Vec<ContractExecutionRecord>> {
        self.storage.get_execution_history(contract)
    }

    /// Get information about all available engines
    pub async fn get_engine_info(&self) -> Vec<EngineInfo> {
        let wasm_info = {
            let engine = self.wasm_engine.read().await;
            engine.engine_info()
        };

        let privacy_info = {
            let engine = self.privacy_engine.read().await;
            engine.engine_info()
        };

        vec![wasm_info, privacy_info]
    }

    /// Get manager statistics
    pub async fn get_statistics(&self) -> Result<ManagerStatistics> {
        let total_contracts = self.storage.list_contracts()?.len();
        let mut wasm_contracts = 0;
        let mut privacy_contracts = 0;
        let mut builtin_contracts = 0;

        let contracts = self.active_contracts.read().await;
        for contract_type in contracts.values() {
            match contract_type {
                ContractType::Wasm { .. } => wasm_contracts += 1,
                ContractType::BuiltIn { .. } => builtin_contracts += 1,
                ContractType::PrivacyEnhanced { .. } => privacy_contracts += 1,
            }
        }

        Ok(ManagerStatistics {
            total_contracts,
            wasm_contracts,
            privacy_contracts,
            builtin_contracts,
            active_engines: 2, // WASM and Privacy engines
        })
    }

    /// Clean up inactive contracts from cache
    pub async fn cleanup_cache(&self) -> Result<usize> {
        let mut contracts = self.active_contracts.write().await;
        let all_stored = self.storage.list_contracts()?;

        // Remove contracts from cache that no longer exist in storage
        let mut removed = 0;
        contracts.retain(|address, _| {
            let exists = all_stored.contains(address);
            if !exists {
                removed += 1;
            }
            exists
        });

        Ok(removed)
    }

    /// Get contract type, loading from storage if necessary
    async fn get_contract_type(&self, address: &str) -> Result<ContractType> {
        // Check cache first
        {
            let contracts = self.active_contracts.read().await;
            if let Some(contract_type) = contracts.get(address) {
                return Ok(contract_type.clone());
            }
        }

        // Load from storage
        if let Some(metadata) = self.storage.get_contract_metadata(address)? {
            let contract_type = metadata.contract_type.clone();

            // Cache for future use
            {
                let mut contracts = self.active_contracts.write().await;
                contracts.insert(address.to_string(), contract_type.clone());
            }

            Ok(contract_type)
        } else {
            Err(anyhow::anyhow!("Contract not found: {}", address))
        }
    }

    /// Deploy an ERC20 token (convenience method)
    pub async fn deploy_erc20(
        &self,
        name: String,
        symbol: String,
        decimals: u8,
        initial_supply: u64,
        owner: String,
        contract_address: String,
    ) -> Result<String> {
        let mut parameters = HashMap::new();
        parameters.insert("name".to_string(), name.clone());
        parameters.insert("symbol".to_string(), symbol.clone());
        parameters.insert("decimals".to_string(), decimals.to_string());
        parameters.insert("initial_supply".to_string(), initial_supply.to_string());

        let metadata = UnifiedContractMetadata {
            address: contract_address.clone(),
            name: format!("ERC20: {}", name),
            description: format!("ERC20 token {} ({})", name, symbol),
            contract_type: ContractType::BuiltIn {
                contract_name: "ERC20".to_string(),
                parameters,
            },
            deployment_tx: uuid::Uuid::new_v4().to_string(),
            deployment_time: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            owner,
            is_active: true,
        };

        self.deploy_contract(metadata, Vec::new()).await
    }

    /// Deploy a privacy-enhanced contract (convenience method)
    pub async fn deploy_privacy_contract(
        &self,
        name: String,
        description: String,
        circuit_id: String,
        owner: String,
        contract_address: String,
        circuit_description: Vec<u8>,
    ) -> Result<String> {
        let metadata = UnifiedContractMetadata {
            address: contract_address.clone(),
            name,
            description,
            contract_type: ContractType::PrivacyEnhanced {
                circuit_id,
                obfuscated: false,
            },
            deployment_tx: uuid::Uuid::new_v4().to_string(),
            deployment_time: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            owner,
            is_active: true,
        };

        self.deploy_contract(metadata, circuit_description).await
    }
}

/// Manager statistics
#[derive(Debug, Clone)]
pub struct ManagerStatistics {
    pub total_contracts: usize,
    pub wasm_contracts: usize,
    pub privacy_contracts: usize,
    pub builtin_contracts: usize,
    pub active_engines: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::smart_contract::unified_engine::UnifiedContractExecution;

    #[tokio::test]
    async fn test_manager_creation() {
        let manager = UnifiedContractManager::in_memory().unwrap();

        let engine_info = manager.get_engine_info().await;
        assert_eq!(engine_info.len(), 2); // WASM and Privacy engines

        let stats = manager.get_statistics().await.unwrap();
        assert_eq!(stats.total_contracts, 0);
        assert_eq!(stats.active_engines, 2);
    }

    #[tokio::test]
    async fn test_erc20_deployment() {
        let manager = UnifiedContractManager::in_memory().unwrap();

        let address = manager
            .deploy_erc20(
                "Test Token".to_string(),
                "TTK".to_string(),
                18,
                1000000,
                "0x1234567890".to_string(),
                "0xcontract123".to_string(),
            )
            .await
            .unwrap();

        assert_eq!(address, "0xcontract123");

        // Verify contract exists
        let metadata = manager.get_contract(&address).await.unwrap();
        assert!(metadata.is_some());

        let stats = manager.get_statistics().await.unwrap();
        assert_eq!(stats.builtin_contracts, 1);
    }

    #[tokio::test]
    async fn test_privacy_contract_deployment() {
        let manager = UnifiedContractManager::in_memory().unwrap();

        let address = manager
            .deploy_privacy_contract(
                "Privacy Contract".to_string(),
                "A privacy-enhanced contract".to_string(),
                "test_circuit".to_string(),
                "0x1234567890".to_string(),
                "0xprivacy123".to_string(),
                b"circuit description".to_vec(),
            )
            .await
            .unwrap();

        assert_eq!(address, "0xprivacy123");

        // Verify contract exists
        let metadata = manager.get_contract(&address).await.unwrap();
        assert!(metadata.is_some());

        let stats = manager.get_statistics().await.unwrap();
        assert_eq!(stats.privacy_contracts, 1);
    }

    #[tokio::test]
    async fn test_contract_execution() {
        let manager = UnifiedContractManager::in_memory().unwrap();

        // Deploy ERC20 contract
        let contract_address = manager
            .deploy_erc20(
                "Test Token".to_string(),
                "TTK".to_string(),
                18,
                1000000,
                "0x1234567890".to_string(),
                "0xcontract123".to_string(),
            )
            .await
            .unwrap();

        // Execute balance_of function
        let mut input_data = vec![0u8; 32];
        input_data[..11].copy_from_slice(b"0x123456789");

        let execution = UnifiedContractExecution {
            contract_address,
            function_name: "balance_of".to_string(),
            input_data,
            caller: "0x1234567890".to_string(),
            value: 0,
            gas_limit: 100000,
        };

        let result = manager.execute_contract(execution).await.unwrap();
        assert!(result.success);
    }

    #[tokio::test]
    async fn test_gas_estimation() {
        let manager = UnifiedContractManager::in_memory().unwrap();

        let execution = UnifiedContractExecution {
            contract_address: "0xcontract123".to_string(),
            function_name: "transfer".to_string(),
            input_data: vec![0; 40],
            caller: "0x1234567890".to_string(),
            value: 0,
            gas_limit: 100000,
        };

        let estimated_gas = manager.estimate_gas(&execution).await.unwrap();
        assert!(estimated_gas > 0);
    }

    #[tokio::test]
    async fn test_contract_listing() {
        let manager = UnifiedContractManager::in_memory().unwrap();

        // Deploy multiple contracts
        manager
            .deploy_erc20(
                "Token A".to_string(),
                "TKA".to_string(),
                18,
                1000000,
                "0x1111".to_string(),
                "0xcontract1".to_string(),
            )
            .await
            .unwrap();

        manager
            .deploy_privacy_contract(
                "Privacy A".to_string(),
                "Privacy contract A".to_string(),
                "circuit_a".to_string(),
                "0x2222".to_string(),
                "0xcontract2".to_string(),
                b"circuit desc".to_vec(),
            )
            .await
            .unwrap();

        // Test listing by type
        let builtin_contracts = manager.list_contracts_by_type("builtin").await.unwrap();
        assert_eq!(builtin_contracts.len(), 1);

        let privacy_contracts = manager.list_contracts_by_type("privacy").await.unwrap();
        assert_eq!(privacy_contracts.len(), 1);

        // Test listing all contracts
        let all_contracts = manager.list_contracts().await.unwrap();
        assert_eq!(all_contracts.len(), 2);
    }
}
