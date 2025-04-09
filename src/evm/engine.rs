use std::sync::{Arc, Mutex};

use crate::evm::types::{ContractState, CallResult, Transaction};
use crate::Result;
use ethers::etherscan::gas;
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

    pub fn deploy(
        &self,
        from: H160,
        code: Vec<u8>,
        value: U256,
        gas_limit: U256,
    ) -> Result<CallResult> {
        let tx = Transaction {
            from,
            to: H160::zero(),
            value,
            gas_limit,
            gas_price: U256::zero(),
            data: code.clone(),
            nonce: U256::zero(),
        };

        self.execute_transaction(tx)
    }

    pub fn execute_transaction(&self, tx: Transaction) -> Result<CallResult> {
        // Implement transaction execution logic here
        todo!("Implement transaction execution")
    }
}
