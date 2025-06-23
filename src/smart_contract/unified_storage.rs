//! Unified Contract State Storage Implementation
//!
//! This module provides a unified storage backend for all smart contracts,
//! replacing the fragmented storage approaches.

use std::{collections::HashMap, path::Path, sync::Arc};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use sled::{Db, Tree};
use tokio::sync::RwLock;

use super::unified_engine::{
    ContractExecutionRecord, ContractStateStorage, UnifiedContractMetadata,
};

/// Unified storage implementation using Sled database
pub struct UnifiedContractStorage {
    db: Arc<Db>,
    contracts_tree: Tree,
    state_tree: Tree,
    history_tree: Tree,
    memory_cache: Arc<RwLock<HashMap<String, Vec<u8>>>>,
}

impl UnifiedContractStorage {
    /// Create a new unified storage instance
    pub fn new<P: AsRef<Path>>(db_path: P) -> Result<Self> {
        let db = Arc::new(sled::open(db_path)?);

        let contracts_tree = db.open_tree(b"contracts")?;
        let state_tree = db.open_tree(b"contract_state")?;
        let history_tree = db.open_tree(b"execution_history")?;
        let memory_cache = Arc::new(RwLock::new(HashMap::new()));

        Ok(Self {
            db,
            contracts_tree,
            state_tree,
            history_tree,
            memory_cache,
        })
    }

    /// Create an in-memory storage for testing
    pub fn in_memory() -> Result<Self> {
        let db = Arc::new(sled::Config::new().temporary(true).open()?);

        let contracts_tree = db.open_tree(b"contracts")?;
        let state_tree = db.open_tree(b"contract_state")?;
        let history_tree = db.open_tree(b"execution_history")?;
        let memory_cache = Arc::new(RwLock::new(HashMap::new()));

        Ok(Self {
            db,
            contracts_tree,
            state_tree,
            history_tree,
            memory_cache,
        })
    }

    /// Get storage statistics
    pub fn get_stats(&self) -> Result<StorageStats> {
        let contracts_count = self.contracts_tree.len();
        let state_entries = self.state_tree.len();
        let history_entries = self.history_tree.len();
        let db_size = self.db.size_on_disk()?;

        Ok(StorageStats {
            contracts_count,
            state_entries,
            history_entries,
            db_size_bytes: db_size,
        })
    }

    /// Flush all data to disk
    pub fn flush(&self) -> Result<()> {
        self.db.flush()?;
        Ok(())
    }

    /// Compact the database
    pub async fn compact(&self) -> Result<()> {
        // Clear memory cache during compaction
        self.memory_cache.write().await.clear();

        // Compact the database
        tokio::task::spawn_blocking({
            let db = Arc::clone(&self.db);
            move || -> Result<()> {
                db.flush()?;
                // Sled auto-compacts, but we can force a flush
                Ok(())
            }
        })
        .await??;

        Ok(())
    }

    /// Create a composite key for contract state
    fn make_state_key(&self, contract: &str, key: &str) -> String {
        format!("{}:{}", contract, key)
    }
}

impl ContractStateStorage for UnifiedContractStorage {
    fn store_contract_metadata(&self, metadata: &UnifiedContractMetadata) -> Result<()> {
        let serialized = bincode::serialize(metadata)?;
        self.contracts_tree.insert(&metadata.address, serialized)?;
        self.contracts_tree.flush()?;
        Ok(())
    }

    fn get_contract_metadata(&self, address: &str) -> Result<Option<UnifiedContractMetadata>> {
        if let Some(data) = self.contracts_tree.get(address)? {
            let metadata: UnifiedContractMetadata = bincode::deserialize(&data)?;
            Ok(Some(metadata))
        } else {
            Ok(None)
        }
    }

    fn set_contract_state(&self, contract: &str, key: &str, value: &[u8]) -> Result<()> {
        let composite_key = self.make_state_key(contract, key);

        // Update persistent storage
        self.state_tree.insert(&composite_key, value)?;

        // Update memory cache for fast access
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            tokio::task::block_in_place(|| {
                handle.block_on(async {
                    let mut cache = self.memory_cache.write().await;
                    cache.insert(composite_key, value.to_vec());
                })
            });
        } else {
            // No tokio runtime, create one temporarily
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(async {
                let mut cache = self.memory_cache.write().await;
                cache.insert(composite_key, value.to_vec());
            });
        }

        Ok(())
    }

    fn get_contract_state(&self, contract: &str, key: &str) -> Result<Option<Vec<u8>>> {
        let composite_key = self.make_state_key(contract, key);

        // First check memory cache
        let cached_result = if let Ok(handle) = tokio::runtime::Handle::try_current() {
            tokio::task::block_in_place(|| {
                handle.block_on(async {
                    let cache = self.memory_cache.read().await;
                    cache.get(&composite_key).cloned()
                })
            })
        } else {
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(async {
                let cache = self.memory_cache.read().await;
                cache.get(&composite_key).cloned()
            })
        };

        if let Some(value) = cached_result {
            return Ok(Some(value));
        }

        // Fall back to persistent storage
        if let Some(data) = self.state_tree.get(&composite_key)? {
            let value = data.to_vec();

            // Update cache
            if let Ok(handle) = tokio::runtime::Handle::try_current() {
                tokio::task::block_in_place(|| {
                    handle.block_on(async {
                        let mut cache = self.memory_cache.write().await;
                        cache.insert(composite_key, value.clone());
                    })
                });
            } else {
                let rt = tokio::runtime::Runtime::new()?;
                rt.block_on(async {
                    let mut cache = self.memory_cache.write().await;
                    cache.insert(composite_key, value.clone());
                });
            }

            Ok(Some(value))
        } else {
            Ok(None)
        }
    }

    fn delete_contract_state(&self, contract: &str, key: &str) -> Result<()> {
        let composite_key = self.make_state_key(contract, key);

        // Remove from persistent storage
        self.state_tree.remove(&composite_key)?;

        // Remove from memory cache
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            tokio::task::block_in_place(|| {
                handle.block_on(async {
                    let mut cache = self.memory_cache.write().await;
                    cache.remove(&composite_key);
                })
            });
        } else {
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(async {
                let mut cache = self.memory_cache.write().await;
                cache.remove(&composite_key);
            });
        }

        Ok(())
    }

    fn list_contracts(&self) -> Result<Vec<String>> {
        let mut contracts = Vec::new();

        for result in self.contracts_tree.iter() {
            let (key, _) = result?;
            if let Ok(address) = String::from_utf8(key.to_vec()) {
                contracts.push(address);
            }
        }

        Ok(contracts)
    }

    fn store_execution(&self, execution: &ContractExecutionRecord) -> Result<()> {
        let key = format!("{}:{}", execution.contract_address, execution.execution_id);
        let serialized = bincode::serialize(execution)?;
        self.history_tree.insert(&key, serialized)?;
        Ok(())
    }

    fn get_execution_history(&self, contract: &str) -> Result<Vec<ContractExecutionRecord>> {
        let mut history = Vec::new();
        let prefix = format!("{}:", contract);

        for result in self.history_tree.scan_prefix(&prefix) {
            let (_, value) = result?;
            let execution: ContractExecutionRecord = bincode::deserialize(&value)?;
            history.push(execution);
        }

        // Sort by timestamp (newest first)
        history.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));

        Ok(history)
    }
}

/// Storage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageStats {
    pub contracts_count: usize,
    pub state_entries: usize,
    pub history_entries: usize,
    pub db_size_bytes: u64,
}

/// Synchronous in-memory storage implementation for testing
pub struct SyncInMemoryContractStorage {
    contracts: Arc<std::sync::RwLock<HashMap<String, UnifiedContractMetadata>>>,
    state: Arc<std::sync::RwLock<HashMap<String, Vec<u8>>>>,
    history: Arc<std::sync::RwLock<HashMap<String, Vec<ContractExecutionRecord>>>>,
}

impl SyncInMemoryContractStorage {
    pub fn new() -> Self {
        Self {
            contracts: Arc::new(std::sync::RwLock::new(HashMap::new())),
            state: Arc::new(std::sync::RwLock::new(HashMap::new())),
            history: Arc::new(std::sync::RwLock::new(HashMap::new())),
        }
    }

    fn make_state_key(&self, contract: &str, key: &str) -> String {
        format!("{}:{}", contract, key)
    }
}

impl Default for SyncInMemoryContractStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl ContractStateStorage for SyncInMemoryContractStorage {
    fn store_contract_metadata(&self, metadata: &UnifiedContractMetadata) -> Result<()> {
        let mut contracts = self.contracts.write().unwrap();
        contracts.insert(metadata.address.clone(), metadata.clone());
        Ok(())
    }

    fn get_contract_metadata(&self, address: &str) -> Result<Option<UnifiedContractMetadata>> {
        let contracts = self.contracts.read().unwrap();
        Ok(contracts.get(address).cloned())
    }

    fn set_contract_state(&self, contract: &str, key: &str, value: &[u8]) -> Result<()> {
        let composite_key = self.make_state_key(contract, key);
        let mut state = self.state.write().unwrap();
        state.insert(composite_key, value.to_vec());
        Ok(())
    }

    fn get_contract_state(&self, contract: &str, key: &str) -> Result<Option<Vec<u8>>> {
        let composite_key = self.make_state_key(contract, key);
        let state = self.state.read().unwrap();
        Ok(state.get(&composite_key).cloned())
    }

    fn delete_contract_state(&self, contract: &str, key: &str) -> Result<()> {
        let composite_key = self.make_state_key(contract, key);
        let mut state = self.state.write().unwrap();
        state.remove(&composite_key);
        Ok(())
    }

    fn list_contracts(&self) -> Result<Vec<String>> {
        let contracts = self.contracts.read().unwrap();
        Ok(contracts.keys().cloned().collect())
    }

    fn store_execution(&self, execution: &ContractExecutionRecord) -> Result<()> {
        let mut history = self.history.write().unwrap();
        history
            .entry(execution.contract_address.clone())
            .or_default()
            .push(execution.clone());
        Ok(())
    }

    fn get_execution_history(&self, contract: &str) -> Result<Vec<ContractExecutionRecord>> {
        let history = self.history.read().unwrap();
        Ok(history.get(contract).cloned().unwrap_or_default())
    }
}

/// Async in-memory storage implementation for testing and temporary use
pub struct InMemoryContractStorage {
    contracts: Arc<RwLock<HashMap<String, UnifiedContractMetadata>>>,
    state: Arc<RwLock<HashMap<String, Vec<u8>>>>,
    history: Arc<RwLock<HashMap<String, Vec<ContractExecutionRecord>>>>,
}

impl InMemoryContractStorage {
    pub fn new() -> Self {
        Self {
            contracts: Arc::new(RwLock::new(HashMap::new())),
            state: Arc::new(RwLock::new(HashMap::new())),
            history: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    fn make_state_key(&self, contract: &str, key: &str) -> String {
        format!("{}:{}", contract, key)
    }
}

impl Default for InMemoryContractStorage {
    fn default() -> Self {
        Self::new()
    }
}

impl ContractStateStorage for InMemoryContractStorage {
    fn store_contract_metadata(&self, metadata: &UnifiedContractMetadata) -> Result<()> {
        // Try async first, fallback to sync if no runtime
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            tokio::task::block_in_place(|| {
                handle.block_on(async {
                    let mut contracts = self.contracts.write().await;
                    contracts.insert(metadata.address.clone(), metadata.clone());
                })
            });
        } else {
            // No tokio runtime available, use blocking approach
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(async {
                let mut contracts = self.contracts.write().await;
                contracts.insert(metadata.address.clone(), metadata.clone());
            });
        }
        Ok(())
    }

    fn get_contract_metadata(&self, address: &str) -> Result<Option<UnifiedContractMetadata>> {
        let result = if let Ok(handle) = tokio::runtime::Handle::try_current() {
            tokio::task::block_in_place(|| {
                handle.block_on(async {
                    let contracts = self.contracts.read().await;
                    contracts.get(address).cloned()
                })
            })
        } else {
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(async {
                let contracts = self.contracts.read().await;
                contracts.get(address).cloned()
            })
        };
        Ok(result)
    }

    fn set_contract_state(&self, contract: &str, key: &str, value: &[u8]) -> Result<()> {
        let composite_key = self.make_state_key(contract, key);
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            tokio::task::block_in_place(|| {
                handle.block_on(async {
                    let mut state = self.state.write().await;
                    state.insert(composite_key, value.to_vec());
                })
            });
        } else {
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(async {
                let mut state = self.state.write().await;
                state.insert(composite_key, value.to_vec());
            });
        }
        Ok(())
    }

    fn get_contract_state(&self, contract: &str, key: &str) -> Result<Option<Vec<u8>>> {
        let composite_key = self.make_state_key(contract, key);
        let result = if let Ok(handle) = tokio::runtime::Handle::try_current() {
            tokio::task::block_in_place(|| {
                handle.block_on(async {
                    let state = self.state.read().await;
                    state.get(&composite_key).cloned()
                })
            })
        } else {
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(async {
                let state = self.state.read().await;
                state.get(&composite_key).cloned()
            })
        };
        Ok(result)
    }

    fn delete_contract_state(&self, contract: &str, key: &str) -> Result<()> {
        let composite_key = self.make_state_key(contract, key);
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            tokio::task::block_in_place(|| {
                handle.block_on(async {
                    let mut state = self.state.write().await;
                    state.remove(&composite_key);
                })
            });
        } else {
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(async {
                let mut state = self.state.write().await;
                state.remove(&composite_key);
            });
        }
        Ok(())
    }

    fn list_contracts(&self) -> Result<Vec<String>> {
        let result = if let Ok(handle) = tokio::runtime::Handle::try_current() {
            tokio::task::block_in_place(|| {
                handle.block_on(async {
                    let contracts = self.contracts.read().await;
                    contracts.keys().cloned().collect()
                })
            })
        } else {
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(async {
                let contracts = self.contracts.read().await;
                contracts.keys().cloned().collect()
            })
        };
        Ok(result)
    }

    fn store_execution(&self, execution: &ContractExecutionRecord) -> Result<()> {
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            tokio::task::block_in_place(|| {
                handle.block_on(async {
                    let mut history = self.history.write().await;
                    history
                        .entry(execution.contract_address.clone())
                        .or_default()
                        .push(execution.clone());
                })
            });
        } else {
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(async {
                let mut history = self.history.write().await;
                history
                    .entry(execution.contract_address.clone())
                    .or_default()
                    .push(execution.clone());
            });
        }
        Ok(())
    }

    fn get_execution_history(&self, contract: &str) -> Result<Vec<ContractExecutionRecord>> {
        let result = if let Ok(handle) = tokio::runtime::Handle::try_current() {
            tokio::task::block_in_place(|| {
                handle.block_on(async {
                    let history = self.history.read().await;
                    history.get(contract).cloned().unwrap_or_default()
                })
            })
        } else {
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(async {
                let history = self.history.read().await;
                history.get(contract).cloned().unwrap_or_default()
            })
        };
        Ok(result)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::smart_contract::unified_engine::ContractType;

    fn create_test_metadata() -> UnifiedContractMetadata {
        UnifiedContractMetadata {
            address: "0x1234567890".to_string(),
            name: "Test Contract".to_string(),
            description: "A test contract".to_string(),
            contract_type: ContractType::Wasm {
                bytecode: vec![1, 2, 3, 4],
                abi: Some("test_abi".to_string()),
            },
            deployment_tx: "0xabcdef".to_string(),
            deployment_time: 1234567890,
            owner: "0x9876543210".to_string(),
            is_active: true,
        }
    }

    #[test]
    fn test_sync_in_memory_storage() {
        let storage = SyncInMemoryContractStorage::new();
        let metadata = create_test_metadata();

        // Store contract metadata
        storage.store_contract_metadata(&metadata).unwrap();

        // Retrieve contract metadata
        let retrieved = storage.get_contract_metadata(&metadata.address).unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, metadata.name);

        // Test state operations
        storage
            .set_contract_state(&metadata.address, "test_key", b"test_value")
            .unwrap();
        let value = storage
            .get_contract_state(&metadata.address, "test_key")
            .unwrap();
        assert_eq!(value, Some(b"test_value".to_vec()));

        // Test state deletion
        storage
            .delete_contract_state(&metadata.address, "test_key")
            .unwrap();
        let value = storage
            .get_contract_state(&metadata.address, "test_key")
            .unwrap();
        assert!(value.is_none());

        // Test contract listing
        let contracts = storage.list_contracts().unwrap();
        assert_eq!(contracts.len(), 1);
        assert_eq!(contracts[0], metadata.address);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_async_in_memory_storage() {
        let storage = InMemoryContractStorage::new();
        let metadata = create_test_metadata();

        // Store contract metadata
        storage.store_contract_metadata(&metadata).unwrap();

        // Retrieve contract metadata
        let retrieved = storage.get_contract_metadata(&metadata.address).unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, metadata.name);

        // Test state operations
        storage
            .set_contract_state(&metadata.address, "test_key", b"test_value")
            .unwrap();
        let value = storage
            .get_contract_state(&metadata.address, "test_key")
            .unwrap();
        assert_eq!(value, Some(b"test_value".to_vec()));

        // Test state deletion
        storage
            .delete_contract_state(&metadata.address, "test_key")
            .unwrap();
        let value = storage
            .get_contract_state(&metadata.address, "test_key")
            .unwrap();
        assert!(value.is_none());

        // Test contract listing
        let contracts = storage.list_contracts().unwrap();
        assert_eq!(contracts.len(), 1);
        assert_eq!(contracts[0], metadata.address);
    }

    #[test]
    fn test_sync_execution_history() {
        let storage = SyncInMemoryContractStorage::new();
        let contract_address = "0x1234567890";

        let execution = ContractExecutionRecord {
            execution_id: "exec_1".to_string(),
            contract_address: contract_address.to_string(),
            function_name: "test_function".to_string(),
            caller: "0x9876543210".to_string(),
            timestamp: 1234567890,
            gas_used: 50000,
            success: true,
            error_message: None,
        };

        storage.store_execution(&execution).unwrap();

        let history = storage.get_execution_history(contract_address).unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].execution_id, execution.execution_id);
        assert_eq!(history[0].gas_used, execution.gas_used);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_async_execution_history() {
        let storage = InMemoryContractStorage::new();
        let contract_address = "0x1234567890";

        let execution = ContractExecutionRecord {
            execution_id: "exec_1".to_string(),
            contract_address: contract_address.to_string(),
            function_name: "test_function".to_string(),
            caller: "0x9876543210".to_string(),
            timestamp: 1234567890,
            gas_used: 50000,
            success: true,
            error_message: None,
        };

        storage.store_execution(&execution).unwrap();

        let history = storage.get_execution_history(contract_address).unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].execution_id, execution.execution_id);
        assert_eq!(history[0].gas_used, execution.gas_used);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_sled_storage() {
        let storage = UnifiedContractStorage::in_memory().unwrap();
        let metadata = create_test_metadata();

        // Store contract metadata
        storage.store_contract_metadata(&metadata).unwrap();

        // Retrieve contract metadata
        let retrieved = storage.get_contract_metadata(&metadata.address).unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, metadata.name);

        // Test state operations with caching
        storage
            .set_contract_state(&metadata.address, "cached_key", b"cached_value")
            .unwrap();

        // Should hit cache
        let value1 = storage
            .get_contract_state(&metadata.address, "cached_key")
            .unwrap();
        assert_eq!(value1, Some(b"cached_value".to_vec()));

        // Should still work after cache clear
        storage.memory_cache.write().await.clear();
        let value2 = storage
            .get_contract_state(&metadata.address, "cached_key")
            .unwrap();
        assert_eq!(value2, Some(b"cached_value".to_vec()));

        // Test storage stats
        let stats = storage.get_stats().unwrap();
        assert!(stats.contracts_count > 0);
        assert!(stats.state_entries > 0);
    }
}
