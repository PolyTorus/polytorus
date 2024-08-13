use super::transaction::Transaction;
use std::fmt;

pub struct Pool {
    transactions: Vec<Transaction>,
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