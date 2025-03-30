use std::fmt::{self, Display, Formatter};
use std::str::FromStr;
use serde::{Serialize, Deserialize};

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct BlockHash(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct TransactionId(pub String);

#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Address(pub String);

impl Display for TransactionId {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl Display for Address {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for BlockHash {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(BlockHash(s.to_string()))
    }
}

impl FromStr for TransactionId {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(TransactionId(s.to_string()))
    }
}

impl FromStr for Address {
    type Err = String;
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        Ok(Address(s.to_string()))
    }
}

impl BlockHash {
    pub fn empty() -> Self {
        BlockHash(String::new())
    }

    pub fn as_str(&self) -> &str {
        &self.0
    }

    pub fn is_empty(&self) -> bool {
        self.0.is_empty()
    }
}