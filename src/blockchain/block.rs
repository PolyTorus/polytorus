use std::fmt;
use std::time::SystemTime;
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Block {
    pub timestamp: u64,
    pub last_hash: String,
    pub hash: String,
    pub data: String,
}

impl Block {
    pub fn new(timestamp: u64, last_hash: String, hash: String, data: String) -> Block {
        Block {
            timestamp,
            last_hash,
            hash,
            data,
        }
    }

    pub fn genesis() -> Block {
        Block::new(0, "genesis_last_hash".to_string(), "genesis_hash".to_string(), "genesis_data".to_string())
    }

    pub fn mine_block(last_block: &Block, data: String) -> Block {
        let timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_secs();
        let last_hash = last_block.hash.clone();
        let hash = "todo".to_string();

        Block::new(timestamp, last_hash, hash, data)
    }

    pub fn hash(timestamp: u64, last_hash: String, data: String) -> String {
        let input = format!("{}{}{}", timestamp, last_hash, data);
        let mut hasher = Sha256::new();
        hasher.update(input);
        format!("{:x}", hasher.finalize())
    }

    pub fn hash_block(block: &Block) -> String {
        Block::hash(block.timestamp, block.last_hash.clone(), block.data.clone())
    }
}

impl fmt::Display for Block {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "Block - Timestamp: {}, Last Hash: {}, Hash: {}, Data: {}",
            self.timestamp, self.last_hash, self.hash, self.data
        )
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn block_new() {
        let block = Block::new(0, "foo".to_string(), "bar".to_string(), "baz".to_string());

        assert_eq!(block.timestamp, 0);
        assert_eq!(block.last_hash, "foo".to_string());
        assert_eq!(block.hash, "bar".to_string());
        assert_eq!(block.data, "baz".to_string());
    }

    #[test]
    fn block_display() {
        let block = Block::new(0, "foo".to_string(), "bar".to_string(), "baz".to_string());

        assert_eq!(
            format!("{}", block),
            "Block - Timestamp: 0, Last Hash: foo, Hash: bar, Data: baz"
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

    #[test]
    fn block_hash() {
        let timestamp = 0;
        let last_hash = "foo".to_string();
        let data = "bar".to_string();
        let hash = Block::hash(timestamp, last_hash.clone(), data.clone());

        assert_eq!(hash, Block::hash(timestamp, last_hash, data));
    }

}