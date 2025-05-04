#![feature(test)]
extern crate test;

use std::sync::Arc;
use polytorus::blockchain::blockchain::Blockchain;
use polytorus::blockchain::utxoset::UTXOSet;
use polytorus::crypto::fndsa::FnDsaCrypto;
use polytorus::crypto::ecdsa::EcdsaCrypto;
use polytorus::crypto::traits::CryptoProvider;
use polytorus::crypto::wallets::{Wallets, Wallet};
use polytorus::crypto::types::EncryptionType;
use polytorus::crypto::transaction::Transaction;

#[derive(Clone)]
pub struct BenchmarkSetup {
    pub blockchain: Blockchain,
    pub utxo_set: UTXOSet,
    pub wallets: Wallets,
    pub addresses: Vec<String>,
    pub fndsa_crypto: Arc<FnDsaCrypto>,
    pub ecdsa_crypto: Arc<EcdsaCrypto>,
}

impl BenchmarkSetup {
    pub fn new(wallet_count: usize) -> Self {
        std::fs::remove_dir_all("data/blocks").ok();
        std::fs::remove_dir_all("data/utxos").ok();
        std::fs::remove_dir_all("data/wallets").ok();
        let mut wallets = Wallets::new().unwrap();
        
        
        let genesis_address = wallets.create_wallet(EncryptionType::FNDSA);
        wallets.save_all().unwrap();
        
        
        let blockchain = Blockchain::create_blockchain(genesis_address.clone()).unwrap();
        let mut utxo_set = UTXOSet { blockchain };
        
        
        let mut addresses = vec![genesis_address.clone()];
        
        
        for _ in 0..wallet_count {
            let address = wallets.create_wallet(EncryptionType::FNDSA);
            addresses.push(address);
        }
        wallets.save_all().unwrap();
        
        Self::distribute_funds(&mut utxo_set, &wallets, &addresses).unwrap();
        
        BenchmarkSetup {
            blockchain: utxo_set.blockchain.clone(),
            utxo_set,
            wallets,
            addresses,
            fndsa_crypto: Arc::new(FnDsaCrypto),
            ecdsa_crypto: Arc::new(EcdsaCrypto),
        }
    }
    
    fn distribute_funds(
        utxo_set: &mut UTXOSet,
        wallets: &Wallets,
        addresses: &[String],
    ) -> Result<(), Box<dyn std::error::Error>> {
        let genesis_addr = &addresses[0];
        let genesis_wallet = wallets.get_wallet(genesis_addr).unwrap();
        let crypto = FnDsaCrypto;
        
        for addr in addresses.iter().skip(1) {
            let tx = Transaction::new_UTXO(
                genesis_wallet,
                addr,
                1000,
                utxo_set,
                &crypto
            )?;
            
            let cbtx = Transaction::new_coinbase(
                genesis_addr.clone(),
                "initial distribution".to_string()
            )?;
            
            let block = utxo_set.blockchain.mine_block(vec![cbtx, tx])?;
            utxo_set.update(&block)?;
        }
        
        Ok(())
    }
}
