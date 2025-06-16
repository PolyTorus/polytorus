use std::time::Duration;

use criterion::{
    black_box,
    criterion_group,
    criterion_main,
    BenchmarkId,
    Criterion,
};
use polytorus::blockchain::block::{
    Block,
    DifficultyAdjustmentConfig,
    MiningStats,
};
use polytorus::blockchain::types::{
    block_states,
    network,
};
use polytorus::crypto::transaction::{
    TXInput,
    TXOutput,
    Transaction,
};

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
                    format!("prev_hash_{i}"),
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
            |b, &tx_count| {                b.iter(|| {
                    // Create first transaction as coinbase
                    let mut transactions = vec![create_test_transaction()];                    // Add regular transactions if needed
                    for i in 1..tx_count {
                        let tx = create_simple_transaction(
                            format!("multi_addr_{i}"),
                            format!("multi_dest_{i}"),
                            10 + i,
                            i,
                        );
                        transactions.push(tx);
                    }

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

/// Create a simple test transaction (non-coinbase)
fn create_simple_transaction(from: String, to: String, amount: i32, nonce: i32) -> Transaction {
    // Create a fake input referencing a previous transaction
    let prev_tx_id = format!("prev_tx_{nonce}");
    let input = TXInput {
        txid: prev_tx_id,
        vout: 0,
        signature: Vec::new(),
        pub_key: format!("pubkey_{from}").into_bytes(),
        redeemer: None,
    };

    // Create output
    let output = TXOutput::new(amount, to).expect("Failed to create output");

    let mut tx = Transaction {
        id: String::new(),
        vin: vec![input],
        vout: vec![output],
        contract_data: None,
    };

    // Generate transaction ID
    tx.id = tx.hash().expect("Failed to hash transaction");
    tx
}

/// TPS (Transactions Per Second) benchmark
fn benchmark_tps(c: &mut Criterion) {
    let mut group = c.benchmark_group("tps_throughput");
    group.measurement_time(Duration::from_secs(20));
    group.sample_size(10); // Test different transaction volumes to measure TPS
    for tx_count in [10, 25, 50].iter() {
        group.bench_with_input(
            BenchmarkId::new("tps", tx_count),
            tx_count,
            |b, &tx_count| {
                b.iter_custom(|iters| {
                    let start = std::time::Instant::now();
                    let mut total_transactions = 0i32;

                    for _ in 0..iters {
                        // Create first transaction as coinbase (block reward)
                        let mut transactions = vec![Transaction::new_coinbase(
                            "block_reward_address".to_string(),
                            "Block reward".to_string(),
                        ).expect("Failed to create coinbase transaction")];                        // Add regular transactions
                        for i in 1..tx_count {
                            let tx = create_simple_transaction(
                                format!("addr_{i}"),
                                format!("dest_{i}"),
                                10 + i,
                                total_transactions + i,
                            );
                            transactions.push(tx);
                        }

                        total_transactions += transactions.len() as i32;

                        // Process transactions in batches (simulating real blockchain behavior)
                        let config = DifficultyAdjustmentConfig {
                            base_difficulty: 1, // Low difficulty for speed
                            min_difficulty: 1,
                            max_difficulty: 2,
                            adjustment_factor: 0.1,
                            tolerance_percentage: 30.0,
                        };

                        let block = Block::<block_states::Building, network::Development>::new_building_with_config(
                            transactions,
                            format!("tps_prev_{total_transactions}"),
                            1,
                            1, // Minimal difficulty for maximum TPS
                            config,
                            MiningStats::default(),
                        );

                        // Mine and validate the block
                        let mined = black_box(block.mine()).expect("Mining failed");
                        let _validated = black_box(mined.validate()).expect("Validation failed");
                    }

                    start.elapsed()
                });
            },
        );
    }

    group.finish();
}

/// Benchmark transaction processing without mining (pure TPS)
fn benchmark_pure_transaction_processing(c: &mut Criterion) {
    let mut group = c.benchmark_group("pure_transaction_tps");
    group.measurement_time(Duration::from_secs(15));
    group.sample_size(10);

    for tx_count in [50, 100, 500].iter() {
        group.bench_with_input(
            BenchmarkId::new("pure_tps", tx_count),
            tx_count,
            |b, &tx_count| {
                b.iter_custom(|iters| {
                    let start = std::time::Instant::now();
                    for _ in 0..iters {
                        // Create first transaction as coinbase
                        let mut transactions = vec![create_test_transaction()]; // Create regular transactions
                        for i in 1..tx_count {
                            let tx = create_simple_transaction(
                                format!("pure_addr_{i}"),
                                format!("pure_dest_{i}"),
                                10 + i,
                                i,
                            );
                            transactions.push(tx);
                        }

                        // Just measure transaction creation and basic validation
                        for tx in transactions {
                            black_box(tx.is_coinbase());
                            black_box(&tx.id);
                        }
                    }

                    start.elapsed()
                });
            },
        );
    }

    group.finish();
}

/// Benchmark concurrent transaction processing
fn benchmark_concurrent_tps(c: &mut Criterion) {
    use std::thread;

    let mut group = c.benchmark_group("concurrent_tps");
    group.measurement_time(Duration::from_secs(20));
    group.sample_size(10);

    for thread_count in [2, 4].iter() {
        group.bench_with_input(
            BenchmarkId::new("concurrent", thread_count),
            thread_count,
            |b, &thread_count| {
                b.iter_custom(|iters| {
                    let start = std::time::Instant::now();

                    for _ in 0..iters {
                        let handles: Vec<thread::JoinHandle<()>> = (0..thread_count)
                            .map(|thread_id| {
                                thread::spawn(move || {
                                    // Each thread processes transactions
                                    // First create a coinbase transaction
                                    let mut transactions = vec![Transaction::new_coinbase(
                                        format!("concurrent_address_{thread_id}"),
                                        format!("concurrent_reward_{thread_id}"),
                                    )
                                    .expect("Failed to create coinbase transaction")];
                                    // Add regular transactions
                                    for i in 1..50 {
                                        let tx = create_simple_transaction(
                                            format!("concurrent_addr_{thread_id}_{i}"),
                                            format!("concurrent_dest_{thread_id}_{i}"),
                                            10 + i,
                                            thread_id * 1000 + i,
                                        );
                                        transactions.push(tx);
                                    }

                                    // Simulate processing
                                    for tx in transactions {
                                        black_box(tx.hash().unwrap());
                                    }
                                })
                            })
                            .collect();

                        // Wait for all threads to complete
                        for handle in handles {
                            handle.join().unwrap();
                        }
                    }

                    start.elapsed()
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
    benchmark_multiple_transactions,
    benchmark_tps,
    benchmark_pure_transaction_processing,
    benchmark_concurrent_tps
);

criterion_main!(benches);
