use bincode::de;
use serde::{Deserialize, Serialize};
use std::{collections::HashMap, hash::Hash};

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ScriptType {
    Basic,
    Wasm,
}

// define a script
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Script {
    pub script_type: ScriptType,
    pub code: Vec<u8>,
    pub hash: String,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub enum ValueType {
    Integer(i64),
    Boolean(bool),
    String(String),
    Bytes(Vec<u8>),
    List(Vec<Value>),
    Map(HashMap<String, Value>),
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct Value {
    pub value_type: ValueType,
}
