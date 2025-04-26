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
                            .help("ターゲットノードのアドレス (例: 54.123.45.67:7000)"),
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

        if let Some(matches) = matches.subcommand_matches("getbalance") {
            if let Some(address) = matches.value_of("address") {
                let balance = cmd_get_balance(address)?;
                println!("Balance: {}\n", balance);
            }
        } else if let Some(matches) = matches.subcommand_matches("createwallet") {
            let encryption = matches.value_of("encryption").unwrap().trim();
            let encryption: EncryptionType = match encryption {
                "ECDSA" => EncryptionType::ECDSA,
                "FNDSA" => EncryptionType::FNDSA,
                _ => EncryptionType::FNDSA,
            };
            println!("address: {}", cmd_create_wallet(encryption)?);
        } else if matches.subcommand_matches("printchain").is_some() {
            cmd_print_chain()?;
        } else if matches.subcommand_matches("reindex").is_some() {
            let count = cmd_reindex()?;
            println!("Done! There are {} transactions in the UTXO set.", count);
        } else if matches.subcommand_matches("listaddresses").is_some() {
            cmd_list_address()?;
        } else if matches.subcommand_matches("server").is_some() {
            cmd_server().await?;
        } else if let Some(matches) = matches.subcommand_matches("createblockchain") {
            if let Some(address) = matches.value_of("address") {
                cmd_create_blockchain(address)?;
            }
        } else if let Some(matches) = matches.subcommand_matches("send") {
            let from = get_value("from", matches)?;
            let to = get_value("to", matches)?;

            let amount: i32 = if let Some(amount) = matches.value_of("amount") {
                amount.parse()?
            } else {
                error_start_miner("amount", matches.usage())
            };
            let target_node = matches.value_of("node");
            cmd_send(from, to, amount, matches.is_present("mine"), target_node)?;
        } else if let Some(matches) = matches.subcommand_matches("startnode") {
            if let Some(port) = matches.value_of("port") {
                println!("Start node...");
                let bc = Blockchain::new()?;
                let utxo_set = UTXOSet { blockchain: bc };
                let server = Server::new(
                    matches.value_of("host").unwrap_or("0.0.0.0"),
                    port,
                    "",
                    matches.value_of("bootstrap"),
                    utxo_set,
                )?;
                server.start_server()?;
            }
        } else if let Some(matches) = matches.subcommand_matches("startminer") {
            let mining_address = get_value("address", matches)?;

            let port = get_value("port", matches)?;

            println!("Start miner node...");
            let bc = Blockchain::new()?;
            let utxo_set = UTXOSet { blockchain: bc };
            let server = Server::new(
                matches.value_of("host").unwrap_or("0.0.0.0"),
                port,
                mining_address,
                matches.value_of("bootstrap"),
                utxo_set,
            )?;
            server.start_server()?;
        } else if let Some(matches) = matches.subcommand_matches("remotesend") {
            let from = matches.value_of("from").unwrap();
            let to = matches.value_of("to").unwrap();
            let amount: i32 = matches.value_of("amount").unwrap().parse()?;
            let node = matches.value_of("node").unwrap();
            let mine = matches.is_present("mine");

            cmd_remote_send(from, to, amount, node, mine)?;
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

fn cmd_remote_send(from: &str, to: &str, amount: i32, node: &str, mine_now: bool) -> Result<()> {
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

    // テスト実行用の結果型（実際のプロジェクトで使っている Result 型に合わせてください）
    type TestResult = std::result::Result<(), Box<dyn std::error::Error>>;

    /// ローカルで即時採掘を行う send コマンドのテスト
    #[test]
    fn test_cli_send_with_mine() -> TestResult {
        // 2 つのウォレットを作成
        let addr1 = cmd_create_wallet(EncryptionType::FNDSA)?;
        let addr2 = cmd_create_wallet(EncryptionType::FNDSA)?;
        // ジェネシスブロック作成：addr1 に初期報酬が入る（例では 10 とする）
        cmd_create_blockchain(&addr1)?;

        // 初期残高確認
        let balance1 = cmd_get_balance(&addr1)?;
        let balance2 = cmd_get_balance(&addr2)?;
        assert_eq!(balance1, 10);
        assert_eq!(balance2, 0);

        // addr1 から addr2 へ 5 単位送金（-m オプション：即時採掘モード、target_node は None）
        cmd_send(&addr1, &addr2, 5, true, None)?;

        // 採掘が行われたので、残高が更新されるはず
        let balance1_after = cmd_get_balance(&addr1)?;
        let balance2_after = cmd_get_balance(&addr2)?;
        // ※ このテストでは、採掘により報酬分の UTXO 更新が行われるため、例として addr1 の残高が 15, addr2 が 5 になる前提
        assert_eq!(balance1_after, 15);
        assert_eq!(balance2_after, 5);

        // addr2 から addr1 へ、残高以上（15 単位）の送金を試みる → エラーとなるはず
        let res = cmd_send(&addr2, &addr1, 15, true, None);
        assert!(res.is_err());

        // 再度残高確認（変化はないはず）
        let balance1_final = cmd_get_balance(&addr1)?;
        let balance2_final = cmd_get_balance(&addr2)?;
        assert_eq!(balance1_final, 15);
        assert_eq!(balance2_final, 5);

        Ok(())
    }

    #[test]
    fn test_cli_send_with_target_node() -> TestResult {
        let addr1 = cmd_create_wallet(EncryptionType::FNDSA)?;
        let addr2 = cmd_create_wallet(EncryptionType::FNDSA)?;
        cmd_create_blockchain(&addr1)?;

        let balance1 = cmd_get_balance(&addr1)?;
        let balance2 = cmd_get_balance(&addr2)?;
        assert_eq!(balance1, 10);
        assert_eq!(balance2, 0);

        let _ = cmd_send(&addr1, &addr2, 5, false, Some("127.0.0.1:7000"));
        Ok(())
    }
}
