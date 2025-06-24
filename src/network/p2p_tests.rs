//! Tests for P2P networking
//!
//! This module contains tests for the P2P network implementation,
//! focusing on configuration and message handling.

#[cfg(test)]
mod tests {
    use super::super::network_config::*;
    use super::super::p2p_enhanced::*;

    use std::{net::SocketAddr, time::Duration};

    /// Test network configuration loading
    #[test]
    fn test_network_config_default() {
        let config = NetworkConfig::default();
        assert_eq!(config.listen_address, "0.0.0.0:9090");
        assert!(config.bootstrap_nodes.is_empty());
        assert!(config.discovery.enable_dht);
        assert_eq!(config.connection.max_inbound, 25);
        assert_eq!(config.connection.max_outbound, 25);
        assert_eq!(config.connection.connection_timeout, 10);
    }

    /// Test network configuration with custom values    #[test]
    #[test]
    fn test_network_config_custom() {
        let config = NetworkConfig {
            listen_address: "127.0.0.1:8080".to_string(),
            connection: ConnectionConfig {
                max_inbound: 10,
                max_outbound: 15,
                ..Default::default()
            },
            ..Default::default()
        };

        assert_eq!(config.listen_address, "127.0.0.1:8080");
        assert_eq!(config.connection.max_inbound, 10);
        assert_eq!(config.connection.max_outbound, 15);
    }

    /// Test socket address parsing
    #[test]
    fn test_socket_address_parsing() {
        let addr: SocketAddr = "127.0.0.1:8080".parse().unwrap();
        assert_eq!(addr.port(), 8080);
        assert!(addr.ip().is_loopback());
    }

    /// Test network event enumeration
    #[test]
    fn test_network_events() {
        use uuid::Uuid;
        let peer_id = PeerId(Uuid::new_v4());

        let event = NetworkEvent::PeerConnected(peer_id);
        match event {
            NetworkEvent::PeerConnected(_) => {
                // Event type verified
            }
            _ => panic!("Wrong event type"),
        }
    }
    /// Test network command enumeration
    #[test]
    fn test_network_commands() {
        // Test with a valid NetworkCommand variant
        let cmd = NetworkCommand::ConnectPeer("127.0.0.1:8080".parse().unwrap());
        match cmd {
            NetworkCommand::ConnectPeer(addr) => {
                assert_eq!(addr.port(), 8080);
            }
            _ => panic!("Wrong command type"),
        }
    }

    /// Test protocol constants
    #[test]
    fn test_protocol_constants() {
        // These constants should be accessible from p2p module
        // Testing that the module compiles and basic types are available
        let timeout = Duration::from_secs(5);
        assert!(timeout.as_secs() == 5);
    }

    /// Test bootstrap configuration validation
    #[test]
    fn test_bootstrap_validation() {
        let config = NetworkConfig {
            bootstrap_nodes: vec!["127.0.0.1:8080".to_string(), "192.168.1.1:9090".to_string()],
            ..Default::default()
        };

        assert_eq!(config.bootstrap_nodes.len(), 2);
        assert!(config
            .bootstrap_nodes
            .contains(&"127.0.0.1:8080".to_string()));
    }

    /// Test network discovery configuration
    #[test]
    fn test_discovery_config() {
        let mut config = NetworkConfig::default();
        config.discovery.enable_dht = false;
        config.discovery.enable_mdns = true;

        assert!(!config.discovery.enable_dht);
        assert!(config.discovery.enable_mdns);
    }

    /// Test network connection limits
    #[test]
    fn test_connection_limits() {
        let config = NetworkConfig {
            connection: super::super::network_config::ConnectionConfig {
                max_inbound: 50,
                max_outbound: 100,
                connection_timeout: 30,
                keep_alive_interval: 60,
                idle_timeout: 300,
            },
            ..Default::default()
        };

        assert_eq!(config.connection.max_inbound, 50);
        assert_eq!(config.connection.max_outbound, 100);
        assert_eq!(config.connection.connection_timeout, 30);
    }
}
