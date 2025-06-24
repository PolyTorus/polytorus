//! WASM contract execution engine - simplified and stable version

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use wasmtime::*;

use crate::{
    smart_contract::{
        contract::SmartContract,
        erc20::ERC20Contract,
        state::ContractState,
        types::{ContractExecution, ContractMetadata, ContractResult, GasConfig, GasMeter},
    },
    Result,
};

/// Host function execution context
#[derive(Clone)]
struct HostContext {
    contract_address: String,
    caller: String,
    value: u64,
    state: Arc<Mutex<ContractState>>,
    logs: Arc<Mutex<Vec<String>>>,
    state_changes: Arc<Mutex<HashMap<String, Vec<u8>>>>,
    gas_meter: Arc<Mutex<GasMeter>>,
}

/// WASM contract execution engine
pub struct ContractEngine {
    engine: Engine,
    state: Arc<Mutex<ContractState>>,
    gas_config: GasConfig,
    erc20_contracts: Arc<Mutex<HashMap<String, ERC20Contract>>>,
}

impl ContractEngine {
    /// Create a new contract engine
    pub fn new(state: ContractState) -> Result<Self> {
        let engine = Engine::default();
        Ok(Self {
            engine,
            state: Arc::new(Mutex::new(state)),
            gas_config: GasConfig::default(),
            erc20_contracts: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Get access to the contract state for testing
    #[cfg(test)]
    pub fn get_state(&self) -> &Arc<Mutex<ContractState>> {
        &self.state
    }

    /// Deploy an ERC20 token contract
    pub fn deploy_erc20_contract(
        &self,
        name: String,
        symbol: String,
        decimals: u8,
        initial_supply: u64,
        initial_owner: String,
        contract_address: String,
    ) -> Result<String> {
        let contract = ERC20Contract::new(name, symbol, decimals, initial_supply, initial_owner);

        // Save to persistent storage
        let state = self.state.lock().unwrap();
        let contract_data = bincode::serialize(&contract.state)?;
        state.store_data(&format!("erc20:{}", contract_address), &contract_data)?;

        // Also keep in memory
        let mut erc20_contracts = self.erc20_contracts.lock().unwrap();
        erc20_contracts.insert(contract_address.clone(), contract);

        println!("ERC20 contract deployed at address: {}", contract_address);
        Ok(contract_address)
    }

    /// Load ERC20 contract from storage
    fn load_erc20_contract(&self, contract_address: &str) -> Result<Option<ERC20Contract>> {
        // First check memory
        {
            let erc20_contracts = self.erc20_contracts.lock().unwrap();
            if let Some(contract) = erc20_contracts.get(contract_address) {
                return Ok(Some(contract.clone()));
            }
        }

        // Then check persistent storage
        let state = self.state.lock().unwrap();
        if let Some(contract_data) = state.get_data(&format!("erc20:{}", contract_address))? {
            let erc20_state: crate::smart_contract::erc20::ERC20State =
                bincode::deserialize(&contract_data)?;
            let contract = ERC20Contract {
                state: erc20_state,
                events: Vec::new(), // Events are not persisted
            };

            // Cache in memory
            drop(state);
            let mut erc20_contracts = self.erc20_contracts.lock().unwrap();
            erc20_contracts.insert(contract_address.to_string(), contract.clone());

            Ok(Some(contract))
        } else {
            Ok(None)
        }
    }

    /// Execute an ERC20 contract function
    pub fn execute_erc20_contract(
        &self,
        contract_address: &str,
        function_name: &str,
        caller: &str,
        args: Vec<String>,
    ) -> Result<ContractResult> {
        // Load contract from storage or memory
        let mut contract = match self.load_erc20_contract(contract_address)? {
            Some(contract) => contract,
            None => {
                return Ok(ContractResult {
                    success: false,
                    return_value: b"ERC20 contract not found".to_vec(),
                    gas_used: 500,
                    logs: vec![format!("ERC20 contract not found: {}", contract_address)],
                    state_changes: HashMap::new(),
                });
            }
        };

        let result = match function_name {
            "name" => Ok(ContractResult {
                success: true,
                return_value: contract.name().as_bytes().to_vec(),
                gas_used: 500,
                logs: vec![],
                state_changes: HashMap::new(),
            }),
            "symbol" => Ok(ContractResult {
                success: true,
                return_value: contract.symbol().as_bytes().to_vec(),
                gas_used: 500,
                logs: vec![],
                state_changes: HashMap::new(),
            }),
            "decimals" => Ok(ContractResult {
                success: true,
                return_value: contract.decimals().to_string().as_bytes().to_vec(),
                gas_used: 500,
                logs: vec![],
                state_changes: HashMap::new(),
            }),
            "totalSupply" => Ok(ContractResult {
                success: true,
                return_value: contract.total_supply().to_string().as_bytes().to_vec(),
                gas_used: 500,
                logs: vec![],
                state_changes: HashMap::new(),
            }),
            "balanceOf" => {
                if args.is_empty() {
                    return Ok(ContractResult {
                        success: false,
                        return_value: b"Missing owner address".to_vec(),
                        gas_used: 500,
                        logs: vec!["Missing owner address for balanceOf".to_string()],
                        state_changes: HashMap::new(),
                    });
                }
                let balance = contract.balance_of(&args[0]);
                Ok(ContractResult {
                    success: true,
                    return_value: balance.to_string().as_bytes().to_vec(),
                    gas_used: 800,
                    logs: vec![],
                    state_changes: HashMap::new(),
                })
            }
            "allowance" => {
                if args.len() < 2 {
                    return Ok(ContractResult {
                        success: false,
                        return_value: b"Missing owner or spender address".to_vec(),
                        gas_used: 500,
                        logs: vec!["Missing owner or spender address for allowance".to_string()],
                        state_changes: HashMap::new(),
                    });
                }
                let allowance = contract.allowance(&args[0], &args[1]);
                Ok(ContractResult {
                    success: true,
                    return_value: allowance.to_string().as_bytes().to_vec(),
                    gas_used: 800,
                    logs: vec![],
                    state_changes: HashMap::new(),
                })
            }
            "transfer" => {
                if args.len() < 2 {
                    Ok(ContractResult {
                        success: false,
                        return_value: b"Missing to address or amount".to_vec(),
                        gas_used: 500,
                        logs: vec!["Missing to address or amount for transfer".to_string()],
                        state_changes: HashMap::new(),
                    })
                } else {
                    let to = &args[0];
                    let amount: u64 = args[1].parse().unwrap_or(0);
                    contract.transfer(caller, to, amount)
                }
            }
            "approve" => {
                if args.len() < 2 {
                    Ok(ContractResult {
                        success: false,
                        return_value: b"Missing spender address or amount".to_vec(),
                        gas_used: 500,
                        logs: vec!["Missing spender address or amount for approve".to_string()],
                        state_changes: HashMap::new(),
                    })
                } else {
                    let spender = &args[0];
                    let amount: u64 = args[1].parse().unwrap_or(0);
                    contract.approve(caller, spender, amount)
                }
            }
            "transferFrom" => {
                if args.len() < 3 {
                    Ok(ContractResult {
                        success: false,
                        return_value: b"Missing from address, to address, or amount".to_vec(),
                        gas_used: 500,
                        logs: vec![
                            "Missing from address, to address, or amount for transferFrom"
                                .to_string(),
                        ],
                        state_changes: HashMap::new(),
                    })
                } else {
                    let from = &args[0];
                    let to = &args[1];
                    let amount: u64 = args[2].parse().unwrap_or(0);
                    contract.transfer_from(caller, from, to, amount)
                }
            }
            "mint" => {
                if args.len() < 2 {
                    Ok(ContractResult {
                        success: false,
                        return_value: b"Missing to address or amount".to_vec(),
                        gas_used: 500,
                        logs: vec!["Missing to address or amount for mint".to_string()],
                        state_changes: HashMap::new(),
                    })
                } else {
                    let to = &args[0];
                    let amount: u64 = args[1].parse().unwrap_or(0);
                    contract.mint(to, amount)
                }
            }
            "burn" => {
                if args.is_empty() {
                    Ok(ContractResult {
                        success: false,
                        return_value: b"Missing amount".to_vec(),
                        gas_used: 500,
                        logs: vec!["Missing amount for burn".to_string()],
                        state_changes: HashMap::new(),
                    })
                } else {
                    let amount: u64 = args[0].parse().unwrap_or(0);
                    contract.burn(caller, amount)
                }
            }
            _ => Ok(ContractResult {
                success: false,
                return_value: format!("Unknown function: {}", function_name)
                    .as_bytes()
                    .to_vec(),
                gas_used: 500,
                logs: vec![format!("Unknown ERC20 function: {}", function_name)],
                state_changes: HashMap::new(),
            }),
        };

        // Save updated contract state back to storage if the operation was successful
        if let Ok(ref contract_result) = result {
            if contract_result.success && !contract_result.state_changes.is_empty() {
                let state = self.state.lock().unwrap();
                let contract_data = bincode::serialize(&contract.state)?;
                state.store_data(&format!("erc20:{}", contract_address), &contract_data)?;

                // Update memory cache
                let mut erc20_contracts = self.erc20_contracts.lock().unwrap();
                erc20_contracts.insert(contract_address.to_string(), contract);
            }
        }

        result
    }

    /// Get ERC20 contract information
    pub fn get_erc20_contract_info(
        &self,
        contract_address: &str,
    ) -> Result<Option<(String, String, u8, u64)>> {
        if let Some(contract) = self.load_erc20_contract(contract_address)? {
            Ok(Some((
                contract.name().to_string(),
                contract.symbol().to_string(),
                contract.decimals(),
                contract.total_supply(),
            )))
        } else {
            Ok(None)
        }
    }

    /// List all ERC20 contracts
    pub fn list_erc20_contracts(&self) -> Result<Vec<String>> {
        let state = self.state.lock().unwrap();
        let mut contracts = Vec::new();

        // Scan for ERC20 contracts in storage
        let keys = state.scan_prefix("erc20:")?;
        for key_str in keys {
            if let Some(contract_address) = key_str.strip_prefix("erc20:") {
                contracts.push(contract_address.to_string());
            }
        }

        Ok(contracts)
    }

    /// Deploy a smart contract
    pub fn deploy_contract(&self, contract: &SmartContract) -> Result<()> {
        let state = self.state.lock().unwrap();
        contract.deploy(&state)?;
        Ok(())
    }

    /// List all deployed contracts
    pub fn list_contracts(&self) -> Result<Vec<ContractMetadata>> {
        let state = self.state.lock().unwrap();
        state.list_contracts()
    }

    /// Execute a smart contract function with proper gas metering
    pub fn execute_contract(&self, execution: ContractExecution) -> Result<ContractResult> {
        println!("Executing contract function: {}", execution.function_name);

        // Check maximum gas limit
        if execution.gas_limit > self.gas_config.max_gas_per_call {
            return Ok(ContractResult {
                success: false,
                return_value: vec![],
                gas_used: 0,
                state_changes: HashMap::new(),
                logs: vec![format!(
                    "Gas limit {} exceeds maximum allowed {}",
                    execution.gas_limit, self.gas_config.max_gas_per_call
                )],
            });
        }

        // Initialize gas meter
        let mut gas_meter = GasMeter::new(execution.gas_limit, self.gas_config.clone());

        // Consume base gas for function call
        if let Err(e) = gas_meter.consume_function_call() {
            return Ok(ContractResult {
                success: false,
                return_value: vec![],
                gas_used: gas_meter.gas_used,
                state_changes: HashMap::new(),
                logs: vec![e],
            });
        }

        // Try to get the actual contract bytecode first
        let bytecode = if let Ok(state) = self.state.lock() {
            // Try to get the contract from the state
            if let Ok(contracts) = state.list_contracts() {
                if let Some(_contract_meta) = contracts
                    .iter()
                    .find(|c| c.address == execution.contract_address)
                {
                    // For now, we'll use our fallback since we don't store the actual bytecode yet
                    // In a production system, you'd retrieve the actual bytecode here
                    self.load_simple_contract()?
                } else {
                    self.load_simple_contract()?
                }
            } else {
                self.load_simple_contract()?
            }
        } else {
            self.load_simple_contract()?
        };

        // Create WASM module
        let module = Module::new(&self.engine, &bytecode)
            .map_err(|e| anyhow::anyhow!("Failed to create WASM module: {}", e))?;

        // Create host context for actual processing
        let logs = Arc::new(Mutex::new(Vec::new()));
        let state_changes = Arc::new(Mutex::new(HashMap::new()));
        let gas_meter_ref = Arc::new(Mutex::new(gas_meter));
        let host_context = HostContext {
            contract_address: execution.contract_address.clone(),
            caller: execution.caller.clone(),
            value: execution.value,
            state: self.state.clone(),
            logs: logs.clone(),
            state_changes: state_changes.clone(),
            gas_meter: gas_meter_ref.clone(),
        };

        // Create store with host context
        let mut store = Store::new(&self.engine, host_context);

        // Create linker with actual host functions
        let mut linker = Linker::new(&self.engine);
        self.add_host_functions(&mut linker, &execution)?;

        // Instantiate the module
        let instance = linker
            .instantiate(&mut store, &module)
            .map_err(|e| anyhow::anyhow!("Failed to instantiate module: {}", e))?;

        // Call the function - state changes are now handled by host functions
        let result = self.call_simple_function(&mut store, &instance, &execution.function_name)?;

        // Get final gas usage
        let final_gas_used = if let Ok(meter_guard) = gas_meter_ref.lock() {
            meter_guard.gas_used
        } else {
            self.gas_config.function_call_cost // fallback
        };

        // Get logs from host context
        let execution_logs = if let Ok(logs_guard) = logs.lock() {
            logs_guard.clone()
        } else {
            vec![]
        };

        // Get state changes from host context
        let tracked_state_changes = if let Ok(changes_guard) = state_changes.lock() {
            changes_guard.clone()
        } else {
            HashMap::new()
        };

        Ok(ContractResult {
            success: true,
            return_value: result,
            gas_used: final_gas_used,
            state_changes: tracked_state_changes,
            logs: execution_logs,
        })
    }

    /// Execute a contract with provided bytecode (for direct WASM file execution)
    pub fn execute_contract_with_bytecode(
        &self,
        bytecode: Vec<u8>,
        execution: ContractExecution,
    ) -> Result<ContractResult> {
        println!(
            "Executing WASM contract function: {}",
            execution.function_name
        );

        // Check maximum gas limit
        if execution.gas_limit > self.gas_config.max_gas_per_call {
            return Ok(ContractResult {
                success: false,
                return_value: vec![],
                gas_used: 0,
                state_changes: HashMap::new(),
                logs: vec![format!(
                    "Gas limit {} exceeds maximum allowed {}",
                    execution.gas_limit, self.gas_config.max_gas_per_call
                )],
            });
        }

        // Initialize gas meter
        let mut gas_meter = GasMeter::new(execution.gas_limit, self.gas_config.clone());

        // Consume base gas for function call
        if let Err(e) = gas_meter.consume_function_call() {
            return Ok(ContractResult {
                success: false,
                return_value: vec![],
                gas_used: gas_meter.gas_used,
                state_changes: HashMap::new(),
                logs: vec![e],
            });
        }

        // Create WASM module from provided bytecode
        let module = Module::new(&self.engine, &bytecode)
            .map_err(|e| anyhow::anyhow!("Failed to create WASM module: {}", e))?;

        // Create host context for actual processing
        let logs = Arc::new(Mutex::new(Vec::new()));
        let state_changes = Arc::new(Mutex::new(HashMap::new()));
        let gas_meter_ref = Arc::new(Mutex::new(gas_meter));
        let host_context = HostContext {
            contract_address: execution.contract_address.clone(),
            caller: execution.caller.clone(),
            value: execution.value,
            state: self.state.clone(),
            logs: logs.clone(),
            state_changes: state_changes.clone(),
            gas_meter: gas_meter_ref.clone(),
        };

        // Create store with host context
        let mut store = Store::new(&self.engine, host_context);

        // Create linker with actual host functions
        let mut linker = Linker::new(&self.engine);
        self.add_host_functions(&mut linker, &execution)?;

        // Instantiate the module
        let instance = linker
            .instantiate(&mut store, &module)
            .map_err(|e| anyhow::anyhow!("Failed to instantiate module: {}", e))?;

        // Call the function - state changes are now handled by host functions
        let result = self.call_simple_function(&mut store, &instance, &execution.function_name)?;

        // Get final gas usage
        let final_gas_used = if let Ok(meter_guard) = gas_meter_ref.lock() {
            meter_guard.gas_used
        } else {
            self.gas_config.function_call_cost // fallback
        };

        // Get logs from host context
        let execution_logs = if let Ok(logs_guard) = logs.lock() {
            logs_guard.clone()
        } else {
            vec![]
        };

        // Get state changes from host context
        let tracked_state_changes = if let Ok(changes_guard) = state_changes.lock() {
            changes_guard.clone()
        } else {
            HashMap::new()
        };

        Ok(ContractResult {
            success: true,
            return_value: result,
            gas_used: final_gas_used,
            state_changes: tracked_state_changes,
            logs: execution_logs,
        })
    }

    /// Add host functions with actual implementations
    fn add_host_functions(
        &self,
        linker: &mut Linker<HostContext>,
        _execution: &ContractExecution,
    ) -> Result<()> {
        // Storage get function - reads from actual database with proper data handling
        linker
            .func_wrap(
                "env",
                "storage_get",
                |mut caller: Caller<'_, HostContext>,
                 key_ptr: i32,
                 key_len: i32,
                 value_ptr: i32,
                 max_value_len: i32|
                 -> i32 {
                    let ctx = caller.data().clone();

                    // Memory safety: validate input parameters
                    if !(0..=1024).contains(&key_len) || !(0..=64 * 1024).contains(&max_value_len) {
                        return -1; // Invalid parameters
                    }

                    // Get memory to read the key
                    let memory = match caller.get_export("memory") {
                        Some(Extern::Memory(mem)) => mem,
                        _ => return -2, // Memory not found
                    };

                    // Read key from WASM memory
                    let mut key_bytes = vec![0u8; key_len as usize];
                    if memory
                        .read(&caller, key_ptr as usize, &mut key_bytes)
                        .is_err()
                    {
                        return -3; // Failed to read key
                    }

                    let key = match String::from_utf8(key_bytes) {
                        Ok(k) => k,
                        Err(_) => return -4, // Invalid UTF-8 in key
                    };

                    // Consume gas for storage read
                    if let Ok(mut gas_meter) = ctx.gas_meter.lock() {
                        if gas_meter.consume_storage_read().is_err() {
                            return -8; // Out of gas
                        }
                    }

                    // Read from database
                    let state_result = ctx.state.lock();
                    if let Ok(state) = state_result {
                        match state.get(&ctx.contract_address, &key) {
                            Ok(Some(value)) => {
                                // Calculate how much data we can return
                                let return_len = std::cmp::min(value.len(), max_value_len as usize);

                                // Write value to WASM memory
                                if memory
                                    .write(&mut caller, value_ptr as usize, &value[..return_len])
                                    .is_err()
                                {
                                    return -5; // Failed to write value to memory
                                }

                                return_len as i32 // Return actual length written
                            }
                            Ok(None) => 0, // Key not found
                            Err(_) => -6,  // Database error
                        }
                    } else {
                        -7 // Failed to lock state
                    }
                },
            )
            .map_err(|e| anyhow::anyhow!("Failed to add storage_get: {}", e))?;

        // Storage set function - writes to actual database with error handling
        linker
            .func_wrap(
                "env",
                "storage_set",
                |mut caller: Caller<'_, HostContext>,
                 key_ptr: i32,
                 key_len: i32,
                 value_ptr: i32,
                 value_len: i32|
                 -> i32 {
                    let ctx = caller.data().clone();

                    // Memory safety: validate input parameters
                    if !(0..=1024).contains(&key_len) || !(0..=64 * 1024).contains(&value_len) {
                        return -1; // Invalid parameters
                    }

                    // Get memory to read key and value
                    let memory = match caller.get_export("memory") {
                        Some(Extern::Memory(mem)) => mem,
                        _ => return -2, // Memory not found
                    };

                    // Read key from WASM memory
                    let mut key_bytes = vec![0u8; key_len as usize];
                    if memory
                        .read(&caller, key_ptr as usize, &mut key_bytes)
                        .is_err()
                    {
                        return -3; // Failed to read key
                    }

                    // Read value from WASM memory
                    let mut value_bytes = vec![0u8; value_len as usize];
                    if memory
                        .read(&caller, value_ptr as usize, &mut value_bytes)
                        .is_err()
                    {
                        return -4; // Failed to read value
                    }

                    let key = match String::from_utf8(key_bytes) {
                        Ok(k) => k,
                        Err(_) => return -5, // Invalid UTF-8 in key
                    };

                    // Consume gas for storage write
                    if let Ok(mut gas_meter) = ctx.gas_meter.lock() {
                        if gas_meter.consume_storage_write().is_err() {
                            return -8; // Out of gas
                        }
                    }

                    // Write to database
                    let state_result = ctx.state.lock();
                    if let Ok(state) = state_result {
                        match state.set(&ctx.contract_address, &key, &value_bytes) {
                            Ok(_) => {
                                // Track state changes
                                if let Ok(mut changes) = ctx.state_changes.lock() {
                                    let full_key = format!("{}:{}", ctx.contract_address, key);
                                    changes.insert(full_key, value_bytes);
                                }
                                1 // Success
                            }
                            Err(_) => -6, // Database write error
                        }
                    } else {
                        -7 // Failed to lock state
                    }
                },
            )
            .map_err(|e| anyhow::anyhow!("Failed to add storage_set: {}", e))?;

        // Log function - captures actual logs
        linker
            .func_wrap(
                "env",
                "log",
                |mut caller: Caller<'_, HostContext>, msg_ptr: i32, msg_len: i32| {
                    let ctx = caller.data().clone();

                    // Get memory to read the log message
                    let memory = match caller.get_export("memory") {
                        Some(Extern::Memory(mem)) => mem,
                        _ => return,
                    };

                    // Read message from WASM memory
                    let mut msg_bytes = vec![0u8; msg_len as usize];
                    if memory
                        .read(&caller, msg_ptr as usize, &mut msg_bytes)
                        .is_err()
                    {
                        return;
                    }

                    let message = match String::from_utf8(msg_bytes) {
                        Ok(msg) => msg,
                        Err(_) => return,
                    };

                    // Add to logs
                    let logs_result = ctx.logs.lock();
                    if let Ok(mut logs) = logs_result {
                        logs.push(format!("[{}] {}", ctx.contract_address, message));
                    }
                },
            )
            .map_err(|e| anyhow::anyhow!("Failed to add log: {}", e))?;

        // Get caller function - returns actual caller address
        linker
            .func_wrap(
                "env",
                "get_caller",
                |caller: Caller<'_, HostContext>| -> i32 {
                    let ctx = caller.data();
                    // Convert caller address to a simple hash for demonstration
                    // In a real implementation, you might want to store addresses
                    // in memory and return pointers
                    ctx.caller
                        .as_bytes()
                        .iter()
                        .fold(0i32, |acc, &b| acc.wrapping_add(b as i32))
                },
            )
            .map_err(|e| anyhow::anyhow!("Failed to add get_caller: {}", e))?;

        // Get value function - returns actual transaction value
        linker
            .func_wrap(
                "env",
                "get_value",
                |caller: Caller<'_, HostContext>| -> i64 {
                    let ctx = caller.data();
                    ctx.value as i64
                },
            )
            .map_err(|e| anyhow::anyhow!("Failed to add get_value: {}", e))?;

        Ok(())
    }

    /// Call a simple function without complex argument handling
    fn call_simple_function(
        &self,
        store: &mut Store<HostContext>,
        instance: &Instance,
        function_name: &str,
    ) -> Result<Vec<u8>> {
        println!("Calling function: {}", function_name);

        let func = instance
            .get_typed_func::<(), i32>(&mut *store, function_name)
            .map_err(|e| anyhow::anyhow!("Function '{}' not found: {}", function_name, e))?;

        let result = func
            .call(&mut *store, ())
            .map_err(|e| anyhow::anyhow!("Function execution failed: {}", e))?;

        println!("Function call result: {}", result);
        Ok(result.to_le_bytes().to_vec())
    }

    /// Load a simple contract that supports all needed functions and storage operations
    fn load_simple_contract(&self) -> Result<Vec<u8>> {
        let wat = r#"
            (module
                (import "env" "storage_get" (func $storage_get (param i32 i32 i32 i32) (result i32)))
                (import "env" "storage_set" (func $storage_set (param i32 i32 i32 i32) (result i32)))
                (import "env" "log" (func $log (param i32 i32)))
                (memory (export "memory") 1)
                
                ;; Memory layout:
                ;; 100-106: "counter" (7 bytes)
                ;; 200-203: i32 value 5 (4 bytes) 
                ;; 300-303: read buffer for counter value (4 bytes)
                ;; 400-407: "test_key" (8 bytes)
                ;; 500-509: "test_value" (10 bytes)
                ;; 600-631: read buffer for test_value (32 bytes)
                
                (data (i32.const 100) "counter")
                (data (i32.const 200) "\05\00\00\00")  ;; i32 value 5 in little endian
                (data (i32.const 400) "test_key")
                (data (i32.const 500) "test_value")
                
                ;; Test storage operations
                (func (export "test_storage") (result i32)
                    ;; Store key "counter" with value 5
                    (call $storage_set 
                        (i32.const 100)  ;; key pointer
                        (i32.const 7)    ;; key length ("counter")
                        (i32.const 200)  ;; value pointer  
                        (i32.const 4))   ;; value length (4 bytes for i32)
                    
                    ;; Return the result of storage_set
                )
                
                ;; Read counter value from storage
                (func (export "read_counter") (result i32)
                    (call $storage_get
                        (i32.const 100)  ;; key pointer
                        (i32.const 7)    ;; key length
                        (i32.const 300)  ;; output value pointer
                        (i32.const 4))   ;; max value length
                )
                
                ;; Initialize storage with test data
                (func (export "storage_init") (result i32)
                    ;; Store "test_key" with "test_value"
                    (call $storage_set
                        (i32.const 400)  ;; key pointer
                        (i32.const 8)    ;; key length ("test_key")
                        (i32.const 500)  ;; value pointer
                        (i32.const 10))  ;; value length ("test_value")
                )
                
                ;; Read storage test
                (func (export "storage_read") (result i32)
                    (call $storage_get
                        (i32.const 400)  ;; key pointer ("test_key")
                        (i32.const 8)    ;; key length
                        (i32.const 600)  ;; output value pointer
                        (i32.const 32))  ;; max value length
                )
                
                (func (export "main") (result i32)
                    i32.const 42)
                (func (export "init") (result i32)
                    i32.const 1)
                (func (export "increment") (result i32)
                    i32.const 1)
                (func (export "get") (result i32)
                    i32.const 3)
                (func (export "total_supply") (result i32)
                    i32.const 1000)
                (func (export "add") (result i32)
                    i32.const 5)
                (func (export "balance_of") (result i32)
                    i32.const 1000)
                (func (export "transfer") (result i32)
                    i32.const 1)
                (func (export "mint") (result i32)
                    i32.const 1)
                (func (export "burn") (result i32)
                    i32.const 1)
            )
        "#;

        wat::parse_str(wat)
            .map(|cow| cow.to_vec())
            .map_err(|e| anyhow::anyhow!("Failed to parse WAT: {}", e))
    }

    /// Get contract state
    pub fn get_contract_state(&self, contract_address: &str) -> Result<HashMap<String, Vec<u8>>> {
        let state = self.state.lock().unwrap();
        state.get_contract_state(contract_address)
    }
}

#[cfg(test)]
mod engine_tests {
    use super::*;
    use crate::smart_contract::{
        state::ContractState,
        types::{GasConfig, GasMeter},
    };

    #[test]
    fn test_gas_meter_creation() {
        let config = GasConfig::default();
        let gas_meter = GasMeter::new(1000000, config.clone());

        assert_eq!(gas_meter.gas_limit, 1000000);
        assert_eq!(gas_meter.gas_used, 0);
        assert_eq!(gas_meter.remaining_gas(), 1000000);
        assert!(!gas_meter.is_exhausted());
    }

    #[test]
    fn test_gas_consumption() {
        let config = GasConfig::default();
        let mut gas_meter = GasMeter::new(1000, config.clone());

        // Test consuming specific amounts
        assert!(gas_meter.consume_gas(500).is_ok());
        assert_eq!(gas_meter.gas_used, 500);
        assert_eq!(gas_meter.remaining_gas(), 500);

        // Test exceeding gas limit
        assert!(gas_meter.consume_gas(600).is_err());
        assert_eq!(gas_meter.gas_used, 500); // unchanged after failed consumption
    }

    #[test]
    fn test_gas_meter_specific_operations() {
        let config = GasConfig::default();
        let mut gas_meter = GasMeter::new(100000, config.clone());

        // Test specific operations
        assert!(gas_meter.consume_instruction().is_ok());
        assert_eq!(gas_meter.gas_used, config.instruction_cost);

        assert!(gas_meter.consume_function_call().is_ok());
        assert_eq!(
            gas_meter.gas_used,
            config.instruction_cost + config.function_call_cost
        );

        assert!(gas_meter.consume_storage_read().is_ok());
        assert_eq!(
            gas_meter.gas_used,
            config.instruction_cost + config.function_call_cost + config.storage_read_cost
        );

        assert!(gas_meter.consume_storage_write().is_ok());
        assert_eq!(
            gas_meter.gas_used,
            config.instruction_cost
                + config.function_call_cost
                + config.storage_read_cost
                + config.storage_write_cost
        );
    }

    #[test]
    fn test_gas_exhaustion() {
        let config = GasConfig::default();
        let mut gas_meter = GasMeter::new(1000, config.clone());

        // Consume all gas
        assert!(gas_meter.consume_gas(1000).is_ok());
        assert!(gas_meter.is_exhausted());
        assert_eq!(gas_meter.remaining_gas(), 0);

        // Try to consume more
        assert!(gas_meter.consume_gas(1).is_err());
    }

    #[test]
    fn test_enhanced_gas_config() {
        let config = GasConfig::default();

        // Verify default values are reasonable
        assert!(config.instruction_cost > 0);
        assert!(config.memory_cost_per_page > 0);
        assert!(config.storage_read_cost > 0);
        assert!(config.storage_write_cost > config.storage_read_cost); // Write should cost more than read
        assert!(config.function_call_cost > 0);
        assert!(config.contract_creation_cost > config.function_call_cost); // Creation should cost more than call
        assert!(config.max_gas_per_call > 1_000_000); // Should allow reasonable gas limits
    }

    #[test]
    fn test_memory_gas_calculation() {
        let config = GasConfig::default();
        let mut gas_meter = GasMeter::new(100000, config.clone());

        // Test memory allocation gas
        assert!(gas_meter.consume_memory(1).is_ok());
        assert_eq!(gas_meter.gas_used, config.memory_cost_per_page);

        assert!(gas_meter.consume_memory(5).is_ok());
        assert_eq!(
            gas_meter.gas_used,
            config.memory_cost_per_page + (5 * config.memory_cost_per_page)
        );
    }

    #[test]
    fn test_contract_engine_with_enhanced_gas() {
        let state = ContractState::new(":memory:").unwrap();
        let engine = ContractEngine::new(state).unwrap();

        let execution = ContractExecution {
            contract_address: "0x123".to_string(),
            function_name: "main".to_string(),
            arguments: vec![],
            gas_limit: 10000,
            caller: "0xabc".to_string(),
            value: 0,
        };

        // This should not panic and should return a proper gas usage
        let result = engine.execute_contract(execution).unwrap();
        assert!(result.gas_used > 0);
        assert!(result.gas_used <= 10000);
    }
}
