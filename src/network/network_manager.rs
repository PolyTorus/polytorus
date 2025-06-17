//! Network Management Module
//!
//! Provides comprehensive network management features including node health monitoring,
//! connection management, and network topology optimization.

use std::{
    collections::{HashMap, HashSet},
    net::SocketAddr,
    sync::{Arc, Mutex},
    time::{Duration, SystemTime},
};

use serde::{Deserialize, Serialize};
use tokio::{
    sync::{mpsc, RwLock},
    time::interval,
};

use crate::{network::PeerId, Result};

/// Network health status
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum NodeHealth {
    Healthy,
    Degraded,
    Unhealthy,
    Disconnected,
}

/// Network topology information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkTopology {
    pub total_nodes: usize,
    pub connected_peers: usize,
    pub healthy_peers: usize,
    pub degraded_peers: usize,
    pub unhealthy_peers: usize,
    pub average_latency: Duration,
    pub network_diameter: usize,
}

/// Peer statistics and health information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub peer_id: PeerId,
    pub address: SocketAddr,
    pub health: NodeHealth,
    pub last_seen: SystemTime,
    pub connection_time: SystemTime,
    pub latency: Duration,
    pub messages_sent: u64,
    pub messages_received: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
    pub failed_connections: u32,
    pub version: String,
    pub capabilities: HashSet<String>,
}

impl Default for PeerInfo {
    fn default() -> Self {
        Self {
            peer_id: PeerId::random(),
            address: "127.0.0.1:0".parse().unwrap(),
            health: NodeHealth::Healthy,
            last_seen: SystemTime::now(),
            connection_time: SystemTime::now(),
            latency: Duration::from_millis(0),
            messages_sent: 0,
            messages_received: 0,
            bytes_sent: 0,
            bytes_received: 0,
            failed_connections: 0,
            version: "1.0.0".to_string(),
            capabilities: HashSet::new(),
        }
    }
}

/// Network management configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkManagerConfig {
    pub health_check_interval: Duration,
    pub peer_timeout: Duration,
    pub max_failed_connections: u32,
    pub target_peer_count: usize,
    pub max_peer_count: usize,
    pub enable_auto_healing: bool,
    pub enable_topology_optimization: bool,
}

impl Default for NetworkManagerConfig {
    fn default() -> Self {
        Self {
            health_check_interval: Duration::from_secs(30),
            peer_timeout: Duration::from_secs(120),
            max_failed_connections: 3,
            target_peer_count: 8,
            max_peer_count: 50,
            enable_auto_healing: true,
            enable_topology_optimization: true,
        }
    }
}

/// Comprehensive network management system
pub struct NetworkManager {
    config: NetworkManagerConfig,
    peers: Arc<RwLock<HashMap<PeerId, PeerInfo>>>,
    bootstrap_nodes: Vec<SocketAddr>,
    blacklisted_peers: Arc<RwLock<HashSet<PeerId>>>,
    event_sender: mpsc::UnboundedSender<NetworkManagerEvent>,
    event_receiver: Arc<Mutex<mpsc::UnboundedReceiver<NetworkManagerEvent>>>,
}

/// Network manager events
#[derive(Debug, Clone)]
pub enum NetworkManagerEvent {
    PeerHealthChanged(PeerId, NodeHealth),
    NetworkTopologyChanged(NetworkTopology),
    PeerBlacklisted(PeerId, String),
    AutoHealingTriggered(String),
    TopologyOptimized(usize),
}

impl NetworkManager {
    /// Create a new network manager
    pub fn new(config: NetworkManagerConfig, bootstrap_nodes: Vec<SocketAddr>) -> Self {
        let (event_sender, event_receiver) = mpsc::unbounded_channel();

        Self {
            config,
            peers: Arc::new(RwLock::new(HashMap::new())),
            bootstrap_nodes,
            blacklisted_peers: Arc::new(RwLock::new(HashSet::new())),
            event_sender,
            event_receiver: Arc::new(Mutex::new(event_receiver)),
        }
    }

    /// Start the network manager
    pub async fn start(&self) -> Result<()> {
        // Initial connection to bootstrap nodes
        self.connect_to_bootstrap_if_needed().await?;

        let peers_clone = self.peers.clone();
        let blacklisted_clone = self.blacklisted_peers.clone();
        let config = self.config.clone();
        let event_sender = self.event_sender.clone();

        // Start health monitoring task
        tokio::spawn(async move {
            let mut interval = interval(config.health_check_interval);

            loop {
                interval.tick().await;

                if let Err(e) = Self::perform_health_check(
                    &peers_clone,
                    &blacklisted_clone,
                    &config,
                    &event_sender,
                )
                .await
                {
                    log::error!("Health check failed: {}", e);
                }
            }
        });

        // Start topology optimization task
        if self.config.enable_topology_optimization {
            let peers_clone = self.peers.clone();
            let config = self.config.clone();
            let event_sender = self.event_sender.clone();

            tokio::spawn(async move {
                let mut interval = interval(Duration::from_secs(300)); // Every 5 minutes

                loop {
                    interval.tick().await;

                    if let Err(e) =
                        Self::optimize_topology(&peers_clone, &config, &event_sender).await
                    {
                        log::error!("Topology optimization failed: {}", e);
                    }
                }
            });
        }

        Ok(())
    }

    /// Add or update peer information
    pub async fn update_peer(&self, peer_info: PeerInfo) -> Result<()> {
        let mut peers = self.peers.write().await;
        peers.insert(peer_info.peer_id.clone(), peer_info);
        Ok(())
    }

    /// Remove a peer
    pub async fn remove_peer(&self, peer_id: &PeerId) -> Result<()> {
        let mut peers = self.peers.write().await;
        peers.remove(peer_id);
        Ok(())
    }

    /// Get peer information
    pub async fn get_peer(&self, peer_id: &PeerId) -> Option<PeerInfo> {
        let peers = self.peers.read().await;
        peers.get(peer_id).cloned()
    }

    /// Get peer information by ID
    pub async fn get_peer_info(&self, peer_id: PeerId) -> Result<Option<PeerInfo>> {
        Ok(self.get_peer(&peer_id).await)
    }

    /// Get all healthy peers
    pub async fn get_healthy_peers(&self) -> Vec<PeerInfo> {
        let peers = self.peers.read().await;
        peers
            .values()
            .filter(|peer| peer.health == NodeHealth::Healthy)
            .cloned()
            .collect()
    }

    /// Blacklist a peer
    pub async fn blacklist_peer(&self, peer_id: PeerId, reason: String) -> Result<()> {
        let mut blacklisted = self.blacklisted_peers.write().await;
        blacklisted.insert(peer_id.clone());

        let _ = self
            .event_sender
            .send(NetworkManagerEvent::PeerBlacklisted(peer_id, reason));
        Ok(())
    }

    /// Remove peer from blacklist
    pub async fn unblacklist_peer(&self, peer_id: PeerId) -> Result<()> {
        let mut blacklisted = self.blacklisted_peers.write().await;
        blacklisted.remove(&peer_id);
        log::info!("Removed peer {} from blacklist", peer_id);
        Ok(())
    }

    /// Check if a peer is blacklisted
    pub async fn is_blacklisted(&self, peer_id: &PeerId) -> bool {
        let blacklisted = self.blacklisted_peers.read().await;
        blacklisted.contains(peer_id)
    }

    /// Get network topology information
    pub async fn get_network_topology(&self) -> NetworkTopology {
        let peers = self.peers.read().await;

        let total_nodes = peers.len();
        let connected_peers = peers
            .values()
            .filter(|p| p.health != NodeHealth::Disconnected)
            .count();
        let healthy_peers = peers
            .values()
            .filter(|p| p.health == NodeHealth::Healthy)
            .count();
        let degraded_peers = peers
            .values()
            .filter(|p| p.health == NodeHealth::Degraded)
            .count();
        let unhealthy_peers = peers
            .values()
            .filter(|p| p.health == NodeHealth::Unhealthy)
            .count();

        let average_latency = if connected_peers > 0 {
            let total_latency: Duration = peers
                .values()
                .filter(|p| p.health != NodeHealth::Disconnected)
                .map(|p| p.latency)
                .sum();
            total_latency / connected_peers as u32
        } else {
            Duration::from_millis(0)
        };

        NetworkTopology {
            total_nodes,
            connected_peers,
            healthy_peers,
            degraded_peers,
            unhealthy_peers,
            average_latency,
            network_diameter: Self::calculate_network_diameter(&peers),
        }
    }

    /// Perform health check on all peers
    async fn perform_health_check(
        peers: &Arc<RwLock<HashMap<PeerId, PeerInfo>>>,
        blacklisted: &Arc<RwLock<HashSet<PeerId>>>,
        config: &NetworkManagerConfig,
        event_sender: &mpsc::UnboundedSender<NetworkManagerEvent>,
    ) -> Result<()> {
        let mut peers_guard = peers.write().await;
        let now = SystemTime::now();

        for (peer_id, peer_info) in peers_guard.iter_mut() {
            if let Ok(duration) = now.duration_since(peer_info.last_seen) {
                let old_health = peer_info.health.clone();

                if duration > config.peer_timeout {
                    peer_info.health = NodeHealth::Disconnected;
                } else if duration > config.peer_timeout / 2 {
                    peer_info.health = NodeHealth::Degraded;
                } else if peer_info.failed_connections > config.max_failed_connections {
                    peer_info.health = NodeHealth::Unhealthy;
                } else {
                    peer_info.health = NodeHealth::Healthy;
                }

                // Notify if health changed
                if old_health != peer_info.health {
                    let _ = event_sender.send(NetworkManagerEvent::PeerHealthChanged(
                        peer_id.clone(),
                        peer_info.health.clone(),
                    ));
                }

                // Auto-blacklist persistently unhealthy peers
                if peer_info.health == NodeHealth::Unhealthy
                    && peer_info.failed_connections > config.max_failed_connections * 2
                {
                    let mut blacklisted_guard = blacklisted.write().await;
                    blacklisted_guard.insert(peer_id.clone());
                    let _ = event_sender.send(NetworkManagerEvent::PeerBlacklisted(
                        peer_id.clone(),
                        "Persistent connection failures".to_string(),
                    ));
                }
            }
        }

        Ok(())
    }

    /// Optimize network topology
    async fn optimize_topology(
        peers: &Arc<RwLock<HashMap<PeerId, PeerInfo>>>,
        config: &NetworkManagerConfig,
        event_sender: &mpsc::UnboundedSender<NetworkManagerEvent>,
    ) -> Result<()> {
        let peers_guard = peers.read().await;
        let healthy_count = peers_guard
            .values()
            .filter(|p| p.health == NodeHealth::Healthy)
            .count();

        if healthy_count < config.target_peer_count {
            let _ = event_sender.send(NetworkManagerEvent::AutoHealingTriggered(format!(
                "Low peer count: {} < {}",
                healthy_count, config.target_peer_count
            )));
        }

        let _ = event_sender.send(NetworkManagerEvent::TopologyOptimized(healthy_count));
        Ok(())
    }

    /// Calculate network diameter (simplified version)
    fn calculate_network_diameter(peers: &HashMap<PeerId, PeerInfo>) -> usize {
        // Simplified calculation - in a real implementation, this would use graph algorithms
        match peers.len() {
            0..=2 => 1,
            3..=8 => 2,
            9..=20 => 3,
            21..=50 => 4,
            _ => 5,
        }
    }

    /// Get network statistics
    pub async fn get_network_stats(&self) -> HashMap<String, u64> {
        let peers = self.peers.read().await;
        let blacklisted = self.blacklisted_peers.read().await;

        let total_messages_sent: u64 = peers.values().map(|p| p.messages_sent).sum();
        let total_messages_received: u64 = peers.values().map(|p| p.messages_received).sum();
        let total_bytes_sent: u64 = peers.values().map(|p| p.bytes_sent).sum();
        let total_bytes_received: u64 = peers.values().map(|p| p.bytes_received).sum();

        let mut stats = HashMap::new();
        stats.insert("total_peers".to_string(), peers.len() as u64);
        stats.insert("blacklisted_peers".to_string(), blacklisted.len() as u64);
        stats.insert("total_messages_sent".to_string(), total_messages_sent);
        stats.insert(
            "total_messages_received".to_string(),
            total_messages_received,
        );
        stats.insert("total_bytes_sent".to_string(), total_bytes_sent);
        stats.insert("total_bytes_received".to_string(), total_bytes_received);

        stats
    }

    /// Get event receiver for external monitoring
    pub fn get_event_receiver(&self) -> Arc<Mutex<mpsc::UnboundedReceiver<NetworkManagerEvent>>> {
        self.event_receiver.clone()
    }

    /// Get bootstrap nodes for initial connections
    pub fn get_bootstrap_nodes(&self) -> &Vec<SocketAddr> {
        &self.bootstrap_nodes
    }

    /// Connect to bootstrap nodes if peer count is below target
    pub async fn connect_to_bootstrap_if_needed(&self) -> crate::Result<()> {
        let peer_count = self.peers.read().await.len();

        if peer_count < self.config.target_peer_count {
            log::info!(
                "Peer count ({}) below target ({}), connecting to bootstrap nodes",
                peer_count,
                self.config.target_peer_count
            );

            for bootstrap_addr in &self.bootstrap_nodes {
                log::debug!(
                    "Attempting to connect to bootstrap node: {}",
                    bootstrap_addr
                );
                // In a real implementation, this would trigger actual connections
                // For now, we just log the attempt
            }
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_network_manager_creation() {
        let config = NetworkManagerConfig::default();
        let bootstrap_nodes = vec!["127.0.0.1:8000".parse().unwrap()];
        let manager = NetworkManager::new(config, bootstrap_nodes);

        assert_eq!(manager.peers.read().await.len(), 0);
    }

    #[tokio::test]
    async fn test_peer_management() {
        let config = NetworkManagerConfig::default();
        let manager = NetworkManager::new(config, vec![]);

        let peer_info = PeerInfo {
            peer_id: PeerId::random(),
            ..Default::default()
        };
        let peer_id = peer_info.peer_id.clone();

        manager.update_peer(peer_info.clone()).await.unwrap();

        let retrieved = manager.get_peer(&peer_id).await;
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().peer_id, peer_id);

        manager.remove_peer(&peer_id).await.unwrap();
        assert!(manager.get_peer(&peer_id).await.is_none());
    }

    #[tokio::test]
    async fn test_blacklist_functionality() {
        let config = NetworkManagerConfig::default();
        let manager = NetworkManager::new(config, vec![]);

        let peer_id = PeerId::random();
        assert!(!manager.is_blacklisted(&peer_id).await);

        manager
            .blacklist_peer(peer_id.clone(), "Test reason".to_string())
            .await
            .unwrap();
        assert!(manager.is_blacklisted(&peer_id).await);

        manager.unblacklist_peer(peer_id.clone()).await.unwrap();
        assert!(!manager.is_blacklisted(&peer_id).await);
    }

    #[tokio::test]
    async fn test_network_topology() {
        let config = NetworkManagerConfig::default();
        let manager = NetworkManager::new(config, vec![]);

        // Add some test peers
        for i in 0..5 {
            let peer_info = PeerInfo {
                peer_id: PeerId::random(),
                health: if i < 3 {
                    NodeHealth::Healthy
                } else {
                    NodeHealth::Degraded
                },
                ..Default::default()
            };
            manager.update_peer(peer_info).await.unwrap();
        }

        let topology = manager.get_network_topology().await;
        assert_eq!(topology.total_nodes, 5);
        assert_eq!(topology.healthy_peers, 3);
        assert_eq!(topology.degraded_peers, 2);
    }
}
