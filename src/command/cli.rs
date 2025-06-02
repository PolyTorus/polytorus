//! cli process

use crate::blockchain::blockchain::*;
use crate::blockchain::utxoset::*;
use crate::crypto::transaction::*;
use crate::crypto::types::EncryptionType;
use crate::crypto::wallets::*;
use crate::network::server::Server;
use crate::webserver::webserver::WebServer;
use crate::Result;
use bitcoincash_addr::Address;
use clap::{App, Arg, ArgMatches};
use hex;
use std::fs;
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
                    .arg(Arg::from_usage("<node> 'Remote node address (host:port)'"))                    .arg(Arg::from_usage(
                        "-m --mine 'mine immediately on the remote node'",
                    )),
            )
            .subcommand(
                App::new("deploycontract")
                    .about("deploy a smart contract")
                    .arg(Arg::from_usage("<wallet> 'Wallet address to pay for deployment'"))
                    .arg(Arg::from_usage("<bytecode-file> 'Path to WASM bytecode file'"))
                    .arg(Arg::from_usage("[gas-limit] 'Gas limit for deployment (default: 1000000)'"))
                    .arg(Arg::from_usage("-m --mine 'mine immediately'"))
            )
            .subcommand(
                App::new("callcontract")
                    .about("call a smart contract function")
                    .arg(Arg::from_usage("<wallet> 'Wallet address to pay for call'"))
                    .arg(Arg::from_usage("<contract> 'Contract address'"))
                    .arg(Arg::from_usage("<function> 'Function name to call'"))
                    .arg(Arg::from_usage("[value] 'Value to send (default: 0)'"))
                    .arg(Arg::from_usage("[gas-limit] 'Gas limit for call (default: 100000)'"))
                    .arg(Arg::from_usage("-m --mine 'mine immediately'"))
            )
            .subcommand(
                App::new("listcontracts")
                    .about("list all deployed contracts")
            )
            .subcommand(
                App::new("contractstate")
                    .about("get contract state")
                    .arg(Arg::from_usage("<contract> 'Contract address'"))
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
            }            ("remotesend", Some(sub_m)) => {
                let from = sub_m.value_of("from").unwrap();
                let to = sub_m.value_of("to").unwrap();
                let amount: i32 = sub_m.value_of("amount").unwrap().parse()?;
                let node = sub_m.value_of("node").unwrap();
                let mine = sub_m.is_present("mine");
                cmd_remote_send(from, to, amount, node, mine)?;
            }
            ("deploycontract", Some(sub_m)) => {
                let wallet = get_value("wallet", sub_m)?;
                let bytecode_file = get_value("bytecode-file", sub_m)?;
                let gas_limit: u64 = if let Some(gas) = sub_m.value_of("gas-limit") {
                    gas.parse()?
                } else {
                    1000000
                };
                let mine_now = sub_m.is_present("mine");
                cmd_deploy_contract(wallet, bytecode_file, gas_limit, mine_now)?;
            }
            ("callcontract", Some(sub_m)) => {
                let wallet = get_value("wallet", sub_m)?;
                let contract = get_value("contract", sub_m)?;
                let function = get_value("function", sub_m)?;
                let value: i32 = if let Some(v) = sub_m.value_of("value") {
                    v.parse()?
                } else {
                    0
                };
                let gas_limit: u64 = if let Some(gas) = sub_m.value_of("gas-limit") {
                    gas.parse()?
                } else {
                    100000
                };
                let mine_now = sub_m.is_present("mine");
                cmd_call_contract(wallet, contract, function, value, gas_limit, mine_now)?;
            }
            ("listcontracts", Some(_)) => {
                cmd_list_contracts()?;
            }
            ("contractstate", Some(sub_m)) => {
                let contract = get_value("contract", sub_m)?;
                cmd_contract_state(contract)?;
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

    // ウォレットの暗号化方式を使用
    let crypto = crate::crypto::get_crypto_provider(&wallet.encryption_type);
    let tx = Transaction::new_UTXO(wallet, to, amount, &utxo_set, crypto.as_ref())?;
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
    // Extract base address without encryption suffix
    let (base_address, _) = extract_encryption_type(address)?;
    let pub_key_hash = Address::decode(&base_address).unwrap().body;
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
        contract_data: None,
    };

    let server = Server::new("0.0.0.0", "0", "", None, utxo_set)?;

    let signed_tx = server.send_sign_request(node, from, &tx)?;

    server.send_tx(node, &signed_tx)?;

    println!("Transaction sent successfully!");
    Ok(())
}

// Smart contract command functions
fn cmd_deploy_contract(wallet: &str, bytecode_file: &str, gas_limit: u64, mine_now: bool) -> Result<()> {    // Read bytecode from file
    let bytecode = fs::read(bytecode_file)
        .map_err(|e| failure::Error::from(e))?;
      // Validate bytecode size
    if bytecode.len() > 1024 * 1024 {  // 1MB limit
        return Err(failure::err_msg("Bytecode file too large (max 1MB)"));
    }
    
    let bc = Blockchain::new()?;
    let mut utxo_set = UTXOSet { blockchain: bc };    let wallets = Wallets::new()?;
    let wallet_obj = wallets.get_wallet(wallet)
        .ok_or_else(|| failure::err_msg(format!("Wallet '{}' not found", wallet)))?;

    // Create contract deployment transaction
    let crypto = crate::crypto::get_crypto_provider(&wallet_obj.encryption_type);
    let tx = Transaction::new_contract_deployment(
        wallet_obj,
        bytecode,
        Vec::new(), // constructor_args
        gas_limit,
        &utxo_set,
        crypto.as_ref()    )?;

    if mine_now {
        // Mine immediately
        let contract_address = tx.id.clone(); // Store contract address before moving
        let cbtx = Transaction::new_coinbase(wallet.to_string(), String::from("reward!"))?;
        let new_block = utxo_set.blockchain.mine_block(vec![cbtx, tx])?;
        utxo_set.update(&new_block)?;
          // Get contract address from the transaction
        if let Some(contract_data) = new_block.get_transaction().last().unwrap().contract_data.as_ref() {
            if let ContractTransactionData { tx_type: ContractTransactionType::Deploy { gas_limit, .. }, .. } = contract_data {
                println!("Contract deployed successfully!");
                println!("Contract Address: {}", contract_address); // Use stored contract address
                println!("Gas Limit: {}", gas_limit);
                return Ok(());
            }
        }
        println!("Contract deployed successfully! (Address will be available after mining)");    } else {
        // Send to network (not implemented in this example)
        let tx_id = tx.id.clone(); // Store transaction ID before moving
        println!("Contract deployment transaction created. Use --mine to deploy immediately.");
        println!("Transaction ID: {}", tx_id);
    }

    Ok(())
}

fn cmd_call_contract(wallet: &str, contract: &str, function: &str, value: i32, gas_limit: u64, mine_now: bool) -> Result<()> {
    let bc = Blockchain::new()?;
    let mut utxo_set = UTXOSet { blockchain: bc };    let wallets = Wallets::new()?;
    let wallet_obj = wallets.get_wallet(wallet)
        .ok_or_else(|| failure::err_msg(format!("Wallet '{}' not found", wallet)))?;    // Verify contract exists
    let contract_state = utxo_set.blockchain.get_contract_state(contract)?;
    if contract_state.is_empty() {
        return Err(failure::err_msg(format!("Contract '{}' not found", contract)));
    }

    // Create function call data (simplified - in practice you'd want proper ABI encoding)
    let call_data = format!("{}()", function).into_bytes();    // Create contract call transaction
    let crypto = crate::crypto::get_crypto_provider(&wallet_obj.encryption_type);
    let tx = Transaction::new_contract_call(
        wallet_obj,
        contract.to_string(),
        function.to_string(),
        call_data,
        gas_limit,
        value as u64,
        &utxo_set,
        crypto.as_ref()
    )?;    if mine_now {
        // Mine immediately
        let _tx_id = tx.id.clone(); // Store transaction ID before moving
        let cbtx = Transaction::new_coinbase(wallet.to_string(), String::from("reward!"))?;
        let new_block = utxo_set.blockchain.mine_block(vec![cbtx, tx])?;
        utxo_set.update(&new_block)?;
        
        println!("Contract function called successfully!");
        println!("Function: {}", function);
        println!("Gas Limit: {}", gas_limit);
    } else {
        // Send to network (not implemented in this example)
        let tx_id = tx.id.clone(); // Store transaction ID before moving
        println!("Contract call transaction created. Use --mine to execute immediately.");
        println!("Transaction ID: {}", tx_id);
    }

    Ok(())
}

fn cmd_list_contracts() -> Result<()> {
    let bc = Blockchain::new()?;
    let contracts = bc.list_contracts()?;
    
    if contracts.is_empty() {
        println!("No contracts deployed.");
        return Ok(());
    }
      println!("Deployed Contracts:");
    println!("==================");
    for contract_metadata in contracts {
        println!("Address: {}", contract_metadata.address);
        println!("Creator: {}", contract_metadata.creator);
        println!("Created At: {}", contract_metadata.created_at);
        println!("Bytecode Hash: {}", contract_metadata.bytecode_hash);
        if let Some(abi) = &contract_metadata.abi {
            println!("ABI: {}", abi);
        }
        println!("---");
    }
    
    Ok(())
}

fn cmd_contract_state(contract: &str) -> Result<()> {
    let bc = Blockchain::new()?;
    let state = bc.get_contract_state(contract)?;
    
    if !state.is_empty() {
        println!("Contract State for: {}", contract);
        println!("==================");
        
        // Display storage
        println!("Storage:");
        for (key, value) in state {
            println!("  {}: {}", 
                hex::encode(key.as_bytes()), 
                String::from_utf8_lossy(&value)
            );
        }
    } else {
        println!("Contract '{}' not found or has no state.", contract);
    }
    
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::blockchain::blockchain::Blockchain;
    use crate::blockchain::utxoset::UTXOSet;
    use crate::crypto::fndsa::FnDsaCrypto;
    use crate::crypto::wallets::Wallets;
    use crate::test_helpers::{cleanup_test_context, create_test_context};

    type TestResult = std::result::Result<(), Box<dyn std::error::Error>>;

    #[test]
    fn test_cli_send_with_mine() -> TestResult {
        // テスト用のコンテキストを作成
        let context = create_test_context();

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
        let (base_addr1, _) = extract_encryption_type(&addr1)?;
        let (base_addr2, _) = extract_encryption_type(&addr2)?;
        let pub_key_hash1 = Address::decode(&base_addr1).unwrap().body;
        let pub_key_hash2 = Address::decode(&base_addr2).unwrap().body;

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

        cleanup_test_context(&context);
        Ok(())
    }

    #[test]
    fn test_cli_send_with_target_node() -> TestResult {
        let context = create_test_context();

        let mut wallets = Wallets::new_with_context(context.clone())?;
        let addr1 = wallets.create_wallet(EncryptionType::FNDSA);
        let addr2 = wallets.create_wallet(EncryptionType::FNDSA);
        wallets.save_all()?;

        let bc = Blockchain::create_blockchain_with_context(addr1.clone(), context.clone())?;
        let utxo_set = UTXOSet { blockchain: bc };
        utxo_set.reindex()?;

        let (base_addr1, _) = extract_encryption_type(&addr1)?;
        let (base_addr2, _) = extract_encryption_type(&addr2)?;
        let pub_key_hash1 = Address::decode(&base_addr1).unwrap().body;
        let pub_key_hash2 = Address::decode(&base_addr2).unwrap().body;

        let utxos1 = utxo_set.find_UTXO(&pub_key_hash1)?;
        let balance1: i32 = utxos1.outputs.iter().map(|out| out.value).sum();
        let utxos2 = utxo_set.find_UTXO(&pub_key_hash2)?;
        let balance2: i32 = utxos2.outputs.iter().map(|out| out.value).sum();

        assert_eq!(balance1, 10);
        assert_eq!(balance2, 0);

        // ネットワーク機能のテストは実際のノードが必要なため省略
        cleanup_test_context(&context);
        Ok(())
    }
}
