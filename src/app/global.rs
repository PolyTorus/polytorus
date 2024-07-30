use crate::blockchain::chain::Chain;
use lazy_static::lazy_static;
use std::sync::Mutex;
use serde::{Deserialize, Serialize};

lazy_static! {
	pub static ref CHAIN: Mutex<Chain> = Mutex::new(Chain::new());
}

// block struct to json
#[derive(Serialize, Deserialize)]
pub struct BlockJson {
    pub timestamp: u64,
    pub last_hash: String,
    pub hash: String,
    pub data: String,
}

// post
#[derive(Serialize, Deserialize)]
pub struct PostBlockJson {
    pub data: String,
}