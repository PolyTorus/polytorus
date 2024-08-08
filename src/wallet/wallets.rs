use std::fmt;
use secp256k1::rand::rngs::OsRng;
use secp256k1::Secp256k1;
use uuid::Uuid;
use crate::blockchain::config::INITIAL_BALANCE;

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
}