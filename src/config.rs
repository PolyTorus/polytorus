/// Configuration for data directories
use std::path::PathBuf;

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
    }    pub fn utxos_dir(&self) -> PathBuf {
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
