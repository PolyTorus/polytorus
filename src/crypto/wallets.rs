use super::types::*;
use crate::{config, Result};
use bincode::{deserialize, serialize};
use bitcoincash_addr::*;
use crypto::digest::Digest;
use crypto::ripemd160::Ripemd160;
use crypto::sha2::Sha256;
use fn_dsa::{
    sign_key_size, vrfy_key_size, KeyPairGenerator, KeyPairGeneratorStandard, FN_DSA_LOGN_512,
};
use secp256k1::rand::rngs::OsRng;
use secp256k1::Secp256k1;
use serde::{Deserialize, Serialize};
use sled;
use std::collections::HashMap;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Wallet {
    pub secret_key: Vec<u8>,
    pub public_key: Vec<u8>,
}

impl Wallet {
    /// NewWallet creates and returns a Wallet
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
                }
            }
            EncryptionType::ECDSA => {
                let secp = Secp256k1::new();
                let (secret_key, public_key) = secp.generate_keypair(&mut OsRng);

                Wallet {
                    secret_key: secret_key.secret_bytes().to_vec(),
                    public_key: public_key.serialize().to_vec(),
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
        address.encode().unwrap()
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
    hasher1.input(pubKey);
    hasher1.result(pubKey);
    let mut hasher2 = Ripemd160::new();
    hasher2.input(pubKey);
    pubKey.resize(20, 0);
    hasher2.result(pubKey);
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Wallets {
    wallets: HashMap<String, Wallet>,
}

impl Wallets {
    /// NewWallets creates Wallets and fills it from a file if it exists
    pub fn new() -> Result<Wallets> {
        let mut wlt = Wallets {
            wallets: HashMap::<String, Wallet>::new(),
        };
        let config = config::get_config();
        let wallets_path = config.wallets_path();
        let db = sled::open(wallets_path)?;

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
        let config = config::get_config();
        let wallets_path = config.wallets_path();
        let db = sled::open(wallets_path)?;

        for (address, wallet) in &self.wallets {
            let data = serialize(wallet)?;
            db.insert(address, data)?;
        }

        db.flush()?;
        drop(db);
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::config::{self, Config};
    use fn_dsa::{
        signature_size, SigningKey, SigningKeyStandard, VerifyingKey, VerifyingKeyStandard,
        DOMAIN_NONE, HASH_ID_RAW,
    };
    
    // テスト実行前に設定を初期化
    fn setup_test() -> Config {
        let test_config = Config::new_test_config();
        config::init_config(test_config.clone());
        test_config
    }
    
    // テスト終了時にディレクトリをクリーンアップ
    struct TestCleanup {
        config: Config,
    }
    
    impl Drop for TestCleanup {
        fn drop(&mut self) {
            self.config.cleanup_test_dir();
        }
    }
    
    #[test]
    fn test_create_wallet_and_hash() {
        let _config = setup_test();
        
        let w1 = Wallet::default();
        let w2 = Wallet::default();
        assert_ne!(w1, w2);
        assert_ne!(w1.get_address(), w2.get_address());

        let mut p2 = w2.public_key.clone();
        hash_pub_key(&mut p2);
        assert_eq!(p2.len(), 20);
        let pub_key_hash = Address::decode(&w2.get_address()).unwrap().body;
        assert_eq!(pub_key_hash, p2);
    }
}
