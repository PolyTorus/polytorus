use secp256k1::PublicKey;

use super::transaction::Transaction;
use std::fmt;

#[derive(Debug, Clone)]
pub struct Pool {
    pub transactions: Vec<Transaction>,
}

impl Pool {
    pub fn new() -> Pool {
        Pool {
            transactions: Vec::new(),
        }
    }

    pub fn update_or_add_transaction(&mut self, transaction: Transaction) {
        if let Some(index) = self
            .transactions
            .iter()
            .position(|t| t.id == transaction.id)
        {
            self.transactions[index] = transaction;
        } else {
            self.transactions.push(transaction);
        }
    }

    pub fn exists(&self, address: PublicKey) -> Option<Transaction> {
        self.transactions
            .clone()
            .into_iter()
            .find(|t| t.input.iter().any(|i| i.address.public_key == address))
    }

    pub fn valid_transactions(&self) -> Vec<Transaction> {
        self.transactions
            .clone()
            .into_iter()
            .filter(|t| t.is_valid())
            .collect()
    }

    pub fn clear(&mut self) {
        self.transactions.clear();
    }
}

impl fmt::Display for Pool {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Pool {{ transactions: {:?} }}", self.transactions)
    }
}

#[cfg(test)]
mod tests {
    use crate::wallet::wallets::Wallet;

    use super::*;

    #[test]
    fn test_pool_new() {
        let pool = Pool::new();
        println!("{}", pool);
    }

    #[test]
    fn test_pool_update_or_add_transaction() {
        let mut pool = Pool::new();
        let transaction = Transaction::new(Wallet::new(), "recipient".to_string(), 10).unwrap();
        pool.update_or_add_transaction(transaction);
        println!("{}", pool);
    }

    #[test]
    fn test_valid_transactions() {
        let mut pool = Pool::new();
        let wallet = Wallet::new();
        let recipient = "recipient".to_string();
        let amount = 10;
        let transaction = Transaction::new(wallet.clone(), recipient.clone(), amount).unwrap();
        pool.update_or_add_transaction(transaction);
        println!("{}", pool);
        let valid_transactions = pool.valid_transactions();
        println!("{:?}", valid_transactions);
    }
}
