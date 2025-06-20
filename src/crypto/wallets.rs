use std::collections::HashMap;

use bincode::{deserialize, serialize};
use bitcoincash_addr::*;
use fn_dsa::{
    sign_key_size, vrfy_key_size, KeyPairGenerator, KeyPairGeneratorStandard, FN_DSA_LOGN_512,
};
use ripemd::Ripemd160;
use secp256k1::{rand::rngs::OsRng, Secp256k1};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use sled;

use super::types::*;
use crate::{config::DataContext, Result};

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Wallet {
    pub secret_key: Vec<u8>,
    pub public_key: Vec<u8>,
    pub encryption_type: EncryptionType,
}

/// NewWallet creates and returns a Wallet
impl Wallet {
    fn new(encryption: EncryptionType) -> Self {
        match encryption {
            EncryptionType::FNDSA => {
                let mut kg = KeyPairGeneratorStandard::default();
                let mut sign_key = [0u8; sign_key_size(FN_DSA_LOGN_512)];
                let mut vrfy_key = [0u8; vrfy_key_size(FN_DSA_LOGN_512)];
                kg.keygen(FN_DSA_LOGN_512, &mut OsRng, &mut sign_key, &mut vrfy_key);

                Wallet {
                    secret_key: sign_key.to_vec(),
                    public_key: vrfy_key.to_vec(),
                    encryption_type: EncryptionType::FNDSA,
                }
            }
            EncryptionType::ECDSA => {
                let secp = Secp256k1::new();
                let (secret_key, public_key) = secp.generate_keypair(&mut OsRng);

                Wallet {
                    secret_key: secret_key.secret_bytes().to_vec(),
                    public_key: public_key.serialize().to_vec(),
                    encryption_type: EncryptionType::ECDSA,
                }
            }
        }
    }
    /// GetAddress returns wallet address
    pub fn get_address(&self) -> String {
        let mut pub_hash: Vec<u8> = self.public_key.clone();
        hash_pub_key(&mut pub_hash);
        let address = Address {
            body: pub_hash,
            scheme: Scheme::Base58,
            hash_type: HashType::Script,
            ..Default::default()
        };
        let base_address = address.encode().unwrap();

        // Append encryption type to the end
        let encryption_suffix = match self.encryption_type {
            EncryptionType::ECDSA => "-ECDSA",
            EncryptionType::FNDSA => "-FNDSA",
        };

        format!("{}{}", base_address, encryption_suffix)
    }

    /// Create a wallet with a specific address (for genesis)
    pub fn new_with_address(address: String) -> Self {
        // Create a simple wallet for genesis purposes
        Wallet {
            secret_key: vec![0; 32], // Genesis wallets don't need real keys
            public_key: address.as_bytes().to_vec(),
            encryption_type: EncryptionType::FNDSA,
        }
    }
}

impl Default for Wallet {
    fn default() -> Self {
        Wallet::new(EncryptionType::FNDSA)
    }
}

/// HashPubKey hashes public key
pub fn hash_pub_key(pubKey: &mut Vec<u8>) {
    let mut hasher1 = Sha256::new();
    hasher1.update(&*pubKey);
    let sha256_result = hasher1.finalize();

    let mut hasher2 = Ripemd160::new();
    hasher2.update(sha256_result);
    let ripemd_result = hasher2.finalize();

    pubKey.clear();
    pubKey.extend_from_slice(&ripemd_result[..]);
}

/// Extract encryption type from address
pub fn extract_encryption_type(address: &str) -> Result<(String, EncryptionType)> {
    if address.ends_with("-ECDSA") {
        let base_address = address.strip_suffix("-ECDSA").unwrap().to_string();
        Ok((base_address, EncryptionType::ECDSA))
    } else if address.ends_with("-FNDSA") {
        let base_address = address.strip_suffix("-FNDSA").unwrap().to_string();
        Ok((base_address, EncryptionType::FNDSA))
    } else {
        // Use FNDSA by default for backward compatibility
        Ok((address.to_string(), EncryptionType::FNDSA))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Wallets {
    wallets: HashMap<String, Wallet>,
    #[serde(skip)]
    context: DataContext,
}

impl Wallets {
    pub fn new() -> Result<Wallets> {
        Self::new_with_context(DataContext::default())
    }

    /// NewWallets creates Wallets and fills it from a file if it exists
    pub fn new_with_context(context: DataContext) -> Result<Wallets> {
        let mut wlt = Wallets {
            wallets: HashMap::<String, Wallet>::new(),
            context: context.clone(),
        };
        let db = sled::open(context.wallets_dir())?;

        for item in db.into_iter() {
            let i = item?;
            let address = String::from_utf8(i.0.to_vec())?;
            let wallet = deserialize(&i.1)?;
            wlt.wallets.insert(address, wallet);
        }
        drop(db);
        Ok(wlt)
    }

    /// CreateWallet adds a Wallet to Wallets
    pub fn create_wallet(&mut self, encryption: EncryptionType) -> String {
        let wallet = Wallet::new(encryption);
        let address = wallet.get_address();
        self.wallets.insert(address.clone(), wallet);
        info!("create wallet: {}", address);
        address
    }

    /// GetAddresses returns an array of addresses stored in the wallet file
    pub fn get_all_addresses(&self) -> Vec<String> {
        let mut addresses = Vec::<String>::new();
        for address in self.wallets.keys() {
            addresses.push(address.clone());
        }
        addresses
    }

    /// GetWallet returns a Wallet by its address
    pub fn get_wallet(&self, address: &str) -> Option<&Wallet> {
        self.wallets.get(address)
    }

    /// SaveToFile saves wallets to a file
    pub fn save_all(&self) -> Result<()> {
        let db = sled::open(self.context.wallets_dir())?;

        for (address, wallet) in &self.wallets {
            let data = serialize(wallet)?;
            db.insert(address, data)?;
        }

        db.flush()?;
        drop(db);
        Ok(())
    }
}

/// Modern wallet manager for testnet
use std::sync::{Arc, RwLock};

#[derive(Clone)]
pub struct WalletManager {
    wallets: Arc<RwLock<HashMap<String, Wallet>>>,
}

impl WalletManager {
    pub fn new() -> Self {
        Self {
            wallets: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn add_wallet(&self, address: String, wallet: Wallet) -> Result<()> {
        let mut wallets = self.wallets.write().unwrap();
        wallets.insert(address, wallet);
        Ok(())
    }

    pub async fn get_wallet(&self, address: &str) -> Option<Wallet> {
        let wallets = self.wallets.read().unwrap();
        wallets.get(address).cloned()
    }

    pub async fn list_addresses(&self) -> Vec<String> {
        let wallets = self.wallets.read().unwrap();
        wallets.keys().cloned().collect()
    }

    pub async fn create_wallet(&self, encryption_type: EncryptionType) -> Result<String> {
        let wallet = Wallet::new(encryption_type);
        let address = wallet.get_address();

        {
            let mut wallets = self.wallets.write().unwrap();
            wallets.insert(address.clone(), wallet);
        }

        Ok(address)
    }
}

impl Default for WalletManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod test {
    use fn_dsa::{
        signature_size, SigningKey, SigningKeyStandard, VerifyingKey, VerifyingKeyStandard,
        DOMAIN_NONE, HASH_ID_RAW,
    };

    use super::*;
    use crate::test_helpers::{cleanup_test_context, create_test_context, TestContextGuard};

    #[test]
    fn test_create_wallet_and_hash() {
        let w1 = Wallet::default();
        let w2 = Wallet::default();
        assert_ne!(w1, w2);
        assert_ne!(w1.get_address(), w2.get_address());
        let mut p2 = w2.public_key.clone();
        hash_pub_key(&mut p2);
        assert_eq!(p2.len(), 20);
        let (base_address, _) = extract_encryption_type(&w2.get_address()).unwrap();
        let pub_key_hash = Address::decode(&base_address).unwrap().body;
        assert_eq!(pub_key_hash, p2);
    }

    #[test]
    fn test_wallets() {
        let context = create_test_context();
        let _guard = TestContextGuard::new(context.clone());

        let mut ws = Wallets::new_with_context(context.clone()).unwrap();
        let wa1 = ws.create_wallet(EncryptionType::FNDSA);
        let w1 = ws.get_wallet(&wa1).unwrap().clone();
        ws.save_all().unwrap();

        let ws2 = Wallets::new_with_context(context.clone()).unwrap();
        let w2 = ws2.get_wallet(&wa1).unwrap();
        assert_eq!(&w1, w2);

        cleanup_test_context(&context.clone());
    }

    #[test]
    #[should_panic]
    fn test_wallets_not_exist() {
        let context = create_test_context();
        let _guard = TestContextGuard::new(context.clone());

        let w3 = Wallet::default();
        let ws2 = Wallets::new_with_context(context.clone()).unwrap();
        ws2.get_wallet(&w3.get_address()).unwrap();

        cleanup_test_context(&context.clone());
    }

    #[test]
    fn test_signature() {
        let w = Wallet::default();
        let mut sk = SigningKeyStandard::decode(&w.secret_key).unwrap();
        let mut sig = vec![0u8; signature_size(sk.get_logn())];
        sk.sign(&mut OsRng, &DOMAIN_NONE, &HASH_ID_RAW, b"message", &mut sig);

        match VerifyingKeyStandard::decode(&w.public_key) {
            Some(vk) => {
                assert!(vk.verify(&sig, &DOMAIN_NONE, &HASH_ID_RAW, b"message"));
            }
            None => {
                panic!("failed to decode verifying key");
            }
        }
    }
}
