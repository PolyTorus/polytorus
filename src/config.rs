use crate::crypto::types::CryptoType;

#[derive(Debug, Clone)]
pub struct BlockchainConfig {
    pub initial_difficulty: usize,
    pub desired_block_time_ms: u128,
    pub subsidy: i32,
    pub data_dir: String,
}

impl Default for BlockchainConfig {
    fn default() -> Self {
        Self {
            initial_difficulty: 4,
            desired_block_time_ms: 10_000,
            subsidy: 10,
            data_dir: String::from("data"),
        }
    }
}

#[derive(Debug, Clone)]
pub struct NetworkConfig {
    pub cmd_length: usize,
    pub version: i32,
    pub known_nodes: Vec<String>,
    pub default_port: u16,
    pub connection_timeout_ms: u64,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            cmd_length: 12,
            version: 1,
            known_nodes: vec![],
            default_port: 8333,
            connection_timeout_ms: 10_000,
        }
    }
}

#[derive(Debug, Clone)]
pub struct WalletConfig {
    pub data_dir: String,
    pub default_key_type: CryptoType,
}

impl Default for WalletConfig {
    fn default() -> Self {
        Self {
            data_dir: String::from("data/wallet"),
            default_key_type: CryptoType::FNDSA,
        }
    }
}

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub blockchain: BlockchainConfig,
    pub network: NetworkConfig,
    pub wallet: WalletConfig,
}

impl Default for AppConfig {
    fn default() -> Self {
        Self {
            blockchain: BlockchainConfig::default(),
            network: NetworkConfig::default(),
            wallet: WalletConfig::default(),
        }
    }
}

pub fn load_config() -> AppConfig {
    // Load the configuration from a file or environment variables
    // For now, we will just return the default configuration
    AppConfig::default()
}

pub fn create_config_for_environment(environment: &str) -> AppConfig {
    match environment {
        "development" => {
            AppConfig {
                blockchain: BlockchainConfig {
                    initial_difficulty: 2,
                    desired_block_time_ms: 5000,
                    subsidy: 5,
                    data_dir: String::from("data/dev"),
                },

                network: NetworkConfig {
                    cmd_length: 12,
                    version: 1,
                    known_nodes: vec!["127.0.0.1:8333".to_string()],
                    default_port: 8333,
                    connection_timeout_ms: 5000,
                },

                wallet: WalletConfig {
                    data_dir: String::from("data/dev/wallet"),
                    default_key_type: CryptoType::FNDSA,
                },
            }
        }

        "test" => {
            AppConfig {
                blockchain: BlockchainConfig {
                    initial_difficulty: 1,
                    desired_block_time_ms: 1_000,
                    subsidy: 10,
                    data_dir: String::from("data/test"),
                },

                network: NetworkConfig {
                    cmd_length: 12,
                    version: 1,
                    known_nodes: Vec::new(),
                    default_port: 8333,
                    connection_timeout_ms: 1000,
                },

                wallet: WalletConfig {
                    data_dir: String::from("data/test/wallet"),
                    default_key_type: CryptoType::FNDSA,
                },
            }
        }

        "production" | _ => {
            AppConfig::default()
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = AppConfig::default();
        
        // ブロックチェーン設定の確認
        assert_eq!(config.blockchain.initial_difficulty, 4);
        assert_eq!(config.blockchain.desired_block_time_ms, 10_000);
        assert_eq!(config.blockchain.subsidy, 10);
        assert_eq!(config.blockchain.data_dir, "data");
        
        // ネットワーク設定の確認
        assert_eq!(config.network.cmd_length, 12);
        assert_eq!(config.network.version, 1);
        assert_eq!(config.network.default_port, 7000);
        
        // ウォレット設定の確認
        assert_eq!(config.wallet.data_dir, "data/wallets");
        assert_eq!(config.wallet.default_key_type, CryptoType::FNDSA);
    }
    
    #[test]
    fn test_environment_specific_config() {
        // 開発環境の設定
        let dev_config = create_config_for_environment("development");
        assert_eq!(dev_config.blockchain.initial_difficulty, 2);
        assert_eq!(dev_config.blockchain.data_dir, "dev_data");
        
        // テスト環境の設定
        let test_config = create_config_for_environment("test");
        assert_eq!(test_config.blockchain.initial_difficulty, 1);
        assert_eq!(test_config.blockchain.data_dir, "test_data");
        
        // 本番環境の設定
        let prod_config = create_config_for_environment("production");
        assert_eq!(prod_config.blockchain.initial_difficulty, 4);
        assert_eq!(prod_config.blockchain.data_dir, "data");
    }
}
