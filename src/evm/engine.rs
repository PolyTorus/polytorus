use std::sync::{Arc, Mutex};

use crate::evm::types::{ContractState, CallResult, Transaction};
use crate::Result;
use ethers::types::{Address, H160, H256, U256};
use primitive_types::H256 as PrimitiveH256;
use revm::primitives::HashMap;


pub struct Engine {
    pub contract_state: Arc<Mutex<HashMap<H160, ContractState>>>,
}

impl Engine {
    pub fn new() -> Self {
        Engine { contract_state: Arc::new(Mutex::new(HashMap::new())) }
    }
}