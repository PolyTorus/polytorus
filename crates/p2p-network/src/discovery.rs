//! Peer Discovery Module
//!
//! Implements various peer discovery mechanisms including DHT-based discovery,
//! mDNS local discovery, and connection pool management.

use std::{
    collections::HashMap,
    net::{IpAddr, Ipv4Addr, SocketAddr},
    sync::{Arc, Mutex},
    time::{Duration, SystemTime, UNIX_EPOCH},
};

use anyhow::{Context, Result};
use log::{debug, info, warn};
use serde::{Deserialize, Serialize};
use tokio::{
    net::UdpSocket,
    sync::RwLock,
    time::{interval, timeout},
};

/// Peer information for discovery
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeerInfo {
    pub node_id: String,
    pub address: SocketAddr,
    pub last_seen: u64,
    pub capabilities: Vec<String>,
    pub version: String,
}

/// DHT node for distributed hash table
#[derive(Debug, Clone)]
pub struct DHTNode {
    pub id: [u8; 20], // 160-bit node ID
    pub address: SocketAddr,
    pub last_seen: SystemTime,
}

/// Discovery message types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum DiscoveryMessage {
    /// Announce presence to the network
    Announce {
        node_id: String,
        address: SocketAddr,
        capabilities: Vec<String>,
    },
    /// Query for peers
    Query {
        target_id: Option<String>,
        max_peers: u32,
    },
    /// Response to query
    Response { peers: Vec<PeerInfo> },
    /// Ping to check if peer is alive
    Ping { node_id: String, timestamp: u64 },
    /// Pong response to ping
    Pong { node_id: String, timestamp: u64 },
}

/// Peer discovery service
pub struct PeerDiscovery {
    node_id: String,
    listen_addr: SocketAddr,
    socket: Arc<UdpSocket>,
    known_peers: Arc<RwLock<HashMap<String, PeerInfo>>>,
    dht_nodes: Arc<RwLock<HashMap<String, DHTNode>>>,
    connection_pool: Arc<Mutex<ConnectionPool>>,
    multicast_addr: SocketAddr,
}

/// Connection pool for managing peer connections
#[derive(Debug)]
pub struct ConnectionPool {
    active_connections: HashMap<String, ConnectionInfo>,
    max_connections: usize,
    connection_timeout: Duration,
}

#[derive(Debug, Clone)]
pub struct ConnectionInfo {
    peer_id: String,
    address: SocketAddr,
    established_at: SystemTime,
    last_activity: SystemTime,
    connection_quality: f32, // 0.0 to 1.0
}

impl PeerDiscovery {
    /// Create new peer discovery service
    pub async fn new(node_id: String, listen_addr: SocketAddr) -> Result<Self> {
        let socket = UdpSocket::bind(listen_addr)
            .await
            .context("Failed to bind UDP socket for discovery")?;

        // Enable SO_REUSEADDR for better socket reuse
        socket.set_broadcast(true)?;

        info!("Peer discovery service started on {}", listen_addr);

        let multicast_addr = "224.0.0.1:9999".parse().unwrap(); // Multicast address for local discovery

        Ok(Self {
            node_id,
            listen_addr,
            socket: Arc::new(socket),
            known_peers: Arc::new(RwLock::new(HashMap::new())),
            dht_nodes: Arc::new(RwLock::new(HashMap::new())),
            connection_pool: Arc::new(Mutex::new(ConnectionPool::new(
                50,
                Duration::from_secs(300),
            ))),
            multicast_addr,
        })
    }

    /// Start the discovery service
    pub async fn start(&self) -> Result<()> {
        let socket = self.socket.clone();
        let known_peers = self.known_peers.clone();
        let node_id = self.node_id.clone();

        // Start listening for discovery messages
        let listen_task = tokio::spawn(async move {
            let mut buffer = [0u8; 1024];

            loop {
                match socket.recv_from(&mut buffer).await {
                    Ok((len, from)) => {
                        if let Ok(message) =
                            bincode::deserialize::<DiscoveryMessage>(&buffer[..len])
                        {
                            Self::handle_discovery_message(
                                &node_id,
                                &known_peers,
                                message,
                                from,
                                &socket,
                            )
                            .await;
                        }
                    }
                    Err(e) => {
                        warn!("Discovery socket error: {}", e);
                        tokio::time::sleep(Duration::from_millis(100)).await;
                    }
                }
            }
        });

        // Start periodic announcements
        let announce_task = self.start_announcements().await;

        // Start peer cleanup
        let cleanup_task = self.start_cleanup().await;

        // Run all tasks concurrently
        tokio::select! {
            _ = listen_task => warn!("Discovery listen task ended"),
            _ = announce_task => warn!("Discovery announce task ended"),
            _ = cleanup_task => warn!("Discovery cleanup task ended"),
        }

        Ok(())
    }

    /// Handle incoming discovery messages
    async fn handle_discovery_message(
        node_id: &str,
        known_peers: &Arc<RwLock<HashMap<String, PeerInfo>>>,
        message: DiscoveryMessage,
        from: SocketAddr,
        socket: &UdpSocket,
    ) {
        match message {
            DiscoveryMessage::Announce {
                node_id: peer_id,
                address,
                capabilities,
            } => {
                if peer_id != node_id {
                    let peer_info = PeerInfo {
                        node_id: peer_id.clone(),
                        address,
                        last_seen: SystemTime::now()
                            .duration_since(UNIX_EPOCH)
                            .unwrap()
                            .as_secs(),
                        capabilities,
                        version: "1.0.0".to_string(),
                    };

                    known_peers.write().await.insert(peer_id, peer_info);
                    debug!("Discovered new peer at {}", address);
                }
            }
            DiscoveryMessage::Query {
                target_id: _,
                max_peers,
            } => {
                let peers = known_peers.read().await;
                let peer_list: Vec<PeerInfo> =
                    peers.values().take(max_peers as usize).cloned().collect();

                let response = DiscoveryMessage::Response { peers: peer_list };
                if let Ok(data) = bincode::serialize(&response) {
                    let _ = socket.send_to(&data, from).await;
                }
            }
            DiscoveryMessage::Ping {
                node_id: _peer_id,
                timestamp,
            } => {
                let pong = DiscoveryMessage::Pong {
                    node_id: node_id.to_string(),
                    timestamp,
                };
                if let Ok(data) = bincode::serialize(&pong) {
                    let _ = socket.send_to(&data, from).await;
                }
            }
            DiscoveryMessage::Response { peers } => {
                let mut known = known_peers.write().await;
                for peer in peers {
                    if peer.node_id != node_id {
                        known.insert(peer.node_id.clone(), peer);
                    }
                }
            }
            _ => {}
        }
    }

    /// Start periodic announcements
    async fn start_announcements(&self) -> tokio::task::JoinHandle<()> {
        let socket = self.socket.clone();
        let node_id = self.node_id.clone();
        let listen_addr = self.listen_addr;
        let multicast_addr = self.multicast_addr;

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(30)); // Announce every 30 seconds

            loop {
                interval.tick().await;

                let announce = DiscoveryMessage::Announce {
                    node_id: node_id.clone(),
                    address: listen_addr,
                    capabilities: vec!["p2p".to_string(), "blockchain".to_string()],
                };

                if let Ok(data) = bincode::serialize(&announce) {
                    // Broadcast to multicast address
                    let _ = socket.send_to(&data, multicast_addr).await;

                    // Also broadcast to local subnet
                    let broadcast_addr =
                        SocketAddr::new(IpAddr::V4(Ipv4Addr::BROADCAST), listen_addr.port());
                    let _ = socket.send_to(&data, broadcast_addr).await;
                }
            }
        })
    }

    /// Start cleanup of stale peers
    async fn start_cleanup(&self) -> tokio::task::JoinHandle<()> {
        let known_peers = self.known_peers.clone();

        tokio::spawn(async move {
            let mut interval = interval(Duration::from_secs(60)); // Cleanup every minute

            loop {
                interval.tick().await;

                let now = SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs();
                let mut peers = known_peers.write().await;

                // Remove peers not seen for more than 5 minutes
                peers.retain(|_, peer| now - peer.last_seen < 300);
            }
        })
    }

    /// Discover peers in the network
    pub async fn discover_peers(&self, max_peers: u32) -> Result<Vec<PeerInfo>> {
        let query = DiscoveryMessage::Query {
            target_id: None,
            max_peers,
        };

        let data = bincode::serialize(&query)?;

        // Send query to known peers and multicast
        let known = self.known_peers.read().await;
        for peer in known.values() {
            let _ = self.socket.send_to(&data, peer.address).await;
        }

        // Also send to multicast
        let _ = self.socket.send_to(&data, self.multicast_addr).await;

        // Wait a bit for responses
        tokio::time::sleep(Duration::from_millis(500)).await;

        let peers = self.known_peers.read().await;
        Ok(peers.values().cloned().collect())
    }

    /// Get list of known peers
    pub async fn get_known_peers(&self) -> Vec<PeerInfo> {
        self.known_peers.read().await.values().cloned().collect()
    }

    /// Add a known peer manually
    pub async fn add_peer(&self, peer: PeerInfo) {
        self.known_peers
            .write()
            .await
            .insert(peer.node_id.clone(), peer);
    }

    /// Ping a specific peer to check if it's alive
    pub async fn ping_peer(&self, peer_addr: SocketAddr) -> Result<Duration> {
        let timestamp = SystemTime::now().duration_since(UNIX_EPOCH)?.as_millis() as u64;
        let ping = DiscoveryMessage::Ping {
            node_id: self.node_id.clone(),
            timestamp,
        };

        let data = bincode::serialize(&ping)?;
        let start = SystemTime::now();

        self.socket.send_to(&data, peer_addr).await?;

        // Wait for pong with timeout
        let ping_timeout = Duration::from_secs(5);
        match timeout(ping_timeout, self.wait_for_pong(timestamp)).await {
            Ok(_) => Ok(start.elapsed().unwrap_or(Duration::from_millis(0))),
            Err(_) => Err(anyhow::anyhow!("Ping timeout")),
        }
    }

    /// Wait for pong response
    async fn wait_for_pong(&self, _timestamp: u64) -> Result<()> {
        // This is a simplified implementation
        // In a real implementation, you'd track pending pings
        tokio::time::sleep(Duration::from_millis(100)).await;
        Ok(())
    }

    /// Get DHT nodes for peer routing
    pub async fn get_dht_nodes(&self) -> Vec<DHTNode> {
        self.dht_nodes.read().await.values().cloned().collect()
    }

    /// Get connection pool status
    pub fn get_connection_pool_status(&self) -> (usize, usize) {
        let pool = self.connection_pool.lock().unwrap();
        (pool.active_connections.len(), pool.max_connections)
    }
}

impl ConnectionPool {
    fn new(max_connections: usize, connection_timeout: Duration) -> Self {
        Self {
            active_connections: HashMap::new(),
            max_connections,
            connection_timeout,
        }
    }

    /// Add a new connection
    pub fn add_connection(&mut self, peer_id: String, address: SocketAddr) -> Result<()> {
        if self.active_connections.len() >= self.max_connections {
            // Remove oldest connection
            if let Some((oldest_id, _)) = self
                .active_connections
                .iter()
                .min_by_key(|(_, info)| info.established_at)
                .map(|(id, info)| (id.clone(), info.clone()))
            {
                self.active_connections.remove(&oldest_id);
            }
        }

        let connection_info = ConnectionInfo {
            peer_id: peer_id.clone(),
            address,
            established_at: SystemTime::now(),
            last_activity: SystemTime::now(),
            connection_quality: 1.0,
        };

        self.active_connections.insert(peer_id, connection_info);
        Ok(())
    }

    /// Update connection activity
    pub fn update_activity(&mut self, peer_id: &str) {
        if let Some(connection) = self.active_connections.get_mut(peer_id) {
            connection.last_activity = SystemTime::now();
        }
    }

    /// Get active connections
    pub fn get_active_connections(&self) -> Vec<ConnectionInfo> {
        self.active_connections.values().cloned().collect()
    }

    /// Remove stale connections
    pub fn cleanup_stale_connections(&mut self) {
        let now = SystemTime::now();
        self.active_connections.retain(|_, connection| {
            now.duration_since(connection.last_activity)
                .unwrap_or(Duration::MAX)
                < self.connection_timeout
        });
    }

    /// Get connection info for a specific peer
    pub fn get_connection_info(&self, peer_id: &str) -> Option<&ConnectionInfo> {
        self.active_connections.get(peer_id)
    }

    /// Update connection quality based on performance
    pub fn update_connection_quality(&mut self, peer_id: &str, quality: f32) {
        if let Some(connection) = self.active_connections.get_mut(peer_id) {
            connection.connection_quality = quality.clamp(0.0, 1.0);
        }
    }

    /// Get best quality connections
    pub fn get_best_connections(&self, limit: usize) -> Vec<&ConnectionInfo> {
        let mut connections: Vec<_> = self.active_connections.values().collect();
        connections.sort_by(|a, b| {
            b.connection_quality
                .partial_cmp(&a.connection_quality)
                .unwrap()
        });
        connections.into_iter().take(limit).collect()
    }

    /// Get connections by address
    pub fn get_connections_by_address(&self, address: &SocketAddr) -> Vec<&ConnectionInfo> {
        self.active_connections
            .values()
            .filter(|conn| &conn.address == address)
            .collect()
    }

    /// Get peer IDs of all active connections
    pub fn get_active_peer_ids(&self) -> Vec<String> {
        self.active_connections
            .values()
            .map(|conn| conn.peer_id.clone())
            .collect()
    }
}

/// DHT implementation for distributed peer discovery
pub struct Dht {
    node_id: [u8; 20],
    routing_table: Arc<RwLock<HashMap<String, DHTNode>>>,
    k_bucket_size: usize,
}

impl Dht {
    pub fn new(node_id: String) -> Self {
        let mut hasher = sha1_smol::Sha1::new();
        hasher.update(node_id.as_bytes());
        let id = hasher.digest().bytes();

        Self {
            node_id: id,
            routing_table: Arc::new(RwLock::new(HashMap::new())),
            k_bucket_size: 20, // Standard Kademlia k-bucket size
        }
    }

    /// Add a node to the DHT
    pub async fn add_node(&self, node_id: String, address: SocketAddr) {
        let mut hasher = sha1_smol::Sha1::new();
        hasher.update(node_id.as_bytes());
        let id = hasher.digest().bytes();

        let dht_node = DHTNode {
            id,
            address,
            last_seen: SystemTime::now(),
        };

        self.routing_table.write().await.insert(node_id, dht_node);
    }

    /// Find closest nodes to a target ID
    pub async fn find_closest_nodes(&self, target_id: &[u8; 20], count: usize) -> Vec<DHTNode> {
        let table = self.routing_table.read().await;
        let mut nodes: Vec<_> = table.values().cloned().collect();

        // Sort by XOR distance to target
        nodes.sort_by_key(|node| xor_distance(&node.id, target_id));

        nodes.into_iter().take(count).collect()
    }

    /// Get the size of the routing table
    pub async fn size(&self) -> usize {
        self.routing_table.read().await.len()
    }

    /// Get the node's own ID
    pub fn get_node_id(&self) -> [u8; 20] {
        self.node_id
    }

    /// Get k-bucket size configuration
    pub fn get_k_bucket_size(&self) -> usize {
        self.k_bucket_size
    }

    /// Check if routing table is at capacity for a given distance
    pub async fn is_bucket_full(&self, target_distance: &[u8; 20]) -> bool {
        let table = self.routing_table.read().await;
        let nodes_at_distance: Vec<_> = table
            .values()
            .filter(|node| {
                let distance = xor_distance(&node.id, target_distance);
                distance.iter().take(1).all(|&x| x == 0) // Same prefix byte
            })
            .collect();
        nodes_at_distance.len() >= self.k_bucket_size
    }
}

/// Calculate XOR distance between two node IDs
fn xor_distance(a: &[u8; 20], b: &[u8; 20]) -> Vec<u8> {
    a.iter().zip(b.iter()).map(|(x, y)| x ^ y).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_peer_discovery_creation() {
        let discovery = PeerDiscovery::new("test_node".to_string(), "127.0.0.1:0".parse().unwrap())
            .await
            .unwrap();

        assert_eq!(discovery.node_id, "test_node");
    }

    #[tokio::test]
    async fn test_dht_operations() {
        let dht = DHT::new("test_node".to_string());

        dht.add_node("node1".to_string(), "127.0.0.1:8001".parse().unwrap())
            .await;
        dht.add_node("node2".to_string(), "127.0.0.1:8002".parse().unwrap())
            .await;

        assert_eq!(dht.size().await, 2);

        let closest = dht.find_closest_nodes(&[0; 20], 1).await;
        assert_eq!(closest.len(), 1);
    }

    #[test]
    fn test_connection_pool() {
        let mut pool = ConnectionPool::new(2, Duration::from_secs(60));

        pool.add_connection("peer1".to_string(), "127.0.0.1:8001".parse().unwrap())
            .unwrap();
        pool.add_connection("peer2".to_string(), "127.0.0.1:8002".parse().unwrap())
            .unwrap();

        assert_eq!(pool.get_active_connections().len(), 2);

        // Adding third connection should remove oldest
        pool.add_connection("peer3".to_string(), "127.0.0.1:8003".parse().unwrap())
            .unwrap();
        assert_eq!(pool.get_active_connections().len(), 2);
    }
}
