use crate::crypto::transaction::Transaction;

/// Run performs a proof-of-work
pub(crate) fn run_proof_of_work<F>(
    prev_hash: String,
    transactions: &[Transaction],
    timestamp: u128,
    difficulty: usize,
    hash_fn: F,
) -> crate::Result<(String, i32)>
where
    F: Fn(&str, &[u8], u128, usize, i32) -> crate::Result<String>,
{
    let tx_hash = super::hash::transactions(transactions)?;
    let mut nonce = 0;

    loop {
        let hash = hash_fn(&prev_hash, &tx_hash, timestamp, difficulty, nonce)?;
        if super::hash::meets_difficulty(&hash, difficulty) {
            return Ok((hash, nonce));
        }
        nonce += 1;
    }
}

pub fn proof_of_work_curried(
    prev_hash: String,
    transactions: &[Transaction],
    timestamp: u128,
) -> impl Fn(usize) -> crate::Result<(String, i32)> {
    let tx_hash = super::hash::transactions(transactions).unwrap_or_default();
    let transactions_owned = transactions.to_vec();
    
    move |difficulty| {
        let mut nonce = 0;
        loop {
            let hash = super::hash::header(
                &prev_hash,
                &tx_hash,
                timestamp,
                difficulty,
                nonce,
            )?;
            
            if super::hash::meets_difficulty(&hash, difficulty) {
                return Ok((hash, nonce));
            }
            nonce += 1;
        }
    }
}
