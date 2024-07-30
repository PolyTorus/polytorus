// blockをimport
use super::block::Block;

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
}