//! Smart contract state management

use std::collections::HashMap;

use serde::{Deserialize, Serialize};
use sled;

use crate::{smart_contract::types::ContractMetadata, Result};

/// Contract state storage
#[derive(Debug, Clone)]
pub struct ContractState {
    db: sled::Db,
}

/// State entry for contract storage
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StateEntry {
    pub key: String,
    pub value: Vec<u8>,
    pub contract_address: String,
}

impl ContractState {
    /// Create new contract state storage
    pub fn new(db_path: &str) -> Result<Self> {
        let db = sled::open(db_path)?;
        Ok(Self { db })
    }

    /// Store contract metadata
    pub fn store_contract(&self, metadata: &ContractMetadata) -> Result<()> {
        let key = format!("contract:{}", metadata.address);
        let data = bincode::serialize(metadata)?;
        self.db.insert(key.as_bytes(), data)?;
        self.db.flush()?;
        Ok(())
    }

    /// Get contract metadata
    pub fn get_contract(&self, address: &str) -> Result<Option<ContractMetadata>> {
        let key = format!("contract:{}", address);
        if let Some(data) = self.db.get(key.as_bytes())? {
            let metadata: ContractMetadata = bincode::deserialize(&data)?;
            Ok(Some(metadata))
        } else {
            Ok(None)
        }
    }

    /// Store contract state value
    pub fn set(&self, contract_address: &str, key: &str, value: &[u8]) -> Result<()> {
        let storage_key = format!("state:{}:{}", contract_address, key);
        self.db.insert(storage_key.as_bytes(), value)?;
        self.db.flush()?;
        Ok(())
    }

    /// Get contract state value
    pub fn get(&self, contract_address: &str, key: &str) -> Result<Option<Vec<u8>>> {
        let storage_key = format!("state:{}:{}", contract_address, key);
        if let Some(data) = self.db.get(storage_key.as_bytes())? {
            Ok(Some(data.to_vec()))
        } else {
            Ok(None)
        }
    }

    /// Apply multiple state changes atomically
    pub fn apply_changes(&self, changes: &HashMap<String, Vec<u8>>) -> Result<()> {
        let mut batch = sled::Batch::default();
        for (key, value) in changes {
            batch.insert(key.as_bytes(), value.as_slice());
        }
        self.db.apply_batch(batch)?;
        self.db.flush()?;
        Ok(())
    }

    /// Get all state entries for a contract
    pub fn get_contract_state(&self, contract_address: &str) -> Result<HashMap<String, Vec<u8>>> {
        let prefix = format!("state:{}:", contract_address);
        let mut state = HashMap::new();

        for item in self.db.scan_prefix(prefix.as_bytes()) {
            let (key, value) = item?;
            let key_str = String::from_utf8(key.to_vec())?;
            // Remove the prefix to get the actual key
            if let Some(actual_key) = key_str.strip_prefix(&prefix) {
                state.insert(actual_key.to_string(), value.to_vec());
            }
        }

        Ok(state)
    }

    /// Delete contract and all its state
    pub fn delete_contract(&self, contract_address: &str) -> Result<()> {
        // Delete contract metadata
        let contract_key = format!("contract:{}", contract_address);
        self.db.remove(contract_key.as_bytes())?;

        // Delete all state entries
        let state_prefix = format!("state:{}:", contract_address);
        let keys_to_delete: Vec<_> = self
            .db
            .scan_prefix(state_prefix.as_bytes())
            .filter_map(|item| item.ok().map(|(k, _)| k))
            .collect();

        for key in keys_to_delete {
            self.db.remove(&key)?;
        }

        self.db.flush()?;
        Ok(())
    }

    /// List all deployed contracts
    pub fn list_contracts(&self) -> Result<Vec<ContractMetadata>> {
        self.list_contracts_with_limit(None)
    }

    /// List deployed contracts with optional limit
    pub fn list_contracts_with_limit(&self, limit: Option<usize>) -> Result<Vec<ContractMetadata>> {
        let mut contracts = Vec::new();
        let prefix = b"contract:";

        // Use iterator with timeout protection
        let iter = self.db.scan_prefix(prefix);
        let mut count = 0;
        let max_items = limit.unwrap_or(100); // Default to 100 contracts

        for item_result in iter {
            count += 1;

            if count > max_items {
                break;
            }

            match item_result {
                Ok((key, value)) => {
                    let key_str = String::from_utf8_lossy(&key);

                    if !key_str.starts_with("contract:") {
                        continue;
                    }

                    match bincode::deserialize::<ContractMetadata>(&value) {
                        Ok(metadata) => {
                            contracts.push(metadata);
                        }
                        Err(e) => {
                            eprintln!(
                                "Warning: Failed to deserialize contract metadata for key {}: {}",
                                key_str, e
                            );
                            continue;
                        }
                    }
                }
                Err(e) => {
                    eprintln!("Warning: Failed to read database entry: {}", e);
                    continue;
                }
            }
        }

        Ok(contracts)
    }

    /// Store generic data with a key
    pub fn store_data(&self, key: &str, data: &[u8]) -> Result<()> {
        self.db.insert(key.as_bytes(), data)?;
        self.db.flush()?;
        Ok(())
    }

    /// Get generic data by key
    pub fn get_data(&self, key: &str) -> Result<Option<Vec<u8>>> {
        if let Some(data) = self.db.get(key.as_bytes())? {
            Ok(Some(data.to_vec()))
        } else {
            Ok(None)
        }
    }

    /// Remove data by key
    pub fn remove_data(&self, key: &str) -> Result<()> {
        self.db.remove(key.as_bytes())?;
        self.db.flush()?;
        Ok(())
    }

    /// Scan for keys with a given prefix
    pub fn scan_prefix(&self, prefix: &str) -> Result<Vec<String>> {
        let mut keys = Vec::new();
        for item in self.db.scan_prefix(prefix.as_bytes()) {
            let (key, _) = item?;
            let key_str = String::from_utf8(key.to_vec())?;
            keys.push(key_str);
        }
        Ok(keys)
    }
}
