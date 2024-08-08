use std::fmt;
use secp256k1::rand::rngs::OsRng;
use secp256k1::{Secp256k1, Message};
use crate::blockchain::config::INITIAL_BALANCE;
use secp256k1::hashes::{sha256, Hash};

pub struct Wallet {
    pub balance: u64,
    pub keypair: secp256k1::Keypair,
    pub public_key: secp256k1::PublicKey,
}

impl Wallet {
    fn new() -> Wallet {
        let secp = Secp256k1::new();
        let (secret_key, public_key) = secp.generate_keypair(&mut OsRng);
        let keypair = secp256k1::Keypair::from_secret_key(&secp, &secret_key);
        Wallet {
            balance: INITIAL_BALANCE,
            keypair: keypair,
            public_key: public_key,
        }
    }

    fn sign(message: String, secret_key: &secp256k1::SecretKey) -> secp256k1::ecdsa::Signature {
        let secp = Secp256k1::new();
        let digest = sha256::Hash::hash(message.as_bytes());
        let message = Message::from_digest(digest.to_byte_array());
        let sig = secp.sign_ecdsa(&message, &secret_key);
        sig
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
        let message = "Hello, world!".to_string();
        let signature = Wallet::sign(message, &wallet.keypair.secret_key());
        println!("wallet keypair secret_key: {:?}", wallet.keypair.secret_key());
        println!("signature: {:?}", signature);
    }
}