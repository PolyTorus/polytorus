use crate::Result;
use crypto::digest::Digest;
use crypto::sha2::Sha256;
use bincode::serialize;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

pub const PREFIX: &str = "BURN";
const VERIFICATION_DEPTH: i32 = 6;
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
    burn_records: HashMap<String, Vec<BurnInfo>>,
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
