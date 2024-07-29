use std::fmt;

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
}