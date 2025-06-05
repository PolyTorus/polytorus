# Difficulty Adjustment System Usage Guide

PolyTorus's new difficulty adjustment system provides advanced functionality that allows fine-grained difficulty adjustments for each mining block.

## Feature Overview

### 1. Flexible Difficulty Settings

```rust
use polytorus::blockchain::block::DifficultyAdjustmentConfig;

let config = DifficultyAdjustmentConfig {
    base_difficulty: 4,        // Base difficulty
    min_difficulty: 1,         // Minimum difficulty
    max_difficulty: 32,        // Maximum difficulty
    adjustment_factor: 0.25,   // Adjustment strength (0.0-1.0)
    tolerance_percentage: 20.0, // Tolerance percentage (%)
};
```

### 2. Mining Statistics Tracking

```rust
use polytorus::blockchain::block::MiningStats;

let mut stats = MiningStats::default();
stats.record_mining_time(1500); // Record mining time
stats.record_attempt();          // Record attempt count

println!("Average mining time: {}ms", stats.avg_mining_time);
println!("Success rate: {:.2}%", stats.success_rate() * 100.0);
```

### 3. Block Creation and Configuration

```rust
use polytorus::blockchain::block::{Block, BuildingBlock};
use polytorus::blockchain::types::network;

// Create block with custom configuration
let building_block: BuildingBlock<network::Development> = Block::new_building_with_config(
    transactions,
    prev_hash,
    height,
    difficulty,
    difficulty_config,
    mining_stats,
);
```

## Mining Methods

### 1. Standard Mining

```rust
let mined_block = building_block.mine()?;
```

### 2. Custom Difficulty Mining

```rust
let mined_block = building_block.mine_with_difficulty(6)?;
```

### 3. Adaptive Mining

```rust
// Dynamically calculate difficulty based on recent blocks
let mined_block = building_block.mine_adaptive(&recent_blocks)?;
```

## Difficulty Adjustment Algorithm

### Dynamic Difficulty Calculation

The system adjusts difficulty considering the following factors:

1. **Average of recent block times**
2. **Comparison with target block time**
3. **Configured tolerance margin**
4. **Adjustment strength parameters**

```rust
let dynamic_difficulty = block.calculate_dynamic_difficulty(&recent_blocks);
```

### Advanced Difficulty Adjustment

Adjustment considering multiple block history and time variance:

```rust
let advanced_difficulty = finalized_block.adjust_difficulty_advanced(&previous_blocks);
```

## Performance Analysis

### Mining Efficiency Calculation

```rust
let efficiency = finalized_block.calculate_mining_efficiency();
println!("Mining efficiency: {:.2}%", efficiency * 100.0);
```

### Network Difficulty Recommendation

```rust
let network_difficulty = finalized_block.recommend_network_difficulty(
    current_hash_rate,
    target_hash_rate
);
```

## Practical Examples

### Scenario 1: Fast Mining in Development Environment

```rust
let dev_config = DifficultyAdjustmentConfig {
    base_difficulty: 1,
    min_difficulty: 1,
    max_difficulty: 4,
    adjustment_factor: 0.5,
    tolerance_percentage: 30.0,
};
```

### Scenario 2: Stable Mining in Production Environment

```rust
let prod_config = DifficultyAdjustmentConfig {
    base_difficulty: 6,
    min_difficulty: 4,
    max_difficulty: 20,
    adjustment_factor: 0.1,
    tolerance_percentage: 10.0,
};
```

### Scenario 3: Experimental Settings for Testnet

```rust
let test_config = DifficultyAdjustmentConfig {
    base_difficulty: 3,
    min_difficulty: 1,
    max_difficulty: 10,
    adjustment_factor: 0.3,
    tolerance_percentage: 25.0,
};
```

## Best Practices

1. **Adjustment Strength**: Range of 0.1-0.3 is recommended
2. **Tolerance Margin**: Set within 10-30% range
3. **Max/Min Difficulty**: Set appropriately according to network performance
4. **Statistics Tracking**: Regularly analyze mining statistics for optimization

## Sample Execution

To run difficulty adjustment sample code:

```bash
cargo run --example difficulty_adjustment
```

This sample demonstrates various difficulty adjustment features usage examples.
