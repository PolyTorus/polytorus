//! Real P2P Multi-Node Transaction Simulation
//!
//! This example demonstrates real P2P communication between PolyTorus nodes
//! without using HTTP APIs, showcasing actual blockchain network behavior.

use std::{
    net::SocketAddr,
    sync::Arc,
    time::Duration,
};

use clap::{Arg, Command};
use polytorus::{
    config::DataContext,
    crypto::transaction::Transaction,
    modular::{default_modular_config, UnifiedModularOrchestrator},
    network::p2p_enhanced::{EnhancedP2PNode, NetworkCommand, NetworkEvent},
    Result,
};
use serde::{Deserialize, Serialize};
use bincode;
use tokio::{
    sync::{mpsc, Mutex},
    time::{interval, sleep},
};
// Remove unused import

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2PNodeConfig {
    pub node_id: String,
    pub p2p_addr: SocketAddr,
    pub data_dir: String,
    pub bootstrap_peers: Vec<SocketAddr>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct P2PSimulationConfig {
    pub num_nodes: usize,
    pub base_p2p_port: u16,
    pub transaction_interval: u64, // milliseconds
    pub transactions_per_batch: usize,
    pub simulation_duration: u64, // seconds
}

impl Default for P2PSimulationConfig {
    fn default() -> Self {
        Self {
            num_nodes: 4,
            base_p2p_port: 8000,
            transaction_interval: 5000, // 5 seconds
            transactions_per_batch: 3,
            simulation_duration: 300, // 5 minutes
        }
    }
}

#[derive(Clone)]
pub struct P2PNodeInstance {
    pub config: P2PNodeConfig,
    pub orchestrator: Arc<UnifiedModularOrchestrator>,
    pub p2p_command_tx: mpsc::UnboundedSender<NetworkCommand>,
    pub tx_count: Arc<Mutex<u64>>,
    pub rx_count: Arc<Mutex<u64>>,
}

pub struct P2PMultiNodeSimulator {
    config: P2PSimulationConfig,
    nodes: Vec<P2PNodeInstance>,
    is_running: Arc<Mutex<bool>>,
}

impl P2PMultiNodeSimulator {
    pub fn new(config: P2PSimulationConfig) -> Self {
        Self {
            config,
            nodes: Vec::new(),
            is_running: Arc::new(Mutex::new(false)),
        }
    }

    /// Generate P2P node configurations with real network addresses
    pub fn generate_node_configs(&self) -> Vec<P2PNodeConfig> {
        let mut configs = Vec::new();
        let mut bootstrap_peers = Vec::new();

        // Create all node addresses first
        for i in 0..self.config.num_nodes {
            let addr = SocketAddr::from(([127, 0, 0, 1], self.config.base_p2p_port + i as u16));
            bootstrap_peers.push(addr);
        }

        for i in 0..self.config.num_nodes {
            let node_id = format!("p2p-node-{}", i);
            let p2p_addr = bootstrap_peers[i];
            
            // Each node connects to all other nodes as bootstrap peers
            let mut node_bootstrap_peers = bootstrap_peers.clone();
            node_bootstrap_peers.remove(i); // Don't include self

            let config = P2PNodeConfig {
                node_id: node_id.clone(),
                p2p_addr,
                data_dir: format!("./data/simulation/p2p_node_{}", i),
                bootstrap_peers: node_bootstrap_peers,
            };

            configs.push(config);
        }

        configs
    }

    /// Initialize all P2P nodes with real network connections
    pub async fn initialize_nodes(&mut self) -> Result<()> {
        let node_configs = self.generate_node_configs();
        println!("🚀 Initializing {} P2P nodes with real network connections...", node_configs.len());

        for (_i, config) in node_configs.into_iter().enumerate() {
            // Create data context for the node
            let data_context = DataContext::new(config.data_dir.clone().into());
            
            // Create modular config with P2P settings
            let mut modular_config = default_modular_config();
            modular_config.data_availability.network_config.listen_addr = config.p2p_addr.to_string();
            modular_config.data_availability.network_config.bootstrap_peers = 
                config.bootstrap_peers.iter().map(|addr| addr.to_string()).collect();

            // Create unified modular orchestrator with defaults
            let orchestrator = Arc::new(
                UnifiedModularOrchestrator::create_and_start_with_defaults(
                    modular_config,
                    data_context,
                ).await?
            );

            // Create real P2P node
            let (mut p2p_node, event_rx, command_tx) = EnhancedP2PNode::new(
                config.p2p_addr,
                config.bootstrap_peers.clone(),
            )?;

            // Start P2P node in background using blocking task
            let node_id_clone = config.node_id.clone();
            std::thread::spawn(move || {
                let rt = tokio::runtime::Runtime::new().unwrap();
                rt.block_on(async move {
                    if let Err(e) = p2p_node.run().await {
                        eprintln!("❌ P2P node {} error: {}", node_id_clone, e);
                    }
                });
            });

            // Start event processing for this node
            let orchestrator_clone = orchestrator.clone();
            let node_id_clone = config.node_id.clone();
            tokio::spawn(async move {
                Self::process_p2p_events(event_rx, orchestrator_clone, node_id_clone).await;
            });

            let node_instance = P2PNodeInstance {
                config: config.clone(),
                orchestrator,
                p2p_command_tx: command_tx,
                tx_count: Arc::new(Mutex::new(0)),
                rx_count: Arc::new(Mutex::new(0)),
            };

            self.nodes.push(node_instance);

            println!("✅ P2P Node {} initialized on {}", config.node_id, config.p2p_addr);
            
            // Small delay between node startups to avoid port conflicts
            sleep(Duration::from_millis(500)).await;
        }

        // Wait for P2P connections to establish
        println!("🔗 Waiting for P2P connections to establish...");
        sleep(Duration::from_secs(5)).await;

        Ok(())
    }

    /// Process P2P network events for a node
    async fn process_p2p_events(
        mut event_rx: mpsc::UnboundedReceiver<NetworkEvent>,
        orchestrator: Arc<UnifiedModularOrchestrator>,
        node_id: String,
    ) {
        while let Some(event) = event_rx.recv().await {
            match event {
                NetworkEvent::TransactionReceived(tx, peer_id) => {
                    println!("📥 Node {} received transaction {} from peer {}", node_id, tx.id, peer_id);
                    
                    // Process transaction through the modular orchestrator
                    // Serialize transaction to bytes for processing
                    match bincode::serialize(&*tx) {
                        Ok(tx_bytes) => {
                            if let Err(e) = orchestrator.execute_transaction(tx_bytes).await {
                                eprintln!("❌ Failed to process transaction on {}: {}", node_id, e);
                            }
                        }
                        Err(e) => {
                            eprintln!("❌ Failed to serialize transaction on {}: {}", node_id, e);
                        }
                    }
                }
                NetworkEvent::BlockReceived(block, peer_id) => {
                    println!("📦 Node {} received block {} from peer {}", node_id, block.get_hash(), peer_id);
                    
                    // Process block through the modular orchestrator
                    // Note: For now, we'll log the block received but skip processing
                    // since block type conversion needs proper implementation
                    println!("🔄 Block processing skipped for P2P demo - block {} received", block.get_hash());
                }
                NetworkEvent::PeerConnected(peer_id) => {
                    println!("🤝 Node {} connected to peer {}", node_id, peer_id);
                }
                NetworkEvent::PeerDisconnected(peer_id) => {
                    println!("👋 Node {} disconnected from peer {}", node_id, peer_id);
                }
                _ => {
                    // Handle other network events
                }
            }
        }
    }

    /// Create and broadcast a transaction using real P2P communication
    pub async fn create_and_broadcast_transaction(
        &self,
        sender_node: &P2PNodeInstance,
        receiver_node: &P2PNodeInstance,
        tx_id: u64,
    ) -> Result<()> {
        // Create a real transaction
        let transaction = Transaction::new_coinbase(
            format!("wallet_{}", receiver_node.config.node_id),
            format!("P2P Transaction {} from {} to {}", tx_id, sender_node.config.node_id, receiver_node.config.node_id),
        )?;

        println!(
            "🚀 Broadcasting transaction {} from {} via real P2P network",
            transaction.id, sender_node.config.node_id
        );

        // Broadcast transaction via real P2P network
        let command = NetworkCommand::BroadcastTransaction(transaction.clone());
        
        if let Err(e) = sender_node.p2p_command_tx.send(command) {
            eprintln!("❌ Failed to broadcast transaction via P2P: {}", e);
            return Err(anyhow::anyhow!("P2P broadcast failed: {}", e));
        }

        // Update transaction counts
        {
            let mut tx_count = sender_node.tx_count.lock().await;
            *tx_count += 1;
        }

        println!("✅ Transaction {} broadcasted via P2P from {}", transaction.id, sender_node.config.node_id);
        Ok(())
    }

    /// Run the P2P simulation with real network communication
    pub async fn run_simulation(&mut self) -> Result<()> {
        // Initialize all nodes
        self.initialize_nodes().await?;

        println!("🎯 Starting P2P multi-node simulation...");
        println!("📊 Simulation parameters:");
        println!("   • Nodes: {}", self.config.num_nodes);
        println!("   • Duration: {}s", self.config.simulation_duration);
        println!("   • Transaction interval: {}ms", self.config.transaction_interval);
        println!("   • Transactions per batch: {}", self.config.transactions_per_batch);

        // Set running flag
        {
            let mut is_running = self.is_running.lock().await;
            *is_running = true;
        }

        // Create transaction interval timer
        let mut transaction_timer = interval(Duration::from_millis(self.config.transaction_interval));
        let mut transaction_id = 1;

        let start_time = std::time::Instant::now();
        let simulation_duration = Duration::from_secs(self.config.simulation_duration);

        // Main simulation loop
        loop {
            tokio::select! {
                _ = transaction_timer.tick() => {
                    // Check if simulation should continue
                    if start_time.elapsed() >= simulation_duration {
                        break;
                    }

                    // Create batch of transactions
                    for _ in 0..self.config.transactions_per_batch {
                        if self.nodes.len() < 2 {
                            continue;
                        }

                        // Select random sender and receiver
                        let sender_idx = transaction_id as usize % self.nodes.len();
                        let mut receiver_idx = (transaction_id as usize + 1) % self.nodes.len();
                        
                        // Ensure sender and receiver are different
                        if sender_idx == receiver_idx {
                            receiver_idx = (receiver_idx + 1) % self.nodes.len();
                        }

                        let sender_node = &self.nodes[sender_idx];
                        let receiver_node = &self.nodes[receiver_idx];

                        if let Err(e) = self.create_and_broadcast_transaction(
                            sender_node,
                            receiver_node,
                            transaction_id,
                        ).await {
                            eprintln!("❌ Failed to create transaction {}: {}", transaction_id, e);
                        }

                        transaction_id += 1;
                    }
                }
            }
        }

        println!("🏁 P2P simulation completed!");
        self.print_final_statistics().await;

        Ok(())
    }

    /// Print final simulation statistics
    async fn print_final_statistics(&self) {
        println!("\n📊 Final P2P Simulation Statistics:");
        println!("═══════════════════════════════════");
        
        let mut total_tx_sent = 0;
        let mut total_tx_received = 0;

        for node in &self.nodes {
            let tx_count = *node.tx_count.lock().await;
            let rx_count = *node.rx_count.lock().await;
            
            println!("🔸 {}: {} sent, {} received", node.config.node_id, tx_count, rx_count);
            
            total_tx_sent += tx_count;
            total_tx_received += rx_count;
        }

        println!("═══════════════════════════════════");
        println!("📈 Total transactions sent: {}", total_tx_sent);
        println!("📉 Total transactions received: {}", total_tx_received);
        println!("🌐 Network efficiency: {:.1}%", 
            if total_tx_sent > 0 { 
                (total_tx_received as f64 / total_tx_sent as f64) * 100.0 
            } else { 
                0.0 
            }
        );
    }
}

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();

    let matches = Command::new("PolyTorus P2P Multi-Node Simulation")
        .version("1.0")
        .author("PolyTorus Team")
        .about("Simulates real P2P communication between PolyTorus nodes")
        .arg(
            Arg::new("nodes")
                .long("nodes")
                .value_name("COUNT")
                .help("Number of nodes to simulate")
                .default_value("4"),
        )
        .arg(
            Arg::new("duration")
                .long("duration")
                .value_name("SECONDS")
                .help("Simulation duration in seconds")
                .default_value("300"),
        )
        .arg(
            Arg::new("interval")
                .long("interval")
                .value_name("MILLISECONDS")
                .help("Transaction interval in milliseconds")
                .default_value("5000"),
        )
        .arg(
            Arg::new("p2p-port")
                .long("p2p-port")
                .value_name("PORT")
                .help("Base P2P port")
                .default_value("8000"),
        )
        .get_matches();

    let config = P2PSimulationConfig {
        num_nodes: matches.get_one::<String>("nodes").unwrap().parse()?,
        base_p2p_port: matches.get_one::<String>("p2p-port").unwrap().parse()?,
        transaction_interval: matches.get_one::<String>("interval").unwrap().parse()?,
        transactions_per_batch: 2,
        simulation_duration: matches.get_one::<String>("duration").unwrap().parse()?,
    };

    println!("🚀 Starting PolyTorus P2P Multi-Node Simulation");
    println!("================================================");

    let mut simulator = P2PMultiNodeSimulator::new(config);
    simulator.run_simulation().await?;

    println!("✅ P2P Simulation completed successfully!");
    Ok(())
}