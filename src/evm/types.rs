use ethers::types::{H160, H256, U256};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractState {
    pub address: H160,
    pub code: Vec<u8>,
    pub storage: HashMap<H256, H256>,
    pub balance: U256,
    pub nonce: U256,
}


#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Transaction {
    pub from: H160,
    pub to: H160,
    pub value: U256,
    pub gas_limit: U256,
    pub gas_price: U256,
    pub data: Vec<u8>,
    pub nonce: U256,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CallResult {
    pub success: bool,
    pub return_data: Vec<u8>,
    pub gas_used: U256,
    pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TransactionType {
    Regular,
    ContractDeployment,
    ContaractCall,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ScriptContext {
    pub tx_hash: String,
    pub input_index: usize,
    pub output_index: usize,
    pub block_height: i32,
    pub signature: Vec<u8>,
    pub public_key: Vec<u8>,
}
