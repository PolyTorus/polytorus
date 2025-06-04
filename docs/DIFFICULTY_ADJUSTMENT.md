# 難易度調整システム使用ガイド

PolyTorusの新しい難易度調整システムは、マイニングブロックごとに細かい難易度調整が可能な高度な機能を提供します。

## 機能概要

### 1. 柔軟な難易度設定

```rust
use polytorus::blockchain::block::DifficultyAdjustmentConfig;

let config = DifficultyAdjustmentConfig {
    base_difficulty: 4,        // 基本難易度
    min_difficulty: 1,         // 最小難易度
    max_difficulty: 32,        // 最大難易度
    adjustment_factor: 0.25,   // 調整の強度 (0.0-1.0)
    tolerance_percentage: 20.0, // 許容誤差 (%)
};
```

### 2. マイニング統計の追跡

```rust
use polytorus::blockchain::block::MiningStats;

let mut stats = MiningStats::default();
stats.record_mining_time(1500); // マイニング時間を記録
stats.record_attempt();          // 試行回数を記録

println!("平均マイニング時間: {}ms", stats.avg_mining_time);
println!("成功率: {:.2}%", stats.success_rate() * 100.0);
```

### 3. ブロック作成と設定

```rust
use polytorus::blockchain::block::{Block, BuildingBlock};
use polytorus::blockchain::types::network;

// カスタム設定でブロックを作成
let building_block: BuildingBlock<network::Development> = Block::new_building_with_config(
    transactions,
    prev_hash,
    height,
    difficulty,
    difficulty_config,
    mining_stats,
);
```

## マイニング方法

### 1. 標準マイニング

```rust
let mined_block = building_block.mine()?;
```

### 2. カスタム難易度でマイニング

```rust
let mined_block = building_block.mine_with_difficulty(6)?;
```

### 3. 適応的マイニング

```rust
// 最近のブロックを基に動的に難易度を計算
let mined_block = building_block.mine_adaptive(&recent_blocks)?;
```

## 難易度調整アルゴリズム

### 動的難易度計算

システムは以下の要素を考慮して難易度を調整します：

1. **最近のブロック時間の平均**
2. **目標ブロック時間との比較**
3. **設定された許容誤差**
4. **調整強度パラメータ**

```rust
let dynamic_difficulty = block.calculate_dynamic_difficulty(&recent_blocks);
```

### 高度な難易度調整

複数ブロックの履歴と時間の分散を考慮した調整：

```rust
let advanced_difficulty = finalized_block.adjust_difficulty_advanced(&previous_blocks);
```

## パフォーマンス分析

### マイニング効率の計算

```rust
let efficiency = finalized_block.calculate_mining_efficiency();
println!("マイニング効率: {:.2}%", efficiency * 100.0);
```

### ネットワーク難易度推奨

```rust
let network_difficulty = finalized_block.recommend_network_difficulty(
    current_hash_rate,
    target_hash_rate
);
```

## 実用的な例

### シナリオ1: 開発環境での高速マイニング

```rust
let dev_config = DifficultyAdjustmentConfig {
    base_difficulty: 1,
    min_difficulty: 1,
    max_difficulty: 4,
    adjustment_factor: 0.5,
    tolerance_percentage: 30.0,
};
```

### シナリオ2: 本番環境での安定したマイニング

```rust
let prod_config = DifficultyAdjustmentConfig {
    base_difficulty: 6,
    min_difficulty: 4,
    max_difficulty: 20,
    adjustment_factor: 0.1,
    tolerance_percentage: 10.0,
};
```

### シナリオ3: テストネットでの実験的設定

```rust
let test_config = DifficultyAdjustmentConfig {
    base_difficulty: 3,
    min_difficulty: 1,
    max_difficulty: 10,
    adjustment_factor: 0.3,
    tolerance_percentage: 25.0,
};
```

## ベストプラクティス

1. **調整強度**: 0.1-0.3の範囲が推奨です
2. **許容誤差**: 10-30%の範囲で設定してください
3. **最大・最小難易度**: ネットワークの性能に応じて適切に設定
4. **統計の追跡**: マイニング統計を定期的に分析して最適化

## サンプル実行

難易度調整のサンプルコードを実行するには：

```bash
cargo run --example difficulty_adjustment
```

このサンプルでは、様々な難易度調整機能の使用例を確認できます。
