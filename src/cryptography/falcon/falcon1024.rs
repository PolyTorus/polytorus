use super::base_falcon;

pub type SecretKey = base_falcon::SecretKey<1024>;
pub type PublicKey = base_falcon::PublicKey<1024>;
pub type Signature = base_falcon::Signature<1024>;

pub fn keygen(seed: [u8; 32]) -> (SecretKey, PublicKey) {
    base_falcon::keygen(seed)
}

pub fn sign(msg: &[u8], sk: &SecretKey) -> Signature {
    base_falcon::sign(msg, sk)
}

pub fn verify(msg: &[u8], sig: &Signature, pk: &PublicKey) -> bool {
    base_falcon::verify(msg, sig, pk)
}
