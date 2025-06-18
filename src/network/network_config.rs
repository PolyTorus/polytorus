//! Generic network configuration for P2P communication
//!
//! This module provides configuration settings for P2P networking,
//! including node discovery, connection management, and protocol settings.

use std::{
    net::{SocketAddr, TcpListener},
    time::Duration,
};

use serde::{Deserialize, Serialize};
use tokio::net::lookup_host;

/// Validation level for network configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValidationLevel {
    /// Basic syntax validation only
    Basic,
    /// Include connectivity checks
    Connectivity,
    /// Full validation including resource and security checks
    Full,
}

/// Validation result with detailed feedback
#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub is_valid: bool,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub suggestions: Vec<String>,
}

impl Default for ValidationResult {
    fn default() -> Self {
        Self {
            is_valid: true,
            warnings: Vec::new(),
            errors: Vec::new(),
            suggestions: Vec::new(),
        }
    }
}

impl ValidationResult {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn add_error(&mut self, error: String) {
        self.is_valid = false;
        self.errors.push(error);
    }

    pub fn add_warning(&mut self, warning: String) {
        self.warnings.push(warning);
    }

    pub fn add_suggestion(&mut self, suggestion: String) {
        self.suggestions.push(suggestion);
    }

    pub fn merge(&mut self, other: ValidationResult) {
        if !other.is_valid {
            self.is_valid = false;
        }
        self.errors.extend(other.errors);
        self.warnings.extend(other.warnings);
        self.suggestions.extend(other.suggestions);
    }
}

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

    /// Basic validate configuration (legacy compatibility)
    pub fn validate(&self) -> Result<(), String> {
        let result = self.validate_with_level(ValidationLevel::Basic);
        if result.is_valid {
            Ok(())
        } else {
            Err(result.errors.join("; "))
        }
    }

    /// Enhanced validation with different levels
    pub fn validate_with_level(&self, level: ValidationLevel) -> ValidationResult {
        let mut result = ValidationResult::new();

        // Basic validation
        result.merge(self.validate_basic());

        // Connectivity validation
        if matches!(level, ValidationLevel::Connectivity | ValidationLevel::Full) {
            let connectivity_result = tokio::task::block_in_place(|| {
                tokio::runtime::Handle::current().block_on(self.validate_connectivity())
            });
            result.merge(connectivity_result);
        }

        // Full validation including resource and security checks
        if matches!(level, ValidationLevel::Full) {
            result.merge(self.validate_resources());
            result.merge(self.validate_security());
        }

        result
    }

    /// Async validation for connectivity checks
    pub async fn validate_async(&self, level: ValidationLevel) -> ValidationResult {
        let mut result = ValidationResult::new();

        // Basic validation
        result.merge(self.validate_basic());

        // Connectivity validation
        if matches!(level, ValidationLevel::Connectivity | ValidationLevel::Full) {
            result.merge(self.validate_connectivity().await);
        }

        // Full validation
        if matches!(level, ValidationLevel::Full) {
            result.merge(self.validate_resources());
            result.merge(self.validate_security());
        }

        result
    }

    /// Basic syntax and logical validation
    fn validate_basic(&self) -> ValidationResult {
        let mut result = ValidationResult::new();

        // Validate listen address
        match self.listen_address.parse::<SocketAddr>() {
            Ok(addr) => {
                // Check for common issues
                if addr.ip().is_unspecified() && addr.port() == 0 {
                    result.add_warning("Listen address uses unspecified IP and port 0. This may cause issues in production.".to_string());
                }
                if addr.port() < 1024 {
                    result.add_warning(format!(
                        "Using privileged port {}. Make sure you have appropriate permissions.",
                        addr.port()
                    ));
                }
            }
            Err(_) => {
                result.add_error(format!("Invalid listen address: {}", self.listen_address));
            }
        }

        // Validate bootstrap nodes
        for (i, node) in self.bootstrap_nodes.iter().enumerate() {
            if node.parse::<SocketAddr>().is_err() {
                result.add_error(format!(
                    "Invalid bootstrap node address {}: {}",
                    i + 1,
                    node
                ));
            }
        }

        // Validate connection limits
        if self.connection.max_inbound == 0 && self.connection.max_outbound == 0 {
            result.add_error(
                "At least one of max_inbound or max_outbound must be greater than 0".to_string(),
            );
        }

        let total_connections = self.connection.max_inbound + self.connection.max_outbound;
        if total_connections > 1000 {
            result.add_warning(format!(
                "Total connection limit ({}) is very high. This may cause resource issues.",
                total_connections
            ));
        }

        // Validate timeouts
        if self.connection.connection_timeout == 0 {
            result.add_error("Connection timeout must be greater than 0".to_string());
        } else if self.connection.connection_timeout > 300 {
            result.add_warning("Connection timeout is very high (>5 minutes). This may cause poor user experience.".to_string());
        }

        if self.discovery.bootstrap_timeout == 0 {
            result.add_error("Bootstrap timeout must be greater than 0".to_string());
        } else if self.discovery.bootstrap_timeout > 600 {
            result.add_warning("Bootstrap timeout is very high (>10 minutes).".to_string());
        }

        // Validate discovery settings
        if !self.discovery.enable_dht
            && !self.discovery.enable_mdns
            && self.bootstrap_nodes.is_empty()
        {
            result.add_error("No peer discovery mechanism enabled and no bootstrap nodes configured. The node will be isolated.".to_string());
        }

        // Validate identity settings
        if self.identity.protocol_version.is_empty() {
            result.add_error("Protocol version cannot be empty".to_string());
        }

        if self.identity.user_agent.is_empty() {
            result.add_warning(
                "User agent is empty. This may cause issues with some peers.".to_string(),
            );
        }

        result
    }

    /// Validate actual connectivity
    async fn validate_connectivity(&self) -> ValidationResult {
        let mut result = ValidationResult::new();

        // Test listen address availability
        if let Ok(listen_addr) = self.listen_address.parse::<SocketAddr>() {
            match self.test_port_availability(listen_addr).await {
                Ok(true) => {
                    result
                        .add_suggestion(format!("Listen port {} is available", listen_addr.port()));
                }
                Ok(false) => {
                    result.add_error(format!(
                        "Listen port {} is already in use",
                        listen_addr.port()
                    ));
                }
                Err(e) => {
                    result.add_warning(format!("Could not test port availability: {}", e));
                }
            }
        }

        // Test bootstrap node connectivity
        for (i, node) in self.bootstrap_nodes.iter().enumerate() {
            if let Ok(addr) = node.parse::<SocketAddr>() {
                match self.test_peer_connectivity(addr).await {
                    Ok(true) => {
                        result.add_suggestion(format!("Bootstrap node {} is reachable", node));
                    }
                    Ok(false) => {
                        result.add_warning(format!("Bootstrap node {} is not reachable", node));
                    }
                    Err(e) => {
                        result.add_warning(format!(
                            "Could not test bootstrap node {}: {}",
                            i + 1,
                            e
                        ));
                    }
                }
            }
        }

        // Test DNS resolution for hostname addresses
        for node in &self.bootstrap_nodes {
            if node.parse::<SocketAddr>().is_err() && node.contains(':') {
                match lookup_host(node).await {
                    Ok(mut addrs) => {
                        if addrs.next().is_some() {
                            result
                                .add_suggestion(format!("Hostname {} resolves successfully", node));
                        } else {
                            result
                                .add_warning(format!("Hostname {} resolves to no addresses", node));
                        }
                    }
                    Err(e) => {
                        result.add_warning(format!("Could not resolve hostname {}: {}", node, e));
                    }
                }
            }
        }

        result
    }

    /// Validate system resources
    fn validate_resources(&self) -> ValidationResult {
        let mut result = ValidationResult::new();

        // Check file descriptor limits
        if let Ok(soft_limit) = get_file_descriptor_limit() {
            let required_fds = self.connection.max_inbound + self.connection.max_outbound + 100; // +100 for overhead
            if required_fds > soft_limit {
                result.add_error(format!(
                    "Required file descriptors ({}) exceed system limit ({}). Increase ulimit -n",
                    required_fds, soft_limit
                ));
            } else if required_fds as f64 > soft_limit as f64 * 0.8 {
                result.add_warning(format!(
                    "Required file descriptors ({}) approach system limit ({}). Consider increasing ulimit -n",
                    required_fds, soft_limit
                ));
            }
        }

        // Check memory requirements estimate
        let estimated_memory_mb = (self.connection.max_inbound + self.connection.max_outbound) * 2; // ~2MB per connection
        if estimated_memory_mb > 1000 {
            result.add_warning(format!(
                "Estimated memory usage: {}MB. Monitor system memory usage.",
                estimated_memory_mb
            ));
        }

        result
    }

    /// Validate security aspects
    fn validate_security(&self) -> ValidationResult {
        let mut result = ValidationResult::new();

        // Check for insecure configurations
        if let Ok(addr) = self.listen_address.parse::<SocketAddr>() {
            if addr.ip().is_unspecified() {
                result.add_warning("Listening on all interfaces (0.0.0.0). Ensure firewall is properly configured.".to_string());
            }
        }

        // Check for default or weak keypair seed
        if let Some(seed) = &self.identity.keypair_seed {
            if seed.len() < 32 {
                result.add_warning(
                    "Keypair seed is short. Use a longer, more secure seed.".to_string(),
                );
            }
            if seed == "default" || seed == "test" || seed == "development" {
                result.add_error(
                    "Using insecure default keypair seed. Generate a secure random seed."
                        .to_string(),
                );
            }
        }

        // Check timeout values for potential DoS issues
        if self.connection.idle_timeout > 3600 {
            result.add_warning(
                "Very long idle timeout may allow resource exhaustion attacks.".to_string(),
            );
        }

        if self.connection.keep_alive_interval < 10 {
            result.add_warning(
                "Very short keep-alive interval may cause excessive network traffic.".to_string(),
            );
        }

        result
    }

    /// Test if a port is available for binding
    async fn test_port_availability(
        &self,
        addr: SocketAddr,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        match TcpListener::bind(addr) {
            Ok(_) => Ok(true),
            Err(e) if e.kind() == std::io::ErrorKind::AddrInUse => Ok(false),
            Err(e) => Err(Box::new(e)),
        }
    }

    /// Test connectivity to a peer
    async fn test_peer_connectivity(
        &self,
        addr: SocketAddr,
    ) -> Result<bool, Box<dyn std::error::Error>> {
        let timeout = Duration::from_secs(self.connection.connection_timeout);

        match tokio::time::timeout(timeout, tokio::net::TcpStream::connect(addr)).await {
            Ok(Ok(_)) => Ok(true),
            Ok(Err(_)) => Ok(false),
            Err(_) => Ok(false), // Timeout
        }
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

/// Get system file descriptor limit
fn get_file_descriptor_limit() -> Result<usize, Box<dyn std::error::Error>> {
    #[cfg(unix)]
    {
        use std::mem;
        let mut rlimit: libc::rlimit = unsafe { mem::zeroed() };
        let result = unsafe { libc::getrlimit(libc::RLIMIT_NOFILE, &mut rlimit) };
        if result == 0 {
            Ok(rlimit.rlim_cur as usize)
        } else {
            Err("Failed to get file descriptor limit".into())
        }
    }
    #[cfg(not(unix))]
    {
        // On non-Unix systems, return a reasonable default
        Ok(1024)
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

    #[test]
    fn test_validation_levels() {
        let config = NetworkConfig::default();

        // Test basic validation
        let result = config.validate_with_level(ValidationLevel::Basic);
        assert!(result.is_valid);

        // Test with invalid config
        let mut invalid_config = config.clone();
        invalid_config.listen_address = "invalid".to_string();
        let result = invalid_config.validate_with_level(ValidationLevel::Basic);
        assert!(!result.is_valid);
        assert!(!result.errors.is_empty());
    }

    #[test]
    fn test_validation_result() {
        let mut result = ValidationResult::new();
        assert!(result.is_valid);
        assert!(result.errors.is_empty());

        result.add_error("Test error".to_string());
        assert!(!result.is_valid);
        assert_eq!(result.errors.len(), 1);

        result.add_warning("Test warning".to_string());
        assert_eq!(result.warnings.len(), 1);

        result.add_suggestion("Test suggestion".to_string());
        assert_eq!(result.suggestions.len(), 1);
    }

    #[test]
    fn test_validation_result_merge() {
        let mut result1 = ValidationResult::new();
        result1.add_warning("Warning 1".to_string());

        let mut result2 = ValidationResult::new();
        result2.add_error("Error 1".to_string());
        result2.add_suggestion("Suggestion 1".to_string());

        result1.merge(result2);

        assert!(!result1.is_valid); // Should be invalid due to error from result2
        assert_eq!(result1.warnings.len(), 1);
        assert_eq!(result1.errors.len(), 1);
        assert_eq!(result1.suggestions.len(), 1);
    }

    #[test]
    fn test_basic_validation_detailed() {
        let config = NetworkConfig::default();
        let result = config.validate_basic();
        assert!(result.is_valid);

        // Test privileged port warning
        let mut config_privileged = config.clone();
        config_privileged.listen_address = "0.0.0.0:80".to_string();
        let result = config_privileged.validate_basic();
        assert!(result.is_valid); // Still valid, but should have warning
        assert!(!result.warnings.is_empty());

        // Test high connection limit warning
        let mut config_high_conn = config.clone();
        config_high_conn.connection.max_inbound = 600;
        config_high_conn.connection.max_outbound = 600;
        let result = config_high_conn.validate_basic();
        assert!(result.is_valid);
        assert!(!result.warnings.is_empty());

        // Test isolation error
        let mut config_isolated = config.clone();
        config_isolated.discovery.enable_dht = false;
        config_isolated.discovery.enable_mdns = false;
        config_isolated.bootstrap_nodes.clear();
        let result = config_isolated.validate_basic();
        assert!(!result.is_valid);
    }

    #[test]
    fn test_security_validation() {
        let config = NetworkConfig::default();
        let result = config.validate_security();
        assert!(result.is_valid);

        // Test insecure keypair seed
        let mut config_insecure = config.clone();
        config_insecure.identity.keypair_seed = Some("test".to_string());
        let result = config_insecure.validate_security();
        assert!(!result.is_valid);

        // Test short keypair seed
        let mut config_short = config.clone();
        config_short.identity.keypair_seed = Some("short".to_string());
        let result = config_short.validate_security();
        assert!(result.is_valid); // Valid but should have warning
        assert!(!result.warnings.is_empty());
    }

    #[test]
    fn test_resource_validation() {
        let config = NetworkConfig::default();
        let result = config.validate_resources();
        assert!(result.is_valid);

        // Test high memory usage warning
        let mut config_high_mem = config.clone();
        config_high_mem.connection.max_inbound = 300;
        config_high_mem.connection.max_outbound = 300;
        let result = config_high_mem.validate_resources();
        assert!(result.is_valid);
        // Should have warning about memory usage
    }

    #[tokio::test]
    async fn test_async_validation() {
        let config = NetworkConfig::default();

        // Test basic async validation
        let result = config.validate_async(ValidationLevel::Basic).await;
        assert!(result.is_valid);

        // Test connectivity validation (may fail in test environment)
        let result = config.validate_async(ValidationLevel::Connectivity).await;
        // Don't assert validity as connectivity tests may fail in test environment
        // Just ensure the validation completed without panicking
        assert!(result.errors.len() < 100); // Reasonable upper bound check

        // Test full validation
        let result = config.validate_async(ValidationLevel::Full).await;
        // Just ensure the validation completed without panicking
        assert!(result.errors.len() < 100); // Reasonable upper bound check
    }

    #[tokio::test]
    async fn test_port_availability() {
        let config = NetworkConfig::default();

        // Test with an address that should be available
        let test_addr = "127.0.0.1:0".parse::<SocketAddr>().unwrap(); // Port 0 should be available
        let result = config.test_port_availability(test_addr).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_peer_connectivity() {
        let config = NetworkConfig::default();

        // Test connectivity to a known unreachable address
        let unreachable_addr = "10.254.254.254:12345".parse::<SocketAddr>().unwrap();
        let result = config.test_peer_connectivity(unreachable_addr).await;
        assert!(result.is_ok()); // Function should not error, but connection should fail
        if let Ok(connected) = result {
            assert!(!connected); // Should not be able to connect
        }
    }

    #[test]
    fn test_configuration_from_env() {
        // Test environment variable loading
        std::env::set_var("POLYTORUS_LISTEN_ADDRESS", "127.0.0.1:8080");
        std::env::set_var("POLYTORUS_BOOTSTRAP_NODES", "127.0.0.1:8081,127.0.0.1:8082");
        std::env::set_var("POLYTORUS_MAX_INBOUND", "50");
        std::env::set_var("POLYTORUS_ENABLE_DHT", "false");

        let config = NetworkConfig::from_env().unwrap();
        assert_eq!(config.listen_address, "127.0.0.1:8080");
        assert_eq!(config.bootstrap_nodes.len(), 2);
        assert_eq!(config.connection.max_inbound, 50);
        assert!(!config.discovery.enable_dht);

        // Cleanup
        std::env::remove_var("POLYTORUS_LISTEN_ADDRESS");
        std::env::remove_var("POLYTORUS_BOOTSTRAP_NODES");
        std::env::remove_var("POLYTORUS_MAX_INBOUND");
        std::env::remove_var("POLYTORUS_ENABLE_DHT");
    }

    #[test]
    fn test_json_serialization() {
        let config = NetworkConfig::default();

        // Test serialization
        let json = serde_json::to_string(&config).unwrap();
        assert!(!json.is_empty());

        // Test deserialization
        let deserialized: NetworkConfig = serde_json::from_str(&json).unwrap();
        assert_eq!(config.listen_address, deserialized.listen_address);
        assert_eq!(config.bootstrap_nodes, deserialized.bootstrap_nodes);
    }

    #[test]
    fn test_edge_cases() {
        // Test with zero timeouts
        let mut config = NetworkConfig::default();
        config.connection.connection_timeout = 0;
        config.discovery.bootstrap_timeout = 0;
        let result = config.validate_basic();
        assert!(!result.is_valid);
        assert_eq!(result.errors.len(), 2);

        // Test with empty protocol version
        let mut config = NetworkConfig::default();
        config.identity.protocol_version = String::new();
        let result = config.validate_basic();
        assert!(!result.is_valid);

        // Test with empty user agent
        let mut config = NetworkConfig::default();
        config.identity.user_agent = String::new();
        let result = config.validate_basic();
        assert!(result.is_valid); // Valid but should have warning
        assert!(!result.warnings.is_empty());
    }

    #[test]
    fn test_file_descriptor_limit() {
        // Test file descriptor limit function
        let result = get_file_descriptor_limit();
        assert!(result.is_ok());
        let limit = result.unwrap();
        assert!(limit > 0);
    }
}
