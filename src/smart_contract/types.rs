//! Smart contract types and definitions

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

/// Smart contract execution result
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractResult {
    pub success: bool,
    pub return_value: Vec<u8>,
    pub gas_used: u64,
    pub logs: Vec<String>,
    pub state_changes: HashMap<String, Vec<u8>>,
}

/// Contract deployment parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractDeployment {
    pub bytecode: Vec<u8>,
    pub constructor_args: Vec<u8>,
    pub gas_limit: u64,
}

/// Contract execution parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractExecution {
    pub contract_address: String,
    pub function_name: String,
    pub arguments: Vec<u8>,
    pub gas_limit: u64,
    pub caller: String,
    pub value: u64,
}

/// Gas configuration with detailed cost structure
#[derive(Debug, Clone)]
pub struct GasConfig {
    pub instruction_cost: u64,
    pub memory_cost_per_page: u64,
    pub storage_read_cost: u64,
    pub storage_write_cost: u64,
    pub function_call_cost: u64,
    pub contract_creation_cost: u64,
    pub max_gas_per_call: u64,
    pub max_memory_pages: u32,
}

impl Default for GasConfig {
    fn default() -> Self {
        Self {
            instruction_cost: 1,
            memory_cost_per_page: 1000,
            storage_read_cost: 200,
            storage_write_cost: 5000,
            function_call_cost: 700,
            contract_creation_cost: 32000,
            max_gas_per_call: 10_000_000,
            max_memory_pages: 256,
        }
    }
}

/// Gas meter for tracking execution costs
#[derive(Debug, Clone)]
pub struct GasMeter {
    pub gas_limit: u64,
    pub gas_used: u64,
    pub config: GasConfig,
}

impl GasMeter {
    pub fn new(gas_limit: u64, config: GasConfig) -> Self {
        Self {
            gas_limit,
            gas_used: 0,
            config,
        }
    }

    pub fn consume_gas(&mut self, amount: u64) -> Result<(), String> {
        if self.gas_used + amount > self.gas_limit {
            return Err(format!(
                "Out of gas: trying to use {} gas, have {} used, limit {}",
                amount, self.gas_used, self.gas_limit
            ));
        }
        self.gas_used += amount;
        Ok(())
    }

    pub fn consume_instruction(&mut self) -> Result<(), String> {
        self.consume_gas(self.config.instruction_cost)
    }

    pub fn consume_memory(&mut self, pages: u32) -> Result<(), String> {
        let cost = self.config.memory_cost_per_page * pages as u64;
        self.consume_gas(cost)
    }

    pub fn consume_storage_read(&mut self) -> Result<(), String> {
        self.consume_gas(self.config.storage_read_cost)
    }

    pub fn consume_storage_write(&mut self) -> Result<(), String> {
        self.consume_gas(self.config.storage_write_cost)
    }

    pub fn consume_function_call(&mut self) -> Result<(), String> {
        self.consume_gas(self.config.function_call_cost)
    }

    pub fn remaining_gas(&self) -> u64 {
        self.gas_limit.saturating_sub(self.gas_used)
    }

    pub fn is_exhausted(&self) -> bool {
        self.gas_used >= self.gas_limit
    }
}

/// Contract metadata with enhanced ABI support
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractMetadata {
    pub address: String,
    pub creator: String,
    pub created_at: u64,
    pub bytecode_hash: String,
    pub abi: Option<ContractAbi>,
}

/// Contract ABI (Application Binary Interface)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractAbi {
    pub functions: Vec<AbiFunction>,
    pub events: Vec<AbiEvent>,
    pub constructor: Option<AbiFunction>,
}

/// ABI function definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbiFunction {
    pub name: String,
    pub inputs: Vec<AbiParameter>,
    pub outputs: Vec<AbiParameter>,
    pub state_mutability: StateMutability,
}

/// ABI event definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbiEvent {
    pub name: String,
    pub inputs: Vec<AbiParameter>,
    pub anonymous: bool,
}

/// ABI parameter definition
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AbiParameter {
    pub name: String,
    pub param_type: AbiType,
    pub indexed: bool, // for events
}

/// ABI type system
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AbiType {
    Bool,
    Int { size: u16 },
    Uint { size: u16 },
    Address,
    Bytes { size: Option<u16> },
    String,
    Array { inner: Box<AbiType>, size: Option<u64> },
    Tuple { components: Vec<AbiParameter> },
}

/// State mutability of contract functions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum StateMutability {
    Pure,
    View,
    NonPayable,
    Payable,
}

impl ContractAbi {
    pub fn new() -> Self {
        Self {
            functions: Vec::new(),
            events: Vec::new(),
            constructor: None,
        }
    }

    pub fn add_function(&mut self, function: AbiFunction) {
        self.functions.push(function);
    }

    pub fn add_event(&mut self, event: AbiEvent) {
        self.events.push(event);
    }

    pub fn set_constructor(&mut self, constructor: AbiFunction) {
        self.constructor = Some(constructor);
    }

    pub fn get_function(&self, name: &str) -> Option<&AbiFunction> {
        self.functions.iter().find(|f| f.name == name)
    }

    pub fn get_event(&self, name: &str) -> Option<&AbiEvent> {
        self.events.iter().find(|e| e.name == name)
    }

    pub fn validate_function_call(&self, function_name: &str, input_data: &[u8]) -> Result<(), String> {
        let function = self.get_function(function_name)
            .ok_or_else(|| format!("Function {} not found in ABI", function_name))?;
        
        // Basic validation - in a real implementation, this would decode and validate the input data
        if input_data.len() < 4 {
            return Err("Input data too short for function call".to_string());
        }

        // Validate that we have the expected number of parameters
        // This is a simplified check - real ABI validation would be more complex
        let expected_params = function.inputs.len();
        if expected_params > 0 && input_data.len() < 4 + (expected_params * 32) {
            return Err(format!(
                "Input data length {} insufficient for {} parameters",
                input_data.len(),
                expected_params
            ));
        }

        Ok(())
    }
}

#[cfg(test)]
mod abi_tests {
    use super::*;

    #[test]
    fn test_contract_abi_creation() {
        let abi = ContractAbi::new();
        
        assert!(abi.functions.is_empty());
        assert!(abi.events.is_empty());
        assert!(abi.constructor.is_none());
    }

    #[test]
    fn test_abi_function_management() {
        let mut abi = ContractAbi::new();
        
        let function = AbiFunction {
            name: "transfer".to_string(),
            inputs: vec![
                AbiParameter {
                    name: "to".to_string(),
                    param_type: AbiType::Address,
                    indexed: false,
                },
                AbiParameter {
                    name: "amount".to_string(),
                    param_type: AbiType::Uint { size: 256 },
                    indexed: false,
                },
            ],
            outputs: vec![
                AbiParameter {
                    name: "success".to_string(),
                    param_type: AbiType::Bool,
                    indexed: false,
                },
            ],
            state_mutability: StateMutability::NonPayable,
        };
        
        abi.add_function(function.clone());
        assert_eq!(abi.functions.len(), 1);
        
        let retrieved = abi.get_function("transfer");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "transfer");
        
        let not_found = abi.get_function("nonexistent");
        assert!(not_found.is_none());
    }

    #[test]
    fn test_abi_event_management() {
        let mut abi = ContractAbi::new();
        
        let event = AbiEvent {
            name: "Transfer".to_string(),
            inputs: vec![
                AbiParameter {
                    name: "from".to_string(),
                    param_type: AbiType::Address,
                    indexed: true,
                },
                AbiParameter {
                    name: "to".to_string(),
                    param_type: AbiType::Address,
                    indexed: true,
                },
                AbiParameter {
                    name: "value".to_string(),
                    param_type: AbiType::Uint { size: 256 },
                    indexed: false,
                },
            ],
            anonymous: false,
        };
        
        abi.add_event(event.clone());
        assert_eq!(abi.events.len(), 1);
        
        let retrieved = abi.get_event("Transfer");
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "Transfer");
    }

    #[test]
    fn test_abi_function_call_validation() {
        let mut abi = ContractAbi::new();
        
        let function = AbiFunction {
            name: "transfer".to_string(),
            inputs: vec![
                AbiParameter {
                    name: "to".to_string(),
                    param_type: AbiType::Address,
                    indexed: false,
                },
                AbiParameter {
                    name: "amount".to_string(),
                    param_type: AbiType::Uint { size: 256 },
                    indexed: false,
                },
            ],
            outputs: vec![],
            state_mutability: StateMutability::NonPayable,
        };
        
        abi.add_function(function);
        
        // Test valid function call
        let valid_input = vec![0u8; 68]; // 4 bytes selector + 64 bytes for two parameters
        assert!(abi.validate_function_call("transfer", &valid_input).is_ok());
        
        // Test invalid function name
        assert!(abi.validate_function_call("nonexistent", &valid_input).is_err());
        
        // Test insufficient input data
        let short_input = vec![0u8; 3];
        assert!(abi.validate_function_call("transfer", &short_input).is_err());
        
        // Test insufficient parameter data
        let insufficient_input = vec![0u8; 36]; // Less than required for 2 parameters
        assert!(abi.validate_function_call("transfer", &insufficient_input).is_err());
    }

    #[test]
    fn test_abi_type_system() {
        let bool_type = AbiType::Bool;
        let int_type = AbiType::Int { size: 256 };
        let uint_type = AbiType::Uint { size: 64 };
        let address_type = AbiType::Address;
        let bytes_type = AbiType::Bytes { size: Some(32) };
        let dynamic_bytes_type = AbiType::Bytes { size: None };
        let string_type = AbiType::String;
        
        // Verify types can be created without issues
        assert!(matches!(bool_type, AbiType::Bool));
        assert!(matches!(int_type, AbiType::Int { size: 256 }));
        assert!(matches!(uint_type, AbiType::Uint { size: 64 }));
        assert!(matches!(address_type, AbiType::Address));
        assert!(matches!(bytes_type, AbiType::Bytes { size: Some(32) }));
        assert!(matches!(dynamic_bytes_type, AbiType::Bytes { size: None }));
        assert!(matches!(string_type, AbiType::String));
    }

    #[test]
    fn test_state_mutability() {
        let pure = StateMutability::Pure;
        let view = StateMutability::View;
        let non_payable = StateMutability::NonPayable;
        let payable = StateMutability::Payable;
        
        // Verify all mutability states can be created
        assert!(matches!(pure, StateMutability::Pure));
        assert!(matches!(view, StateMutability::View));
        assert!(matches!(non_payable, StateMutability::NonPayable));
        assert!(matches!(payable, StateMutability::Payable));
    }

    #[test]
    fn test_complex_abi_types() {
        let array_type = AbiType::Array {
            inner: Box::new(AbiType::Uint { size: 256 }),
            size: Some(10),
        };
        
        let dynamic_array_type = AbiType::Array {
            inner: Box::new(AbiType::Address),
            size: None,
        };
        
        let tuple_type = AbiType::Tuple {
            components: vec![
                AbiParameter {
                    name: "field1".to_string(),
                    param_type: AbiType::Uint { size: 256 },
                    indexed: false,
                },
                AbiParameter {
                    name: "field2".to_string(),
                    param_type: AbiType::Address,
                    indexed: false,
                },
            ],
        };
        
        // Verify complex types can be created
        assert!(matches!(array_type, AbiType::Array { .. }));
        assert!(matches!(dynamic_array_type, AbiType::Array { .. }));
        assert!(matches!(tuple_type, AbiType::Tuple { .. }));
    }
}
