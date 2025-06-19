//! Transaction Mempool Implementation
//!
//! This module provides a comprehensive transaction mempool with validation,
//! prioritization, and management for the modular blockchain architecture.

use std::{
    collections::{BTreeMap, HashMap, HashSet, VecDeque},
    sync::{Arc, RwLock},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::crypto::transaction::Transaction;

/// Transaction priority levels for mempool ordering
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum TransactionPriority {
    Low = 1,
    Normal = 2,
    High = 3,
    Critical = 4,
}

impl Default for TransactionPriority {
    fn default() -> Self {
        Self::Normal
    }
}

/// Transaction status in the mempool
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum TransactionStatus {
    Pending,
    Validated,
    Invalid(String),
    Included(String), // Block hash
    Expired,
}

/// Transaction wrapper with metadata for mempool management
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MempoolTransaction {
    pub transaction: Transaction,
    pub priority: TransactionPriority,
    pub status: TransactionStatus,
    pub received_at: u64,
    pub validated_at: Option<u64>,
    pub attempts: u32,
    pub fee: u64,
    pub gas_price: u64,
    pub dependencies: Vec<String>, // Transaction IDs this depends on
}

impl MempoolTransaction {
    pub fn new(transaction: Transaction, fee: u64, gas_price: u64) -> Self {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs(); // Keep as seconds for storage

        // Calculate priority based on fee and gas price
        let priority = if gas_price > 1000 {
            TransactionPriority::High
        } else if gas_price >= 100 {
            TransactionPriority::Normal
        } else {
            TransactionPriority::Low
        };

        Self {
            transaction,
            priority,
            status: TransactionStatus::Pending,
            received_at: now,
            validated_at: None,
            attempts: 0,
            fee,
            gas_price,
            dependencies: Vec::new(),
        }
    }

    pub fn get_id(&self) -> String {
        self.transaction.get_id()
    }

    pub fn get_score(&self) -> u64 {
        // Score based on fee, gas price, and age (older = higher score)
        let age_bonus = (SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs()
            .saturating_sub(self.received_at))
        .min(3600); // Cap at 1 hour

        self.fee + (self.gas_price * 10) + age_bonus
    }
}

/// Mempool configuration parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MempoolConfig {
    pub max_transactions: usize,
    pub max_transaction_age: Duration,
    pub max_attempts: u32,
    pub validation_timeout: Duration,
    pub min_fee: u64,
    pub max_transaction_size: usize,
    pub enable_fee_estimation: bool,
    pub cleanup_interval: Duration,
}

impl Default for MempoolConfig {
    fn default() -> Self {
        Self {
            max_transactions: 10000,
            max_transaction_age: Duration::from_secs(3600), // 1 hour
            max_attempts: 3,
            validation_timeout: Duration::from_secs(30),
            min_fee: 1,
            max_transaction_size: 1024 * 1024, // 1MB
            enable_fee_estimation: true,
            cleanup_interval: Duration::from_secs(60), // 1 minute
        }
    }
}

/// Mempool statistics for monitoring
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MempoolStats {
    pub total_transactions: usize,
    pub pending_transactions: usize,
    pub validated_transactions: usize,
    pub invalid_transactions: usize,
    pub expired_transactions: usize,
    pub average_fee: f64,
    pub memory_usage_bytes: usize,
    pub last_cleanup: u64,
}

/// Events emitted by the mempool
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MempoolEvent {
    TransactionAdded {
        transaction_id: String,
        priority: TransactionPriority,
    },
    TransactionValidated {
        transaction_id: String,
        is_valid: bool,
        validation_time_ms: u64,
    },
    TransactionIncluded {
        transaction_id: String,
        block_hash: String,
    },
    TransactionExpired {
        transaction_id: String,
        reason: String,
    },
    MempoolFull {
        rejected_transaction_id: String,
        current_size: usize,
    },
}

/// Comprehensive transaction mempool implementation
pub struct TransactionMempool {
    /// Configuration parameters
    config: MempoolConfig,

    /// Transactions indexed by ID
    transactions: Arc<RwLock<HashMap<String, MempoolTransaction>>>,

    /// Priority-ordered transactions for selection
    priority_queue: Arc<RwLock<BTreeMap<u64, String>>>, // score -> tx_id

    /// Transactions by status
    pending_transactions: Arc<RwLock<VecDeque<String>>>,
    validated_transactions: Arc<RwLock<VecDeque<String>>>,

    /// Nonce tracking for accounts
    account_nonces: Arc<RwLock<HashMap<String, u64>>>,

    /// Transaction dependencies
    dependency_graph: Arc<RwLock<HashMap<String, HashSet<String>>>>,

    /// Event channel
    event_tx: mpsc::UnboundedSender<MempoolEvent>,

    /// Statistics
    stats: Arc<RwLock<MempoolStats>>,

    /// Fee estimation
    recent_fees: Arc<RwLock<VecDeque<u64>>>,
}

impl TransactionMempool {
    /// Create a new transaction mempool
    pub fn new(config: MempoolConfig) -> (Self, mpsc::UnboundedReceiver<MempoolEvent>) {
        let (event_tx, event_rx) = mpsc::unbounded_channel();

        let mempool = Self {
            config,
            transactions: Arc::new(RwLock::new(HashMap::new())),
            priority_queue: Arc::new(RwLock::new(BTreeMap::new())),
            pending_transactions: Arc::new(RwLock::new(VecDeque::new())),
            validated_transactions: Arc::new(RwLock::new(VecDeque::new())),
            account_nonces: Arc::new(RwLock::new(HashMap::new())),
            dependency_graph: Arc::new(RwLock::new(HashMap::new())),
            event_tx,
            stats: Arc::new(RwLock::new(MempoolStats {
                total_transactions: 0,
                pending_transactions: 0,
                validated_transactions: 0,
                invalid_transactions: 0,
                expired_transactions: 0,
                average_fee: 0.0,
                memory_usage_bytes: 0,
                last_cleanup: 0,
            })),
            recent_fees: Arc::new(RwLock::new(VecDeque::new())),
        };

        (mempool, event_rx)
    }

    /// Add a transaction to the mempool
    pub async fn add_transaction(
        &self,
        transaction: Transaction,
        fee: u64,
        gas_price: u64,
    ) -> Result<()> {
        // Validate basic transaction parameters
        if fee < self.config.min_fee {
            return Err(anyhow!(
                "Transaction fee {} below minimum {}",
                fee,
                self.config.min_fee
            ));
        }

        // Check mempool capacity
        {
            let transactions = self.transactions.read().unwrap();
            if transactions.len() >= self.config.max_transactions {
                let tx_id = transaction.get_id();
                let _ = self.event_tx.send(MempoolEvent::MempoolFull {
                    rejected_transaction_id: tx_id,
                    current_size: transactions.len(),
                });
                return Err(anyhow!("Mempool is full"));
            }
        }

        let mempool_tx = MempoolTransaction::new(transaction, fee, gas_price);
        let tx_id = mempool_tx.get_id();
        let priority = mempool_tx.priority;
        let score = mempool_tx.get_score();

        // Add to main storage
        {
            let mut transactions = self.transactions.write().unwrap();
            transactions.insert(tx_id.clone(), mempool_tx);
        }

        // Add to priority queue
        {
            let mut priority_queue = self.priority_queue.write().unwrap();
            priority_queue.insert(score, tx_id.clone());
        }

        // Add to pending queue
        {
            let mut pending = self.pending_transactions.write().unwrap();
            pending.push_back(tx_id.clone());
        }

        // Update statistics
        self.update_stats().await;

        // Emit event
        let _ = self.event_tx.send(MempoolEvent::TransactionAdded {
            transaction_id: tx_id,
            priority,
        });

        Ok(())
    }

    /// Validate a pending transaction
    pub async fn validate_transaction(&self, transaction_id: &str) -> Result<bool> {
        let start_time = SystemTime::now();

        // Get transaction for validation
        let transaction = {
            let transactions = self.transactions.read().unwrap();
            if let Some(tx) = transactions.get(transaction_id) {
                tx.transaction.clone()
            } else {
                return Err(anyhow!("Transaction not found: {}", transaction_id));
            }
        };

        // Validate transaction logic
        let is_valid = self.validate_transaction_logic(&transaction).await?;

        // Update transaction status
        {
            let mut transactions = self.transactions.write().unwrap();
            if let Some(tx) = transactions.get_mut(transaction_id) {
                if is_valid {
                    tx.status = TransactionStatus::Validated;
                    tx.validated_at = Some(
                        SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_secs(),
                    );

                    // Move to validated queue
                    let mut validated = self.validated_transactions.write().unwrap();
                    validated.push_back(transaction_id.to_string());
                } else {
                    tx.status = TransactionStatus::Invalid("Validation failed".to_string());
                }
            }
        };

        let validation_time = start_time.elapsed().unwrap().as_millis() as u64;

        // Emit validation event
        let _ = self.event_tx.send(MempoolEvent::TransactionValidated {
            transaction_id: transaction_id.to_string(),
            is_valid,
            validation_time_ms: validation_time,
        });

        self.update_stats().await;
        Ok(is_valid)
    }

    /// Get transactions for block creation
    pub async fn get_transactions_for_block(
        &self,
        max_transactions: usize,
        max_gas: u64,
    ) -> Result<Vec<Transaction>> {
        let mut selected_transactions = Vec::new();
        let mut total_gas = 0u64;

        // Get transactions ordered by priority/score
        let priority_queue = self.priority_queue.read().unwrap();
        let transactions = self.transactions.read().unwrap();

        for (_, tx_id) in priority_queue.iter().rev() {
            if selected_transactions.len() >= max_transactions {
                break;
            }

            if let Some(mempool_tx) = transactions.get(tx_id) {
                if mempool_tx.status == TransactionStatus::Validated {
                    // Estimate gas (simplified)
                    let estimated_gas = 21000u64; // Base transaction gas

                    if total_gas + estimated_gas <= max_gas {
                        selected_transactions.push(mempool_tx.transaction.clone());
                        total_gas += estimated_gas;
                    }
                }
            }
        }

        Ok(selected_transactions)
    }

    /// Mark transactions as included in a block
    pub async fn mark_transactions_included(
        &self,
        transaction_ids: &[String],
        block_hash: &str,
    ) -> Result<()> {
        {
            let mut transactions = self.transactions.write().unwrap();
            for tx_id in transaction_ids {
                if let Some(tx) = transactions.get_mut(tx_id) {
                    tx.status = TransactionStatus::Included(block_hash.to_string());

                    // Emit event
                    let _ = self.event_tx.send(MempoolEvent::TransactionIncluded {
                        transaction_id: tx_id.clone(),
                        block_hash: block_hash.to_string(),
                    });
                }
            }
        }

        let _ = self.cleanup_included_transactions().await;
        self.update_stats().await;
        Ok(())
    }

    /// Remove expired transactions
    pub async fn cleanup_expired_transactions(&self) -> Result<usize> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_secs();

        let mut expired_count = 0;
        let max_age = self.config.max_transaction_age.as_secs();

        let mut expired_ids = Vec::new();

        {
            let transactions = self.transactions.read().unwrap();
            for (tx_id, tx) in transactions.iter() {
                let age = now.saturating_sub(tx.received_at);
                if age >= max_age {
                    // Changed from > to >= to be more inclusive
                    expired_ids.push(tx_id.clone());
                }
            }
        }

        for tx_id in expired_ids {
            self.remove_transaction(&tx_id).await?;
            expired_count += 1;

            let _ = self.event_tx.send(MempoolEvent::TransactionExpired {
                transaction_id: tx_id,
                reason: "Transaction expired".to_string(),
            });
        }

        // Update cleanup timestamp
        {
            let mut stats = self.stats.write().unwrap();
            stats.last_cleanup = now;
        }

        // Update stats after cleanup
        self.update_stats().await;

        Ok(expired_count)
    }

    /// Get mempool statistics
    pub async fn get_stats(&self) -> MempoolStats {
        self.stats.read().unwrap().clone()
    }

    /// Estimate transaction fee
    pub async fn estimate_fee(&self) -> u64 {
        if !self.config.enable_fee_estimation {
            return self.config.min_fee;
        }

        let recent_fees = self.recent_fees.read().unwrap();
        if recent_fees.is_empty() {
            return self.config.min_fee;
        }

        let sum: u64 = recent_fees.iter().sum();
        let average = sum / recent_fees.len() as u64;

        // Return slightly above average for priority
        (average as f64 * 1.1) as u64
    }

    /// Get transaction by ID
    pub async fn get_transaction(&self, transaction_id: &str) -> Option<MempoolTransaction> {
        self.transactions
            .read()
            .unwrap()
            .get(transaction_id)
            .cloned()
    }

    /// Get account nonce
    pub fn get_account_nonce(&self, address: &str) -> Option<u64> {
        self.account_nonces.read().unwrap().get(address).copied()
    }

    /// Get transaction dependencies
    pub fn get_transaction_dependencies(&self, transaction_id: &str) -> Vec<String> {
        self.dependency_graph
            .read()
            .unwrap()
            .get(transaction_id)
            .map(|deps| deps.iter().cloned().collect())
            .unwrap_or_default()
    }

    /// Remove transaction from mempool
    async fn remove_transaction(&self, transaction_id: &str) -> Result<()> {
        // Remove from main storage
        let removed_tx = {
            let mut transactions = self.transactions.write().unwrap();
            transactions.remove(transaction_id)
        };

        if let Some(tx) = removed_tx {
            // Remove from priority queue
            {
                let mut priority_queue = self.priority_queue.write().unwrap();
                let score = tx.get_score();
                priority_queue.remove(&score);
            }

            // Remove from pending/validated queues
            {
                let mut pending = self.pending_transactions.write().unwrap();
                pending.retain(|id| id != transaction_id);
            }
            {
                let mut validated = self.validated_transactions.write().unwrap();
                validated.retain(|id| id != transaction_id);
            }
        }

        Ok(())
    }

    /// Clean up included transactions
    async fn cleanup_included_transactions(&self) -> Result<()> {
        let mut included_ids = Vec::new();

        {
            let transactions = self.transactions.read().unwrap();
            for (tx_id, tx) in transactions.iter() {
                if matches!(tx.status, TransactionStatus::Included(_)) {
                    included_ids.push(tx_id.clone());
                }
            }
        }

        for tx_id in included_ids {
            self.remove_transaction(&tx_id).await?;
        }

        Ok(())
    }

    /// Update mempool statistics
    async fn update_stats(&self) {
        let transactions = self.transactions.read().unwrap();

        let mut stats = self.stats.write().unwrap();

        stats.total_transactions = transactions.len();
        stats.pending_transactions = transactions
            .values()
            .filter(|tx| tx.status == TransactionStatus::Pending)
            .count();
        stats.validated_transactions = transactions
            .values()
            .filter(|tx| tx.status == TransactionStatus::Validated)
            .count();
        stats.invalid_transactions = transactions
            .values()
            .filter(|tx| matches!(tx.status, TransactionStatus::Invalid(_)))
            .count();

        // Calculate average fee
        if !transactions.is_empty() {
            let total_fee: u64 = transactions.values().map(|tx| tx.fee).sum();
            stats.average_fee = total_fee as f64 / transactions.len() as f64;
        }

        // Estimate memory usage (simplified)
        stats.memory_usage_bytes = transactions.len() * 1024; // Rough estimate
    }

    /// Validate transaction logic (implement actual validation)
    async fn validate_transaction_logic(&self, _transaction: &Transaction) -> Result<bool> {
        // Implement actual transaction validation logic here
        // For now, return true as a placeholder
        Ok(true)
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;

    #[tokio::test]
    async fn test_mempool_basic_operations() {
        let config = MempoolConfig::default();
        let (mempool, mut event_rx) = TransactionMempool::new(config);

        // Create a test transaction
        let transaction = Transaction::new("test_from".to_string(), "test_to".to_string(), 100);

        // Add transaction
        mempool
            .add_transaction(transaction.clone(), 10, 100)
            .await
            .unwrap();

        // Check event was emitted
        if let Some(event) = event_rx.recv().await {
            match event {
                MempoolEvent::TransactionAdded {
                    transaction_id,
                    priority,
                } => {
                    assert_eq!(transaction_id, transaction.get_id());
                    assert_eq!(priority, TransactionPriority::Normal);
                }
                _ => panic!("Unexpected event"),
            }
        }

        // Get stats
        let stats = mempool.get_stats().await;
        assert_eq!(stats.total_transactions, 1);
        assert_eq!(stats.pending_transactions, 1);
    }

    #[tokio::test]
    async fn test_transaction_validation() {
        let config = MempoolConfig::default();
        let (mempool, mut event_rx) = TransactionMempool::new(config);

        let transaction = Transaction::new("test_from".to_string(), "test_to".to_string(), 100);
        let tx_id = transaction.get_id();

        mempool.add_transaction(transaction, 10, 100).await.unwrap();

        // Skip add event
        event_rx.recv().await;

        // Validate transaction
        let is_valid = mempool.validate_transaction(&tx_id).await.unwrap();
        assert!(is_valid);

        // Check validation event
        if let Some(event) = event_rx.recv().await {
            match event {
                MempoolEvent::TransactionValidated {
                    transaction_id,
                    is_valid,
                    ..
                } => {
                    assert_eq!(transaction_id, tx_id);
                    assert!(is_valid);
                }
                _ => panic!("Unexpected event"),
            }
        }
    }

    #[tokio::test]
    async fn test_transaction_selection() {
        let config = MempoolConfig::default();
        let (mempool, _) = TransactionMempool::new(config);

        // Add multiple transactions with different fees
        for i in 0..5 {
            let transaction = Transaction::new(
                format!("from_{}", i),
                format!("to_{}", i),
                100 + i as u64 * 10,
            );
            let fee = 10 + i as u64 * 5;
            let gas_price = 100 + i as u64 * 50;

            mempool
                .add_transaction(transaction.clone(), fee, gas_price)
                .await
                .unwrap();
            mempool
                .validate_transaction(&transaction.get_id())
                .await
                .unwrap();
        }

        // Get transactions for block
        let selected = mempool
            .get_transactions_for_block(3, 1000000)
            .await
            .unwrap();
        assert_eq!(selected.len(), 3);
    }

    #[tokio::test]
    async fn test_mempool_cleanup() {
        let config = MempoolConfig {
            max_transaction_age: Duration::from_millis(100),
            ..Default::default()
        };

        let (mempool, _) = TransactionMempool::new(config);

        let transaction = Transaction::new("test_from".to_string(), "test_to".to_string(), 100);
        mempool.add_transaction(transaction, 10, 100).await.unwrap();

        // Wait for expiration
        tokio::time::sleep(Duration::from_millis(150)).await;

        // Cleanup expired transactions
        let expired_count = mempool.cleanup_expired_transactions().await.unwrap();
        assert_eq!(expired_count, 1);

        let stats = mempool.get_stats().await;
        assert_eq!(stats.total_transactions, 0);
    }
}
