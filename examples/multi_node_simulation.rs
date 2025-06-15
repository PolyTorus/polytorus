//! Multi-Node Transaction Simulation
//!
//! This example demonstrates how to run multiple PolyTorus nodes locally
//! and simulate transaction propagation across the network.

use std::path::PathBuf;
use std::sync::Arc;
use std::time::Duration;

use actix_web::{
    web,
    App,
    HttpServer,
    Result as ActixResult,
};
use clap::{
    App as ClapApp,
    Arg,
};
use polytorus::config::{
    ConfigManager,
    DataContext,
};
use polytorus::modular::{
    default_modular_config,
    UnifiedModularOrchestrator,
};
use polytorus::Result;
use reqwest::Client;
use serde::{
    Deserialize,
    Serialize,
};
use tokio::sync::Mutex;
use tokio::time::{
    interval,
    sleep,
};
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeConfig {
    pub node_id: String,
    pub port: u16,     // HTTP API port
    pub p2p_port: u16, // P2P network port
    pub data_dir: String,
    pub bootstrap_peers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionRequest {
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub nonce: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionResponse {
    pub status: String,
    pub transaction_id: String,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationConfig {
    pub num_nodes: usize,
    pub base_port: u16,
    pub base_p2p_port: u16,
    pub transaction_interval: u64, // milliseconds
    pub transactions_per_batch: usize,
    pub simulation_duration: u64, // seconds
}

impl Default for SimulationConfig {
    fn default() -> Self {
        Self {
            num_nodes: 4,
            base_port: 9000,
            base_p2p_port: 8000,
            transaction_interval: 5000, // 5 seconds
            transactions_per_batch: 3,
            simulation_duration: 300, // 5 minutes
        }
    }
}

#[derive(Clone)]
pub struct NodeInstance {
    pub config: NodeConfig,
    pub orchestrator: Arc<UnifiedModularOrchestrator>,
    pub tx_count: Arc<Mutex<u64>>,
    pub rx_count: Arc<Mutex<u64>>,
    pub http_client: Client,
}

pub struct MultiNodeSimulator {
    config: SimulationConfig,
    nodes: Vec<NodeInstance>,
    is_running: Arc<Mutex<bool>>,
}

impl MultiNodeSimulator {
    pub fn new(config: SimulationConfig) -> Self {
        Self {
            config,
            nodes: Vec::new(),
            is_running: Arc::new(Mutex::new(false)),
        }
    }

    /// Generate node configurations
    pub fn generate_node_configs(&self) -> Vec<NodeConfig> {
        let mut configs = Vec::new();

        for i in 0..self.config.num_nodes {
            let node_id = format!("node-{}", i);
            let port = self.config.base_port + i as u16;
            let p2p_port = self.config.base_p2p_port + i as u16;
            let data_dir = format!("./data/simulation/{}", node_id);

            // Generate bootstrap peers (connect to previous nodes)
            let mut bootstrap_peers = Vec::new();
            for j in 0..i {
                let peer_port = self.config.base_p2p_port + j as u16;
                bootstrap_peers.push(format!("127.0.0.1:{}", peer_port));
            }

            configs.push(NodeConfig {
                node_id,
                port,
                p2p_port,
                data_dir,
                bootstrap_peers,
            });
        }

        configs
    }

    /// Initialize and start all nodes
    pub async fn start_nodes(&mut self) -> Result<()> {
        println!(
            "🚀 Starting {} nodes for simulation...",
            self.config.num_nodes
        );

        let node_configs = self.generate_node_configs();

        for (i, node_config) in node_configs.iter().enumerate() {
            println!("📡 Starting node {} ({})", i + 1, node_config.node_id);

            // Create data directory
            let data_context = DataContext::new(PathBuf::from(node_config.data_dir.clone()));
            data_context.ensure_directories()?;

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
                data_context,
            )
            .await?;

            let node_instance = NodeInstance {
                config: node_config.clone(),
                orchestrator: Arc::new(orchestrator),
                tx_count: Arc::new(Mutex::new(0)),
                rx_count: Arc::new(Mutex::new(0)),
                http_client: Client::new(),
            };

            self.nodes.push(node_instance);

            // Small delay between node starts to avoid port conflicts
            sleep(Duration::from_millis(1000)).await;
        }

        // Wait for network to stabilize
        println!("⏳ Waiting for network to stabilize...");
        sleep(Duration::from_secs(5)).await;

        println!("✅ All nodes started successfully!");
        Ok(())
    }

    /// Start the HTTP API servers for each node
    pub async fn start_api_servers(&self) -> Result<()> {
        println!("🌐 Starting HTTP API servers...");

        for node in &self.nodes {
            let node_config = node.config.clone();
            let orchestrator = node.orchestrator.clone();
            let tx_count = node.tx_count.clone();
            let rx_count = node.rx_count.clone();

            tokio::spawn(async move {
                let server = HttpServer::new(move || {
                    let orchestrator = orchestrator.clone();
                    let tx_count = tx_count.clone();
                    let rx_count = rx_count.clone();

                    App::new()
                        .app_data(web::Data::new(orchestrator))
                        .app_data(web::Data::new(tx_count))
                        .app_data(web::Data::new(rx_count))
                        .route("/status", web::get().to(get_node_status))
                        .route("/transaction", web::post().to(submit_transaction))
                        .route("/stats", web::get().to(get_node_stats))
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

    /// Start transaction simulation
    pub async fn start_simulation(&self) -> Result<()> {
        println!("🎯 Starting transaction simulation...");
        *self.is_running.lock().await = true;

        let is_running = self.is_running.clone();
        let nodes = self.nodes.clone();
        let config = self.config.clone();

        // Transaction generator task
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_millis(config.transaction_interval));
            let mut tx_counter = 0u64;

            while *is_running.lock().await {
                interval.tick().await;

                // Generate transactions
                for _ in 0..config.transactions_per_batch {
                    let sender_idx = tx_counter as usize % nodes.len();
                    let receiver_idx = (tx_counter as usize + 1) % nodes.len();

                    if let Err(e) = Self::generate_and_submit_transaction(
                        &nodes[sender_idx],
                        &nodes[receiver_idx],
                        tx_counter,
                    )
                    .await
                    {
                        eprintln!("Failed to generate transaction {}: {}", tx_counter, e);
                    }

                    tx_counter += 1;
                }

                // Print progress
                if tx_counter % 10 == 0 {
                    println!("📊 Generated {} transactions", tx_counter);
                }
            }
        });

        // Statistics reporter task
        let nodes_clone = self.nodes.clone();
        let is_running_clone = self.is_running.clone();
        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(30));

            while *is_running_clone.lock().await {
                interval.tick().await;
                Self::print_network_statistics(&nodes_clone).await;
            }
        });

        // Run simulation for specified duration
        sleep(Duration::from_secs(self.config.simulation_duration)).await;

        println!("⏹️  Simulation completed!");
        *self.is_running.lock().await = false;

        Ok(())
    }

    async fn generate_and_submit_transaction(
        sender_node: &NodeInstance,
        receiver_node: &NodeInstance,
        tx_id: u64,
    ) -> Result<()> {
        // Create transaction request
        let tx_request = TransactionRequest {
            from: format!("wallet_{}", sender_node.config.node_id),
            to: format!("wallet_{}", receiver_node.config.node_id),
            amount: 100 + (tx_id % 900), // Random amount between 100-1000
            nonce: Some(tx_id),
        };

        // First, submit to sender node's /send endpoint to record it as sent
        let sender_url = format!("http://127.0.0.1:{}/send", sender_node.config.port);
        match sender_node
            .http_client
            .post(&sender_url)
            .json(&tx_request)
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    if let Ok(_tx_response) = response.json::<TransactionResponse>().await {
                        println!(
                            "📤 Transaction {} sent from {}: {} -> {} (amount: {})",
                            tx_id,
                            sender_node.config.node_id,
                            tx_request.from,
                            tx_request.to,
                            tx_request.amount
                        );
                    }
                } else {
                    eprintln!(
                        "❌ Failed to send transaction to sender node: {}",
                        response.status()
                    );
                }
            }
            Err(e) => {
                eprintln!("❌ HTTP error when sending to sender node: {}", e);
            }
        }

        // Then, submit to receiver node's /transaction endpoint to record it as received
        let receiver_url = format!("http://127.0.0.1:{}/transaction", receiver_node.config.port);
        match receiver_node
            .http_client
            .post(&receiver_url)
            .json(&tx_request)
            .send()
            .await
        {
            Ok(response) => {
                if response.status().is_success() {
                    if let Ok(_tx_response) = response.json::<TransactionResponse>().await {
                        println!(
                            "� Transaction {} received by {}: {} -> {} (amount: {})",
                            tx_id,
                            receiver_node.config.node_id,
                            tx_request.from,
                            tx_request.to,
                            tx_request.amount
                        );
                    }
                } else {
                    eprintln!(
                        "❌ Failed to submit transaction to receiver node: {}",
                        response.status()
                    );
                }
            }
            Err(e) => {
                eprintln!("❌ HTTP error when submitting to receiver node: {}", e);
            }
        }

        Ok(())
    }

    async fn print_network_statistics(nodes: &[NodeInstance]) {
        println!("\n📈 Network Statistics:");
        println!("======================");

        let mut total_tx = 0u64;
        let mut total_rx = 0u64;

        for node in nodes {
            let tx_count = *node.tx_count.lock().await;
            let rx_count = *node.rx_count.lock().await;

            println!(
                "📡 {}: TX: {}, RX: {}",
                node.config.node_id, tx_count, rx_count
            );

            total_tx += tx_count;
            total_rx += rx_count;
        }

        println!("📊 Total: TX: {}, RX: {}", total_tx, total_rx);
        println!();
    }

    pub async fn stop(&self) -> Result<()> {
        println!("🛑 Stopping simulation...");
        *self.is_running.lock().await = false;

        for node in &self.nodes {
            // Stop orchestrator
            // Note: Add actual stop method to orchestrator if needed
            println!("⏹️  Stopping node {}", node.config.node_id);
        }

        println!("✅ Simulation stopped!");
        Ok(())
    }
}

// HTTP API handlers
async fn get_node_status(
    orchestrator: web::Data<Arc<UnifiedModularOrchestrator>>,
) -> ActixResult<web::Json<serde_json::Value>> {
    let state = orchestrator.get_state().await;
    let metrics = orchestrator.get_metrics().await;

    let status = serde_json::json!({
        "status": "running",
        "block_height": state.current_block_height,
        "is_running": state.is_running,
        "total_transactions": metrics.total_transactions_processed,
        "total_blocks": metrics.total_blocks_processed,
        "error_rate": metrics.error_rate
    });

    Ok(web::Json(status))
}

async fn submit_transaction(
    _orchestrator: web::Data<Arc<UnifiedModularOrchestrator>>,
    tx_count: web::Data<Arc<Mutex<u64>>>,
    _transaction: web::Json<serde_json::Value>,
) -> ActixResult<web::Json<serde_json::Value>> {
    *tx_count.lock().await += 1;

    let response = serde_json::json!({
        "status": "accepted",
        "transaction_id": Uuid::new_v4().to_string()
    });

    Ok(web::Json(response))
}

async fn get_node_stats(
    tx_count: web::Data<Arc<Mutex<u64>>>,
    rx_count: web::Data<Arc<Mutex<u64>>>,
) -> ActixResult<web::Json<serde_json::Value>> {
    let tx = *tx_count.lock().await;
    let rx = *rx_count.lock().await;

    let stats = serde_json::json!({
        "transactions_sent": tx,
        "transactions_received": rx,
        "timestamp": chrono::Utc::now().to_rfc3339()
    });

    Ok(web::Json(stats))
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let matches = ClapApp::new("Multi-Node Simulation")
        .version("0.1.0")
        .about("Simulate multiple PolyTorus nodes for transaction testing")
        .arg(
            Arg::with_name("nodes")
                .short("n")
                .long("nodes")
                .value_name("NUMBER")
                .help("Number of nodes to simulate")
                .default_value("4"),
        )
        .arg(
            Arg::with_name("duration")
                .short("d")
                .long("duration")
                .value_name("SECONDS")
                .help("Simulation duration in seconds")
                .default_value("300"),
        )
        .arg(
            Arg::with_name("interval")
                .short("i")
                .long("interval")
                .value_name("MILLISECONDS")
                .help("Transaction generation interval")
                .default_value("5000"),
        )
        .get_matches();

    let config = SimulationConfig {
        num_nodes: matches.value_of("nodes").unwrap().parse().unwrap(),
        simulation_duration: matches.value_of("duration").unwrap().parse().unwrap(),
        transaction_interval: matches.value_of("interval").unwrap().parse().unwrap(),
        ..Default::default()
    };

    println!("🎭 Multi-Node Transaction Simulation");
    println!("=====================================");
    println!("📊 Configuration:");
    println!("   Nodes: {}", config.num_nodes);
    println!("   Duration: {} seconds", config.simulation_duration);
    println!("   TX Interval: {} ms", config.transaction_interval);
    println!("   Base Port: {}", config.base_port);
    println!("   Base P2P Port: {}", config.base_p2p_port);
    println!();

    let mut simulator = MultiNodeSimulator::new(config);

    // Start nodes
    simulator.start_nodes().await?;

    // Start API servers
    simulator.start_api_servers().await?;

    println!("🌐 Node APIs available at:");
    for node in &simulator.nodes {
        println!(
            "   {}: http://127.0.0.1:{}",
            node.config.node_id, node.config.port
        );
    }
    println!();

    // Start simulation
    simulator.start_simulation().await?;

    // Final statistics
    MultiNodeSimulator::print_network_statistics(&simulator.nodes).await;

    // Cleanup
    simulator.stop().await?;

    Ok(())
}
