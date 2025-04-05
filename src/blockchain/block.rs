//! Block implement of blockchain

use crate::crypto::transaction::*;
use crate::Result;
use bincode::serialize;
use crypto::digest::Digest;
use crypto::sha2::Sha256;
use merkle_cbt::merkle_tree::Merge;
use merkle_cbt::merkle_tree::CBMT;
use serde::{Deserialize, Serialize};
use std::time::SystemTime;

const INITIAL_DIFFICULTY: usize = 4;
const DESIRED_BLOCK_TIME: u128 = 10_000;
/// Block keeps block headers
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Block {
    pub(crate) timestamp: u128,
    pub(crate) transactions: Vec<Transaction>,
    pub(crate) prev_block_hash: String,
    pub(crate) hash: String,
    pub(crate) nonce: i32,
    pub(crate) height: i32,
    pub(crate) difficulty: usize,
}

impl Block {
    pub fn get_hash(&self) -> String {
        self.hash.clone()
    }

    pub fn get_prev_hash(&self) -> String {
        self.prev_block_hash.clone()
    }

    pub fn get_transaction(&self) -> &Vec<Transaction> {
        &self.transactions
    }

    pub fn get_height(&self) -> i32 {
        self.height
    }

    pub fn get_difficulty(&self) -> usize {
        self.difficulty
    }

    pub fn get_timestamp(&self) -> u128 {
        self.timestamp
    }

    /// NewBlock creates and returns Block
    pub fn new_block(
        transactions: Vec<Transaction>,
        prev_block_hash: String,
        height: i32,
        difficulty: usize,
    ) -> Result<Block> {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_millis();
        let mut block = Block {
            timestamp,
            transactions,
            prev_block_hash,
            hash: String::new(),
            nonce: 0,
            height,
            difficulty,
        };
        block.run_proof_of_work()?;
        Ok(block)
    }

    /// NewGenesisBlock creates and returns genesis Block
    pub fn new_genesis_block(coinbase: Transaction) -> Block {
        Block::new_block(vec![coinbase], String::new(), 0, INITIAL_DIFFICULTY).unwrap()
    }

    /// Run performs a proof-of-work
    fn run_proof_of_work(&mut self) -> Result<()> {
        info!("Mining the block");
        while !self.validate()? {
            self.nonce += 1;
        }
        let data = self.prepare_hash_data()?;
        let mut hasher = Sha256::new();
        hasher.input(&data[..]);
        self.hash = hasher.result_str();
        Ok(())
    }

    /// HashTransactions returns a hash of the transactions in the block
    fn hash_transactions(&self) -> Result<Vec<u8>> {
        let mut transactions = Vec::new();
        for tx in &self.transactions {
            transactions.push(tx.hash()?.as_bytes().to_owned());
        }
        let tree = CBMT::<Vec<u8>, MergeVu8>::build_merkle_tree(transactions);

        Ok(tree.root())
    }

    fn prepare_hash_data(&self) -> Result<Vec<u8>> {
        let content = (
            self.prev_block_hash.clone(),
            self.hash_transactions()?,
            self.timestamp,
            self.difficulty,
            self.nonce,
        );
        let bytes = serialize(&content)?;
        Ok(bytes)
    }

    /// Validate validates block's PoW
    fn validate(&self) -> Result<bool> {
        let data = self.prepare_hash_data()?;
        let mut hasher = Sha256::new();
        hasher.input(&data[..]);
        let hash_str = hasher.result_str();
        let prefix = "0".repeat(self.difficulty);
        Ok(hash_str.starts_with(&prefix))
    }

    pub fn adjust_difficulty(prev_block: &Block, current_timestamp: u128) -> usize {
        let time_diff = current_timestamp - prev_block.timestamp;
        let mut new_difficulty = prev_block.difficulty;

        if time_diff < DESIRED_BLOCK_TIME {
            new_difficulty += 1;
        } else if time_diff > DESIRED_BLOCK_TIME && new_difficulty > 1 {
            new_difficulty -= 1;
        }

        new_difficulty
    }
}

pub struct MergeVu8 {}

impl Merge for MergeVu8 {
    type Item = Vec<u8>;
    fn merge(left: &Self::Item, right: &Self::Item) -> Self::Item {
        let mut hasher = Sha256::new();
        let mut data: Vec<u8> = left.clone();
        data.append(&mut right.clone());
        hasher.input(&data);
        let mut re: [u8; 32] = [0; 32];
        hasher.result(&mut re);
        re.to_vec()
    }
}
