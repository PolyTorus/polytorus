use crate::crypto::types::{CryptoType, PrivateKey, PublicKey, Signature};
use crate::errors::Result;

pub trait CryptoProvider {
    fn gen_keypair(&self, encryption_type: CryptoType) -> Result<(PrivateKey, PublicKey)>;
    fn sign(&self, private_key: &PrivateKey, message: &[u8]) -> Result<Signature>;
    fn verify(&self, public_key: &PublicKey, message: &[u8], signature: &Signature) -> bool;
}
