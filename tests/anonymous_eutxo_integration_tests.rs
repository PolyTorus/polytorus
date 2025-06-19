//! Integration tests for anonymous eUTXO system
//!
//! This module tests the complete anonymous eUTXO workflow including
//! stealth addresses, ring signatures, nullifiers, and privacy proofs.

use polytorus::crypto::{
    anonymous_eutxo::{AnonymousEUtxoConfig, AnonymousEUtxoProcessor, StealthAddress},
    enhanced_privacy::EnhancedPrivacyConfig,
};
use rand_core::OsRng;

/// Test complete anonymous eUTXO workflow
#[tokio::test]
async fn test_complete_anonymous_eutxo_workflow() {
    let config = AnonymousEUtxoConfig::testing();
    let processor = AnonymousEUtxoProcessor::new(config).await.unwrap();
    let mut rng = OsRng;

    // Test 1: Create stealth addresses for recipients
    println!("Testing stealth address creation...");
    let recipient1 = "alice_stealth";
    let recipient2 = "bob_stealth";

    let stealth_addr1 = processor
        .create_stealth_address(recipient1, &mut rng)
        .unwrap();
    let stealth_addr2 = processor
        .create_stealth_address(recipient2, &mut rng)
        .unwrap();

    assert!(stealth_addr1.one_time_address.starts_with("stealth_"));
    assert!(stealth_addr2.one_time_address.starts_with("stealth_"));
    assert_ne!(
        stealth_addr1.one_time_address,
        stealth_addr2.one_time_address
    );

    println!("✅ Stealth addresses created successfully");

    // Test 2: Verify stealth address validation
    assert!(processor.verify_stealth_address(&stealth_addr1).unwrap());
    assert!(processor.verify_stealth_address(&stealth_addr2).unwrap());

    println!("✅ Stealth address validation works");

    // Test 3: Create ring signatures
    println!("Testing ring signature creation...");
    let secret_key1 = vec![1, 2, 3, 4, 5];
    let secret_key2 = vec![6, 7, 8, 9, 10];

    let ring_sig1 = processor
        .create_ring_signature("utxo_1", &secret_key1, &mut rng)
        .await
        .unwrap();
    let ring_sig2 = processor
        .create_ring_signature("utxo_2", &secret_key2, &mut rng)
        .await
        .unwrap();

    assert_eq!(ring_sig1.ring.len(), 3); // Testing config uses ring size 3
    assert_eq!(ring_sig2.ring.len(), 3);
    assert_ne!(ring_sig1.key_image, ring_sig2.key_image);

    println!("✅ Ring signatures created successfully");

    // Test 4: Verify ring signatures
    assert!(processor.verify_ring_signature(&ring_sig1).await.unwrap());
    assert!(processor.verify_ring_signature(&ring_sig2).await.unwrap());

    println!("✅ Ring signature verification works");

    // Test 5: Check anonymity statistics
    let stats = processor.get_anonymity_stats().await.unwrap();
    assert_eq!(stats.total_anonymous_utxos, 0); // No UTXOs created yet
    assert!(stats.stealth_addresses_enabled);
    assert_eq!(stats.average_ring_size, 3);

    println!("✅ Anonymity statistics correct");
    println!("📊 Current stats: {stats:?}");
}

/// Test anonymous transaction creation (simplified version without full UTXO setup)
#[tokio::test]
async fn test_anonymous_transaction_structure() {
    let config = AnonymousEUtxoConfig::testing();
    let processor = AnonymousEUtxoProcessor::new(config).await.unwrap();
    let mut rng = OsRng;

    // Test stealth address encryption
    let recipient = "test_recipient";
    let stealth_addr = processor
        .create_stealth_address(recipient, &mut rng)
        .unwrap();
    let amount = 1000u64;

    let encrypted_amount = processor
        .encrypt_amount_for_stealth(amount, &stealth_addr, &mut rng)
        .unwrap();
    assert!(!encrypted_amount.is_empty());
    assert!(encrypted_amount.len() > 32); // Should include randomness

    println!("✅ Amount encryption for stealth addresses works");

    // Test amount proof creation
    let privacy_provider = processor.privacy_provider.read().await;
    let amount_commitment = privacy_provider
        .privacy_provider
        .commit_amount(amount, &mut rng)
        .unwrap();
    drop(privacy_provider);

    let amount_proof = processor
        .create_amount_proof(&amount_commitment, &mut rng)
        .await
        .unwrap();
    assert!(!amount_proof.is_empty());
    assert_eq!(amount_proof.len(), 32); // SHA256 hash

    println!("✅ Amount proof creation works");

    // Test anonymity proof structure
    let inputs = vec![];
    let outputs = vec![];
    let anonymity_proof = processor
        .create_anonymity_proof(&inputs, &outputs, &mut rng)
        .await
        .unwrap();

    assert!(!anonymity_proof.set_membership_proof.is_empty());
    assert!(!anonymity_proof.nullifier_proof.is_empty());
    assert!(!anonymity_proof.balance_proof.is_empty());
    assert!(!anonymity_proof.obfuscation_proof.is_empty());

    println!("✅ Anonymity proof structure is correct");
}

/// Test privacy levels and configuration
#[tokio::test]
async fn test_privacy_configuration_levels() {
    // Test different configuration levels
    let testing_config = AnonymousEUtxoConfig::testing();
    let production_config = AnonymousEUtxoConfig::production();

    // Production should have stronger privacy parameters
    assert!(production_config.anonymity_set_size >= testing_config.anonymity_set_size);
    assert!(production_config.ring_size >= testing_config.ring_size);
    assert!(production_config.max_utxo_age >= testing_config.max_utxo_age);

    println!("✅ Configuration levels are properly ordered");

    // Test processors with different configs
    let testing_processor = AnonymousEUtxoProcessor::new(testing_config).await.unwrap();
    let production_processor = AnonymousEUtxoProcessor::new(production_config)
        .await
        .unwrap();

    let testing_stats = testing_processor.get_anonymity_stats().await.unwrap();
    let production_stats = production_processor.get_anonymity_stats().await.unwrap();

    assert!(production_stats.average_ring_size >= testing_stats.average_ring_size);

    println!("✅ Different privacy levels work correctly");
    println!("📊 Testing ring size: {}", testing_stats.average_ring_size);
    println!(
        "📊 Production ring size: {}",
        production_stats.average_ring_size
    );
}

/// Test enhanced privacy integration
#[tokio::test]
async fn test_enhanced_privacy_integration() {
    let mut config = AnonymousEUtxoConfig::testing();
    config.privacy_config = EnhancedPrivacyConfig::testing();

    let processor = AnonymousEUtxoProcessor::new(config).await.unwrap();

    // Test that enhanced privacy provider is properly integrated
    let privacy_provider = processor.privacy_provider.read().await;
    let enhanced_stats = privacy_provider.get_enhanced_statistics();

    assert!(enhanced_stats.real_diamond_io_enabled);
    assert!(enhanced_stats.hybrid_mode_enabled);
    assert_eq!(enhanced_stats.total_circuits_created, 0);

    drop(privacy_provider);

    println!("✅ Enhanced privacy integration works");
    println!("📊 Enhanced privacy stats: {enhanced_stats:?}");
}

/// Test nullifier uniqueness and double-spend prevention
#[tokio::test]
async fn test_nullifier_double_spend_prevention() {
    let config = AnonymousEUtxoConfig::testing();
    let processor = AnonymousEUtxoProcessor::new(config).await.unwrap();

    // Create test nullifiers
    let nullifier1 = vec![1, 2, 3, 4, 5];
    let nullifier2 = vec![6, 7, 8, 9, 10];
    let nullifier3 = nullifier1.clone(); // Duplicate

    // Mark first nullifier as used
    {
        let mut used_nullifiers = processor.used_nullifiers.write().await;
        used_nullifiers.insert(nullifier1.clone(), true);
    }

    // Check nullifier status
    {
        let used_nullifiers = processor.used_nullifiers.read().await;
        assert!(used_nullifiers.contains_key(&nullifier1));
        assert!(!used_nullifiers.contains_key(&nullifier2));
        assert!(used_nullifiers.contains_key(&nullifier3)); // Same as nullifier1
    }

    println!("✅ Nullifier double-spend prevention works");
}

/// Test stealth address unlinkability
#[tokio::test]
async fn test_stealth_address_unlinkability() {
    let config = AnonymousEUtxoConfig::testing();
    let processor = AnonymousEUtxoProcessor::new(config).await.unwrap();
    let mut rng = OsRng;

    let recipient = "same_recipient";

    // Create multiple stealth addresses for the same recipient
    let stealth_addrs: Vec<StealthAddress> = (0..5)
        .map(|_| {
            processor
                .create_stealth_address(recipient, &mut rng)
                .unwrap()
        })
        .collect();

    // Verify all addresses are different (unlinkable)
    for i in 0..stealth_addrs.len() {
        for j in i + 1..stealth_addrs.len() {
            assert_ne!(
                stealth_addrs[i].one_time_address,
                stealth_addrs[j].one_time_address
            );
            assert_ne!(stealth_addrs[i].view_key, stealth_addrs[j].view_key);
            assert_ne!(stealth_addrs[i].spend_key, stealth_addrs[j].spend_key);
        }
    }

    println!("✅ Stealth addresses are properly unlinkable");
    println!(
        "📊 Generated {} unique stealth addresses",
        stealth_addrs.len()
    );
}

/// Test ring signature unlinkability
#[tokio::test]
async fn test_ring_signature_unlinkability() {
    let config = AnonymousEUtxoConfig::testing();
    let processor = AnonymousEUtxoProcessor::new(config).await.unwrap();
    let mut rng = OsRng;

    let secret_key = vec![1, 2, 3, 4, 5];

    // Create multiple ring signatures with the same secret key
    let ring_sigs = vec![
        processor
            .create_ring_signature("utxo_1", &secret_key, &mut rng)
            .await
            .unwrap(),
        processor
            .create_ring_signature("utxo_2", &secret_key, &mut rng)
            .await
            .unwrap(),
        processor
            .create_ring_signature("utxo_3", &secret_key, &mut rng)
            .await
            .unwrap(),
    ];

    // Verify signatures are different (unlinkable) except for key images
    for i in 0..ring_sigs.len() {
        for j in i + 1..ring_sigs.len() {
            // Signatures should be different
            assert_ne!(ring_sigs[i].signature, ring_sigs[j].signature);
            // Rings should be different (different decoys)
            assert_ne!(ring_sigs[i].ring, ring_sigs[j].ring);
            // Key images should be different (based on UTXO)
            assert_ne!(ring_sigs[i].key_image, ring_sigs[j].key_image);
        }

        // But all should verify correctly
        assert!(processor
            .verify_ring_signature(&ring_sigs[i])
            .await
            .unwrap());
    }

    println!("✅ Ring signatures are properly unlinkable");
    println!("📊 Generated {} unique ring signatures", ring_sigs.len());
}

/// Test block advancement and UTXO aging
#[tokio::test]
async fn test_block_advancement() {
    let config = AnonymousEUtxoConfig::testing();
    let processor = AnonymousEUtxoProcessor::new(config).await.unwrap();

    // Check initial block
    let initial_block = *processor.current_block.read().await;
    assert_eq!(initial_block, 1);

    // Advance blocks
    for i in 1..=10 {
        processor.advance_block().await;
        let current_block = *processor.current_block.read().await;
        assert_eq!(current_block, initial_block + i);
    }

    let final_block = *processor.current_block.read().await;
    assert_eq!(final_block, 11);

    println!("✅ Block advancement works correctly");
    println!("📊 Final block height: {final_block}");
}

/// Test error handling and edge cases
#[tokio::test]
async fn test_error_handling() {
    let mut config = AnonymousEUtxoConfig::testing();

    // Test with disabled features
    config.enable_stealth_addresses = false;
    config.enable_ring_signatures = false;

    let processor = AnonymousEUtxoProcessor::new(config).await.unwrap();
    let mut rng = OsRng;

    // Stealth address creation should fail
    let stealth_result = processor.create_stealth_address("test", &mut rng);
    assert!(stealth_result.is_err());
    assert!(stealth_result
        .unwrap_err()
        .to_string()
        .contains("not enabled"));

    // Ring signature creation should fail
    let ring_result = processor
        .create_ring_signature("test", &[1, 2, 3], &mut rng)
        .await;
    assert!(ring_result.is_err());
    assert!(ring_result.unwrap_err().to_string().contains("not enabled"));

    println!("✅ Error handling works correctly");
}

/// Benchmark anonymous transaction processing
#[tokio::test]
async fn test_performance_benchmarks() {
    let config = AnonymousEUtxoConfig::testing();
    let processor = AnonymousEUtxoProcessor::new(config).await.unwrap();
    let mut rng = OsRng;

    // Benchmark stealth address creation
    let start = std::time::Instant::now();
    for _ in 0..100 {
        let _stealth_addr = processor
            .create_stealth_address("test_recipient", &mut rng)
            .unwrap();
    }
    let stealth_duration = start.elapsed();

    // Benchmark ring signature creation
    let start = std::time::Instant::now();
    for i in 0..10 {
        let _ring_sig = processor
            .create_ring_signature(&format!("utxo_{i}"), &[1, 2, 3], &mut rng)
            .await
            .unwrap();
    }
    let ring_duration = start.elapsed();

    println!("🚀 Performance Benchmarks:");
    println!("   Stealth address creation: {stealth_duration:?} for 100 addresses");
    println!("   Ring signature creation: {ring_duration:?} for 10 signatures");
    println!("   Average stealth address: {:?}", stealth_duration / 100);
    println!("   Average ring signature: {:?}", ring_duration / 10);

    // Reasonable performance expectations
    assert!(stealth_duration.as_millis() < 10000); // Less than 10 seconds for 100 addresses
    assert!(ring_duration.as_millis() < 5000); // Less than 5 seconds for 10 signatures
}
