//! Test utilities for smart contracts
//!
//! This module provides utilities for testing smart contract functionality.

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use anyhow::Result;

use super::unified_engine::{
    ContractExecutionRecord, ContractStateStorage, UnifiedContractMetadata,
};

/// Simple test storage implementation that doesn't use async
pub struct TestContractStorage {
    contracts: Arc<Mutex<HashMap<String, UnifiedContractMetadata>>>,
    state: Arc<Mutex<HashMap<String, Vec<u8>>>>,
    history: Arc<Mutex<HashMap<String, Vec<ContractExecutionRecord>>>>,
}

impl TestContractStorage {
    pub fn new() -> Self {
        Self {
            contracts: Arc::new(Mutex::new(HashMap::new())),
            state: Arc::new(Mutex::new(HashMap::new())),
            history: Arc::new(Mutex::new(HashMap::new())),
        }
    }
}

impl Default for TestContractStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl TestContractStorage {
    fn make_state_key(&self, contract: &str, key: &str) -> String {
        format!("{}:{}", contract, key)
    }
}

impl ContractStateStorage for TestContractStorage {
    fn store_contract_metadata(&self, metadata: &UnifiedContractMetadata) -> Result<()> {
        let mut contracts = self.contracts.lock().unwrap();
        contracts.insert(metadata.address.clone(), metadata.clone());
        Ok(())
    }

    fn get_contract_metadata(&self, address: &str) -> Result<Option<UnifiedContractMetadata>> {
        let contracts = self.contracts.lock().unwrap();
        Ok(contracts.get(address).cloned())
    }

    fn set_contract_state(&self, contract: &str, key: &str, value: &[u8]) -> Result<()> {
        let composite_key = self.make_state_key(contract, key);
        let mut state = self.state.lock().unwrap();
        state.insert(composite_key, value.to_vec());
        Ok(())
    }

    fn get_contract_state(&self, contract: &str, key: &str) -> Result<Option<Vec<u8>>> {
        let composite_key = self.make_state_key(contract, key);
        let state = self.state.lock().unwrap();
        Ok(state.get(&composite_key).cloned())
    }

    fn delete_contract_state(&self, contract: &str, key: &str) -> Result<()> {
        let composite_key = self.make_state_key(contract, key);
        let mut state = self.state.lock().unwrap();
        state.remove(&composite_key);
        Ok(())
    }

    fn list_contracts(&self) -> Result<Vec<String>> {
        let contracts = self.contracts.lock().unwrap();
        Ok(contracts.keys().cloned().collect())
    }

    fn store_execution(&self, execution: &ContractExecutionRecord) -> Result<()> {
        let mut history = self.history.lock().unwrap();
        history
            .entry(execution.contract_address.clone())
            .or_default()
            .push(execution.clone());
        Ok(())
    }

    fn get_execution_history(&self, contract: &str) -> Result<Vec<ContractExecutionRecord>> {
        let history = self.history.lock().unwrap();
        Ok(history.get(contract).cloned().unwrap_or_default())
    }
}
