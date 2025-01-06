use std::fmt;
use std::time::SystemTime;
// use sha2::{Digest, Sha256};
use super::config::{DIFFICULTY, MINE_RATE};
use secp256k1::hashes::{sha256, Hash};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Block {
    pub timestamp: u64,
    pub last_hash: String,
    pub hash: String,
    pub nonce: u64,
    pub data: String,
    pub difficulty: u32,
}

impl Block {
    pub fn new(
        timestamp: u64,
        last_hash: String,
        hash: String,
        nonce: u64,
        data: String,
        difficulty: u32,
    ) -> Block {
        Block {
            timestamp,
            last_hash,
            hash,
            nonce,
            data,
            difficulty,
        }
    }

    pub fn genesis() -> Block {
        Block::new(
            0,
            "genesis_last_hash".to_string(),
            "genesis_hash".to_string(),
            0,
            "genesis_data".to_string(),
            DIFFICULTY,
        )
    }

    pub fn mine_block(last_block: &Block, data: String) -> Block {
        let last_hash = last_block.hash.clone();
        let mut timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let mut nonce = 0;
        let mut difficulty = last_block.difficulty;

        let mut hash = Block::hash(
            timestamp,
            last_hash.clone(),
            nonce,
            data.clone(),
            difficulty,
        );
        while !hash
            .to_string()
            .starts_with(&"0".repeat(difficulty as usize))
        {
            nonce += 1;
            timestamp = SystemTime::now()
                .duration_since(SystemTime::UNIX_EPOCH)
                .unwrap()
                .as_secs();
            difficulty = Block::adjust_difficulty(last_block, timestamp);
            hash = Block::hash(
                timestamp,
                last_hash.clone(),
                nonce,
                data.clone(),
                difficulty,
            );
        }

        Block::new(
            timestamp,
            last_hash,
            hash.to_string(),
            nonce,
            data,
            last_block.difficulty,
        )
    }

    pub fn hash(
        timestamp: u64,
        last_hash: String,
        nonce: u64,
        data: String,
        difficulty: u32,
    ) -> sha256::Hash {
        let input = format!("{}{}{}{}{}", timestamp, last_hash, nonce, data, difficulty);
        let hash = sha256::Hash::hash(input.as_bytes());
        hash
    }

    pub fn hash_block(block: &Block) -> sha256::Hash {
        Block::hash(
            block.timestamp,
            block.last_hash.clone(),
            block.nonce,
            block.data.clone(),
            block.difficulty,
        )
    }

    pub fn adjust_difficulty(last_block: &Block, current_time: u64) -> u32 {
        let difficulty = last_block.difficulty;
        if difficulty < 1 {
            return 1;
        }

        if (current_time - last_block.timestamp) < MINE_RATE {
            difficulty + 1
        } else {
            difficulty - 1
        }
    }
}

impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Block - Timestamp: {}, Last Hash: {}, Hash: {},Nonce: {}, Data: {}, Difficulty: {}",
            self.timestamp, self.last_hash, self.hash, self.nonce, self.data, self.difficulty
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn block_new() {
        let block = Block::new(
            0,
            "foo".to_string(),
            "bar".to_string(),
            0,
            "baz".to_string(),
            0,
        );

        assert_eq!(block.timestamp, 0);
        assert_eq!(block.last_hash, "foo".to_string());
        assert_eq!(block.hash, "bar".to_string());
        assert_eq!(block.nonce, 0);
        assert_eq!(block.data, "baz".to_string());
        assert_eq!(block.difficulty, 0);
    }

    #[test]
    fn block_display() {
        let block = Block::new(
            0,
            "foo".to_string(),
            "bar".to_string(),
            0,
            "baz".to_string(),
            0,
        );

        assert_eq!(
            format!("{}", block),
            "Block - Timestamp: 0, Last Hash: foo, Hash: bar,Nonce: 0, Data: baz, Difficulty: 0"
        );
    }

    #[test]
    fn block_genesis() {
        let genesis_block = Block::genesis();

        assert_eq!(genesis_block.timestamp, 0);
        assert_eq!(genesis_block.last_hash, "genesis_last_hash".to_string());
        assert_eq!(genesis_block.hash, "genesis_hash".to_string());
        assert_eq!(genesis_block.data, "genesis_data".to_string());
    }

    #[test]
    fn block_mine_block() {
        let last_block = Block::genesis();
        let data = "mined data".to_string();
        let mined_block = Block::mine_block(&last_block, data.clone());

        assert_eq!(mined_block.last_hash, last_block.hash);
        assert_eq!(mined_block.data, data);
    }
}
