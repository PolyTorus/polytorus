use crate::blockchain::block::Block;
use crate::crypto::transaction::Transaction;
use crate::Result;
use std::collections::HashMap;

pub trait StorateProvider: Send + Sync {
    fn get(&self, key: &[u8]) -> Result<Option<Vec<u8>>>;
    fn put(&self, key: &[u8], value: &[u8]) -> Result<()>;
    fn delete(&self, key: &[u8]) -> Result<()>;
    fn has(&self, key: &[u8]) -> Result<bool>;
    fn iterate<F>(&self, f: F) -> Result<()>
    where
        F: FnMut(&[u8], &[u8]) -> Result<bool>;

    fn flush(&self) -> Result<()>;
}

pub trait ConsensusEngine: Send + Sync {
    fn validate_block(&self, block: &Block) -> Result<bool>;
    fn prepare_block(&self, transactions: Vec<Transaction>, prev_hash: String, height: i32) -> Result<Block>;
    fn adjust_difficulty(&self, prev_block: &Block, current_timestamp: u128) -> usize;
    fn get_name(&self) -> String;
}

pub trait TransactionValidator: Send + Sync {
    fn validate_transaction(&self, tx: &Transaction, prev_txs: HashMap<String, Transaction>) -> Result<bool>;
}

pub trait ChainState: Send + Sync {
    fn get_best_height(&self) -> Result<i32>;
    fn get_block(&self, hash: &str) -> Result<Block>;
    fn add_block(&mut self, block: Block) -> Result<()>;
    fn get_blocks_iterator(&self) -> Box<dyn Iterator<Item = Block> + '_>;
    fn mine_block(&mut self, transactions: Vec<Transaction>) -> Result<Block>;
    fn find_transaction(&self, id: &str) -> Result<Transaction>;
}