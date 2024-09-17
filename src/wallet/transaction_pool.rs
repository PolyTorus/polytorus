use super::{transaction::{Input, Transaction}, wallets::Wallet};
use std::fmt;

#[derive(Debug, Clone)]
pub struct Pool {
    pub transactions: Vec<Transaction>,
}

impl Pool {
    pub fn new() -> Pool {
        Pool {
            transactions: vec![],
        }
    }

    pub fn update_or_add_transaction(&mut self, transaction: Transaction) {
        let index = self.transactions.iter().position(|t| t.id == transaction.id);
        match index {
            Some(i) => self.transactions[i] = transaction,
            None => self.transactions.push(transaction),
        }
    }

    pub fn exists(&self, address: Wallet) -> Option<Transaction> {
        self.transactions.iter().find(|t| <Vec<Input> as Clone>::clone(&t.input).into_iter().any(|i| i.address.public_key == address.public_key)).cloned()
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
}
