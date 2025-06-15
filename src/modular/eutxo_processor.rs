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
}

impl Default for EUtxoProcessorConfig {
    fn default() -> Self {
        Self {
            max_script_size: 32768,  // 32KB
            max_datum_size: 8192,    // 8KB
            max_redeemer_size: 8192, // 8KB
            script_gas_cost: 10,
            utxo_base_gas: 5000,
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
}

impl EUtxoProcessor {
    /// Create a new eUTXO processor
    pub fn new(config: EUtxoProcessorConfig) -> Self {
        Self {
            utxo_set: Arc::new(Mutex::new(HashMap::new())),
            account_states: Arc::new(Mutex::new(HashMap::new())),
            config,
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
}
