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

pub type Result<T> = std::result::Result<T, BlockchainError>;
