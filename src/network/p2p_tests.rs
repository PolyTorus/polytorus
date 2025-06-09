//! Tests for P2P networking
//!
//! This module contains comprehensive tests for the P2P network implementation,
//! including peer discovery, message broadcasting, and network configuration.

#[cfg(test)]
mod tests {
    use super::super::manager::*;
    use super::super::network_config::*;
    use super::super::p2p::*;

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
    }

    #[test]
    fn test_network_config_address_conversion() {
        let config = NetworkConfig::default();
        let listen_addr = config.get_listen_address();
        assert_eq!(listen_addr, "0.0.0.0:9090");

        let bootstrap_addrs = config.get_bootstrap_addresses();
        assert!(bootstrap_addrs.is_empty());
    }

    #[test]
    fn test_network_config_validation() {
        let mut config = NetworkConfig::default();
        assert!(config.validate().is_ok());

        // Test invalid listen address
        config.listen_address = "invalid".to_string();
        assert!(config.validate().is_err());

        // Test valid address
        config.listen_address = "127.0.0.1:9090".to_string();
        assert!(config.validate().is_ok());

        // Test invalid bootstrap node
        config.bootstrap_nodes.push("invalid".to_string());
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_connection_config() {
        let config = ConnectionConfig::default();
        assert_eq!(config.max_inbound, 25);
        assert_eq!(config.max_outbound, 25);
        assert_eq!(config.connection_timeout, 10);
        assert_eq!(config.keep_alive_interval, 30);
        assert_eq!(config.idle_timeout, 300);
    }

    #[test]
    fn test_discovery_config() {
        let config = DiscoveryConfig::default();
        assert!(config.enable_dht);
        assert!(config.enable_mdns);
        assert_eq!(config.bootstrap_timeout, 30);
        assert_eq!(config.discovery_interval, 300);
    }

    /// Test P2P node creation (requires runtime)
    #[tokio::test]
    async fn test_p2p_node_creation() {
        let listen_addr: SocketAddr = "127.0.0.1:0".parse().unwrap();
        let bootstrap_nodes = vec![];

        let result = P2PNode::new(listen_addr, bootstrap_nodes);
        assert!(result.is_ok());

        let (_node, _event_rx, _command_tx) = result.unwrap();
        // Node is created successfully
    }    /// Test network manager creation
    #[tokio::test]
    async fn test_network_manager_creation() {
        let config = NetworkConfig {
            listen_address: "127.0.0.1:0".to_string(), // Use random port for testing
            ..Default::default()
        };

        let result = NetworkManager::new(config).await;
        assert!(result.is_ok());

        let _manager = result.unwrap();
        // Manager is created successfully
    }

    /// Integration test for peer discovery (simplified)
    #[tokio::test]    async fn test_peer_discovery_simulation() {
        // Create two network managers
        let config1 = NetworkConfig {
            listen_address: "127.0.0.1:0".to_string(), // Random port
            ..Default::default()
        };

        let config2 = NetworkConfig {
            listen_address: "127.0.0.1:0".to_string(), // Random port
            ..Default::default()
        };

        let manager1_result = NetworkManager::new(config1).await;
        let manager2_result = NetworkManager::new(config2).await;

        assert!(manager1_result.is_ok());
        assert!(manager2_result.is_ok());

        // In a real test, we would connect these managers and verify discovery
        // This is a placeholder for more complex integration tests
    }

    /// Test network statistics
    #[tokio::test]
    async fn test_network_stats() {
        let config = NetworkConfig::default();
        let manager = NetworkManager::new(config).await.unwrap();

        let stats = manager.get_network_stats();
        assert_eq!(stats.connected_peers, 0);
        assert_eq!(stats.pending_block_requests, 0);
        assert_eq!(stats.pending_transaction_requests, 0);
        assert_eq!(stats.best_height, -1);
    }

    /// Test message serialization/deserialization
    #[test]
    fn test_protocol_message_serialization() {
        let messages = vec![
            P2PMessage::BlockRequest {
                block_hash: "test_hash".to_string(),
            },
            P2PMessage::TransactionRequest {
                tx_hash: "tx_hash".to_string(),
            },
            P2PMessage::Ping {
                nonce: 12345,
                timestamp: 1000000,
            },
            P2PMessage::StatusUpdate { best_height: 42 },
        ];

        for msg in messages {
            let serialized = bincode::serialize(&msg).unwrap();
            let deserialized: P2PMessage = bincode::deserialize(&serialized).unwrap();

            match (&msg, &deserialized) {
                (
                    P2PMessage::BlockRequest { block_hash: h1 },
                    P2PMessage::BlockRequest { block_hash: h2 },
                ) => {
                    assert_eq!(h1, h2);
                }
                (
                    P2PMessage::TransactionRequest { tx_hash: h1 },
                    P2PMessage::TransactionRequest { tx_hash: h2 },
                ) => {
                    assert_eq!(h1, h2);
                }
                (
                    P2PMessage::Ping {
                        nonce: n1,
                        timestamp: t1,
                    },
                    P2PMessage::Ping {
                        nonce: n2,
                        timestamp: t2,
                    },
                ) => {
                    assert_eq!(n1, n2);
                    assert_eq!(t1, t2);
                }
                (
                    P2PMessage::StatusUpdate { best_height: h1 },
                    P2PMessage::StatusUpdate { best_height: h2 },
                ) => {
                    assert_eq!(h1, h2);
                }
                _ => panic!("Message types don't match"),
            }
        }
    }

    /// Test network event handling
    #[test]
    fn test_network_events() {
        let peer_id = PeerId::random();
        let events = vec![
            NetworkEvent::PeerConnected(peer_id),
            NetworkEvent::PeerDisconnected(peer_id),
            NetworkEvent::PeerInfo(peer_id, 100),
        ];

        // Test that events can be created and pattern matched
        for event in events {
            match event {
                NetworkEvent::PeerConnected(id) => assert_eq!(id, peer_id),
                NetworkEvent::PeerDisconnected(id) => assert_eq!(id, peer_id),
                NetworkEvent::PeerInfo(id, height) => {
                    assert_eq!(id, peer_id);
                    assert_eq!(height, 100);
                }
                _ => {}
            }
        }
    }

    /// Test network commands
    #[test]
    fn test_network_commands() {
        use std::net::SocketAddr;

        let peer_id = PeerId::random();
        let socket_addr: SocketAddr = "127.0.0.1:9090".parse().unwrap();

        let commands = vec![
            NetworkCommand::ConnectPeer(socket_addr),
            NetworkCommand::RequestBlock("test_hash".to_string(), peer_id),
            NetworkCommand::RequestTransaction("tx_hash".to_string(), peer_id),
            NetworkCommand::GetPeers,
        ];

        // Test that commands can be created and pattern matched
        for command in commands {
            match command {
                NetworkCommand::ConnectPeer(addr) => assert_eq!(addr, socket_addr),
                NetworkCommand::RequestBlock(hash, id) => {
                    assert_eq!(hash, "test_hash");
                    assert_eq!(id, peer_id);
                }
                NetworkCommand::RequestTransaction(hash, id) => {
                    assert_eq!(hash, "tx_hash");
                    assert_eq!(id, peer_id);
                }
                NetworkCommand::GetPeers => {}
                _ => {}
            }
        }
    }

    /// Test bootstrap node management
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

    /// Test configuration validation edge cases
    #[test]
    fn test_config_validation_edge_cases() {
        let mut config = NetworkConfig::default();

        // Test zero connections
        config.connection.max_inbound = 0;
        config.connection.max_outbound = 0;
        assert!(config.validate().is_err());

        // Restore valid connection count
        config.connection.max_outbound = 1;
        assert!(config.validate().is_ok());

        // Test zero timeout
        config.connection.connection_timeout = 0;
        assert!(config.validate().is_err());

        config.connection.connection_timeout = 1;
        config.discovery.bootstrap_timeout = 0;
        assert!(config.validate().is_err());
    }

    /// Test peer status tracking
    #[test]
    fn test_peer_status() {
        use std::time::Instant;

        let peer_id = PeerId::random();
        let now = Instant::now();

        let mut peer_status = PeerStatus {
            peer_id,
            best_height: 100,
            connected_at: now,
            last_activity: now,
            successful_interactions: 5,
            failed_interactions: 1,
        };

        assert_eq!(peer_status.peer_id, peer_id);
        assert_eq!(peer_status.best_height, 100);
        assert_eq!(peer_status.successful_interactions, 5);
        assert_eq!(peer_status.failed_interactions, 1);

        // Update peer status
        peer_status.best_height = 101;
        peer_status.successful_interactions += 1;

        assert_eq!(peer_status.best_height, 101);
        assert_eq!(peer_status.successful_interactions, 6);
    }

    /// Performance test for message handling
    #[test]
    fn test_message_performance() {
        use std::time::Instant;

        let start = Instant::now();

        // Simulate processing 1000 messages
        for i in 0..1000 {
            let msg = P2PMessage::StatusUpdate { best_height: i };
            let serialized = bincode::serialize(&msg).unwrap();
            let _deserialized: P2PMessage = bincode::deserialize(&serialized).unwrap();
        }

        let duration = start.elapsed();
        println!("Processed 1000 messages in {:?}", duration);

        // This should complete quickly (less than 1 second on modern hardware)
        assert!(duration < Duration::from_secs(1));
    }

    /// Test network configuration from environment
    #[test]
    fn test_config_from_env() {
        // This test would be more comprehensive with actual environment variables
        // For now, just test that the function doesn't panic
        let result = NetworkConfig::from_env();
        assert!(result.is_ok());

        let config = result.unwrap();
        assert!(config.validate().is_ok());
    }

    /// Test configuration helper functions
    #[test]
    fn test_config_helpers() {
        let config = NetworkConfig::default();

        assert_eq!(config.max_connections(), 50); // 25 + 25
        assert!(config.is_dht_enabled());
        assert!(config.is_local_discovery_enabled());
    }
}

/// Integration tests that require multiple nodes
#[cfg(test)]
mod integration_tests {

    /// This would be a comprehensive integration test
    /// In practice, you'd want to set up multiple nodes and test:
    /// - Peer discovery
    /// - Block propagation
    /// - Transaction broadcasting
    /// - Network partitions and recovery
    /// - Load balancing and failover
    #[tokio::test]
    #[ignore] // Ignored by default as it requires more setup
    async fn test_full_network_integration() {
        // This test would create multiple network managers,
        // connect them, and verify full blockchain functionality
        // across the network. It's marked as ignored because
        // it requires significant setup and may be slow.

        println!("Full integration test would go here");
        // TODO: Implement comprehensive integration test
    }
}
