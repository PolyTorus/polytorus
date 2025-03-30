use crate::types::{Address, BlockHash, TransactionId};
use crate::crypto::types::{CryptoType, PrivateKey, PublicKey, Signature};
use crate::errors::Result;

pub fn to_block_hash(hash: &str) -> BlockHash {
    BlockHash(hash.to_string())
}

