use crate::config::WalletConfig;
use crate::Result;
use bincode::{deserialize, serialize};
use bitcoincash_addr::*;
use crypto::digest::Digest;
use crypto::ripemd160::Ripemd160;
use crypto::sha2::Sha256;
use serde::{Deserialize, Serialize};
use sled;
use std::collections::HashMap;
use super::traits::CryptoProvider;
use super::types::*;
use crate::types::Address;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct Wallet {
    pub secret_key: PrivateKey,
    pub public_key: PublicKey,
}

impl Wallet {
    /// NewWallet creates and returns a Wallet
    fn new(crypto_provider: &dyn CryptoProvider, encryption: CryptoType) -> Result<Self> {
        let (secret_key, public_key) = crypto_provider.gen_keypair(encryption)?;

        Ok(Self {
            secret_key,
            public_key,
        })
    }

    /// GetAddress returns wallet address
    pub fn get_address(&self) -> Address {
        let mut pub_hash = self.public_key.data.clone();
        hash_pub_key(&mut pub_hash);
        let address = bitcoincash_addr::Address {
            body: pub_hash,
            scheme: Scheme::Base58,
            hash_type: HashType::Script,
            ..Default::default()
        };
        Address(address.encode().unwrap())
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

pub struct Wallets {
    wallets: HashMap<Address, Wallet>,
    config: WalletConfig,
}

impl Wallets {
    /// NewWallets creates Wallets and fills it from a file if it exists
    pub fn new(config: WalletConfig) -> Result<Wallets> {
        let mut wlt = Wallets {
            wallets: HashMap::<Address, Wallet>::new(),
            config,
        };
        let db = sled::open("data/wallets")?;

        for item in db.into_iter() {
            let i = item?;
            let address = Address(String::from_utf8(i.0.to_vec())?);
            let wallet = deserialize(&i.1)?;
            wlt.wallets.insert(address, wallet);
        }
        drop(db);
        Ok(wlt)
    }

    /// CreateWallet adds a Wallet to Wallets
    pub fn create_wallet(&mut self, crypto_provider: &dyn CryptoProvider, encryption: CryptoType) -> Result<Address> {
        let wallet = Wallet::new(crypto_provider, encryption)?;
        let address = wallet.get_address();
        self.wallets.insert(address.clone(), wallet);
        info!("create wallet: {}", address);
        Ok(address)
    }

    /// GetAddresses returns an array of addresses stored in the wallet file
    pub fn get_all_addresses(&self) -> Vec<Address> {
        self.wallets.keys().cloned().collect()
    }

    /// GetWallet returns a Wallet by its address
    pub fn get_wallet(&self, address: &Address) -> Option<&Wallet> {
        self.wallets.get(address)
    }

    /// SaveToFile saves wallets to a file
    pub fn save_all(&self) -> Result<()> {
        let db = sled::open(&self.config.data_dir)?;

        for (address, wallet) in &self.wallets {
            let data = serialize(wallet)?;
            db.insert(address.as_str().as_bytes(), data)?;
        }

        db.flush()?;
        drop(db);
        Ok(())
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::crypto::fndsa::FnDsaCrypto;

    #[test]
    fn test_create_wallet_and_address() -> Result<()> {
        
        let crypto_provider = FnDsaCrypto;
        let wallet = Wallet::new(&crypto_provider, CryptoType::FNDSA)?;
        let address = wallet.get_address();

        assert!(!address.is_empty());
        
        let mut pub_key_hash = wallet.public_key.data.clone();
        hash_pub_key(&mut pub_key_hash);
        
        let expected_address = bitcoincash_addr::Address {
            body: pub_key_hash,
            scheme: Scheme::Base58,
            hash_type: HashType::Script,
            ..Default::default()
        }.encode().unwrap();
        
        assert_eq!(address.as_str(), &expected_address);

        Ok(())
    }

    #[test]
    fn test_wallets_management() -> Result<()> {
        let config = WalletConfig {
            data_dir: String::from("data/test/wallets"),
            default_key_type: CryptoType::FNDSA,
        };
        
        std::fs::create_dir_all(&config.data_dir).ok();

        let crypto = FnDsaCrypto;
        let mut wallets = Wallets::new(config.clone())?;

        let address1 = wallets.create_wallet(&crypto, CryptoType::FNDSA)?;
        let address2 = wallets.create_wallet(&crypto, CryptoType::FNDSA)?;

        wallets.save_all()?;

        let wallets2 = Wallets::new(config)?;

        assert!(wallets2.get_wallet((&address1)).is_some());
        assert!(wallets2.get_wallet((&address2)).is_some());

        let addresses = wallets2.get_all_addresses();
        assert!(addresses.contains(&address1));
        assert!(addresses.contains(&address2));

        std::fs::remove_dir_all("data/test").ok();

        Ok(())
    }
}
