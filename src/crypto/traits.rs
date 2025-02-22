pub trait CryptoProvider {
    fn sign(&self, private_key: &[u8], message: &[u8]) -> Vec<u8>;
    fn verify(&self, public_key: &[u8], message: &[u8], signature: &[u8]) -> bool;
}