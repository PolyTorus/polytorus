//! Generic network configuration for P2P communication
//!
//! This module provides configuration settings for P2P networking,
//! including node discovery, connection management, and protocol settings.

use serde::{Deserialize, Serialize};

/// Network configuration for P2P nodes
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NetworkConfig {
    /// Listen address (IP and port)
    pub listen_address: String,
    /// Bootstrap nodes for initial connections
    pub bootstrap_nodes: Vec<String>,
    /// Network identity and security
    pub identity: IdentityConfig,
    /// Peer discovery settings
    pub discovery: DiscoveryConfig,
    /// Connection management
    pub connection: ConnectionConfig,
}

/// Node identity and security configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct IdentityConfig {
    /// Node keypair seed (optional, generated if not provided)
    pub keypair_seed: Option<String>,
    /// Network protocol version
    pub protocol_version: String,
    /// User agent string
    pub user_agent: String,
}

/// Peer discovery configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiscoveryConfig {
    /// Enable DHT for peer discovery
    pub enable_dht: bool,
    /// Enable mDNS for local network discovery
    pub enable_mdns: bool,
    /// Bootstrap timeout in seconds
    pub bootstrap_timeout: u64,
    /// Periodic peer discovery interval in seconds
    pub discovery_interval: u64,
}

/// Connection management configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConnectionConfig {
    /// Maximum number of inbound connections
    pub max_inbound: usize,
    /// Maximum number of outbound connections
    pub max_outbound: usize,
    /// Connection timeout in seconds
    pub connection_timeout: u64,
    /// Keep-alive interval in seconds
    pub keep_alive_interval: u64,
    /// Idle connection timeout in seconds
    pub idle_timeout: u64,
}

impl Default for NetworkConfig {
    fn default() -> Self {
        Self {
            listen_address: "0.0.0.0:9090".to_string(),
            bootstrap_nodes: vec![],
            identity: IdentityConfig::default(),
            discovery: DiscoveryConfig::default(),
            connection: ConnectionConfig::default(),
        }
    }
}

impl Default for IdentityConfig {
    fn default() -> Self {
        Self {
            keypair_seed: None,
            protocol_version: "/polytorus/1.0.0".to_string(),
            user_agent: format!("polytorus/{}", env!("CARGO_PKG_VERSION")),
        }
    }
}

impl Default for DiscoveryConfig {
    fn default() -> Self {
        Self {
            enable_dht: true,
            enable_mdns: true,
            bootstrap_timeout: 30,
            discovery_interval: 300, // 5 minutes
        }
    }
}

impl Default for ConnectionConfig {
    fn default() -> Self {
        Self {
            max_inbound: 25,
            max_outbound: 25,
            connection_timeout: 10,
            keep_alive_interval: 30,
            idle_timeout: 300, // 5 minutes
        }
    }
}

impl NetworkConfig {
    /// Load configuration from environment variables and config file
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error>> {
        let mut config = Self::default();

        // Listen address
        if let Ok(listen_addr) = std::env::var("POLYTORUS_LISTEN_ADDRESS") {
            config.listen_address = listen_addr;
        }

        // Bootstrap nodes
        if let Ok(bootstrap) = std::env::var("POLYTORUS_BOOTSTRAP_NODES") {
            config.bootstrap_nodes = bootstrap
                .split(',')
                .map(|s| s.trim().to_string())
                .filter(|s| !s.is_empty())
                .collect();
        }

        // Identity configuration
        if let Ok(seed) = std::env::var("POLYTORUS_KEYPAIR_SEED") {
            config.identity.keypair_seed = Some(seed);
        }

        if let Ok(protocol) = std::env::var("POLYTORUS_PROTOCOL_VERSION") {
            config.identity.protocol_version = protocol;
        }

        if let Ok(user_agent) = std::env::var("POLYTORUS_USER_AGENT") {
            config.identity.user_agent = user_agent;
        }

        // Discovery configuration
        if let Ok(enable_dht) = std::env::var("POLYTORUS_ENABLE_DHT") {
            config.discovery.enable_dht = enable_dht.parse().unwrap_or(true);
        }

        if let Ok(enable_mdns) = std::env::var("POLYTORUS_ENABLE_MDNS") {
            config.discovery.enable_mdns = enable_mdns.parse().unwrap_or(true);
        }

        if let Ok(bootstrap_timeout) = std::env::var("POLYTORUS_BOOTSTRAP_TIMEOUT") {
            config.discovery.bootstrap_timeout = bootstrap_timeout.parse()?;
        }

        if let Ok(discovery_interval) = std::env::var("POLYTORUS_DISCOVERY_INTERVAL") {
            config.discovery.discovery_interval = discovery_interval.parse()?;
        }

        // Connection configuration
        if let Ok(max_inbound) = std::env::var("POLYTORUS_MAX_INBOUND") {
            config.connection.max_inbound = max_inbound.parse()?;
        }

        if let Ok(max_outbound) = std::env::var("POLYTORUS_MAX_OUTBOUND") {
            config.connection.max_outbound = max_outbound.parse()?;
        }

        if let Ok(conn_timeout) = std::env::var("POLYTORUS_CONNECTION_TIMEOUT") {
            config.connection.connection_timeout = conn_timeout.parse()?;
        }

        if let Ok(keep_alive) = std::env::var("POLYTORUS_KEEP_ALIVE_INTERVAL") {
            config.connection.keep_alive_interval = keep_alive.parse()?;
        }

        if let Ok(idle_timeout) = std::env::var("POLYTORUS_IDLE_TIMEOUT") {
            config.connection.idle_timeout = idle_timeout.parse()?;
        }

        Ok(config)
    }

    /// Load configuration from JSON file
    pub fn from_json_file(path: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let content = std::fs::read_to_string(path)?;
        let config: NetworkConfig = serde_json::from_str(&content)?;
        Ok(config)
    }

    /// Save configuration to JSON file
    pub fn to_json_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let content = serde_json::to_string_pretty(self)?;
        std::fs::write(path, content)?;
        Ok(())
    }

    /// Get listen address as SocketAddr
    pub fn get_listen_address(&self) -> &str {
        &self.listen_address
    }

    /// Get bootstrap addresses
    pub fn get_bootstrap_addresses(&self) -> &[String] {
        &self.bootstrap_nodes
    }

    /// Add bootstrap node
    pub fn add_bootstrap_node(&mut self, address: String) {
        if !self.bootstrap_nodes.contains(&address) {
            self.bootstrap_nodes.push(address);
        }
    }

    /// Remove bootstrap node
    pub fn remove_bootstrap_node(&mut self, address: &str) {
        self.bootstrap_nodes.retain(|node| node != address);
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<(), String> {
        // Validate listen address
        if self.listen_address.parse::<std::net::SocketAddr>().is_err() {
            return Err(format!("Invalid listen address: {}", self.listen_address));
        }

        // Validate bootstrap nodes
        for node in &self.bootstrap_nodes {
            if node.parse::<std::net::SocketAddr>().is_err() {
                return Err(format!("Invalid bootstrap node address: {}", node));
            }
        }

        // Validate connection limits
        if self.connection.max_inbound == 0 && self.connection.max_outbound == 0 {
            return Err("At least one of max_inbound or max_outbound must be greater than 0".to_string());
        }

        // Validate timeouts
        if self.connection.connection_timeout == 0 {
            return Err("Connection timeout must be greater than 0".to_string());
        }

        if self.discovery.bootstrap_timeout == 0 {
            return Err("Bootstrap timeout must be greater than 0".to_string());
        }

        Ok(())
    }

    /// Get total maximum connections
    pub fn max_connections(&self) -> usize {
        self.connection.max_inbound + self.connection.max_outbound
    }

    /// Check if local discovery is enabled
    pub fn is_local_discovery_enabled(&self) -> bool {
        self.discovery.enable_mdns
    }

    /// Check if DHT discovery is enabled
    pub fn is_dht_enabled(&self) -> bool {
        self.discovery.enable_dht
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = NetworkConfig::default();
        assert_eq!(config.listen_address, "0.0.0.0:9090");
        assert!(config.bootstrap_nodes.is_empty());
        assert!(config.discovery.enable_dht);
        assert!(config.discovery.enable_mdns);
    }

    #[test]
    fn test_config_validation() {
        let mut config = NetworkConfig::default();
        assert!(config.validate().is_ok());

        config.listen_address = "invalid".to_string();
        assert!(config.validate().is_err());

        config.listen_address = "127.0.0.1:9090".to_string();
        config.bootstrap_nodes.push("invalid".to_string());
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_bootstrap_node_management() {
        let mut config = NetworkConfig::default();
        
        config.add_bootstrap_node("127.0.0.1:9091".to_string());
        assert_eq!(config.bootstrap_nodes.len(), 1);
        
        // Adding the same node again should not duplicate
        config.add_bootstrap_node("127.0.0.1:9091".to_string());
        assert_eq!(config.bootstrap_nodes.len(), 1);
        
        config.remove_bootstrap_node("127.0.0.1:9091");
        assert_eq!(config.bootstrap_nodes.len(), 0);
    }
}
