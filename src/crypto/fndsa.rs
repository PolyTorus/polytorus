use super::traits::CryptoProvider;
use fn_dsa::{
    signature_size, SigningKey, SigningKeyStandard, VerifyingKey, VerifyingKeyStandard,
    DOMAIN_NONE, HASH_ID_RAW,
};
use rand;

pub struct FnDsaCrypto;

impl CryptoProvider for FnDsaCrypto {
    fn sign(&self, private_key: &[u8], message: &[u8]) -> Vec<u8> {
        let mut sk = SigningKeyStandard::decode(private_key).unwrap();
        let mut signature = vec![0u8; signature_size(sk.get_logn())];
        let mut rng = rand::thread_rng();
        sk.sign(
            &mut rng,
            &DOMAIN_NONE,
            &HASH_ID_RAW,
            message,
            &mut signature,
        );
        signature
    }

    fn verify(&self, public_key: &[u8], message: &[u8], signature: &[u8]) -> bool {
        VerifyingKeyStandard::decode(public_key).unwrap().verify(
            signature,
            &DOMAIN_NONE,
            &HASH_ID_RAW,
            message,
        )
    }
}
