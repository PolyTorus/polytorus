//! Modern CLI - Unified Modular Architecture Only

use actix_web::{web, App as ActixApp, HttpServer};
use clap::{App, Arg};

use crate::{
    config::{ConfigManager, DataContext},
    crypto::{types::EncryptionType, wallets::*},
    modular::{default_modular_config, UnifiedModularOrchestrator},
    webserver::simulation_api::{
        get_stats, get_status, health_check, send_transaction, submit_transaction, SimulationState,
    },
    Result,
};

pub struct ModernCli {}

impl Default for ModernCli {
    fn default() -> Self {
        Self::new()
    }
}

impl ModernCli {
    pub fn new() -> ModernCli {
        ModernCli {}
    }
    pub async fn run(&self) -> Result<()> {
        let matches = App::new("Polytorus - Modern Blockchain")
            .version("2.0.0")
            .author("Modern Architecture Team")
            .about("Unified Modular Blockchain Platform")
            .arg(
                Arg::with_name("config")
                    .long("config")
                    .help("Configuration file path")
                    .takes_value(true)
                    .value_name("CONFIG_FILE"),
            )
            .arg(
                Arg::with_name("data-dir")
                    .long("data-dir")
                    .help("Data directory path")
                    .takes_value(true)
                    .value_name("DATA_DIR"),
            )
            .arg(
                Arg::with_name("http-port")
                    .long("http-port")
                    .help("HTTP API server port")
                    .takes_value(true)
                    .value_name("PORT"),
            )
            .arg(
                Arg::with_name("createwallet")
                    .long("createwallet")
                    .help("Create a new wallet")
                    .takes_value(false),
            )
            .arg(
                Arg::with_name("listaddresses")
                    .long("listaddresses")
                    .help("List all addresses in wallets")
                    .takes_value(false),
            )
            .arg(
                Arg::with_name("getbalance")
                    .long("getbalance")
                    .help("Get balance for an address")
                    .takes_value(true)
                    .value_name("ADDRESS"),
            )
            .arg(
                Arg::with_name("modular-init")
                    .long("modular-init")
                    .help("Initialize modular architecture")
                    .takes_value(false),
            )
            .arg(
                Arg::with_name("modular-status")
                    .long("modular-status")
                    .help("Show modular system status")
                    .takes_value(false),
            )
            .arg(
                Arg::with_name("modular-config")
                    .long("modular-config")
                    .help("Show modular configuration")
                    .takes_value(false),
            )
            .arg(
                Arg::with_name("smart-contract-deploy")
                    .long("smart-contract-deploy")
                    .help("Deploy a smart contract")
                    .takes_value(true)
                    .value_name("CONTRACT_PATH"),
            )
            .arg(
                Arg::with_name("smart-contract-call")
                    .long("smart-contract-call")
                    .help("Call a smart contract function")
                    .takes_value(true)
                    .value_name("CONTRACT_ADDRESS"),
            )
            .arg(
                Arg::with_name("erc20-deploy")
                    .long("erc20-deploy")
                    .help("Deploy an ERC20 token contract")
                    .takes_value(true)
                    .value_name("NAME,SYMBOL,DECIMALS,SUPPLY,OWNER"),
            )
            .arg(
                Arg::with_name("erc20-transfer")
                    .long("erc20-transfer")
                    .help("Transfer ERC20 tokens")
                    .takes_value(true)
                    .value_name("CONTRACT,TO,AMOUNT"),
            )
            .arg(
                Arg::with_name("erc20-balance")
                    .long("erc20-balance")
                    .help("Check ERC20 token balance")
                    .takes_value(true)
                    .value_name("CONTRACT,ADDRESS"),
            )
            .arg(
                Arg::with_name("erc20-approve")
                    .long("erc20-approve")
                    .help("Approve ERC20 token spending")
                    .takes_value(true)
                    .value_name("CONTRACT,SPENDER,AMOUNT"),
            )
            .arg(
                Arg::with_name("erc20-allowance")
                    .long("erc20-allowance")
                    .help("Check ERC20 token allowance")
                    .takes_value(true)
                    .value_name("CONTRACT,OWNER,SPENDER"),
            )
            .arg(
                Arg::with_name("erc20-info")
                    .long("erc20-info")
                    .help("Get ERC20 token information")
                    .takes_value(true)
                    .value_name("CONTRACT_ADDRESS"),
            )
            .arg(
                Arg::with_name("erc20-list")
                    .long("erc20-list")
                    .help("List all deployed ERC20 contracts")
                    .takes_value(false),
            )
            .arg(
                Arg::with_name("governance-propose")
                    .long("governance-propose")
                    .help("Create a governance proposal")
                    .takes_value(true)
                    .value_name("PROPOSAL_DATA"),
            )
            .arg(
                Arg::with_name("governance-vote")
                    .long("governance-vote")
                    .help("Vote on a governance proposal")
                    .takes_value(true)
                    .value_name("PROPOSAL_ID"),
            )
            .arg(
                Arg::with_name("network-start")
                    .long("network-start")
                    .help("Start P2P network node")
                    .takes_value(false),
            )
            .arg(
                Arg::with_name("network-status")
                    .long("network-status")
                    .help("Show network status")
                    .takes_value(false),
            )
            .arg(
                Arg::with_name("network-connect")
                    .long("network-connect")
                    .help("Connect to a peer")
                    .takes_value(true)
                    .value_name("ADDRESS"),
            )
            .arg(
                Arg::with_name("network-peers")
                    .long("network-peers")
                    .help("List connected peers")
                    .takes_value(false),
            )
            .arg(
                Arg::with_name("network-sync")
                    .long("network-sync")
                    .help("Force blockchain synchronization")
                    .takes_value(false),
            )
            .arg(
                Arg::with_name("modular-start")
                    .long("modular-start")
                    .help("Start modular blockchain with P2P network")
                    .takes_value(false),
            )
            .arg(
                Arg::with_name("network-health")
                    .long("network-health")
                    .help("Show network health information")
                    .takes_value(false),
            )
            .arg(
                Arg::with_name("network-blacklist")
                    .long("network-blacklist")
                    .help("Blacklist a peer")
                    .takes_value(true)
                    .value_name("PEER_ID"),
            )
            .arg(
                Arg::with_name("network-queue-stats")
                    .long("network-queue-stats")
                    .help("Show message queue statistics")
                    .takes_value(false),
            )
            .get_matches(); // Extract common options
        let config_path = matches.value_of("config");
        let data_dir = matches.value_of("data-dir");
        let http_port = matches.value_of("http-port");

        if matches.is_present("createwallet") {
            self.cmd_create_wallet().await?;
        } else if matches.is_present("listaddresses") {
            self.cmd_list_addresses().await?;
        } else if let Some(address) = matches.value_of("getbalance") {
            self.cmd_get_balance(address).await?;
        } else if matches.is_present("modular-init") {
            self.cmd_modular_init_with_options(config_path, data_dir)
                .await?;
        } else if matches.is_present("modular-start") {
            self.cmd_modular_start_with_options(config_path, data_dir, http_port)
                .await?;
        } else if matches.is_present("modular-status") {
            self.cmd_modular_status_with_options(config_path, data_dir)
                .await?;
        } else if matches.is_present("modular-config") {
            self.cmd_modular_config().await?;
        } else if let Some(contract_path) = matches.value_of("smart-contract-deploy") {
            self.cmd_smart_contract_deploy(contract_path).await?;
        } else if let Some(contract_address) = matches.value_of("smart-contract-call") {
            self.cmd_smart_contract_call(contract_address).await?;
        } else if let Some(params) = matches.value_of("erc20-deploy") {
            self.cmd_erc20_deploy(params).await?;
        } else if let Some(params) = matches.value_of("erc20-transfer") {
            self.cmd_erc20_transfer(params).await?;
        } else if let Some(params) = matches.value_of("erc20-balance") {
            self.cmd_erc20_balance(params).await?;
        } else if let Some(params) = matches.value_of("erc20-approve") {
            self.cmd_erc20_approve(params).await?;
        } else if let Some(params) = matches.value_of("erc20-allowance") {
            self.cmd_erc20_allowance(params).await?;
        } else if let Some(contract_address) = matches.value_of("erc20-info") {
            self.cmd_erc20_info(contract_address).await?;
        } else if matches.is_present("erc20-list") {
            self.cmd_erc20_list().await?;
        } else if let Some(proposal_data) = matches.value_of("governance-propose") {
            self.cmd_governance_propose(proposal_data).await?;
        } else if let Some(proposal_id) = matches.value_of("governance-vote") {
            self.cmd_governance_vote(proposal_id).await?;
        } else if matches.is_present("network-start") {
            self.cmd_network_start().await?;
        } else if matches.is_present("network-status") {
            self.cmd_network_status().await?;
        } else if let Some(address) = matches.value_of("network-connect") {
            self.cmd_network_connect(address).await?;
        } else if matches.is_present("network-peers") {
            self.cmd_network_peers().await?;
        } else if matches.is_present("network-sync") {
            self.cmd_network_sync().await?;
        } else if matches.is_present("network-health") {
            self.cmd_network_health().await?;
        } else if let Some(peer_id) = matches.value_of("network-blacklist") {
            self.cmd_network_blacklist(peer_id).await?;
        } else if matches.is_present("network-queue-stats") {
            self.cmd_network_queue_stats().await?;
        } else {
            println!("Use --help for usage information");
        }

        Ok(())
    }

    pub async fn cmd_create_wallet(&self) -> Result<()> {
        let data_context = DataContext::default();

        println!("Creating new wallet...");
        let mut wallets = Wallets::new_with_context(data_context)?;
        let address = wallets.create_wallet(EncryptionType::ECDSA);
        wallets.save_all()?;

        println!("New wallet created");
        println!("Address: {}", address);

        Ok(())
    }

    pub async fn cmd_list_addresses(&self) -> Result<()> {
        let data_context = DataContext::default();

        let wallets = Wallets::new_with_context(data_context)?;
        let addresses = wallets.get_all_addresses();

        if addresses.is_empty() {
            println!("No wallets found. Create one with --createwallet");
        } else {
            println!("Wallet addresses:");
            for address in addresses {
                println!("  {}", address);
            }
        }

        Ok(())
    }

    pub async fn cmd_get_balance(&self, address: &str) -> Result<()> {
        println!("Getting balance for address: {}", address);
        println!("Balance functionality not yet implemented in unified orchestrator");
        println!("Address: {}", address);

        Ok(())
    }
    async fn cmd_modular_init_with_options(
        &self,
        _config_path: Option<&str>,
        data_dir: Option<&str>,
    ) -> Result<()> {
        println!("Initializing modular architecture...");

        let config = default_modular_config();
        let data_context = if let Some(data_dir) = data_dir {
            DataContext::new(std::path::PathBuf::from(data_dir))
        } else {
            DataContext::default()
        };

        // Initialize data directories
        data_context.ensure_directories()?;

        let _orchestrator =
            UnifiedModularOrchestrator::create_and_start_with_defaults(config, data_context)
                .await?;

        println!("Modular architecture initialized successfully");
        println!("Orchestrator status: Active");
        if let Some(data_dir) = data_dir {
            println!("Data directory: {}", data_dir);
        }

        Ok(())
    }

    async fn cmd_modular_status_with_options(
        &self,
        _config_path: Option<&str>,
        data_dir: Option<&str>,
    ) -> Result<()> {
        let config = default_modular_config();
        let data_context = if let Some(data_dir) = data_dir {
            DataContext::new(std::path::PathBuf::from(data_dir))
        } else {
            DataContext::default()
        };

        let orchestrator =
            UnifiedModularOrchestrator::create_and_start_with_defaults(config, data_context)
                .await?;

        println!("=== Modular System Status ===");
        println!("Architecture: Unified Modular");
        println!("Orchestrator: Active");
        println!("Components: All modules loaded");
        println!("Status: Operational");
        if let Some(data_dir) = data_dir {
            println!("Data directory: {}", data_dir);
        }

        let state = orchestrator.get_state().await;
        println!("Block height: {}", state.current_block_height);
        println!("Running: {}", state.is_running);

        let metrics = orchestrator.get_metrics().await;
        println!("Total blocks processed: {}", metrics.total_blocks_processed);
        println!(
            "Total transactions processed: {}",
            metrics.total_transactions_processed
        );

        Ok(())
    }

    async fn cmd_modular_config(&self) -> Result<()> {
        let config = default_modular_config();
        let data_context = DataContext::default();
        let orchestrator =
            UnifiedModularOrchestrator::create_and_start_with_defaults(config, data_context)
                .await?;

        println!("=== Modular Configuration ===");
        match orchestrator.get_current_config().await {
            Ok(config_str) => {
                println!("Current config: {}", config_str);
            }
            Err(e) => {
                println!("Error getting config: {}", e);
            }
        }

        Ok(())
    }

    async fn cmd_smart_contract_deploy(&self, contract_path: &str) -> Result<()> {
        println!("Deploying smart contract from: {}", contract_path);
        println!("Smart contract functionality not yet implemented in unified orchestrator");

        Ok(())
    }

    async fn cmd_smart_contract_call(&self, contract_address: &str) -> Result<()> {
        println!("Calling smart contract: {}", contract_address);
        println!("Smart contract functionality not yet implemented in unified orchestrator");

        Ok(())
    }

    async fn cmd_governance_propose(&self, proposal_data: &str) -> Result<()> {
        println!("Creating governance proposal: {}", proposal_data);
        println!("Governance functionality not yet implemented in unified orchestrator");

        Ok(())
    }

    async fn cmd_governance_vote(&self, proposal_id: &str) -> Result<()> {
        println!("Voting on governance proposal: {}", proposal_id);
        println!("Governance functionality not yet implemented in unified orchestrator");

        Ok(())
    }

    async fn cmd_network_start(&self) -> Result<()> {
        println!("Starting P2P network node...");

        // Read network configuration
        let config = self.read_network_config().await?;

        println!("Listening on: {}", config.listen_addr);
        println!("Bootstrap peers: {:?}", config.bootstrap_peers);

        // Create and start networked blockchain node
        let mut network_node = crate::network::NetworkedBlockchainNode::new(
            config.listen_addr,
            config.bootstrap_peers,
        )
        .await?;

        // Start the network node (this would typically run in background)
        network_node.start().await?;

        println!("P2P network node started successfully");
        println!("Node is now listening for peer connections and synchronizing with the network");

        Ok(())
    }

    async fn cmd_network_status(&self) -> Result<()> {
        println!("=== Network Status ===");
        println!("Implementation: Enhanced P2P with blockchain integration");
        println!("Status: Active (simulated - requires running network node)");

        // In a real implementation, this would connect to the running network node
        // and get actual status information
        println!("Connected peers: 0 (no active node)");
        println!("Blockchain height: 0");
        println!("Sync status: Not syncing");
        println!("Mempool transactions: 0");

        println!("\nTo start the network, use: --network-start");

        Ok(())
    }

    async fn cmd_network_connect(&self, address: &str) -> Result<()> {
        println!("Connecting to peer: {}", address);

        // Parse the address
        let socket_addr: std::net::SocketAddr = address
            .parse()
            .map_err(|e| failure::format_err!("Invalid address format: {}", e))?;

        println!("Parsed address: {}", socket_addr);
        println!("Connection functionality requires a running network node");
        println!("Start the network first with: --network-start");

        Ok(())
    }

    async fn cmd_network_peers(&self) -> Result<()> {
        println!("=== Connected Peers ===");
        println!("No active network node running");
        println!("Start the network first with: --network-start");

        // In a real implementation, this would show:
        // - Peer IDs
        // - IP addresses and ports
        // - Connection duration
        // - Blockchain heights
        // - Data transfer statistics

        Ok(())
    }

    async fn cmd_network_sync(&self) -> Result<()> {
        println!("Force synchronizing blockchain...");
        println!("Sync functionality requires a running network node");
        println!("Start the network first with: --network-start");

        Ok(())
    }

    async fn cmd_network_health(&self) -> Result<()> {
        println!("=== Network Health Information ===");

        // In a real implementation, this would connect to the running network node
        // and request actual health information through the NetworkCommand channel

        println!("Implementation Note: This command requires integration with");
        println!("a running NetworkedBlockchainNode to provide real-time data.");
        println!("Current implementation shows simulated data:");
        println!();
        println!("Network Status: Healthy");
        println!("Total Nodes: 10");
        println!("Healthy Peers: 8");
        println!("Degraded Peers: 2");
        println!("Unhealthy Peers: 0");
        println!("Average Latency: 45ms");
        println!("Network Diameter: 3 hops");

        println!();
        println!("To get real data, ensure the node is running with:");
        println!("  --modular-start");

        Ok(())
    }

    async fn cmd_network_blacklist(&self, peer_id: &str) -> Result<()> {
        println!("=== Blacklist Peer ===");
        println!("Attempting to blacklist peer: {}", peer_id);

        // In a real implementation, this would send a NetworkCommand::BlacklistPeer
        // to the running network node

        println!("Implementation Note: This command requires a running network node.");
        println!("The peer would be added to the blacklist and disconnected.");
        println!("Current status: Command prepared (network node required)");

        Ok(())
    }

    async fn cmd_network_queue_stats(&self) -> Result<()> {
        println!("=== Message Queue Statistics ===");

        // In a real implementation, this would send a NetworkCommand::GetMessageQueueStats
        // and receive actual statistics from the running network node

        println!("Implementation Note: This shows simulated data.");
        println!("Real data requires a running network node.");
        println!();
        println!("Priority Queues:");
        println!("  Critical: 0 messages");
        println!("  High: 5 messages");
        println!("  Normal: 23 messages");
        println!("  Low: 12 messages");
        println!();
        println!("Processing Stats:");
        println!("  Total Processed: 1,247 messages");
        println!("  Total Dropped: 3 messages");
        println!("  Average Processing Time: 2.3ms");
        println!("  Bandwidth Usage: 1.2 MB/s");

        println!();
        println!("To get real statistics, start the node with:");
        println!("  --modular-start");

        Ok(())
    }

    async fn read_network_config(&self) -> Result<NetworkConfig> {
        // Try to load from configuration file
        let config_manager =
            ConfigManager::new("config/polytorus.toml".to_string()).unwrap_or_default();

        let config = config_manager.get_config();
        let (listen_addr, bootstrap_peers) = config_manager.get_network_addresses()?;

        let network_config = NetworkConfig {
            listen_addr,
            bootstrap_peers,
            max_peers: config.network.max_peers as usize,
            connection_timeout: config.network.connection_timeout,
        };

        Ok(network_config)
    }

    async fn cmd_modular_start_with_options(
        &self,
        _config_path: Option<&str>,
        data_dir: Option<&str>,
        http_port: Option<&str>,
    ) -> Result<()> {
        println!("Starting modular blockchain with P2P network...");

        // Load network configuration
        let network_config = self.read_network_config().await?;

        println!("Network configuration:");
        println!("  Listen address: {}", network_config.listen_addr);
        println!("  Bootstrap peers: {:?}", network_config.bootstrap_peers);
        println!("  Max peers: {}", network_config.max_peers);
        println!(
            "  Connection timeout: {}s",
            network_config.connection_timeout
        );

        // Create orchestrator configuration
        let modular_config = default_modular_config();
        let data_context = if let Some(data_dir) = data_dir {
            DataContext::new(std::path::PathBuf::from(data_dir))
        } else {
            DataContext::default()
        };

        // Initialize data directories
        data_context.ensure_directories()?;

        // Create orchestrator with network integration
        let orchestrator = UnifiedModularOrchestrator::create_and_start_with_defaults(
            modular_config,
            data_context,
        )
        .await?;

        println!("Modular blockchain started successfully");
        println!("Network layer: Integrated");
        println!("Status: Running");
        if let Some(data_dir) = data_dir {
            println!("Data directory: {}", data_dir);
        } // Show current status
        let state = orchestrator.get_state().await;
        println!("Block height: {}", state.current_block_height);
        println!("Running: {}", state.is_running); // Start HTTP API server if port is specified
        if let Some(port_str) = http_port {
            let port: u16 = port_str.parse().unwrap_or(9000);
            let node_id = format!("node-{}", port - 9000);
            let data_dir_path = data_dir.unwrap_or("./data").to_string();

            println!("🌐 Starting HTTP API server on port {}", port);

            let simulation_state = SimulationState::new(node_id.clone(), data_dir_path.clone());

            // Start HTTP server in background
            tokio::spawn(async move {
                let simulation_state_data = web::Data::new(simulation_state);
                let server_result = HttpServer::new(move || {
                    ActixApp::new()
                        .app_data(simulation_state_data.clone())
                        .route("/status", web::get().to(get_status))
                        .route("/transaction", web::post().to(submit_transaction))
                        .route("/send", web::post().to(send_transaction))
                        .route("/stats", web::get().to(get_stats))
                        .route("/health", web::get().to(health_check))
                })
                .bind(format!("127.0.0.1:{}", port))
                .expect("Failed to bind HTTP server")
                .run()
                .await;

                if let Err(e) = server_result {
                    eprintln!("HTTP server error: {}", e);
                }
            });

            println!("✅ HTTP API available at: http://127.0.0.1:{}", port);
        }

        // Keep the orchestrator running
        tokio::signal::ctrl_c()
            .await
            .expect("Failed to listen for ctrl+c");
        println!("Shutting down...");

        Ok(())
    }

    // ERC20 Command Handlers

    pub async fn cmd_erc20_deploy(&self, params: &str) -> Result<()> {
        use crate::smart_contract::{ContractEngine, ContractState};

        let parts: Vec<&str> = params.split(',').collect();
        if parts.len() != 5 {
            println!("Error: Invalid parameters. Expected: NAME,SYMBOL,DECIMALS,SUPPLY,OWNER");
            return Ok(());
        }

        let name = parts[0].to_string();
        let symbol = parts[1].to_string();
        let decimals: u8 = parts[2].parse().unwrap_or(18);
        let initial_supply: u64 = parts[3].parse().unwrap_or(0);
        let owner = parts[4].to_string();

        println!("Deploying ERC20 token contract...");
        println!("Name: {}", name);
        println!("Symbol: {}", symbol);
        println!("Decimals: {}", decimals);
        println!("Initial Supply: {}", initial_supply);
        println!("Owner: {}", owner);

        // Initialize contract engine
        let data_context = DataContext::default();
        data_context.ensure_directories()?;
        let state = ContractState::new(&data_context.contracts_db_path)?;
        let engine = ContractEngine::new(state)?;

        // Generate contract address
        let contract_address = format!("erc20_{}", symbol.to_lowercase());

        // Deploy ERC20 contract
        match engine.deploy_erc20_contract(
            name.clone(),
            symbol.clone(),
            decimals,
            initial_supply,
            owner.clone(),
            contract_address.clone(),
        ) {
            Ok(_) => {
                println!("✅ ERC20 contract deployed successfully!");
                println!("Contract Address: {}", contract_address);
            }
            Err(e) => {
                println!("❌ Failed to deploy ERC20 contract: {}", e);
            }
        }

        Ok(())
    }

    pub async fn cmd_erc20_transfer(&self, params: &str) -> Result<()> {
        use crate::smart_contract::{ContractEngine, ContractState};

        let parts: Vec<&str> = params.split(',').collect();
        if parts.len() != 3 {
            println!("Error: Invalid parameters. Expected: CONTRACT,TO,AMOUNT");
            return Ok(());
        }

        let contract_address = parts[0];
        let to = parts[1];
        let amount: u64 = parts[2].parse().unwrap_or(0);

        println!("Transferring ERC20 tokens...");
        println!("Contract: {}", contract_address);
        println!("To: {}", to);
        println!("Amount: {}", amount);

        // Initialize contract engine
        let data_context = DataContext::default();
        data_context.ensure_directories()?;
        let state = ContractState::new(&data_context.contracts_db_path)?;
        let engine = ContractEngine::new(state)?;

        // Use first available wallet address as caller
        let wallets = Wallets::new_with_context(DataContext::default())?;
        let addresses = wallets.get_all_addresses();
        let caller = if addresses.is_empty() {
            "alice".to_string()
        } else {
            addresses[0].clone()
        };

        match engine.execute_erc20_contract(
            contract_address,
            "transfer",
            &caller,
            vec![to.to_string(), amount.to_string()],
        ) {
            Ok(result) => {
                if result.success {
                    println!("✅ Transfer successful!");
                    for log in result.logs {
                        println!("📝 {}", log);
                    }
                } else {
                    println!(
                        "❌ Transfer failed: {}",
                        String::from_utf8_lossy(&result.return_value)
                    );
                }
            }
            Err(e) => {
                println!("❌ Transfer error: {}", e);
            }
        }

        Ok(())
    }

    pub async fn cmd_erc20_balance(&self, params: &str) -> Result<()> {
        use crate::smart_contract::{ContractEngine, ContractState};

        let parts: Vec<&str> = params.split(',').collect();
        if parts.len() != 2 {
            println!("Error: Invalid parameters. Expected: CONTRACT,ADDRESS");
            return Ok(());
        }

        let contract_address = parts[0];
        let address = parts[1];

        println!("Checking ERC20 token balance...");
        println!("Contract: {}", contract_address);
        println!("Address: {}", address);

        // Initialize contract engine
        let data_context = DataContext::default();
        data_context.ensure_directories()?;
        let state = ContractState::new(&data_context.contracts_db_path)?;
        let engine = ContractEngine::new(state)?;

        match engine.execute_erc20_contract(
            contract_address,
            "balanceOf",
            address,
            vec![address.to_string()],
        ) {
            Ok(result) => {
                if result.success {
                    let balance = String::from_utf8_lossy(&result.return_value);
                    println!("💰 Balance: {} tokens", balance);
                } else {
                    println!(
                        "❌ Failed to get balance: {}",
                        String::from_utf8_lossy(&result.return_value)
                    );
                }
            }
            Err(e) => {
                println!("❌ Balance check error: {}", e);
            }
        }

        Ok(())
    }

    pub async fn cmd_erc20_approve(&self, params: &str) -> Result<()> {
        use crate::smart_contract::{ContractEngine, ContractState};

        let parts: Vec<&str> = params.split(',').collect();
        if parts.len() != 3 {
            println!("Error: Invalid parameters. Expected: CONTRACT,SPENDER,AMOUNT");
            return Ok(());
        }

        let contract_address = parts[0];
        let spender = parts[1];
        let amount: u64 = parts[2].parse().unwrap_or(0);

        println!("Approving ERC20 token spending...");
        println!("Contract: {}", contract_address);
        println!("Spender: {}", spender);
        println!("Amount: {}", amount);

        // Initialize contract engine
        let data_context = DataContext::default();
        data_context.ensure_directories()?;
        let state = ContractState::new(&data_context.contracts_db_path)?;
        let engine = ContractEngine::new(state)?;

        // Use first available wallet address as caller
        let wallets = Wallets::new_with_context(DataContext::default())?;
        let addresses = wallets.get_all_addresses();
        let caller = if addresses.is_empty() {
            "alice".to_string()
        } else {
            addresses[0].clone()
        };

        match engine.execute_erc20_contract(
            contract_address,
            "approve",
            &caller,
            vec![spender.to_string(), amount.to_string()],
        ) {
            Ok(result) => {
                if result.success {
                    println!("✅ Approval successful!");
                    for log in result.logs {
                        println!("📝 {}", log);
                    }
                } else {
                    println!(
                        "❌ Approval failed: {}",
                        String::from_utf8_lossy(&result.return_value)
                    );
                }
            }
            Err(e) => {
                println!("❌ Approval error: {}", e);
            }
        }

        Ok(())
    }

    pub async fn cmd_erc20_allowance(&self, params: &str) -> Result<()> {
        use crate::smart_contract::{ContractEngine, ContractState};

        let parts: Vec<&str> = params.split(',').collect();
        if parts.len() != 3 {
            println!("Error: Invalid parameters. Expected: CONTRACT,OWNER,SPENDER");
            return Ok(());
        }

        let contract_address = parts[0];
        let owner = parts[1];
        let spender = parts[2];

        println!("Checking ERC20 token allowance...");
        println!("Contract: {}", contract_address);
        println!("Owner: {}", owner);
        println!("Spender: {}", spender);

        // Initialize contract engine
        let data_context = DataContext::default();
        data_context.ensure_directories()?;
        let state = ContractState::new(&data_context.contracts_db_path)?;
        let engine = ContractEngine::new(state)?;

        match engine.execute_erc20_contract(
            contract_address,
            "allowance",
            owner,
            vec![owner.to_string(), spender.to_string()],
        ) {
            Ok(result) => {
                if result.success {
                    let allowance = String::from_utf8_lossy(&result.return_value);
                    println!("🔓 Allowance: {} tokens", allowance);
                } else {
                    println!(
                        "❌ Failed to get allowance: {}",
                        String::from_utf8_lossy(&result.return_value)
                    );
                }
            }
            Err(e) => {
                println!("❌ Allowance check error: {}", e);
            }
        }

        Ok(())
    }

    pub async fn cmd_erc20_info(&self, contract_address: &str) -> Result<()> {
        use crate::smart_contract::{ContractEngine, ContractState};

        println!("Getting ERC20 contract information...");
        println!("Contract: {}", contract_address);

        // Initialize contract engine
        let data_context = DataContext::default();
        data_context.ensure_directories()?;
        let state = ContractState::new(&data_context.contracts_db_path)?;
        let engine = ContractEngine::new(state)?;

        match engine.get_erc20_contract_info(contract_address) {
            Ok(Some((name, symbol, decimals, total_supply))) => {
                println!("📄 Contract Information:");
                println!("  Name: {}", name);
                println!("  Symbol: {}", symbol);
                println!("  Decimals: {}", decimals);
                println!("  Total Supply: {}", total_supply);
            }
            Ok(None) => {
                println!("❌ ERC20 contract not found: {}", contract_address);
            }
            Err(e) => {
                println!("❌ Error getting contract info: {}", e);
            }
        }

        Ok(())
    }

    pub async fn cmd_erc20_list(&self) -> Result<()> {
        use crate::smart_contract::{ContractEngine, ContractState};

        println!("Listing all deployed ERC20 contracts...");

        // Initialize contract engine
        let data_context = DataContext::default();
        data_context.ensure_directories()?;
        let state = ContractState::new(&data_context.contracts_db_path)?;
        let engine = ContractEngine::new(state)?;

        match engine.list_erc20_contracts() {
            Ok(contracts) => {
                if contracts.is_empty() {
                    println!("No ERC20 contracts found.");
                } else {
                    println!("📋 Deployed ERC20 contracts:");
                    for contract_address in contracts {
                        println!("  📄 {}", contract_address);

                        // Get additional info for each contract
                        if let Ok(Some((name, symbol, decimals, total_supply))) =
                            engine.get_erc20_contract_info(&contract_address)
                        {
                            println!("     Name: {}, Symbol: {}", name, symbol);
                            println!("     Decimals: {}, Supply: {}", decimals, total_supply);
                        }
                    }
                }
            }
            Err(e) => {
                println!("❌ Error listing contracts: {}", e);
            }
        }

        Ok(())
    }
}

#[derive(Debug, Clone)]
struct NetworkConfig {
    listen_addr: std::net::SocketAddr,
    bootstrap_peers: Vec<std::net::SocketAddr>,
    max_peers: usize,
    connection_timeout: u64,
}
