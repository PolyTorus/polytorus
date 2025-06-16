//! Extended UTXO (eUTXO) processor for modular blockchain architecture
//!
//! This module integrates the eUTXO transaction model into the modular blockchain
//! architecture, providing script validation, datum handling, and redeemer support.

use std::collections::HashMap;
use std::sync::{
    Arc,
    Mutex,
};

use serde::{
    Deserialize,
    Serialize,
};

use crate::crypto::privacy::{
    PrivacyConfig,
    PrivacyProvider,
    PrivateTransaction,
    PrivacyStats,
};
use crate::crypto::transaction::{
    TXOutput,
    Transaction,
};
use crate::modular::transaction_processor::{
    ProcessorAccountState,
    TransactionEvent,
    TransactionResult,
};
use crate::Result;

/// UTXO state for tracking unspent outputs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UtxoState {
    /// Transaction ID
    pub txid: String,
    /// Output index
    pub vout: i32,
    /// The actual output
    pub output: TXOutput,
    /// Block height when this UTXO was created
    pub block_height: u64,
    /// Whether this UTXO is spent
    pub is_spent: bool,
}

/// Extended UTXO processor configuration
#[derive(Debug, Clone)]
pub struct EUtxoProcessorConfig {
    /// Maximum script size in bytes
    pub max_script_size: usize,
    /// Maximum datum size in bytes
    pub max_datum_size: usize,
    /// Maximum redeemer size in bytes
    pub max_redeemer_size: usize,
    /// Gas cost per script byte
    pub script_gas_cost: u64,
    /// Base gas cost for UTXO operations
    pub utxo_base_gas: u64,
    /// Privacy configuration for confidential transactions
    pub privacy_config: PrivacyConfig,
}

impl Default for EUtxoProcessorConfig {
    fn default() -> Self {
        Self {
            max_script_size: 32768,  // 32KB
            max_datum_size: 8192,    // 8KB
            max_redeemer_size: 8192, // 8KB
            script_gas_cost: 10,
            utxo_base_gas: 5000,
            privacy_config: PrivacyConfig::default(),
        }
    }
}

/// Extended UTXO processor for modular blockchain
pub struct EUtxoProcessor {
    /// UTXO set
    utxo_set: Arc<Mutex<HashMap<String, UtxoState>>>,
    /// Account states (for hybrid model)
    account_states: Arc<Mutex<HashMap<String, ProcessorAccountState>>>,
    /// Configuration
    config: EUtxoProcessorConfig,
    /// Privacy provider for confidential transactions
    privacy_provider: Arc<Mutex<PrivacyProvider>>,
}

impl EUtxoProcessor {
    /// Create a new eUTXO processor
    pub fn new(config: EUtxoProcessorConfig) -> Self {
        let privacy_provider = PrivacyProvider::new(config.privacy_config.clone());
        Self {
            utxo_set: Arc::new(Mutex::new(HashMap::new())),
            account_states: Arc::new(Mutex::new(HashMap::new())),
            config,
            privacy_provider: Arc::new(Mutex::new(privacy_provider)),
        }
    }

    /// Process a transaction using eUTXO model
    pub fn process_transaction(&self, tx: &Transaction) -> Result<TransactionResult> {
        let mut result = TransactionResult {
            success: false,
            gas_used: self.config.utxo_base_gas,
            error: None,
            events: Vec::new(),
            state_changes: HashMap::new(),
        };

        // Validate inputs
        if let Err(e) = self.validate_inputs(tx, &mut result) {
            result.error = Some(e.to_string());
            return Ok(result);
        }

        // Process outputs
        if let Err(e) = self.process_outputs(tx, &mut result) {
            result.error = Some(e.to_string());
            return Ok(result);
        }

        // Update UTXO set
        if let Err(e) = self.update_utxo_set(tx) {
            result.error = Some(e.to_string());
            return Ok(result);
        }

        result.success = true;
        Ok(result)
    }

    /// Validate transaction inputs using eUTXO rules
    fn validate_inputs(&self, tx: &Transaction, result: &mut TransactionResult) -> Result<()> {
        let utxo_set = self
            .utxo_set
            .lock()
            .map_err(|_| failure::format_err!("Failed to acquire UTXO set lock"))?;

        for input in &tx.vin {
            // Skip coinbase inputs
            if input.txid.is_empty() && input.vout == -1 {
                continue;
            }

            // Find the referenced UTXO
            let utxo_key = format!("{}:{}", input.txid, input.vout);
            let utxo = utxo_set
                .get(&utxo_key)
                .ok_or_else(|| failure::format_err!("UTXO not found: {}", utxo_key))?;

            if utxo.is_spent {
                return Err(failure::format_err!("UTXO already spent: {}", utxo_key));
            }

            // Validate spending conditions (script + redeemer)
            if !utxo.output.validate_spending(input)? {
                return Err(failure::format_err!(
                    "Invalid spending conditions for UTXO: {}",
                    utxo_key
                ));
            }

            // Calculate gas for script execution
            if let Some(ref script) = utxo.output.script {
                result.gas_used += script.len() as u64 * self.config.script_gas_cost;
            }

            // Calculate gas for redeemer
            if let Some(ref redeemer) = input.redeemer {
                result.gas_used += redeemer.len() as u64 / 10;
            }

            result.events.push(TransactionEvent {
                address: format!("utxo_{}", utxo_key),
                topics: vec!["utxo_spent".to_string()],
                data: format!("UTXO {} spent with value {}", utxo_key, utxo.output.value)
                    .into_bytes(),
            });
        }

        Ok(())
    }

    /// Process transaction outputs
    fn process_outputs(&self, tx: &Transaction, result: &mut TransactionResult) -> Result<()> {
        for (index, output) in tx.vout.iter().enumerate() {
            // Validate output constraints
            if let Some(ref script) = output.script {
                if script.len() > self.config.max_script_size {
                    return Err(failure::format_err!(
                        "Script too large: {} bytes",
                        script.len()
                    ));
                }
            }

            if let Some(ref datum) = output.datum {
                if datum.len() > self.config.max_datum_size {
                    return Err(failure::format_err!(
                        "Datum too large: {} bytes",
                        datum.len()
                    ));
                }
            }

            let utxo_key = format!("{}:{}", tx.id, index);
            result.events.push(TransactionEvent {
                address: format!("utxo_{}", utxo_key),
                topics: vec!["utxo_created".to_string()],
                data: format!("UTXO {} created with value {}", utxo_key, output.value).into_bytes(),
            });

            // If this is an eUTXO, add extra gas
            if output.is_eUTXO() {
                result.gas_used += 1000; // Extra gas for eUTXO features
            }
        }

        Ok(())
    }

    /// Update the UTXO set after transaction processing
    fn update_utxo_set(&self, tx: &Transaction) -> Result<()> {
        let mut utxo_set = self
            .utxo_set
            .lock()
            .map_err(|_| failure::format_err!("Failed to acquire UTXO set lock"))?;

        // Mark spent UTXOs
        for input in &tx.vin {
            // Skip coinbase inputs
            if input.txid.is_empty() && input.vout == -1 {
                continue;
            }

            let utxo_key = format!("{}:{}", input.txid, input.vout);
            if let Some(utxo) = utxo_set.get_mut(&utxo_key) {
                utxo.is_spent = true;
            }
        }

        // Add new UTXOs
        for (index, output) in tx.vout.iter().enumerate() {
            let utxo_key = format!("{}:{}", tx.id, index);
            let utxo_state = UtxoState {
                txid: tx.id.clone(),
                vout: index as i32,
                output: output.clone(),
                block_height: 0, // This would be set by the consensus layer
                is_spent: false,
            };
            utxo_set.insert(utxo_key, utxo_state);
        }

        Ok(())
    }

    /// Get UTXO by transaction ID and output index
    pub fn get_utxo(&self, txid: &str, vout: i32) -> Result<Option<UtxoState>> {
        let utxo_set = self
            .utxo_set
            .lock()
            .map_err(|_| failure::format_err!("Failed to acquire UTXO set lock"))?;

        let utxo_key = format!("{}:{}", txid, vout);
        Ok(utxo_set.get(&utxo_key).cloned())
    }

    /// Get all UTXOs for a given address
    pub fn get_utxos_for_address(&self, address: &str) -> Result<Vec<UtxoState>> {
        let utxo_set = self
            .utxo_set
            .lock()
            .map_err(|_| failure::format_err!("Failed to acquire UTXO set lock"))?;

        let mut result = Vec::new();

        // Calculate expected pub_key_hash for the address
        let expected_pub_key_hash = self.address_to_pub_key_hash(address)?;

        for utxo in utxo_set.values() {
            if !utxo.is_spent {
                // Check if this UTXO belongs to the address by comparing pub_key_hash
                if utxo.output.pub_key_hash == expected_pub_key_hash {
                    result.push(utxo.clone());
                }
            }
        }

        Ok(result)
    }

    /// Get account balance (sum of UTXOs)
    pub fn get_balance(&self, address: &str) -> Result<u64> {
        let utxos = self.get_utxos_for_address(address)?;
        let balance = utxos.iter().map(|utxo| utxo.output.value as u64).sum();
        Ok(balance)
    }

    /// Find spendable UTXOs for a given amount
    pub fn find_spendable_utxos(&self, address: &str, amount: u64) -> Result<Vec<UtxoState>> {
        let utxos = self.get_utxos_for_address(address)?;
        let mut spendable = Vec::new();
        let mut total = 0u64;

        for utxo in utxos {
            spendable.push(utxo.clone());
            total += utxo.output.value as u64;
            if total >= amount {
                break;
            }
        }

        if total < amount {
            return Err(failure::format_err!(
                "Insufficient balance: need {}, have {}",
                amount,
                total
            ));
        }

        Ok(spendable)
    }

    /// Create a hybrid account state that includes UTXO information
    pub fn get_hybrid_account_state(&self, address: &str) -> Result<ProcessorAccountState> {
        let balance = self.get_balance(address)?;
        let utxos = self.get_utxos_for_address(address)?;

        // Check if we have an existing account state
        let account_states = self
            .account_states
            .lock()
            .map_err(|_| failure::format_err!("Failed to acquire account states lock"))?;

        let mut state = account_states.get(address).cloned().unwrap_or_default();

        // Update balance from UTXO set
        state.balance = balance;

        // Store UTXO information in storage
        let utxo_data = bincode::serialize(&utxos)?;
        state.storage.insert("utxos".to_string(), utxo_data);

        Ok(state)
    }

    /// Process a private transaction with confidential amounts and ZK proofs
    pub fn process_private_transaction(&self, private_tx: &PrivateTransaction) -> Result<TransactionResult> {
        let mut result = TransactionResult {
            success: false,
            gas_used: self.config.utxo_base_gas,
            error: None,
            events: Vec::new(),
            state_changes: HashMap::new(),
        };

        // Verify the private transaction
        let privacy_provider = self.privacy_provider
            .lock()
            .map_err(|_| failure::format_err!("Failed to acquire privacy provider lock"))?;

        if !privacy_provider.verify_private_transaction(private_tx)? {
            result.error = Some("Private transaction verification failed".to_string());
            return Ok(result);
        }

        // Additional gas for privacy features
        result.gas_used += private_tx.private_inputs.len() as u64 * 1000; // ZK proof verification cost
        result.gas_used += private_tx.private_outputs.len() as u64 * 500;  // Range proof verification cost

        // Process the underlying transaction
        drop(privacy_provider); // Release lock before processing base transaction
        let base_result = self.process_transaction(&private_tx.base_transaction)?;
        
        if !base_result.success {
            result.error = base_result.error;
            return Ok(result);
        }

        // Add privacy-specific events
        for (i, input) in private_tx.private_inputs.iter().enumerate() {
            result.events.push(TransactionEvent {
                address: format!("private_input_{}", i),
                topics: vec!["confidential_spend".to_string()],
                data: format!("Private input with nullifier hash: {}", 
                             hex::encode(&input.validity_proof.nullifier[..8])).into_bytes(),
            });
        }

        for (i, output) in private_tx.private_outputs.iter().enumerate() {
            result.events.push(TransactionEvent {
                address: format!("private_output_{}", i),
                topics: vec!["confidential_output".to_string()],
                data: format!("Private output with commitment: {}", 
                             hex::encode(&output.amount_commitment.commitment[..8])).into_bytes(),
            });
        }

        result.gas_used += base_result.gas_used;
        result.success = true;
        Ok(result)
    }

    /// Create a private transaction from regular inputs
    pub fn create_private_transaction(
        &self,
        base_transaction: Transaction,
        input_amounts: Vec<u64>,
        output_amounts: Vec<u64>,
        secret_keys: Vec<Vec<u8>>,
    ) -> Result<PrivateTransaction> {
        use rand_core::OsRng;
        let mut rng = OsRng;
        
        let mut privacy_provider = self.privacy_provider
            .lock()
            .map_err(|_| failure::format_err!("Failed to acquire privacy provider lock"))?;

        privacy_provider.create_private_transaction(
            base_transaction,
            input_amounts,
            output_amounts,
            secret_keys,
            &mut rng,
        )
    }

    /// Get privacy statistics
    pub fn get_privacy_stats(&self) -> Result<PrivacyStats> {
        let privacy_provider = self.privacy_provider
            .lock()
            .map_err(|_| failure::format_err!("Failed to acquire privacy provider lock"))?;

        Ok(privacy_provider.get_privacy_stats())
    }

    /// Check if privacy features are enabled
    pub fn is_privacy_enabled(&self) -> bool {
        self.config.privacy_config.enable_zk_proofs || 
        self.config.privacy_config.enable_confidential_amounts
    }

    /// Validate a private UTXO for spending
    pub fn validate_private_spending(&self, nullifier: &[u8]) -> Result<bool> {
        let privacy_provider = self.privacy_provider
            .lock()
            .map_err(|_| failure::format_err!("Failed to acquire privacy provider lock"))?;

        Ok(!privacy_provider.is_nullifier_used(nullifier))
    }

    /// Set hybrid account state
    pub fn set_hybrid_account_state(
        &self,
        address: &str,
        state: ProcessorAccountState,
    ) -> Result<()> {
        let mut account_states = self
            .account_states
            .lock()
            .map_err(|_| failure::format_err!("Failed to acquire account states lock"))?;

        account_states.insert(address.to_string(), state);
        Ok(())
    }

    /// Get UTXO set statistics
    pub fn get_utxo_stats(&self) -> Result<UtxoStats> {
        let utxo_set = self
            .utxo_set
            .lock()
            .map_err(|_| failure::format_err!("Failed to acquire UTXO set lock"))?;

        let total_utxos = utxo_set.len();
        let unspent_utxos = utxo_set.values().filter(|utxo| !utxo.is_spent).count();
        let total_value: u64 = utxo_set
            .values()
            .filter(|utxo| !utxo.is_spent)
            .map(|utxo| utxo.output.value as u64)
            .sum();
        let eutxo_count = utxo_set
            .values()
            .filter(|utxo| !utxo.is_spent && utxo.output.is_eUTXO())
            .count();

        Ok(UtxoStats {
            total_utxos,
            unspent_utxos,
            total_value,
            eutxo_count,
        })
    }

    /// Convert address to pub_key_hash for UTXO matching
    fn address_to_pub_key_hash(&self, address: &str) -> Result<Vec<u8>> {
        use bitcoincash_addr::Address;
        use crypto::digest::Digest;
        use crypto::sha2::Sha256;

        use crate::crypto::wallets::extract_encryption_type;

        // Extract base address without encryption suffix
        let (base_address, _) = extract_encryption_type(address)?;

        // Try to decode the address, but handle failure gracefully for modular testing
        match Address::decode(&base_address) {
            Ok(addr) => Ok(addr.body),
            Err(_) => {
                // For modular blockchain testing, use address hash as fallback
                let mut hasher = Sha256::new();
                hasher.input_str(&base_address);
                let hash_bytes = hasher.result_str();
                // Convert hex string to bytes and take first 20 bytes
                match hex::decode(&hash_bytes[..40]) {
                    Ok(hash_vec) => Ok(hash_vec),
                    Err(_) => {
                        // Fallback: use first 20 bytes of address string as bytes
                        let addr_bytes = base_address.as_bytes();
                        let len = addr_bytes.len().min(20);
                        let mut pub_key_hash = addr_bytes[..len].to_vec();
                        // Pad with zeros if needed
                        while pub_key_hash.len() < 20 {
                            pub_key_hash.push(0);
                        }
                        Ok(pub_key_hash)
                    }
                }
            }
        }
    }
}

/// UTXO set statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UtxoStats {
    pub total_utxos: usize,
    pub unspent_utxos: usize,
    pub total_value: u64,
    pub eutxo_count: usize,
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::crypto::transaction::Transaction;

    #[test]
    fn test_eutxo_processor_creation() {
        let config = EUtxoProcessorConfig::default();
        let processor = EUtxoProcessor::new(config);

        let stats = processor.get_utxo_stats().unwrap();
        assert_eq!(stats.total_utxos, 0);
        assert_eq!(stats.unspent_utxos, 0);
    }

    #[test]
    fn test_coinbase_transaction_processing() {
        let config = EUtxoProcessorConfig::default();
        let processor = EUtxoProcessor::new(config);

        // Create a coinbase transaction
        let tx =
            Transaction::new_coinbase("test_address".to_string(), "reward".to_string()).unwrap();

        let result = processor.process_transaction(&tx).unwrap();
        assert!(result.success);
        assert!(result.gas_used > 0);

        let stats = processor.get_utxo_stats().unwrap();
        assert_eq!(stats.unspent_utxos, 1);
    }

    #[test]
    fn test_utxo_balance_calculation() {
        let config = EUtxoProcessorConfig::default();
        let processor = EUtxoProcessor::new(config);

        // Create and process a coinbase transaction
        let tx =
            Transaction::new_coinbase("test_address".to_string(), "reward".to_string()).unwrap();
        processor.process_transaction(&tx).unwrap();

        // Check balance
        let balance = processor.get_balance("test_address").unwrap();
        assert!(balance > 0);
    }

    #[test]
    fn test_privacy_features_enabled() {
        let mut config = EUtxoProcessorConfig::default();
        config.privacy_config.enable_zk_proofs = true;
        config.privacy_config.enable_confidential_amounts = true;
        
        let processor = EUtxoProcessor::new(config);
        assert!(processor.is_privacy_enabled());

        let stats = processor.get_privacy_stats().unwrap();
        assert!(stats.zk_proofs_enabled);
        assert!(stats.confidential_amounts_enabled);
    }

    #[test]
    fn test_private_transaction_creation() {
        let config = EUtxoProcessorConfig::default();
        let processor = EUtxoProcessor::new(config);

        // Create a simple coinbase transaction
        let base_tx = Transaction::new_coinbase("test_address".to_string(), "test_data".to_string()).unwrap();
        
        let input_amounts = vec![0u64];  // Coinbase has 1 input with zero value
        let output_amounts = vec![10u64];  // One output with value 10
        let secret_keys = vec![vec![1, 2, 3]];  // Dummy secret key for coinbase

        let private_tx = processor.create_private_transaction(
            base_tx,
            input_amounts,
            output_amounts,
            secret_keys,
        ).unwrap();

        assert_eq!(private_tx.private_inputs.len(), 1);  // Coinbase has 1 input
        assert_eq!(private_tx.private_outputs.len(), 1);
        assert!(!private_tx.transaction_proof.is_empty());
    }

    #[test]
    fn test_private_transaction_processing() {
        let config = EUtxoProcessorConfig::default();
        let processor = EUtxoProcessor::new(config);

        // Create a simple coinbase transaction
        let base_tx = Transaction::new_coinbase("test_address".to_string(), "test_data".to_string()).unwrap();
        
        let private_tx = processor.create_private_transaction(
            base_tx,
            vec![0u64],      // Coinbase input with zero value
            vec![10u64], // One output
            vec![vec![1, 2, 3]],      // Dummy secret key for coinbase
        ).unwrap();

        let result = processor.process_private_transaction(&private_tx).unwrap();
        assert!(result.success);
        assert!(result.gas_used > 0);
        
        // Should have privacy-specific events
        let privacy_events: Vec<_> = result.events.iter()
            .filter(|e| e.topics.contains(&"confidential_output".to_string()))
            .collect();
        assert!(!privacy_events.is_empty());
    }

    #[test]
    fn test_nullifier_validation() {
        let config = EUtxoProcessorConfig::default();
        let processor = EUtxoProcessor::new(config);

        let test_nullifier = vec![1, 2, 3, 4, 5];
        
        // Initially, nullifier should be valid (not used)
        assert!(processor.validate_private_spending(&test_nullifier).unwrap());
    }
}
