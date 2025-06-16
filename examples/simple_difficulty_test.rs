//! Simple difficulty adjustment test

use polytorus::{
    blockchain::{
        block::{Block, DifficultyAdjustmentConfig, MiningStats},
        types::{block_states, network},
    },
    crypto::transaction::Transaction,
};

fn main() -> polytorus::Result<()> {
    println!("=== Simple Difficulty Adjustment Demo ===");

    // Create transaction
    let tx = Transaction::new_coinbase("test_address".to_string(), "reward".to_string())?;

    // Difficulty configuration
    let config = DifficultyAdjustmentConfig {
        base_difficulty: 1, // Very low difficulty
        min_difficulty: 1,
        max_difficulty: 3,
        adjustment_factor: 0.25,
        tolerance_percentage: 20.0,
    };
    // Create block
    let building_block =
        Block::<block_states::Building, network::Development>::new_building_with_config(
            vec![tx],
            "genesis".to_string(),
            1,
            1,
            config,
            MiningStats::default(),
        );

    println!("1. Block creation completed");
    println!("   - Height: {}", building_block.get_height());
    println!("   - Difficulty: {}", building_block.get_difficulty());

    // Mining
    println!("\n2. Starting mining...");
    let mined_block = building_block.mine()?;
    println!("   - Mining completed!");
    println!("   - Nonce: {}", mined_block.get_nonce());
    println!("   - Hash: {}", &mined_block.get_hash()[..16]);

    // Display statistics
    let stats = mined_block.get_mining_stats();
    println!("\n3. Mining statistics:");
    println!("   - Attempts: {}", stats.total_attempts);
    println!("   - Successful mines: {}", stats.successful_mines);
    println!("   - Average time: {}ms", stats.avg_mining_time);

    println!("\n=== Demo Complete ===");
    Ok(())
}
