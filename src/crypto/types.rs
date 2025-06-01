use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EncryptionType {
    ECDSA,
    FNDSA,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DecryptionType {
    ECDSA,
    FNDSA,
}
