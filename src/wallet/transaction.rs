use uuid::Uuid;
use super::wallets::Wallet;
use std::time::SystemTime;
use secp256k1::hashes::{sha256, Hash};
use bincode;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct Transaction {
    pub id: Uuid,
    pub input: Vec<Input>,
    pub output: Vec<Output>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Input {
    pub timestamp: SystemTime,
    pub amount: u64,
    pub address: Wallet,
    pub signature: secp256k1::ecdsa::Signature,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
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
            address: wallet.clone(),
            signature: wallet.sign(sha256::Hash::hash(&bincode::serialize(&transaction.output).unwrap())),
        });
        transaction
    }

    // verify transaction
    pub fn verify(&self) -> bool {
        let mut message = self.clone();
        message.input = vec![];
        let message_hash = sha256::Hash::hash(&bincode::serialize(&message.output).unwrap());
        self.input.iter().all(|input| {
            input.address.verify(message_hash, input.signature)
        })
    }

    pub fn update(&mut self, sender_wallet: Wallet, receipient: String, amount: u64) -> Result<Self, String> {
        let sender_output = self.output.iter_mut().find(|output| output.address == sender_wallet.public_key.to_string()).unwrap();
        if amount > sender_output.amount {
            return Err("Amount exceeds balance".to_string());
        }
    
        sender_output.amount -= amount;
        self.output.push(Output {
            amount,
            address: receipient,
        });
        
        Ok(self.sign(&sender_wallet))
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

    #[test]
    fn test_transaction_invalid_amount() {
        let wallet = Wallet::new();
        let transaction = Transaction::new(wallet.clone(), "recipient".to_string(), 1000);
        assert!(transaction.is_err());
    }

    #[test]
    fn test_transaction_verify() {
        let wallet = Wallet::new();
        let transaction = Transaction::new(wallet.clone(), "recipient".to_string(), 10).unwrap();
        let signed_transaction = transaction.sign(&wallet);
        assert!(signed_transaction.verify());
    }
}