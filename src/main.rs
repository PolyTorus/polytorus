use env_logger::Env;
use polytorus::command::cli::Cli;
use polytorus::config::{Config, init_config};
use std::env;
use std::path::Path;

#[actix_web::main]
async fn main() {
    env_logger::from_env(Env::default().default_filter_or("warning")).init();

    // 設定の初期化
    let mut config = Config::default();
    
    // 環境変数から設定を更新
    config.update_from_env();
    
    // 設定ファイルが存在する場合は読み込む
    let config_path = env::var("POLYTORUS_CONFIG_FILE").unwrap_or_else(|_| "config.toml".to_string());
    if Path::new(&config_path).exists() {
        if let Ok(file_config) = Config::from_file(&config_path) {
            config = file_config;
            // ファイルから読み込んだ後も環境変数を適用（環境変数が優先）
            config.update_from_env();
        }
    }
    
    // グローバル設定を初期化
    init_config(config);

    // CLIの実行
    let mut cli = Cli::new();
    if let Err(e) = cli.run().await {
        println!("Error: {}", e);
    }
}
