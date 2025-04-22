//! Blockchain

use crate::blockchain::block::*;
use crate::consensus::proof_of_burn::{BurnInfo, BurnManager};
use crate::crypto::traits::CryptoProvider;
use crate::crypto::transaction::*;
use crate::crypto::wallets::hash_pub_key;
use crate::Result;
use bincode::{deserialize, serialize};
use bitcoincash_addr::{Address, HashType, Scheme};
use crypto::digest::Digest;
use crypto::sha2::Sha256;
use failure::format_err;
use sled;
use std::collections::HashMap;
use std::time::SystemTime;

const GENESIS_COINBASE_DATA: &str =
    "The Times 03/Jan/2009 Chancellor on brink of second bailout for banks";

/// Blockchain implements interactions with a DB
#[derive(Debug)]
pub struct Blockchain {
    pub tip: String,
    pub db: sled::Db,
    pub mining_address: String,
}

/// BlockchainIterator is used to iterate over blockchain blocks
pub struct BlockchainIterator<'a> {
    current_hash: String,
    bc: &'a Blockchain,
}

impl Blockchain {
    /// NewBlockchain creates a new Blockchain db
    pub fn new() -> Result<Blockchain> {
        info!("open blockchain");

        let db = sled::open("data/blocks")?;
        let hash = match db.get("LAST")? {
            Some(l) => l.to_vec(),
            None => Vec::new(),
        };
        info!("Found block database");
        let lasthash = if hash.is_empty() {
            String::new()
        } else {
            String::from_utf8(hash.to_vec())?
        };

        let mining_address = match db.get("MINING_ADDRESS")? {
            Some(a) => String::from_utf8(a.to_vec())?,
            None => String::new(),
        };

        Ok(Blockchain { tip: lasthash, db, mining_address })
    }

    /// CreateBlockchain creates a new blockchain DB
    pub fn create_blockchain(address: String) -> Result<Blockchain> {
        info!("Creating new blockchain");

        std::fs::remove_dir_all("data/blocks").ok();
        let db = sled::open("data/blocks")?;
        debug!("Creating new block database");
        let cbtx = Transaction::new_coinbase(address.clone(), String::from(GENESIS_COINBASE_DATA))?;
        let genesis: Block = Block::new_genesis_block(cbtx, address.clone());
        db.insert(genesis.get_hash(), serialize(&genesis)?)?;
        db.insert("LAST", genesis.get_hash().as_bytes())?;
        db.insert("MINING_ADDRESS", address.as_bytes())?;

        let bc = Blockchain {
            tip: genesis.get_hash(),
            db,
            mining_address: address,
        };
        bc.db.flush()?;
        Ok(bc)
    }

    /// MineBlock mines a new block with the provided transactions
    pub fn mine_block(&mut self, transactions: Vec<Transaction>) -> Result<Block> {
        info!("mine a new block");

        for tx in &transactions {
            if !self.verify_transacton(tx)? {
                return Err(format_err!("ERROR: Invalid transaction"));
            }
        }

        let lasthash = self.db.get("LAST")?.unwrap();
        let prev_hash = String::from_utf8(lasthash.to_vec())?;
        let prev_block = self.get_block(&prev_hash)?;
        let current_timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_millis();
        let new_difficulty = Block::adjust_difficulty(&prev_block, current_timestamp);

        let newblock = Block::new_block(
            transactions,
            prev_hash,
            self.get_best_height()? + 1,
            new_difficulty,
            self.mining_address.clone(),
        )?;
        self.db.insert(newblock.get_hash(), serialize(&newblock)?)?;
        self.db.insert("LAST", newblock.get_hash().as_bytes())?;
        self.db.flush()?;

        self.tip = newblock.get_hash();
        Ok(newblock)
    }

    pub fn mine_block_with_pob(&mut self, transactions: Vec<Transaction>, burn_manager: &BurnManager) -> Result<Block> {
        info!("mine a new block with proof of burn");

        for tx in &transactions {
            if !self.verify_transacton(tx)? {
                return Err(format_err!("ERROR: Invalid transaction"));
            }
        }

        let lasthash = self.db.get("LAST")?.unwrap();
        let prev_hash = String::from_utf8(lasthash.to_vec())?;
        let prev_block = self.get_block(&prev_hash)?;
        let current_timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_millis();
        let new_difficulty = Block::adjust_difficulty(&prev_block, current_timestamp);


        let mut newblock = Block::new_block(
            transactions, 
            prev_hash, 
            self.get_best_height()? + 1, 
            new_difficulty,
            self.mining_address.clone()
        )?;

        newblock.run_proof_of_burn(burn_manager)?;

        self.db.insert(newblock.get_hash(), serialize(&newblock)?)?;
        self.db.insert("LAST", newblock.get_hash().as_bytes())?;
        self.db.flush()?;

        self.tip = newblock.get_hash();

        Ok(newblock)
    }

    /// Iterator returns a BlockchainIterat
    pub fn iter(&self) -> BlockchainIterator {
        BlockchainIterator {
            current_hash: self.tip.clone(),
            bc: self,
        }
    }

    /// FindUTXO finds and returns all unspent transaction outputs
    pub fn find_UTXO(&self) -> HashMap<String, TXOutputs> {
        let mut utxos: HashMap<String, TXOutputs> = HashMap::new();
        let mut spend_txos: HashMap<String, Vec<i32>> = HashMap::new();

        for block in self.iter() {
            for tx in block.get_transaction() {
                for index in 0..tx.vout.len() {
                    if let Some(ids) = spend_txos.get(&tx.id) {
                        if ids.contains(&(index as i32)) {
                            continue;
                        }
                    }

                    match utxos.get_mut(&tx.id) {
                        Some(v) => {
                            v.outputs.push(tx.vout[index].clone());
                        }
                        None => {
                            utxos.insert(
                                tx.id.clone(),
                                TXOutputs {
                                    outputs: vec![tx.vout[index].clone()],
                                },
                            );
                        }
                    }
                }

                if !tx.is_coinbase() {
                    for i in &tx.vin {
                        match spend_txos.get_mut(&i.txid) {
                            Some(v) => {
                                v.push(i.vout);
                            }
                            None => {
                                spend_txos.insert(i.txid.clone(), vec![i.vout]);
                            }
                        }
                    }
                }
            }
        }

        utxos
    }

    /// FindTransaction finds a transaction by its ID
    pub fn find_transacton(&self, id: &str) -> Result<Transaction> {
        for b in self.iter() {
            for tx in b.get_transaction() {
                if tx.id == id {
                    return Ok(tx.clone());
                }
            }
        }
        Err(format_err!("Transaction is not found"))
    }

    fn get_prev_TXs(&self, tx: &Transaction) -> Result<HashMap<String, Transaction>> {
        let mut prev_TXs = HashMap::new();
        for vin in &tx.vin {
            let prev_TX = self.find_transacton(&vin.txid)?;
            prev_TXs.insert(prev_TX.id.clone(), prev_TX);
        }
        Ok(prev_TXs)
    }

    /// SignTransaction signs inputs of a Transaction
    pub fn sign_transacton(
        &self,
        tx: &mut Transaction,
        private_key: &[u8],
        crypto: &dyn CryptoProvider,
    ) -> Result<()> {
        let prev_TXs = self.get_prev_TXs(tx)?;
        tx.sign(private_key, prev_TXs, crypto)?;
        Ok(())
    }

    /// VerifyTransaction verifies transaction input signatures
    pub fn verify_transacton(&self, tx: &Transaction) -> Result<bool> {
        if tx.is_coinbase() {
            return Ok(true);
        }
        let prev_TXs = self.get_prev_TXs(tx)?;
        tx.verify(prev_TXs)
    }

    /// AddBlock saves the block into the blockchain
    pub fn add_block(&mut self, block: Block) -> Result<()> {
        let data = serialize(&block)?;
        if let Some(_) = self.db.get(block.get_hash())? {
            return Ok(());
        }
        self.db.insert(block.get_hash(), data)?;

        let lastheight = self.get_best_height()?;
        if block.get_height() > lastheight {
            self.db.insert("LAST", block.get_hash().as_bytes())?;
            self.tip = block.get_hash();
            self.db.flush()?;
        }
        Ok(())
    }

    // GetBlock finds a block by its hash and returns it
    pub fn get_block(&self, block_hash: &str) -> Result<Block> {
        let data = self.db.get(block_hash)?.unwrap();
        let block = deserialize(&data)?;
        Ok(block)
    }

    /// GetBestHeight returns the height of the latest block
    pub fn get_best_height(&self) -> Result<i32> {
        let lasthash = if let Some(h) = self.db.get("LAST")? {
            h
        } else {
            return Ok(-1);
        };
        let last_data = self.db.get(lasthash)?.unwrap();
        let last_block: Block = deserialize(&last_data)?;
        Ok(last_block.get_height())
    }

    /// GetBlockHashes returns a list of hashes of all the blocks in the chain
    pub fn get_block_hashs(&self) -> Vec<String> {
        let mut list = Vec::new();
        for b in self.iter() {
            list.push(b.get_hash());
        }
        list
    }

    pub fn initial_burn_manager(&self) -> Result<BurnManager> {
        let mut burn_manager = BurnManager::new();
        
        for b in self.iter() {
            for tx in b.get_transaction() {
                if self.is_burn_transaction(tx)? {
                    if let Some(burn_info) = self.extract_burn_info(tx, b.get_height())? {
                        burn_manager.register_burn(burn_info);
                    }
                }
            }
        }

        Ok(burn_manager)
    }

    fn is_burn_transaction(&self, tx: &Transaction) -> Result<bool> {
        if tx.is_coinbase() {
            return Ok(false);
        }
        
        if tx.vin.is_empty() {
            return Ok(false);
        }
        
        let burn_manager = BurnManager::new();
        
        for output in &tx.vout {
            let output_addr = Address {
                body: output.pub_key_hash.clone(),
                scheme: Scheme::Base58,
                hash_type: HashType::Script,
                ..Default::default()
            };
            
            let output_addr_str = match output_addr.encode() {
                Ok(s) => s,
                Err(_) => continue, 
            };
            
            let pub_key = &tx.vin[0].pub_key;
            let mut pub_key_hash = pub_key.clone();
            hash_pub_key(&mut pub_key_hash);
            
            let sender_addr = Address {
                body: pub_key_hash,
                scheme: Scheme::Base58,
                hash_type: HashType::Script,
                ..Default::default()
            };
            
            let sender_addr_str = match sender_addr.encode() {
                Ok(s) => s,
                Err(_) => continue,
            };
            
            if burn_manager.verify_burn_address(&output_addr_str, &sender_addr_str) {
                if output_addr_str.starts_with("BURN") {
                    return Ok(true);
                }
                
                let mut hasher = Sha256::new();
                hasher.input_str(&format!("{}{}", crate::consensus::proof_of_burn::PREFIX, sender_addr_str));
                let mut hash = hasher.result_str();
                
                let last_char = hash.chars().last().unwrap();
                let flipped_char = if last_char == '0' { '1' } else { '0' };
                hash.pop();
                hash.push(flipped_char);
                
                if hash == output_addr_str {
                    return Ok(true);
                }
            }
        }

        Ok(false)
    }

    fn extract_burn_info(&self, tx: &Transaction, block_height: i32) -> Result<Option<BurnInfo>> {
        if tx.is_coinbase() {
            return Ok(None);
        }

        if tx.vin.is_empty() {
            return Ok(None);
        }

        let burn_manager = BurnManager::new();

        for output in &tx.vout {
            let output_addr = Address {
                body: output.pub_key_hash.clone(),
                scheme: Scheme::Base58,
                hash_type: HashType::Script,
                ..Default::default()
            };

            let output_addr_str = match output_addr.encode() {
                Ok(s) => s,
                Err(_) => continue,
            };

            let pub_key = &tx.vin[0].pub_key;
            let mut pub_key_hash = pub_key.clone();
            hash_pub_key(&mut pub_key_hash);

            let sender_addr = Address {
                body: pub_key_hash,
                scheme: Scheme::Base58,
                hash_type: HashType::Script,
                ..Default::default()
            };

            let sender_addr_str = match sender_addr.encode() {
                Ok(s) => s,
                Err(_) => continue,
            };

            if burn_manager.verify_burn_address(&output_addr_str, &sender_addr_str) {
                return Ok(Some(BurnInfo {
                    address: sender_addr_str,
                    amount: output.value,
                    block_height,
                    burn_txid: tx.id.clone(),
                }));
            }
        }

        Ok(None)
    }
    
}

impl Iterator for BlockchainIterator<'_> {
    type Item = Block;

    fn next(&mut self) -> Option<Self::Item> {
        if let Ok(encoded_block) = self.bc.db.get(&self.current_hash) {
            return match encoded_block {
                Some(b) => {
                    if let Ok(block) = deserialize::<Block>(&b) {
                        self.current_hash = block.get_prev_hash();
                        Some(block)
                    } else {
                        None
                    }
                }
                None => None,
            };
        }
        None
    }
}
