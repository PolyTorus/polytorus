// src/command/cli.rs
//! CLIプロセス

use crate::blockchain::blockchain::*;
use crate::blockchain::utxoset::*;
use crate::crypto::fndsa::*;
use crate::crypto::ecdsa::*;
use crate::crypto::traits::CryptoProvider;
use crate::crypto::transaction::*;
use crate::crypto::types::CryptoType;
use crate::crypto::wallets::*;
use crate::network::server::Server;
use crate::webserver::webserver::WebServer;
use crate::types::{Address, BlockHash, TransactionId};
use crate::errors::{BlockchainError, Result};
use crate::config::{AppConfig, BlockchainConfig, NetworkConfig, WalletConfig};
use bitcoincash_addr::Address as BcAddress;
use clap::{App, Arg, ArgMatches};
use std::process::exit;
use std::vec;

/// コマンドラインインターフェース
pub struct Cli {
    /// アプリケーション設定
    config: AppConfig,
}

impl Default for Cli {
    fn default() -> Self {
        Self::new()
    }
}

impl Cli {
    /// 新しいCLIインスタンスを作成
    pub fn new() -> Cli {
        Cli {
            config: AppConfig::default(),
        }
    }

    /// カスタム設定でCLIインスタンスを作成
    pub fn with_config(config: AppConfig) -> Cli {
        Cli { config }
    }

    /// CLIを実行
    pub async fn run(&mut self) -> Result<()> {
        info!("アプリを実行");
        
        let matches = App::new("polytorus")
            .version(env!("CARGO_PKG_VERSION"))
            .author("quantumshiro")
            .about("post quantum blockchain")
            .subcommand(App::new("printchain").about("チェーンの全ブロックを表示"))
            .subcommand(App::new("createwallet").about("ウォレットを作成")
                .arg(Arg::from_usage("<encryption> '暗号化タイプ'")
                    .possible_values(&["ECDSA", "FNDSA"])
                    .default_value("FNDSA")
                    .help("暗号化タイプ")))
            .subcommand(App::new("listaddresses").about("全アドレスをリスト"))
            .subcommand(App::new("reindex").about("UTXOを再インデックス"))
            .subcommand(App::new("server").about("サーバーを実行"))
            .subcommand(
                App::new("startnode")
                    .about("ノードサーバーを開始")
                    .arg(Arg::from_usage("<port> 'ローカルにバインドするポート'"))
                    .arg(
                        Arg::with_name("host")
                            .long("host")
                            .takes_value(true)
                            .default_value("0.0.0.0")
                            .help("着信接続をバインドするホストIP"),
                    )
                    .arg(
                        Arg::with_name("bootstrap")
                            .long("bootstrap")
                            .takes_value(true)
                            .help("最初に接続する既存のノードのアドレス (host:port)"),
                    ),
            )
            .subcommand(
                App::new("startminer")
                    .about("マイナーサーバーを開始")
                    .arg(Arg::from_usage("<port> 'ローカルにバインドするポート'"))
                    .arg(Arg::from_usage("<address> 'ウォレットアドレス'")),
            )
            .subcommand(
                App::new("getbalance")
                    .about("ブロックチェーンの残高を取得")
                    .arg(Arg::from_usage(
                        "<address> '残高を取得するアドレス'",
                    )),
            )
            .subcommand(App::new("createblockchain").about("ブロックチェーンを作成").arg(
                Arg::from_usage("<address> 'ジェネシスブロック報酬を送信するアドレス'"),
            ))
            .subcommand(
                App::new("send")
                    .about("ブロックチェーンで送金")
                    .arg(Arg::from_usage("<from> '送信元ウォレットアドレス'"))
                    .arg(Arg::from_usage("<to> '送信先ウォレットアドレス'"))
                    .arg(Arg::from_usage("<amount> '送金額'"))
                    .arg(Arg::from_usage(
                        "-m --mine '送信元アドレスがすぐに採掘'",
                    ))
                    .arg(
                        Arg::with_name("node")
                            .long("node")
                            .takes_value(true)
                            .help("ターゲットノードのアドレス（例: 54.123.45.67:7000）"),
                    ),
            )
            .subcommand(
                App::new("remotesend")
                    .about("リモートウォレットを使用してトランザクションを送信")
                    .arg(Arg::from_usage(
                        "<from> 'リモートノード上の送信元ウォレットアドレス'",
                    ))
                    .arg(Arg::from_usage("<to> '送信先ウォレットアドレス'"))
                    .arg(Arg::from_usage("<amount> '送金額'"))
                    .arg(Arg::from_usage("<node> 'リモートノードアドレス（host:port）'"))
                    .arg(Arg::from_usage(
                        "-m --mine 'リモートノードですぐに採掘'",
                    )),
            )
            .get_matches();

        if let Some(matches) = matches.subcommand_matches("getbalance") {
            if let Some(address_str) = matches.value_of("address") {
                let address = Address(address_str.to_string());
                let balance = self.cmd_get_balance(&address)?;
                println!("残高: {}\n", balance);
            }
        } else if let Some(ref matches) = matches.subcommand_matches("createwallet") {
            let encryption_str = matches.value_of("encryption").unwrap().trim();
            let encryption: CryptoType = match encryption_str {
                "ECDSA" => CryptoType::ECDSA,
                "FNDSA" => CryptoType::FNDSA,
                _ => CryptoType::FNDSA,
            };
            
            println!("アドレス: {}", self.cmd_create_wallet(encryption)?.as_str());
        } else if let Some(_) = matches.subcommand_matches("printchain") {
            self.cmd_print_chain()?;
        } else if matches.subcommand_matches("reindex").is_some() {
            let count = self.cmd_reindex()?;
            println!("完了！UTXOセットには{}件のトランザクションがあります。", count);
        } else if matches.subcommand_matches("listaddresses").is_some() {
            self.cmd_list_address()?;
        } else if let Some(_) = matches.subcommand_matches("server") {
            self.cmd_server().await?;
        } else if let Some(matches) = matches.subcommand_matches("createblockchain") {
            if let Some(address_str) = matches.value_of("address") {
                let address = Address(address_str.to_string());
                self.cmd_create_blockchain(address)?;
            }
        } else if let Some(matches) = matches.subcommand_matches("send") {
            let from_str = get_value("from", matches)?;
            let to_str = get_value("to", matches)?;
            let from = Address(from_str.to_string());
            let to = Address(to_str.to_string());

            let amount: i32 = if let Some(amount) = matches.value_of("amount") {
                amount.parse::<i32>().map_err(|e| BlockchainError::SerializationError(e.to_string()))?
            } else {
                error_start_miner("amount", matches.usage())
            };
            
            let target_node = matches.value_of("node");
            self.cmd_send(from, to, amount, matches.is_present("mine"), target_node)?;
        } else if let Some(ref matches) = matches.subcommand_matches("startnode") {
            if let Some(port) = matches.value_of("port") {
                println!("ノードを開始中...");
                let blockchain_config = self.config.blockchain.clone();
                let network_config = self.config.network.clone();
                
                let bc = Blockchain::new(blockchain_config.clone())?;
                let utxo_set = UTXOSet { blockchain: bc };
                
                let server = Server::new(
                    matches.value_of("host").unwrap_or("0.0.0.0"),
                    port,
                    &Address::empty(),
                    matches.value_of("bootstrap"),
                    utxo_set,
                    network_config,
                )?;
                
                server.start_server()?;
            }
        } else if let Some(matches) = matches.subcommand_matches("startminer") {
            let mining_address_str = get_value("address", matches)?;
            let mining_address = Address(mining_address_str.to_string());

            let port = get_value("port", matches)?;

            println!("マイナーノードを開始中...");
            let blockchain_config = self.config.blockchain.clone();
            let network_config = self.config.network.clone();
            
            let bc = Blockchain::new(blockchain_config.clone())?;
            let utxo_set = UTXOSet { blockchain: bc };
            
            let server = Server::new(
                matches.value_of("host").unwrap_or("0.0.0.0"),
                port,
                &mining_address,
                matches.value_of("bootstrap"),
                utxo_set,
                network_config,
            )?;
            
            server.start_server()?;
        } else if let Some(matches) = matches.subcommand_matches("remotesend") {
            let from_str = matches.value_of("from").unwrap();
            let to_str = matches.value_of("to").unwrap();
            let from = Address(from_str.to_string());
            let to = Address(to_str.to_string());
            
            let amount: i32 = matches.value_of("amount")
                .unwrap()
                .parse::<i32>()
                .map_err(|e| BlockchainError::SerializationError(e.to_string()))?;
                
            let node = matches.value_of("node").unwrap();
            let mine = matches.is_present("mine");

            self.cmd_remote_send(from, to, amount, node, mine)?;
        }

        Ok(())
    }
    
    /// Webサーバーを起動
    async fn cmd_server(&self) -> Result<()> {
        WebServer::new().await.map_err(|e| BlockchainError::Other(e.to_string()))?;
        Ok(())
    }

    /// 送金コマンド
    fn cmd_send(
        &self,
        from: Address,
        to: Address,
        amount: i32,
        mine_now: bool,
        target_node: Option<&str>,
    ) -> Result<()> {
        let blockchain_config = self.config.blockchain.clone();
        let network_config = self.config.network.clone();
        let wallet_config = self.config.wallet.clone();
        
        let bc = Blockchain::new(blockchain_config.clone())?;
        let mut utxo_set = UTXOSet { blockchain: bc };
        let wallets = Wallets::new(wallet_config)?;
        
        let wallet = wallets.get_wallet(&from).ok_or(
            BlockchainError::WalletNotFound(format!("ウォレットが見つかりません: {}", from))
        )?;
        
        // TODO: 暗号化方式を選択
        let crypto: Box<dyn CryptoProvider> = match wallet.public_key.key_type {
            CryptoType::FNDSA => Box::new(FnDsaCrypto),
            CryptoType::ECDSA => Box::new(EcdsaCrypto),
        };
        
        let tx = Transaction::new_UTXO(wallet, &to, amount, &utxo_set, crypto.as_ref())?;
        
        if mine_now {
            let cbtx = Transaction::new_coinbase(from, String::from("reward!"), &blockchain_config)?;
            let new_block = utxo_set.blockchain.mine_block(vec![cbtx, tx])?;
            utxo_set.update(&new_block)?;
        } else {
            Server::send_transaction(&tx, utxo_set, target_node.unwrap_or("0.0.0.0:7000"), network_config)?;
        }

        println!("送金成功！");
        Ok(())
    }

    /// ウォレット作成コマンド
    fn cmd_create_wallet(&self, encryption: CryptoType) -> Result<Address> {
        let wallet_config = self.config.wallet.clone();
        let mut ws = Wallets::new(wallet_config)?;
        
        let crypto: Box<dyn CryptoProvider> = match encryption {
            CryptoType::FNDSA => Box::new(FnDsaCrypto),
            CryptoType::ECDSA => Box::new(EcdsaCrypto),
        };
        
        let address = ws.create_wallet(crypto.as_ref(), encryption)?;
        ws.save_all()?;
        Ok(address)
    }

    /// UTXO再インデックスコマンド
    fn cmd_reindex(&self) -> Result<i32> {
        let blockchain_config = self.config.blockchain.clone();
        
        let bc = Blockchain::new(blockchain_config)?;
        let utxo_set = UTXOSet { blockchain: bc };
        utxo_set.reindex()?;
        utxo_set.count_transactions()
    }

    /// ブロックチェーン作成コマンド
    fn cmd_create_blockchain(&self, address: Address) -> Result<()> {
        let blockchain_config = self.config.blockchain.clone();
        
        let bc = Blockchain::create_blockchain(address, blockchain_config)?;
        let utxo_set = UTXOSet { blockchain: bc };
        utxo_set.reindex()?;
        
        println!("ブロックチェーンを作成しました");
        Ok(())
    }

    /// 残高取得コマンド
    fn cmd_get_balance(&self, address: &Address) -> Result<i32> {
        let pub_key_hash = BcAddress::decode(address.as_str())
            .map_err(|_| BlockchainError::InvalidTransaction("アドレスのデコードに失敗しました".to_string()))?
            .body;
        
        let blockchain_config = self.config.blockchain.clone();
        let bc = Blockchain::new(blockchain_config)?;
        let utxo_set = UTXOSet { blockchain: bc };
        let utxos = utxo_set.find_UTXO(&pub_key_hash)?;

        let balance = utxos.outputs.iter().map(|out| out.value).sum();
        Ok(balance)
    }

    /// ブロックチェーン表示コマンド
    fn cmd_print_chain(&self) -> Result<()> {
        let blockchain_config = self.config.blockchain.clone();
        let bc = Blockchain::new(blockchain_config)?;
        
        for b in bc.iter() {
            println!("{:#?}", b);
        }
        
        Ok(())
    }

    /// アドレス一覧表示コマンド
    fn cmd_list_address(&self) -> Result<()> {
        let wallet_config = self.config.wallet.clone();
        let ws = Wallets::new(wallet_config)?;
        let addresses = ws.get_all_addresses();
        
        println!("アドレス一覧: ");
        for ad in addresses {
            println!("{}", ad);
        }
        
        Ok(())
    }

    /// リモート送金コマンド
    fn cmd_remote_send(&self, from: Address, to: Address, amount: i32, node: &str, mine_now: bool) -> Result<()> {
        let blockchain_config = self.config.blockchain.clone();
        let network_config = self.config.network.clone();
        
        let bc = Blockchain::new(blockchain_config.clone())?;
        let utxo_set = UTXOSet { blockchain: bc };

        let tx = Transaction {
            id: TransactionId::empty(),
            vin: Vec::new(),
            vout: vec![TXOutput::new(amount, to)?],
        };

        let server = Server::new("0.0.0.0", "0", &Address::empty(), None, utxo_set, network_config)?;
        let signed_tx = server.send_sign_request(node, from.as_str(), &tx)?;
        server.send_tx(node, &signed_tx)?;

        println!("トランザクションを正常に送信しました！");
        Ok(())
    }
}

/// コマンドライン引数から値を取得
fn get_value<'a>(name: &str, matches: &'a ArgMatches<'_>) -> Result<&'a str> {
    if let Some(value) = matches.value_of(name) {
        Ok(value)
    } else {
        error_start_miner(name, matches.usage())
    }
}

/// マイナー起動エラー
fn error_start_miner(name: &str, usage: &str) -> ! {
    println!("{}が指定されていません: 使用方法\n{}", name, usage);
    exit(1)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::config::{AppConfig, BlockchainConfig, NetworkConfig, WalletConfig};

    /// ローカルで即時採掘を行うsendコマンドのテスト
    #[test]
    fn test_cli_send_with_mine() -> Result<()> {
        // テスト用の一時データディレクトリ
        let test_dir = "test_data_send_mine";
        std::fs::create_dir_all(test_dir).ok();
        
        // テスト用の設定
        let blockchain_config = BlockchainConfig {
            data_dir: test_dir.to_string(),
            ..BlockchainConfig::default()
        };
        
        let wallet_config = WalletConfig {
            data_dir: format!("{}/wallets", test_dir),
            ..WalletConfig::default()
        };
        
        let network_config = NetworkConfig::default();
        
        let app_config = AppConfig {
            blockchain: blockchain_config,
            wallet: wallet_config,
            network: network_config,
        };
        
        let cli = Cli::with_config(app_config);
        
        // 2つのウォレットを作成
        let addr1 = cli.cmd_create_wallet(CryptoType::FNDSA)?;
        let addr2 = cli.cmd_create_wallet(CryptoType::FNDSA)?;
        
        // ジェネシスブロック作成：addr1に初期報酬が入る
        cli.cmd_create_blockchain(addr1.clone())?;

        // 初期残高確認
        let balance1 = cli.cmd_get_balance(&addr1)?;
        let balance2 = cli.cmd_get_balance(&addr2)?;
        assert_eq!(balance1, 10); // 初期報酬は10
        assert_eq!(balance2, 0);

        // addr1からaddr2へ5単位送金（即時採掘モード）
        cli.cmd_send(addr1.clone(), addr2.clone(), 5, true, None)?;

        // 採掘が行われたので、残高が更新されるはず
        let balance1_after = cli.cmd_get_balance(&addr1)?;
        let balance2_after = cli.cmd_get_balance(&addr2)?;
        
        // 採掘報酬（10）+ 残り（5）= 15, 送金分（5）= 5
        assert_eq!(balance1_after, 15);
        assert_eq!(balance2_after, 5);

        // addr2からaddr1へ、残高以上（15単位）の送金を試みる → エラーとなるはず
        let res = cli.cmd_send(addr2.clone(), addr1.clone(), 15, true, None);
        assert!(res.is_err());

        // 再度残高確認（変化はないはず）
        let balance1_final = cli.cmd_get_balance(&addr1)?;
        let balance2_final = cli.cmd_get_balance(&addr2)?;
        assert_eq!(balance1_final, 15);
        assert_eq!(balance2_final, 5);

        // テスト用ディレクトリを削除
        std::fs::remove_dir_all(test_dir).ok();
        
        Ok(())
    }
}
