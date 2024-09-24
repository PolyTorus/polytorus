use crate::app::minner::Minner;
use crate::app::p2p::P2p;
use crate::blockchain::chain::Chain;
use crate::wallet::transaction_pool::Pool;
use crate::wallet::wallets::Wallet;
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use once_cell::sync::Lazy;
use std::sync::Arc;

pub static CHAIN: Lazy<Arc<Mutex<Chain>>> = Lazy::new(|| Arc::new(Mutex::new(Chain::new())));
pub static WALLET: Lazy<Arc<Mutex<Wallet>>> = Lazy::new(|| Arc::new(Mutex::new(Wallet::new())));
pub static POOL: Lazy<Arc<Mutex<Pool>>> = Lazy::new(|| Arc::new(Mutex::new(Pool::new())));

pub static SERVER: Lazy<Arc<Mutex<Option<P2p>>>> = Lazy::new(|| Arc::new(Mutex::new(None)));
pub static MINER: Lazy<Arc<Mutex<Option<Minner>>>> = Lazy::new(|| Arc::new(Mutex::new(None)));

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

#[derive(Serialize, Deserialize)]
pub struct PostPoolJson {
    pub recipient: String,
    pub amount: u64,
}

// p2p server
pub async fn start_p2p() {
    let server = SERVER.clone();
    tokio::spawn(async move {
        if let Some(ref mut p2p) = *server.lock().await {
            if let Err(e) = p2p.listen().await {
                eprintln!("Error listening: {}", e);
            }
        }
    });
}