use crate::blockchain::block::*;
use crate::blockchain::blockchain::*;
use crate::blockchain::types::{network, NetworkConfig};
use crate::crypto::transaction::*;
use crate::Result;
use bincode::{deserialize, serialize};
use sled;
use std::collections::HashMap;

/// Type-safe UTXO set
pub struct UTXOSet<N: NetworkConfig = network::Mainnet> {
    pub blockchain: Blockchain<N>,
}

impl<N: NetworkConfig> UTXOSet<N> {
    /// FindUnspentTransactions returns a list of transactions containing unspent outputs
    pub fn find_spendable_outputs(
        &self,
        pub_key_hash: &[u8],
        amount: i32,
    ) -> Result<(i32, HashMap<String, Vec<i32>>)> {
        let mut unspent_outputs: HashMap<String, Vec<i32>> = HashMap::new();
        let mut accumulated = 0;

        let db = sled::open(self.blockchain.context.utxos_dir())?;
        for kv in db.iter() {
            let (k, v) = kv?;
            let txid = String::from_utf8(k.to_vec())?;
            let outs: TXOutputs = deserialize(&v)?;

            for out_idx in 0..outs.outputs.len() {
                if outs.outputs[out_idx].is_locked_with_key(pub_key_hash) && accumulated < amount {
                    accumulated += outs.outputs[out_idx].value;
                    match unspent_outputs.get_mut(&txid) {
                        Some(v) => v.push(out_idx as i32),
                        None => {
                            unspent_outputs.insert(txid.clone(), vec![out_idx as i32]);
                        }
                    }
                }
            }
        }

        Ok((accumulated, unspent_outputs))
    }

    /// FindUTXO finds UTXO for a public key hash
    pub fn find_UTXO(&self, pub_key_hash: &[u8]) -> Result<TXOutputs> {
        let mut utxos = TXOutputs {
            outputs: Vec::new(),
        };
        let db = sled::open(self.blockchain.context.utxos_dir())?;

        for kv in db.iter() {
            let (_, v) = kv?;
            let outs: TXOutputs = deserialize(&v)?;

            for out in outs.outputs {
                if out.is_locked_with_key(pub_key_hash) {
                    utxos.outputs.push(out.clone())
                }
            }
        }

        Ok(utxos)
    }

    /// CountTransactions returns the number of transactions in the UTXO set
    pub fn count_transactions(&self) -> Result<i32> {
        let mut counter = 0;
        let db = sled::open(self.blockchain.context.utxos_dir())?;
        for kv in db.iter() {
            kv?;
            counter += 1;
        }
        Ok(counter)
    }

    /// Reindex rebuilds the UTXO set
    pub fn reindex(&self) -> Result<()> {
        let db_path = self.blockchain.context.utxos_dir();
        std::fs::remove_dir_all(&db_path).ok();
        let db = sled::open(db_path)?;

        let utxos = self.blockchain.find_UTXO();

        for (txid, outs) in utxos {
            db.insert(txid.as_bytes(), serialize(&outs)?)?;
        }

        Ok(())
    }
    /// Update updates the UTXO set with transactions from the Block
    ///
    /// The Block is considered to be the tip of a blockchain
    pub fn update(&self, block: &FinalizedBlock<N>) -> Result<()> {
        let db = sled::open(self.blockchain.context.utxos_dir())?;

        for tx in block.get_transactions() {
            if !tx.is_coinbase() {
                for vin in &tx.vin {
                    let mut update_outputs = TXOutputs {
                        outputs: Vec::new(),
                    };
                    let outs: TXOutputs = deserialize(&db.get(&vin.txid)?.unwrap())?;
                    for out_idx in 0..outs.outputs.len() {
                        if out_idx != vin.vout as usize {
                            update_outputs.outputs.push(outs.outputs[out_idx].clone());
                        }
                    }

                    if update_outputs.outputs.is_empty() {
                        db.remove(&vin.txid)?;
                    } else {
                        db.insert(vin.txid.as_bytes(), serialize(&update_outputs)?)?;
                    }
                }
            }

            let mut new_outputs = TXOutputs {
                outputs: Vec::new(),
            };
            for out in &tx.vout {
                new_outputs.outputs.push(out.clone());
            }

            db.insert(tx.id.as_bytes(), serialize(&new_outputs)?)?;
        }
        Ok(())
    }

    /// Validate eUTXO spending conditions for a transaction
    pub fn validate_eUTXO_spending(&self, tx: &Transaction) -> Result<bool> {
        // Skip validation for coinbase transactions
        if tx.is_coinbase() {
            return Ok(true);
        }

        let db = sled::open(self.blockchain.context.utxos_dir())?;

        // Validate each input against its corresponding output
        for input in &tx.vin {
            // Get the outputs for this transaction ID
            let outputs_data = db.get(&input.txid)?;
            if outputs_data.is_none() {
                return Ok(false); // Referenced transaction not found
            }

            let outputs: TXOutputs = deserialize(&outputs_data.unwrap())?;

            // Check if the output index is valid
            if input.vout < 0 || input.vout as usize >= outputs.outputs.len() {
                return Ok(false); // Invalid output index
            }

            let output = &outputs.outputs[input.vout as usize];

            // Validate eUTXO spending conditions
            if !output.validate_spending(input)? {
                return Ok(false); // Spending validation failed
            }
        }

        Ok(true)
    }

    /// Find eUTXO outputs with specific script conditions
    pub fn find_eUTXO_outputs(&self, pub_key_hash: &[u8]) -> Result<Vec<(String, i32, TXOutput)>> {
        let mut eUTXO_outputs = Vec::new();
        let db = sled::open(self.blockchain.context.utxos_dir())?;

        for kv in db.iter() {
            let (k, v) = kv?;
            let txid = String::from_utf8(k.to_vec())?;
            let outs: TXOutputs = deserialize(&v)?;

            for (out_idx, output) in outs.outputs.iter().enumerate() {
                if output.is_locked_with_key(pub_key_hash) && output.is_eUTXO() {
                    eUTXO_outputs.push((txid.clone(), out_idx as i32, output.clone()));
                }
            }
        }

        Ok(eUTXO_outputs)
    }
}
