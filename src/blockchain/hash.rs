use bincode::serialize;
use crypto::{digest::Digest, sha2::Sha256};

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
