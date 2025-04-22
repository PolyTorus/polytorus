use crate::Result;
use crypto::digest::Digest;
use crypto::sha2::Sha256;
use bincode::serialize;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const PREFIX: &str = "BURN";
const VERIFICATION_DEPTH: i32 = 10;
const WEIGHT_DECAY: f64 = 0.9;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct BurnInfo {
    pub address: String,
    pub amount: i32,
    pub block_height: i32,
    pub burn_txid: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BurnProof {
    pub address: String,
    pub total_burn_score: f64,
    pub burns: Vec<BurnInfo>,
    pub nonce: i32,
    pub timestamp: u128,
}

pub struct BurnManager {
    pub burn_records: HashMap<String, Vec<BurnInfo>>,
}

impl BurnManager {
    pub fn new() -> Self {
        Self {
            burn_records: HashMap::new(),
        }
    }

    pub fn generate_burn_address(&self, tag: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.input_str(&format!("{}{}", PREFIX, tag));
        let mut hash = hasher.result_str();

        let last_byte = hash.pop().unwrap();
        let new_last_byte = if last_byte == '0' { '1' } else { '0' };
        hash.push(new_last_byte);

        hash
    }

    pub fn verify_burn_address(&self, address: &str, tag: &str) -> bool {
        let generated = self.generate_burn_address(tag);
        generated == address
    }

    pub fn register_burn(&mut self, burn_info: BurnInfo) {
        let burns = self.burn_records.entry(burn_info.address.clone()).or_insert_with(Vec::new);
        burns.push(burn_info);
    }

    pub fn calculate_burn_score(&self, address: &str, current_height: i32) -> f64 {
        match self.burn_records.get(address) {
            Some(burns) => {
                burns.iter().fold(0.0, |score, burn| {
                    let age = current_height - burn.block_height;

                    if age >= VERIFICATION_DEPTH {
                        return score;
                    }

                    let decay = WEIGHT_DECAY.powi(age);

                    score + (burn.amount as f64 * decay)
                })
            },
            None => 0.0,
        }
    }

    pub fn create_burn_proof(&self, address: &str, current_height: i32) -> Result<BurnProof> {
        let burns = match self.burn_records.get(address) {
            Some(b) => b.clone(),
            None => Vec::new(),
        };

        let total_burn_score = self.calculate_burn_score(address, current_height);

        let proof = BurnProof {
            address: address.to_string(),
            total_burn_score,
            burns: burns.clone(),
            nonce: 0,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .expect("Time went backwards")
                .as_millis(),
        };

        Ok(proof)
    }

    pub fn verify_burn_proof(&self, proof: &BurnProof, difficulty: usize, block_data: &[u8]) -> Result<bool> {
        for burn in &proof.burns {
            if !self.burn_records
                .get(&proof.address)
                .map_or(false, |burns| burns.contains(burn)) {
                return Ok(false);
                }
        }

        let current_height = self.get_latest_block_height()?;
        let calculated_score = self.calculate_burn_score(&proof.address, current_height);

        if (calculated_score - proof.total_burn_score).abs() > f64::EPSILON {
            return Ok(false);
        }

        self.meets_difficulty(proof, difficulty, block_data)
    }

    fn get_latest_block_height(&self) -> Result<i32> {
        // Placeholder for actual implementation
        Ok(0)
    }

    fn meets_difficulty(&self, proof: &BurnProof, difficulty: usize, block_data: &[u8]) -> Result<bool> {
        let mut hasher = Sha256::new();

        hasher.input(block_data);
        hasher.input(&serialize(proof)?);
        let hash = hasher.result_str();

        let burn_factor = (proof.total_burn_score.sqrt() as usize).max(1);
        let effective_difficulty = difficulty.saturating_sub(burn_factor);

        let prefix = "0".repeat(effective_difficulty);
        Ok(hash.starts_with(&prefix))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_burn_address_generation() {
        let burn_manager = BurnManager::new();
        
        let tag1 = "test_address";
        let addr1 = burn_manager.generate_burn_address(tag1);
        let addr2 = burn_manager.generate_burn_address(tag1);
        assert_eq!(addr1, addr2, "Generated addresses should match");

        let tag2 = "different_address";
        let addr3 = burn_manager.generate_burn_address(tag2);
        assert_ne!(addr1, addr3, "Generated addresses should be different");

        assert!(burn_manager.verify_burn_address(&addr1, tag1), "address should be valid for the tag");
        assert!(!burn_manager.verify_burn_address(&addr1, tag2), "address should not be valid for a different tag");
    }

    #[test]
    fn test_burn_registration_and_scoring() {
        let mut burn_manager = BurnManager::new();

        let burn_info1 = BurnInfo {
            address: "miner1".to_string(),
            amount: 100,
            block_height: 10,
            burn_txid: "tx1".to_string(),
        };
        
        let burn_info2 = BurnInfo {
            address: "miner1".to_string(),
            amount: 50,
            block_height: 15,
            burn_txid: "tx2".to_string(),
        };
        
        let burn_info3 = BurnInfo {
            address: "miner2".to_string(),
            amount: 200,
            block_height: 12,
            burn_txid: "tx3".to_string(),
        };
        
        burn_manager.register_burn(burn_info1.clone());
        burn_manager.register_burn(burn_info2.clone());
        burn_manager.register_burn(burn_info3.clone());
        
        let current_height = 20;
        
        let score_miner1 = burn_manager.calculate_burn_score("miner1", current_height);
        let score_miner2 = burn_manager.calculate_burn_score("miner2", current_height);
        let score_nonexistent = burn_manager.calculate_burn_score("nonexistent", current_height);
        
        assert!(score_miner1 > 0.0, "burn records should yield a positive score");
        assert!(score_miner2 > 0.0, "burn records should yield a positive score");
        assert_eq!(score_nonexistent, 0.0, "nonexistent address should yield a score of 0");
        
        let newer_burn_score = burn_manager.calculate_burn_score("miner1", 16);
        let older_burn_score = burn_manager.calculate_burn_score("miner1", current_height);
        assert!(newer_burn_score > older_burn_score, "newer burns should yield a higher score");
    }
}