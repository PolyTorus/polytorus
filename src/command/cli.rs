//! cli process

use crate::blockchain::blockchain::*;
use crate::blockchain::utxoset::*;
use crate::crypto::fndsa::*;
use crate::crypto::transaction::*;
use crate::crypto::types::EncryptionType;
use crate::crypto::wallets::*;
use crate::network::server::Server;
use crate::webserver::webserver::WebServer;
use crate::Result;
use bitcoincash_addr::Address;
use clap::{App, Arg, ArgMatches};
use std::process::exit;
use std::vec;

pub struct Cli {}

impl Default for Cli {
    fn default() -> Self {
        Self::new()
    }
}

impl Cli {
    pub fn new() -> Cli {
        Cli {}
    }

    pub async fn run(&mut self) -> Result<()> {
        info!("run app");
        let matches = App::new("polytorus")
            .version(env!("CARGO_PKG_VERSION"))
            .author("quantumshiro")
            .about("post quantum blockchain")
            .subcommand(App::new("printchain").about("print all the chain blocks"))
            .subcommand(
                App::new("createwallet").about("create a wallet").arg(
                    Arg::from_usage("<encryption> 'encryption type'")
                        .possible_values(&["ECDSA", "FNDSA"])
                        .default_value("FNDSA")
                        .help("encryption type"),
                ),
            )
            .subcommand(App::new("listaddresses").about("list all addresses"))
            .subcommand(App::new("reindex").about("reindex UTXO"))
            .subcommand(App::new("server").about("run server"))
            .subcommand(
                App::new("startnode")
                    .about("start the node server")
                    .arg(Arg::from_usage("<port> 'the port server bind to locally'"))
                    .arg(
                        Arg::with_name("host")
                            .long("host")
                            .takes_value(true)
                            .default_value("0.0.0.0")
                            .help("the host IP to bind for inbound connections"),
                    )
                    .arg(
                        Arg::with_name("bootstrap")
                            .long("bootstrap")
                            .takes_value(true)
                            .help("the address of an existing node (host:port) to connect first"),
                    ),
            )
            .subcommand(
                App::new("startminer")
                    .about("start the minner server")
                    .arg(Arg::from_usage("<port> 'the port server bind to locally'"))
                    .arg(Arg::from_usage("<address> 'wallet address'")),
            )
            .subcommand(
                App::new("getbalance")
                    .about("get balance in the blockchain")
                    .arg(Arg::from_usage(
                        "<address> 'The address to get balance for'",
                    )),
            )
            .subcommand(App::new("createblockchain").about("create blockchain").arg(
                Arg::from_usage("<address> 'The address to send genesis block reward to'"),
            ))
            .subcommand(
                App::new("send")
                    .about("send in the blockchain")
                    .arg(Arg::from_usage("<from> 'Source wallet address'"))
                    .arg(Arg::from_usage("<to> 'Destination wallet address'"))
                    .arg(Arg::from_usage("<amount> 'Amount to send'"))
                    .arg(Arg::from_usage(
                        "-m --mine 'the from address mine immediately'",
                    ))
                    .arg(
                        Arg::with_name("node")
                            .long("node")
                            .takes_value(true)
                            .help("Address of target node (e.g., 54.123.45.67:7000)"),
                    ),
            )
            .subcommand(
                App::new("remotesend")
                    .about("send transaction using remote wallet")
                    .arg(Arg::from_usage(
                        "<from> 'Source wallet address on remote node'",
                    ))
                    .arg(Arg::from_usage("<to> 'Destination wallet address'"))
                    .arg(Arg::from_usage("<amount> 'Amount to send'"))
                    .arg(Arg::from_usage("<node> 'Remote node address (host:port)'"))
                    .arg(Arg::from_usage(
                        "-m --mine 'mine immediately on the remote node'",
                    )),
            )
            .get_matches();

        match matches.subcommand() {
            ("getbalance", Some(sub_m)) => {
                if let Some(address) = sub_m.value_of("address") {
                    let balance = cmd_get_balance(address)?;
                    println!("Balance: {}\n", balance);
                }
            }
            ("createwallet", Some(sub_m)) => {
                let encryption = sub_m.value_of("encryption").unwrap().trim();
                let encryption: EncryptionType = match encryption {
                    "ECDSA" => EncryptionType::ECDSA,
                    "FNDSA" => EncryptionType::FNDSA,
                    _ => EncryptionType::FNDSA,
                };
                println!("address: {}", cmd_create_wallet(encryption)?);
            }
            ("printchain", Some(_)) => {
                cmd_print_chain()?;
            }
            ("reindex", Some(_)) => {
                let count = cmd_reindex()?;
                println!("Done! There are {} transactions in the UTXO set.", count);
            }
            ("listaddresses", Some(_)) => {
                cmd_list_address()?;
            }
            ("server", Some(_)) => {
                cmd_server().await?;
            }
            ("createblockchain", Some(sub_m)) => {
                if let Some(address) = sub_m.value_of("address") {
                    cmd_create_blockchain(address)?;
                }
            }
            ("send", Some(sub_m)) => {
                let from = get_value("from", sub_m)?;
                let to = get_value("to", sub_m)?;
                let amount: i32 = if let Some(amount) = sub_m.value_of("amount") {
                    amount.parse()?
                } else {
                    error_start_miner("amount", sub_m.usage())
                };
                let target_node = sub_m.value_of("node");
                cmd_send(from, to, amount, sub_m.is_present("mine"), target_node)?;
            }
            ("startnode", Some(sub_m)) => {
                if let Some(port) = sub_m.value_of("port") {
                    println!("Start node...");
                    let bc = Blockchain::new()?;
                    let utxo_set = UTXOSet { blockchain: bc };
                    let server = Server::new(
                        sub_m.value_of("host").unwrap_or("0.0.0.0"),
                        port,
                        "",
                        sub_m.value_of("bootstrap"),
                        utxo_set,
                    )?;
                    server.start_server()?;
                }
            }
            ("startminer", Some(sub_m)) => {
                let mining_address = get_value("address", sub_m)?;
                let port = get_value("port", sub_m)?;
                println!("Start miner node...");
                let bc = Blockchain::new()?;
                let utxo_set = UTXOSet { blockchain: bc };
                let server = Server::new(
                    sub_m.value_of("host").unwrap_or("0.0.0.0"),
                    port,
                    mining_address,
                    sub_m.value_of("bootstrap"),
                    utxo_set,
                )?;
                server.start_server()?;
            }
            ("remotesend", Some(sub_m)) => {
                let from = sub_m.value_of("from").unwrap();
                let to = sub_m.value_of("to").unwrap();
                let amount: i32 = sub_m.value_of("amount").unwrap().parse()?;
                let node = sub_m.value_of("node").unwrap();
                let mine = sub_m.is_present("mine");
                cmd_remote_send(from, to, amount, node, mine)?;
            }
            _ => {}
        }

        Ok(())
    }
}

async fn cmd_server() -> Result<()> {
    WebServer::new().await?;

    Ok(())
}

fn cmd_send(
    from: &str,
    to: &str,
    amount: i32,
    mine_now: bool,
    target_node: Option<&str>,
) -> Result<()> {
    let bc = Blockchain::new()?;
    let mut utxo_set = UTXOSet { blockchain: bc };
    let wallets = Wallets::new()?;
    let wallet = wallets.get_wallet(from).unwrap();
    // TODO: 暗号化方式を選択
    let crypto = FnDsaCrypto;
    let tx = Transaction::new_UTXO(wallet, to, amount, &utxo_set, &crypto)?;
    if mine_now {
        let cbtx = Transaction::new_coinbase(from.to_string(), String::from("reward!"))?;
        let new_block = utxo_set.blockchain.mine_block(vec![cbtx, tx])?;

        utxo_set.update(&new_block)?;
    } else {
        Server::send_transaction(&tx, utxo_set, target_node.unwrap_or("0.0.0.0:7000"))?;
    }

    println!("success!");
    Ok(())
}

fn get_value<'a>(name: &str, matches: &'a ArgMatches<'_>) -> Result<&'a str> {
    if let Some(value) = matches.value_of(name) {
        Ok(value)
    } else {
        error_start_miner(name, matches.usage())
    }
}

fn error_start_miner(name: &str, usage: &str) -> ! {
    println!("{} not supply!: usage\n{}", name, usage);

    exit(1)
}

pub fn cmd_create_wallet(encryption: EncryptionType) -> Result<String> {
    let mut ws = Wallets::new()?;
    let address = ws.create_wallet(encryption);
    ws.save_all()?;
    Ok(address)
}

fn cmd_reindex() -> Result<i32> {
    let bc = Blockchain::new()?;
    let utxo_set = UTXOSet { blockchain: bc };
    utxo_set.reindex()?;
    utxo_set.count_transactions()
}

fn cmd_create_blockchain(address: &str) -> Result<()> {
    let address = String::from(address);
    let bc = Blockchain::create_blockchain(address)?;

    let utxo_set = UTXOSet { blockchain: bc };
    utxo_set.reindex()?;
    println!("create blockchain");
    Ok(())
}

fn cmd_get_balance(address: &str) -> Result<i32> {
    let pub_key_hash = Address::decode(address).unwrap().body;
    let bc = Blockchain::new()?;
    let utxo_set = UTXOSet { blockchain: bc };
    let utxos = utxo_set.find_UTXO(&pub_key_hash)?;

    let balance = utxos.outputs.iter().map(|out| out.value).sum();
    Ok(balance)
}

pub fn cmd_print_chain() -> Result<()> {
    let bc = Blockchain::new()?;
    for b in bc.iter() {
        println!("{:#?}", b);
    }
    Ok(())
}

fn cmd_list_address() -> Result<()> {
    let ws = Wallets::new()?;
    let addresses = ws.get_all_addresses();
    println!("addresses: ");
    for ad in addresses {
        println!("{}", ad);
    }
    Ok(())
}

fn cmd_remote_send(from: &str, to: &str, amount: i32, node: &str, _mine_now: bool) -> Result<()> {
    let bc = Blockchain::new()?;
    let utxo_set = UTXOSet { blockchain: bc };

    let tx = Transaction {
        id: String::new(),
        vin: Vec::new(),
        vout: vec![TXOutput::new(amount, to.to_string())?],
    };

    let server = Server::new("0.0.0.0", "0", "", None, utxo_set)?;

    let signed_tx = server.send_sign_request(node, from, &tx)?;

    server.send_tx(node, &signed_tx)?;

    println!("Transaction sent successfully!");
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_helpers::{create_test_context, TestContextGuard};
    use crate::crypto::wallets::Wallets;
    use crate::blockchain::blockchain::Blockchain;
    use crate::blockchain::utxoset::UTXOSet;
    use crate::crypto::fndsa::FnDsaCrypto;

    type TestResult = std::result::Result<(), Box<dyn std::error::Error>>;

    #[test]
    fn test_cli_send_with_mine() -> TestResult {
        // テスト用のコンテキストを作成
        let context = create_test_context();
        let _guard = TestContextGuard::new(context.clone());
        
        // ウォレットを作成
        let mut wallets = Wallets::new_with_context(context.clone())?;
        let addr1 = wallets.create_wallet(EncryptionType::FNDSA);
        let addr2 = wallets.create_wallet(EncryptionType::FNDSA);
        wallets.save_all()?;
        
        // ジェネシスブロック作成
        let bc = Blockchain::create_blockchain_with_context(addr1.clone(), context.clone())?;
        
        // UTXOセットを作成し、インデックスを再構築
        let mut utxo_set = UTXOSet { blockchain: bc };
        utxo_set.reindex()?;
        
        // 残高確認
        let pub_key_hash1 = Address::decode(&addr1).unwrap().body;
        let pub_key_hash2 = Address::decode(&addr2).unwrap().body;
        
        let utxos1 = utxo_set.find_UTXO(&pub_key_hash1)?;
        let balance1: i32 = utxos1.outputs.iter().map(|out| out.value).sum();
        let utxos2 = utxo_set.find_UTXO(&pub_key_hash2)?;
        let balance2: i32 = utxos2.outputs.iter().map(|out| out.value).sum();
        
        assert_eq!(balance1, 10);
        assert_eq!(balance2, 0);

        // 送金と採掘
        let wallet1 = wallets.get_wallet(&addr1).unwrap();
        let crypto = FnDsaCrypto;
        let tx = Transaction::new_UTXO(wallet1, &addr2, 5, &utxo_set, &crypto)?;
        let cbtx = Transaction::new_coinbase(addr1.clone(), String::from("reward!"))?;
        
        // ブロックを採掘（既存のblockchainを使用）
        let new_block = utxo_set.blockchain.mine_block(vec![cbtx, tx])?;
        utxo_set.update(&new_block)?;

        // 採掘後の残高確認
        let utxos1_after = utxo_set.find_UTXO(&pub_key_hash1)?;
        let balance1_after: i32 = utxos1_after.outputs.iter().map(|out| out.value).sum();
        let utxos2_after = utxo_set.find_UTXO(&pub_key_hash2)?;
        let balance2_after: i32 = utxos2_after.outputs.iter().map(|out| out.value).sum();
        
        assert_eq!(balance1_after, 15); // 10 (initial) - 5 (sent) + 10 (mining reward)
        assert_eq!(balance2_after, 5);

        // addr2 から addr1 へ、残高以上（15 単位）の送金を試みる
        let wallet2 = wallets.get_wallet(&addr2).unwrap();
        let res = Transaction::new_UTXO(wallet2, &addr1, 15, &utxo_set, &crypto);
        assert!(res.is_err());

        // 再度残高確認（変化はないはず）
        let utxos1_final = utxo_set.find_UTXO(&pub_key_hash1)?;
        let balance1_final: i32 = utxos1_final.outputs.iter().map(|out| out.value).sum();
        let utxos2_final = utxo_set.find_UTXO(&pub_key_hash2)?;
        let balance2_final: i32 = utxos2_final.outputs.iter().map(|out| out.value).sum();
        
        assert_eq!(balance1_final, 15);
        assert_eq!(balance2_final, 5);

        Ok(())
    }

    #[test]
    fn test_cli_send_with_target_node() -> TestResult {
        let context = create_test_context();
        let _guard = TestContextGuard::new(context.clone());
        
        let mut wallets = Wallets::new_with_context(context.clone())?;
        let addr1 = wallets.create_wallet(EncryptionType::FNDSA);
        let addr2 = wallets.create_wallet(EncryptionType::FNDSA);
        wallets.save_all()?;
        
        let bc = Blockchain::create_blockchain_with_context(addr1.clone(), context.clone())?;
        let utxo_set = UTXOSet { blockchain: bc };
        utxo_set.reindex()?;
        
        let pub_key_hash1 = Address::decode(&addr1).unwrap().body;
        let pub_key_hash2 = Address::decode(&addr2).unwrap().body;
        
        let utxos1 = utxo_set.find_UTXO(&pub_key_hash1)?;
        let balance1: i32 = utxos1.outputs.iter().map(|out| out.value).sum();
        let utxos2 = utxo_set.find_UTXO(&pub_key_hash2)?;
        let balance2: i32 = utxos2.outputs.iter().map(|out| out.value).sum();
        
        assert_eq!(balance1, 10);
        assert_eq!(balance2, 0);

        // ネットワーク機能のテストは実際のノードが必要なため省略
        
        Ok(())
    }
}

