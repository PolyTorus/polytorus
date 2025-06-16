//! Integration tests for real Diamond IO privacy features
//!
//! These tests verify the complete integration between PolyTorus privacy features
//! and the real Diamond IO library from MachinaIO.

use std::collections::HashMap;
use rand_core::OsRng;
use tokio;

use polytorus::crypto::privacy::{PrivacyConfig, UtxoValidityProof, PedersenCommitment};
use polytorus::crypto::real_diamond_io::{
    RealDiamondIOProvider, RealDiamondIOConfig, RealDiamondIOProof, DiamondIOResult
};
use polytorus::crypto::enhanced_privacy::{
    EnhancedPrivacyProvider, EnhancedPrivacyConfig
};
use polytorus::crypto::transaction::Transaction;

#[tokio::test]
async fn test_real_diamond_io_provider_lifecycle() {
    let config = RealDiamondIOConfig::testing();
    let privacy_config = PrivacyConfig::default();
    
    // Create provider
    let mut provider = RealDiamondIOProvider::new(config, privacy_config)
        .await
        .expect("Failed to create Diamond IO provider");

    // Check initial statistics
    let stats = provider.get_statistics();
    assert_eq!(stats.active_circuits, 0);
    assert_eq!(stats.ring_dimension, 4);
    assert_eq!(stats.crt_depth, 2);
    assert!(!stats.disk_storage_enabled);

    // Create a test validity proof
    let test_proof = UtxoValidityProof {
        commitment_proof: vec![1, 2, 3, 4, 5],
        range_proof: vec![6, 7, 8, 9, 10],
        nullifier: vec![11, 12, 13, 14, 15],
        params_hash: vec![16, 17, 18, 19, 20],
    };

    // Create circuit
    let circuit = provider
        .create_privacy_circuit("test_lifecycle_circuit".to_string(), &test_proof)
        .await
        .expect("Failed to create privacy circuit");

    // Verify circuit properties
    assert_eq!(circuit.circuit_id, "test_lifecycle_circuit");
    assert!(!circuit.obfuscated_data.is_empty());
    assert_eq!(circuit.metadata.ring_dim, 4);
    assert_eq!(circuit.metadata.crt_depth, 2);
    assert_eq!(circuit.metadata.complexity, "privacy_circuit");
    assert_eq!(circuit.metadata.matrix_files.len(), 2);

    // Check statistics after circuit creation
    let stats = provider.get_statistics();
    assert_eq!(stats.active_circuits, 1);

    // Test circuit evaluation
    let test_inputs = vec![true, false, true, false, true];
    let evaluation_result = provider
        .evaluate_circuit(&circuit, test_inputs.clone())
        .await
        .expect("Failed to evaluate circuit");

    // Verify evaluation result
    assert!(!evaluation_result.result.is_empty());
    assert!(evaluation_result.evaluation_time > 0);
    assert!(evaluation_result.metrics.contains_key("input_size"));
    assert!(evaluation_result.metrics.contains_key("output_size"));
    assert!(evaluation_result.metrics.contains_key("ring_dimension"));
    assert_eq!(evaluation_result.metrics["input_size"], test_inputs.len() as f64);

    // Test evaluation verification
    let verification_result = provider
        .verify_evaluation(&circuit, &test_inputs, &evaluation_result)
        .await
        .expect("Failed to verify evaluation");
    assert!(verification_result, "Circuit evaluation verification failed");

    // Test circuit cleanup
    provider
        .cleanup_circuit("test_lifecycle_circuit")
        .await
        .expect("Failed to cleanup circuit");

    let stats = provider.get_statistics();
    assert_eq!(stats.active_circuits, 0);
}

#[tokio::test]
async fn test_real_diamond_io_multiple_circuits() {
    let config = RealDiamondIOConfig::testing();
    let privacy_config = PrivacyConfig::default();
    
    let mut provider = RealDiamondIOProvider::new(config, privacy_config)
        .await
        .expect("Failed to create Diamond IO provider");

    let mut circuits = Vec::new();
    let num_circuits = 3;

    // Create multiple circuits
    for i in 0..num_circuits {
        let test_proof = UtxoValidityProof {
            commitment_proof: vec![i as u8; 5],
            range_proof: vec![(i + 1) as u8; 5],
            nullifier: vec![(i + 2) as u8; 5],
            params_hash: vec![(i + 3) as u8; 5],
        };

        let circuit_id = format!("multi_circuit_{}", i);
        let circuit = provider
            .create_privacy_circuit(circuit_id.clone(), &test_proof)
            .await
            .expect(&format!("Failed to create circuit {}", i));

        circuits.push(circuit);
    }

    // Verify all circuits were created
    let stats = provider.get_statistics();
    assert_eq!(stats.active_circuits, num_circuits);

    // Test each circuit
    for (i, circuit) in circuits.iter().enumerate() {
        let test_inputs = vec![i % 2 == 0; i + 1]; // Different input patterns
        
        let evaluation_result = provider
            .evaluate_circuit(circuit, test_inputs.clone())
            .await
            .expect(&format!("Failed to evaluate circuit {}", i));

        assert!(!evaluation_result.result.is_empty());
        
        let verification_result = provider
            .verify_evaluation(circuit, &test_inputs, &evaluation_result)
            .await
            .expect(&format!("Failed to verify circuit {}", i));
        assert!(verification_result);
    }

    // Cleanup all circuits
    for i in 0..num_circuits {
        let circuit_id = format!("multi_circuit_{}", i);
        provider
            .cleanup_circuit(&circuit_id)
            .await
            .expect(&format!("Failed to cleanup circuit {}", i));
    }

    let stats = provider.get_statistics();
    assert_eq!(stats.active_circuits, 0);
}

#[tokio::test]
async fn test_enhanced_privacy_provider_complete_flow() {
    let config = EnhancedPrivacyConfig::testing();
    let mut provider = EnhancedPrivacyProvider::new(config)
        .await
        .expect("Failed to create enhanced privacy provider");

    // Create a test transaction
    let base_tx = Transaction::new_coinbase("test_address".to_string(), "test_data".to_string())
        .expect("Failed to create base transaction");
    
    let mut rng = OsRng;
    
    // Create enhanced private transaction
    let enhanced_tx = provider
        .create_enhanced_private_transaction(
            base_tx,
            vec![0u64],              // Coinbase input amount
            vec![100u64],            // Output amount
            vec![vec![1, 2, 3, 4]],  // Secret key
            &mut rng,
        )
        .await
        .expect("Failed to create enhanced private transaction");

    // Verify transaction structure
    assert_eq!(enhanced_tx.base_private_transaction.private_inputs.len(), 1);
    assert_eq!(enhanced_tx.base_private_transaction.private_outputs.len(), 1);
    assert_eq!(enhanced_tx.diamond_io_proofs.len(), 1);
    assert_eq!(enhanced_tx.circuit_ids.len(), 1);

    // Verify enhanced metadata
    let metadata = &enhanced_tx.enhanced_metadata;
    assert!(metadata.created_at > 0);
    assert_eq!(metadata.privacy_level, "maximum_privacy");
    assert!(metadata.total_gas_cost > 0);
    assert!(metadata.diamond_io_stats.contains_key("active_circuits"));
    assert!(metadata.diamond_io_stats.contains_key("hybrid_mode"));

    // Verify Diamond IO proof
    let diamond_proof = &enhanced_tx.diamond_io_proofs[0];
    assert!(!diamond_proof.circuit_id.is_empty());
    assert!(!diamond_proof.evaluation_result.result.is_empty());
    assert!(diamond_proof.performance_metrics.contains_key("ring_dimension"));
    assert!(diamond_proof.performance_metrics.contains_key("obfuscated_size"));

    // Verify the enhanced transaction
    let verification_result = provider
        .verify_enhanced_private_transaction(&enhanced_tx)
        .await
        .expect("Failed to verify enhanced transaction");
    assert!(verification_result, "Enhanced transaction verification failed");

    // Test enhanced statistics
    let stats = provider.get_enhanced_statistics();
    assert!(stats.real_diamond_io_enabled);
    assert!(stats.hybrid_mode_enabled);
    assert_eq!(stats.total_circuits_created, 1);
    assert!(stats.diamond_io_stats.contains_key("active_circuits"));
}

#[tokio::test]
async fn test_enhanced_privacy_with_multiple_inputs_outputs() {
    let config = EnhancedPrivacyConfig::testing();
    let mut provider = EnhancedPrivacyProvider::new(config)
        .await
        .expect("Failed to create enhanced privacy provider");

    // Create a transaction with multiple inputs and outputs
    let mut base_tx = Transaction::new_coinbase("test_address".to_string(), "test_data".to_string())
        .expect("Failed to create base transaction");
    
    // Add additional inputs and outputs to simulate a more complex transaction
    use polytorus::crypto::transaction::{TXInput, TXOutput};
    
    base_tx.vin.push(TXInput {
        txid: "prev_tx_1".to_string(),
        vout: 0,
        signature: vec![1, 2, 3],
        pub_key: vec![4, 5, 6],
        redeemer: None,
    });
    
    base_tx.vout.push(TXOutput {
        value: 50,
        pub_key_hash: vec![7, 8, 9],
        script: Some(vec![10, 11, 12]),
        datum: None,
        reference_script: Some("output_script_2".to_string()),
    });

    let mut rng = OsRng;
    
    // Create enhanced private transaction with multiple I/O
    let enhanced_tx = provider
        .create_enhanced_private_transaction(
            base_tx,
            vec![0u64, 75u64],       // Two inputs: coinbase + regular
            vec![100u64, 50u64],     // Two outputs
            vec![vec![1, 2, 3], vec![4, 5, 6]],  // Two secret keys
            &mut rng,
        )
        .await
        .expect("Failed to create enhanced private transaction with multiple I/O");

    // Verify transaction structure
    assert_eq!(enhanced_tx.base_private_transaction.private_inputs.len(), 2);
    assert_eq!(enhanced_tx.base_private_transaction.private_outputs.len(), 2);
    assert_eq!(enhanced_tx.diamond_io_proofs.len(), 2);  // One proof per input
    assert_eq!(enhanced_tx.circuit_ids.len(), 2);

    // Verify each Diamond IO proof has unique circuit ID
    let circuit_id_1 = &enhanced_tx.circuit_ids[0];
    let circuit_id_2 = &enhanced_tx.circuit_ids[1];
    assert_ne!(circuit_id_1, circuit_id_2);

    // Verify the enhanced transaction
    let verification_result = provider
        .verify_enhanced_private_transaction(&enhanced_tx)
        .await
        .expect("Failed to verify enhanced transaction with multiple I/O");
    assert!(verification_result, "Multi I/O enhanced transaction verification failed");

    // Test statistics after multiple circuits
    let stats = provider.get_enhanced_statistics();
    assert_eq!(stats.total_circuits_created, 2);
}

#[tokio::test]
async fn test_diamond_io_config_levels() {
    // Test different configuration levels
    let testing_config = RealDiamondIOConfig::testing();
    let production_config = RealDiamondIOConfig::production();

    // Verify testing config has smaller parameters
    assert!(testing_config.ring_dim <= production_config.ring_dim);
    assert!(testing_config.d <= production_config.d);
    assert!(testing_config.input_size <= production_config.input_size);
    assert!(!testing_config.enable_disk_storage);
    assert!(production_config.enable_disk_storage);

    // Test both configurations can create providers
    let privacy_config = PrivacyConfig::default();
    
    let testing_provider = RealDiamondIOProvider::new(testing_config.clone(), privacy_config.clone())
        .await
        .expect("Failed to create testing provider");
    
    let production_provider = RealDiamondIOProvider::new(production_config.clone(), privacy_config)
        .await
        .expect("Failed to create production provider");

    // Verify statistics reflect configuration differences
    let testing_stats = testing_provider.get_statistics();
    let production_stats = production_provider.get_statistics();
    
    assert_eq!(testing_stats.ring_dimension, testing_config.ring_dim);
    assert_eq!(production_stats.ring_dimension, production_config.ring_dim);
    assert_eq!(testing_stats.disk_storage_enabled, testing_config.enable_disk_storage);
    assert_eq!(production_stats.disk_storage_enabled, production_config.enable_disk_storage);
}

#[tokio::test]
async fn test_diamond_io_proof_serialization() {
    // Create test data
    let test_validity_proof = UtxoValidityProof {
        commitment_proof: vec![1, 2, 3, 4, 5],
        range_proof: vec![6, 7, 8, 9, 10],
        nullifier: vec![11, 12, 13, 14, 15],
        params_hash: vec![16, 17, 18, 19, 20],
    };

    let test_evaluation_result = DiamondIOResult {
        result: vec![true, false, true],
        evaluation_time: 1234567890,
        metrics: {
            let mut metrics = HashMap::new();
            metrics.insert("test_metric".to_string(), 42.0);
            metrics
        },
    };

    let test_commitment = PedersenCommitment {
        commitment: vec![21, 22, 23, 24, 25],
        blinding_factor: vec![26, 27, 28, 29, 30],
    };

    let test_performance_metrics = {
        let mut metrics = HashMap::new();
        metrics.insert("ring_dimension".to_string(), 4.0);
        metrics.insert("crt_depth".to_string(), 2.0);
        metrics
    };

    let diamond_proof = RealDiamondIOProof {
        base_proof: test_validity_proof,
        circuit_id: "test_serialization_circuit".to_string(),
        evaluation_result: test_evaluation_result,
        params_commitment: test_commitment,
        performance_metrics: test_performance_metrics,
    };

    // Test JSON serialization
    let json_serialized = serde_json::to_string(&diamond_proof)
        .expect("Failed to serialize Diamond IO proof to JSON");
    assert!(!json_serialized.is_empty());

    // Test JSON deserialization
    let json_deserialized: RealDiamondIOProof = serde_json::from_str(&json_serialized)
        .expect("Failed to deserialize Diamond IO proof from JSON");
    
    assert_eq!(json_deserialized.circuit_id, "test_serialization_circuit");
    assert_eq!(json_deserialized.evaluation_result.result, vec![true, false, true]);
    assert_eq!(json_deserialized.evaluation_result.evaluation_time, 1234567890);
    assert_eq!(json_deserialized.performance_metrics.get("ring_dimension"), Some(&4.0));

    // Test binary serialization (using bincode)
    let binary_serialized = bincode::serialize(&diamond_proof)
        .expect("Failed to serialize Diamond IO proof to binary");
    assert!(!binary_serialized.is_empty());

    // Test binary deserialization
    let binary_deserialized: RealDiamondIOProof = bincode::deserialize(&binary_serialized)
        .expect("Failed to deserialize Diamond IO proof from binary");
    
    assert_eq!(binary_deserialized.circuit_id, diamond_proof.circuit_id);
    assert_eq!(binary_deserialized.base_proof.nullifier, diamond_proof.base_proof.nullifier);
}

#[tokio::test]
async fn test_privacy_level_determination() {
    // Test different privacy configurations
    let configs = vec![
        (EnhancedPrivacyConfig {
            privacy_config: PrivacyConfig {
                enable_zk_proofs: false,
                enable_confidential_amounts: false,
                enable_nullifiers: false,
                range_proof_bits: 32,
                commitment_randomness_size: 32,
            },
            enable_real_diamond_io: false,
            use_hybrid_mode: false,
            ..EnhancedPrivacyConfig::testing()
        }, "basic"),
        
        (EnhancedPrivacyConfig {
            privacy_config: PrivacyConfig {
                enable_zk_proofs: false,
                enable_confidential_amounts: true,
                enable_nullifiers: false,
                range_proof_bits: 32,
                commitment_randomness_size: 32,
            },
            enable_real_diamond_io: false,
            use_hybrid_mode: false,
            ..EnhancedPrivacyConfig::testing()
        }, "confidential"),
        
        (EnhancedPrivacyConfig {
            privacy_config: PrivacyConfig {
                enable_zk_proofs: true,
                enable_confidential_amounts: true,
                enable_nullifiers: true,
                range_proof_bits: 32,
                commitment_randomness_size: 32,
            },
            enable_real_diamond_io: false,
            use_hybrid_mode: false,
            ..EnhancedPrivacyConfig::testing()
        }, "zero_knowledge"),
        
        (EnhancedPrivacyConfig {
            privacy_config: PrivacyConfig {
                enable_zk_proofs: true,
                enable_confidential_amounts: true,
                enable_nullifiers: true,
                range_proof_bits: 32,
                commitment_randomness_size: 32,
            },
            enable_real_diamond_io: true,
            use_hybrid_mode: false,
            ..EnhancedPrivacyConfig::testing()
        }, "indistinguishable_obfuscation"),
        
        (EnhancedPrivacyConfig::testing(), "maximum_privacy"),
    ];

    for (config, expected_level) in configs {
        let provider = EnhancedPrivacyProvider::new(config)
            .await
            .expect("Failed to create enhanced privacy provider");
        
        let determined_level = provider.determine_privacy_level();
        assert_eq!(determined_level, expected_level, 
                  "Privacy level mismatch for configuration");
    }
}

#[tokio::test]
async fn test_circuit_cleanup_and_memory_management() {
    let config = RealDiamondIOConfig::testing();
    let privacy_config = PrivacyConfig::default();
    
    let mut provider = RealDiamondIOProvider::new(config, privacy_config)
        .await
        .expect("Failed to create Diamond IO provider");

    let circuit_ids = vec!["cleanup_test_1", "cleanup_test_2", "cleanup_test_3"];
    
    // Create multiple circuits
    for circuit_id in &circuit_ids {
        let test_proof = UtxoValidityProof {
            commitment_proof: vec![1, 2, 3],
            range_proof: vec![4, 5, 6],
            nullifier: vec![7, 8, 9],
            params_hash: vec![10, 11, 12],
        };

        provider
            .create_privacy_circuit(circuit_id.to_string(), &test_proof)
            .await
            .expect(&format!("Failed to create circuit {}", circuit_id));
    }

    // Verify all circuits exist
    let stats = provider.get_statistics();
    assert_eq!(stats.active_circuits, circuit_ids.len());

    // Cleanup individual circuits
    for circuit_id in &circuit_ids {
        provider
            .cleanup_circuit(circuit_id)
            .await
            .expect(&format!("Failed to cleanup circuit {}", circuit_id));
    }

    // Verify all circuits cleaned up
    let stats = provider.get_statistics();
    assert_eq!(stats.active_circuits, 0);

    // Test cleanup of non-existent circuit (should not error)
    provider
        .cleanup_circuit("non_existent_circuit")
        .await
        .expect("Cleanup of non-existent circuit should not error");
}

#[tokio::test]
async fn test_enhanced_privacy_cleanup_integration() {
    let config = EnhancedPrivacyConfig::testing();
    let mut provider = EnhancedPrivacyProvider::new(config)
        .await
        .expect("Failed to create enhanced privacy provider");

    // Create enhanced private transaction to generate circuits
    let base_tx = Transaction::new_coinbase("test_address".to_string(), "test_data".to_string())
        .expect("Failed to create base transaction");
    
    let mut rng = OsRng;
    
    let _enhanced_tx = provider
        .create_enhanced_private_transaction(
            base_tx,
            vec![0u64],
            vec![100u64],
            vec![vec![1, 2, 3, 4]],
            &mut rng,
        )
        .await
        .expect("Failed to create enhanced private transaction");

    // Verify circuit was created
    let stats = provider.get_enhanced_statistics();
    assert_eq!(stats.total_circuits_created, 1);

    // Test cleanup functionality
    provider
        .cleanup_old_circuits()
        .await
        .expect("Failed to cleanup old circuits");

    // In a real implementation, this would verify actual cleanup occurred
    // For now, we just verify the function executes without error
}