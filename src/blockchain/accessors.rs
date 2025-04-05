use crate::{blockchain::block::Block, crypto::transaction::Transaction};

pub fn hash(block: &Block) -> String {
    block.get_hash()
}

pub fn prev_hash(block: &Block) -> String {
    block.get_prev_hash()
}

pub fn transactions(block: &Block) -> &Vec<Transaction> {
    block.get_transaction()
}

pub fn height(block: &Block) -> i32 {
    block.get_height()
}

pub fn difficulty(block: &Block) -> usize {
    block.get_difficulty()
}

pub fn timestamp(block: &Block) -> u128 {
    block.get_timestamp()
}
