//! Blockchain

use crate::blockchain::block::*;
use crate::crypto::traits::CryptoProvider;
use crate::crypto::transaction::*;
use crate::Result;
use bincode::{deserialize, serialize};
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
    pub width: usize,
    pub height: usize,
    pub map: HashMap<(usize, usize), String>,
}

/// BlockchainIterator is used to iterate over blockchain blocks
#[derive(Debug)]
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

        let width =  match db.get("WIDTH")? {
            Some(w) => deserialize(&w)?,
            None => 3,
        };

        let height = match db.get("HEIGHT")? {
            Some(h) => deserialize(&h)?,
            None => 3,
        };

        let map = match db.get("MAP")? {
            Some(m) => deserialize(&m)?,
            None => HashMap::new(),
        };


        Ok(Blockchain { tip: lasthash, db, width, height, map })
    }

    /// CreateBlockchain creates a new blockchain DB
    pub fn create_blockchain(address: String, width: usize, height: usize) -> Result<Blockchain> {
        info!("Creating new blockchain");

        if width == 0 || height == 0 {
            return Err(format_err!("width and height must be greater than 0"));
        }


        std::fs::remove_dir_all("data/blocks").ok();
        let db = sled::open("data/blocks")?;
        debug!("Creating new block database");

        let cbtx = Transaction::new_coinbase(address, String::from(GENESIS_COINBASE_DATA))?;
        let genesis: Block = Block::new_genesis_block(cbtx);

        let mut map: HashMap<(usize, usize), String> = HashMap::new();
        map.insert((0, 0), genesis.get_hash());

        db.insert(genesis.get_hash(), serialize(&genesis)?)?;
        db.insert("LAST", genesis.get_hash().as_bytes())?;
        db.insert("WIDTH", serialize(&width)?)?;
        db.insert("HEIGHT", serialize(&height)?)?;
        db.insert("MAP", serialize(&map)?)?;

        let bc = Blockchain {
            tip: genesis.get_hash(),
            db,
            width,
            height,
            map,
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

        let (current_x, current_y) = prev_block.get_coordinates();
        let next_x = (current_x + 1) % self.width;
        let next_y = if next_x == 0 { (current_y + 1) % self.height } else { current_y };

        let parallel_hash = self.get_block_at_coordinates((next_x, (current_y + self.height - 1) % self.height)).unwrap_or_else(|_| String::new());
        let cross_hash = self.get_block_at_coordinates(((current_x + self.width - 1) % self.width,
                                                        (current_y + self.height - 1) % self.height))
            .unwrap_or_else(|_| String::new());

        let current_timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_millis();
        let new_difficulty = Block::adjust_difficulty(&prev_block, current_timestamp);

        let newblock = Block::new_block(
            transactions,
            prev_hash,
            parallel_hash,
            cross_hash,
            self.get_best_height()? + 1,
            new_difficulty,
            next_x,
            next_y,
        )?;

        self.map.insert((next_x, next_y), newblock.get_hash());
        self.db.insert("MAP", serialize(&self.map)?)?;

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
            bc: &self,
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

            let (x, y) = block.get_coordinates();
            self.map.insert((x, y), block.get_hash());
            self.db.insert("MAP", serialize(&self.map)?)?;
            self.db.flush()?;
        }
        Ok(())
    }

    // GetBlock finds a block by its hash and returns it
    pub fn get_block(&self, block_hash: &str) -> Result<Block> {
        let data = self.db.get(block_hash)?.unwrap();
        let block = deserialize(&data.to_vec())?;
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
        let last_block: Block = deserialize(&last_data.to_vec())?;
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

    pub fn get_block_at_coordinates(&self, coords: (usize, usize)) -> Result<String> {
        match self.map.get(&coords) {
            Some(hash) => Ok(hash.clone()),
            None => Err(format_err!("No block at coords {:?}", coords)),
        }
    }

    pub fn visualize(&self) -> Result<String> {
        let mut output = String::new();
        output.push_str(&format!("Torus Blockchain Structure ({}x{})\n", self.width, self.height));
        output.push_str("-----------------------------\n");

        for y in 0..self.height {
            for x in 0..self.width {
                match self.map.get(&(x, y)) {
                    Some(hash) => {
                        let short_hash = &hash[0..6]; // ハッシュの最初の6文字だけ表示
                        output.push_str(&format!("[{}] ", short_hash));
                    },
                    None => {
                        output.push_str("[     ] ");
                    }
                }
            }
            output.push_str("\n");
        }

        Ok(output)
    }

}

impl<'a> Iterator for BlockchainIterator<'a> {
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
