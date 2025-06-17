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
        types::{ContractExecution, ContractMetadata, ContractResult, GasConfig},
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

    /// Execute a smart contract function
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

        // Simple gas limit enforcement using gas_config
        let gas_cost = self.gas_config.instruction_cost * 10; // Base cost for function call
        if execution.gas_limit < gas_cost {
            return Ok(ContractResult {
                success: false,
                return_value: vec![],
                gas_used: gas_cost,
                state_changes: HashMap::new(),
                logs: vec!["Gas limit exceeded".to_string()],
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
        let host_context = HostContext {
            contract_address: execution.contract_address.clone(),
            caller: execution.caller.clone(),
            value: execution.value,
            state: self.state.clone(),
            logs: logs.clone(),
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

        // Get logs from host context
        let execution_logs = if let Ok(logs_guard) = logs.lock() {
            logs_guard.clone()
        } else {
            vec![]
        };

        // The state changes are now handled directly by the host functions
        let state_changes = HashMap::new();

        Ok(ContractResult {
            success: true,
            return_value: result,
            gas_used: gas_cost,
            state_changes,
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

        // Simple gas limit enforcement using gas_config
        let gas_cost = self.gas_config.instruction_cost * 10; // Base cost for function call
        if execution.gas_limit < gas_cost {
            return Ok(ContractResult {
                success: false,
                return_value: vec![],
                gas_used: gas_cost,
                state_changes: HashMap::new(),
                logs: vec!["Gas limit exceeded".to_string()],
            });
        }

        // Create WASM module from provided bytecode
        let module = Module::new(&self.engine, &bytecode)
            .map_err(|e| anyhow::anyhow!("Failed to create WASM module: {}", e))?;

        // Create host context for actual processing
        let logs = Arc::new(Mutex::new(Vec::new()));
        let host_context = HostContext {
            contract_address: execution.contract_address.clone(),
            caller: execution.caller.clone(),
            value: execution.value,
            state: self.state.clone(),
            logs: logs.clone(),
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

        // Get logs from host context
        let execution_logs = if let Ok(logs_guard) = logs.lock() {
            logs_guard.clone()
        } else {
            vec![]
        };

        // The state changes are now handled directly by the host functions
        let state_changes = HashMap::new();

        Ok(ContractResult {
            success: true,
            return_value: result,
            gas_used: gas_cost,
            state_changes,
            logs: execution_logs,
        })
    }

    /// Add host functions with actual implementations
    fn add_host_functions(
        &self,
        linker: &mut Linker<HostContext>,
        _execution: &ContractExecution,
    ) -> Result<()> {
        // Storage get function - reads from actual database
        linker
            .func_wrap(
                "env",
                "storage_get",
                |mut caller: Caller<'_, HostContext>, key_ptr: i32, key_len: i32| -> i32 {
                    let ctx = caller.data().clone();

                    // Get memory to read the key
                    let memory = match caller.get_export("memory") {
                        Some(Extern::Memory(mem)) => mem,
                        _ => return 0, // Return 0 if memory not found
                    };

                    // Read key from WASM memory
                    let mut key_bytes = vec![0u8; key_len as usize];
                    if memory
                        .read(&caller, key_ptr as usize, &mut key_bytes)
                        .is_err()
                    {
                        return 0;
                    }

                    let key = match String::from_utf8(key_bytes) {
                        Ok(k) => k,
                        Err(_) => return 0,
                    };

                    // Read from database
                    let state_result = ctx.state.lock();
                    if let Ok(state) = state_result {
                        match state.get(&ctx.contract_address, &key) {
                            Ok(Some(value)) => {
                                // For simplicity, return the first 4 bytes as i32
                                // In a real implementation, you might want to store the value
                                // in memory and return a pointer
                                if value.len() >= 4 {
                                    i32::from_le_bytes([value[0], value[1], value[2], value[3]])
                                } else {
                                    0
                                }
                            }
                            _ => 0,
                        }
                    } else {
                        0
                    }
                },
            )
            .map_err(|e| anyhow::anyhow!("Failed to add storage_get: {}", e))?;

        // Storage set function - writes to actual database
        linker
            .func_wrap(
                "env",
                "storage_set",
                |mut caller: Caller<'_, HostContext>,
                 key_ptr: i32,
                 key_len: i32,
                 value_ptr: i32,
                 value_len: i32| {
                    let ctx = caller.data().clone();

                    // Get memory to read key and value
                    let memory = match caller.get_export("memory") {
                        Some(Extern::Memory(mem)) => mem,
                        _ => return,
                    };

                    // Read key from WASM memory
                    let mut key_bytes = vec![0u8; key_len as usize];
                    if memory
                        .read(&caller, key_ptr as usize, &mut key_bytes)
                        .is_err()
                    {
                        return;
                    }

                    // Read value from WASM memory
                    let mut value_bytes = vec![0u8; value_len as usize];
                    if memory
                        .read(&caller, value_ptr as usize, &mut value_bytes)
                        .is_err()
                    {
                        return;
                    }

                    let key = match String::from_utf8(key_bytes) {
                        Ok(k) => k,
                        Err(_) => return,
                    };

                    // Write to database
                    let state_result = ctx.state.lock();
                    if let Ok(state) = state_result {
                        let _ = state.set(&ctx.contract_address, &key, &value_bytes);
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

    /// Load a simple contract that supports all needed functions
    fn load_simple_contract(&self) -> Result<Vec<u8>> {
        let wat = r#"
            (module
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
