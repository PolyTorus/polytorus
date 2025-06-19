//! Advanced Peer Discovery and Network Bootstrap
//!
//! This module implements sophisticated peer discovery mechanisms
//! for the modular blockchain network, including DHT-like discovery,
//! bootstrap nodes, and network topology management.

use std::{
    collections::{HashMap, HashSet},
    net::SocketAddr,
    sync::{Arc, RwLock},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use tokio::{
    net::{TcpStream, UdpSocket},
    sync::mpsc,
    time::{interval, timeout},
};
use uuid::Uuid;

/// Node identifier in the network
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct NodeId(pub Uuid);

impl NodeId {
    pub fn random() -> Self {
        Self(Uuid::new_v4())
    }

    pub fn distance(&self, other: &NodeId) -> u64 {
        // Simple XOR distance for DHT-like routing
        let a = self.0.as_u128();
        let b = other.0.as_u128();
        (a ^ b) as u64
    }
}

impl std::fmt::Display for NodeId {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", &self.0.to_string()[..8])
    }
}

/// Network node information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkNode {
    pub node_id: NodeId,
    pub address: SocketAddr,
    pub last_seen: u64,
    pub capabilities: NodeCapabilities,
    pub reputation: f64,
    pub ping_ms: Option<u64>,
    pub version: String,
    pub chain_height: u64,
}

/// Node capabilities for specialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeCapabilities {
    pub full_node: bool,
    pub mining: bool,
    pub archive: bool,
    pub bootstrap: bool,
    pub services: Vec<String>,
}

impl Default for NodeCapabilities {
    fn default() -> Self {
        Self {
            full_node: true,
            mining: false,
            archive: false,
            bootstrap: false,
            services: Vec::new(),
        }
    }
}

/// Discovery message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiscoveryMessage {
    Ping {
        node_id: NodeId,
        timestamp: u64,
        capabilities: NodeCapabilities,
        chain_height: u64,
    },
    Pong {
        node_id: NodeId,
        timestamp: u64,
        capabilities: NodeCapabilities,
        chain_height: u64,
    },
    FindNode {
        target: NodeId,
        requester: NodeId,
    },
    NodesFound {
        target: NodeId,
        nodes: Vec<NetworkNode>,
        requester: NodeId,
    },
    Announce {
        node: NetworkNode,
    },
}

/// Bootstrap configuration for network startup
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BootstrapConfig {
    pub bootstrap_nodes: Vec<SocketAddr>,
    pub discovery_port: u16,
    pub max_peers: usize,
    pub ping_interval: Duration,
    pub discovery_interval: Duration,
    pub bootstrap_timeout: Duration,
    pub enable_mdns: bool,
    pub enable_upnp: bool,
}

impl Default for BootstrapConfig {
    fn default() -> Self {
        Self {
            bootstrap_nodes: vec![
                "127.0.0.1:8000".parse().unwrap(),
                "127.0.0.1:8001".parse().unwrap(),
            ],
            discovery_port: 8900,
            max_peers: 50,
            ping_interval: Duration::from_secs(30),
            discovery_interval: Duration::from_secs(60),
            bootstrap_timeout: Duration::from_secs(30),
            enable_mdns: true,
            enable_upnp: false,
        }
    }
}

/// Network topology management
#[derive(Debug, Clone)]
pub struct NetworkTopology {
    pub total_nodes: usize,
    pub connected_nodes: usize,
    pub bootstrap_nodes: usize,
    pub mining_nodes: usize,
    pub archive_nodes: usize,
    pub average_ping: f64,
    pub network_health: f64,
}

/// Events from peer discovery system
#[derive(Debug, Clone)]
pub enum DiscoveryEvent {
    NodeDiscovered(NetworkNode),
    NodeLost(NodeId),
    NodeUpdated(NetworkNode),
    NetworkTopologyUpdate(NetworkTopology),
    BootstrapComplete,
    BootstrapFailed(String),
}

/// Advanced peer discovery system
pub struct PeerDiscoveryService {
    node_id: NodeId,
    config: BootstrapConfig,
    known_nodes: Arc<RwLock<HashMap<NodeId, NetworkNode>>>,
    active_connections: Arc<RwLock<HashSet<NodeId>>>,
    routing_table: Arc<RwLock<Vec<Vec<NetworkNode>>>>, // Kademlia-style buckets
    event_tx: mpsc::UnboundedSender<DiscoveryEvent>,
    discovery_socket: Option<Arc<UdpSocket>>,
    capabilities: NodeCapabilities,
    chain_height: Arc<RwLock<u64>>,
}

impl PeerDiscoveryService {
    /// Create a new peer discovery service
    pub async fn new(
        config: BootstrapConfig,
        capabilities: NodeCapabilities,
    ) -> Result<(Self, mpsc::UnboundedReceiver<DiscoveryEvent>)> {
        let node_id = NodeId::random();
        let (event_tx, event_rx) = mpsc::unbounded_channel();

        // Create UDP socket for discovery
        let discovery_socket =
            UdpSocket::bind(format!("0.0.0.0:{}", config.discovery_port)).await?;
        discovery_socket.set_broadcast(true)?;

        let service = Self {
            node_id,
            config,
            known_nodes: Arc::new(RwLock::new(HashMap::new())),
            active_connections: Arc::new(RwLock::new(HashSet::new())),
            routing_table: Arc::new(RwLock::new(vec![Vec::new(); 256])), // 256 buckets
            event_tx,
            discovery_socket: Some(Arc::new(discovery_socket)),
            capabilities,
            chain_height: Arc::new(RwLock::new(0)),
        };

        Ok((service, event_rx))
    }

    /// Start the discovery service
    pub async fn start(&mut self) -> Result<()> {
        // Bootstrap from known nodes
        self.bootstrap().await?;

        // Start periodic discovery
        self.start_discovery_loop().await;

        // Start UDP discovery listener
        if let Some(socket) = &self.discovery_socket {
            self.start_udp_listener(Arc::clone(socket)).await;
        }

        // Start mDNS discovery if enabled
        if self.config.enable_mdns {
            self.start_mdns_discovery().await;
        }

        Ok(())
    }

    /// Bootstrap from configured bootstrap nodes
    async fn bootstrap(&self) -> Result<()> {
        log::info!(
            "Starting bootstrap process with {} nodes",
            self.config.bootstrap_nodes.len()
        );

        let mut successful_connections = 0;

        for bootstrap_addr in &self.config.bootstrap_nodes {
            match timeout(
                self.config.bootstrap_timeout,
                self.connect_bootstrap_node(*bootstrap_addr),
            )
            .await
            {
                Ok(Ok(_)) => {
                    successful_connections += 1;
                    log::info!(
                        "Successfully connected to bootstrap node: {}",
                        bootstrap_addr
                    );
                }
                Ok(Err(e)) => {
                    log::warn!(
                        "Failed to connect to bootstrap node {}: {}",
                        bootstrap_addr,
                        e
                    );
                }
                Err(_) => {
                    log::warn!("Timeout connecting to bootstrap node: {}", bootstrap_addr);
                }
            }
        }

        if successful_connections > 0 {
            let _ = self.event_tx.send(DiscoveryEvent::BootstrapComplete);
            log::info!(
                "Bootstrap completed with {} successful connections",
                successful_connections
            );
            Ok(())
        } else {
            let error_msg = "Bootstrap failed - no connections established".to_string();
            let _ = self
                .event_tx
                .send(DiscoveryEvent::BootstrapFailed(error_msg.clone()));
            Err(anyhow!(error_msg))
        }
    }

    /// Connect to a bootstrap node
    async fn connect_bootstrap_node(&self, addr: SocketAddr) -> Result<()> {
        // Try to establish TCP connection for handshake
        let stream = TcpStream::connect(addr).await?;

        // Send discovery ping via UDP
        if let Some(socket) = &self.discovery_socket {
            let ping_msg = DiscoveryMessage::Ping {
                node_id: self.node_id,
                timestamp: SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs(),
                capabilities: self.capabilities.clone(),
                chain_height: *self.chain_height.read().unwrap(),
            };

            let serialized = bincode::serialize(&ping_msg)?;
            socket.send_to(&serialized, addr).await?;
        }

        drop(stream);
        Ok(())
    }

    /// Start the discovery loop
    async fn start_discovery_loop(&self) {
        let known_nodes = Arc::clone(&self.known_nodes);
        let discovery_socket = self.discovery_socket.as_ref().map(Arc::clone);
        let event_tx = self.event_tx.clone();
        let node_id = self.node_id;
        let capabilities = self.capabilities.clone();
        let chain_height = Arc::clone(&self.chain_height);
        let discovery_interval = self.config.discovery_interval;

        tokio::spawn(async move {
            let mut interval = interval(discovery_interval);

            loop {
                interval.tick().await;

                // Ping known nodes
                let nodes: Vec<_> = {
                    let nodes_map = known_nodes.read().unwrap();
                    nodes_map.values().cloned().collect()
                };

                if let Some(socket) = &discovery_socket {
                    for node in &nodes {
                        let ping_msg = DiscoveryMessage::Ping {
                            node_id,
                            timestamp: SystemTime::now()
                                .duration_since(UNIX_EPOCH)
                                .unwrap()
                                .as_secs(),
                            capabilities: capabilities.clone(),
                            chain_height: *chain_height.read().unwrap(),
                        };

                        if let Ok(serialized) = bincode::serialize(&ping_msg) {
                            let _ = socket.send_to(&serialized, node.address).await;
                        }
                    }
                }

                // Update network topology
                let topology = Self::calculate_topology(&nodes);
                let _ = event_tx.send(DiscoveryEvent::NetworkTopologyUpdate(topology));
            }
        });
    }

    /// Start UDP listener for discovery messages
    async fn start_udp_listener(&self, socket: Arc<UdpSocket>) {
        let known_nodes = Arc::clone(&self.known_nodes);
        let event_tx = self.event_tx.clone();
        let node_id = self.node_id;
        let capabilities = self.capabilities.clone();
        let chain_height = Arc::clone(&self.chain_height);

        tokio::spawn(async move {
            let mut buf = [0u8; 1024];

            loop {
                match socket.recv_from(&mut buf).await {
                    Ok((len, addr)) => {
                        if let Ok(msg) = bincode::deserialize::<DiscoveryMessage>(&buf[..len]) {
                            Self::handle_discovery_message(
                                msg,
                                addr,
                                &known_nodes,
                                &event_tx,
                                &socket,
                                node_id,
                                &capabilities,
                                &chain_height,
                            )
                            .await;
                        }
                    }
                    Err(e) => {
                        log::error!("UDP receive error: {}", e);
                    }
                }
            }
        });
    }

    /// Handle incoming discovery messages
    async fn handle_discovery_message(
        msg: DiscoveryMessage,
        sender_addr: SocketAddr,
        known_nodes: &Arc<RwLock<HashMap<NodeId, NetworkNode>>>,
        event_tx: &mpsc::UnboundedSender<DiscoveryEvent>,
        socket: &Arc<UdpSocket>,
        our_node_id: NodeId,
        our_capabilities: &NodeCapabilities,
        chain_height: &Arc<RwLock<u64>>,
    ) {
        match msg {
            DiscoveryMessage::Ping {
                node_id,
                timestamp,
                capabilities,
                chain_height: peer_height,
            } => {
                // Create or update node entry
                let node = NetworkNode {
                    node_id,
                    address: sender_addr,
                    last_seen: timestamp,
                    capabilities,
                    reputation: 1.0,
                    ping_ms: None,
                    version: "1.0.0".to_string(),
                    chain_height: peer_height,
                };

                let is_new = {
                    let mut nodes = known_nodes.write().unwrap();
                    let is_new = !nodes.contains_key(&node_id);
                    nodes.insert(node_id, node.clone());
                    is_new
                };

                if is_new {
                    let _ = event_tx.send(DiscoveryEvent::NodeDiscovered(node));
                } else {
                    let _ = event_tx.send(DiscoveryEvent::NodeUpdated(node));
                }

                // Send pong response
                let pong_msg = DiscoveryMessage::Pong {
                    node_id: our_node_id,
                    timestamp: SystemTime::now()
                        .duration_since(UNIX_EPOCH)
                        .unwrap()
                        .as_secs(),
                    capabilities: our_capabilities.clone(),
                    chain_height: *chain_height.read().unwrap(),
                };

                if let Ok(serialized) = bincode::serialize(&pong_msg) {
                    let _ = socket.send_to(&serialized, sender_addr).await;
                }
            }
            DiscoveryMessage::Pong {
                node_id,
                timestamp,
                capabilities,
                chain_height: peer_height,
            } => {
                // Update node entry with pong response
                let node = NetworkNode {
                    node_id,
                    address: sender_addr,
                    last_seen: timestamp,
                    capabilities,
                    reputation: 1.0,
                    ping_ms: Some(50), // Simplified ping calculation
                    version: "1.0.0".to_string(),
                    chain_height: peer_height,
                };

                {
                    let mut nodes = known_nodes.write().unwrap();
                    nodes.insert(node_id, node.clone());
                }

                let _ = event_tx.send(DiscoveryEvent::NodeUpdated(node));
            }
            DiscoveryMessage::FindNode { target, requester } => {
                // Find closest nodes to target
                let closest_nodes: Vec<_> = {
                    let nodes = known_nodes.read().unwrap();
                    let mut node_distances: Vec<_> = nodes
                        .values()
                        .map(|node| (node.clone(), node.node_id.distance(&target)))
                        .collect();

                    node_distances.sort_by_key(|(_, distance)| *distance);
                    node_distances
                        .into_iter()
                        .take(8) // Return up to 8 closest nodes
                        .map(|(node, _)| node)
                        .collect()
                };

                let response = DiscoveryMessage::NodesFound {
                    target,
                    nodes: closest_nodes,
                    requester,
                };

                if let Ok(serialized) = bincode::serialize(&response) {
                    let _ = socket.send_to(&serialized, sender_addr).await;
                }
            }
            DiscoveryMessage::NodesFound { nodes, .. } => {
                // Add discovered nodes to our routing table
                let mut new_nodes = Vec::new();
                {
                    let mut known_nodes = known_nodes.write().unwrap();
                    for node in nodes {
                        if let std::collections::hash_map::Entry::Vacant(e) =
                            known_nodes.entry(node.node_id)
                        {
                            e.insert(node.clone());
                            new_nodes.push(node);
                        }
                    }
                }

                for node in new_nodes {
                    let _ = event_tx.send(DiscoveryEvent::NodeDiscovered(node));
                }
            }
            DiscoveryMessage::Announce { node } => {
                let is_new = {
                    let mut nodes = known_nodes.write().unwrap();
                    let is_new = !nodes.contains_key(&node.node_id);
                    nodes.insert(node.node_id, node.clone());
                    is_new
                };

                if is_new {
                    let _ = event_tx.send(DiscoveryEvent::NodeDiscovered(node));
                } else {
                    let _ = event_tx.send(DiscoveryEvent::NodeUpdated(node));
                }
            }
        }
    }

    /// Start mDNS discovery for local network
    async fn start_mdns_discovery(&self) {
        log::info!("Starting mDNS discovery for local network");
        // Simplified mDNS implementation would go here
        // For now, we'll implement basic broadcast discovery on local network

        let socket = match UdpSocket::bind("0.0.0.0:0").await {
            Ok(socket) => socket,
            Err(e) => {
                log::error!("Failed to create mDNS socket: {}", e);
                return;
            }
        };

        let broadcast_addr: SocketAddr = "255.255.255.255:8900".parse().unwrap();
        let node_id = self.node_id;
        let capabilities = self.capabilities.clone();
        let chain_height = Arc::clone(&self.chain_height);

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(60));

            loop {
                interval.tick().await;

                let announce_msg = DiscoveryMessage::Announce {
                    node: NetworkNode {
                        node_id,
                        address: "0.0.0.0:0".parse().unwrap(), // Will be replaced by receiver
                        last_seen: SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_secs(),
                        capabilities: capabilities.clone(),
                        reputation: 1.0,
                        ping_ms: None,
                        version: "1.0.0".to_string(),
                        chain_height: *chain_height.read().unwrap(),
                    },
                };

                if let Ok(serialized) = bincode::serialize(&announce_msg) {
                    let _ = socket.send_to(&serialized, broadcast_addr).await;
                }
            }
        });
    }

    /// Calculate network topology metrics
    fn calculate_topology(nodes: &[NetworkNode]) -> NetworkTopology {
        let total_nodes = nodes.len();
        let connected_nodes = nodes.iter().filter(|n| n.ping_ms.is_some()).count();
        let bootstrap_nodes = nodes.iter().filter(|n| n.capabilities.bootstrap).count();
        let mining_nodes = nodes.iter().filter(|n| n.capabilities.mining).count();
        let archive_nodes = nodes.iter().filter(|n| n.capabilities.archive).count();

        let average_ping = if connected_nodes > 0 {
            nodes.iter().filter_map(|n| n.ping_ms).sum::<u64>() as f64 / connected_nodes as f64
        } else {
            0.0
        };

        let network_health = if total_nodes > 0 {
            connected_nodes as f64 / total_nodes as f64
        } else {
            0.0
        };

        NetworkTopology {
            total_nodes,
            connected_nodes,
            bootstrap_nodes,
            mining_nodes,
            archive_nodes,
            average_ping,
            network_health,
        }
    }

    /// Get all known nodes
    pub fn get_known_nodes(&self) -> Vec<NetworkNode> {
        self.known_nodes.read().unwrap().values().cloned().collect()
    }

    /// Get nodes with specific capabilities
    pub fn get_nodes_with_capability(&self, capability: &str) -> Vec<NetworkNode> {
        self.known_nodes
            .read()
            .unwrap()
            .values()
            .filter(|node| match capability {
                "mining" => node.capabilities.mining,
                "archive" => node.capabilities.archive,
                "bootstrap" => node.capabilities.bootstrap,
                _ => node.capabilities.services.contains(&capability.to_string()),
            })
            .cloned()
            .collect()
    }

    /// Update our chain height
    pub fn update_chain_height(&self, height: u64) {
        *self.chain_height.write().unwrap() = height;
    }

    /// Find nodes close to a target ID (for DHT-like routing)
    pub fn find_closest_nodes(&self, target: NodeId, count: usize) -> Vec<NetworkNode> {
        let nodes = self.known_nodes.read().unwrap();
        let mut node_distances: Vec<_> = nodes
            .values()
            .map(|node| (node.clone(), node.node_id.distance(&target)))
            .collect();

        node_distances.sort_by_key(|(_, distance)| *distance);
        node_distances
            .into_iter()
            .take(count)
            .map(|(node, _)| node)
            .collect()
    }

    /// Get our node ID
    pub fn node_id(&self) -> NodeId {
        self.node_id
    }

    /// Get active connections
    pub fn get_active_connections(&self) -> Vec<NodeId> {
        self.active_connections
            .read()
            .unwrap()
            .iter()
            .copied()
            .collect()
    }

    /// Get routing table bucket count
    pub fn get_routing_table_size(&self) -> usize {
        self.routing_table.read().unwrap().len()
    }
}

#[cfg(test)]
mod tests {

    use super::*;

    #[tokio::test]
    async fn test_node_id_distance() {
        let id1 = NodeId::random();
        let id2 = NodeId::random();

        let distance1 = id1.distance(&id2);
        let distance2 = id2.distance(&id1);

        assert_eq!(distance1, distance2);
        assert_eq!(id1.distance(&id1), 0);
    }

    #[tokio::test]
    async fn test_peer_discovery_creation() {
        let config = BootstrapConfig::default();
        let capabilities = NodeCapabilities::default();

        let result = PeerDiscoveryService::new(config, capabilities).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_bootstrap_process() {
        let config = BootstrapConfig {
            bootstrap_nodes: vec!["127.0.0.1:9999".parse().unwrap()], // Non-existent node
            bootstrap_timeout: Duration::from_millis(100),
            ..Default::default()
        };

        let capabilities = NodeCapabilities::default();
        let (service, mut event_rx) = PeerDiscoveryService::new(config, capabilities)
            .await
            .unwrap();

        // Bootstrap will fail but shouldn't panic
        let result = service.bootstrap().await;
        assert!(result.is_err());

        // Should receive bootstrap failed event
        if let Some(event) = event_rx.recv().await {
            match event {
                DiscoveryEvent::BootstrapFailed(_) => {}
                _ => panic!("Expected bootstrap failed event"),
            }
        }
    }

    #[tokio::test]
    async fn test_network_topology_calculation() {
        let nodes = vec![
            NetworkNode {
                node_id: NodeId::random(),
                address: "127.0.0.1:8000".parse().unwrap(),
                last_seen: 0,
                capabilities: NodeCapabilities {
                    mining: true,
                    ..Default::default()
                },
                reputation: 1.0,
                ping_ms: Some(50),
                version: "1.0.0".to_string(),
                chain_height: 100,
            },
            NetworkNode {
                node_id: NodeId::random(),
                address: "127.0.0.1:8001".parse().unwrap(),
                last_seen: 0,
                capabilities: NodeCapabilities {
                    bootstrap: true,
                    ..Default::default()
                },
                reputation: 1.0,
                ping_ms: None,
                version: "1.0.0".to_string(),
                chain_height: 95,
            },
        ];

        let topology = PeerDiscoveryService::calculate_topology(&nodes);

        assert_eq!(topology.total_nodes, 2);
        assert_eq!(topology.connected_nodes, 1);
        assert_eq!(topology.mining_nodes, 1);
        assert_eq!(topology.bootstrap_nodes, 1);
        assert_eq!(topology.average_ping, 50.0);
        assert_eq!(topology.network_health, 0.5);
    }
}
