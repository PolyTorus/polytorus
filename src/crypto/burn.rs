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

pub struct BurnManager {
    db: sled::Db,
    reward_pool: RewardPool,
}

impl BurnManager {
    pub fn new() -> Result<Self> {
        let db = sled::open("data/burn")?;

        let reward_pool = match db.get("reward_pool")? {
            Some(data) => bincode::deserialize(&data)?,
            None => {
                let now = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();
                let next_distribution = now + 24 * 60 * 60 * 1000;

                let pool = RewardPool {
                    total: 0,
                    last_distribution: now,
                    next_distribution,
                };

                let pool_data = bincode::serialize(&pool)?;
                db.insert("reward_pool", pool_data)?;
                db.flush()?;

                pool
            },
        };

        Ok(Self { db, reward_pool })
    }

    pub fn add_burn_record(&mut self, address: &str, amount: i32, txid: &str) -> Result<BurnRecord> {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis();

        let actual_burn = (amount as f64 * ACTUAL_BURN_RATIP) as i32;
        let reward_pool_amount = amount - actual_burn;

        self.reward_pool.total += reward_pool_amount;
        let pool_data = bincode::serialize(&self.reward_pool)?;
        self.db.insert("reward_pool", pool_data)?;

        let record = BurnRecord {
            address: address.to_string(),
            total: amount,
            actual_burn,
            reward: reward_pool_amount,
            timestamp,
            txid: txid.to_string(),
        };

        let key = format!("{}:{}", address, txid);
        self.db.insert(key.as_bytes(), bincode::serialize(&record)?)?;
        self.db.flush()?;

        Ok(record)
    }
}