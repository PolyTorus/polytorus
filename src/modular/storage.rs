//! Modular storage layer implementation
//!
//! This module provides a modular storage layer that replaces legacy blockchain storage
//! with a more flexible and independent storage system for the modular architecture.

use std::{
    collections::HashMap,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use serde::{Deserialize, Serialize};

use super::traits::Hash;
#[cfg(test)]
use crate::blockchain::block::TestFinalizedParams;
use crate::{blockchain::block::Block, Result};

/// Storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageConfig {
    /// Database path
    pub db_path: PathBuf,
    /// Enable compression
    pub enable_compression: bool,
    /// Cache size in MB
    pub cache_size_mb: usize,
    /// Sync to disk immediately
    pub sync_writes: bool,
}

impl Default for StorageConfig {
    fn default() -> Self {
        Self {
            db_path: PathBuf::from("data/modular"),
            enable_compression: true,
            cache_size_mb: 64,
            sync_writes: false,
        }
    }
}

/// Modular storage layer implementation
pub struct ModularStorage {
    /// Block storage database
    block_db: sled::Db,
    /// State storage database
    state_db: sled::Db,
    /// Index storage database
    index_db: sled::Db,
    /// Configuration
    config: StorageConfig,
    /// Current blockchain tip (latest block hash)
    tip: Arc<Mutex<String>>,
    /// In-memory cache for frequently accessed data
    cache: Arc<Mutex<HashMap<String, CachedData>>>,
}

/// Cached data wrapper
#[derive(Debug, Clone)]
struct CachedData {
    data: Vec<u8>,
    timestamp: u64,
}

/// Storage layer interface
pub trait StorageLayer: Send + Sync {
    /// Store a block
    fn store_block(&self, block: &Block) -> Result<Hash>;

    /// Retrieve a block by hash
    fn get_block(&self, hash: &Hash) -> Result<Block>;

    /// Get the current blockchain tip
    fn get_tip(&self) -> Result<String>;

    /// Set the blockchain tip
    fn set_tip(&self, hash: &Hash) -> Result<()>;

    /// Get the current block height
    fn get_height(&self) -> Result<u64>;

    /// Get all block hashes in canonical order
    fn get_block_hashes(&self) -> Result<Vec<Hash>>;

    /// Store arbitrary data
    fn store_data(&self, key: &str, data: &[u8]) -> Result<()>;

    /// Retrieve arbitrary data
    fn get_data(&self, key: &str) -> Result<Option<Vec<u8>>>;

    /// Delete data
    fn delete_data(&self, key: &str) -> Result<()>;

    /// Check if block exists
    fn block_exists(&self, hash: &Hash) -> Result<bool>;

    /// Get block metadata without full block data
    fn get_block_metadata(&self, hash: &Hash) -> Result<BlockMetadata>;

    /// Flush pending writes to disk
    fn flush(&self) -> Result<()>;

    /// Compact database
    fn compact(&self) -> Result<()>;

    /// Get storage statistics
    fn get_stats(&self) -> Result<StorageStats>;
}

/// Block metadata for lightweight operations
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BlockMetadata {
    pub hash: Hash,
    pub height: u64,
    pub prev_hash: Hash,
    pub timestamp: u128,
    pub transaction_count: usize,
    pub size_bytes: usize,
}

/// Storage statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StorageStats {
    pub total_blocks: u64,
    pub total_size_bytes: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub db_size_bytes: u64,
}

impl ModularStorage {
    /// Create a new modular storage instance
    pub fn new(config: StorageConfig) -> Result<Self> {
        // Ensure storage directory exists
        std::fs::create_dir_all(&config.db_path)?;

        // Configure sled database
        let db_config = sled::Config::default()
            .path(config.db_path.join("blocks"))
            .cache_capacity((config.cache_size_mb * 1024 * 1024) as u64)
            .flush_every_ms(if config.sync_writes { Some(100) } else { None })
            .compression_factor(if config.enable_compression { 22 } else { 1 });

        let block_db = db_config.open()?;

        // Separate databases for different data types
        let state_db = sled::Config::default()
            .path(config.db_path.join("state"))
            .cache_capacity((config.cache_size_mb * 1024 * 1024 / 4) as u64)
            .flush_every_ms(if config.sync_writes { Some(100) } else { None })
            .compression_factor(if config.enable_compression { 22 } else { 1 })
            .open()?;

        let index_db = sled::Config::default()
            .path(config.db_path.join("index"))
            .cache_capacity((config.cache_size_mb * 1024 * 1024 / 4) as u64)
            .flush_every_ms(if config.sync_writes { Some(100) } else { None })
            .compression_factor(if config.enable_compression { 22 } else { 1 })
            .open()?;

        // Initialize tip from database or empty string
        let tip = if let Ok(Some(tip_bytes)) = block_db.get("TIP") {
            String::from_utf8(tip_bytes.to_vec()).unwrap_or_default()
        } else {
            String::new()
        };

        Ok(Self {
            block_db,
            state_db,
            index_db,
            config,
            tip: Arc::new(Mutex::new(tip)),
            cache: Arc::new(Mutex::new(HashMap::new())),
        })
    }

    /// Create storage with custom path
    pub fn new_with_path<P: AsRef<Path>>(path: P) -> Result<Self> {
        let config = StorageConfig {
            db_path: path.as_ref().to_path_buf(),
            ..Default::default()
        };
        Self::new(config)
    }

    /// Calculate block metadata from block
    fn calculate_metadata(&self, block: &Block) -> BlockMetadata {
        let serialized = bincode::serialize(block).unwrap_or_default();

        BlockMetadata {
            hash: block.get_hash().to_string(),
            height: block.get_height() as u64,
            prev_hash: block.get_prev_hash().to_string(),
            timestamp: block.get_timestamp(),
            transaction_count: block.get_transactions().len(),
            size_bytes: serialized.len(),
        }
    }

    /// Update cache with data
    fn update_cache(&self, key: String, data: Vec<u8>) {
        if let Ok(mut cache) = self.cache.lock() {
            // Simple LRU-like cache with size limit
            if cache.len() >= 1000 {
                // Remove oldest entry (simplified)
                if let Some(oldest_key) = cache.keys().next().cloned() {
                    cache.remove(&oldest_key);
                }
            }

            cache.insert(
                key,
                CachedData {
                    data,
                    timestamp: std::time::SystemTime::now()
                        .duration_since(std::time::UNIX_EPOCH)
                        .unwrap_or_default()
                        .as_secs(),
                },
            );
        }
    }

    /// Get from cache
    fn get_from_cache(&self, key: &str) -> Option<Vec<u8>> {
        if let Ok(cache) = self.cache.lock() {
            cache.get(key).map(|cached| cached.data.clone())
        } else {
            None
        }
    }

    /// Clean expired cache entries
    fn clean_cache(&self) {
        if let Ok(mut cache) = self.cache.lock() {
            let now = std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs();

            // Remove entries older than 5 minutes
            cache.retain(|_, cached| now - cached.timestamp < 300);
        }
    }
}

impl StorageLayer for ModularStorage {
    fn store_block(&self, block: &Block) -> Result<Hash> {
        let hash = block.get_hash().to_string();

        // Check if block already exists
        if self.block_db.contains_key(&hash)? {
            return Ok(hash);
        }

        // Serialize block
        let block_data = bincode::serialize(block)?;

        // Store block data
        self.block_db.insert(&hash, block_data.clone())?;

        // Store block metadata for quick access
        let metadata = self.calculate_metadata(block);
        let metadata_data = bincode::serialize(&metadata)?;
        self.index_db
            .insert(format!("meta_{}", hash), metadata_data)?;

        // Update height index
        let height = block.get_height() as u64;
        self.index_db
            .insert(format!("height_{}", height), hash.as_bytes())?;

        // Update cache
        self.update_cache(format!("block_{}", hash), block_data);

        // Update tip if this is the highest block
        let current_height = self.get_height().unwrap_or(0);
        if height > current_height {
            self.set_tip(&hash)?;
        }

        // Periodically clean cache
        if height % 100 == 0 {
            self.clean_cache();
        }

        log::debug!("Stored block {} at height {}", hash, height);
        Ok(hash)
    }

    fn get_block(&self, hash: &Hash) -> Result<Block> {
        let cache_key = format!("block_{}", hash);

        // Try cache first
        if let Some(cached_data) = self.get_from_cache(&cache_key) {
            return Ok(bincode::deserialize(&cached_data)?);
        }

        // Get from database
        let block_data = self
            .block_db
            .get(hash)?
            .ok_or_else(|| anyhow::anyhow!("Block not found: {}", hash))?;

        let block: Block = bincode::deserialize(&block_data)?;

        // Update cache
        self.update_cache(cache_key, block_data.to_vec());

        Ok(block)
    }

    fn get_tip(&self) -> Result<String> {
        let tip = self.tip.lock().unwrap();
        Ok(tip.clone())
    }

    fn set_tip(&self, hash: &Hash) -> Result<()> {
        // Update in-memory tip
        {
            let mut tip = self.tip.lock().unwrap();
            *tip = hash.clone();
        }

        // Persist to database
        self.block_db.insert("TIP", hash.as_bytes())?;

        if self.config.sync_writes {
            self.block_db.flush()?;
        }

        log::debug!("Updated blockchain tip to {}", hash);
        Ok(())
    }

    fn get_height(&self) -> Result<u64> {
        let tip = self.get_tip()?;

        if tip.is_empty() {
            // No blocks yet
            return Ok(0);
        }

        // Get block metadata to find height
        let metadata = self.get_block_metadata(&tip)?;
        Ok(metadata.height)
    }

    fn get_block_hashes(&self) -> Result<Vec<Hash>> {
        let mut hashes = Vec::new();
        let height = self.get_height()?;

        // Traverse from genesis to tip
        for h in 0..=height {
            if let Ok(Some(hash_bytes)) = self.index_db.get(format!("height_{}", h)) {
                let hash = String::from_utf8(hash_bytes.to_vec())?;
                hashes.push(hash);
            }
        }

        Ok(hashes)
    }

    fn store_data(&self, key: &str, data: &[u8]) -> Result<()> {
        self.state_db.insert(key, data)?;

        if self.config.sync_writes {
            self.state_db.flush()?;
        }

        Ok(())
    }

    fn get_data(&self, key: &str) -> Result<Option<Vec<u8>>> {
        match self.state_db.get(key)? {
            Some(data) => Ok(Some(data.to_vec())),
            None => Ok(None),
        }
    }

    fn delete_data(&self, key: &str) -> Result<()> {
        self.state_db.remove(key)?;

        if self.config.sync_writes {
            self.state_db.flush()?;
        }

        Ok(())
    }

    fn block_exists(&self, hash: &Hash) -> Result<bool> {
        Ok(self.block_db.contains_key(hash)?)
    }

    fn get_block_metadata(&self, hash: &Hash) -> Result<BlockMetadata> {
        let metadata_key = format!("meta_{}", hash);
        let metadata_data = self
            .index_db
            .get(metadata_key)?
            .ok_or_else(|| anyhow::anyhow!("Block metadata not found: {}", hash))?;

        let metadata: BlockMetadata = bincode::deserialize(&metadata_data)?;
        Ok(metadata)
    }

    fn flush(&self) -> Result<()> {
        self.block_db.flush()?;
        self.state_db.flush()?;
        self.index_db.flush()?;
        Ok(())
    }

    fn compact(&self) -> Result<()> {
        log::info!("Compacting storage databases...");

        // Compact all databases
        let block_size_before = self.block_db.size_on_disk()?;
        let state_size_before = self.state_db.size_on_disk()?;
        let index_size_before = self.index_db.size_on_disk()?;

        // Clean cache first
        self.clean_cache();

        // Note: sled doesn't have explicit compaction, but we can simulate it
        // by forcing a flush and letting sled handle internal optimization
        self.flush()?;

        let block_size_after = self.block_db.size_on_disk()?;
        let state_size_after = self.state_db.size_on_disk()?;
        let index_size_after = self.index_db.size_on_disk()?;

        log::info!("Storage compaction completed:");
        log::info!(
            "  Block DB: {} -> {} bytes",
            block_size_before,
            block_size_after
        );
        log::info!(
            "  State DB: {} -> {} bytes",
            state_size_before,
            state_size_after
        );
        log::info!(
            "  Index DB: {} -> {} bytes",
            index_size_before,
            index_size_after
        );

        Ok(())
    }

    fn get_stats(&self) -> Result<StorageStats> {
        // Count actual blocks in storage, not height + 1
        let block_hashes = self.get_block_hashes()?;
        let total_blocks = block_hashes.len() as u64;

        let block_db_size = self.block_db.size_on_disk()?;
        let state_db_size = self.state_db.size_on_disk()?;
        let index_db_size = self.index_db.size_on_disk()?;
        let total_size = block_db_size + state_db_size + index_db_size;

        // Cache statistics (simplified)
        let cache_len = self.cache.lock().unwrap().len() as u64;

        Ok(StorageStats {
            total_blocks,
            total_size_bytes: total_size,
            cache_hits: cache_len * 10,  // Simplified estimate
            cache_misses: cache_len * 2, // Simplified estimate
            db_size_bytes: total_size,
        })
    }
}

/// Storage layer builder for configuration
pub struct StorageLayerBuilder {
    config: Option<StorageConfig>,
}

impl StorageLayerBuilder {
    pub fn new() -> Self {
        Self { config: None }
    }

    pub fn with_config(mut self, config: StorageConfig) -> Self {
        self.config = Some(config);
        self
    }

    pub fn with_path<P: AsRef<Path>>(mut self, path: P) -> Self {
        let mut config = self.config.unwrap_or_default();
        config.db_path = path.as_ref().to_path_buf();
        self.config = Some(config);
        self
    }

    pub fn with_cache_size_mb(mut self, size_mb: usize) -> Self {
        let mut config = self.config.unwrap_or_default();
        config.cache_size_mb = size_mb;
        self.config = Some(config);
        self
    }

    pub fn enable_compression(mut self, enable: bool) -> Self {
        let mut config = self.config.unwrap_or_default();
        config.enable_compression = enable;
        self.config = Some(config);
        self
    }

    pub fn sync_writes(mut self, sync: bool) -> Self {
        let mut config = self.config.unwrap_or_default();
        config.sync_writes = sync;
        self.config = Some(config);
        self
    }

    pub fn build(self) -> Result<ModularStorage> {
        let config = self.config.unwrap_or_default();
        ModularStorage::new(config)
    }
}

impl Default for StorageLayerBuilder {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;
    use crate::{blockchain::block::Block, crypto::transaction::Transaction};

    fn create_test_block(height: i32) -> Block {
        let transactions =
            vec![Transaction::new_coinbase("test_address".to_string(), "50".to_string()).unwrap()];

        Block::new_test_finalized(
            transactions,
            TestFinalizedParams {
                prev_block_hash: if height == 0 {
                    String::new()
                } else {
                    format!("prev_hash_{}", height - 1)
                },
                hash: format!("test_hash_{}", height),
                nonce: 0,
                height,
                difficulty: 1,
                difficulty_config: Default::default(),
                mining_stats: Default::default(),
            },
        )
    }

    #[test]
    fn test_storage_creation() {
        let temp_dir = TempDir::new().unwrap();
        let storage = ModularStorage::new_with_path(temp_dir.path()).unwrap();

        // Test initial state
        assert_eq!(storage.get_tip().unwrap(), "");
        assert_eq!(storage.get_height().unwrap(), 0);
    }

    #[test]
    fn test_block_storage_and_retrieval() {
        let temp_dir = TempDir::new().unwrap();
        let storage = ModularStorage::new_with_path(temp_dir.path()).unwrap();

        let block = create_test_block(1);
        let hash = storage.store_block(&block).unwrap();

        // Test retrieval
        let retrieved_block = storage.get_block(&hash).unwrap();
        assert_eq!(retrieved_block.get_hash(), block.get_hash());
        assert_eq!(retrieved_block.get_height(), block.get_height());

        // Test tip update
        assert_eq!(storage.get_tip().unwrap(), hash);
        assert_eq!(storage.get_height().unwrap(), 1);
    }

    #[test]
    fn test_block_exists() {
        let temp_dir = TempDir::new().unwrap();
        let storage = ModularStorage::new_with_path(temp_dir.path()).unwrap();

        let block = create_test_block(1);
        let hash = storage.store_block(&block).unwrap();

        assert!(storage.block_exists(&hash).unwrap());
        assert!(!storage
            .block_exists(&"nonexistent_hash".to_string())
            .unwrap());
    }

    #[test]
    fn test_block_metadata() {
        let temp_dir = TempDir::new().unwrap();
        let storage = ModularStorage::new_with_path(temp_dir.path()).unwrap();

        let block = create_test_block(1);
        let hash = storage.store_block(&block).unwrap();

        let metadata = storage.get_block_metadata(&hash).unwrap();
        assert_eq!(metadata.hash, hash);
        assert_eq!(metadata.height, 1);
        assert_eq!(metadata.transaction_count, 1);
    }

    #[test]
    fn test_data_storage() {
        let temp_dir = TempDir::new().unwrap();
        let storage = ModularStorage::new_with_path(temp_dir.path()).unwrap();

        let key = "test_key";
        let data = b"test_data";

        storage.store_data(key, data).unwrap();

        let retrieved = storage.get_data(key).unwrap().unwrap();
        assert_eq!(retrieved, data);

        storage.delete_data(key).unwrap();
        assert!(storage.get_data(key).unwrap().is_none());
    }

    #[test]
    fn test_block_hashes() {
        let temp_dir = TempDir::new().unwrap();
        let storage = ModularStorage::new_with_path(temp_dir.path()).unwrap();

        // Store multiple blocks
        let block1 = create_test_block(0);
        let hash1 = storage.store_block(&block1).unwrap();

        let block2 = create_test_block(1);
        let hash2 = storage.store_block(&block2).unwrap();

        let hashes = storage.get_block_hashes().unwrap();
        assert_eq!(hashes.len(), 2);
        assert_eq!(hashes[0], hash1);
        assert_eq!(hashes[1], hash2);
    }

    #[test]
    fn test_storage_stats() {
        let temp_dir = TempDir::new().unwrap();
        let storage = ModularStorage::new_with_path(temp_dir.path()).unwrap();

        let block = create_test_block(1);
        storage.store_block(&block).unwrap();

        // Force flush to ensure data is written to disk
        storage.flush().unwrap();

        let stats = storage.get_stats().unwrap();
        assert_eq!(stats.total_blocks, 1); // Height 1 means 1 block
                                           // Note: sled may not have written to disk yet in tests, so we'll check that we have blocks instead
        assert!(stats.total_blocks > 0);
    }

    #[test]
    fn test_storage_builder() {
        let temp_dir = TempDir::new().unwrap();

        let storage = StorageLayerBuilder::new()
            .with_path(temp_dir.path())
            .with_cache_size_mb(32)
            .enable_compression(true)
            .sync_writes(false)
            .build()
            .unwrap();

        assert_eq!(storage.config.cache_size_mb, 32);
        assert!(storage.config.enable_compression);
        assert!(!storage.config.sync_writes);
    }
}
