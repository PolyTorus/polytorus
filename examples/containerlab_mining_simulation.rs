//! ContainerLab Mining Simulation
//!
//! This example demonstrates how to run a complete testnet simulation with
//! actual mining using the modular architecture and ContainerLab.

use std::{path::PathBuf, sync::Arc, time::Duration};

use actix_web::{web, App, HttpServer, Result as ActixResult};
use clap::{Arg, Command};
use polytorus::{
    blockchain::block::BuildingBlock,
    config::{ConfigManager, DataContext},
    crypto::{
        transaction::{TXOutput, Transaction},
        types::EncryptionType,
        wallets::Wallets,
    },
    modular::{
        consensus::PolyTorusConsensusLayer,
        default_modular_config,
        traits::{ConsensusConfig, ConsensusLayer},
        UnifiedModularOrchestrator,
    },
    Result,
};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::{
    sync::Mutex,
    time::{interval, sleep},
};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MinerNodeConfig {
    pub node_id: String,
    pub port: u16,
    pub p2p_port: u16,
    pub data_dir: String,
    pub bootstrap_peers: Vec<String>,
    pub is_miner: bool,
    pub mining_address: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContainerLabConfig {
    pub num_nodes: usize,
    pub num_miners: usize,
    pub base_port: u16,
    pub base_p2p_port: u16,
    pub mining_interval: u64,      // milliseconds between mining attempts
    pub transaction_interval: u64, // milliseconds between transactions
    pub simulation_duration: u64,  // seconds
}

impl Default for ContainerLabConfig {
    fn default() -> Self {
        Self {
            num_nodes: 4,
            num_miners: 2,
            base_port: 9000,
            base_p2p_port: 8000,
            mining_interval: 15000,      // 15 seconds
            transaction_interval: 10000, // 10 seconds
            simulation_duration: 600,    // 10 minutes
        }
    }
}

#[derive(Clone)]
pub struct MinerNode {
    pub config: MinerNodeConfig,
    pub orchestrator: Arc<UnifiedModularOrchestrator>,
    pub consensus: Arc<PolyTorusConsensusLayer>,
    pub mining_address: Option<String>,
    pub blocks_mined: Arc<Mutex<u64>>,
    pub tx_count: Arc<Mutex<u64>>,
    pub http_client: Client,
}

pub struct ContainerLabMiningSimulator {
    config: ContainerLabConfig,
    nodes: Vec<MinerNode>,
    is_running: Arc<Mutex<bool>>,
}

impl ContainerLabMiningSimulator {
    pub fn new(config: ContainerLabConfig) -> Self {
        Self {
            config,
            nodes: Vec::new(),
            is_running: Arc::new(Mutex::new(false)),
        }
    }

    /// Generate node configurations for ContainerLab environment
    pub fn generate_node_configs(&self) -> Vec<MinerNodeConfig> {
        let mut configs = Vec::new();

        for i in 0..self.config.num_nodes {
            let node_id = format!("node-{i}");
            let port = self.config.base_port + i as u16;
            let p2p_port = self.config.base_p2p_port + i as u16;
            let data_dir = format!("./data/containerlab/{node_id}");
            let is_miner = i > 0 && i <= self.config.num_miners; // Skip bootstrap node for mining

            // Generate bootstrap peers (connect to previous nodes)
            let mut bootstrap_peers = Vec::new();
            for j in 0..i {
                let peer_port = self.config.base_p2p_port + j as u16;
                bootstrap_peers.push(format!("127.0.0.1:{peer_port}"));
            }

            configs.push(MinerNodeConfig {
                node_id,
                port,
                p2p_port,
                data_dir,
                bootstrap_peers,
                is_miner,
                mining_address: None, // Will be set after wallet creation
            });
        }

        configs
    }

    /// Create mining wallets for miner nodes
    pub async fn create_mining_wallets(&mut self) -> Result<()> {
        println!("🔑 Creating mining wallets for miner nodes...");

        for config in self.generate_node_configs() {
            if config.is_miner {
                println!("   Creating wallet for miner: {}", config.node_id);

                // Create data context for this node
                let data_context = DataContext::new(PathBuf::from(config.data_dir.clone()));
                data_context.ensure_directories()?;

                // Create wallet for this miner
                let mut wallets = Wallets::new_with_context(data_context)?;
                let mining_address = wallets.create_wallet(EncryptionType::ECDSA);
                wallets.save_all()?;

                println!("   ✅ Mining wallet created: {mining_address}");

                // Store the mining address
                let address_file = format!("{}/mining_address.txt", config.data_dir);
                std::fs::write(&address_file, &mining_address)?;

                println!("   📝 Mining address saved to: {address_file}");
            }
        }

        Ok(())
    }

    /// Initialize and start all nodes with mining capabilities
    pub async fn start_nodes(&mut self) -> Result<()> {
        println!(
            "🚀 Starting {} nodes ({} miners) for ContainerLab simulation...",
            self.config.num_nodes, self.config.num_miners
        );

        let node_configs = self.generate_node_configs();

        for (i, mut node_config) in node_configs.into_iter().enumerate() {
            println!("📡 Starting node {} ({})", i + 1, node_config.node_id);

            // Create data directory
            let data_context = DataContext::new(PathBuf::from(node_config.data_dir.clone()));
            data_context.ensure_directories()?;

            // Load mining address if this is a miner
            if node_config.is_miner {
                let address_file = format!("{}/mining_address.txt", node_config.data_dir);
                if let Ok(address) = std::fs::read_to_string(&address_file) {
                    node_config.mining_address = Some(address.trim().to_string());
                    println!("   ⛏️  Mining address: {}", address.trim());
                }
            }

            // Create custom configuration for this node
            let config_manager = ConfigManager::default();
            let mut config = config_manager.get_config().clone();

            // Configure network settings
            config.network.listen_addr = format!("127.0.0.1:{}", node_config.p2p_port);
            config.network.bootstrap_peers = node_config.bootstrap_peers.clone();

            // Create modular orchestrator
            let modular_config = default_modular_config();
            let orchestrator = UnifiedModularOrchestrator::create_and_start_with_defaults(
                modular_config,
                data_context.clone(),
            )
            .await?;

            // Create consensus layer for mining
            let consensus_config = ConsensusConfig {
                block_time: 15000,           // 15 seconds for testnet
                difficulty: 4,               // Low difficulty for testing
                max_block_size: 1024 * 1024, // 1MB
            };

            let consensus = Arc::new(PolyTorusConsensusLayer::new(
                data_context,
                consensus_config,
                node_config.is_miner,
            )?);

            let miner_node = MinerNode {
                config: node_config.clone(),
                orchestrator: Arc::new(orchestrator),
                consensus,
                mining_address: node_config.mining_address.clone(),
                blocks_mined: Arc::new(Mutex::new(0)),
                tx_count: Arc::new(Mutex::new(0)),
                http_client: Client::new(),
            };

            self.nodes.push(miner_node);

            // Small delay between node starts
            sleep(Duration::from_millis(2000)).await;
        }

        // Wait for network to stabilize
        println!("⏳ Waiting for network to stabilize...");
        sleep(Duration::from_secs(10)).await;

        println!("✅ All nodes started successfully!");
        Ok(())
    }

    /// Start mining processes on miner nodes
    pub async fn start_mining(&self) -> Result<()> {
        println!("⛏️  Starting mining processes...");

        let is_running = self.is_running.clone();
        *is_running.lock().await = true;

        for (i, node) in self.nodes.iter().enumerate() {
            if node.config.is_miner {
                println!(
                    "   🔥 Starting miner on node {}: {}",
                    i, node.config.node_id
                );

                let node_clone = node.clone();
                let is_running_clone = is_running.clone();
                let mining_interval = self.config.mining_interval;

                // Start mining task for this node
                tokio::spawn(async move {
                    let mut mining_timer = interval(Duration::from_millis(mining_interval));
                    let mut block_number = 0u64;

                    while *is_running_clone.lock().await {
                        mining_timer.tick().await;

                        // Attempt to mine a block
                        match Self::mine_single_block(&node_clone, block_number).await {
                            Ok(mined) => {
                                if mined {
                                    let mut blocks_mined = node_clone.blocks_mined.lock().await;
                                    *blocks_mined += 1;
                                    println!(
                                        "   ⛏️  {} mined block #{} (total: {})",
                                        node_clone.config.node_id, block_number, *blocks_mined
                                    );
                                }
                            }
                            Err(e) => {
                                eprintln!(
                                    "   ❌ Mining error on {}: {}",
                                    node_clone.config.node_id, e
                                );
                            }
                        }

                        block_number += 1;
                    }
                });
            }
        }

        println!("✅ Mining processes started!");
        Ok(())
    }

    /// Mine a single block on a node
    async fn mine_single_block(node: &MinerNode, block_number: u64) -> Result<bool> {
        // Using already imported types

        // Create a simple coinbase transaction for the miner
        let mining_address = node
            .mining_address
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("Mining address not set"))?;

        let coinbase_tx = Transaction {
            id: format!(
                "coinbase_{}_{}_{}",
                node.config.node_id,
                block_number,
                uuid::Uuid::new_v4()
            ),
            vin: vec![], // Coinbase has no inputs
            vout: vec![TXOutput {
                value: 50 * 100_000_000, // 50 coins in satoshis
                pub_key_hash: mining_address.as_bytes().to_vec(),
                script: None,
                datum: None,
                reference_script: None,
            }],
            contract_data: None,
        };

        // Create building block using the proper constructor
        let building_block = BuildingBlock::new_building(
            vec![coinbase_tx],
            "0000000000000000000000000000000000000000000000000000000000000000".to_string(),
            block_number as i32,
            4, // difficulty
        );

        // Attempt to mine the block
        match node.consensus.mine_block(&building_block) {
            Ok(mined_block) => {
                // Validate and add the block
                if node.consensus.validate_block(&mined_block) {
                    println!(
                        "     ✅ Block validated and added: {}",
                        mined_block.get_hash()
                    );
                    Ok(true)
                } else {
                    println!("     ❌ Block validation failed");
                    Ok(false)
                }
            }
            Err(e) => {
                // Mining can fail, which is normal
                println!("     ⏭️  Mining attempt failed: {e}");
                Ok(false)
            }
        }
    }

    /// Start the HTTP API servers for monitoring
    pub async fn start_api_servers(&self) -> Result<()> {
        println!("🌐 Starting HTTP API servers...");

        for node in &self.nodes {
            let node_config = node.config.clone();
            let orchestrator = node.orchestrator.clone();
            let blocks_mined = node.blocks_mined.clone();
            let tx_count = node.tx_count.clone();

            tokio::spawn(async move {
                let server = HttpServer::new(move || {
                    let orchestrator = orchestrator.clone();
                    let blocks_mined = blocks_mined.clone();
                    let tx_count = tx_count.clone();

                    App::new()
                        .app_data(web::Data::new(orchestrator))
                        .app_data(web::Data::new(blocks_mined))
                        .app_data(web::Data::new(tx_count))
                        .route("/status", web::get().to(get_mining_status))
                        .route("/mining-stats", web::get().to(get_mining_stats))
                        .route("/transaction", web::post().to(submit_transaction))
                })
                .bind(format!("127.0.0.1:{}", node_config.port))
                .expect("Failed to bind server")
                .run();

                if let Err(e) = server.await {
                    eprintln!("Server error for {}: {}", node_config.node_id, e);
                }
            });
        }

        println!("✅ API servers started!");
        Ok(())
    }

    /// Start the complete simulation
    pub async fn run_simulation(&self) -> Result<()> {
        println!("🎯 Starting ContainerLab mining simulation...");

        // Start mining
        self.start_mining().await?;

        // Start transaction generation
        self.start_transaction_generation().await?;

        // Run simulation for specified duration
        sleep(Duration::from_secs(self.config.simulation_duration)).await;

        println!("⏹️  Simulation completed!");
        *self.is_running.lock().await = false;

        Ok(())
    }

    async fn start_transaction_generation(&self) -> Result<()> {
        println!("💸 Starting transaction generation...");

        let is_running = self.is_running.clone();
        let nodes = self.nodes.clone();
        let tx_interval = self.config.transaction_interval;

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_millis(tx_interval));
            let mut tx_counter = 0u64;

            while *is_running.lock().await {
                interval.tick().await;

                // Generate transaction between random nodes
                let sender_idx = tx_counter as usize % nodes.len();
                let receiver_idx = (tx_counter as usize + 1) % nodes.len();

                if let Err(e) =
                    Self::generate_transaction(&nodes[sender_idx], &nodes[receiver_idx], tx_counter)
                        .await
                {
                    eprintln!("Failed to generate transaction {tx_counter}: {e}");
                }

                tx_counter += 1;

                if tx_counter % 5 == 0 {
                    println!("📊 Generated {tx_counter} transactions");
                }
            }
        });

        Ok(())
    }

    async fn generate_transaction(
        sender: &MinerNode,
        receiver: &MinerNode,
        tx_id: u64,
    ) -> Result<()> {
        let tx_data = serde_json::json!({
            "from": sender.config.node_id,
            "to": receiver.config.node_id,
            "amount": 100 + (tx_id % 900),
            "nonce": tx_id
        });

        let url = format!("http://127.0.0.1:{}/transaction", receiver.config.port);
        match sender.http_client.post(&url).json(&tx_data).send().await {
            Ok(response) => {
                if response.status().is_success() {
                    println!(
                        "   💸 TX {}: {} -> {} ({})",
                        tx_id, sender.config.node_id, receiver.config.node_id, tx_data["amount"]
                    );
                    *sender.tx_count.lock().await += 1;
                }
            }
            Err(e) => {
                eprintln!("Transaction submit error: {e}");
            }
        }

        Ok(())
    }

    pub async fn print_final_stats(&self) {
        println!("\n📈 Final Mining Statistics:");
        println!("===========================");

        let mut total_blocks = 0u64;
        let mut total_txs = 0u64;

        for node in &self.nodes {
            let blocks_mined = *node.blocks_mined.lock().await;
            let tx_count = *node.tx_count.lock().await;

            let node_type = if node.config.is_miner {
                "Miner"
            } else {
                "Validator"
            };

            println!(
                "📡 {} ({}): Blocks: {}, Transactions: {}",
                node.config.node_id, node_type, blocks_mined, tx_count
            );

            total_blocks += blocks_mined;
            total_txs += tx_count;
        }

        println!("📊 Total: {total_blocks} blocks mined, {total_txs} transactions processed");
    }
}

// HTTP API handlers
async fn get_mining_status(
    orchestrator: web::Data<Arc<UnifiedModularOrchestrator>>,
    blocks_mined: web::Data<Arc<Mutex<u64>>>,
) -> ActixResult<web::Json<serde_json::Value>> {
    let state = orchestrator.get_state().await;
    let blocks = *blocks_mined.lock().await;

    let status = serde_json::json!({
        "status": "mining",
        "block_height": state.current_block_height,
        "blocks_mined": blocks,
        "is_running": state.is_running
    });

    Ok(web::Json(status))
}

async fn get_mining_stats(
    blocks_mined: web::Data<Arc<Mutex<u64>>>,
    tx_count: web::Data<Arc<Mutex<u64>>>,
) -> ActixResult<web::Json<serde_json::Value>> {
    let blocks = *blocks_mined.lock().await;
    let txs = *tx_count.lock().await;

    let stats = serde_json::json!({
        "blocks_mined": blocks,
        "transactions_processed": txs,
        "timestamp": chrono::Utc::now().to_rfc3339()
    });

    Ok(web::Json(stats))
}

async fn submit_transaction(
    tx_count: web::Data<Arc<Mutex<u64>>>,
    _transaction: web::Json<serde_json::Value>,
) -> ActixResult<web::Json<serde_json::Value>> {
    *tx_count.lock().await += 1;

    let response = serde_json::json!({
        "status": "received",
        "transaction_id": Uuid::new_v4().to_string()
    });

    Ok(web::Json(response))
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let matches = Command::new("ContainerLab Mining Simulation")
        .version("0.1.0")
        .about("Simulate PolyTorus mining in ContainerLab environment")
        .arg(
            Arg::new("nodes")
                .short('n')
                .long("nodes")
                .value_name("NUMBER")
                .help("Number of nodes to simulate")
                .default_value("4"),
        )
        .arg(
            Arg::new("miners")
                .short('m')
                .long("miners")
                .value_name("NUMBER")
                .help("Number of miner nodes")
                .default_value("2"),
        )
        .arg(
            Arg::new("duration")
                .short('d')
                .long("duration")
                .value_name("SECONDS")
                .help("Simulation duration in seconds")
                .default_value("600"),
        )
        .get_matches();

    let config = ContainerLabConfig {
        num_nodes: matches.get_one::<String>("nodes").unwrap().parse().unwrap(),
        num_miners: matches
            .get_one::<String>("miners")
            .unwrap()
            .parse()
            .unwrap(),
        simulation_duration: matches
            .get_one::<String>("duration")
            .unwrap()
            .parse()
            .unwrap(),
        ..Default::default()
    };

    println!("⛏️  ContainerLab Mining Simulation");
    println!("==================================");
    println!("📊 Configuration:");
    println!("   Total Nodes: {}", config.num_nodes);
    println!("   Miner Nodes: {}", config.num_miners);
    println!("   Duration: {} seconds", config.simulation_duration);
    println!("   Mining Interval: {} ms", config.mining_interval);
    println!();

    let mut simulator = ContainerLabMiningSimulator::new(config);

    // Create mining wallets
    simulator.create_mining_wallets().await?;

    // Start nodes
    simulator.start_nodes().await?;

    // Start API servers
    simulator.start_api_servers().await?;

    println!("🌐 Node APIs available at:");
    for node in &simulator.nodes {
        let node_type = if node.config.is_miner {
            "Miner"
        } else {
            "Validator"
        };
        println!(
            "   {} ({}): http://127.0.0.1:{}",
            node.config.node_id, node_type, node.config.port
        );
    }
    println!();

    // Run simulation
    simulator.run_simulation().await?;

    // Print final statistics
    simulator.print_final_stats().await;

    Ok(())
}
