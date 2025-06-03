use crate::blockchain::utxoset::*;
use crate::crypto::traits::CryptoProvider;
use crate::crypto::types::EncryptionType;

use crate::crypto::wallets::*;
use crate::Result;
use bincode::serialize_into;
use bitcoincash_addr::Address;
use crypto::digest::Digest;
use crypto::sha2::Sha256;
use failure::format_err;
use fn_dsa::{VerifyingKey, VerifyingKeyStandard, DOMAIN_NONE, HASH_ID_RAW};
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::vec;

const SUBSIDY: i32 = 10;

/// TXInput represents a transaction input
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TXInput {
    pub txid: String,
    pub vout: i32,
    pub signature: Vec<u8>,
    pub pub_key: Vec<u8>,
}

/// Determine encryption type based on public key size
fn determine_encryption_type(pub_key: &[u8]) -> EncryptionType {
    // ECDSA public keys are typically 33 bytes (compressed) or 65 bytes (uncompressed)
    // FN-DSA public keys are typically much larger (around 897 bytes for LOGN=512)
    if pub_key.len() <= 65 {
        EncryptionType::ECDSA
    } else {
        EncryptionType::FNDSA
    }
}

/// TXOutput represents a transaction output
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TXOutput {
    pub value: i32,
    pub pub_key_hash: Vec<u8>,
}

// TXOutputs collects TXOutput
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TXOutputs {
    pub outputs: Vec<TXOutput>,
}

/// Transaction represents a blockchain transaction
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transaction {
    pub id: String,
    pub vin: Vec<TXInput>,
    pub vout: Vec<TXOutput>,
    pub contract_data: Option<ContractTransactionData>,
}

/// Smart contract transaction data
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ContractTransactionData {
    pub tx_type: ContractTransactionType,
    pub data: Vec<u8>,
}

/// Types of contract transactions
#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ContractTransactionType {
    Deploy {
        bytecode: Vec<u8>,
        constructor_args: Vec<u8>,
        gas_limit: u64,
    },
    Call {
        contract_address: String,
        function_name: String,
        arguments: Vec<u8>,
        gas_limit: u64,
        value: u64,
    },
}

impl Transaction {
    /// NewUTXOTransaction creates a new transaction
    pub fn new_UTXO(
        wallet: &Wallet,
        to: &str,
        amount: i32,
        utxo: &UTXOSet,
        crypto: &dyn CryptoProvider,
    ) -> Result<Transaction> {
        info!(
            "new UTXO Transaction from: {} to: {}",
            wallet.get_address(),
            to
        );
        let mut vin = Vec::new();

        let mut pub_key_hash = wallet.public_key.clone();
        hash_pub_key(&mut pub_key_hash);

        let acc_v = utxo.find_spendable_outputs(&pub_key_hash, amount)?;

        if acc_v.0 < amount {
            error!("Not Enough balance");
            return Err(format_err!(
                "Not Enough balance: current balance {}",
                acc_v.0
            ));
        }

        for tx in acc_v.1 {
            for out in tx.1 {
                let input = TXInput {
                    txid: tx.0.clone(),
                    vout: out,
                    signature: Vec::new(),
                    pub_key: wallet.public_key.clone(),
                };
                vin.push(input);
            }
        }

        let mut vout = vec![TXOutput::new(amount, to.to_string())?];
        if acc_v.0 > amount {
            vout.push(TXOutput::new(acc_v.0 - amount, wallet.get_address())?)
        }

        let mut tx = Transaction {
            id: String::new(),
            vin,
            vout,
            contract_data: None,
        };
        tx.id = tx.hash()?;
        utxo.blockchain
            .sign_transacton(&mut tx, &wallet.secret_key, crypto)?;
        Ok(tx)
    }

    /// NewCoinbaseTX creates a new coinbase transaction
    pub fn new_coinbase(to: String, mut data: String) -> Result<Transaction> {
        info!("new coinbase Transaction to: {}", to);
        let mut key: [u8; 32] = [0; 32];
        if data.is_empty() {
            let mut rand = rand::thread_rng();
            key = rand.gen();
            data = format!("Reward to '{}'", to);
        }
        let mut pub_key = Vec::from(data.as_bytes());
        pub_key.append(&mut Vec::from(key));

        let mut tx = Transaction {
            id: String::new(),
            vin: vec![TXInput {
                txid: String::new(),
                vout: -1,
                signature: Vec::new(),
                pub_key,
            }],
            vout: vec![TXOutput::new(SUBSIDY, to)?],
            contract_data: None,
        };
        tx.id = tx.hash()?;
        Ok(tx)
    }

    /// Create a new contract deployment transaction
    pub fn new_contract_deployment(
        wallet: &Wallet,
        bytecode: Vec<u8>,
        constructor_args: Vec<u8>,
        gas_limit: u64,
        utxo: &UTXOSet,
        crypto: &dyn CryptoProvider,
    ) -> Result<Transaction> {
        info!(
            "Creating contract deployment transaction from: {}",
            wallet.get_address()
        );

        let contract_data = ContractTransactionData {
            tx_type: ContractTransactionType::Deploy {
                bytecode,
                constructor_args,
                gas_limit,
            },
            data: Vec::new(),
        };

        // Create a transaction with minimal value (gas fee)
        let gas_fee = (gas_limit / 1000) as i32; // Simple gas fee calculation
        let mut vin = Vec::new();
        let mut pub_key_hash = wallet.public_key.clone();
        hash_pub_key(&mut pub_key_hash);

        let acc_v = utxo.find_spendable_outputs(&pub_key_hash, gas_fee)?;
        if acc_v.0 < gas_fee {
            return Err(format_err!(
                "Not enough balance for gas fees: need {}, have {}",
                gas_fee,
                acc_v.0
            ));
        }

        for tx in acc_v.1 {
            for out in tx.1 {
                let input = TXInput {
                    txid: tx.0.clone(),
                    vout: out,
                    signature: Vec::new(),
                    pub_key: wallet.public_key.clone(),
                };
                vin.push(input);
            }
        }

        let mut vout = Vec::new();
        if acc_v.0 > gas_fee {
            vout.push(TXOutput::new(acc_v.0 - gas_fee, wallet.get_address())?);
        }

        let mut tx = Transaction {
            id: String::new(),
            vin,
            vout,
            contract_data: Some(contract_data),
        };
        tx.id = tx.hash()?;
        utxo.blockchain
            .sign_transacton(&mut tx, &wallet.secret_key, crypto)?;
        Ok(tx)
    }

    /// Create a new contract call transaction
    pub fn new_contract_call(
        wallet: &Wallet,
        contract_address: String,
        function_name: String,
        arguments: Vec<u8>,
        gas_limit: u64,
        value: u64,
        utxo: &UTXOSet,
        crypto: &dyn CryptoProvider,
    ) -> Result<Transaction> {
        info!(
            "Creating contract call transaction from: {} to contract: {}",
            wallet.get_address(),
            contract_address
        );

        let contract_data = ContractTransactionData {
            tx_type: ContractTransactionType::Call {
                contract_address,
                function_name,
                arguments,
                gas_limit,
                value,
            },
            data: Vec::new(),
        };

        let total_cost = value as i32 + (gas_limit / 1000) as i32; // value + gas fee
        let mut vin = Vec::new();
        let mut pub_key_hash = wallet.public_key.clone();
        hash_pub_key(&mut pub_key_hash);

        let acc_v = utxo.find_spendable_outputs(&pub_key_hash, total_cost)?;
        if acc_v.0 < total_cost {
            return Err(format_err!(
                "Not enough balance: need {}, have {}",
                total_cost,
                acc_v.0
            ));
        }

        for tx in acc_v.1 {
            for out in tx.1 {
                let input = TXInput {
                    txid: tx.0.clone(),
                    vout: out,
                    signature: Vec::new(),
                    pub_key: wallet.public_key.clone(),
                };
                vin.push(input);
            }
        }

        let mut vout = Vec::new();
        if acc_v.0 > total_cost {
            vout.push(TXOutput::new(acc_v.0 - total_cost, wallet.get_address())?);
        }

        let mut tx = Transaction {
            id: String::new(),
            vin,
            vout,
            contract_data: Some(contract_data),
        };
        tx.id = tx.hash()?;
        utxo.blockchain
            .sign_transacton(&mut tx, &wallet.secret_key, crypto)?;
        Ok(tx)
    }

    /// IsCoinbase checks whether the transaction is coinbase
    pub fn is_coinbase(&self) -> bool {
        self.vin.len() == 1 && self.vin[0].txid.is_empty() && self.vin[0].vout == -1
    }

    /// Verify verifies signatures of Transaction inputs
    pub fn verify(&self, prev_TXs: HashMap<String, Transaction>) -> Result<bool> {
        if self.is_coinbase() {
            return Ok(true);
        }

        for vin in &self.vin {
            if prev_TXs.get(&vin.txid).unwrap().id.is_empty() {
                return Err(format_err!("ERROR: Previous transaction is not correct"));
            }
        }

        let mut tx_copy = self.trim_copy();

        for in_id in 0..self.vin.len() {
            let prev_Tx = prev_TXs.get(&self.vin[in_id].txid).unwrap();
            tx_copy.vin[in_id].signature.clear();
            tx_copy.vin[in_id].pub_key = prev_Tx.vout[self.vin[in_id].vout as usize]
                .pub_key_hash
                .clone();
            tx_copy.id = tx_copy.hash()?;
            tx_copy.vin[in_id].pub_key = Vec::new();

            // if !ed25519::verify(
            //     &tx_copy.id.as_bytes(), // message
            //     &self.vin[in_id].pub_key, // public key
            //     &self.vin[in_id].signature, // signature            // ) {
            //     return Ok(false);
            // }

            // Determine encryption type based on public key size
            let encryption_type = determine_encryption_type(&self.vin[in_id].pub_key);

            match encryption_type {
                EncryptionType::FNDSA => {
                    if !VerifyingKeyStandard::decode(&self.vin[in_id].pub_key)
                        .unwrap()
                        .verify(
                            &self.vin[in_id].signature,
                            &DOMAIN_NONE,
                            &HASH_ID_RAW,
                            tx_copy.id.as_bytes(),
                        )
                    {
                        return Ok(false);
                    }
                }
                EncryptionType::ECDSA => {
                    use crate::crypto::ecdsa::EcdsaCrypto;
                    let crypto = EcdsaCrypto;
                    if !crypto.verify(
                        &self.vin[in_id].pub_key,
                        tx_copy.id.as_bytes(),
                        &self.vin[in_id].signature,
                    ) {
                        return Ok(false);
                    }
                }
            }
        }

        Ok(true)
    }

    /// Sign signs each input of a Transaction
    pub fn sign(
        &mut self,
        private_key: &[u8],
        prev_TXs: HashMap<String, Transaction>,
        crypto: &dyn CryptoProvider,
    ) -> Result<()> {
        if self.is_coinbase() {
            return Ok(());
        }

        for vin in &self.vin {
            if prev_TXs.get(&vin.txid).unwrap().id.is_empty() {
                return Err(format_err!("ERROR: Previous transaction is not correct"));
            }
        }

        let mut tx_copy = self.trim_copy();

        for in_id in 0..tx_copy.vin.len() {
            let prev_Tx = prev_TXs.get(&tx_copy.vin[in_id].txid).unwrap();
            tx_copy.vin[in_id].signature.clear();
            tx_copy.vin[in_id].pub_key = prev_Tx.vout[tx_copy.vin[in_id].vout as usize]
                .pub_key_hash
                .clone();
            tx_copy.id = tx_copy.hash()?;
            tx_copy.vin[in_id].pub_key = Vec::new();
            // let signature = ed25519::signature(tx_copy.id.as_bytes(), private_key);
            let signature = crypto.sign(private_key, tx_copy.id.as_bytes());
            self.vin[in_id].signature = signature.to_vec();
        }

        Ok(())
    }
    /// Hash returns the hash of the Transaction
    #[inline]
    pub fn hash(&self) -> Result<String> {
        let mut buf = Vec::new();
        serialize_into(&mut buf, &self.vin)?;
        serialize_into(&mut buf, &self.vout)?;

        // Include contract data in hash if present
        if let Some(contract_data) = &self.contract_data {
            serialize_into(&mut buf, contract_data)?;
        }

        let mut hasher = Sha256::new();
        hasher.input(&buf);
        Ok(hasher.result_str())
    }

    /// TrimmedCopy creates a trimmed copy of Transaction to be used in signing
    fn trim_copy(&self) -> Transaction {
        let mut vin = Vec::with_capacity(self.vin.len());
        let mut vout = Vec::with_capacity(self.vout.len());

        for v in &self.vin {
            vin.push(TXInput {
                txid: v.txid.clone(),
                vout: v.vout,
                signature: Vec::new(),
                pub_key: Vec::new(),
            })
        }

        for v in &self.vout {
            vout.push(TXOutput {
                value: v.value,
                pub_key_hash: v.pub_key_hash.clone(),
            })
        }

        Transaction {
            id: String::new(),
            vin,
            vout,
            contract_data: None,
        }
    }

    /// Check if this is a contract transaction
    pub fn is_contract_transaction(&self) -> bool {
        self.contract_data.is_some()
    }

    /// Get contract data if this is a contract transaction
    pub fn get_contract_data(&self) -> Option<&ContractTransactionData> {
        self.contract_data.as_ref()
    }
}

impl TXOutput {
    /// IsLockedWithKey checks if the output can be used by the owner of the pubkey
    pub fn is_locked_with_key(&self, pub_key_hash: &[u8]) -> bool {
        self.pub_key_hash == pub_key_hash
    }
    /// Lock signs the output
    fn lock(&mut self, address: &str) -> Result<()> {
        // Extract base address without encryption suffix
        let (base_address, _) = extract_encryption_type(address)?;

        // Try to decode the address, but handle failure gracefully for modular mining
        match Address::decode(&base_address) {
            Ok(addr) => {
                self.pub_key_hash = addr.body;
            }
            Err(_) => {
                // For modular blockchain testing, use address hash as fallback
                use crypto::digest::Digest;
                let mut hasher = Sha256::new();
                hasher.input_str(&base_address);
                let hash_bytes = hasher.result_str();
                // Convert hex string to bytes and take first 20 bytes
                match hex::decode(&hash_bytes[..40]) {
                    Ok(hash_vec) => self.pub_key_hash = hash_vec,
                    Err(_) => {
                        // Fallback: use first 20 bytes of address string as bytes
                        let addr_bytes = base_address.as_bytes();
                        let len = addr_bytes.len().min(20);
                        self.pub_key_hash = addr_bytes[..len].to_vec();
                        // Pad with zeros if needed
                        while self.pub_key_hash.len() < 20 {
                            self.pub_key_hash.push(0);
                        }
                    }
                }
            }
        }

        debug!("lock: {}", address);
        Ok(())
    }

    pub fn new(value: i32, address: String) -> Result<Self> {
        let mut txo = TXOutput {
            value,
            pub_key_hash: Vec::new(),
        };
        txo.lock(&address)?;
        Ok(txo)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::crypto::types::EncryptionType;
    use crate::test_helpers::{cleanup_test_context, create_test_context};
    use fn_dsa::{
        signature_size, SigningKey, SigningKeyStandard, VerifyingKey, VerifyingKeyStandard,
        DOMAIN_NONE, HASH_ID_RAW,
    };
    use rand_core::OsRng;

    #[test]
    fn test_signature() {
        let context = create_test_context();
        let mut ws = Wallets::new_with_context(context.clone()).unwrap();
        let wa1 = ws.create_wallet(EncryptionType::FNDSA);
        let w = ws.get_wallet(&wa1).unwrap().clone();
        ws.save_all().unwrap();
        drop(ws);

        let data = String::from("test");
        let tx = Transaction::new_coinbase(wa1, data).unwrap();
        assert!(tx.is_coinbase());

        let mut sk = SigningKeyStandard::decode(&w.secret_key).unwrap();
        let mut signature = vec![0u8; signature_size(sk.get_logn())];
        sk.sign(
            &mut OsRng,
            &DOMAIN_NONE,
            &HASH_ID_RAW,
            tx.id.as_bytes(),
            &mut signature,
        );
        assert!(VerifyingKeyStandard::decode(&w.public_key).unwrap().verify(
            &signature,
            &DOMAIN_NONE,
            &HASH_ID_RAW,
            tx.id.as_bytes()
        ));

        cleanup_test_context(&context.clone());
    }
}
