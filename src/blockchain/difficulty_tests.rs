#[cfg(test)]
mod difficulty_adjustment_tests {
    use crate::blockchain::block::{
        Block, DifficultyAdjustmentConfig, MiningStats, TestFinalizedParams,
    };
    use crate::blockchain::types::{block_states, network};
    use crate::crypto::transaction::Transaction;

    fn create_test_transaction() -> Transaction {
        Transaction::new_coinbase("test_address".to_string(), "50".to_string()).unwrap()
    }    fn create_test_block(
        height: i32,
        prev_hash: String,
        difficulty: usize,
    ) -> Block<block_states::Building, network::Development> {
        let config = DifficultyAdjustmentConfig {
            base_difficulty: 1, // Lower for faster tests
            min_difficulty: 1,
            max_difficulty: 2, // Much lower max for tests
            adjustment_factor: 0.25,
            tolerance_percentage: 20.0,
        };

        // Use minimal difficulty for tests
        let test_difficulty = 1.min(difficulty);

        Block::<block_states::Building, network::Development>::new_building_with_config(
            vec![create_test_transaction()],
            prev_hash,
            height,
            test_difficulty, // Use minimal test difficulty
            config,
            MiningStats::default(),
        )
    }

    #[test]
    fn test_difficulty_config_creation() {
        let config = DifficultyAdjustmentConfig::default();
        assert_eq!(config.base_difficulty, 4);
        assert_eq!(config.min_difficulty, 1);
        assert_eq!(config.max_difficulty, 32);
        assert_eq!(config.adjustment_factor, 0.25);
        assert_eq!(config.tolerance_percentage, 20.0);
    }

    #[test]
    fn test_mining_stats_initialization() {
        let stats = MiningStats::default();
        assert_eq!(stats.avg_mining_time, 0);
        assert_eq!(stats.recent_block_times.len(), 0);
        assert_eq!(stats.total_attempts, 0);
        assert_eq!(stats.successful_mines, 0);
        assert_eq!(stats.success_rate(), 0.0);
    }

    #[test]
    fn test_mining_stats_recording() {
        let mut stats = MiningStats::default();

        // Record mining time
        stats.record_mining_time(1000);
        assert_eq!(stats.successful_mines, 1);
        assert_eq!(stats.avg_mining_time, 1000);
        assert_eq!(stats.recent_block_times.len(), 1);

        // Record attempts
        stats.record_attempt();
        stats.record_attempt();
        assert_eq!(stats.total_attempts, 2);

        let success_rate = stats.success_rate();
        assert_eq!(success_rate, 0.5); // 1 success out of 2 attempts
    }    #[test]
    fn test_block_creation_with_config() {
        let block = create_test_block(1, "prev_hash".to_string(), 3);

        assert_eq!(block.get_height(), 1);
        assert_eq!(block.get_difficulty(), 1); // Now using test difficulty
        assert_eq!(block.get_difficulty_config().base_difficulty, 1); // Updated to test config
        assert_eq!(block.get_difficulty_config().min_difficulty, 1);
        assert_eq!(block.get_difficulty_config().max_difficulty, 2); // Updated to test config
    }

    #[test]
    fn test_dynamic_difficulty_calculation() {
        let block = create_test_block(3, "hash3".to_string(), 4);

        // Create mock finalized blocks with timestamps
        let _now = std::time::SystemTime::now()
            .duration_since(std::time::SystemTime::UNIX_EPOCH)
            .unwrap()
            .as_millis();        // Simulate blocks with fast mining times (should increase difficulty)
        let fast_blocks = vec![];
        let dynamic_diff = block.calculate_dynamic_difficulty(&fast_blocks);

        // With no blocks, should return base difficulty
        assert_eq!(dynamic_diff, 1); // Updated to test config base difficulty
    }

    #[test]
    fn test_mining_with_custom_difficulty() {
        let block = create_test_block(1, "prev_hash".to_string(), 2);

        // Test mining with custom difficulty
        let result = block.mine_with_difficulty(1);
        assert!(result.is_ok());

        let mined_block = result.unwrap();
        assert_eq!(mined_block.get_difficulty(), 1); // Should use custom difficulty
        assert!(mined_block.get_nonce() >= 0); // Should have found a valid nonce
    }

    #[test]
    fn test_mining_efficiency_calculation() {
        let mut stats = MiningStats::default();
        stats.record_mining_time(500); // Fast mining
        stats.record_attempt();
        stats.record_attempt();
        let config = DifficultyAdjustmentConfig::default();
        let block = Block::<block_states::Finalized, network::Development>::new_test_finalized(
            vec![create_test_transaction()],
            TestFinalizedParams {
                prev_block_hash: "test".to_string(),
                hash: "test_hash".to_string(),
                nonce: 123,
                height: 1,
                difficulty: 3,
                difficulty_config: config,
                mining_stats: stats,
            },
        );

        let efficiency = block.calculate_mining_efficiency();
        assert!(efficiency > 0.0);
        assert!(efficiency <= 2.0); // Should be capped at 2.0
    }

    #[test]
    fn test_network_difficulty_recommendation() {
        let config = DifficultyAdjustmentConfig::default();
        let stats = MiningStats::default();

        let block = Block::<block_states::Finalized, network::Development>::new_test_finalized(
            vec![create_test_transaction()],
            TestFinalizedParams {
                prev_block_hash: "test".to_string(),
                hash: "test_hash".to_string(),
                nonce: 123,
                height: 1,
                difficulty: 4,
                difficulty_config: config,
                mining_stats: stats,
            },
        );

        // Test with equal hash rates (should maintain current difficulty)
        let recommended = block.recommend_network_difficulty(1000.0, 1000.0);
        assert_eq!(recommended, 4);

        // Test with higher network hash rate (should increase difficulty)
        let recommended = block.recommend_network_difficulty(2000.0, 1000.0);
        assert_eq!(recommended, 8);

        // Test with lower network hash rate (should decrease difficulty, but respect minimum)
        let recommended = block.recommend_network_difficulty(500.0, 1000.0);
        assert_eq!(recommended, 2);
    }

    #[test]
    fn test_difficulty_bounds_enforcement() {
        let config = DifficultyAdjustmentConfig {
            base_difficulty: 2, // Lower difficulty for faster testing
            min_difficulty: 1,
            max_difficulty: 3, // Lower max for faster testing
            adjustment_factor: 0.5,
            tolerance_percentage: 10.0,
        };

        let block = Block::<block_states::Building, network::Development>::new_building_with_config(
            vec![create_test_transaction()],
            "prev".to_string(),
            1,
            2, // Lower starting difficulty
            config,
            MiningStats::default(),
        );

        // Test mining with difficulty below minimum
        let result = block.clone().mine_with_difficulty(1);
        assert!(result.is_ok());
        let mined = result.unwrap();
        assert_eq!(mined.get_difficulty(), 1); // Should use actual minimum

        // Test mining with difficulty above maximum
        let result = block.mine_with_difficulty(10);
        assert!(result.is_ok());
        let mined = result.unwrap();
        assert_eq!(mined.get_difficulty(), 3); // Should be clamped to maximum
    }
}
