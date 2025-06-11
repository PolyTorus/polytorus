#[cfg(test)]
mod network_integration_tests {
    use crate::modular::network::{ModularNetwork, ModularNetworkConfig};
    use tokio;

    fn create_test_network_config() -> ModularNetworkConfig {
        ModularNetworkConfig {
            listen_address: "127.0.0.1:8080".to_string(),
            bootstrap_peers: vec![
                "127.0.0.1:8081".to_string(),
                "127.0.0.1:8082".to_string(),
            ],
            max_connections: 10,
            request_timeout: 10,
        }
    }

    #[tokio::test]
    async fn test_network_startup_and_peer_connection() {
        let config = create_test_network_config();
        let network = ModularNetwork::new(config).unwrap();

        // Test network startup
        let result = network.start().await;
        assert!(result.is_ok(), "Network should start successfully");

        // Check that bootstrap peers were processed
        let stats = network.get_stats();
        assert_eq!(stats.connected_peers, 2, "Should connect to 2 bootstrap peers");
    }

    #[tokio::test]
    async fn test_data_storage_and_retrieval() {
        let config = create_test_network_config();
        let network = ModularNetwork::new(config).unwrap();

        let test_hash = "test_hash_123";
        let test_data = b"Hello, World!".to_vec();

        // Test data storage
        let result = network.store_data(test_hash, test_data.clone());
        assert!(result.is_ok(), "Data storage should succeed");

        // Test data retrieval
        let retrieved_data = network.get_local_data(test_hash);
        assert!(retrieved_data.is_some(), "Data should be retrievable");
        assert_eq!(retrieved_data.unwrap(), test_data, "Retrieved data should match stored data");

        // Test data availability check
        assert!(network.is_data_available(test_hash), "Data should be available");
        assert!(!network.is_data_available("nonexistent_hash"), "Non-existent data should not be available");
    }

    #[tokio::test]
    async fn test_data_broadcasting() {
        let config = create_test_network_config();
        let network = ModularNetwork::new(config).unwrap();

        // Start network to establish peer connections
        network.start().await.unwrap();

        let test_hash = "broadcast_hash_456";
        let test_data = b"Broadcasting this data".to_vec();

        // Test data broadcasting
        let result = network.broadcast_data(test_hash, &test_data).await;
        assert!(result.is_ok(), "Data broadcasting should succeed");

        // Verify data was stored locally
        let local_data = network.get_local_data(test_hash);
        assert!(local_data.is_some(), "Broadcasted data should be stored locally");
        assert_eq!(local_data.unwrap(), test_data, "Local data should match broadcasted data");
    }

    #[tokio::test]
    async fn test_data_request_response() {
        let config = create_test_network_config();
        let network = ModularNetwork::new(config).unwrap();

        // Start network
        network.start().await.unwrap();

        let test_hash = "request_hash_789";
        let test_data = b"Data to be requested".to_vec();

        // First store some data locally
        network.store_data(test_hash, test_data.clone()).unwrap();

        // Test data request (should find locally)
        let result = network.request_data(test_hash).await;
        assert!(result.is_ok(), "Data request should succeed");
        let retrieved_data = result.unwrap();
        assert!(retrieved_data.is_some(), "Data should be found");
        assert_eq!(retrieved_data.unwrap(), test_data, "Retrieved data should match stored data");

        // Test request for non-existent data
        let result = network.request_data("nonexistent_hash").await;
        assert!(result.is_ok(), "Request should complete without error");
        let retrieved_data = result.unwrap();
        assert!(retrieved_data.is_none(), "Non-existent data should return None");
    }

    #[tokio::test]
    async fn test_peer_connection_limits() {
        let mut config = create_test_network_config();
        config.max_connections = 2; // Set low limit for testing
        config.bootstrap_peers = vec![
            "127.0.0.1:8081".to_string(),
            "127.0.0.1:8082".to_string(),
            "127.0.0.1:8083".to_string(), // This should exceed the limit
        ];

        let network = ModularNetwork::new(config).unwrap();

        // Start network - should handle connection limit
        let result = network.start().await;
        assert!(result.is_ok(), "Network should start even with connection limit");

        let stats = network.get_stats();
        assert!(stats.connected_peers <= 2, "Should respect connection limit");
    }

    #[tokio::test]
    async fn test_network_statistics() {
        let config = create_test_network_config();
        let network = ModularNetwork::new(config).unwrap();

        // Start network
        network.start().await.unwrap();

        // Store some data and make requests
        network.store_data("hash1", b"data1".to_vec()).unwrap();
        network.store_data("hash2", b"data2".to_vec()).unwrap();
        network.broadcast_data("hash3", b"data3").await.unwrap();

        let stats = network.get_stats();
        assert_eq!(stats.stored_data_items, 3, "Should track stored data items");
        assert!(stats.connected_peers > 0, "Should have connected peers");
    }

    #[tokio::test]
    async fn test_error_handling() {
        let config = create_test_network_config();
        let network = ModularNetwork::new(config).unwrap();

        // Start network
        network.start().await.unwrap();

        // Test behavior with unreachable peers
        let mut config_with_bad_peer = create_test_network_config();
        config_with_bad_peer.bootstrap_peers = vec!["unreachable_peer:9999".to_string()];
        let network_bad = ModularNetwork::new(config_with_bad_peer).unwrap();

        // Should handle connection failures gracefully
        let result = network_bad.start().await;
        assert!(result.is_ok(), "Network should handle connection failures gracefully");
    }
}
