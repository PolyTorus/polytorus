use std::fmt;
use secp256k1::rand::rngs::OsRng;
use secp256k1::{Secp256k1, Message};
use crate::blockchain::config::INITIAL_BALANCE;
use secp256k1::hashes::{sha256, Hash};
use lazy_static::lazy_static;

lazy_static! {
    pub static ref SECP: Secp256k1<secp256k1::All> = Secp256k1::new();
}

#[derive(Debug, Clone)]
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
            keypair: keypair,
            public_key: public_key,
        }
    }

    pub fn get_private_key(&self) -> secp256k1::SecretKey {
        self.keypair.secret_key()
    }

    pub fn sign(&self, message_hash: sha256::Hash) -> secp256k1::ecdsa::Signature {
        let message = Message::from_digest_slice(message_hash.as_ref()).unwrap();
        SECP.sign_ecdsa(&message, &self.keypair.secret_key())
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
}