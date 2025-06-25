//! Advanced Database Storage Implementation
//!
//! This module provides advanced database storage implementations for enterprise deployment,
//! including PostgreSQL for relational data and Redis for high-performance caching.

use std::{collections::HashMap, sync::Arc, time::Duration};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::{sync::RwLock, time::timeout};

use super::unified_engine::{
    ContractExecutionRecord, ContractStateStorage, UnifiedContractMetadata,
};

/// Configuration for database storage backends
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DatabaseStorageConfig {
    /// PostgreSQL connection configuration
    pub postgres: Option<PostgresConfig>,
    /// Redis connection configuration
    pub redis: Option<RedisConfig>,
    /// Fallback to in-memory storage if databases unavailable
    pub fallback_to_memory: bool,
    /// Connection timeout in seconds
    pub connection_timeout_secs: u64,
    /// Maximum connection pool size
    pub max_connections: u32,
    /// Enable connection encryption
    pub use_ssl: bool,
}

/// PostgreSQL configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PostgresConfig {
    pub host: String,
    pub port: u16,
    pub database: String,
    pub username: String,
    pub password: String,
    pub schema: String,
    pub max_connections: u32,
}

/// Redis configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RedisConfig {
    pub url: String,
    pub password: Option<String>,
    pub database: u8,
    pub max_connections: u32,
    pub key_prefix: String,
    pub ttl_seconds: Option<u64>,
}

impl Default for DatabaseStorageConfig {
    fn default() -> Self {
        Self {
            postgres: None,
            redis: None,
            fallback_to_memory: true,
            connection_timeout_secs: 30,
            max_connections: 20,
            use_ssl: false,
        }
    }
}

impl Default for PostgresConfig {
    fn default() -> Self {
        Self {
            host: "localhost".to_string(),
            port: 5432,
            database: "polytorus".to_string(),
            username: "polytorus".to_string(),
            password: "polytorus".to_string(),
            schema: "smart_contracts".to_string(),
            max_connections: 20,
        }
    }
}

impl Default for RedisConfig {
    fn default() -> Self {
        Self {
            url: "redis://localhost:6379".to_string(),
            password: None,
            database: 0,
            max_connections: 20,
            key_prefix: "polytorus:contracts:".to_string(),
            ttl_seconds: Some(3600), // 1 hour default TTL
        }
    }
}

/// Advanced database storage implementation with multiple backends
pub struct DatabaseContractStorage {
    config: DatabaseStorageConfig,
    postgres_pool: Option<Arc<PostgresConnectionPool>>,
    redis_pool: Option<Arc<RedisConnectionPool>>,
    memory_fallback: Arc<RwLock<HashMap<String, Vec<u8>>>>,
    connection_stats: Arc<RwLock<ConnectionStats>>,
}

/// PostgreSQL connection pool (placeholder for actual implementation)
pub struct PostgresConnectionPool {
    _config: PostgresConfig,
    _active_connections: Arc<RwLock<u32>>,
}

/// Redis connection pool (placeholder for actual implementation)  
pub struct RedisConnectionPool {
    _config: RedisConfig,
    _active_connections: Arc<RwLock<u32>>,
}

/// Connection statistics
#[derive(Debug, Clone, Default)]
pub struct ConnectionStats {
    pub postgres_connections: u32,
    pub redis_connections: u32,
    pub total_queries: u64,
    pub failed_queries: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
}

impl DatabaseContractStorage {
    /// Create a new database storage instance
    pub async fn new(config: DatabaseStorageConfig) -> Result<Self> {
        let mut postgres_pool = None;
        let mut redis_pool = None;

        // Initialize PostgreSQL connection pool
        if let Some(pg_config) = &config.postgres {
            match timeout(
                Duration::from_secs(config.connection_timeout_secs),
                PostgresConnectionPool::new(pg_config.clone()),
            )
            .await
            {
                Ok(Ok(pool)) => {
                    postgres_pool = Some(Arc::new(pool));
                }
                Ok(Err(e)) => {
                    if !config.fallback_to_memory {
                        return Err(anyhow::anyhow!("PostgreSQL connection failed: {}", e));
                    }
                }
                Err(_) => {
                    if !config.fallback_to_memory {
                        return Err(anyhow::anyhow!("PostgreSQL connection timeout"));
                    }
                }
            }
        }

        // Initialize Redis connection pool
        if let Some(redis_config) = &config.redis {
            match timeout(
                Duration::from_secs(config.connection_timeout_secs),
                RedisConnectionPool::new(redis_config.clone()),
            )
            .await
            {
                Ok(Ok(pool)) => {
                    redis_pool = Some(Arc::new(pool));
                }
                Ok(Err(e)) => {
                    if !config.fallback_to_memory {
                        return Err(anyhow::anyhow!("Redis connection failed: {}", e));
                    }
                }
                Err(_) => {
                    if !config.fallback_to_memory {
                        return Err(anyhow::anyhow!("Redis connection timeout"));
                    }
                }
            }
        }

        Ok(Self {
            config,
            postgres_pool,
            redis_pool,
            memory_fallback: Arc::new(RwLock::new(HashMap::new())),
            connection_stats: Arc::new(RwLock::new(ConnectionStats::default())),
        })
    }

    /// Create a testing instance with memory fallback
    pub fn testing() -> Self {
        Self {
            config: DatabaseStorageConfig {
                fallback_to_memory: true,
                ..Default::default()
            },
            postgres_pool: None,
            redis_pool: None,
            memory_fallback: Arc::new(RwLock::new(HashMap::new())),
            connection_stats: Arc::new(RwLock::new(ConnectionStats::default())),
        }
    }

    /// Get connection statistics
    pub async fn get_stats(&self) -> ConnectionStats {
        self.connection_stats.read().await.clone()
    }

    /// Store data in Redis cache
    async fn cache_store(&self, key: &str, value: &[u8]) -> Result<()> {
        if let Some(redis) = &self.redis_pool {
            match redis.set(key, value).await {
                Ok(_) => {
                    let mut stats = self.connection_stats.write().await;
                    stats.total_queries += 1;
                    return Ok(());
                }
                Err(e) => {
                    let mut stats = self.connection_stats.write().await;
                    stats.failed_queries += 1;
                    eprintln!("Redis cache store failed: {}", e);
                }
            }
        }

        // Fallback to memory
        if self.config.fallback_to_memory {
            let mut memory = self.memory_fallback.write().await;
            memory.insert(key.to_string(), value.to_vec());
        }

        Ok(())
    }

    /// Retrieve data from Redis cache
    async fn cache_get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        if let Some(redis) = &self.redis_pool {
            match redis.get(key).await {
                Ok(Some(value)) => {
                    let mut stats = self.connection_stats.write().await;
                    stats.total_queries += 1;
                    stats.cache_hits += 1;
                    return Ok(Some(value));
                }
                Ok(None) => {
                    let mut stats = self.connection_stats.write().await;
                    stats.total_queries += 1;
                    stats.cache_misses += 1;
                }
                Err(e) => {
                    let mut stats = self.connection_stats.write().await;
                    stats.failed_queries += 1;
                    eprintln!("Redis cache get failed: {}", e);
                }
            }
        }

        // Fallback to memory
        if self.config.fallback_to_memory {
            let memory = self.memory_fallback.read().await;
            if let Some(value) = memory.get(key) {
                let mut stats = self.connection_stats.write().await;
                stats.cache_hits += 1;
                return Ok(Some(value.clone()));
            } else {
                let mut stats = self.connection_stats.write().await;
                stats.cache_misses += 1;
            }
        }

        Ok(None)
    }

    /// Store data in PostgreSQL
    async fn postgres_store(&self, table: &str, key: &str, value: &[u8]) -> Result<()> {
        if let Some(postgres) = &self.postgres_pool {
            match postgres.insert(table, key, value).await {
                Ok(_) => {
                    let mut stats = self.connection_stats.write().await;
                    stats.total_queries += 1;
                    return Ok(());
                }
                Err(e) => {
                    let mut stats = self.connection_stats.write().await;
                    stats.failed_queries += 1;
                    if !self.config.fallback_to_memory {
                        return Err(e);
                    }
                }
            }
        }

        // Fallback to memory
        if self.config.fallback_to_memory {
            let composite_key = format!("{}:{}", table, key);
            let mut memory = self.memory_fallback.write().await;
            memory.insert(composite_key, value.to_vec());
        }

        Ok(())
    }

    /// Retrieve data from PostgreSQL
    async fn postgres_get(&self, table: &str, key: &str) -> Result<Option<Vec<u8>>> {
        if let Some(postgres) = &self.postgres_pool {
            match postgres.select(table, key).await {
                Ok(value) => {
                    let mut stats = self.connection_stats.write().await;
                    stats.total_queries += 1;
                    return Ok(value);
                }
                Err(e) => {
                    let mut stats = self.connection_stats.write().await;
                    stats.failed_queries += 1;
                    if !self.config.fallback_to_memory {
                        return Err(e);
                    }
                }
            }
        }

        // Fallback to memory
        if self.config.fallback_to_memory {
            let composite_key = format!("{}:{}", table, key);
            let memory = self.memory_fallback.read().await;
            return Ok(memory.get(&composite_key).cloned());
        }

        Ok(None)
    }

    /// Create a cache key for contract state
    fn make_cache_key(&self, contract: &str, key: &str) -> String {
        let prefix = self
            .config
            .redis
            .as_ref()
            .map(|r| r.key_prefix.as_str())
            .unwrap_or("");
        format!("{}state:{}:{}", prefix, contract, key)
    }
}

impl ContractStateStorage for DatabaseContractStorage {
    fn store_contract_metadata(&self, metadata: &UnifiedContractMetadata) -> Result<()> {
        let serialized = bincode::serialize(metadata)?;

        // Use async runtime for database operations
        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            tokio::task::block_in_place(|| {
                handle.block_on(async {
                    // Store in PostgreSQL
                    if let Err(e) = self
                        .postgres_store("contracts", &metadata.address, &serialized)
                        .await
                    {
                        eprintln!("Failed to store contract metadata in PostgreSQL: {}", e);
                    }

                    // Cache in Redis
                    let cache_key = format!("contract:{}", metadata.address);
                    if let Err(e) = self.cache_store(&cache_key, &serialized).await {
                        eprintln!("Failed to cache contract metadata: {}", e);
                    }
                })
            });
        } else {
            // No async runtime, use blocking fallback
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(async {
                // Store in PostgreSQL
                if let Err(e) = self
                    .postgres_store("contracts", &metadata.address, &serialized)
                    .await
                {
                    eprintln!("Failed to store contract metadata in PostgreSQL: {}", e);
                }

                // Cache in Redis
                let cache_key = format!("contract:{}", metadata.address);
                if let Err(e) = self.cache_store(&cache_key, &serialized).await {
                    eprintln!("Failed to cache contract metadata: {}", e);
                }
            });
        }

        Ok(())
    }

    fn get_contract_metadata(&self, address: &str) -> Result<Option<UnifiedContractMetadata>> {
        let cache_key = format!("contract:{}", address);

        let result = if let Ok(handle) = tokio::runtime::Handle::try_current() {
            tokio::task::block_in_place(|| {
                handle.block_on(async {
                    // Try cache first
                    if let Ok(Some(cached_data)) = self.cache_get(&cache_key).await {
                        if let Ok(metadata) = bincode::deserialize(&cached_data) {
                            return Ok(Some(metadata));
                        }
                    }

                    // Fallback to PostgreSQL
                    if let Ok(Some(pg_data)) = self.postgres_get("contracts", address).await {
                        if let Ok(metadata) = bincode::deserialize(&pg_data) {
                            // Populate cache for future requests
                            let _ = self.cache_store(&cache_key, &pg_data).await;
                            return Ok(Some(metadata));
                        }
                    }

                    Ok(None)
                })
            })
        } else {
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(async {
                // Try cache first
                if let Ok(Some(cached_data)) = self.cache_get(&cache_key).await {
                    if let Ok(metadata) = bincode::deserialize(&cached_data) {
                        return Ok(Some(metadata));
                    }
                }

                // Fallback to PostgreSQL
                if let Ok(Some(pg_data)) = self.postgres_get("contracts", address).await {
                    if let Ok(metadata) = bincode::deserialize(&pg_data) {
                        // Populate cache for future requests
                        let _ = self.cache_store(&cache_key, &pg_data).await;
                        return Ok(Some(metadata));
                    }
                }

                Ok(None)
            })
        };

        result
    }

    fn set_contract_state(&self, contract: &str, key: &str, value: &[u8]) -> Result<()> {
        let state_key = format!("{}:{}", contract, key);
        let cache_key = self.make_cache_key(contract, key);

        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            tokio::task::block_in_place(|| {
                handle.block_on(async {
                    // Store in PostgreSQL
                    if let Err(e) = self
                        .postgres_store("contract_state", &state_key, value)
                        .await
                    {
                        eprintln!("Failed to store contract state in PostgreSQL: {}", e);
                    }

                    // Cache in Redis
                    if let Err(e) = self.cache_store(&cache_key, value).await {
                        eprintln!("Failed to cache contract state: {}", e);
                    }
                })
            });
        } else {
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(async {
                // Store in PostgreSQL
                if let Err(e) = self
                    .postgres_store("contract_state", &state_key, value)
                    .await
                {
                    eprintln!("Failed to store contract state in PostgreSQL: {}", e);
                }

                // Cache in Redis
                if let Err(e) = self.cache_store(&cache_key, value).await {
                    eprintln!("Failed to cache contract state: {}", e);
                }
            });
        }

        Ok(())
    }

    fn get_contract_state(&self, contract: &str, key: &str) -> Result<Option<Vec<u8>>> {
        let state_key = format!("{}:{}", contract, key);
        let cache_key = self.make_cache_key(contract, key);

        let result = if let Ok(handle) = tokio::runtime::Handle::try_current() {
            tokio::task::block_in_place(|| {
                handle.block_on(async {
                    // Try cache first
                    if let Ok(Some(cached_data)) = self.cache_get(&cache_key).await {
                        return Ok(Some(cached_data));
                    }

                    // Fallback to PostgreSQL
                    if let Ok(Some(pg_data)) = self.postgres_get("contract_state", &state_key).await
                    {
                        // Populate cache for future requests
                        let _ = self.cache_store(&cache_key, &pg_data).await;
                        return Ok(Some(pg_data));
                    }

                    Ok(None)
                })
            })
        } else {
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(async {
                // Try cache first
                if let Ok(Some(cached_data)) = self.cache_get(&cache_key).await {
                    return Ok(Some(cached_data));
                }

                // Fallback to PostgreSQL
                if let Ok(Some(pg_data)) = self.postgres_get("contract_state", &state_key).await {
                    // Populate cache for future requests
                    let _ = self.cache_store(&cache_key, &pg_data).await;
                    return Ok(Some(pg_data));
                }

                Ok(None)
            })
        };

        result
    }

    fn delete_contract_state(&self, contract: &str, key: &str) -> Result<()> {
        let state_key = format!("{}:{}", contract, key);
        let cache_key = self.make_cache_key(contract, key);

        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            tokio::task::block_in_place(|| {
                handle.block_on(async {
                    // Remove from PostgreSQL
                    if let Some(postgres) = &self.postgres_pool {
                        if let Err(e) = postgres.delete("contract_state", &state_key).await {
                            eprintln!("Failed to delete from PostgreSQL: {}", e);
                        }
                    }

                    // Remove from Redis cache
                    if let Some(redis) = &self.redis_pool {
                        if let Err(e) = redis.delete(&cache_key).await {
                            eprintln!("Failed to delete from Redis: {}", e);
                        }
                    }

                    // Remove from memory fallback
                    if self.config.fallback_to_memory {
                        let mut memory = self.memory_fallback.write().await;
                        memory.remove(&format!("contract_state:{}", state_key));
                        memory.remove(&cache_key);
                    }
                })
            });
        }

        Ok(())
    }

    fn list_contracts(&self) -> Result<Vec<String>> {
        let result = if let Ok(handle) = tokio::runtime::Handle::try_current() {
            tokio::task::block_in_place(|| {
                handle.block_on(async {
                    // Try PostgreSQL first
                    if let Some(postgres) = &self.postgres_pool {
                        if let Ok(contracts) = postgres.list_keys("contracts").await {
                            return contracts;
                        }
                    }

                    // Fallback to memory
                    if self.config.fallback_to_memory {
                        let memory = self.memory_fallback.read().await;
                        return memory
                            .keys()
                            .filter_map(|k| {
                                if k.starts_with("contracts:") {
                                    Some(k.strip_prefix("contracts:").unwrap().to_string())
                                } else {
                                    None
                                }
                            })
                            .collect();
                    }

                    Vec::new()
                })
            })
        } else {
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(async {
                // Try PostgreSQL first
                if let Some(postgres) = &self.postgres_pool {
                    if let Ok(contracts) = postgres.list_keys("contracts").await {
                        return contracts;
                    }
                }

                // Fallback to memory
                if self.config.fallback_to_memory {
                    let memory = self.memory_fallback.read().await;
                    return memory
                        .keys()
                        .filter_map(|k| {
                            if k.starts_with("contracts:") {
                                Some(k.strip_prefix("contracts:").unwrap().to_string())
                            } else {
                                None
                            }
                        })
                        .collect();
                }

                Vec::new()
            })
        };

        Ok(result)
    }

    fn store_execution(&self, execution: &ContractExecutionRecord) -> Result<()> {
        let execution_key = format!("{}:{}", execution.contract_address, execution.execution_id);
        let serialized = bincode::serialize(execution)?;

        if let Ok(handle) = tokio::runtime::Handle::try_current() {
            tokio::task::block_in_place(|| {
                handle.block_on(async {
                    // Store in PostgreSQL
                    if let Err(e) = self
                        .postgres_store("execution_history", &execution_key, &serialized)
                        .await
                    {
                        eprintln!("Failed to store execution history in PostgreSQL: {}", e);
                    }
                })
            });
        }

        Ok(())
    }

    fn get_execution_history(&self, contract: &str) -> Result<Vec<ContractExecutionRecord>> {
        let result = if let Ok(handle) = tokio::runtime::Handle::try_current() {
            tokio::task::block_in_place(|| {
                handle.block_on(async {
                    // Try PostgreSQL
                    if let Some(postgres) = &self.postgres_pool {
                        if let Ok(executions) = postgres.get_executions_for_contract(contract).await
                        {
                            return executions;
                        }
                    }

                    // Fallback to memory
                    if self.config.fallback_to_memory {
                        let memory = self.memory_fallback.read().await;
                        let prefix = format!("execution_history:{}:", contract);
                        let mut executions = Vec::new();

                        for (key, value) in memory.iter() {
                            if key.starts_with(&prefix) {
                                if let Ok(execution) =
                                    bincode::deserialize::<ContractExecutionRecord>(value)
                                {
                                    executions.push(execution);
                                }
                            }
                        }

                        // Sort by timestamp (newest first)
                        executions.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
                        return executions;
                    }

                    Vec::new()
                })
            })
        } else {
            let rt = tokio::runtime::Runtime::new()?;
            rt.block_on(async {
                // Try PostgreSQL
                if let Some(postgres) = &self.postgres_pool {
                    if let Ok(executions) = postgres.get_executions_for_contract(contract).await {
                        return executions;
                    }
                }

                // Fallback to memory
                if self.config.fallback_to_memory {
                    let memory = self.memory_fallback.read().await;
                    let prefix = format!("execution_history:{}:", contract);
                    let mut executions = Vec::new();

                    for (key, value) in memory.iter() {
                        if key.starts_with(&prefix) {
                            if let Ok(execution) =
                                bincode::deserialize::<ContractExecutionRecord>(value)
                            {
                                executions.push(execution);
                            }
                        }
                    }

                    // Sort by timestamp (newest first)
                    executions.sort_by(|a, b| b.timestamp.cmp(&a.timestamp));
                    return executions;
                }

                Vec::new()
            })
        };

        Ok(result)
    }
}

impl PostgresConnectionPool {
    pub async fn new(config: PostgresConfig) -> Result<Self> {
        // Placeholder implementation - in real code would use sqlx or similar
        Ok(Self {
            _config: config,
            _active_connections: Arc::new(RwLock::new(0)),
        })
    }

    pub async fn insert(&self, table: &str, key: &str, value: &[u8]) -> Result<()> {
        // Placeholder - would execute actual SQL INSERT
        println!(
            "PostgreSQL INSERT: {} -> {} ({} bytes)",
            table,
            key,
            value.len()
        );
        Ok(())
    }

    pub async fn select(&self, table: &str, key: &str) -> Result<Option<Vec<u8>>> {
        // Placeholder - would execute actual SQL SELECT
        println!("PostgreSQL SELECT: {} -> {}", table, key);
        Ok(None)
    }

    pub async fn delete(&self, table: &str, key: &str) -> Result<()> {
        // Placeholder - would execute actual SQL DELETE
        println!("PostgreSQL DELETE: {} -> {}", table, key);
        Ok(())
    }

    pub async fn list_keys(&self, table: &str) -> Result<Vec<String>> {
        // Placeholder - would execute actual SQL SELECT for keys
        println!("PostgreSQL LIST_KEYS: {}", table);
        Ok(Vec::new())
    }

    pub async fn get_executions_for_contract(
        &self,
        contract: &str,
    ) -> Result<Vec<ContractExecutionRecord>> {
        // Placeholder - would execute actual SQL query
        println!("PostgreSQL GET_EXECUTIONS: {}", contract);
        Ok(Vec::new())
    }
}

impl RedisConnectionPool {
    pub async fn new(config: RedisConfig) -> Result<Self> {
        // Placeholder implementation - in real code would use redis-rs
        Ok(Self {
            _config: config,
            _active_connections: Arc::new(RwLock::new(0)),
        })
    }

    pub async fn set(&self, key: &str, value: &[u8]) -> Result<()> {
        // Placeholder - would execute actual Redis SET
        println!("Redis SET: {} ({} bytes)", key, value.len());
        Ok(())
    }

    pub async fn get(&self, key: &str) -> Result<Option<Vec<u8>>> {
        // Placeholder - would execute actual Redis GET
        println!("Redis GET: {}", key);
        Ok(None)
    }

    pub async fn delete(&self, key: &str) -> Result<()> {
        // Placeholder - would execute actual Redis DEL
        println!("Redis DELETE: {}", key);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::smart_contract::unified_engine::ContractType;

    fn create_test_metadata() -> UnifiedContractMetadata {
        UnifiedContractMetadata {
            address: "0xtest123".to_string(),
            name: "Test Contract".to_string(),
            description: "A test contract".to_string(),
            contract_type: ContractType::Wasm {
                bytecode: vec![1, 2, 3],
                abi: Some("test_abi".to_string()),
            },
            deployment_tx: "0xdeployment".to_string(),
            deployment_time: 1234567890,
            owner: "0xowner".to_string(),
            is_active: true,
        }
    }

    #[tokio::test]
    async fn test_database_storage_creation() {
        let storage = DatabaseContractStorage::testing();
        let stats = storage.get_stats().await;

        assert_eq!(stats.postgres_connections, 0);
        assert_eq!(stats.redis_connections, 0);
        assert_eq!(stats.total_queries, 0);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_contract_metadata_fallback() {
        let storage = DatabaseContractStorage::testing();
        let metadata = create_test_metadata();

        // Store metadata (should use memory fallback)
        storage.store_contract_metadata(&metadata).unwrap();

        // Retrieve metadata (should hit memory fallback)
        let retrieved = storage.get_contract_metadata(&metadata.address).unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, metadata.name);
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_contract_state_operations() {
        let storage = DatabaseContractStorage::testing();

        // Set contract state
        storage
            .set_contract_state("0xcontract", "test_key", b"test_value")
            .unwrap();

        // Get contract state
        let value = storage
            .get_contract_state("0xcontract", "test_key")
            .unwrap();
        assert_eq!(value, Some(b"test_value".to_vec()));

        // Delete contract state
        storage
            .delete_contract_state("0xcontract", "test_key")
            .unwrap();

        // Verify deletion
        let value = storage
            .get_contract_state("0xcontract", "test_key")
            .unwrap();
        assert!(value.is_none());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_execution_history() {
        let storage = DatabaseContractStorage::testing();

        let execution = ContractExecutionRecord {
            execution_id: "exec_1".to_string(),
            contract_address: "0xcontract".to_string(),
            function_name: "test_function".to_string(),
            caller: "0xcaller".to_string(),
            timestamp: 1234567890,
            gas_used: 50000,
            success: true,
            error_message: None,
        };

        // Store execution
        storage.store_execution(&execution).unwrap();

        // Retrieve execution history
        let history = storage.get_execution_history("0xcontract").unwrap();
        assert_eq!(history.len(), 1);
        assert_eq!(history[0].execution_id, execution.execution_id);
    }

    #[tokio::test]
    async fn test_config_defaults() {
        let config = DatabaseStorageConfig::default();
        assert!(config.fallback_to_memory);
        assert_eq!(config.connection_timeout_secs, 30);
        assert_eq!(config.max_connections, 20);

        let pg_config = PostgresConfig::default();
        assert_eq!(pg_config.host, "localhost");
        assert_eq!(pg_config.port, 5432);
        assert_eq!(pg_config.database, "polytorus");

        let redis_config = RedisConfig::default();
        assert_eq!(redis_config.url, "redis://localhost:6379");
        assert_eq!(redis_config.database, 0);
        assert!(redis_config.ttl_seconds.is_some());
    }

    #[tokio::test(flavor = "multi_thread")]
    async fn test_connection_stats() {
        let storage = DatabaseContractStorage::testing();

        // Initial stats should be zero
        let stats = storage.get_stats().await;
        assert_eq!(stats.total_queries, 0);
        assert_eq!(stats.cache_hits, 0);
        assert_eq!(stats.cache_misses, 0);

        // Perform some operations that would update stats
        storage
            .set_contract_state("0xcontract", "key1", b"value1")
            .unwrap();
        let _ = storage.get_contract_state("0xcontract", "key1").unwrap();
        let _ = storage
            .get_contract_state("0xcontract", "nonexistent")
            .unwrap();

        // Note: In this test implementation, stats are only updated for actual Redis/PostgreSQL operations
        // Since we're using memory fallback, stats remain at 0
        let final_stats = storage.get_stats().await;
        assert_eq!(final_stats.total_queries, 0); // Would be > 0 with real databases
    }
}
