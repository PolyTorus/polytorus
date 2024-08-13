// blockをimport
use super::block::Block;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
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

            if block.last_hash != last_block.hash || block.hash != Block::hash_block(block).to_string() {
                return false;
            }
        }

        true
    }

    pub fn replace_chain(&mut self, new_chain: &Chain) {
        if new_chain.chain.len() <= self.chain.len() {
            return;
        } else if !new_chain.is_valid_chain() {
            return;
        }

        self.chain = new_chain.chain.clone();
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

    #[test]
    fn chain_replace_chain() {
        let mut blockchain = Chain::new();
        blockchain.add_block("First block after genesis".to_string());

        let mut new_blockchain = Chain::new();
        new_blockchain.replace_chain(&blockchain);

        assert_eq!(new_blockchain.chain.len(), 1);
    }
}