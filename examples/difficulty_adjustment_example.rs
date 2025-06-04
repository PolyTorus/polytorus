#!/usr/bin/env rust

//! 難易度調整機能のサンプル使用例

use polytorus::blockchain::block::{
    Block, DifficultyAdjustmentConfig, MiningStats, BuildingBlock
};
use polytorus::blockchain::types::{block_states, network};
use polytorus::blockchain::types::block_states::Building;
use polytorus::crypto::transaction::Transaction;

fn main() -> polytorus::Result<()> {
    println!("=== 難易度調整機能のデモ ===\n");

    // カスタム難易度設定
    let difficulty_config = DifficultyAdjustmentConfig {
        base_difficulty: 3,
        min_difficulty: 1,
        max_difficulty: 10,
        adjustment_factor: 0.3,
        tolerance_percentage: 15.0,
    };

    let mining_stats = MiningStats::default();

    // ダミートランザクション作成
    let dummy_transaction = Transaction::new_coinbase("miner_address".to_string(), "coinbase_data".to_string())?;

    println!("1. 基本的なマイニング例:");
    let building_block: BuildingBlock<network::Development> = Block::<Building, network::Development>::new_building_with_config(
        vec![dummy_transaction.clone()],
        "previous_hash".to_string(),
        1,
        3,
        difficulty_config.clone(),
        mining_stats.clone(),
    );

    println!("   - 初期難易度: {}", building_block.get_difficulty());
    println!("   - 設定された最小難易度: {}", building_block.get_difficulty_config().min_difficulty);
    println!("   - 設定された最大難易度: {}", building_block.get_difficulty_config().max_difficulty);

    // マイニング実行
    let mined_block = building_block.mine()?;
    println!("   - マイニング完了! Nonce: {}", mined_block.get_nonce());
    println!("   - ブロックハッシュ: {}", &mined_block.get_hash()[..16]);

    // 検証とファイナライズ
    let validated_block = mined_block.validate()?;
    let finalized_block = validated_block.finalize();

    println!("\n2. カスタム難易度でのマイニング例:");
    let building_block2: BuildingBlock<network::Development> = Block::<Building, network::Development>::new_building_with_config(
        vec![dummy_transaction.clone()],
        finalized_block.get_hash().to_string(),
        2,
        2,
        difficulty_config.clone(),
        mining_stats,
    );

    // カスタム難易度でマイニング
    let mined_block2 = building_block2.mine_with_difficulty(4)?;
    println!("   - カスタム難易度: {}", mined_block2.get_difficulty());
    println!("   - マイニング完了! Nonce: {}", mined_block2.get_nonce());

    let validated_block2 = mined_block2.validate()?;
    let finalized_block2 = validated_block2.finalize();

    println!("\n3. 適応的難易度調整の例:");
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

    // 動的難易度を計算
    let dynamic_difficulty = building_block3.calculate_dynamic_difficulty(&recent_blocks);
    println!("   - 計算された動的難易度: {}", dynamic_difficulty);

    // 適応的マイニング
    let mined_block3 = building_block3.mine_adaptive(&recent_blocks)?;
    println!("   - 適応的マイニング完了! 使用された難易度: {}", mined_block3.get_difficulty());

    println!("\n4. マイニング統計の表示:");
    let stats = mined_block3.get_mining_stats();
    println!("   - 平均マイニング時間: {}ms", stats.avg_mining_time);
    println!("   - 総試行回数: {}", stats.total_attempts);
    println!("   - 成功回数: {}", stats.successful_mines);
    if stats.total_attempts > 0 {
        println!("   - 成功率: {:.2}%", stats.success_rate() * 100.0);
    }

    let validated_block3 = mined_block3.validate()?;
    let _finalized_block3 = validated_block3.finalize();

    println!("\n=== デモ完了 ===");
    Ok(())
}
