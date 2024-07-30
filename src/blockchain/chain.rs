// blockをimport
use super::block::Block;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Chain {
    chain: Vec<Block>,
}

impl Chain {
    pub fn new() -> Self {
        Chain {
            chain: vec![Block::genesis()],
        }
    }

    pub fn add_block(&mut self, data: String) -> &Block {
        let last_block = self.chain.last().unwrap();
        let block = Block::mine_block(last_block, data);
        self.chain.push(block);
        self.chain.last().unwrap()
    }

    pub fn is_valid_chain(&self) -> bool {
        if self.chain[0] != Block::genesis() {
            return false;
        }

        for i in 1..self.chain.len() {
            let block = &self.chain[i];
            let last_block = &self.chain[i - 1];

            if block.last_hash != last_block.hash || block.hash != Block::hash_block(block) {
                return false;
            }
        }

        true
    }
}

impl Default for Chain {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chain_new() {
        let chain = Chain::new();

        assert_eq!(chain.chain.len(), 1);
        assert_eq!(chain.chain[0].hash, Block::genesis().hash);
    }

    #[test]
    fn chain_add_block() {
        let mut chain = Chain::new();
        let data = "foo".to_string();
        chain.add_block(data.clone());

        assert_eq!(chain.chain.len(), 2);
        assert_eq!(chain.chain[1].data, data);
    }

    #[test]
    fn chain_is_valid_chain() {
        let mut chain = Chain::new();

        assert!(chain.is_valid_chain());

        chain.chain[0].data = "corrupt_data".to_string();

        assert!(!chain.is_valid_chain());

        chain.chain[0].data = "genesis_data".to_string();
        chain.chain.push(Block::mine_block(&chain.chain[
            chain.chain.len() - 1
        ], "foo".to_string()));

        assert!(!chain.is_valid_chain());
    }
}