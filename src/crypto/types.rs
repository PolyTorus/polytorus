use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CryptoType {
    ECDSA,
    FNDSA,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PublicKey {
    pub key_type: CryptoType,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrivateKey {
    pub key_type: CryptoType,
    pub data: Vec<u8>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct Signature {
    pub key_type: CryptoType,
    pub data: Vec<u8>,
}

impl PublicKey {
    pub fn new(key_type: CryptoType, data: Vec<u8>) -> Self {
        Self { key_type, data }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }
}

impl PrivateKey {
    pub fn new(key_type: CryptoType, data: Vec<u8>) -> Self {
        Self { key_type, data }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }
}

impl Signature {
    pub fn new(key_type: CryptoType, data: Vec<u8>) -> Self {
        Self { key_type, data }
    }

    pub fn as_bytes(&self) -> &[u8] {
        &self.data
    }
}
