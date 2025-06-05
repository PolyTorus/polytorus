//! cli process

use crate::blockchain::blockchain::*;
use crate::blockchain::types::network;
use crate::blockchain::utxoset::*;
use crate::crypto::transaction::*;
use crate::crypto::types::EncryptionType;
use crate::crypto::wallets::*;
use crate::modular::{
    default_modular_config, load_modular_config_from_file, ModularBlockchainBuilder,
};
use crate::network::server::Server;
use crate::smart_contract::{ContractState, SmartContract};
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
                    .arg(Arg::from_usage("<node> 'Remote node address (host:port)'"))
                    .arg(Arg::from_usage(
                        "-m --mine 'mine immediately on the remote node'",
                    )),
            )
            .subcommand(
                App::new("deploycontract")
                    .about("deploy a smart contract")
                    .arg(Arg::from_usage(
                        "<wallet> 'Wallet address to pay for deployment'",
                    ))
                    .arg(Arg::from_usage(
                        "<bytecode-file> 'Path to WASM bytecode file'",
                    ))
                    .arg(Arg::from_usage(
                        "[gas-limit] 'Gas limit for deployment (default: 1000000)'",
                    ))
                    .arg(Arg::from_usage("-m --mine 'mine immediately'")),
            )
            .subcommand(
                App::new("callcontract")
                    .about("call a smart contract function")
                    .arg(Arg::from_usage("<wallet> 'Wallet address to pay for call'"))
                    .arg(Arg::from_usage("<contract> 'Contract address'"))
                    .arg(Arg::from_usage("<function> 'Function name to call'"))
                    .arg(Arg::from_usage("[value] 'Value to send (default: 0)'"))
                    .arg(Arg::from_usage(
                        "[gas-limit] 'Gas limit for call (default: 100000)'",
                    ))
                    .arg(Arg::from_usage("-m --mine 'mine immediately'")),
            )
            .subcommand(App::new("listcontracts").about("list all deployed contracts"))
            .subcommand(
                App::new("contractstate")
                    .about("get contract state")
                    .arg(Arg::from_usage("<contract> 'Contract address'")),
            )
            .subcommand(
                App::new("modular")
                    .about("Modular blockchain operations")
                    .subcommand(
                        App::new("start")
                            .about("Start modular blockchain")
                            .arg(Arg::from_usage("[config] 'Path to configuration file'")),
                    )
                    .subcommand(
                        App::new("mine")
                            .about("Mine block with modular architecture")
                            .arg(Arg::from_usage("<address> 'Mining reward address'"))
                            .arg(Arg::from_usage(
                                "[tx-count] 'Number of transactions to include (default: 10)'",
                            )),
                    )
                    .subcommand(App::new("state").about("Get modular blockchain state information"))
                    .subcommand(App::new("layers").about("Show information about all layers"))
                    .subcommand(
                        App::new("challenge")
                            .about("Submit a settlement challenge")
                            .arg(Arg::from_usage("<batch-id> 'Batch ID to challenge'"))
                            .arg(Arg::from_usage("<reason> 'Challenge reason'")),
                    ),
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
                    let bc: Blockchain<network::Mainnet> = Blockchain::new()?;
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
                let bc: Blockchain<network::Mainnet> = Blockchain::new()?;
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
            ("modular", Some(sub_m)) => match sub_m.subcommand() {
                ("start", Some(start_m)) => {
                    let config_path = start_m.value_of("config");
                    cmd_modular_start(config_path).await?;
                }
                ("mine", Some(mine_m)) => {
                    let address = get_value("address", mine_m)?;
                    let tx_count: usize = if let Some(count) = mine_m.value_of("tx-count") {
                        count.parse()?
                    } else {
                        10
                    };
                    cmd_modular_mine(address, tx_count).await?;
                }
                ("state", Some(_)) => {
                    cmd_modular_state().await?;
                }
                ("layers", Some(_)) => {
                    cmd_modular_layers().await?;
                }
                ("challenge", Some(challenge_m)) => {
                    let batch_id = get_value("batch-id", challenge_m)?;
                    let reason = get_value("reason", challenge_m)?;
                    cmd_modular_challenge(batch_id, reason).await?;
                }
                _ => {
                    println!("Unknown modular subcommand. Use --help for available commands.");
                }
            },
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
    let bc: Blockchain<network::Mainnet> = Blockchain::new()?;
    let mut utxo_set = UTXOSet { blockchain: bc };
    let wallets = Wallets::new()?;
    let wallet = wallets.get_wallet(from).unwrap();

    // Use wallet's encryption type
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
    let bc: Blockchain<network::Mainnet> = Blockchain::new()?;
    let utxo_set = UTXOSet { blockchain: bc };
    utxo_set.reindex()?;
    utxo_set.count_transactions()
}

fn cmd_create_blockchain(address: &str) -> Result<()> {
    let address = String::from(address);
    let bc: Blockchain<network::Mainnet> = Blockchain::create_blockchain(address)?;

    let utxo_set = UTXOSet { blockchain: bc };
    utxo_set.reindex()?;
    println!("create blockchain");
    Ok(())
}

fn cmd_get_balance(address: &str) -> Result<i32> {
    // Extract base address without encryption suffix
    let (base_address, _) = extract_encryption_type(address)?;
    let pub_key_hash = Address::decode(&base_address).unwrap().body;
    let bc: Blockchain<network::Mainnet> = Blockchain::new()?;
    let utxo_set = UTXOSet { blockchain: bc };
    let utxos = utxo_set.find_UTXO(&pub_key_hash)?;

    let balance = utxos.outputs.iter().map(|out| out.value).sum();
    Ok(balance)
}

pub fn cmd_print_chain() -> Result<()> {
    let bc: Blockchain<network::Mainnet> = Blockchain::new()?;
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
    let bc: Blockchain<network::Mainnet> = Blockchain::new()?;
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
fn cmd_deploy_contract(
    wallet: &str,
    bytecode_file: &str,
    gas_limit: u64,
    mine_now: bool,
) -> Result<()> {
    // Read bytecode from file
    let bytecode = fs::read(bytecode_file).map_err(failure::Error::from)?;
    // Validate bytecode size
    if bytecode.len() > 1024 * 1024 {
        // 1MB limit
        return Err(failure::err_msg("Bytecode file too large (max 1MB)"));
    }

    let bc: Blockchain<network::Mainnet> = Blockchain::new()?;
    let mut utxo_set = UTXOSet { blockchain: bc };
    let wallets = Wallets::new()?;
    let wallet_obj = wallets
        .get_wallet(wallet)
        .ok_or_else(|| failure::err_msg(format!("Wallet '{}' not found", wallet)))?;

    // Create contract deployment transaction
    let crypto = crate::crypto::get_crypto_provider(&wallet_obj.encryption_type);
    let tx = Transaction::new_contract_deployment(
        wallet_obj,
        bytecode,
        Vec::new(), // constructor_args
        gas_limit,
        &utxo_set,
        crypto.as_ref(),
    )?;
    if mine_now {
        // Calculate contract address before mining
        let contract_address = if let Some(contract_data) = &tx.contract_data {
            if let ContractTransactionData {
                tx_type: ContractTransactionType::Deploy { bytecode, .. },
                ..
            } = contract_data
            {
                // Generate the same address that SmartContract::new would generate
                SmartContract::new(bytecode.clone(), wallet.to_string(), vec![], None)?
                    .get_address()
                    .to_string()
            } else {
                tx.id.clone()
            }
        } else {
            tx.id.clone()
        };

        // Mine immediately
        let cbtx = Transaction::new_coinbase(wallet.to_string(), String::from("reward!"))?;
        let new_block = utxo_set.blockchain.mine_block(vec![cbtx, tx])?;
        utxo_set.update(&new_block)?;
        // Get contract address from the transaction
        if let Some(contract_data) = new_block
            .get_transactions()
            .last()
            .unwrap()
            .contract_data
            .as_ref()
        {
            if let ContractTransactionData {
                tx_type: ContractTransactionType::Deploy { gas_limit, .. },
                ..
            } = contract_data
            {
                println!("Contract deployed successfully!");
                println!("Contract Address: {}", contract_address);
                println!("Gas Limit: {}", gas_limit);
                return Ok(());
            }
        }
        println!("Contract deployed successfully! (Address will be available after mining)");
    } else {
        // Send to network (not implemented in this example)
        let tx_id = tx.id.clone(); // Store transaction ID before moving
        println!("Contract deployment transaction created. Use --mine to deploy immediately.");
        println!("Transaction ID: {}", tx_id);
    }

    Ok(())
}

fn cmd_call_contract(
    wallet: &str,
    contract: &str,
    function: &str,
    value: i32,
    gas_limit: u64,
    mine_now: bool,
) -> Result<()> {
    let bc: Blockchain<network::Mainnet> = Blockchain::new()?;
    let mut utxo_set = UTXOSet { blockchain: bc };
    let wallets = Wallets::new()?;
    let wallet_obj = wallets
        .get_wallet(wallet)
        .ok_or_else(|| failure::err_msg(format!("Wallet '{}' not found", wallet)))?; // Verify contract exists by checking metadata instead of state
    let contract_state_path = utxo_set.blockchain.context.data_dir().join("contracts");
    let contract_state = ContractState::new(contract_state_path.to_str().unwrap())?;

    let contracts = contract_state.list_contracts()?;
    let contract_exists = contracts.iter().any(|c| c.address == contract);

    if !contract_exists {
        return Err(failure::err_msg(format!(
            "Contract '{}' not found",
            contract
        )));
    }

    // Create function call data (simplified - in practice you'd want proper ABI encoding)
    let call_data = format!("{}()", function).into_bytes(); // Create contract call transaction
    let crypto = crate::crypto::get_crypto_provider(&wallet_obj.encryption_type);
    let tx = Transaction::new_contract_call(
        wallet_obj,
        contract.to_string(),
        function.to_string(),
        call_data,
        gas_limit,
        value as u64,
        &utxo_set,
        crypto.as_ref(),
    )?;
    if mine_now {
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
    let bc: Blockchain<network::Mainnet> = Blockchain::new()?;

    // Use a reasonable limit to prevent timeouts
    const MAX_CONTRACTS_TO_LIST: usize = 100;

    let contracts = match bc.list_contracts_with_limit(Some(MAX_CONTRACTS_TO_LIST)) {
        Ok(contracts) => contracts,
        Err(e) => {
            eprintln!("Error listing contracts: {}", e);
            println!("Attempting fallback with direct contract state access...");

            // Fallback approach using direct contract state
            let contract_state_path = bc.context.data_dir().join("contracts");
            let contract_state = ContractState::new(contract_state_path.to_str().unwrap())?;
            contract_state.list_contracts_with_limit(Some(MAX_CONTRACTS_TO_LIST))?
        }
    };

    if contracts.is_empty() {
        println!("No contracts deployed.");
        return Ok(());
    }

    println!("Deployed Contracts ({}):", contracts.len());
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
    let bc: Blockchain<network::Mainnet> = Blockchain::new()?;

    // Direct access to contract state instead of going through blockchain
    let contract_state_path = bc.context.data_dir().join("contracts");
    let contract_state = ContractState::new(contract_state_path.to_str().unwrap())?;
    let state = contract_state.get_contract_state(contract)?;

    if !state.is_empty() {
        println!("Contract State for: {}", contract);
        println!("==================");

        // Display storage
        println!("Storage:");
        for (key, value) in state {
            println!(
                "  {}: {}",
                hex::encode(key.as_bytes()),
                String::from_utf8_lossy(&value)
            );
        }
    } else {
        println!("Contract '{}' not found or has no state.", contract);
    }

    Ok(())
}

// Modular blockchain command implementations
async fn cmd_modular_start(config_path: Option<&str>) -> Result<()> {
    println!("Starting modular blockchain...");

    let config = if let Some(path) = config_path {
        println!("Loading configuration from: {}", path);
        match load_modular_config_from_file(path) {
            Ok(cfg) => {
                println!("Configuration loaded successfully!");
                cfg
            }
            Err(e) => {
                println!("Failed to load configuration file: {}", e);
                println!("Using default configuration instead...");
                default_modular_config()
            }
        }
    } else {
        println!("No configuration file specified, using defaults...");
        default_modular_config()
    };

    let data_context = crate::config::DataContext::default();
    let blockchain = ModularBlockchainBuilder::new()
        .with_config(config)
        .with_data_context(data_context)
        .build()?;

    blockchain.start().await?;
    println!("Modular blockchain started successfully!");

    // Keep the blockchain running
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(1)).await;
    }
}

async fn cmd_modular_mine(address: &str, tx_count: usize) -> Result<()> {
    println!(
        "Mining block with modular architecture for address: {}",
        address
    );
    println!("Including {} transactions", tx_count);

    let config = default_modular_config();
    let data_context = crate::config::DataContext::default();
    let blockchain = ModularBlockchainBuilder::new()
        .with_config(config)
        .with_data_context(data_context)
        .build()?;

    // Create sample transactions
    let mut transactions = Vec::new();
    for i in 0..tx_count {
        let tx = Transaction::new_coinbase(
            format!("{}_{}", address, i),
            format!("Mining reward {}", i),
        )?;
        transactions.push(tx);
    }
    let block = blockchain.mine_block(transactions).await?;
    println!("Successfully mined block: {}", block.get_hash());
    println!("Block height: {}", block.get_height());
    println!("Timestamp: {}", block.get_timestamp());

    Ok(())
}

async fn cmd_modular_state() -> Result<()> {
    println!("Getting modular blockchain state...");

    let config = default_modular_config();
    let data_context = crate::config::DataContext::default();
    let blockchain = ModularBlockchainBuilder::new()
        .with_config(config)
        .with_data_context(data_context)
        .build()?;

    let state_info = blockchain.get_state_info()?;

    println!("=== Modular Blockchain State ===");
    println!("Execution state root: {}", state_info.execution_state_root);
    println!("Settlement root: {}", state_info.settlement_root);
    println!("Block height: {}", state_info.block_height);
    println!(
        "Canonical chain length: {}",
        state_info.canonical_chain_length
    );

    Ok(())
}

async fn cmd_modular_layers() -> Result<()> {
    println!("=== Modular Blockchain Layers Information ===\n");

    println!("1. Execution Layer:");
    println!("   - Handles transaction execution and smart contracts");
    println!("   - WASM-based contract engine");
    println!("   - State management and gas metering");

    println!("\n2. Settlement Layer:");
    println!("   - Finalizes state transitions");
    println!("   - Fraud proof verification");
    println!("   - Challenge period management");

    println!("\n3. Consensus Layer:");
    println!("   - Proof-of-Work block validation");
    println!("   - Chain management");
    println!("   - Fork resolution");

    println!("\n4. Data Availability Layer:");
    println!("   - P2P data storage and retrieval");
    println!("   - Availability proof generation");
    println!("   - Network communication");

    let config = default_modular_config();
    println!("\n=== Current Configuration ===");
    println!("Gas limit: {}", config.execution.gas_limit);
    println!(
        "Challenge period: {} blocks",
        config.settlement.challenge_period
    );
    println!("Block time: {}ms", config.consensus.block_time);
    println!("Max block size: {} bytes", config.consensus.max_block_size);

    Ok(())
}

async fn cmd_modular_challenge(batch_id: &str, reason: &str) -> Result<()> {
    use crate::modular::{FraudProof, SettlementChallenge};

    println!("Submitting settlement challenge...");
    println!("Batch ID: {}", batch_id);
    println!("Reason: {}", reason);

    let config = default_modular_config();
    let data_context = crate::config::DataContext::default();
    let blockchain = ModularBlockchainBuilder::new()
        .with_config(config)
        .with_data_context(data_context)
        .build()?;
    // Create a fraud proof with the reason as evidence
    let fraud_proof = FraudProof {
        batch_id: batch_id.to_string(),
        proof_data: reason.as_bytes().to_vec(),
        expected_state_root: "expected_state".to_string(),
        actual_state_root: "actual_state".to_string(),
    };

    let challenge = SettlementChallenge {
        challenge_id: format!("challenge_{}", batch_id),
        batch_id: batch_id.to_string(),
        proof: fraud_proof,
        challenger: "cli_user".to_string(),
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
    };

    let result = blockchain.submit_challenge(challenge).await?;

    println!("Challenge submitted successfully!");
    println!("Challenge ID: {}", result.challenge_id);
    println!("Successful: {}", result.successful);
    if let Some(penalty) = result.penalty {
        println!("Penalty applied: {}", penalty);
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
        // Create test context
        let context = create_test_context();

        // Create wallets
        let mut wallets = Wallets::new_with_context(context.clone())?;
        let addr1 = wallets.create_wallet(EncryptionType::FNDSA);
        let addr2 = wallets.create_wallet(EncryptionType::FNDSA);
        wallets.save_all()?;

        // Create genesis block
        let bc = Blockchain::create_blockchain_with_context(addr1.clone(), context.clone())?;

        // Create UTXO set and rebuild index
        let mut utxo_set = UTXOSet { blockchain: bc };
        utxo_set.reindex()?;

        // Check balances
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

        // Send and mine
        let wallet1 = wallets.get_wallet(&addr1).unwrap();
        let crypto = FnDsaCrypto;
        let tx = Transaction::new_UTXO(wallet1, &addr2, 5, &utxo_set, &crypto)?;
        let cbtx = Transaction::new_coinbase(addr1.clone(), String::from("reward!"))?;

        // Mine block (using test difficulty for faster execution)
        let new_block = utxo_set
            .blockchain
            .mine_block_with_test_difficulty(vec![cbtx, tx])?;
        utxo_set.update(&new_block)?;

        // Check balances after mining
        let utxos1_after = utxo_set.find_UTXO(&pub_key_hash1)?;
        let balance1_after: i32 = utxos1_after.outputs.iter().map(|out| out.value).sum();
        let utxos2_after = utxo_set.find_UTXO(&pub_key_hash2)?;
        let balance2_after: i32 = utxos2_after.outputs.iter().map(|out| out.value).sum();

        assert_eq!(balance1_after, 15); // 10 (initial) - 5 (sent) + 10 (mining reward)
        assert_eq!(balance2_after, 5);

        // Try to send 15 units from addr2 to addr1 (more than available balance)
        let wallet2 = wallets.get_wallet(&addr2).unwrap();
        let res = Transaction::new_UTXO(wallet2, &addr1, 15, &utxo_set, &crypto);
        assert!(res.is_err());

        // Check balances again (should be unchanged)
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

        let bc: Blockchain<network::Mainnet> =
            Blockchain::create_blockchain_with_context(addr1.clone(), context.clone())?;
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
        assert_eq!(balance2, 0); // Network functionality tests require actual nodes, so omitted
        cleanup_test_context(&context);
        Ok(())
    }
}
