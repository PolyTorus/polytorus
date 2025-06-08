use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};
use polytorus::blockchain::block::{Block, DifficultyAdjustmentConfig, MiningStats};
use polytorus::blockchain::types::{block_states, network};
use polytorus::crypto::transaction::{Transaction, TXInput, TXOutput};
use std::time::Duration;

/// Create a test transaction for benchmarking (coinbase)
fn create_test_transaction() -> Transaction {
    Transaction::new_coinbase(
        "benchmark_address".to_string(),
        "benchmark_reward".to_string(),
    )
    .expect("Failed to create test transaction")
}

/// Create a simple test transaction (non-coinbase)
fn create_simple_transaction(from: String, to: String, amount: i32, nonce: i32) -> Transaction {
    // Create a fake input referencing a previous transaction
    let prev_tx_id = format!("prev_tx_{}", nonce);
    let input = TXInput {
        txid: prev_tx_id,
        vout: 0,
        signature: Vec::new(),
        pub_key: format!("pubkey_{}", from).into_bytes(),
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

/// Quick TPS test - measures transaction processing speed
fn quick_tps_test(c: &mut Criterion) {
    let mut group = c.benchmark_group("quick_tps");
    group.measurement_time(Duration::from_secs(5));
    group.sample_size(10);
    
    for tx_count in [5, 10, 20].iter() {
        group.bench_with_input(
            BenchmarkId::new("transactions", tx_count),
            tx_count,
            |b, &tx_count| {
                b.iter(|| {
                    // Create first transaction as coinbase (block reward)
                    let mut transactions = vec![create_test_transaction()];
                    
                    // Add regular transactions
                    for i in 1..tx_count {
                        let tx = create_simple_transaction(
                            format!("quick_addr_{}", i),
                            format!("quick_dest_{}", i),
                            10 + i,
                            i,
                        );
                        transactions.push(tx);
                    }
                    
                    // Basic transaction processing
                    let mut processed = 0;
                    for tx in transactions {
                        if tx.is_coinbase() || !tx.vin.is_empty() {
                            processed += 1;
                        }
                        black_box(&tx.id);
                    }
                    black_box(processed);
                });
            },
        );
    }
    group.finish();
}/// Transaction creation speed test
fn transaction_creation_speed(c: &mut Criterion) {
    let mut group = c.benchmark_group("tx_creation_speed");
    group.measurement_time(Duration::from_secs(3));
    group.sample_size(10);
    
    group.bench_function("single_tx", |b| {
        b.iter(|| {
            black_box(create_test_transaction());
        });
    });
    
    group.bench_function("batch_10_tx", |b| {
        b.iter(|| {
            // Create first transaction as coinbase
            let mut transactions = vec![create_test_transaction()];
            
            // Add regular transactions
            for i in 1..10 {
                let tx = create_simple_transaction(
                    format!("batch_addr_{}", i),
                    format!("batch_dest_{}", i),
                    10 + i,
                    i,
                );
                transactions.push(tx);
            }
            black_box(transactions);
        });
    });
    
    group.finish();
}/// Block processing without mining
fn block_processing_no_mining(c: &mut Criterion) {
    let mut group = c.benchmark_group("block_processing");
    group.measurement_time(Duration::from_secs(5));
    group.sample_size(10);
    
    group.bench_function("create_building_block", |b| {
        b.iter(|| {
            let transactions = vec![create_test_transaction()];
            let config = DifficultyAdjustmentConfig {
                base_difficulty: 1,
                min_difficulty: 1,
                max_difficulty: 2,
                adjustment_factor: 0.1,
                tolerance_percentage: 30.0,
            };
            
            let block = Block::<block_states::Building, network::Development>::new_building_with_config(
                transactions,
                "test_prev_hash".to_string(),
                1,
                1,
                config,
                MiningStats::default(),
            );
            
            black_box(block);
        });
    });
    
    group.finish();
}criterion_group!(    quick_benches,    quick_tps_test,    transaction_creation_speed,    block_processing_no_mining);criterion_main!(quick_benches);