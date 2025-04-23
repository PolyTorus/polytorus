use crypto::{digest::Digest, sha2::Sha256};
use serde::{Deserialize, Serialize};

pub mod ecdsa;
pub mod fndsa;
pub mod traits;
pub mod transaction;
pub mod wallets;
pub mod types;
pub mod burn;

pub const PREFIX: &str = "BURN";

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
    pub tag: String,
}

// Burn Protocol
// - GenBurnAddr(1^k, t): Generate a burn address for a given tag t.
// - BurnVerify(1^k, t, burnAddr): Verify if the burn address is valid for the tag t.
pub struct BurnProtocol;

impl BurnProtocol {
    pub fn generate_burn_address(&self, tag: &str) -> String {
        let mut hasher = Sha256::new();
        hasher.input_str(tag);
        let hash = hasher.result_str();

        let mut hash_bytes = hash.into_bytes();
        let last_byte_index = hash_bytes.len() - 1;
        hash_bytes[last_byte_index] ^= 1; // Flip the last bit

        hash_bytes.iter().map(|b| format!("{:02x}", b)).collect::<String>()
    }

    // Verify a burn address
    // 1. Generate a address from the tag
    // 2. Compare the generated address with the provided address
    pub fn burn_verify(&self, tag: &str, burn_addr: &str) -> bool {
        let generated = self.generate_burn_address(tag);
        generated == burn_addr
    }
}
