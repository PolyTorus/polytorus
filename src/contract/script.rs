use bincode::de;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

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

