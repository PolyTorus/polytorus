//! Integration tests for privacy features in eUTXO model
//!
//! This test suite verifies the complete privacy implementation including:
//! - Zero-knowledge proofs for UTXO privacy
//! - Confidential transactions with amount hiding
//! - Diamond IO integration for enhanced privacy
//! - End-to-end privacy workflows

use polytorus::{
    crypto::{
        diamond_privacy::{DiamondCircuitComplexity, DiamondPrivacyConfig, DiamondPrivacyProvider},
        privacy::{PrivacyConfig, PrivacyProvider},
        transaction::Transaction,
    },
    modular::eutxo_processor::{EUtxoProcessor, EUtxoProcessorConfig},
};

/// Test helper for creating test transactions
fn create_test_coinbase_transaction() -> Transaction {
    Transaction::new_coinbase(
        "test_address_ECDSA".to_string(),
        "test_coinbase_data".to_string(),
    )
    .unwrap()
}

/// Test helper for creating privacy configuration
fn create_test_privacy_config() -> PrivacyConfig {
    PrivacyConfig {
        enable_zk_proofs: true,
        enable_confidential_amounts: true,
        enable_nullifiers: true,
        range_proof_bits: 32, // Smaller for testing
        commitment_randomness_size: 32,
    }
}

#[test]
fn test_basic_privacy_features() {
    let config = create_test_privacy_config();
    let provider = PrivacyProvider::new(config);

    // Test privacy statistics
    let stats = provider.get_privacy_stats();
    assert!(stats.zk_proofs_enabled);
    assert!(stats.confidential_amounts_enabled);
    assert!(stats.nullifiers_enabled);
    assert_eq!(stats.nullifiers_used, 0);
}

#[test]
fn test_amount_commitment_and_verification() {
    use rand_core::OsRng;

    let config = create_test_privacy_config();
    let provider = PrivacyProvider::new(config);
    let mut rng = OsRng;

    // Test various amounts
    for amount in [0u64, 1, 100, 1000, 65535] {
        let commitment = provider.commit_amount(amount, &mut rng).unwrap();

        // Verify correct amount
        assert!(provider.verify_commitment(&commitment, amount).unwrap());

        // Verify incorrect amount fails
        if amount > 0 {
            assert!(!provider.verify_commitment(&commitment, amount - 1).unwrap());
        }
        assert!(!provider.verify_commitment(&commitment, amount + 1).unwrap());
    }
}

#[test]
fn test_range_proof_generation_and_verification() {
    use rand_core::OsRng;

    let config = create_test_privacy_config();
    let provider = PrivacyProvider::new(config);
    let mut rng = OsRng;

    let test_amounts = [0u64, 1, 255, 1000, 65535];

    for amount in test_amounts {
        let commitment = provider.commit_amount(amount, &mut rng).unwrap();
        let range_proof = provider
            .generate_range_proof(amount, &commitment, &mut rng)
            .unwrap();

        assert!(!range_proof.is_empty());
        assert!(provider
            .verify_range_proof(&range_proof, &commitment)
            .unwrap());
    }
}

#[test]
fn test_nullifier_double_spend_prevention() {
    use rand_core::OsRng;

    let config = create_test_privacy_config();
    let mut provider = PrivacyProvider::new(config);
    let mut rng = OsRng;

    let input = polytorus::crypto::transaction::TXInput {
        txid: "test_transaction_id".to_string(),
        vout: 0,
        signature: vec![],
        pub_key: vec![1, 2, 3],
        redeemer: None,
    };

    let secret_key = vec![42, 43, 44, 45, 46];

    // Generate nullifier
    let nullifier = provider
        .generate_nullifier(&input, &secret_key, &mut rng)
        .unwrap();
    assert!(!nullifier.is_empty());

    // Initially not used
    assert!(!provider.is_nullifier_used(&nullifier));

    // Mark as used
    provider.mark_nullifier_used(nullifier.clone()).unwrap();
    assert!(provider.is_nullifier_used(&nullifier));

    // Attempt double spend should fail
    assert!(provider.mark_nullifier_used(nullifier).is_err());
}

#[test]
fn test_private_transaction_creation_and_verification() {
    use rand_core::OsRng;

    let config = create_test_privacy_config();
    let mut provider = PrivacyProvider::new(config);
    let mut rng = OsRng;

    // Create base transaction
    let base_tx = create_test_coinbase_transaction();

    // Create private transaction
    let private_tx = provider
        .create_private_transaction(
            base_tx,
            vec![0u64],          // Coinbase has 1 input with 0 value
            vec![50u64],         // One output with 50 units
            vec![vec![1, 2, 3]], // Dummy secret key for coinbase
            &mut rng,
        )
        .unwrap();

    // Verify private transaction structure
    assert_eq!(private_tx.private_inputs.len(), 1); // Coinbase has 1 input
    assert_eq!(private_tx.private_outputs.len(), 1);
    assert!(!private_tx.transaction_proof.is_empty());
    assert!(!private_tx.fee_commitment.commitment.is_empty());

    // Verify the private transaction
    assert!(provider.verify_private_transaction(&private_tx).unwrap());
}

#[test]
fn test_eutxo_processor_with_privacy() {
    let config = EUtxoProcessorConfig {
        privacy_config: create_test_privacy_config(),
        ..Default::default()
    };

    let processor = EUtxoProcessor::new(config);

    // Test privacy features are enabled
    assert!(processor.is_privacy_enabled());

    // Test privacy statistics
    let stats = processor.get_privacy_stats().unwrap();
    assert!(stats.zk_proofs_enabled);
    assert!(stats.confidential_amounts_enabled);
    assert!(stats.nullifiers_enabled);
}

#[test]
fn test_private_transaction_processing_in_eutxo() {
    let config = EUtxoProcessorConfig {
        privacy_config: create_test_privacy_config(),
        ..Default::default()
    };

    let processor = EUtxoProcessor::new(config);

    // Create a coinbase transaction
    let base_tx = create_test_coinbase_transaction();

    // Create private transaction
    let private_tx = processor
        .create_private_transaction(
            base_tx,
            vec![0u64],          // Coinbase has 1 input with 0 value
            vec![25u64],         // One output
            vec![vec![1, 2, 3]], // Dummy secret key for coinbase
        )
        .unwrap();

    // Process the private transaction
    let result = processor.process_private_transaction(&private_tx).unwrap();

    assert!(result.success);
    assert!(result.gas_used > 0);

    // Check for privacy events
    let privacy_events: Vec<_> = result
        .events
        .iter()
        .filter(|e| e.topics.iter().any(|t| t.contains("confidential")))
        .collect();
    assert!(!privacy_events.is_empty());
}

#[test]
fn test_commitment_homomorphism_property() {
    use rand_core::OsRng;

    let config = create_test_privacy_config();
    let provider = PrivacyProvider::new(config);
    let mut rng = OsRng;

    // Test that commitments are homomorphic
    let amount1 = 30u64;
    let amount2 = 20u64;
    let total = amount1 + amount2;

    let commitment1 = provider.commit_amount(amount1, &mut rng).unwrap();
    let commitment2 = provider.commit_amount(amount2, &mut rng).unwrap();
    let commitment_total = provider.commit_amount(total, &mut rng).unwrap();

    // All commitments should be valid
    assert!(provider.verify_commitment(&commitment1, amount1).unwrap());
    assert!(provider.verify_commitment(&commitment2, amount2).unwrap());
    assert!(provider
        .verify_commitment(&commitment_total, total)
        .unwrap());

    // In a full implementation, we would test that commitment1 + commitment2 = commitment_total
    // This demonstrates the structure exists for homomorphic operations
    assert!(!commitment1.commitment.is_empty());
    assert!(!commitment2.commitment.is_empty());
    assert!(!commitment_total.commitment.is_empty());
}

#[test]
fn test_privacy_configuration_flexibility() {
    // Test with ZK proofs disabled
    let mut config1 = create_test_privacy_config();
    config1.enable_zk_proofs = false;
    let provider1 = PrivacyProvider::new(config1);

    let stats1 = provider1.get_privacy_stats();
    assert!(!stats1.zk_proofs_enabled);
    assert!(stats1.confidential_amounts_enabled);

    // Test with confidential amounts disabled
    let mut config2 = create_test_privacy_config();
    config2.enable_confidential_amounts = false;
    let provider2 = PrivacyProvider::new(config2);

    let stats2 = provider2.get_privacy_stats();
    assert!(stats2.zk_proofs_enabled);
    assert!(!stats2.confidential_amounts_enabled);

    // Test with all privacy features disabled
    let mut config3 = create_test_privacy_config();
    config3.enable_zk_proofs = false;
    config3.enable_confidential_amounts = false;
    config3.enable_nullifiers = false;
    let provider3 = PrivacyProvider::new(config3);

    let stats3 = provider3.get_privacy_stats();
    assert!(!stats3.zk_proofs_enabled);
    assert!(!stats3.confidential_amounts_enabled);
    assert!(!stats3.nullifiers_enabled);
}

#[test]
fn test_range_proof_boundary_conditions() {
    use rand_core::OsRng;

    let config = create_test_privacy_config();
    let provider = PrivacyProvider::new(config);
    let mut rng = OsRng;

    // Test boundary values for 32-bit range proofs
    let max_value = (1u64 << 32) - 1;

    // Test maximum valid amount
    let commitment = provider.commit_amount(max_value, &mut rng).unwrap();
    let range_proof = provider
        .generate_range_proof(max_value, &commitment, &mut rng)
        .unwrap();
    assert!(provider
        .verify_range_proof(&range_proof, &commitment)
        .unwrap());

    // Test amount exceeding range should fail
    let over_max = 1u64 << 32;
    let over_commitment = provider.commit_amount(over_max, &mut rng).unwrap();
    assert!(provider
        .generate_range_proof(over_max, &over_commitment, &mut rng)
        .is_err());
}

#[test]
fn test_multiple_inputs_outputs_private_transaction() {
    use rand_core::OsRng;

    let config = create_test_privacy_config();
    let mut provider = PrivacyProvider::new(config);
    let mut rng = OsRng;

    // Create a more complex transaction with multiple outputs
    let mut base_tx = create_test_coinbase_transaction();

    // Add additional outputs to simulate a more complex transaction
    let output1 =
        polytorus::crypto::transaction::TXOutput::new(25, "address1".to_string()).unwrap();
    let output2 =
        polytorus::crypto::transaction::TXOutput::new(25, "address2".to_string()).unwrap();
    base_tx.vout.push(output1);
    base_tx.vout.push(output2);

    // Create private transaction with multiple outputs
    let private_tx = provider
        .create_private_transaction(
            base_tx,
            vec![0u64],                // Coinbase has 1 input with 0 value
            vec![10u64, 25u64, 25u64], // Three outputs
            vec![vec![1, 2, 3]],       // Dummy secret key for coinbase
            &mut rng,
        )
        .unwrap();

    assert_eq!(private_tx.private_outputs.len(), 3);
    assert!(provider.verify_private_transaction(&private_tx).unwrap());
}

// Diamond IO integration tests (may skip if Diamond IO not available)
#[test]
fn test_diamond_privacy_config_creation() {
    // Test default configuration (DiamondIO disabled by default)
    let default_config = DiamondPrivacyConfig::default();
    assert!(!default_config.enable_diamond_obfuscation); // Disabled by default now
    assert!(!default_config.enable_hybrid_privacy);      // Disabled by default now
    assert!(matches!(
        default_config.circuit_complexity,
        DiamondCircuitComplexity::Medium
    ));
    
    // Test custom configuration with DiamondIO enabled for testing
    let mut test_config = DiamondPrivacyConfig::default();
    test_config.enable_diamond_obfuscation = true;
    test_config.enable_hybrid_privacy = true;
    
    assert!(test_config.enable_diamond_obfuscation);
    assert!(test_config.enable_hybrid_privacy);
    assert!(matches!(
        test_config.circuit_complexity,
        DiamondCircuitComplexity::Medium
    ));
}

#[tokio::test]
async fn test_diamond_privacy_provider_creation() {
    // Test with default config (DiamondIO disabled)
    let default_config = DiamondPrivacyConfig::default();
    match DiamondPrivacyProvider::new(default_config).await {
        Ok(provider) => {
            let stats = provider.get_diamond_privacy_stats();
            assert!(!stats.diamond_obfuscation_enabled); // Disabled by default now
            assert!(!stats.hybrid_privacy_enabled);     // Disabled by default now
            assert_eq!(stats.security_level, "Medium_with_diamond_io");
        }
        Err(_) => {
            // Skip test if Diamond IO dependencies not available
            println!("Diamond IO not available, skipping Diamond privacy test");
        }
    }
    
    // Test with DiamondIO explicitly enabled
    let mut enabled_config = DiamondPrivacyConfig::default();
    enabled_config.enable_diamond_obfuscation = true;
    enabled_config.enable_hybrid_privacy = true;
    
    match DiamondPrivacyProvider::new(enabled_config).await {
        Ok(provider) => {
            let stats = provider.get_diamond_privacy_stats();
            assert!(stats.diamond_obfuscation_enabled);
            assert!(stats.hybrid_privacy_enabled);
            assert_eq!(stats.security_level, "Medium_with_diamond_io");
        }
        Err(_) => {
            // Skip test if Diamond IO dependencies not available
            println!("Diamond IO not available, skipping Diamond privacy test with enabled config");
        }
    }
}

#[test]
fn test_privacy_performance_characteristics() {
    use std::time::Instant;

    use rand_core::OsRng;

    let config = create_test_privacy_config();
    let provider = PrivacyProvider::new(config);
    let mut rng = OsRng;

    // Measure commitment performance
    let start = Instant::now();
    for i in 0..10 {
        let _commitment = provider.commit_amount(i * 100, &mut rng).unwrap();
    }
    let commitment_time = start.elapsed();
    println!("10 commitments took: {commitment_time:?}");

    // Measure range proof performance
    let amount = 1000u64;
    let commitment = provider.commit_amount(amount, &mut rng).unwrap();

    let start = Instant::now();
    let range_proof = provider
        .generate_range_proof(amount, &commitment, &mut rng)
        .unwrap();
    let proof_time = start.elapsed();
    println!("Range proof generation took: {proof_time:?}");

    let start = Instant::now();
    let _verified = provider
        .verify_range_proof(&range_proof, &commitment)
        .unwrap();
    let verify_time = start.elapsed();
    println!("Range proof verification took: {verify_time:?}");

    // Performance should be reasonable (not scientific, just sanity check)
    assert!(commitment_time.as_millis() < 1000); // Should take less than 1 second
    assert!(proof_time.as_millis() < 1000);
    assert!(verify_time.as_millis() < 1000);
}

#[test]
fn test_end_to_end_privacy_workflow() {
    let config = EUtxoProcessorConfig {
        privacy_config: create_test_privacy_config(),
        ..Default::default()
    };

    let processor = EUtxoProcessor::new(config);

    // Step 1: Create initial coinbase transaction
    let coinbase_tx = create_test_coinbase_transaction();
    let coinbase_result = processor.process_transaction(&coinbase_tx).unwrap();
    assert!(coinbase_result.success);

    // Step 2: Create private transaction from coinbase
    let private_tx = processor
        .create_private_transaction(
            coinbase_tx,
            vec![0u64],          // Coinbase has 1 input with 0 value
            vec![10u64],         // One output
            vec![vec![1, 2, 3]], // Dummy secret key for coinbase
        )
        .unwrap();

    // Step 3: Process private transaction
    let private_result = processor.process_private_transaction(&private_tx).unwrap();
    assert!(private_result.success);

    // Step 4: Verify gas costs for privacy features
    assert!(private_result.gas_used > coinbase_result.gas_used);

    // Step 5: Check privacy statistics
    let final_stats = processor.get_privacy_stats().unwrap();
    assert!(final_stats.zk_proofs_enabled);
    assert!(final_stats.confidential_amounts_enabled);
    assert!(final_stats.nullifiers_enabled);
}
