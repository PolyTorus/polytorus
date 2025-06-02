pub mod ecdsa;
pub mod fndsa;
pub mod traits;
pub mod transaction;
pub mod types;
pub mod wallets;

use traits::CryptoProvider;
use types::EncryptionType;
use ecdsa::EcdsaCrypto;
use fndsa::FnDsaCrypto;

/// Get crypto provider based on encryption type
pub fn get_crypto_provider(encryption_type: &EncryptionType) -> Box<dyn CryptoProvider> {
    match encryption_type {
        EncryptionType::ECDSA => Box::new(EcdsaCrypto),
        EncryptionType::FNDSA => Box::new(FnDsaCrypto),
    }
}
