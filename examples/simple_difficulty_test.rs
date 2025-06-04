//! シンプルな難易度調整テスト

use polytorus::blockchain::block::{Block, DifficultyAdjustmentConfig, MiningStats};
use polytorus::blockchain::types::{block_states, network};
use polytorus::crypto::transaction::Transaction;

fn main() -> polytorus::Result<()> {
    println!("=== 簡単な難易度調整デモ ===");

    // トランザクション作成
    let tx = Transaction::new_coinbase("test_address".to_string(), "reward".to_string())?;
    
    // 難易度設定
    let config = DifficultyAdjustmentConfig {
        base_difficulty: 1,  // 非常に低い難易度
        min_difficulty: 1,
        max_difficulty: 3,
        adjustment_factor: 0.25,
        tolerance_percentage: 20.0,
    };

    // ブロック作成
    let building_block = Block::<block_states::Building, network::Development>::new_building_with_config(
        vec![tx],
        "genesis".to_string(),
        1,
        1,
        config,
        MiningStats::default(),
    );

    println!("1. ブロック作成完了");
    println!("   - 高さ: {}", building_block.get_height());
    println!("   - 難易度: {}", building_block.get_difficulty());

    // マイニング
    println!("\n2. マイニング開始...");
    let mined_block = building_block.mine()?;
    println!("   - マイニング完了!");
    println!("   - ナンス: {}", mined_block.get_nonce());
    println!("   - ハッシュ: {}", &mined_block.get_hash()[..16]);

    // 統計表示
    let stats = mined_block.get_mining_stats();
    println!("\n3. マイニング統計:");
    println!("   - 試行回数: {}", stats.total_attempts);
    println!("   - 成功回数: {}", stats.successful_mines);
    println!("   - 平均時間: {}ms", stats.avg_mining_time);

    println!("\n=== デモ完了 ===");
    Ok(())
}
