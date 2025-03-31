use crate::types::{self, Address, TransactionId};
use crate::errors::{BlockchainError, Result};
use crate::blockchain::utxoset::UTXOSet;
use crate::crypto::traits::CryptoProvider;
use crate::crypto::types::{CryptoType, PrivateKey, PublicKey, Signature};
use crate::crypto::wallets::{hash_pub_key, Wallet};
use bincode::serialize_into;
use crypto::digest::Digest;
use crypto::sha2::Sha256;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

const SUBSIDY: i32 = 10;

/// TXInput represents a transaction input
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TXInput {
    pub txid: TransactionId,
    pub vout: i32,
    pub signature: Signature,
    pub pub_key: PublicKey,
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

/// Transaction represents a Bitcoin transaction
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Transaction {
    pub id: TransactionId,
    pub vin: Vec<TXInput>,
    pub vout: Vec<TXOutput>,
}

impl Transaction {
    /// NewUTXOTransaction creates a new transaction
    pub fn new_UTXO(
        wallet: &Wallet,
        to: &Address,
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

        let mut pub_key_hash = wallet.public_key.data.clone();
        hash_pub_key(&mut pub_key_hash);

        let acc_v = utxo.find_spendable_outputs(&pub_key_hash, amount)?;

        if acc_v.0 < amount {
            error!("Not Enough balance");
            return Err(BlockchainError::Other(format!(
                "Not Enough balance: current balance {}",
                acc_v.0
            )));
        }

        for tx in acc_v.1 {
            for out in tx.1 {
                let input = TXInput {
                    txid: types::TransactionId(tx.0.clone()),
                    vout: out,
                    signature: Signature::new(wallet.public_key.key_type.clone(), Vec::new()),
                    pub_key: wallet.public_key.clone(),
                };
                vin.push(input);
            }
        }

        let mut vout = vec![TXOutput::new(amount, to.to_string())?];
        if acc_v.0 > amount {
            vout.push(TXOutput::new(acc_v.0 - amount, wallet.get_address().to_string())?)
        }

        let mut tx = Transaction {
            id: TransactionId::empty(),
            vin,
            vout,
        };
        tx.id = types::TransactionId(tx.hash()?);
        utxo.blockchain
            .sign_transacton(&mut tx, &wallet.secret_key.data, crypto)?;
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
            id: TransactionId::empty(),
            vin: vec![TXInput {
                txid: TransactionId::empty(),
                vout: -1,
                signature: Signature::new(CryptoType::FNDSA, Vec::new()),
                pub_key: PublicKey::new(CryptoType::FNDSA, pub_key),
            }],
            vout: vec![TXOutput::new(SUBSIDY, to)?],
        };
        tx.id = types::TransactionId(tx.hash()?);
        Ok(tx)
    }

    /// IsCoinbase checks whether the transaction is coinbase
    pub fn is_coinbase(&self) -> bool {
        self.vin.len() == 1 && self.vin[0].txid.is_empty() && self.vin[0].vout == -1
    }

    /// Verify verifies signatures of Transaction inputs
    pub fn verify(&self, prev_TXs: HashMap<TransactionId, Transaction>) -> Result<bool> {
        if self.is_coinbase() {
            return Ok(true);
        }

        for vin in &self.vin {
            if prev_TXs.get(&vin.txid).unwrap().id.is_empty() {
                return Err(BlockchainError::Other(String::from("ERROR: Previous transaction is not correct")));
            }
        }

        let mut tx_copy = self.trim_copy();

        for in_id in 0..self.vin.len() {
            let prev_Tx = prev_TXs.get(&self.vin[in_id].txid).unwrap();
            tx_copy.vin[in_id].signature = Signature::new(self.vin[in_id].signature.key_type.clone(), Vec::new());
            tx_copy.vin[in_id].pub_key = PublicKey::new(
                self.vin[in_id].pub_key.key_type.clone(),
                prev_Tx.vout[self.vin[in_id].vout as usize].pub_key_hash.clone(),
            );
            tx_copy.id = types::TransactionId(tx_copy.hash()?);
            tx_copy.vin[in_id].pub_key = PublicKey::new(self.vin[in_id].pub_key.key_type.clone(), Vec::new());

            match self.vin[in_id].pub_key.key_type {
                CryptoType::FNDSA => {
                    use fn_dsa::{VerifyingKey, VerifyingKeyStandard, DOMAIN_NONE, HASH_ID_RAW};

                    if !VerifyingKeyStandard::decode(&self.vin[in_id].pub_key.data)
                        .ok_or(BlockchainError::InvalidSignature("Invalid public key".to_string()))?
                        .verify(
                            &self.vin[in_id].signature.data,
                            &DOMAIN_NONE,
                            &HASH_ID_RAW,
                            tx_copy.id.as_str().as_bytes(),
                        )
                        {
                            return Ok(false);
                        }
                },
                CryptoType::ECDSA => {
                    use secp256k1::{Message, PublicKey, Secp256k1, ecdsa::Signature};
                    
                    let secp = Secp256k1::verification_only();
                    let pk = PublicKey::from_slice(&self.vin[in_id].pub_key.data)
                        .map_err(|e| BlockchainError::InvalidSignature(e.to_string()))?;
                    let msg = Message::from_slice(tx_copy.id.as_str().as_bytes())
                        .map_err(|e| BlockchainError::InvalidSignature(e.to_string()))?;
                    let sig = Signature::from_compact(&self.vin[in_id].signature.data)
                        .map_err(|e| BlockchainError::InvalidSignature(e.to_string()))?;
                    
                    if !secp.verify_ecdsa(&msg, &sig, &pk).is_ok() {
                        return Ok(false);
                    }
                },
            }
        }

        Ok(true)
    }

    /// Sign signs each input of a Transaction
    pub fn sign(
        &mut self,
        private_key: &PrivateKey,
        prev_TXs: HashMap<TransactionId, Transaction>,
        crypto: &dyn CryptoProvider,
    ) -> Result<()> {
        if self.is_coinbase() {
            return Ok(());
        }

        for vin in &self.vin {
            if prev_TXs.get(&vin.txid).unwrap().id.is_empty() {
                return Err(BlockchainError::Other(String::from("ERROR: Previous transaction is not correct")));
            }
        }

        let mut tx_copy = self.trim_copy();

        for in_id in 0..tx_copy.vin.len() {
            let prev_Tx = prev_TXs.get(&tx_copy.vin[in_id].txid).unwrap();
            tx_copy.vin[in_id].signature = Signature::new(private_key.key_type.clone(), Vec::new());
            tx_copy.vin[in_id].pub_key = PublicKey::new(
                private_key.key_type.clone(),
                prev_Tx.vout[tx_copy.vin[in_id].vout as usize].pub_key_hash.clone(),
            );
            tx_copy.id = types::TransactionId(tx_copy.hash()?);
            tx_copy.vin[in_id].pub_key = PublicKey::new(private_key.key_type.clone(), Vec::new());
            
            let signature = crypto.sign(private_key, tx_copy.id.as_str().as_bytes())?;
            self.vin[in_id].signature = signature;
        }

        Ok(())
    }

    /// Hash returns the hash of the Transaction
    #[inline]
    pub fn hash(&self) -> Result<String> {
        let mut buf = Vec::new();
        serialize_into(&mut buf, &self.vin)?;
        serialize_into(&mut buf, &self.vout)?;
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
                signature: Signature::new(v.signature.key_type.clone(), Vec::new()),
                pub_key: PublicKey::new(v.pub_key.key_type.clone(), Vec::new()),
            })
        }

        for v in &self.vout {
            vout.push(TXOutput {
                value: v.value,
                pub_key_hash: v.pub_key_hash.clone(),
            })
        }

        Transaction {
            id: TransactionId::empty(),
            vin,
            vout,
        }
    }
}

impl TXOutput {
    /// IsLockedWithKey checks if the output can be used by the owner of the pubkey
    pub fn is_locked_with_key(&self, pub_key_hash: &[u8]) -> bool {
        self.pub_key_hash == pub_key_hash
    }
    /// Lock signs the output
    fn lock(&mut self, address: &str) -> Result<()> {
        let pub_key_hash = bitcoincash_addr::Address::decode(address).unwrap().body;
        debug!("lock: {}", address);
        self.pub_key_hash = pub_key_hash;
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
    use core::hash;

    use super::*;
    use crate::config::WalletConfig;
    use crate::crypto::types::CryptoType;
    use crate::crypto::fndsa::FnDsaCrypto;

    #[test]
    fn test_create_wallet_and_address() -> Result<()> {
        let config = WalletConfig {
            data_dir: "data/test/wallets".to_string(),
            default_key_type: CryptoType::FNDSA,
        };

        let crypto = FnDsaCrypto;
        let wallet = Wallet::new(&crypto, CryptoType::FNDSA)?;
        let address = wallet.get_address();

        assert!(!address.is_empty());

        let mut pub_key_hash = wallet.public_key.data.clone();
        hash_pub_key(&mut pub_key_hash);

        let expected_address = bitcoincash_addr::Address {
            body: pub_key_hash,
            scheme: bitcoincash_addr::Scheme::Base58,
            hash_type: bitcoincash_addr::HashType::Script,
            ..Default::default()
        }.encode().unwrap();

        assert_eq!(address.as_str(), &expected_address);

        Ok(())
    }
}
