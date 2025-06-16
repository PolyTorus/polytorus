//! Modular transaction processor
//!
//! This module provides transaction processing capabilities for the modular blockchain
//! architecture, independent of legacy UTXO systems.

use std::{
    collections::HashMap,
    sync::{Arc, Mutex},
};

use serde::{Deserialize, Serialize};

use crate::{
    crypto::transaction::{ContractTransactionData, ContractTransactionType, Transaction},
    Result,
};

/// Account-based state for modular transaction processing
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ProcessorAccountState {
    pub balance: u64,
    pub nonce: u64,
    pub code: Option<Vec<u8>>,
    pub storage: HashMap<String, Vec<u8>>,
}

/// Transaction processing result
#[derive(Debug, Clone)]
pub struct TransactionResult {
    pub success: bool,
    pub gas_used: u64,
    pub error: Option<String>,
    pub events: Vec<TransactionEvent>,
    pub state_changes: HashMap<String, ProcessorAccountState>,
}

/// Transaction event
#[derive(Debug, Clone)]
pub struct TransactionEvent {
    pub address: String,
    pub topics: Vec<String>,
    pub data: Vec<u8>,
}

/// Configuration for transaction processing
#[derive(Debug, Clone)]
pub struct TransactionProcessorConfig {
    pub gas_limit: u64,
    pub base_gas_cost: u64,
    pub enable_contracts: bool,
}

impl Default for TransactionProcessorConfig {
    fn default() -> Self {
        Self {
            gas_limit: 10_000_000,
            base_gas_cost: 21_000,
            enable_contracts: true,
        }
    }
}

/// Modular transaction processor
pub struct ModularTransactionProcessor {
    /// Account states
    states: Arc<Mutex<HashMap<String, ProcessorAccountState>>>,
    /// Processor configuration
    config: TransactionProcessorConfig,
    /// Transaction pool for pending transactions
    tx_pool: Arc<Mutex<Vec<Transaction>>>,
}

impl ModularTransactionProcessor {
    /// Create a new modular transaction processor
    pub fn new(config: TransactionProcessorConfig) -> Self {
        Self {
            states: Arc::new(Mutex::new(HashMap::new())),
            config,
            tx_pool: Arc::new(Mutex::new(Vec::new())),
        }
    }

    /// Add a transaction to the pool
    pub fn add_transaction(&self, transaction: Transaction) -> Result<()> {
        let mut pool = self
            .tx_pool
            .lock()
            .map_err(|_| failure::format_err!("Failed to acquire transaction pool lock"))?;

        // Basic validation
        if !self.validate_transaction(&transaction)? {
            return Err(failure::format_err!("Transaction validation failed"));
        }

        pool.push(transaction);
        Ok(())
    }

    /// Get pending transactions from the pool
    pub fn get_pending_transactions(&self) -> Result<Vec<Transaction>> {
        let pool = self
            .tx_pool
            .lock()
            .map_err(|_| failure::format_err!("Failed to acquire transaction pool lock"))?;
        Ok(pool.clone())
    }

    /// Process a single transaction
    pub fn process_transaction(&self, tx: &Transaction) -> Result<TransactionResult> {
        let mut result = TransactionResult {
            success: false,
            gas_used: self.config.base_gas_cost,
            error: None,
            events: Vec::new(),
            state_changes: HashMap::new(),
        };

        // Check if this is a contract transaction
        if let Some(contract_data) = &tx.contract_data {
            return self.process_contract_transaction(tx, contract_data);
        }

        // Process regular transaction (account-based)
        if let Err(e) = self.process_regular_transaction(tx, &mut result) {
            result.error = Some(e.to_string());
            return Ok(result);
        }

        result.success = true;
        Ok(result)
    }

    /// Process a batch of transactions
    pub fn process_transactions(
        &self,
        transactions: &[Transaction],
    ) -> Result<Vec<TransactionResult>> {
        let mut results = Vec::new();
        let mut total_gas_used = 0;

        for tx in transactions {
            let result = self.process_transaction(tx)?;
            total_gas_used += result.gas_used;

            // Check gas limit
            if total_gas_used > self.config.gas_limit {
                return Err(failure::format_err!("Block gas limit exceeded"));
            }

            // Apply state changes if transaction succeeded
            if result.success {
                self.apply_state_changes(&result.state_changes)?;
            }

            results.push(result);
        }

        Ok(results)
    }

    /// Get account state
    pub fn get_account_state(&self, address: &str) -> Result<ProcessorAccountState> {
        let states = self
            .states
            .lock()
            .map_err(|_| failure::format_err!("Failed to acquire states lock"))?;

        Ok(states.get(address).cloned().unwrap_or_default())
    }

    /// Set account state
    pub fn set_account_state(&self, address: &str, state: ProcessorAccountState) -> Result<()> {
        let mut states = self
            .states
            .lock()
            .map_err(|_| failure::format_err!("Failed to acquire states lock"))?;

        states.insert(address.to_string(), state);
        Ok(())
    }

    /// Clear the transaction pool
    pub fn clear_transaction_pool(&self) -> Result<()> {
        let mut pool = self
            .tx_pool
            .lock()
            .map_err(|_| failure::format_err!("Failed to acquire transaction pool lock"))?;
        pool.clear();
        Ok(())
    }

    /// Remove specific transactions from pool
    pub fn remove_transactions(&self, tx_ids: &[String]) -> Result<()> {
        let mut pool = self
            .tx_pool
            .lock()
            .map_err(|_| failure::format_err!("Failed to acquire transaction pool lock"))?;

        pool.retain(|tx| !tx_ids.contains(&tx.id));
        Ok(())
    }

    /// Validate a transaction
    fn validate_transaction(&self, _transaction: &Transaction) -> Result<bool> {
        // Basic validation logic
        // In a real implementation, this would include:
        // - Signature verification
        // - Balance checks
        // - Nonce validation
        // - Gas estimation
        Ok(true)
    }

    /// Process a regular (non-contract) transaction
    fn process_regular_transaction(
        &self,
        tx: &Transaction,
        result: &mut TransactionResult,
    ) -> Result<()> {
        // Check if this is a coinbase transaction (mining reward)
        if tx.vin.len() == 1 && tx.vin[0].txid.is_empty() && tx.vin[0].vout == -1 {
            // This is a coinbase transaction - just add rewards to outputs
            for output in &tx.vout {
                let receiver_address =
                    std::str::from_utf8(&output.pub_key_hash).unwrap_or("unknown_address");
                let mut receiver_state = self.get_account_state(receiver_address)?;
                receiver_state.balance += output.value as u64;

                result
                    .state_changes
                    .insert(receiver_address.to_string(), receiver_state);

                result.events.push(TransactionEvent {
                    address: receiver_address.to_string(),
                    topics: vec!["coinbase_reward".to_string()],
                    data: format!("Coinbase reward: {}", output.value).into_bytes(),
                });
            }
            return Ok(());
        }

        // Regular transaction processing (simplified)
        // Extract sender and receiver from transaction
        // This is simplified - in reality, you'd extract from inputs/outputs
        let sender = "sender_address"; // Extract from transaction signature
        let receiver = "receiver_address"; // Extract from transaction outputs
        let amount = 100; // Extract from transaction outputs

        // Get current states
        let mut sender_state = self.get_account_state(sender)?;
        let mut receiver_state = self.get_account_state(receiver)?;

        // Check balance
        if sender_state.balance < amount {
            return Err(failure::format_err!("Insufficient balance"));
        }

        // Update states
        sender_state.balance -= amount;
        sender_state.nonce += 1;
        receiver_state.balance += amount;

        // Record state changes
        result
            .state_changes
            .insert(sender.to_string(), sender_state);
        result
            .state_changes
            .insert(receiver.to_string(), receiver_state);

        // Create transfer event
        result.events.push(TransactionEvent {
            address: sender.to_string(),
            topics: vec!["transfer".to_string()],
            data: format!("Transferred {} to {}", amount, receiver).into_bytes(),
        });

        Ok(())
    }

    /// Process a contract transaction
    fn process_contract_transaction(
        &self,
        tx: &Transaction,
        contract_data: &ContractTransactionData,
    ) -> Result<TransactionResult> {
        let mut result = TransactionResult {
            success: false,
            gas_used: self.config.base_gas_cost,
            error: None,
            events: Vec::new(),
            state_changes: HashMap::new(),
        };

        if !self.config.enable_contracts {
            result.error = Some("Contract execution disabled".to_string());
            return Ok(result);
        }

        match &contract_data.tx_type {
            ContractTransactionType::Deploy {
                bytecode,
                constructor_args,
                gas_limit,
            } => {
                result.gas_used += gas_limit / 10; // Simple gas calculation

                // Create contract account
                let contract_address = format!("contract_{}", tx.id);
                let mut contract_state = ProcessorAccountState {
                    code: Some(bytecode.clone()),
                    ..Default::default()
                };

                // Store constructor arguments in contract state for initialization
                if !constructor_args.is_empty() {
                    contract_state
                        .storage
                        .insert("constructor_args".to_string(), constructor_args.clone());
                    result.gas_used += constructor_args.len() as u64 / 100; // Gas for constructor args
                }

                result
                    .state_changes
                    .insert(contract_address.clone(), contract_state);

                result.events.push(TransactionEvent {
                    address: contract_address,
                    topics: vec!["contract_deployed".to_string()],
                    data: format!("Contract deployed with {} bytes", bytecode.len()).into_bytes(),
                });

                result.success = true;
            }
            ContractTransactionType::Call {
                contract_address,
                function_name,
                arguments: _,
                gas_limit,
                value: _,
            } => {
                result.gas_used += gas_limit / 10; // Simple gas calculation

                // Verify contract exists
                let contract_state = self.get_account_state(contract_address)?;
                if contract_state.code.is_none() {
                    result.error = Some("Contract not found".to_string());
                    return Ok(result);
                }

                result.events.push(TransactionEvent {
                    address: contract_address.clone(),
                    topics: vec!["contract_called".to_string(), function_name.clone()],
                    data: format!("Function {} called", function_name).into_bytes(),
                });

                result.success = true;
            }
        }

        Ok(result)
    }

    /// Apply state changes to the global state
    fn apply_state_changes(&self, changes: &HashMap<String, ProcessorAccountState>) -> Result<()> {
        let mut states = self
            .states
            .lock()
            .map_err(|_| failure::format_err!("Failed to acquire states lock"))?;

        for (address, state) in changes {
            states.insert(address.clone(), state.clone());
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::transaction::Transaction;

    #[test]
    fn test_new_transaction_processor() {
        let config = TransactionProcessorConfig::default();
        let processor = ModularTransactionProcessor::new(config);

        // Test initial state
        let account_state = processor.get_account_state("test_address").unwrap();
        assert_eq!(account_state.balance, 0);
        assert_eq!(account_state.nonce, 0);
    }

    #[test]
    fn test_account_state_management() {
        let config = TransactionProcessorConfig::default();
        let processor = ModularTransactionProcessor::new(config);

        let test_address = "test_address";
        let state = ProcessorAccountState {
            balance: 1000,
            nonce: 1,
            ..Default::default()
        };

        processor
            .set_account_state(test_address, state.clone())
            .unwrap();

        let retrieved_state = processor.get_account_state(test_address).unwrap();
        assert_eq!(retrieved_state.balance, 1000);
        assert_eq!(retrieved_state.nonce, 1);
    }

    #[test]
    fn test_transaction_pool() {
        let config = TransactionProcessorConfig::default();
        let processor = ModularTransactionProcessor::new(config);

        let tx = Transaction {
            id: "test_tx".to_string(),
            vin: vec![],
            vout: vec![],
            contract_data: None,
        };

        processor.add_transaction(tx.clone()).unwrap();

        let pending = processor.get_pending_transactions().unwrap();
        assert_eq!(pending.len(), 1);
        assert_eq!(pending[0].id, "test_tx");

        processor.clear_transaction_pool().unwrap();
        let pending = processor.get_pending_transactions().unwrap();
        assert_eq!(pending.len(), 0);
    }
}
