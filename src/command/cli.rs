//! cli process - Modular Architecture CLI

// Legacy imports removed in Phase 4 - using modular architecture only
use crate::crypto::types::EncryptionType;
use crate::crypto::wallets::*;
use crate::modular::{
    default_modular_config, load_modular_config_from_file, UnifiedModularOrchestrator,
};
use crate::webserver::server::WebServer;
use crate::Result;
use clap::{App, Arg, ArgMatches};
use std::process::exit;

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
            .about("Post Quantum Modular Blockchain")
            .subcommand(
                App::new("modular")
                    .about("Modular blockchain operations [RECOMMENDED]")
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
                    )                    .subcommand(App::new("state").about("Get modular blockchain state information"))
                    .subcommand(App::new("layers").about("Show information about all layers"))
                    .subcommand(
                        App::new("challenge")
                            .about("Submit a settlement challenge")
                            .arg(Arg::from_usage("<batch-id> 'Batch ID to challenge'"))
                            .arg(Arg::from_usage("<reason> 'Challenge reason'")),
                    )
                    .subcommand(
                        App::new("eutxo")
                            .about("Extended UTXO operations")
                            .subcommand(App::new("stats").about("Show eUTXO statistics"))
                            .subcommand(
                                App::new("balance")
                                    .about("Get eUTXO balance for address")
                                    .arg(Arg::from_usage("<address> 'Address to check balance'")),
                            )
                            .subcommand(
                                App::new("utxos")
                                    .about("List UTXOs for address")
                                    .arg(Arg::from_usage("<address> 'Address to list UTXOs'")),
                            ),
                    ),
            )
            .subcommand(App::new("printchain").about("[LEGACY] print all the chain blocks"))
            .subcommand(
                App::new("createwallet").about("create a wallet").arg(
                    Arg::from_usage("<encryption> 'encryption type'")
                        .possible_values(&["ECDSA", "FNDSA"])
                        .default_value("FNDSA")
                        .help("encryption type"),
                ),
            )
            .subcommand(App::new("listaddresses").about("list all addresses"))
            .subcommand(App::new("reindex").about("[LEGACY] reindex UTXO"))
            .subcommand(App::new("server").about("[LEGACY] run server"))
            .subcommand(
                App::new("startnode")
                    .about("[LEGACY] start the node server")
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
                    .about("[LEGACY] start the minner server")
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
            .subcommand(
                App::new("createblockchain")
                    .about("[LEGACY] create blockchain")
                    .arg(Arg::from_usage(
                        "<address> 'The address to send genesis block reward to'",
                    )),
            )
            .subcommand(
                App::new("send")
                    .about("[LEGACY] send in the blockchain")
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

        // Show helpful message for no arguments
        if std::env::args().len() == 1 {
            println!("🔗 PolyTorus - Post Quantum Modular Blockchain");
            println!();
            println!("🚀 Quick start:");
            println!("  polytorus modular start          # Start modular blockchain");
            println!("  polytorus createwallet FNDSA     # Create quantum-resistant wallet");
            println!("  polytorus modular mine <address> # Mine blocks");
            println!();
            println!("💡 Use 'polytorus --help' for all commands");
            println!("📖 Use 'polytorus modular --help' for modular commands");
            return Ok(());
        }

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
            ("startnode", Some(_sub_m)) => {
                println!(
                    "Legacy startnode command removed. Use 'polytorus modular start' instead."
                );
                return Err(failure::err_msg(
                    "Legacy startnode command removed. Use modular architecture.",
                ));
            }
            ("startminer", Some(_sub_m)) => {
                println!(
                    "Legacy startminer command removed. Use 'polytorus modular mine' instead."
                );
                return Err(failure::err_msg(
                    "Legacy startminer command removed. Use modular architecture.",
                ));
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
                }                ("challenge", Some(challenge_m)) => {
                    let batch_id = get_value("batch-id", challenge_m)?;
                    let reason = get_value("reason", challenge_m)?;
                    cmd_modular_challenge(batch_id, reason).await?;
                }
                ("eutxo", Some(eutxo_m)) => match eutxo_m.subcommand() {
                    ("stats", Some(_)) => {
                        cmd_eutxo_stats().await?;
                    }
                    ("balance", Some(balance_m)) => {
                        let address = get_value("address", balance_m)?;
                        cmd_eutxo_balance(address).await?;
                    }
                    ("utxos", Some(utxos_m)) => {
                        let address = get_value("address", utxos_m)?;
                        cmd_eutxo_utxos(address).await?;
                    }
                    _ => {
                        println!("Unknown eUTXO subcommand. Use --help for available commands.");
                    }
                },
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
    WebServer::run().await?;
    Ok(())
}

// Legacy commands removed in Phase 4 - replaced by modular architecture
fn cmd_send(
    _from: &str,
    _to: &str,
    _amount: i32,
    _mine_now: bool,
    _target_node: Option<&str>,
) -> Result<()> {
    Err(failure::err_msg(
        "Legacy send command removed. Use 'polytorus modular' commands instead.",
    ))
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

pub fn cmd_reindex() -> Result<i32> {
    Err(failure::err_msg(
        "Legacy reindex command removed. Use 'polytorus modular' commands instead.",
    ))
}

fn cmd_create_blockchain(_address: &str) -> Result<()> {
    Err(failure::err_msg(
        "Legacy blockchain creation removed. Use 'polytorus modular start' instead.",
    ))
}

fn cmd_get_balance(_address: &str) -> Result<i32> {
    Err(failure::err_msg(
        "Legacy balance command removed. Use 'polytorus modular' commands instead.",
    ))
}

pub fn cmd_print_chain() -> Result<()> {
    Err(failure::err_msg(
        "Legacy print chain command removed. Use 'polytorus modular state' instead.",
    ))
}

pub fn cmd_list_address() -> Result<()> {
    let ws = Wallets::new()?;
    let addresses = ws.get_all_addresses();
    println!("addresses: ");
    for ad in addresses {
        println!("{}", ad);
    }
    Ok(())
}

fn cmd_remote_send(
    _from: &str,
    _to: &str,
    _amount: i32,
    _node: &str,
    _mine_now: bool,
) -> Result<()> {
    Err(failure::err_msg(
        "Legacy remote send command removed. Use 'polytorus modular' commands instead.",
    ))
}

// Smart contract command functions
fn cmd_deploy_contract(
    _wallet: &str,
    _bytecode_file: &str,
    _gas_limit: u64,
    _mine_now: bool,
) -> Result<()> {
    Err(failure::err_msg(
        "Legacy contract deployment removed. Use 'polytorus modular' commands instead.",
    ))
}

fn cmd_call_contract(
    _wallet: &str,
    _contract: &str,
    _function: &str,
    _value: i32,
    _gas_limit: u64,
    _mine_now: bool,
) -> Result<()> {
    Err(failure::err_msg(
        "Legacy contract calls removed. Use 'polytorus modular' commands instead.",
    ))
}

fn cmd_list_contracts() -> Result<()> {
    Err(failure::err_msg(
        "Legacy contract listing removed. Use 'polytorus modular' commands instead.",
    ))
}

fn cmd_contract_state(_contract: &str) -> Result<()> {
    Err(failure::err_msg(
        "Legacy contract state access removed. Use 'polytorus modular' commands instead.",
    ))
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
    };    let data_context = crate::config::DataContext::default();
    let orchestrator = UnifiedModularOrchestrator::create_and_start_with_defaults(config, data_context).await?;
    println!("🚀 Unified Modular Blockchain started successfully!");

    // Show initial orchestrator state
    let initial_state = orchestrator.get_state().await;
    println!("📊 Initial state - Block height: {}", initial_state.current_block_height);
    println!("⚡ Orchestrator running: {}", initial_state.is_running);

    // Keep the orchestrator running and show periodic status
    let mut counter = 0;
    loop {
        tokio::time::sleep(tokio::time::Duration::from_secs(10)).await;
        counter += 1;
        
        if counter % 6 == 0 { // Every minute
            let current_state = orchestrator.get_state().await;
            let metrics = orchestrator.get_metrics().await;
            println!("📈 Status update - Height: {}, Blocks processed: {}, Events: {}", 
                current_state.current_block_height,
                metrics.total_blocks_processed,
                metrics.total_events_handled
            );
        }
    }
}

async fn cmd_modular_mine(address: &str, tx_count: usize) -> Result<()> {
    println!("⛏️ Starting mining with unified orchestrator...");
    println!("📍 Mining address: {}", address);
    println!("📦 Target transactions: {}", tx_count);
    
    let config = default_modular_config();
    let data_context = crate::config::DataContext::default();
    let orchestrator = UnifiedModularOrchestrator::create_and_start_with_defaults(config, data_context).await?;
    
    // Simulate transaction processing
    for i in 0..tx_count {
        let tx_data = format!("tx_{}_{}", i, address).into_bytes();
        let tx_id = orchestrator.execute_transaction(tx_data).await?;
        println!("✅ Executed transaction {}/{}: {}", i + 1, tx_count, tx_id);
        
        // Small delay between transactions
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
    
    // Send a configuration update message
    orchestrator.update_configuration(
        "mining".to_string(), 
        format!("completed_batch_for_{}", address)
    ).await?;
      // Send a message through the message bus
    let message_payload = format!("Mining completed for {} with {} transactions", address, tx_count);
    orchestrator.send_message("mining_complete".to_string(), message_payload.clone().into_bytes()).await?;
    
    // Also broadcast the message
    orchestrator.broadcast_message("mining_broadcast".to_string(), message_payload.into_bytes()).await?;
    
    // Create a test component using the factory
    let factory_info = orchestrator.create_test_component().await?;
    println!("🏭 Factory info: {}", factory_info);
    
    // Show final state
    let final_state = orchestrator.get_state().await;
    let final_metrics = orchestrator.get_metrics().await;
    
    println!("\n=== Mining Summary ===");
    println!("✅ Mining completed successfully!");
    println!("📊 Total transactions processed: {}", final_metrics.total_transactions_processed);
    println!("📨 Total events generated: {}", final_metrics.total_events_handled);
    println!("📏 Current block height: {}", final_state.current_block_height);
    
    Ok(())
}

async fn cmd_modular_state() -> Result<()> {
    println!("🔍 Getting modular orchestrator state...");
    let config = default_modular_config();
    let data_context = crate::config::DataContext::default();
    let orchestrator = UnifiedModularOrchestrator::create_and_start_with_defaults(config, data_context).await?;
    
    let state = orchestrator.get_state().await;
    let metrics = orchestrator.get_metrics().await;
    let health = orchestrator.get_layer_health().await?;
    
    println!("=== Unified Modular Orchestrator State ===");
    println!("🚀 Running: {}", state.is_running);
    println!("📏 Current block height: {}", state.current_block_height);
    println!("🔗 Last finalized block: {:?}", state.last_finalized_block);
    println!("⏳ Pending transactions: {}", state.pending_transactions);
    println!("📊 Total blocks processed: {}", metrics.total_blocks_processed);
    println!("💰 Total transactions processed: {}", metrics.total_transactions_processed);
    println!("📨 Total events handled: {}", metrics.total_events_handled);
    println!("⚡ Average block time: {:.2}ms", metrics.average_block_time_ms);
    println!("📈 Error rate: {:.4}%", metrics.error_rate * 100.0);
    
    println!("\n=== Layer Health ===");
    for (layer, healthy) in health {
        let status = if healthy { "✅" } else { "❌" };
        println!("{} {}: {}", status, layer, if healthy { "Healthy" } else { "Unhealthy" });
    }
    
    Ok(())
}

async fn cmd_modular_layers() -> Result<()> {
    println!("🔍 Getting layer information...");
    let config = default_modular_config();
    let data_context = crate::config::DataContext::default();
    let orchestrator = UnifiedModularOrchestrator::create_and_start_with_defaults(config, data_context).await?;
      let health = orchestrator.get_layer_health().await?;
    let metrics = orchestrator.get_metrics().await;
    let detailed_info = orchestrator.get_detailed_layer_info().await?;
    let config_info = orchestrator.get_current_config().await?;
    
    println!("=== Modular Architecture Layers ===");
    println!("⚙️ Configuration: {}", config_info);
    
    for (layer_name, is_healthy) in &health {
        let status_icon = if *is_healthy { "✅" } else { "❌" };
        let status_text = if *is_healthy { "HEALTHY" } else { "UNHEALTHY" };
        
        println!("\n🔹 {} Layer", layer_name.to_uppercase());
        println!("   Status: {} {}", status_icon, status_text);
        
        // Show detailed layer information
        if let Some(detail) = detailed_info.get(layer_name) {
            println!("   Details: {}", detail);
        }
        
        // Show layer-specific information
        match layer_name.as_str() {
            "execution" => {
                println!("   Function: Transaction execution and smart contract processing");
                println!("   Gas limit: 8,000,000");
            }
            "settlement" => {
                println!("   Function: Batch settlement and fraud proof validation");
                println!("   Challenge period: 100 blocks");
            }
            "consensus" => {
                println!("   Function: Block validation and consensus mechanism");
                println!("   Block time: 10 seconds");
            }
            "data_availability" => {
                println!("   Function: Data storage and retrieval");
                println!("   Retention: 7 days");
            }
            _ => {}
        }
    }
    
    println!("\n=== Overall Performance ===");
    println!("📊 Total events processed: {}", metrics.total_events_handled);
    println!("⚡ System uptime: {}s", metrics.uptime_seconds);
    println!("📈 Error rate: {:.4}%", metrics.error_rate * 100.0);
    
    Ok(())
}

async fn cmd_modular_challenge(_batch_id: &str, _reason: &str) -> Result<()> {
    Err(failure::err_msg("modular challenge: not yet implemented for unified orchestrator"))
}

async fn cmd_eutxo_stats() -> Result<()> {
    Err(failure::err_msg("eutxo stats: not yet implemented for unified orchestrator"))
}

async fn cmd_eutxo_balance(_address: &str) -> Result<()> {
    Err(failure::err_msg("eutxo balance: not yet implemented for unified orchestrator"))
}

async fn cmd_eutxo_utxos(_address: &str) -> Result<()> {
    Err(failure::err_msg("eutxo utxos: not yet implemented for unified orchestrator"))
}

// Tests temporarily disabled during Phase 4 legacy cleanup
// They will be re-implemented using modular architecture in future phases
/*
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
*/
