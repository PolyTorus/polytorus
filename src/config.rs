use crate::Result;
use crate::crypto::types::EncryptionType;
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};
use std::str::FromStr;

pub const DEFAULT_CONFIG_PATH: &str = "setting.toml";

impl Serialize for EncryptionType {
    fn serialize<S>(&self, serializer: S) -> std::result::Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        match self {
            EncryptionType::ECDSA => serializer.serialize_str("ECDSA"),
            EncryptionType::FNDSA => serializer.serialize_str("FNDSA"),
        }
    }
}

impl<'de> Deserialize<'de> for EncryptionType {
    fn deserialize<D>(deserializer: D) -> std::result::Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        EncryptionType::from_str(&s).map_err(|_| serde::de::Error::custom("invalid encryption type"))
    }
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct WalletConfig {
    #[serde(default = "default_encryption")]
    pub default_encryption: EncryptionType,

    #[serde(default = "default_coinbase_message")]
    pub coinbase_message: String,
}

fn default_encryption() -> EncryptionType {
    EncryptionType::ECDSA
}

fn default_coinbase_message() -> String {
    String::from("Default coinbase message")
}

