//! WASM Contract Engine implementing the unified interface
//!
//! This module adapts the existing WASM execution engine to work with the unified interface.

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
    time::Instant,
};

use anyhow::Result;
use uuid::Uuid;
use wasmtime::*;

use super::{
    erc20::ERC20Contract,
    types::GasConfig,
    unified_engine::{
        ContractEvent, ContractExecutionRecord, ContractStateStorage, ContractType, EngineInfo,
        UnifiedContractEngine, UnifiedContractExecution, UnifiedContractMetadata,
        UnifiedContractResult, UnifiedGasManager,
    },
};

/// WASM contract execution engine implementing unified interface
pub struct WasmContractEngine {
    #[allow(dead_code)]
    engine: Engine,
    storage: Arc<dyn ContractStateStorage>,
    gas_manager: UnifiedGasManager,
    erc20_contracts: Arc<Mutex<HashMap<String, ERC20Contract>>>,
    #[allow(dead_code)]
    gas_config: GasConfig,
}

impl WasmContractEngine {
    /// Create a new WASM contract engine
    pub fn new(
        storage: Arc<dyn ContractStateStorage>,
        gas_manager: UnifiedGasManager,
    ) -> Result<Self> {
        let engine = Engine::default();

        Ok(Self {
            engine,
            storage,
            gas_manager,
            erc20_contracts: Arc::new(Mutex::new(HashMap::new())),
            gas_config: GasConfig::default(),
        })
    }

    /// Deploy an ERC20 contract using the unified interface
    pub fn deploy_erc20_unified(
        &mut self,
        name: String,
        symbol: String,
        decimals: u8,
        initial_supply: u64,
        owner: String,
        contract_address: String,
    ) -> Result<String> {
        let contract = ERC20Contract::new(
            name.clone(),
            symbol.clone(),
            decimals,
            initial_supply,
            owner.clone(),
        );

        // Create unified metadata
        let metadata = UnifiedContractMetadata {
            address: contract_address.clone(),
            name: format!("ERC20: {}", name),
            description: format!("ERC20 token {} ({})", name, symbol),
            contract_type: ContractType::BuiltIn {
                contract_name: "ERC20".to_string(),
                parameters: {
                    let mut params = HashMap::new();
                    params.insert("name".to_string(), name);
                    params.insert("symbol".to_string(), symbol);
                    params.insert("decimals".to_string(), decimals.to_string());
                    params.insert("initial_supply".to_string(), initial_supply.to_string());
                    params
                },
            },
            deployment_tx: Uuid::new_v4().to_string(),
            deployment_time: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            owner,
            is_active: true,
        };

        // Store metadata
        self.storage.store_contract_metadata(&metadata)?;

        // Store ERC20 state
        let contract_data = bincode::serialize(&contract.state)?;
        self.storage
            .set_contract_state(&contract_address, "erc20_state", &contract_data)?;

        // Cache in memory
        {
            let mut contracts = self.erc20_contracts.lock().unwrap();
            contracts.insert(contract_address.clone(), contract);
        }

        Ok(contract_address)
    }

    /// Execute ERC20 contract function
    fn execute_erc20_function(
        &mut self,
        contract_address: &str,
        function_name: &str,
        input_data: &[u8],
        caller: &str,
    ) -> Result<UnifiedContractResult> {
        let start_time = Instant::now();

        // Load ERC20 contract
        let mut contract = self
            .load_erc20_contract(contract_address)?
            .ok_or_else(|| anyhow::anyhow!("ERC20 contract not found: {}", contract_address))?;

        let mut events = Vec::new();
        let mut return_data = Vec::new();
        let mut success = true;
        let mut error_message = None;

        // Execute based on function name
        let _result = match function_name {
            "transfer" => {
                if input_data.len() >= 40 {
                    // 32 bytes for address + 8 bytes for amount
                    let to = String::from_utf8_lossy(&input_data[0..32])
                        .trim_end_matches('\0')
                        .to_string();
                    let amount = u64::from_be_bytes([
                        input_data[32],
                        input_data[33],
                        input_data[34],
                        input_data[35],
                        input_data[36],
                        input_data[37],
                        input_data[38],
                        input_data[39],
                    ]);

                    match contract.transfer(caller, &to, amount) {
                        Ok(_) => {
                            events.push(ContractEvent {
                                contract_address: contract_address.to_string(),
                                event_type: "Transfer".to_string(),
                                topics: vec![caller.to_string(), to],
                                data: amount.to_be_bytes().to_vec(),
                                timestamp: std::time::SystemTime::now()
                                    .duration_since(std::time::UNIX_EPOCH)
                                    .unwrap()
                                    .as_secs(),
                            });
                            return_data = vec![1]; // Success
                            Ok(())
                        }
                        Err(e) => {
                            success = false;
                            error_message = Some(e.to_string());
                            return_data = vec![0]; // Failure
                            Err(e)
                        }
                    }
                } else {
                    success = false;
                    error_message = Some("Invalid input data for transfer".to_string());
                    Err(anyhow::anyhow!("Invalid input data"))
                }
            }
            "balance_of" => {
                if input_data.len() >= 32 {
                    let address = String::from_utf8_lossy(&input_data[0..32])
                        .trim_end_matches('\0')
                        .to_string();
                    let balance = contract.balance_of(&address);
                    return_data = balance.to_be_bytes().to_vec();
                    Ok(())
                } else {
                    success = false;
                    error_message = Some("Invalid input data for balance_of".to_string());
                    Err(anyhow::anyhow!("Invalid input data"))
                }
            }
            "approve" => {
                if input_data.len() >= 40 {
                    let spender = String::from_utf8_lossy(&input_data[0..32])
                        .trim_end_matches('\0')
                        .to_string();
                    let amount = u64::from_be_bytes([
                        input_data[32],
                        input_data[33],
                        input_data[34],
                        input_data[35],
                        input_data[36],
                        input_data[37],
                        input_data[38],
                        input_data[39],
                    ]);

                    contract.approve(caller, &spender, amount)?;

                    events.push(ContractEvent {
                        contract_address: contract_address.to_string(),
                        event_type: "Approval".to_string(),
                        topics: vec![caller.to_string(), spender],
                        data: amount.to_be_bytes().to_vec(),
                        timestamp: std::time::SystemTime::now()
                            .duration_since(std::time::UNIX_EPOCH)
                            .unwrap()
                            .as_secs(),
                    });

                    return_data = vec![1]; // Success
                    Ok(())
                } else {
                    success = false;
                    error_message = Some("Invalid input data for approve".to_string());
                    Err(anyhow::anyhow!("Invalid input data"))
                }
            }
            _ => {
                success = false;
                error_message = Some(format!("Unknown function: {}", function_name));
                Err(anyhow::anyhow!("Unknown function: {}", function_name))
            }
        };

        // Update contract state if execution was successful
        if success {
            let contract_data = bincode::serialize(&contract.state)?;
            self.storage
                .set_contract_state(contract_address, "erc20_state", &contract_data)?;

            // Update memory cache
            {
                let mut contracts = self.erc20_contracts.lock().unwrap();
                contracts.insert(contract_address.to_string(), contract);
            }
        }

        let execution_time = start_time.elapsed().as_millis() as u64;

        // Calculate gas used
        let base_gas = self
            .gas_manager
            .calculate_base_gas(&UnifiedContractExecution {
                contract_address: contract_address.to_string(),
                function_name: function_name.to_string(),
                input_data: input_data.to_vec(),
                caller: caller.to_string(),
                value: 0,
                gas_limit: 1000000,
            });
        let computation_gas = self.gas_manager.calculate_computation_gas(execution_time);
        let storage_gas = if success {
            self.gas_manager.calculate_storage_gas(32, 64) // Estimate
        } else {
            0
        };

        let gas_used = base_gas + computation_gas + storage_gas;

        Ok(UnifiedContractResult {
            success,
            return_data,
            gas_used,
            events,
            execution_time_ms: execution_time,
            error_message,
        })
    }

    /// Load ERC20 contract from storage
    fn load_erc20_contract(&self, contract_address: &str) -> Result<Option<ERC20Contract>> {
        // Check memory cache first
        {
            let contracts = self.erc20_contracts.lock().unwrap();
            if let Some(contract) = contracts.get(contract_address) {
                return Ok(Some(contract.clone()));
            }
        }

        // Load from storage
        if let Some(contract_data) = self
            .storage
            .get_contract_state(contract_address, "erc20_state")?
        {
            let erc20_state: crate::smart_contract::erc20::ERC20State =
                bincode::deserialize(&contract_data)?;
            let contract = ERC20Contract {
                state: erc20_state,
                events: Vec::new(), // Events are not persisted in this implementation
            };

            // Cache in memory
            {
                let mut contracts = self.erc20_contracts.lock().unwrap();
                contracts.insert(contract_address.to_string(), contract.clone());
            }

            Ok(Some(contract))
        } else {
            Ok(None)
        }
    }
}

impl UnifiedContractEngine for WasmContractEngine {
    fn deploy_contract(
        &mut self,
        metadata: UnifiedContractMetadata,
        init_data: Vec<u8>,
    ) -> Result<String> {
        match &metadata.contract_type {
            ContractType::BuiltIn {
                contract_name,
                parameters,
            } => {
                if contract_name == "ERC20" {
                    let name = parameters
                        .get("name")
                        .ok_or_else(|| anyhow::anyhow!("Missing name parameter"))?;
                    let symbol = parameters
                        .get("symbol")
                        .ok_or_else(|| anyhow::anyhow!("Missing symbol parameter"))?;
                    let decimals: u8 = parameters
                        .get("decimals")
                        .ok_or_else(|| anyhow::anyhow!("Missing decimals parameter"))?
                        .parse()?;
                    let initial_supply: u64 = parameters
                        .get("initial_supply")
                        .ok_or_else(|| anyhow::anyhow!("Missing initial_supply parameter"))?
                        .parse()?;

                    self.deploy_erc20_unified(
                        name.clone(),
                        symbol.clone(),
                        decimals,
                        initial_supply,
                        metadata.owner.clone(),
                        metadata.address.clone(),
                    )
                } else {
                    Err(anyhow::anyhow!(
                        "Unsupported built-in contract: {}",
                        contract_name
                    ))
                }
            }
            ContractType::Wasm { bytecode, .. } => {
                // Store the metadata
                self.storage.store_contract_metadata(&metadata)?;

                // Store the bytecode
                self.storage
                    .set_contract_state(&metadata.address, "wasm_bytecode", bytecode)?;

                // TODO: Initialize WASM module with init_data
                // For now, just store it
                if !init_data.is_empty() {
                    self.storage
                        .set_contract_state(&metadata.address, "init_data", &init_data)?;
                }

                Ok(metadata.address)
            }
            ContractType::PrivacyEnhanced { .. } => Err(anyhow::anyhow!(
                "Privacy-enhanced contracts not supported by WASM engine"
            )),
        }
    }

    fn execute_contract(
        &mut self,
        execution: UnifiedContractExecution,
    ) -> Result<UnifiedContractResult> {
        // Check if contract exists
        let metadata = self
            .get_contract(&execution.contract_address)?
            .ok_or_else(|| anyhow::anyhow!("Contract not found: {}", execution.contract_address))?;

        // Record execution start
        let execution_record = ContractExecutionRecord {
            execution_id: Uuid::new_v4().to_string(),
            contract_address: execution.contract_address.clone(),
            function_name: execution.function_name.clone(),
            caller: execution.caller.clone(),
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
            gas_used: 0,    // Will be updated after execution
            success: false, // Will be updated after execution
            error_message: None,
        };

        let result = match &metadata.contract_type {
            ContractType::BuiltIn { contract_name, .. } => {
                if contract_name == "ERC20" {
                    self.execute_erc20_function(
                        &execution.contract_address,
                        &execution.function_name,
                        &execution.input_data,
                        &execution.caller,
                    )
                } else {
                    Err(anyhow::anyhow!(
                        "Unsupported built-in contract: {}",
                        contract_name
                    ))
                }
            }
            ContractType::Wasm { .. } => {
                // TODO: Implement WASM execution
                Err(anyhow::anyhow!(
                    "WASM execution not yet implemented in unified engine"
                ))
            }
            ContractType::PrivacyEnhanced { .. } => Err(anyhow::anyhow!(
                "Privacy-enhanced contracts not supported by WASM engine"
            )),
        };

        // Update and store execution record
        let final_result = result.unwrap_or_else(|e| UnifiedContractResult {
            success: false,
            return_data: Vec::new(),
            gas_used: self.gas_manager.calculate_base_gas(&execution),
            events: Vec::new(),
            execution_time_ms: 0,
            error_message: Some(e.to_string()),
        });

        let mut final_record = execution_record;
        final_record.gas_used = final_result.gas_used;
        final_record.success = final_result.success;
        final_record.error_message = final_result.error_message.clone();

        self.storage.store_execution(&final_record)?;

        Ok(final_result)
    }

    fn get_contract(&self, address: &str) -> Result<Option<UnifiedContractMetadata>> {
        self.storage.get_contract_metadata(address)
    }

    fn get_contract_state(&self, contract: &str, key: &str) -> Result<Option<Vec<u8>>> {
        self.storage.get_contract_state(contract, key)
    }

    fn list_contracts(&self) -> Result<Vec<String>> {
        self.storage.list_contracts()
    }

    fn estimate_gas(&self, execution: &UnifiedContractExecution) -> Result<u64> {
        let base_gas = self.gas_manager.calculate_base_gas(execution);

        // Add estimates based on function complexity
        let function_gas = match execution.function_name.as_str() {
            "transfer" | "approve" => 50000,    // Storage operations
            "balance_of" | "allowance" => 5000, // Read operations
            _ => 25000,                         // Default estimate
        };

        Ok(base_gas + function_gas)
    }

    fn get_execution_history(&self, contract: &str) -> Result<Vec<ContractExecutionRecord>> {
        self.storage.get_execution_history(contract)
    }

    fn engine_info(&self) -> EngineInfo {
        EngineInfo {
            name: "WASM Contract Engine".to_string(),
            version: "1.0.0".to_string(),
            supported_contract_types: vec!["BuiltIn".to_string(), "Wasm".to_string()],
            features: vec![
                "ERC20 Support".to_string(),
                "Gas Metering".to_string(),
                "Event System".to_string(),
                "State Persistence".to_string(),
            ],
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::smart_contract::{
        unified_engine::{UnifiedGasConfig, UnifiedGasManager},
        unified_storage::SyncInMemoryContractStorage,
    };

    fn create_test_engine() -> WasmContractEngine {
        let storage = Arc::new(SyncInMemoryContractStorage::new());
        let gas_manager = UnifiedGasManager::new(UnifiedGasConfig::default());
        WasmContractEngine::new(storage, gas_manager).unwrap()
    }

    #[test]
    fn test_erc20_deployment() {
        let mut engine = create_test_engine();

        let address = engine
            .deploy_erc20_unified(
                "Test Token".to_string(),
                "TTK".to_string(),
                18,
                1000000,
                "0x1234567890".to_string(),
                "0xcontract123".to_string(),
            )
            .unwrap();

        assert_eq!(address, "0xcontract123");

        // Verify contract metadata was stored
        let metadata = engine.get_contract(&address).unwrap();
        assert!(metadata.is_some());

        let metadata = metadata.unwrap();
        assert_eq!(metadata.name, "ERC20: Test Token");
        assert!(metadata.is_active);
    }

    #[test]
    fn test_erc20_execution() {
        let mut engine = create_test_engine();

        // Deploy contract
        let contract_address = "0xcontract123";
        engine
            .deploy_erc20_unified(
                "Test Token".to_string(),
                "TTK".to_string(),
                18,
                1000000,
                "0x1234567890".to_string(),
                contract_address.to_string(),
            )
            .unwrap();

        // Test balance_of
        let mut input_data = vec![0u8; 32];
        input_data[..11].copy_from_slice(b"0x123456789");

        let execution = UnifiedContractExecution {
            contract_address: contract_address.to_string(),
            function_name: "balance_of".to_string(),
            input_data,
            caller: "0x1234567890".to_string(),
            value: 0,
            gas_limit: 100000,
        };

        let result = engine.execute_contract(execution).unwrap();
        assert!(result.success);
        assert_eq!(result.return_data.len(), 8); // u64 balance
    }

    #[test]
    fn test_gas_estimation() {
        let engine = create_test_engine();

        let execution = UnifiedContractExecution {
            contract_address: "0xcontract123".to_string(),
            function_name: "transfer".to_string(),
            input_data: vec![0; 40],
            caller: "0x1234567890".to_string(),
            value: 0,
            gas_limit: 100000,
        };

        let estimated_gas = engine.estimate_gas(&execution).unwrap();
        assert!(estimated_gas > 21000); // Should include base cost plus function cost
    }

    #[test]
    fn test_engine_info() {
        let engine = create_test_engine();
        let info = engine.engine_info();

        assert_eq!(info.name, "WASM Contract Engine");
        assert!(
            info.supported_contract_types.contains(&"ERC20".to_string())
                || info
                    .supported_contract_types
                    .contains(&"BuiltIn".to_string())
        );
        assert!(info.features.contains(&"ERC20 Support".to_string()));
    }
}
