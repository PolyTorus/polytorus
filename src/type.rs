use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Block {
    pub timestamp: u128,
    pub transactions: Vec<Transaction>,
    pub previous_hash: String,
    pub hash: String,
    pub nonce: i32,
    pub height: i32,
    pub difficulty: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Transaction {
    pub id: String,
    pub vin: Vec<TXInput>,
    pub vout: Vec<TXOutput>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TXInput {
    pub txid: String,
    pub vout: i32,
    pub signature: Vec<u8>,
    pub public_key: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TXOutput {
    pub value: u64,
    pub public_key_hash: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct TXOutputs {
    pub outputs: Vec<TXOutput>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Wallet {
    pub secret_key: Vec<u8>,
    pub public_key: Vec<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum EncryptionType {
    ECDSA,
    FNDSA,
}

#[derive(Debug)]
pub struct Blockchain {
    pub tip: String,
    pub db: sled::Db,
}

pub struct UTXOSet {
    pub blockchain: Blockchain,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct VersionMessage {
    pub addr_from: String,
    pub best_height: i32,
    pub version: i32,
}

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct BlockMessage {
    pub addr_from: String,
    pub block: Block,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct TransactionMessage {
    pub addr_from: String,
    pub transaction: Transaction,
}


