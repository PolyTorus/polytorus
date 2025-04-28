use super::{ecdsa::EcdsaCrypto, fndsa::FnDsaCrypto, types::EncryptionType};

pub trait CryptoProvider {
    fn sign(&self, private_key: &[u8], message: &[u8]) -> Vec<u8>;
    fn verify(&self, public_key: &[u8], message: &[u8], signature: &[u8]) -> bool;
}

pub fn create_crypto_provider(enc_type: EncryptionType) -> Box<dyn CryptoProvider> {
    match enc_type {
        EncryptionType::ECDSA => Box::new(EcdsaCrypto),
        EncryptionType::FNDSA => Box::new(FnDsaCrypto),
    }
}
