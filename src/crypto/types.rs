use serde::{Deserialize, Serialize};
use std::fmt;

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub enum EncryptionType {
    ECDSA,
    FNDSA,
}

impl EncryptionType {
    pub fn from_code(code: &str) -> Option<EncryptionType> {
        match code {
            "ECDSA" => Some(EncryptionType::ECDSA),
            "FNDSA" => Some(EncryptionType::FNDSA),
            _ => None,
        }
    }

    pub fn to_code(&self) -> &str {
            match self {
                EncryptionType::ECDSA => "ECDSA",
                EncryptionType::FNDSA => "FNDSA",
        }
    }

    pub fn guess_from_pubkey_size(size: usize) -> Option<EncryptionType> {
        match size {
            33 => Some(EncryptionType::ECDSA),
            897 => Some(EncryptionType::FNDSA),
            _ => None,
        }
        
    }
}

impl fmt::Display for EncryptionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.to_code())
    }
    
}
