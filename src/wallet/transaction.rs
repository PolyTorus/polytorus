use uuid::Uuid;
use super::wallets::{self, Wallet};
use secp256k1::hashes::{sha256, Hash};
use secp256k1::rand::rngs::OsRng;
use secp256k1::{Secp256k1, Message};
use std::time::SystemTime;

#[derive(Debug, Clone)]
pub struct Transaction {
    pub id: Uuid,
    pub input: Vec<Input>,
    pub output: Vec<Output>,
}

#[derive(Debug, Clone)]
pub struct Input {
    pub timestamp: SystemTime,
    pub amount: u64,
    pub address: String,
    pub signature: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct Output {
    pub amount: u64,
    pub address: String,
}

impl Transaction {
    pub fn new(sender: Wallet, receipient: String, amount: u64) -> Result<Self, String> {
        if amount > sender.balance {
            return Err("Amount exceeds balance".to_string());
        }

        let mut transaction = Transaction {
            id: Uuid::new_v4(),
            input: vec![],
            output: vec![],
        };

        transaction.output.push(Output {
            amount: sender.balance - amount,
            address: sender.public_key.to_string(),
        });

        transaction.output.push(Output {
            amount: amount,
            address: receipient,
        });

        Ok(transaction)
    }
   

    // sign transaction
    pub fn sign(&self, wallet: &Wallet) -> Self {
        let mut transaction = self.clone();
        transaction.input.push(Input {
            timestamp: SystemTime::now(),
            amount: wallet.balance,
            address: wallet.public_key.to_string(),
            signature: vec![],
        });
        transaction
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_transaction() {
        let wallet = Wallet::new();
        let transaction = Transaction::new(wallet.clone(), "recipient".to_string(), 10).unwrap();
        let signed_transaction = transaction.sign(&wallet);
        println!("{:?}", signed_transaction);
    }
}