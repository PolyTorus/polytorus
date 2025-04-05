use bincode::serialize;
use crypto::{digest::Digest, sha2::Sha256};
use merkle_cbt::CBMT;
use crate::{blockchain::block::MergeVu8, crypto::transaction::Transaction};

pub fn header(
    prev_hash: &str,
    tx_hash: &[u8],
    timestamp: u128,
    difficulty: usize,
    nonce: i32,
) -> crate::Result<String> {
    let content = (
        prev_hash.to_string(),
        tx_hash.to_vec(),
        timestamp,
        difficulty,
        nonce,
    );

    let bytes = serialize(&content)?;
    let mut hasher = Sha256::new();
    hasher.input(&bytes);

    Ok(hasher.result_str())
}

pub fn transactions(transactions: &[Transaction]) -> crate::Result<Vec<u8>> {
    let mut tx_hashes = Vec::new();
    for tx in transactions {
        tx_hashes.push(tx.hash()?.as_bytes().to_owned());
    }

    let tree = CBMT::<Vec<u8>, MergeVu8>::build_merkle_tree(tx_hashes);

    Ok(tree.root())
}

pub fn meets_difficulty(
    hash: &str,
    difficulty: usize,
) -> bool {
    let target = "0".repeat(difficulty);
    hash.starts_with(&target)
}
