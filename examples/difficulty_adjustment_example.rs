#!/usr/bin/env rust

//! Sample usage example for difficulty adjustment functionality

use polytorus::blockchain::block::{
    Block, DifficultyAdjustmentConfig, MiningStats, BuildingBlock
};
use polytorus::blockchain::types::{block_states, network};
use polytorus::blockchain::types::block_states::Building;
use polytorus::crypto::transaction::Transaction;

fn main() -> polytorus::Result<()> {
    println!("=== Difficulty Adjustment Feature Demo ===\n");

    // Custom difficulty configuration
    let difficulty_config = DifficultyAdjustmentConfig {
        base_difficulty: 3,
        min_difficulty: 1,
        max_difficulty: 10,
        adjustment_factor: 0.3,
        tolerance_percentage: 15.0,
    };

    let mining_stats = MiningStats::default();

    // Create dummy transaction
    let dummy_transaction = Transaction::new_coinbase("miner_address".to_string(), "coinbase_data".to_string())?;

    println!("1. Basic mining example:");
    let building_block: BuildingBlock<network::Development> = Block::<Building, network::Development>::new_building_with_config(
        vec![dummy_transaction.clone()],
        "previous_hash".to_string(),
        1,
        3,
        difficulty_config.clone(),
        mining_stats.clone(),
    );

    println!("   - Initial difficulty: {}", building_block.get_difficulty());
    println!("   - Configured minimum difficulty: {}", building_block.get_difficulty_config().min_difficulty);
    println!("   - Configured maximum difficulty: {}", building_block.get_difficulty_config().max_difficulty);

    // Execute mining
    let mined_block = building_block.mine()?;
    println!("   - Mining completed! Nonce: {}", mined_block.get_nonce());
    println!("   - Block hash: {}", &mined_block.get_hash()[..16]);

    // Validate and finalize
    let validated_block = mined_block.validate()?;
    let finalized_block = validated_block.finalize();

    println!("\n2. Custom difficulty mining example:");
    let building_block2: BuildingBlock<network::Development> = Block::<Building, network::Development>::new_building_with_config(
        vec![dummy_transaction.clone()],
        finalized_block.get_hash().to_string(),
        2,
        2,
        difficulty_config.clone(),
        mining_stats,
    );

    // Mine with custom difficulty
    let mined_block2 = building_block2.mine_with_difficulty(4)?;
    println!("   - Custom difficulty: {}", mined_block2.get_difficulty());
    println!("   - Mining completed! Nonce: {}", mined_block2.get_nonce());

    let validated_block2 = mined_block2.validate()?;
    let finalized_block2 = validated_block2.finalize();

    println!("\n3. Adaptive difficulty adjustment example:");
    let recent_blocks: Vec<&Block<block_states::Finalized, network::Development>> = vec![
        &finalized_block,
        &finalized_block2,
    ];

    let building_block3: BuildingBlock<network::Development> = Block::<Building, network::Development>::new_building_with_config(
        vec![dummy_transaction],
        finalized_block2.get_hash().to_string(),
        3,
        3,
        difficulty_config,
        MiningStats::default(),
    );

    // Calculate dynamic difficulty
    let dynamic_difficulty = building_block3.calculate_dynamic_difficulty(&recent_blocks);
    println!("   - Calculated dynamic difficulty: {}", dynamic_difficulty);

    // Adaptive mining
    let mined_block3 = building_block3.mine_adaptive(&recent_blocks)?;
    println!("   - Adaptive mining completed! Used difficulty: {}", mined_block3.get_difficulty());

    println!("\n4. Mining statistics display:");
    let stats = mined_block3.get_mining_stats();
    println!("   - Average mining time: {}ms", stats.avg_mining_time);
    println!("   - Total attempts: {}", stats.total_attempts);
    println!("   - Successful mines: {}", stats.successful_mines);
    if stats.total_attempts > 0 {
        println!("   - Success rate: {:.2}%", stats.success_rate() * 100.0);
    }

    let validated_block3 = mined_block3.validate()?;
    let _finalized_block3 = validated_block3.finalize();

    println!("\n=== Demo Complete ===");
    Ok(())
}
