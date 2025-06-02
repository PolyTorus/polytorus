//! WASM contract execution engine

use crate::smart_contract::types::{ContractExecution, ContractResult, ContractMetadata, GasConfig};
use crate::smart_contract::state::ContractState;
use crate::smart_contract::contract::SmartContract;
use crate::Result;
use failure::format_err;
use std::collections::HashMap;
use std::sync::{Arc, Mutex};
use wasmtime::*;

/// WASM contract execution engine
pub struct ContractEngine {
    engine: Engine,
    state: Arc<Mutex<ContractState>>,
    gas_config: GasConfig,
}

/// Host functions for WASM contracts
struct HostContext {
    state: Arc<Mutex<ContractState>>,
    contract_address: String,
    caller: String,
    value: u64,
    gas_used: Arc<Mutex<u64>>,
    gas_limit: u64,
    gas_config: GasConfig,
    logs: Arc<Mutex<Vec<String>>>,
    state_changes: Arc<Mutex<HashMap<String, Vec<u8>>>>,
}

impl ContractEngine {
    /// Create a new contract engine
    pub fn new(state: ContractState) -> Result<Self> {
        let engine = Engine::default();
        Ok(Self {
            engine,
            state: Arc::new(Mutex::new(state)),
            gas_config: GasConfig::default(),
        })
    }

    /// Deploy a smart contract
    pub fn deploy_contract(&self, contract: &SmartContract) -> Result<()> {
        let state = self.state.lock().unwrap();
        contract.deploy(&*state)?;
        Ok(())
    }

    /// List all deployed contracts
    pub fn list_contracts(&self) -> Result<Vec<ContractMetadata>> {
        let _state = self.state.lock().unwrap();
        // In a real implementation, you would iterate through all contract entries
        // For now, we'll return an empty list as a placeholder
        Ok(vec![])
    }

    /// Execute a smart contract function
    pub fn execute_contract(&self, execution: ContractExecution) -> Result<ContractResult> {
        let state = self.state.lock().unwrap();

        // Get contract metadata
        let _contract_metadata = state.get_contract(&execution.contract_address)?
            .ok_or_else(|| format_err!("Contract not found: {}", execution.contract_address))?;

        // Load contract bytecode (in a real implementation, this would be stored separately)
        let bytecode = self.load_contract_bytecode(&execution.contract_address)?;

        drop(state); // Release lock before execution

        // Check gas limit against configuration
        if execution.gas_limit > self.gas_config.max_gas_per_call {
            return Err(format_err!("Gas limit {} exceeds maximum allowed {}", 
                execution.gas_limit, self.gas_config.max_gas_per_call));
        }

        // Create WASM module
        let module = Module::new(&self.engine, &bytecode)
            .map_err(|e| format_err!("Failed to create WASM module: {}", e))?;

        // Create store for gas metering (fuel APIs have changed in recent wasmtime versions)
        let mut store = Store::new(&self.engine, ());

        // Set up host context
        let host_context = Arc::new(Mutex::new(HostContext {
            state: self.state.clone(),
            contract_address: execution.contract_address.clone(),
            caller: execution.caller,
            value: execution.value,
            gas_used: Arc::new(Mutex::new(0)),
            gas_limit: execution.gas_limit,
            gas_config: self.gas_config.clone(),
            logs: Arc::new(Mutex::new(Vec::new())),
            state_changes: Arc::new(Mutex::new(HashMap::new())),
        }));

        // Create linker and add host functions
        let mut linker = Linker::new(&self.engine);
        self.add_host_functions(&mut linker, host_context.clone())?;

        // Instantiate the module
        let instance = linker.instantiate(&mut store, &module)
            .map_err(|e| format_err!("Failed to instantiate module: {}", e))?;

        // Call the function with gas checking
        let result = self.call_function(&mut store, &instance, &execution.function_name, &execution.arguments, &host_context)?;

        // Get execution results
        let host_ctx = host_context.lock().unwrap();
        let gas_used = *host_ctx.gas_used.lock().unwrap();
        let logs = host_ctx.logs.lock().unwrap().clone();
        let state_changes = host_ctx.state_changes.lock().unwrap().clone();

        // Check if gas limit was exceeded
        if gas_used > host_ctx.gas_limit {
            return Ok(ContractResult {
                success: false,
                return_value: Vec::new(),
                gas_used,
                logs,
                state_changes: HashMap::new(), // Don't apply state changes on gas limit exceeded
            });
        }

        // Apply state changes
        if !state_changes.is_empty() {
            let state = self.state.lock().unwrap();
            state.apply_changes(&state_changes)?;
        }

        Ok(ContractResult {
            success: true,
            return_value: result,
            gas_used,
            logs,
            state_changes,
        })
    }

    /// Add host functions to the linker
    fn add_host_functions(&self, linker: &mut Linker<()>, context: Arc<Mutex<HostContext>>) -> Result<()> {
        // Storage functions
        let ctx_clone = context.clone();
        linker.func_wrap("env", "storage_get", move |mut caller: Caller<'_, ()>, key_ptr: i32, key_len: i32| -> i32 {
            let memory = caller.get_export("memory").unwrap().into_memory().unwrap();
            let data = memory.data(&caller);
            
            if key_ptr < 0 || key_len < 0 || (key_ptr + key_len) as usize > data.len() {
                return 0; // Invalid memory access
            }
            
            let key = &data[key_ptr as usize..(key_ptr + key_len) as usize];
            let key_str = String::from_utf8_lossy(key);
            
            let ctx = ctx_clone.lock().unwrap();
            
            // Add gas cost for storage read
            {
                let mut gas_used = ctx.gas_used.lock().unwrap();
                *gas_used += ctx.gas_config.storage_cost;
            }
            
            let state = ctx.state.lock().unwrap();
            
            match state.get(&ctx.contract_address, &key_str) {
                Ok(Some(value)) => {
                    // In a real implementation, we'd need to allocate memory in WASM
                    // and return the pointer/length
                    value.len() as i32
                }
                _ => 0
            }
        }).map_err(|e| format_err!("Failed to add storage_get: {}", e))?;

        let ctx_clone = context.clone();
        linker.func_wrap("env", "storage_set", move |mut caller: Caller<'_, ()>, key_ptr: i32, key_len: i32, value_ptr: i32, value_len: i32| {
            let memory = caller.get_export("memory").unwrap().into_memory().unwrap();
            let data = memory.data(&caller);
            
            if key_ptr < 0 || key_len < 0 || value_ptr < 0 || value_len < 0 ||
               (key_ptr + key_len) as usize > data.len() ||
               (value_ptr + value_len) as usize > data.len() {
                return; // Invalid memory access
            }
            
            let key = &data[key_ptr as usize..(key_ptr + key_len) as usize];
            let value = &data[value_ptr as usize..(value_ptr + value_len) as usize];
            let key_str = String::from_utf8_lossy(key);
            
            let ctx = ctx_clone.lock().unwrap();
            
            // Add gas cost for storage write
            {
                let mut gas_used = ctx.gas_used.lock().unwrap();
                *gas_used += ctx.gas_config.storage_cost;
            }
            
            let storage_key = format!("state:{}:{}", ctx.contract_address, key_str);
            ctx.state_changes.lock().unwrap().insert(storage_key, value.to_vec());
        }).map_err(|e| format_err!("Failed to add storage_set: {}", e))?;

        // Logging function
        let ctx_clone = context.clone();
        linker.func_wrap("env", "log", move |mut caller: Caller<'_, ()>, msg_ptr: i32, msg_len: i32| {
            let memory = caller.get_export("memory").unwrap().into_memory().unwrap();
            let data = memory.data(&caller);
            
            if msg_ptr < 0 || msg_len < 0 || (msg_ptr + msg_len) as usize > data.len() {
                return; // Invalid memory access
            }
            
            let msg = &data[msg_ptr as usize..(msg_ptr + msg_len) as usize];
            let msg_str = String::from_utf8_lossy(msg);
            
            let ctx = ctx_clone.lock().unwrap();
            ctx.logs.lock().unwrap().push(msg_str.to_string());
        }).map_err(|e| format_err!("Failed to add log: {}", e))?;

        // Caller info functions
        let ctx_clone = context.clone();
        linker.func_wrap("env", "get_caller", move |_caller: Caller<'_, ()>| -> i32 {
            let ctx = ctx_clone.lock().unwrap();
            // Return a simple hash of the caller address for demonstration
            ctx.caller.len() as i32
        }).map_err(|e| format_err!("Failed to add get_caller: {}", e))?;

        // Value transfer functions
        let ctx_clone = context.clone();
        linker.func_wrap("env", "get_value", move |_caller: Caller<'_, ()>| -> i64 {
            let ctx = ctx_clone.lock().unwrap();
            ctx.value as i64
        }).map_err(|e| format_err!("Failed to add get_value: {}", e))?;

        Ok(())
    }

    /// Call a function in the WASM module
    fn call_function(&self, store: &mut Store<()>, instance: &Instance, function_name: &str, args: &[u8], context: &Arc<Mutex<HostContext>>) -> Result<Vec<u8>> {
        // Add basic gas cost for function call
        {
            let ctx = context.lock().unwrap();
            let mut gas_used = ctx.gas_used.lock().unwrap();
            *gas_used += self.gas_config.instruction_cost * 10; // Base cost for function call
        }

        // Try different function signatures based on the function name and arguments
        match function_name {
            "init" => {
                if args.len() >= 4 {
                    // Function that takes one i32 parameter
                    let arg = i32::from_le_bytes([args[0], args[1], args[2], args[3]]);
                    let func = instance.get_typed_func::<i32, i32>(&mut *store, function_name)
                        .map_err(|e| format_err!("Function '{}' not found: {}", function_name, e))?;
                    let result = func.call(store, arg)
                        .map_err(|e| format_err!("Function execution failed: {}", e))?;
                    Ok(result.to_le_bytes().to_vec())
                } else {
                    // Function with no parameters
                    let func = instance.get_typed_func::<(), i32>(&mut *store, function_name)
                        .map_err(|e| format_err!("Function '{}' not found: {}", function_name, e))?;
                    let result = func.call(store, ())
                        .map_err(|e| format_err!("Function execution failed: {}", e))?;
                    Ok(result.to_le_bytes().to_vec())
                }
            },
            "transfer" | "mint" | "add" => {
                if args.len() >= 8 {
                    // Function that takes two i32 parameters
                    let arg1 = i32::from_le_bytes([args[0], args[1], args[2], args[3]]);
                    let arg2 = i32::from_le_bytes([args[4], args[5], args[6], args[7]]);
                    let func = instance.get_typed_func::<(i32, i32), i32>(&mut *store, function_name)
                        .map_err(|e| format_err!("Function '{}' not found: {}", function_name, e))?;
                    let result = func.call(store, (arg1, arg2))
                        .map_err(|e| format_err!("Function execution failed: {}", e))?;
                    Ok(result.to_le_bytes().to_vec())
                } else if args.len() >= 4 {
                    // Function that takes one i32 parameter
                    let arg = i32::from_le_bytes([args[0], args[1], args[2], args[3]]);
                    let func = instance.get_typed_func::<i32, i32>(&mut *store, function_name)
                        .map_err(|e| format_err!("Function '{}' not found: {}", function_name, e))?;
                    let result = func.call(store, arg)
                        .map_err(|e| format_err!("Function execution failed: {}", e))?;
                    Ok(result.to_le_bytes().to_vec())
                } else {
                    // Function with no parameters
                    let func = instance.get_typed_func::<(), i32>(&mut *store, function_name)
                        .map_err(|e| format_err!("Function '{}' not found: {}", function_name, e))?;
                    let result = func.call(store, ())
                        .map_err(|e| format_err!("Function execution failed: {}", e))?;
                    Ok(result.to_le_bytes().to_vec())
                }
            },
            "balance_of" | "burn" => {
                if args.len() >= 4 {
                    // Function that takes one i32 parameter
                    let arg = i32::from_le_bytes([args[0], args[1], args[2], args[3]]);
                    let func = instance.get_typed_func::<i32, i32>(&mut *store, function_name)
                        .map_err(|e| format_err!("Function '{}' not found: {}", function_name, e))?;
                    let result = func.call(store, arg)
                        .map_err(|e| format_err!("Function execution failed: {}", e))?;
                    Ok(result.to_le_bytes().to_vec())
                } else {
                    return Err(format_err!("Function '{}' requires one parameter", function_name));
                }
            },
            _ => {
                // Default: try function with no parameters
                let func = instance.get_typed_func::<(), i32>(&mut *store, function_name)
                    .map_err(|e| format_err!("Function '{}' not found: {}", function_name, e))?;
                let result = func.call(store, ())
                    .map_err(|e| format_err!("Function execution failed: {}", e))?;
                Ok(result.to_le_bytes().to_vec())
            }
        }
    }

    /// Load contract bytecode (placeholder implementation)
    fn load_contract_bytecode(&self, contract_address: &str) -> Result<Vec<u8>> {
        // Check if we have a specific contract deployed
        let state = self.state.lock().unwrap();
        if let Ok(Some(contract)) = state.get_contract(contract_address) {
            // In a real implementation, we'd load the actual bytecode from the contract metadata
            // For now, we'll return different contracts based on the address pattern
            
            if contract.address.contains("counter") {
                // Load counter contract
                if let Ok(bytecode) = std::fs::read("/home/shiro/workspace/polytorus/contracts/counter.wat") {
                    return wat::parse_bytes(&bytecode)
                        .map(|cow| cow.to_vec())
                        .map_err(|e| format_err!("Failed to parse counter WAT: {}", e));
                }
            } else if contract.address.contains("token") {
                // Load token contract
                if let Ok(bytecode) = std::fs::read("/home/shiro/workspace/polytorus/contracts/token.wat") {
                    return wat::parse_bytes(&bytecode)
                        .map(|cow| cow.to_vec())
                        .map_err(|e| format_err!("Failed to parse token WAT: {}", e));
                }
            }
        }
        
        // Fallback to simple contract
        let wat = r#"
            (module
                (func (export "main") (result i32)
                    i32.const 42)
            )
        "#;
        
        wat::parse_str(wat)
            .map(|cow| cow.to_vec())
            .map_err(|e| format_err!("Failed to parse WAT: {}", e))
    }

    /// Get contract state
    pub fn get_contract_state(&self, contract_address: &str) -> Result<HashMap<String, Vec<u8>>> {
        let state = self.state.lock().unwrap();
        state.get_contract_state(contract_address)
    }
}
