use crate::Result;
use serde::{Deserialize, Serialize};
use sled;
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

pub const BURN_ADDRESS: &str = "BURN0000000000000000000000000000";

const ACTUAL_BURN_RATIP: f64 = 0.7;
const REWARD_BURN_RATIO: f64 = 0.3;

const BURN_VALIDITY: u128 = 3 * 24 * 60 * 60; // 3 days

const BASE_DIFFICULTY_REDUCTION: f64 = 0.5;
const BURN_WEIGHT_FACTOR: f64 = 0.2;
const MIN_DIFFICULTY: usize = 1;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BurnRecord {
    pub address: String,
    pub total: i32,
    pub actual_burn: i32,
    pub reward: i32,
    pub timestamp: u128,
    pub txid: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RewardPool {
    pub total: i32,
    pub last_distribution: u128,
    pub next_distribution: u128,
}

