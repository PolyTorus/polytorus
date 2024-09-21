use crate::blockchain::chain::Chain;
use crate::wallet::{transaction_pool::Pool, wallets::Wallet};
use crate::app::p2p::P2p;

pub struct Minner {
    pub chain: Chain,
    pub transaction_pool: Pool,
    pub wallet: Wallet,
    pub p2p: P2p,
}