use crate::app::p2p::P2p;
use crate::blockchain::block::Block;
use crate::blockchain::chain::Chain;
use crate::wallet::transaction::Transaction;
use crate::wallet::{transaction_pool::Pool, wallets::Wallet};

#[derive(Clone)]
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

    pub async fn mine(&mut self) -> Block {
        let mut valid_transaction: Vec<Transaction> = self.transaction_pool.valid_transactions();

        let rewards: Vec<Transaction> = valid_transaction
            .iter()
            .map(|transaction| transaction.reward(self.wallet.clone(), Wallet::blockchain_wallet()))
            .collect();
        valid_transaction.extend(rewards);

        let transactions_json =
            serde_json::to_string(&valid_transaction).expect("Failed to serialize transactions");
        let block = self.chain.add_block(transactions_json);
        self.p2p.sync_chain().await;

        self.transaction_pool.clear();
        self.p2p.broadcast_clear_transactions().await;
        block.clone()
    }
}
