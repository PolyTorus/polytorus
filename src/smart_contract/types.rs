//! Smart contract types and definitions

use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

/// Gas configuration
#[derive(Debug, Clone)]
pub struct GasConfig {
    pub instruction_cost: u64,
    pub memory_cost: u64,
    pub storage_cost: u64,
    pub max_gas_per_call: u64,
}

impl Default for GasConfig {
    fn default() -> Self {
        Self {
            instruction_cost: 1,
            memory_cost: 1,
            storage_cost: 100,
            max_gas_per_call: 1_000_000,
        }
    }
}

/// Contract metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractMetadata {
    pub address: String,
    pub creator: String,
    pub created_at: u64,
    pub bytecode_hash: String,
    pub abi: Option<String>,
}
