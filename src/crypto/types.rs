#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncryptionType {
    ECDSA,
    FNDSA,
}

pub enum DecryptionType {
    ECDSA,
    FNDSA,
}
