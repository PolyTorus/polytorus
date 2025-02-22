use super::traits::CryptoProvider;
use secp256k1::{Message, PublicKey, Secp256k1, SecretKey, Signature};

pub struct EcdsaCrypto;

impl CryptoProvider for EcdsaCrypto {
    fn sign(&self, private_key: &[u8], message: &[u8]) -> Vec<u8> {
        let secp = Secp256k1::signing_only();
        let sk = SecretKey::from_slice(private_key).expect("Invalid private key");
        let msg = Message::from_slice(message).expect("Invalid message");
        let sig = secp.sign(&msg, &sk);
        sig.serialize_compact().to_vec()
    }

    fn verify(&self, public_key: &[u8], message: &[u8], signature: &[u8]) -> bool {
        let secp = Secp256k1::verification_only();
        let pk = PublicKey::from_slice(public_key).expect("Invalid public key");
        let msg = Message::from_slice(message).expect("Invalid message");
        let sig = Signature::from_compact(signature).expect("Invalid signature");
        secp.verify(&msg, &sig, &pk).is_ok()
    }
}
