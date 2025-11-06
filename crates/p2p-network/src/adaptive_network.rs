//! Adaptive Network Methods
//!
//! Adaptive methods for the P2P network with automatic discovery and connection management.

use crate::WebRTCP2PNetwork;
use anyhow::Result;
use log::{debug, info, warn};
use std::time::Duration;
use tokio::time::sleep;
use traits::P2PNetworkLayer;

impl WebRTCP2PNetwork {
    /// Adaptive start method with automatic discovery
    pub async fn start_adaptive(&self) -> Result<()> {
        info!(
            "Starting Adaptive WebRTC P2P Network on {}",
            self.config.listen_addr
        );

        // Update stats
        {
            let mut stats = self.stats.lock().unwrap();
            stats.last_updated = Some(std::time::SystemTime::now());
        }

        // Start auto discovery service
        {
            let mut discovery = self.auto_discovery.write().await;
            discovery.start().await?;
            info!("Auto discovery service started");
        }

        // Wait a moment for discovery to initialize
        sleep(Duration::from_millis(500)).await;

        // Discover peers before connecting to bootstrap
        let discovered_peers = {
            let discovery = self.auto_discovery.read().await;
            discovery.get_discovered_peers().await
        };

        info!(
            "Discovered {} peers through auto discovery",
            discovered_peers.len()
        );

        // Add discovered peers to DHT
        for peer in &discovered_peers {
            self.dht.add_node(peer.node_id.clone(), peer.address).await;
        }

        // Start connection to bootstrap peers with enhanced strategy
        for peer_addr in &self.config.bootstrap_peers {
            match peer_addr.parse() {
                Ok(addr) => {
                    let peer_id = format!("bootstrap_{}", uuid::Uuid::new_v4());
                    match self
                        .connect_to_peer(peer_id.clone(), peer_addr.clone())
                        .await
                    {
                        Ok(_) => {
                            info!("Connected to bootstrap peer: {}", peer_addr);
                            // Add to auto discovery
                            let discovery = self.auto_discovery.read().await;
                            discovery.add_peer(peer_id, addr).await;
                        }
                        Err(e) => warn!("Failed to connect to bootstrap peer {}: {}", peer_addr, e),
                    }
                }
                Err(_) => warn!("Invalid bootstrap peer address: {}", peer_addr),
            }
        }

        // Adaptive peer connection strategy
        self.start_adaptive_peer_connection().await;

        // Start existing maintenance tasks
        self.start_keep_alive_task().await;
        self.start_maintenance_task().await;
        self.start_message_processing_task().await;

        // Start auto discovery maintenance
        self.start_discovery_maintenance().await;

        info!("Adaptive WebRTC P2P Network started successfully");

        // Wait for shutdown signal
        let mut shutdown_rx = {
            let mut rx_option = self.shutdown_rx.lock().unwrap();
            rx_option
                .take()
                .ok_or_else(|| anyhow::anyhow!("Shutdown receiver already taken"))?
        };

        // Block until shutdown signal received
        shutdown_rx.recv().await;
        info!("P2P Network shutdown signal received");

        Ok(())
    }

    /// Adaptive peer connection strategy
    async fn start_adaptive_peer_connection(&self) {
        let auto_discovery = self.auto_discovery.clone();
        let peers = self.peers.clone();
        let config = self.config.clone();
        let dht = self.dht.clone();
        let stats = self.stats.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(10));

            loop {
                interval.tick().await;

                // Get current peer count
                let current_peers = peers.read().await.len();
                let max_peers = config.max_peers;

                if current_peers < max_peers {
                    // Get discovered peers
                    let discovery = auto_discovery.read().await;
                    let discovered = discovery.get_discovered_peers().await;

                    for peer in discovered {
                        if current_peers >= max_peers {
                            break;
                        }

                        // Check if we're already connected
                        let peers_read = peers.read().await;
                        if !peers_read.contains_key(&peer.node_id) {
                            drop(peers_read); // Release the lock

                            info!("Attempting to connect to discovered peer: {}", peer.node_id);

                            // This would need to be implemented as a method that doesn't require &self
                            // For now, just add to DHT
                            dht.add_node(peer.node_id.clone(), peer.address).await;

                            // Update stats
                            {
                                let mut stats_guard = stats.lock().unwrap();
                                stats_guard.total_connections += 1;
                            }
                        }
                    }
                }

                // Adaptive connection strategy: connect to more peers if network is small
                if current_peers < 3 {
                    debug!(
                        "Network is small ({}), actively seeking more peers",
                        current_peers
                    );
                }
            }
        });
    }

    /// Start discovery maintenance task
    async fn start_discovery_maintenance(&self) {
        let auto_discovery = self.auto_discovery.clone();

        tokio::spawn(async move {
            let mut interval = tokio::time::interval(Duration::from_secs(60));

            loop {
                interval.tick().await;

                // Clean up old peers
                let discovery = auto_discovery.read().await;
                discovery.cleanup_old_peers().await;
            }
        });
    }

    /// Get discovered peers from auto discovery
    pub async fn get_discovered_peers(&self) -> Vec<crate::auto_discovery::SimplePeerInfo> {
        let discovery = self.auto_discovery.read().await;
        discovery.get_discovered_peers().await
    }

    /// Adaptive broadcast with better peer targeting
    pub async fn adaptive_broadcast_transaction(
        &self,
        transaction: &traits::UtxoTransaction,
    ) -> Result<()> {
        // Use normal broadcast first
        self.broadcast_transaction(transaction).await?;

        // Also try to broadcast to discovered peers that we might not be connected to
        let discovered_peers = self.get_discovered_peers().await;

        if !discovered_peers.is_empty() {
            debug!(
                "Adaptive broadcast: also considering {} discovered peers",
                discovered_peers.len()
            );

            // In a full implementation, we could establish temporary connections
            // or use other means to reach these peers
        }

        Ok(())
    }

    /// Adaptive peer statistics including discovery info
    pub async fn get_adaptive_network_stats(&self) -> AdaptiveNetworkStats {
        let base_stats = self.get_network_stats();
        let discovered_peers = self.get_discovered_peers().await;
        let dht_size = self.dht.size().await;
        let connected_peers = self.get_connected_peers().await;

        AdaptiveNetworkStats {
            base_stats,
            discovered_peers_count: discovered_peers.len(),
            dht_nodes_count: dht_size,
            connected_peers_count: connected_peers.len(),
            discovery_efficiency: if !discovered_peers.is_empty() {
                connected_peers.len() as f32 / discovered_peers.len() as f32
            } else {
                0.0
            },
        }
    }
}

/// Adaptive network statistics
#[derive(Debug)]
pub struct AdaptiveNetworkStats {
    pub base_stats: crate::NetworkStats,
    pub discovered_peers_count: usize,
    pub dht_nodes_count: usize,
    pub connected_peers_count: usize,
    pub discovery_efficiency: f32, // Ratio of connected to discovered peers
}
