//! Block implement of blockchain

use crate::consensus::proof_of_burn::BurnManager;
use crate::consensus::proof_of_burn::BurnProof;
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
    timestamp: u128,
    transactions: Vec<Transaction>,
    prev_block_hash: String,
    hash: String,
    nonce: i32,
    height: i32,
    difficulty: usize,
    miner_address: String,
    proof_of_burn: Option<BurnProof>,
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

    /// NewBlock creates and returns Block
    pub fn new_block(
        transactions: Vec<Transaction>,
        prev_block_hash: String,
        height: i32,
        difficulty: usize,
        miner_address: String,
    ) -> Result<Block> {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_millis();
        let block = Block {
            timestamp,
            transactions,
            prev_block_hash,
            hash: String::new(),
            nonce: 0,
            height,
            difficulty,
            miner_address,
            proof_of_burn: None,
        };

        // block.run_proof_of_work()?;
        Ok(block)
    }

    /// NewGenesisBlock creates and returns genesis Block
    pub fn new_genesis_block(coinbase: Transaction, miner_address: String) -> Block {
        Block::new_block(vec![coinbase], String::new(), 0, INITIAL_DIFFICULTY, miner_address).unwrap()
    }

    /// Run performs a proof-of-work
    pub fn run_proof_of_burn(&mut self, burn_manager: &BurnManager) -> Result<()> {
        info!("Mining the block with proof of burn ... nonce: {}", self.nonce);

        let mut proof = burn_manager.create_burn_proof(&self.miner_address, self.height)?;

        let data = self.prepare_hash_data()?;

        while !burn_manager.verify_burn_proof(&proof, self.difficulty, &data)? {
            proof.nonce += 1;

            proof.timestamp = std::time::SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)?
                .as_millis();
        }

        let mut hasher = Sha256::new();
        hasher.input(&data);
        hasher.input(&serialize(&proof)?);
        self.hash = hasher.result_str();
        self.proof_of_burn = Some(proof);

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

struct MergeVu8 {}

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
