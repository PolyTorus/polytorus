use super::wallets::Wallet;
use crate::blockchain::config::MINING_REWARD;
use bincode;
use secp256k1::hashes::{sha256, Hash};
use serde::{Deserialize, Serialize};
use std::time::SystemTime;
use uuid::Uuid;

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
            input: Vec::new(),
            output: Vec::new(),
        };

        transaction.input.push(Input {
            timestamp: SystemTime::now(),
            amount: sender.balance,
            address: sender.clone(),
            signature: sender.sign(sha256::Hash::hash(
                &bincode::serialize(&transaction.output).unwrap(),
            )),
        });

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
        let message_hash = sha256::Hash::hash(&bincode::serialize(&self.output).unwrap());
        let signature = wallet.sign(message_hash);
        let mut transaction = self.clone();
        transaction.input = vec![Input {
            timestamp: SystemTime::now(),
            amount: wallet.balance,
            address: wallet.clone(),
            signature,
        }];
        transaction
    }

    // verify transaction
    pub fn verify(&self) -> bool {
        let mut message = self.clone();
        message.input = vec![];
        let message_hash = sha256::Hash::hash(&bincode::serialize(&message.output).unwrap());
        self.input
            .iter()
            .all(|input| input.address.verify(message_hash, input.signature))
    }

    pub fn update(
        &mut self,
        sender_wallet: Wallet,
        recipient: String,
        amount: u64,
    ) -> Result<Self, String> {
        let sender_output = self
            .output
            .iter_mut()
            .find(|output| output.address == sender_wallet.public_key.to_string())
            .unwrap();
        if amount > sender_output.amount {
            return Err("Amount exceeds balance".to_string());
        }

        sender_output.amount -= amount;
        self.output.push(Output {
            amount,
            address: recipient.to_string(),
        });

        Ok(self.sign(&sender_wallet))
    }

    pub fn is_valid(&self) -> bool {
        let output_amount: u64 = self.output.iter().map(|output| output.amount).sum();
        let input_amount: u64 = self.input.iter().map(|input| input.amount).sum();
        output_amount == input_amount
    }

    pub fn reward(&self, miner: Wallet, blockchain: Wallet) -> Self {
        self.without_outputs(
            miner,
            vec![Output {
                amount: MINING_REWARD,
                address: blockchain.public_key.to_string(),
            }],
        )
    }

    pub fn without_outputs(&self, sender: Wallet, outputs: Vec<Output>) -> Self {
        let mut transaction = self.clone();
        transaction.output.push(Output {
            amount: sender.balance - outputs.iter().map(|output| output.amount).sum::<u64>(),
            address: sender.public_key.to_string(),
        });
        transaction.sign(&sender);
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
        println!("{:?}", transaction);
        let signed_transaction = transaction.sign(&wallet);
        println!("{:?}", signed_transaction);
    }

    #[test]
    fn test_transaction_update() {
        let wallet = Wallet::new();
        let mut transaction = Transaction::new(wallet.clone(), "recipient".to_string(), 10).unwrap();
        let updated_transaction = transaction.update(wallet.clone(), "recipient".to_string(), 5).unwrap();
        println!("{:?}", updated_transaction);
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
