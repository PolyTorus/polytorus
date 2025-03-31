use super::traits::CryptoProvider;
use fn_dsa::{
    sign_key_size, signature_size, KeyPairGenerator, KeyPairGeneratorStandard, SigningKey, SigningKeyStandard, VerifyingKey, VerifyingKeyStandard, DOMAIN_NONE, FN_DSA_LOGN_512, HASH_ID_RAW
};
use rand_core::OsRng;
use crate::errors::{BlockchainError, Result};
pub struct FnDsaCrypto;

impl CryptoProvider for FnDsaCrypto {
    fn gen_keypair(&self, encryption_type: super::types::CryptoType) -> Result<(super::types::PrivateKey, super::types::PublicKey)> {
        if encryption_type != super::types::CryptoType::FNDSA {
            return Err(BlockchainError::Other("Invalid encryption type".to_string()));
        }

        let mut kg = KeyPairGeneratorStandard::default();
        let mut sign_key = vec![0u8; sign_key_size(FN_DSA_LOGN_512)];
        let mut vrfy_key = vec![0u8; sign_key_size(FN_DSA_LOGN_512)];

        kg.keygen(FN_DSA_LOGN_512, &mut OsRng, &mut sign_key, &mut vrfy_key);

        Ok((
            super::types::PrivateKey::new(super::types::CryptoType::FNDSA, sign_key),
            super::types::PublicKey::new(super::types::CryptoType::FNDSA, vrfy_key),
        ))
    }

    fn sign(&self, private_key: &super::types::PrivateKey, message: &[u8]) -> Result<super::types::Signature> {

        if private_key.key_type != super::types::CryptoType::FNDSA {
            return Err(BlockchainError::InvalidSignature("Invalid encryption type".to_string()));
        }

        let mut sk = SigningKeyStandard::decode(&private_key.data).ok_or(BlockchainError::InvalidSignature("Invalid private key".to_string()))?;
        let mut signature = vec![0u8; signature_size(sk.get_logn())];
        sk.sign(
            &mut OsRng,
            &DOMAIN_NONE,
            &HASH_ID_RAW,
            message,
            &mut signature,
        );
        
        Ok(super::types::Signature::new(super::types::CryptoType::FNDSA, signature))
    }

    fn verify(&self, public_key: &super::types::PublicKey, message: &[u8], signature: &super::types::Signature) -> bool {

        if public_key.key_type != super::types::CryptoType::FNDSA || signature.key_type != super::types::CryptoType::FNDSA {
            return false;
        }

        if let Some(vk) = VerifyingKeyStandard::decode(&public_key.data) {
            vk.verify(&signature.data, &DOMAIN_NONE, &HASH_ID_RAW, message)
        } else {
            false
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::crypto::types::{CryptoType, PrivateKey, Signature};

    use super::*;
    
    #[test]
    fn test_fndsa_sign_verify() -> Result<()> {
        let crypto = FnDsaCrypto;
        let (private_key, public_key) = crypto.gen_keypair(CryptoType::FNDSA)?;
        
        let message = b"Hello, FNDSA!";
        let signature = crypto.sign(&private_key, message)?;
        
        // 有効な署名を検証
        assert!(crypto.verify(&public_key, message, &signature));
        
        // 改ざんされたメッセージを検証（失敗するはず）
        let tampered_message = b"Tampered message";
        assert!(!crypto.verify(&public_key, tampered_message, &signature));
        
        // 不正な署名を検証（失敗するはず）
        if signature.data.len() > 0 {
            let mut invalid_signature_data = signature.data.clone();
            invalid_signature_data[0] ^= 0xff; // 1バイト変更
            let invalid_signature = Signature::new(CryptoType::FNDSA, invalid_signature_data);
            assert!(!crypto.verify(&public_key, message, &invalid_signature));
        }
        
        Ok(())
    }
    
    #[test]
    fn test_fndsa_incompatible_types() -> Result<()> {
        let crypto = FnDsaCrypto;
        
        // ECDSA型の鍵で署名しようとすると失敗するはず
        let ecdsa_private_key = PrivateKey::new(CryptoType::ECDSA, vec![0; 32]);
        let result = crypto.sign(&ecdsa_private_key, b"test");
        assert!(result.is_err());
        
        // FNDSA鍵とECDSA署名では検証に失敗するはず
        let (_, fndsa_public_key) = crypto.gen_keypair(CryptoType::FNDSA)?;
        let ecdsa_signature = Signature::new(CryptoType::ECDSA, vec![0; 64]);
        assert!(!crypto.verify(&fndsa_public_key, b"test", &ecdsa_signature));
        
        Ok(())
    }
    
    #[test]
    fn test_fndsa_multiple_messages() -> Result<()> {
        let crypto = FnDsaCrypto;
        let (private_key, public_key) = crypto.gen_keypair(CryptoType::FNDSA)?;
        
        // 複数のメッセージに対して署名と検証
        for message in &[
            b"Message 1".to_vec(),
            b"Another test message".to_vec(),
            b"A third, longer message for more thorough testing".to_vec(),
        ] {
            let signature = crypto.sign(&private_key, message)?;
            assert!(crypto.verify(&public_key, message, &signature));
        }
        
        Ok(())
    }
}
