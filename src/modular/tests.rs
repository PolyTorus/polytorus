//! Tests for the modular blockchain architecture

use super::*;
use crate::config::DataContext;
use crate::crypto::transaction::Transaction;
use crate::blockchain::block::Block;

use std::path::PathBuf;

/// Create a test data context
fn create_test_context(test_name: &str) -> DataContext {
    let test_dir = PathBuf::from(format!("test_data_modular_{}", test_name));
    DataContext::new(test_dir)
}

#[tokio::test]
async fn test_modular_blockchain_creation() {
    let config = default_modular_config();
    let data_context = create_test_context("creation");

    let blockchain = ModularBlockchainBuilder::new()
        .with_config(config)
        .with_data_context(data_context)
        .build();

    assert!(blockchain.is_ok());
}

#[tokio::test]
async fn test_execution_layer() {
    let config = ExecutionConfig {
        gas_limit: 1_000_000,
        gas_price: 1,
        wasm_config: WasmConfig {
            max_memory_pages: 256,
            max_stack_size: 65536,
            gas_metering: true,
        },
    };

    let data_context = create_test_context("execution");
    let execution_layer = PolyTorusExecutionLayer::new(data_context, config);

    assert!(execution_layer.is_ok());

    let execution_layer = execution_layer.unwrap();
    let state_root = execution_layer.get_state_root();
    assert!(!state_root.is_empty());
}

#[test]
fn test_consensus_layer() {
    let config = ConsensusConfig {
        block_time: 10000,
        difficulty: 1, // Easy difficulty for testing
        max_block_size: 1024 * 1024,
    };

    let data_context = create_test_context("consensus");
    let consensus_layer = PolyTorusConsensusLayer::new(data_context, config, false);

    assert!(consensus_layer.is_ok());

    let consensus_layer = consensus_layer.unwrap();
    assert!(!consensus_layer.is_validator());
}

#[test]
fn test_settlement_layer() {
    let config = SettlementConfig {
        challenge_period: 10,
        batch_size: 10,
        min_validator_stake: 100,
    };

    let settlement_layer = PolyTorusSettlementLayer::new(config);

    assert!(settlement_layer.is_ok());

    let settlement_layer = settlement_layer.unwrap();
    let settlement_root = settlement_layer.get_settlement_root();
    assert!(!settlement_root.is_empty());
}

#[test]
fn test_data_availability_layer() {
    let config = DataAvailabilityConfig {
        network_config: NetworkConfig {
            listen_addr: "127.0.0.1:0".to_string(),
            bootstrap_peers: Vec::new(),
            max_peers: 10,
        },
        retention_period: 3600, // 1 hour for testing
        max_data_size: 1024,   // 1KB for testing
    };

    let da_layer = PolyTorusDataAvailabilityLayer::new(config);

    assert!(da_layer.is_ok());

    let da_layer = da_layer.unwrap();
    
    // Test data storage and retrieval
    let test_data = b"test data for storage";
    let hash = da_layer.store_data(test_data).unwrap();
    
    let retrieved_data = da_layer.retrieve_data(&hash).unwrap();
    assert_eq!(test_data, retrieved_data.as_slice());
    
    assert!(da_layer.verify_availability(&hash));
}

#[test]
fn test_batch_settlement() {
    let config = SettlementConfig {
        challenge_period: 5,
        batch_size: 5,
        min_validator_stake: 100,
    };

    let settlement_layer = PolyTorusSettlementLayer::new(config).unwrap();

    // Create a test execution batch
    let batch = ExecutionBatch {
        batch_id: "test_batch_1".to_string(),
        transactions: Vec::new(),
        results: Vec::new(),
        prev_state_root: "prev_root".to_string(),
        new_state_root: "new_root".to_string(),
    };

    let result = settlement_layer.settle_batch(&batch);
    assert!(result.is_ok());

    let settlement_result = result.unwrap();
    assert_eq!(settlement_result.settled_batches, vec!["test_batch_1".to_string()]);
}

#[test]
fn test_fraud_proof_verification() {
    let config = SettlementConfig {
        challenge_period: 5,
        batch_size: 5,
        min_validator_stake: 100,
    };

    let settlement_layer = PolyTorusSettlementLayer::new(config).unwrap();

    // Create a valid fraud proof
    let fraud_proof = FraudProof {
        batch_id: "fraudulent_batch".to_string(),
        proof_data: b"fraud proof data".to_vec(),
        expected_state_root: "expected_root".to_string(),
        actual_state_root: "different_root".to_string(),
    };

    assert!(settlement_layer.verify_fraud_proof(&fraud_proof));

    // Create an invalid fraud proof (same roots)
    let invalid_fraud_proof = FraudProof {
        batch_id: "batch".to_string(),
        proof_data: b"proof".to_vec(),
        expected_state_root: "same_root".to_string(),
        actual_state_root: "same_root".to_string(),
    };

    assert!(!settlement_layer.verify_fraud_proof(&invalid_fraud_proof));
}

#[tokio::test]
async fn test_transaction_processing() {
    let config = default_modular_config();
    let data_context = create_test_context("transaction");

    let blockchain = ModularBlockchainBuilder::new()
        .with_config(config)
        .with_data_context(data_context)
        .build()
        .unwrap();

    // Create a test transaction
    let tx = Transaction::new_coinbase("test_address".to_string(), "test_reward".to_string())
        .unwrap();

    let receipt = blockchain.process_transaction(tx).await;
    assert!(receipt.is_ok());

    let receipt = receipt.unwrap();
    assert!(receipt.success);
    assert!(receipt.gas_used > 0);
}

#[tokio::test]
async fn test_block_mining() {
    let config = default_modular_config();
    let data_context = create_test_context("mining");

    let blockchain = ModularBlockchainBuilder::new()
        .with_config(config)
        .with_data_context(data_context)
        .build()
        .unwrap();

    // Create test transactions
    let tx1 = Transaction::new_coinbase("addr1".to_string(), "reward1".to_string()).unwrap();
    let tx2 = Transaction::new_coinbase("addr2".to_string(), "reward2".to_string()).unwrap();
    let transactions = vec![tx1, tx2];

    let block = blockchain.mine_block(transactions).await;
    assert!(block.is_ok());

    let block = block.unwrap();
    assert_eq!(block.get_transaction().len(), 2);
    assert!(!block.get_hash().is_empty());
}

#[test]
fn test_layer_builders() {
    // Test consensus layer builder
    let consensus_layer = super::consensus::ConsensusLayerBuilder::new()
        .with_data_context(create_test_context("builder_consensus"))
        .as_validator()
        .build();
    
    assert!(consensus_layer.is_ok());
    assert!(consensus_layer.unwrap().is_validator());

    // Test settlement layer builder
    let settlement_layer = super::settlement::SettlementLayerBuilder::new()
        .with_challenge_period(50)
        .build();
    
    assert!(settlement_layer.is_ok());

    // Test data availability layer builder
    let da_layer = super::data_availability::DataAvailabilityLayerBuilder::new()
        .with_network_config(NetworkConfig {
            listen_addr: "127.0.0.1:0".to_string(),
            bootstrap_peers: vec!["127.0.0.1:7001".to_string()],
            max_peers: 20,
        })
        .build();
    
    assert!(da_layer.is_ok());
}

#[tokio::test]
async fn test_state_info() {
    let config = default_modular_config();
    let data_context = create_test_context("state_info");

    let blockchain = ModularBlockchainBuilder::new()
        .with_config(config)
        .with_data_context(data_context)
        .build()
        .unwrap();

    let state_info = blockchain.get_state_info();
    assert!(state_info.is_ok());

    let state_info = state_info.unwrap();
    assert!(!state_info.execution_state_root.is_empty());
    assert!(!state_info.settlement_root.is_empty());
    assert_eq!(state_info.block_height, 1); // Genesis block height
}

// Cleanup test data after tests
#[tokio::test]
async fn cleanup_test_data() {
    let test_dirs = [
        "test_data_modular_creation",
        "test_data_modular_execution", 
        "test_data_modular_consensus",
        "test_data_modular_transaction",
        "test_data_modular_mining",
        "test_data_modular_builder_consensus",
        "test_data_modular_state_info",
    ];

    for dir in &test_dirs {
        let _ = std::fs::remove_dir_all(dir);
    }
}
