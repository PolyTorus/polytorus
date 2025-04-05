use std::{rc::Rc, time::SystemTime};
use crate::crypto::transaction::Transaction;
use super::{accessors::BlockResult, block::Block, pow::run_proof_of_work};

fn current_timestamp() -> crate::Result<u128> {
    Ok(
        SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_millis(),
    )
}

pub fn create(
    transactions: Vec<Transaction>,
    prev_block_hash: String,
    height: i32,
    difficulty: usize,
) -> BlockResult<Block> {
    BlockResult(current_timestamp().map_err(|e| e.into()))
            .and_then(|timestamp| {
                // First create a block template
                let tx_ref = Rc::new(transactions);
                let tx_slice = &tx_ref as &Vec<Transaction>;
                
                // Run proof of work
                run_proof_of_work(
                    prev_block_hash.clone(),
                    tx_slice,
                    timestamp,
                    difficulty,
                    super::hash::header,
                )
                .map(|(hash, nonce)| {
                    // Create the final block with the calculated hash and nonce
                    Block {
                        timestamp,
                        transactions: tx_ref.to_vec(),
                        prev_block_hash,
                        hash,
                        nonce,
                        height,
                        difficulty,
                    }
                })
            })
}