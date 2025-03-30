use super::traits::CryptoProvider;
use rand_core::OsRng;
use secp256k1::{Message, Secp256k1, SecretKey};
use super::types::{self, CryptoType};
use crate::errors::{Result, BlockchainError};

pub struct EcdsaCrypto;

impl CryptoProvider for EcdsaCrypto {
    fn gen_keypair(&self, encryption_type: types::CryptoType) -> Result<(types::PrivateKey, types::PublicKey)> {
        if encryption_type != CryptoType::ECDSA {
            return Err(BlockchainError::Other(
                "ECDSACrypto can make ECDSA keypair".to_string()
            ));
        }

        let secp = Secp256k1::new();
        let (secret_key, public_key) = secp.generate_keypair(&mut OsRng);

        Ok((
            types::PrivateKey::new(CryptoType::ECDSA, secret_key.secret_bytes().to_vec()),
            types::PublicKey::new(CryptoType::ECDSA, public_key.serialize().to_vec())
        ))
    }

    fn sign(&self, private_key: &types::PrivateKey, message: &[u8]) -> Result<types::Signature> {
        if private_key.key_type != CryptoType::ECDSA {
            return Err(BlockchainError::InvalidSignature(
                "ECDSACrypto cannot sign other keys".to_string()
            ));
        }

        let secp = Secp256k1::signing_only();
        let sk = SecretKey::from_slice(&private_key.data)
            .map_err(|e| BlockchainError::InvalidSignature(e.to_string()))?;
        let msg = if message.len() == 32 {
            let mut digest = [0u8; 32];
            digest.copy_from_slice(message);
            Message::from_digest(digest)
        } else {
            return Err(BlockchainError::InvalidSignature("Message must be exactly 32 bytes for digest".to_string()));
        };
        let sig = secp.sign_ecdsa(&msg, &sk);
        
        Ok(types::Signature::new(
            CryptoType::ECDSA,
            sig.serialize_compact().to_vec(),
        ))
    }

    fn verify(&self, public_key: &types::PublicKey, message: &[u8], signature: &types::Signature) -> bool {
        if public_key.key_type != CryptoType::ECDSA || signature.key_type != CryptoType::ECDSA {
            return false;
        }
        
        let secp = Secp256k1::verification_only();
        
        match (
            secp256k1::PublicKey::from_slice(&public_key.data),
            message.try_into().ok().map(Message::from_digest),
            secp256k1::ecdsa::Signature::from_compact(&signature.data),
        ) {
            (Ok(pk), Some(msg), Ok(sig)) => secp.verify_ecdsa(&msg, &sig, &pk).is_ok(),
            _ => false,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_ecdsa_sign_verify() -> Result<()> {
        let crypto = EcdsaCrypto;
        let (private_key, public_key) = crypto.gen_keypair(CryptoType::ECDSA)?;
        
        let message = b"Hello, ECDSA!";
        let signature = crypto.sign(&private_key, message)?;
        
        // 有効な署名を検証
        assert!(crypto.verify(&public_key, message, &signature));
        
        // 改ざんされたメッセージを検証（失敗するはず）
        let tampered_message = b"Tampered message";
        assert!(!crypto.verify(&public_key, tampered_message, &signature));
        
        // 不正な署名を検証（失敗するはず）
        let mut invalid_signature_data = signature.data.clone();
        invalid_signature_data[0] ^= 0xff; // 1バイト変更
        let invalid_signature = types::Signature::new(CryptoType::ECDSA, invalid_signature_data);
        assert!(!crypto.verify(&public_key, message, &invalid_signature));
        
        Ok(())
    }
    
    #[test]
    fn test_ecdsa_incompatible_types() -> Result<()> {
        let crypto = EcdsaCrypto;
        
        // FNDSA型の鍵で署名しようとすると失敗するはず
        let fndsa_private_key = types::PrivateKey::new(CryptoType::FNDSA, vec![0; 32]);
        let result = crypto.sign(&fndsa_private_key, b"test");
        assert!(result.is_err());
        
        // ECDSA鍵とFNDSA署名では検証に失敗するはず
        let (_, ecdsa_public_key) = crypto.gen_keypair(CryptoType::ECDSA)?;
        let fndsa_signature = types::Signature::new(CryptoType::FNDSA, vec![0; 64]);
        assert!(!crypto.verify(&ecdsa_public_key, b"test", &fndsa_signature));
        
        Ok(())
    }
}