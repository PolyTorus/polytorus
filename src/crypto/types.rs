use serde::{Deserialize, Serialize};

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EncryptionType {
    ECDSA,
    FNDSA,
}

#[allow(clippy::upper_case_acronyms)]
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum DecryptionType {
    ECDSA,
    FNDSA,
}
