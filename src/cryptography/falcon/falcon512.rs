use super::base_falcon;

pub type SecretKey = base_falcon::SecretKey<512>;
pub type PublicKey = base_falcon::PublicKey<512>;
pub type Signature = base_falcon::Signature<512>;

pub fn keygen(seed: [u8; 32]) -> (SecretKey, PublicKey) {
    base_falcon::keygen(seed)
}

pub fn sign(msg: &[u8], sk: &SecretKey) -> Signature {
    base_falcon::sign(msg, sk)
}

pub fn verify(msg: &[u8], sig: &Signature, pk: &PublicKey) -> bool {
    base_falcon::verify(msg, sig, pk)
}

#[cfg(test)]
mod tests {
    use super::*;
    use rand::{thread_rng, Rng};

    #[test]
    fn test_falcon512() {
        let mut rng = thread_rng();
        let msg : [u8; 5] = rng.gen();
        let (sk, pk) = keygen(rng.gen());
        let sig = sign(&msg, &sk);
        assert!(verify(&msg, &sig, &pk));
    }
}