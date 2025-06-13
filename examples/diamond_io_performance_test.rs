use polytorus::diamond_io_integration::{DiamondIOIntegration, DiamondIOConfig};
use std::time::Instant;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("🧪 Diamond IO パフォーマンス比較テスト\n");

    // 1. ダミーモードのテスト
    println!("1️⃣  ダミーモード（開発用）");
    let start = Instant::now();
    let dummy_config = DiamondIOConfig::dummy();
    let dummy_integration = DiamondIOIntegration::new(dummy_config)?;
    let dummy_circuit = dummy_integration.create_demo_circuit();
    let dummy_obfuscation = dummy_integration.obfuscate_circuit(dummy_circuit).await;
    let dummy_time = start.elapsed();
    println!("   ⏱️  実行時間: {:?}", dummy_time);
    println!("   ✅ 難読化結果: {:?}", dummy_obfuscation.is_ok());

    // 2. テストモードのテスト
    println!("\n2️⃣  テストモード（統合テスト用）");
    let start = Instant::now();
    let test_config = DiamondIOConfig::testing();
    let test_integration = DiamondIOIntegration::new(test_config)?;
    let test_circuit = test_integration.create_demo_circuit();
    let test_obfuscation = test_integration.obfuscate_circuit(test_circuit).await;
    let test_time = start.elapsed();
    println!("   ⏱️  実行時間: {:?}", test_time);
    println!("   ✅ 難読化結果: {:?}", test_obfuscation.is_ok());

    // 3. 本番モードのテスト
    println!("\n3️⃣  本番モード（実際のパラメータ）");
    let start = Instant::now();
    let prod_config = DiamondIOConfig::production();
    let prod_integration = DiamondIOIntegration::new(prod_config)?;
    let prod_circuit = prod_integration.create_demo_circuit();
    let initialization_time = start.elapsed();
    println!("   ⏱️  初期化時間: {:?}", initialization_time);
    
    let start = Instant::now();
    let prod_obfuscation = prod_integration.obfuscate_circuit(prod_circuit).await;
    let obfuscation_time = start.elapsed();
    println!("   ⏱️  難読化時間: {:?}", obfuscation_time);
    println!("   ✅ 難読化結果: {:?}", prod_obfuscation.is_ok());

    // 結果サマリー
    println!("\n📊 パフォーマンス比較サマリー");
    println!("┌─────────────┬─────────────┬─────────────┐");
    println!("│ モード      │ 実行時間    │ 高速化倍率  │");
    println!("├─────────────┼─────────────┼─────────────┤");
    println!("│ ダミー      │ {:>10?} │ {:>10}x │", dummy_time, 1);
    if test_time.as_nanos() > 0 {
        println!("│ テスト      │ {:>10?} │ {:>10.1}x │", test_time, test_time.as_nanos() as f64 / dummy_time.as_nanos() as f64);
    }
    if obfuscation_time.as_nanos() > 0 {
        println!("│ 本番        │ {:>10?} │ {:>10.1}x │", obfuscation_time, obfuscation_time.as_nanos() as f64 / dummy_time.as_nanos() as f64);
    }
    println!("└─────────────┴─────────────┴─────────────┘");

    Ok(())
}
