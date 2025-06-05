use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use polytorus::blockchain::block::{Block, DifficultyAdjustmentConfig, MiningStats};
use polytorus::blockchain::types::{block_states, network};
use polytorus::crypto::transaction::Transaction;
use std::time::Duration;

/// Create a test transaction for benchmarking
fn create_test_transaction() -> Transaction {
    Transaction::new_coinbase(
        "benchmark_address".to_string(),
        "benchmark_reward".to_string(),
    )
    .expect("Failed to create test transaction")
}

/// Create a test block for benchmarking
fn create_test_block(difficulty: usize) -> Block<block_states::Building, network::Development> {
    let config = DifficultyAdjustmentConfig {
        base_difficulty: difficulty,
        min_difficulty: 1,
        max_difficulty: 5,
        adjustment_factor: 0.25,
        tolerance_percentage: 20.0,
    };

    Block::<block_states::Building, network::Development>::new_building_with_config(
        vec![create_test_transaction()],
        "benchmark_prev_hash".to_string(),
        1,
        difficulty,
        config,
        MiningStats::default(),
    )
}

/// Benchmark transaction creation
fn benchmark_transaction_creation(c: &mut Criterion) {
    c.bench_function("create_transaction", |b| {
        b.iter(|| black_box(create_test_transaction()));
    });
}

/// Benchmark block creation
fn benchmark_block_creation(c: &mut Criterion) {
    c.bench_function("create_block", |b| {
        b.iter(|| black_box(create_test_block(2)));
    });
}

/// Benchmark mining with different difficulties
fn benchmark_mining_difficulties(c: &mut Criterion) {
    let mut group = c.benchmark_group("mining_difficulties");
    group.measurement_time(Duration::from_secs(10));
    group.sample_size(10);

    for difficulty in [1, 2, 3].iter() {
        group.bench_with_input(
            BenchmarkId::new("difficulty", difficulty),
            difficulty,
            |b, &difficulty| {
                b.iter(|| {
                    let block = create_test_block(difficulty);
                    let mined = black_box(block.mine()).expect("Mining failed");
                    black_box(mined)
                });
            },
        );
    }

    group.finish();
}

/// Benchmark block validation
fn benchmark_block_validation(c: &mut Criterion) {
    c.bench_function("validate_block", |b| {
        b.iter(|| {
            // Create a new block for each iteration to avoid ownership issues
            let test_block = create_test_block(1);
            let mined_block = test_block.mine().expect("Failed to mine test block");
            let validated = black_box(mined_block.validate()).expect("Validation failed");
            black_box(validated)
        });
    });
}

/// Benchmark difficulty calculations
fn benchmark_difficulty_calculations(c: &mut Criterion) {
    let mut group = c.benchmark_group("difficulty_calculations");

    // Create mock finalized blocks by mining them properly
    let finalized_blocks: Vec<Block<block_states::Finalized, network::Development>> = (0..5)
        .map(|i| {
            let building_block =
                Block::<block_states::Building, network::Development>::new_building_with_config(
                    vec![create_test_transaction()],
                    format!("prev_hash_{}", i),
                    i + 1,
                    1, // Low difficulty for fast mining
                    DifficultyAdjustmentConfig::default(),
                    MiningStats::default(),
                );
            building_block
                .mine()
                .unwrap()
                .validate()
                .unwrap()
                .finalize()
        })
        .collect();

    let block_refs: Vec<&Block<block_states::Finalized, network::Development>> =
        finalized_blocks.iter().collect();

    group.bench_function("dynamic_difficulty", |b| {
        let building_block = create_test_block(3);
        b.iter(|| black_box(building_block.calculate_dynamic_difficulty(&block_refs[..])));
    });

    group.bench_function("advanced_difficulty_adjustment", |b| {
        b.iter(|| black_box(finalized_blocks[0].adjust_difficulty_advanced(&block_refs[..])));
    });

    group.finish();
}

/// Benchmark mining statistics operations
fn benchmark_mining_stats(c: &mut Criterion) {
    let mut group = c.benchmark_group("mining_stats");

    let mut stats = MiningStats::default();
    for i in 0..50 {
        stats.record_mining_time(1000 + i * 10);
        stats.record_attempt();
    }

    group.bench_function("record_mining_time", |b| {
        b.iter(|| {
            let mut test_stats = stats.clone();
            test_stats.record_mining_time(1500);
            black_box(());
        });
    });

    group.bench_function("calculate_success_rate", |b| {
        b.iter(|| black_box(stats.success_rate()));
    });

    group.finish();
}

/// Benchmark multiple transactions
fn benchmark_multiple_transactions(c: &mut Criterion) {
    let mut group = c.benchmark_group("multiple_transactions");
    group.measurement_time(Duration::from_secs(15));
    group.sample_size(10);

    for tx_count in [1, 3, 5, 10].iter() {
        group.bench_with_input(
            BenchmarkId::new("transactions", tx_count),
            tx_count,
            |b, &tx_count| {
                b.iter(|| {
                    let transactions: Vec<Transaction> = (0..tx_count)
                        .map(|i| {
                            Transaction::new_coinbase(
                                format!("address_{}", i),
                                format!("reward_{}", i)
                            ).expect("Failed to create transaction")
                        })
                        .collect();

                    let config = DifficultyAdjustmentConfig {
                        base_difficulty: 1,
                        min_difficulty: 1,
                        max_difficulty: 3,
                        adjustment_factor: 0.25,
                        tolerance_percentage: 20.0,
                    };

                    let block = Block::<block_states::Building, network::Development>::new_building_with_config(
                        transactions,
                        "multi_tx_prev".to_string(),
                        1,
                        1,
                        config,
                        MiningStats::default(),
                    );

                    let mined = black_box(block.mine()).expect("Mining failed");
                    black_box(mined)
                });
            },
        );
    }

    group.finish();
}

criterion_group!(
    benches,
    benchmark_transaction_creation,
    benchmark_block_creation,
    benchmark_mining_difficulties,
    benchmark_block_validation,
    benchmark_difficulty_calculations,
    benchmark_mining_stats,
    benchmark_multiple_transactions
);

criterion_main!(benches);
