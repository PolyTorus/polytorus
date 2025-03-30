use core::fmt;
use std::io;

#[derive(Debug)]
pub enum BlockchainError {
    IoError(io::Error),
    SerializationError(String),
    DatabaseError(String),
    InvalidTransaction(String),
    InvalidBlock(String),
    InvalidSignature(String),
    WalletNotFound(String),
    InsufficientFunds { balance: i32, required: i32 },
    NetworkError(String),
    Other(String),
}

impl fmt::Display for BlockchainError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::IoError(e) => write!(f, "IO Error: {}", e),
            Self::SerializationError(e) => write!(f, "Serialization Error: {}", e),
            Self::DatabaseError(e) => write!(f, "Database Error: {}", e),
            Self::InvalidTransaction(e) => write!(f, "Invalid Transaction: {}", e),
            Self::InvalidBlock(e) => write!(f, "Invalid Block: {}", e),
            Self::InvalidSignature(e) => write!(f, "Invalid Signature: {}", e),
            Self::WalletNotFound(e) => write!(f, "Wallet Not Found: {}", e),
            Self::InsufficientFunds { balance, required } => {
                write!(f, "Insufficient Funds: balance {}, required {}", balance, required)
            }
            Self::NetworkError(e) => write!(f, "Network Error: {}", e),
            Self::Other(e) => write!(f, "Other Error: {}", e),
        }
    }
}

impl std::error::Error for BlockchainError {}

impl From<io::Error> for BlockchainError {
    fn from(err: io::Error) -> Self {
        Self::IoError(err)
    }
}

impl From<bincode::Error> for BlockchainError {
    fn from(err: bincode::Error) -> Self {
        Self::SerializationError(err.to_string())
    }
}

impl From<sled::Error> for BlockchainError {
    fn from(err: sled::Error) -> Self {
        Self::DatabaseError(err.to_string())
    }
}

impl From<std::string::FromUtf8Error> for BlockchainError {
    fn from(err: std::string::FromUtf8Error) -> Self {
        Self::SerializationError(err.to_string())
    }
}

impl From<std::num::ParseIntError> for BlockchainError {
    fn from(err: std::num::ParseIntError) -> Self {
        Self::SerializationError(err.to_string())
    }
}

impl From<std::time::SystemTimeError> for BlockchainError {
    fn from(err: std::time::SystemTimeError) -> Self {
        Self::Other(err.to_string())
    }
}

impl From<failure::Error> for BlockchainError {
    fn from(err: failure::Error) -> Self {
        Self::Other(err.to_string())
    }
}

pub type Result<T> = std::result::Result<T, BlockchainError>;
