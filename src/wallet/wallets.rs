use std::fmt;
use secp256k1::rand::rngs::OsRng;
use secp256k1::{Secp256k1, Message};
use crate::blockchain::chain::Chain;
use crate::blockchain::config::INITIAL_BALANCE;
use secp256k1::hashes::sha256;
use lazy_static::lazy_static;
use super::transaction::Transaction;
use super::transaction_pool::Pool;
use serde::{Serialize, Deserialize};

lazy_static! {
    pub static ref SECP: Secp256k1<secp256k1::All> = Secp256k1::new();
}

#[derive(Debug, Clone, PartialEq, Copy, Serialize, Deserialize)]
pub struct Wallet {
    pub balance: u64,
    pub keypair: secp256k1::Keypair,
    pub public_key: secp256k1::PublicKey,
}

impl Wallet {
    pub fn new() -> Wallet {
        let (secret_key, public_key) = SECP.generate_keypair(&mut OsRng);
        let keypair = secp256k1::Keypair::from_secret_key(&SECP, &secret_key);
        Wallet {
            balance: INITIAL_BALANCE,
            keypair,
            public_key,
        }
    }

    pub fn get_private_key(&self) -> secp256k1::SecretKey {
        self.keypair.secret_key()
    }

    pub fn sign(&self, message_hash: sha256::Hash) -> secp256k1::ecdsa::Signature {
        let message = Message::from_digest_slice(message_hash.as_ref()).unwrap();
        SECP.sign_ecdsa(&message, &self.keypair.secret_key())
    }

    pub fn verify(&self, message_hash: sha256::Hash, signature: secp256k1::ecdsa::Signature) -> bool {
        let message = Message::from_digest_slice(message_hash.as_ref()).unwrap();
        SECP.verify_ecdsa(&message, &signature, &self.public_key).is_ok()
    }

    pub fn create_transaction(&self, recipient: String, amount: u64, pool: &mut Pool) -> Result<Transaction, String> {
        if amount > self.balance {
            return Err("Amount exceeds balance".to_string());
        }
    
        let mut transaction = pool.exists(self.clone());
    
        if transaction.is_none() {
            transaction = Some(Transaction::new(self.clone(), recipient.clone(), amount)?);
        } else {
            let mut transaction = transaction.take().unwrap();
            transaction.output.push(super::transaction::Output {
                amount: amount,
                address: recipient,
            });
        }
        Ok(transaction.unwrap())
    }

    pub fn blockchain_wallet() -> Wallet {
        let (secret_key, public_key) = SECP.generate_keypair(&mut OsRng);
        Wallet {
            balance: 0,
            keypair: secp256k1::Keypair::from_secret_key(&SECP, &secret_key),
            public_key,
        }
    }

    pub fn calc_balance(&self, chain: &Chain) -> u64 {
        let mut balance = self.balance;
        let mut transactions = Vec::new();
        
        for block in &chain.chain {
            if let Ok(trans) = serde_json::from_str::<Vec<Transaction>>(&block.data) {
                transactions.extend(trans);
            }
        }

        let wallet_input_ts: Vec<&Transaction> = transactions.iter().filter(|t| t.input.iter().any(|i| i.address.public_key == self.public_key)).collect();

        if !wallet_input_ts.is_empty() {
            let recent_input_t = wallet_input_ts.into_iter().max_by(|a, b| {
                a.input.iter().map(|i| i.timestamp).max().cmp(&b.input.iter().map(|i| i.timestamp).max())
            }).unwrap();
            balance = recent_input_t.output.iter().find(|o| o.address == self.public_key.to_string()).map(|o| o.amount).unwrap_or(0);

            let start_time = recent_input_t.input.iter().map(|i| i.timestamp).max().unwrap();
            for t in transactions {
                if t.input.iter().any(|input| input.timestamp > start_time) {
                    if let Some(output) = t.output.iter().find(|o| o.address == self.public_key.to_string()) {
                        balance += output.amount;
                    }
                }
            }
        }
        balance
    }
}

impl fmt::Display for Wallet {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Wallet {{ balance: {}, keypair: {:?}, public_key: {} }}",
            self.balance, self.keypair, self.public_key)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use secp256k1::hashes::Hash;

    #[test]
    fn test_wallet_new() {
        let wallet = Wallet::new();
        println!("{}", wallet);
    }

    #[test]
    fn test_wallet_sign() {
        let wallet = Wallet::new();
        let message = "Hello, world!";
        let message_hash = sha256::Hash::hash(message.as_bytes());
        let signature = wallet.sign(message_hash);
        println!("{:?}", signature);
    }

    #[test]
    fn test_wallet_verify() {
        let wallet = Wallet::new();
        let message = "Hello, world!";
        let message_hash = sha256::Hash::hash(message.as_bytes());
        let signature = wallet.sign(message_hash);
        assert!(wallet.verify(message_hash, signature));
    }

    #[test]
    fn test_wallet_create_transaction() {
        let mut pool = Pool::new();
        let wallet = Wallet::new();
        let recipient = "recipient".to_string();
        let amount = 10;
        let transaction = wallet.create_transaction(recipient.clone(), amount, &mut pool).unwrap();
        println!("{:?}", transaction);
    }

    #[test]
    fn test_wallet_blockchain_wallet() {
        let wallet = Wallet::blockchain_wallet();
        println!("{}", wallet);
    }
}
