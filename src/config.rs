use serde::{Deserialize, Serialize};
use std::env;
use std::fs;
use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::RwLock;

use crate::Result;
use lazy_static::lazy_static;

/// デフォルトのデータディレクトリのベース名
const DEFAULT_DATA_DIR: &str = "data";
/// 設定ファイルのデフォルト名
const CONFIG_FILE_NAME: &str = "config.toml";

/// アプリケーション設定を保持する構造体
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// ベースとなるデータディレクトリ
    pub data_dir: String,
    /// ブロックチェーンデータのディレクトリ
    pub blocks_dir: String,
    /// UTXOデータのディレクトリ
    pub utxos_dir: String,
    /// ウォレットデータのディレクトリ
    pub wallets_dir: String,
}

impl Default for Config {
    fn default() -> Self {
        let data_dir = DEFAULT_DATA_DIR.to_string();
        
        Config {
            data_dir: data_dir.clone(),
            blocks_dir: format!("{}/blocks", data_dir),
            utxos_dir: format!("{}/utxos", data_dir),
            wallets_dir: format!("{}/wallets", data_dir),
        }
    }
}

impl Config {
    /// 現在のディレクトリに対する相対パスを絶対パスに変換
    pub fn absolute_path(&self, relative_path: &str) -> PathBuf {
        if Path::new(relative_path).is_absolute() {
            PathBuf::from(relative_path)
        } else {
            let current_dir = env::current_dir().unwrap_or_else(|_| PathBuf::from("."));
            current_dir.join(relative_path)
        }
    }

    /// ブロックチェーンデータの絶対パスを取得
    pub fn blocks_path(&self) -> PathBuf {
        self.absolute_path(&self.blocks_dir)
    }

    /// UTXOデータの絶対パスを取得
    pub fn utxos_path(&self) -> PathBuf {
        self.absolute_path(&self.utxos_dir)
    }

    /// ウォレットデータの絶対パスを取得
    pub fn wallets_path(&self) -> PathBuf {
        self.absolute_path(&self.wallets_dir)
    }

    /// 設定ファイルから設定を読み込む
    pub fn from_file(path: &str) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let config: Config = toml::from_str(&content)?;
        Ok(config)
    }

    /// 設定をファイルに保存
    pub fn save_to_file(&self, path: &str) -> Result<()> {
        let content = toml::to_string_pretty(self)?;
        fs::write(path, content)?;
        Ok(())
    }

    /// 環境変数から設定を更新
    pub fn update_from_env(&mut self) {
        if let Ok(data_dir) = env::var("POLYTORUS_DATA_DIR") {
            self.data_dir = data_dir.clone();
            // 基本ディレクトリが変更された場合、他のディレクトリもデフォルトで更新
            if self.blocks_dir == format!("{}/blocks", DEFAULT_DATA_DIR) {
                self.blocks_dir = format!("{}/blocks", data_dir);
            }
            if self.utxos_dir == format!("{}/utxos", DEFAULT_DATA_DIR) {
                self.utxos_dir = format!("{}/utxos", data_dir);
            }
            if self.wallets_dir == format!("{}/wallets", DEFAULT_DATA_DIR) {
                self.wallets_dir = format!("{}/wallets", data_dir);
            }
        }

        // 個別のディレクトリも環境変数から上書き可能
        if let Ok(blocks_dir) = env::var("POLYTORUS_BLOCKS_DIR") {
            self.blocks_dir = blocks_dir;
        }
        if let Ok(utxos_dir) = env::var("POLYTORUS_UTXOS_DIR") {
            self.utxos_dir = utxos_dir;
        }
        if let Ok(wallets_dir) = env::var("POLYTORUS_WALLETS_DIR") {
            self.wallets_dir = wallets_dir;
        }
    }

    /// コマンドライン引数から設定を更新
    pub fn update_from_args(&mut self, args: &clap::ArgMatches) {
        if let Some(data_dir) = args.value_of("data-dir") {
            self.data_dir = data_dir.to_string();
            // 基本ディレクトリが変更された場合、他のディレクトリもデフォルトで更新
            if self.blocks_dir == format!("{}/blocks", DEFAULT_DATA_DIR) {
                self.blocks_dir = format!("{}/blocks", data_dir);
            }
            if self.utxos_dir == format!("{}/utxos", DEFAULT_DATA_DIR) {
                self.utxos_dir = format!("{}/utxos", data_dir);
            }
            if self.wallets_dir == format!("{}/wallets", DEFAULT_DATA_DIR) {
                self.wallets_dir = format!("{}/wallets", data_dir);
            }
        }

        // 個別のディレクトリもコマンドライン引数から上書き可能
        if let Some(blocks_dir) = args.value_of("blocks-dir") {
            self.blocks_dir = blocks_dir.to_string();
        }
        if let Some(utxos_dir) = args.value_of("utxos-dir") {
            self.utxos_dir = utxos_dir.to_string();
        }
        if let Some(wallets_dir) = args.value_of("wallets-dir") {
            self.wallets_dir = wallets_dir.to_string();
        }
    }
    
    /// テスト用の設定を作成（テストでのみ使用）
    #[cfg(test)]
    pub fn new_test_config() -> Self {
        use uuid::Uuid;
        
        // テスト実行ごとに一意のディレクトリを生成
        let test_id = Uuid::new_v4();
        let test_dir = format!("test_data_{}", test_id);
        
        // テストディレクトリが存在しない場合は作成
        let _ = fs::create_dir_all(&test_dir);
        
        Config {
            data_dir: test_dir.clone(),
            blocks_dir: format!("{}/blocks", test_dir),
            utxos_dir: format!("{}/utxos", test_dir),
            wallets_dir: format!("{}/wallets", test_dir),
        }
    }
    
    /// テスト後にテストディレクトリを削除
    #[cfg(test)]
    pub fn cleanup_test_dir(&self) {
        // テストデータディレクトリのみを削除（通常のデータは削除しない）
        if self.data_dir.starts_with("test_data_") {
            let _ = fs::remove_dir_all(&self.data_dir);
        }
    }
}

// グローバルな設定オブジェクト
lazy_static::lazy_static! {
    static ref GLOBAL_CONFIG: RwLock<Config> = RwLock::new(Config::default());
}

/// グローバル設定を初期化
pub fn init_config(config: Config) {
    let mut global_config = GLOBAL_CONFIG.write().unwrap();
    *global_config = config;
}

/// グローバル設定の参照を取得
pub fn get_config() -> Arc<Config> {
    let config = GLOBAL_CONFIG.read().unwrap();
    Arc::new(config.clone())
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::fs;
    use std::path::Path;
    
    // テスト終了時にテストディレクトリをクリーンアップする
    struct TestCleanup {
        config: Config,
    }
    
    impl Drop for TestCleanup {
        fn drop(&mut self) {
            self.config.cleanup_test_dir();
        }
    }
    
    #[test]
    fn test_default_config() {
        let config = Config::default();
        assert_eq!(config.data_dir, "data");
        assert_eq!(config.blocks_dir, "data/blocks");
        assert_eq!(config.utxos_dir, "data/utxos");
        assert_eq!(config.wallets_dir, "data/wallets");
    }
    
    #[test]
    fn test_absolute_path() {
        let config = Config::default();
        
        // 相対パスをテスト
        let rel_path = "data/blocks";
        let abs_path = config.absolute_path(rel_path);
        let current_dir = env::current_dir().unwrap();
        assert_eq!(abs_path, current_dir.join(rel_path));
        
        // 絶対パスをテスト
        #[cfg(unix)]
        {
            let abs_path_str = "/tmp/data";
            let abs_path = config.absolute_path(abs_path_str);
            assert_eq!(abs_path, Path::new(abs_path_str));
        }
        
        #[cfg(windows)]
        {
            let abs_path_str = "C:\\tmp\\data";
            let abs_path = config.absolute_path(abs_path_str);
            assert_eq!(abs_path, Path::new(abs_path_str));
        }
    }
    
    #[test]
    fn test_save_and_load_config() {
        let config = Config::new_test_config();
        let _cleanup = TestCleanup { config: config.clone() };
        
        // ディレクトリが存在することを確認
        fs::create_dir_all(&config.data_dir).unwrap();
        
        // 設定ファイルを保存
        let config_path = format!("{}/test_config.toml", config.data_dir);
        config.save_to_file(&config_path).unwrap();
        
        // 設定ファイルが存在することを確認
        assert!(Path::new(&config_path).exists());
        
        // 設定ファイルを読み込み
        let loaded_config = Config::from_file(&config_path).unwrap();
        
        // 元の設定と一致することを確認
        assert_eq!(loaded_config.data_dir, config.data_dir);
        assert_eq!(loaded_config.blocks_dir, config.blocks_dir);
        assert_eq!(loaded_config.utxos_dir, config.utxos_dir);
        assert_eq!(loaded_config.wallets_dir, config.wallets_dir);
    }
    
    #[test]
    fn test_update_from_env() {
        // 元の環境変数を保存
        let orig_data_dir = env::var("POLYTORUS_DATA_DIR").ok();
        let orig_blocks_dir = env::var("POLYTORUS_BLOCKS_DIR").ok();
        
        // テスト用の環境変数を設定
        env::set_var("POLYTORUS_DATA_DIR", "/tmp/test_data");
        env::set_var("POLYTORUS_BLOCKS_DIR", "/tmp/custom_blocks");
        
        let mut config = Config::default();
        config.update_from_env();
        
        // 環境変数から値が更新されたことを確認
        assert_eq!(config.data_dir, "/tmp/test_data");
        assert_eq!(config.blocks_dir, "/tmp/custom_blocks");
        
        // 元の環境変数を復元
        match orig_data_dir {
            Some(val) => env::set_var("POLYTORUS_DATA_DIR", val),
            None => env::remove_var("POLYTORUS_DATA_DIR"),
        }
        
        match orig_blocks_dir {
            Some(val) => env::set_var("POLYTORUS_BLOCKS_DIR", val),
            None => env::remove_var("POLYTORUS_BLOCKS_DIR"),
        }
    }
    
    #[test]
    fn test_global_config() {
        let test_config = Config::new_test_config();
        let _cleanup = TestCleanup { config: test_config.clone() };
        
        // グローバル設定を初期化
        init_config(test_config.clone());
        
        // グローバル設定を取得
        let global_config = get_config();
        
        // 期待通りの値であることを確認
        assert_eq!(global_config.data_dir, test_config.data_dir);
        assert_eq!(global_config.blocks_dir, test_config.blocks_dir);
        assert_eq!(global_config.utxos_dir, test_config.utxos_dir);
        assert_eq!(global_config.wallets_dir, test_config.wallets_dir);
    }
    
    #[test]
    fn test_test_config() {
        // テスト用の設定を作成
        let config1 = Config::new_test_config();
        let _cleanup1 = TestCleanup { config: config1.clone() };
        
        // 別のテスト用の設定を作成
        let config2 = Config::new_test_config();
        let _cleanup2 = TestCleanup { config: config2.clone() };
        
        // 2つの設定が異なることを確認
        assert_ne!(config1.data_dir, config2.data_dir);
        assert_ne!(config1.blocks_dir, config2.blocks_dir);
        assert_ne!(config1.utxos_dir, config2.utxos_dir);
        assert_ne!(config1.wallets_dir, config2.wallets_dir);
        
        // ディレクトリが存在することを確認
        assert!(Path::new(&config1.data_dir).exists());
        assert!(Path::new(&config2.data_dir).exists());
    }
    
    #[test]
    fn test_cleanup() {
        let config = Config::new_test_config();
        
        // ディレクトリが存在することを確認
        assert!(Path::new(&config.data_dir).exists());
        
        {
            // スコープを作成してクリーンアップをトリガー
            let _cleanup = TestCleanup { config: config.clone() };
        }
        
        // スコープを抜けた後にディレクトリが削除されていることを確認
        assert!(!Path::new(&config.data_dir).exists());
    }
}