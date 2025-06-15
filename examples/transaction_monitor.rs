//! Transaction Monitor
//! 
//! A simple monitoring tool to observe transaction flow between nodes

use clap::{Arg, App};
use reqwest::Client;
use serde_json::Value;
use std::collections::HashMap;
use std::time::Duration;
use tokio::time::{interval, sleep};

#[derive(Debug, Clone)]
pub struct NodeStats {
    pub node_id: String,
    pub endpoint: String,
    pub transactions_sent: u64,
    pub transactions_received: u64,
    pub block_height: u64,
    pub is_online: bool,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

pub struct TransactionMonitor {
    client: Client,
    nodes: Vec<String>,
    stats: HashMap<String, NodeStats>,
}

impl TransactionMonitor {
    pub fn new(base_port: u16, num_nodes: usize) -> Self {
        let client = Client::new();
        let nodes = (0..num_nodes)
            .map(|i| format!("http://127.0.0.1:{}", base_port + i as u16))
            .collect();
            
        Self {
            client,
            nodes,
            stats: HashMap::new(),
        }
    }

    pub async fn start_monitoring(&mut self, interval_seconds: u64) -> Result<(), Box<dyn std::error::Error>> {
        println!("🔍 Starting Transaction Monitor");
        println!("================================");
        println!("Monitoring {} nodes", self.nodes.len());
        println!("Update interval: {} seconds", interval_seconds);
        println!();

        let mut interval = interval(Duration::from_secs(interval_seconds));
        
        loop {
            interval.tick().await;
            self.update_stats().await;
            self.display_stats();
            println!();
        }
    }

    async fn update_stats(&mut self) {
        for (i, endpoint) in self.nodes.iter().enumerate() {
            let node_id = format!("node-{}", i);
            
            let mut stats = NodeStats {
                node_id: node_id.clone(),
                endpoint: endpoint.clone(),
                transactions_sent: 0,
                transactions_received: 0,
                block_height: 0,
                is_online: false,
                last_updated: chrono::Utc::now(),
            };

            // Try to get status
            if let Ok(status) = self.fetch_node_status(endpoint).await {
                stats.is_online = true;
                if let Some(height) = status.get("block_height").and_then(|v| v.as_u64()) {
                    stats.block_height = height;
                }
                if let Some(tx_count) = status.get("total_transactions").and_then(|v| v.as_u64()) {
                    stats.transactions_received = tx_count;
                }
            }

            // Try to get node-specific stats
            if let Ok(node_stats) = self.fetch_node_stats(endpoint).await {
                if let Some(tx_sent) = node_stats.get("transactions_sent").and_then(|v| v.as_u64()) {
                    stats.transactions_sent = tx_sent;
                }
                if let Some(tx_received) = node_stats.get("transactions_received").and_then(|v| v.as_u64()) {
                    stats.transactions_received = tx_received;
                }
            }

            self.stats.insert(node_id, stats);
        }
    }

    async fn fetch_node_status(&self, endpoint: &str) -> Result<Value, Box<dyn std::error::Error>> {
        let url = format!("{}/status", endpoint);
        let response = self.client
            .get(&url)
            .timeout(Duration::from_secs(5))
            .send()
            .await?;
        
        let json: Value = response.json().await?;
        Ok(json)
    }

    async fn fetch_node_stats(&self, endpoint: &str) -> Result<Value, Box<dyn std::error::Error>> {
        let url = format!("{}/stats", endpoint);
        let response = self.client
            .get(&url)
            .timeout(Duration::from_secs(5))
            .send()
            .await?;
        
        let json: Value = response.json().await?;
        Ok(json)
    }

    fn display_stats(&self) {
        let now = chrono::Utc::now();
        println!("📊 Network Statistics - {}", now.format("%Y-%m-%d %H:%M:%S UTC"));
        println!("┌─────────┬────────┬──────────┬──────────┬────────────┬─────────────┐");
        println!("│ Node    │ Status │ TX Sent  │ TX Recv  │ Block Height│ Last Update │");
        println!("├─────────┼────────┼──────────┼──────────┼────────────┼─────────────┤");

        let mut total_sent = 0u64;
        let mut total_received = 0u64;
        let mut online_nodes = 0;

        for i in 0..self.nodes.len() {
            let node_id = format!("node-{}", i);
            if let Some(stats) = self.stats.get(&node_id) {
                let status = if stats.is_online { "🟢 Online " } else { "🔴 Offline" };
                let last_update = if stats.is_online {
                    let duration = now - stats.last_updated;
                    if duration.num_seconds() < 60 {
                        format!("{}s ago", duration.num_seconds())
                    } else {
                        format!("{}m ago", duration.num_minutes())
                    }
                } else {
                    "N/A".to_string()
                };

                println!(
                    "│ {:7} │ {:6} │ {:8} │ {:8} │ {:10} │ {:11} │",
                    stats.node_id,
                    status,
                    stats.transactions_sent,
                    stats.transactions_received,
                    stats.block_height,
                    last_update
                );

                if stats.is_online {
                    online_nodes += 1;
                    total_sent += stats.transactions_sent;
                    total_received += stats.transactions_received;
                }
            } else {
                println!(
                    "│ {:7} │ {:6} │ {:8} │ {:8} │ {:10} │ {:11} │",
                    node_id, "🔴 Unknown", "N/A", "N/A", "N/A", "N/A"
                );
            }
        }

        println!("├─────────┼────────┼──────────┼──────────┼────────────┼─────────────┤");
        println!(
            "│ Total   │ {:2}/{:<2} ON │ {:8} │ {:8} │ {:10} │ {:11} │",
            online_nodes,
            self.nodes.len(),
            total_sent,
            total_received,
            "N/A",
            "Summary"
        );
        println!("└─────────┴────────┴──────────┴──────────┴────────────┴─────────────┘");

        // Network health indicators
        println!("🏥 Network Health:");
        let health_percentage = (online_nodes as f64 / self.nodes.len() as f64) * 100.0;
        println!("   Network Connectivity: {:.1}% ({}/{} nodes online)", 
                health_percentage, online_nodes, self.nodes.len());
        
        if total_sent > 0 {
            let propagation_rate = (total_received as f64 / total_sent as f64) * 100.0;
            println!("   Transaction Propagation: {:.1}% ({} received / {} sent)", 
                    propagation_rate, total_received, total_sent);
        }

        // Show recent activity
        if let Some(max_height) = self.stats.values()
            .filter(|s| s.is_online)
            .map(|s| s.block_height)
            .max() 
        {
            let synced_nodes = self.stats.values()
                .filter(|s| s.is_online && s.block_height == max_height)
                .count();
            println!("   Block Synchronization: {}/{} nodes at height {}", 
                    synced_nodes, online_nodes, max_height);
        }
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let matches = App::new("Transaction Monitor")
        .version("0.1.0")
        .about("Monitor transaction flow between PolyTorus nodes")
        .arg(
            Arg::with_name("nodes")
                .short("n")
                .long("nodes")
                .value_name("NUMBER")
                .help("Number of nodes to monitor")
                .takes_value(true)
                .default_value("4"),
        )
        .arg(
            Arg::with_name("base-port")
                .short("p")
                .long("base-port")
                .value_name("PORT")
                .help("Base HTTP port number")
                .takes_value(true)
                .default_value("9000"),
        )
        .arg(
            Arg::with_name("interval")
                .short("i")
                .long("interval")
                .value_name("SECONDS")
                .help("Update interval in seconds")
                .takes_value(true)
                .default_value("10"),
        )
        .get_matches();

    let num_nodes: usize = matches.value_of("nodes").unwrap().parse()?;
    let base_port: u16 = matches.value_of("base-port").unwrap().parse()?;
    let interval: u64 = matches.value_of("interval").unwrap().parse()?;

    let mut monitor = TransactionMonitor::new(base_port, num_nodes);
    
    println!("🚀 PolyTorus Transaction Monitor");
    println!("=================================");
    println!("Monitoring ports: {} - {}", base_port, base_port + num_nodes as u16 - 1);
    println!("Press Ctrl+C to stop monitoring");
    println!();
    
    // Initial stats fetch
    monitor.update_stats().await;
    monitor.display_stats();
    
    // Wait a bit then start continuous monitoring
    sleep(Duration::from_secs(2)).await;
    monitor.start_monitoring(interval).await?;

    Ok(())
}
