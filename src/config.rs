/// Configuration for data directories
use std::env;
use std::path::{Path, PathBuf};
use std::sync::OnceLock;

const DEFAULT_DATA_DIR: &str = "data";

const DATA_DIR: &str = "DATA_DIR";
const BLOCK_DIR: &str = "BLOCK_DIR";
const UTXO_DIR: &str = "UTXO_DIR";
const WALLET_DIR: &str = "WALLET_DIR";

static CONFIG: OnceLock<Config> = OnceLock::new();

#[derive(Debug, Clone)]
pub struct Config {
    pub data_dir: PathBuf,
    pub blocks_dir: PathBuf,
    pub utxo_dir: PathBuf,
    pub wallets_dir: PathBuf,
}

impl Default for Config {
    fn default() -> Self {
        let data_dir = env::var(DATA_DIR)
            .map(PathBuf::from)
            .unwrap_or_else(|_| PathBuf::from(DEFAULT_DATA_DIR));

        // Allow specific overrides for subdirectories
        let blocks_dir = env::var(BLOCK_DIR)
            .map(PathBuf::from)
            .unwrap_or_else(|_| data_dir.join("blocks"));

        let utxo_dir = env::var(UTXO_DIR)
            .map(PathBuf::from)
            .unwrap_or_else(|_| data_dir.join("utxos"));

        let wallets_dir = env::var(WALLET_DIR)
            .map(PathBuf::from)
            .unwrap_or_else(|_| data_dir.join("wallets"));

        Self {
            data_dir,
            blocks_dir,
            utxo_dir,
            wallets_dir,
        }
    }
}

/// Legacy data context for compatibility
#[derive(Clone, Debug)]
pub struct DataContext {
    pub base_dir: PathBuf,
}

impl DataContext {
    pub fn new(base_dir: PathBuf) -> Self {
        Self { base_dir }
    }

    pub fn blocks_dir(&self) -> PathBuf {
        self.base_dir.join("blocks")
    }

    pub fn wallets_dir(&self) -> PathBuf {
        self.base_dir.join("wallets")
    }
    pub fn utxos_dir(&self) -> PathBuf {
        self.base_dir.join("utxos")
    }

    pub fn data_dir(&self) -> PathBuf {
        self.base_dir.clone()
    }
}

impl Default for DataContext {
    fn default() -> Self {
        Self {
            base_dir: PathBuf::from("data"),
        }
    }
}

pub fn get_config() -> &'static Config {
    CONFIG.get_or_init(Config::default)
}
