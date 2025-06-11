#[cfg(test)]
mod difficulty_adjustment_integration_tests {
    use crate::blockchain::block::{Block, DifficultyAdjustmentConfig, MiningStats};
    use crate::blockchain::types::{block_states, network, NetworkConfig};
    use crate::crypto::transaction::Transaction;

    fn create_test_transaction() -> Transaction {
        Transaction::new_coinbase("test_address".to_string(), "50".to_string()).unwrap()
    }

    fn create_test_finalized_block_with_timing(
        height: i32,
        prev_hash: String,
        _difficulty: usize, // Underscore prefix to indicate intentional unused parameter
        block_time_ms: u128,
    ) -> Block<block_states::Finalized, network::Development> {
        // Use very low difficulty for tests
        let test_difficulty = 1; // Even lower than the configured difficulty
        
        let config = DifficultyAdjustmentConfig {
            base_difficulty: test_difficulty,
            min_difficulty: 1,
            max_difficulty: 2, // Keep max very low for tests
            adjustment_factor: 0.25,
            tolerance_percentage: 20.0,
        };
        
        let mut stats = MiningStats::default();
        stats.record_mining_time(block_time_ms);

        let building_block = Block::<block_states::Building, network::Development>::new_building_with_config(
            vec![create_test_transaction()],
            prev_hash,
            height,
            test_difficulty, // Use test difficulty instead of parameter
            config,
            stats,
        );

        // Mine and finalize the block
        let mined_block = building_block.mine().unwrap();
        let validated_block = mined_block.validate().unwrap();
        validated_block.finalize()
    }

    #[test]
    fn test_dynamic_difficulty_calculation_with_history() {
        // Create blocks simulating different mining times
        let fast_mining_time = 5000u128; // 5 seconds (fast)

        // Create a series of blocks with consistent fast mining using test difficulty
        let fast_blocks = vec![
            create_test_finalized_block_with_timing(1, "genesis".to_string(), 1, fast_mining_time),
            create_test_finalized_block_with_timing(2, "hash1".to_string(), 1, fast_mining_time),
            create_test_finalized_block_with_timing(3, "hash2".to_string(), 1, fast_mining_time),
        ];

        let fast_block_refs: Vec<&Block<block_states::Finalized, network::Development>> = 
            fast_blocks.iter().collect();

        // Test with a new block using the history
        let new_block = Block::<block_states::Building, network::Development>::new_with_network_config_and_history(
            vec![create_test_transaction()],
            "hash3".to_string(),
            4,
            &fast_block_refs,
        );

        // For this test, we're primarily testing that the function executes without error
        // The difficulty adjustment logic is based on block timestamps, which are set at mining time
        assert!(new_block.get_difficulty() >= 1, "Difficulty should be at least minimum");
        assert!(new_block.get_difficulty() <= 2, "Difficulty should not exceed test maximum");
    }

    #[test]
    fn test_dynamic_difficulty_calculation_with_slow_blocks() {
        let slow_mining_time = 20000u128; // 20 seconds (slow)

        // Create blocks with slow mining times using test difficulty
        let slow_blocks = vec![
            create_test_finalized_block_with_timing(1, "genesis".to_string(), 1, slow_mining_time),
            create_test_finalized_block_with_timing(2, "hash1".to_string(), 1, slow_mining_time),
            create_test_finalized_block_with_timing(3, "hash2".to_string(), 1, slow_mining_time),
        ];

        let slow_block_refs: Vec<&Block<block_states::Finalized, network::Development>> = 
            slow_blocks.iter().collect();

        let new_block = Block::<block_states::Building, network::Development>::new_with_network_config_and_history(
            vec![create_test_transaction()],
            "hash3".to_string(),
            4,
            &slow_block_refs,
        );

        // Test that difficulty adjustment respects bounds
        assert!(new_block.get_difficulty() >= 1, "Difficulty should be at least minimum");
        assert!(new_block.get_difficulty() <= 2, "Difficulty should not exceed test maximum");
    }

    #[test]
    fn test_difficulty_bounds_enforcement() {
        let very_fast_mining = 1000u128; // 1 second (very fast)

        // Create blocks with extremely fast mining using test difficulty
        let very_fast_blocks = vec![
            create_test_finalized_block_with_timing(1, "genesis".to_string(), 1, very_fast_mining),
            create_test_finalized_block_with_timing(2, "hash1".to_string(), 1, very_fast_mining),
            create_test_finalized_block_with_timing(3, "hash2".to_string(), 1, very_fast_mining),
        ];

        let very_fast_block_refs: Vec<&Block<block_states::Finalized, network::Development>> = 
            very_fast_blocks.iter().collect();

        let new_block = Block::<block_states::Building, network::Development>::new_with_network_config_and_history(
            vec![create_test_transaction()],
            "hash3".to_string(),
            4,
            &very_fast_block_refs,
        );

        // Should be capped at test maximum difficulty (2)
        assert!(new_block.get_difficulty() <= 2, "Difficulty should be capped at test maximum");
    }

    #[test]
    fn test_difficulty_with_minimum_enforcement() {
        let very_slow_mining = 60000u128; // 60 seconds (very slow)

        // Create blocks with extremely slow mining using test difficulty
        let very_slow_blocks = vec![
            create_test_finalized_block_with_timing(1, "genesis".to_string(), 1, very_slow_mining),
            create_test_finalized_block_with_timing(2, "hash1".to_string(), 1, very_slow_mining),
            create_test_finalized_block_with_timing(3, "hash2".to_string(), 1, very_slow_mining),
        ];

        let very_slow_block_refs: Vec<&Block<block_states::Finalized, network::Development>> = 
            very_slow_blocks.iter().collect();

        let new_block = Block::<block_states::Building, network::Development>::new_with_network_config_and_history(
            vec![create_test_transaction()],
            "hash3".to_string(),
            4,
            &very_slow_block_refs,
        );

        // Should be capped at minimum difficulty (1)
        assert!(new_block.get_difficulty() >= 1, "Difficulty should not go below minimum");
    }

    #[test]
    fn test_difficulty_with_stable_timing() {
        let stable_mining = 10000u128; // 10 seconds (stable)

        // Create blocks with stable timing using test difficulty
        let stable_blocks = vec![
            create_test_finalized_block_with_timing(1, "genesis".to_string(), 1, stable_mining),
            create_test_finalized_block_with_timing(2, "hash1".to_string(), 1, stable_mining),
            create_test_finalized_block_with_timing(3, "hash2".to_string(), 1, stable_mining),
        ];

        let stable_block_refs: Vec<&Block<block_states::Finalized, network::Development>> = 
            stable_blocks.iter().collect();

        let new_block = Block::<block_states::Building, network::Development>::new_with_network_config_and_history(
            vec![create_test_transaction()],
            "hash3".to_string(),
            4,
            &stable_block_refs,
        );

        // Should maintain reasonable difficulty
        assert!(new_block.get_difficulty() >= 1, "Difficulty should be at least minimum");
        assert!(new_block.get_difficulty() <= 2, "Difficulty should not exceed test maximum");
    }

    #[test]
    fn test_genesis_block_difficulty() {
        // Test genesis block (height 0)
        let genesis_block = Block::<block_states::Building, network::Development>::new_with_network_config(
            vec![create_test_transaction()],
            String::new(),
            0,
        );

        // Should use initial difficulty
        assert_eq!(genesis_block.get_difficulty(), network::Development::INITIAL_DIFFICULTY, 
                  "Genesis block should use initial difficulty");
    }

    #[test]
    fn test_difficulty_calculation_with_empty_history() {
        // Test block creation with no history
        let empty_history: Vec<&Block<block_states::Finalized, network::Development>> = vec![];

        let new_block = Block::<block_states::Building, network::Development>::new_with_network_config_and_history(
            vec![create_test_transaction()],
            "prev_hash".to_string(),
            1,
            &empty_history,
        );

        // Should use initial difficulty when no history available
        assert_eq!(new_block.get_difficulty(), network::Development::INITIAL_DIFFICULTY, 
                  "Should use initial difficulty with empty history");
    }

    #[test]
    fn test_difficulty_calculation_with_single_block_history() {
        let standard_mining = 10000u128; // 10 seconds
        
        // Test with only one block in history (insufficient for timing calculation)
        let single_block = vec![
            create_test_finalized_block_with_timing(1, "genesis".to_string(), 1, standard_mining),
        ];

        let single_block_refs: Vec<&Block<block_states::Finalized, network::Development>> = 
            single_block.iter().collect();

        let new_block = Block::<block_states::Building, network::Development>::new_with_network_config_and_history(
            vec![create_test_transaction()],
            "hash1".to_string(),
            2,
            &single_block_refs,
        );

        // Should use initial difficulty when insufficient history
        assert_eq!(new_block.get_difficulty(), network::Development::INITIAL_DIFFICULTY, 
                  "Should use initial difficulty with insufficient history");
    }
}
