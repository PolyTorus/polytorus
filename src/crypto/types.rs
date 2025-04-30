use std::fmt;
use std::str::FromStr;
use failure::{Fail, format_err};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum EncryptionType {
    ECDSA,
    FNDSA,
}

#[derive(Debug, Fail)]
pub enum CryptoError {
    #[fail(display = "Fail Encrypt type: {}", _0)]
    InvalidKeyLength(String),
    #[fail(display = "Fail Encrypt type: {}", _0)]
    InvalidKeyType(String),
}

impl fmt::Display for EncryptionType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            EncryptionType::ECDSA => write!(f, "ECDSA"),
            EncryptionType::FNDSA => write!(f, "FNDSA"),
        }
    }
}

