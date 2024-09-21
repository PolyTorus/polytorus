use crate::blockchain::chain::Chain;
use crate::wallet::transaction::Transaction;
use crate::wallet::{transaction_pool::Pool, wallets::Wallet};
use crate::app::p2p::P2p;

pub struct Minner {
    pub chain: Chain,
    pub transaction_pool: Pool,
    pub wallet: Wallet,
    pub p2p: P2p,
}

impl Minner {
    pub fn new(chain: Chain, transaction_pool: Pool, wallet: Wallet, p2p: P2p) -> Self {
        Minner {
            chain,
            transaction_pool,
            wallet,
            p2p,
        }
    }

    pub fn mine(&self) -> Vec<Transaction> {
        self.transaction_pool.valid_transactions()
    }
}
