//! Integration tests for ZK-STARKs based anonymous eUTXO system
//!
//! This module tests the complete ZK-STARKs anonymous eUTXO workflow including
//! quantum-resistant proofs, stealth addresses, and post-quantum security.

use polytorus::crypto::zk_starks_anonymous_eutxo::{
    StarkAnonymityStats, ZkStarksEUtxoConfig, ZkStarksEUtxoProcessor,
};
use rand_core::OsRng;

/// Test complete ZK-STARKs anonymous eUTXO workflow
#[tokio::test]
async fn test_complete_zk_starks_eutxo_workflow() {
    let config = ZkStarksEUtxoConfig::testing();
    let processor = ZkStarksEUtxoProcessor::new(config).await.unwrap();
    let mut rng = OsRng;

    // Test 1: Create processor and verify initial state
    println!("Testing ZK-STARKs processor creation...");
    let stats = processor.get_stark_anonymity_stats().await.unwrap();
    assert_eq!(stats.total_stark_utxos, 0);
    assert!(stats.post_quantum_secure);
    assert_eq!(stats.proof_system, "ZK-STARKs");
    assert!(stats.security_level_bits >= 80);

    println!("✅ ZK-STARKs processor created successfully");
    println!("   📊 Security level: {} bits", stats.security_level_bits);
    println!("   🔒 Post-quantum secure: {}", stats.post_quantum_secure);
    println!("   🎯 Proof system: {}", stats.proof_system);

    // Test 2: Create stealth addresses
    println!("\nTesting STARK stealth address creation...");
    let recipients = vec!["alice_stark", "bob_stark", "charlie_stark"];

    let mut stealth_addresses = Vec::new();
    for recipient in &recipients {
        let stealth_addr = processor
            .create_stealth_address(recipient, &mut rng)
            .unwrap();

        assert!(stealth_addr.one_time_address.starts_with("stark_stealth_"));
        assert!(!stealth_addr.view_key.is_empty());
        assert!(!stealth_addr.spend_key.is_empty());
        assert!(processor.verify_stealth_address(&stealth_addr).unwrap());

        stealth_addresses.push(stealth_addr);
    }

    println!("✅ STARK stealth addresses created successfully");
    println!(
        "   📝 Created {} unique stealth addresses",
        stealth_addresses.len()
    );

    // Test 3: Create STARK proofs
    println!("\nTesting STARK proof creation...");

    // Test ownership proof
    let ownership_proof = processor
        .create_stark_ownership_proof("test_utxo", &[1, 2, 3, 4, 5], &mut rng)
        .await
        .unwrap();

    assert!(!ownership_proof.proof_data.is_empty());
    assert!(!ownership_proof.public_inputs.is_empty());
    assert!(ownership_proof.metadata.proof_size > 0);
    assert!(ownership_proof.metadata.security_level >= 80);

    println!("✅ STARK ownership proof created");
    println!(
        "   📏 Proof size: {} bytes",
        ownership_proof.metadata.proof_size
    );
    println!(
        "   ⏱️ Generation time: {}ms",
        ownership_proof.metadata.generation_time
    );

    // Test range proof
    let amount = 1000u64;
    let privacy_provider = processor.privacy_provider.read().await;
    let commitment = privacy_provider
        .privacy_provider
        .commit_amount(amount, &mut rng)
        .unwrap();
    drop(privacy_provider);

    let range_proof = processor
        .create_stark_range_proof(amount, &commitment, &mut rng)
        .await
        .unwrap();

    assert!(!range_proof.proof_data.is_empty());
    assert_eq!(range_proof.public_inputs[0], amount);

    println!("✅ STARK range proof created");
    println!("   💰 Amount: {}", amount);
    println!(
        "   📏 Proof size: {} bytes",
        range_proof.metadata.proof_size
    );

    // Test 4: STARK proof verification
    println!("\nTesting STARK proof verification...");

    let ownership_valid = processor
        .verify_stark_proof(&ownership_proof)
        .await
        .unwrap();
    let range_valid = processor.verify_stark_proof(&range_proof).await.unwrap();

    assert!(ownership_valid);
    assert!(range_valid);

    println!("✅ STARK proof verification successful");
    println!("   🔐 Ownership proof valid: {}", ownership_valid);
    println!("   📊 Range proof valid: {}", range_valid);

    // Test 5: Security level verification
    println!("\nTesting security levels...");

    let testing_config = ZkStarksEUtxoConfig::testing();
    let production_config = ZkStarksEUtxoConfig::production();

    let testing_processor = ZkStarksEUtxoProcessor::new(testing_config).await.unwrap();
    let production_processor = ZkStarksEUtxoProcessor::new(production_config)
        .await
        .unwrap();

    let testing_security = testing_processor.calculate_security_bits();
    let production_security = production_processor.calculate_security_bits();

    assert!(production_security >= testing_security);
    assert!(testing_security >= 80);
    assert!(production_security >= 100);

    println!("✅ Security levels validated");
    println!("   🧪 Testing security: {} bits", testing_security);
    println!("   🏭 Production security: {} bits", production_security);

    println!("\n🎉 ZK-STARKs anonymous eUTXO workflow completed successfully!");
}

/// Test ZK-STARKs configuration levels
#[tokio::test]
async fn test_zk_starks_configuration_levels() {
    let testing_config = ZkStarksEUtxoConfig::testing();
    let production_config = ZkStarksEUtxoConfig::production();

    // Production should have stronger parameters
    assert!(
        production_config.proof_options.num_queries >= testing_config.proof_options.num_queries
    );
    assert!(
        production_config.proof_options.blowup_factor >= testing_config.proof_options.blowup_factor
    );
    assert!(
        production_config.proof_options.grinding_bits >= testing_config.proof_options.grinding_bits
    );
    assert!(production_config.anonymity_set_size >= testing_config.anonymity_set_size);

    println!("✅ ZK-STARKs configuration levels verified");
    println!(
        "   🧪 Testing queries: {}",
        testing_config.proof_options.num_queries
    );
    println!(
        "   🏭 Production queries: {}",
        production_config.proof_options.num_queries
    );
    println!(
        "   🧪 Testing blowup: {}",
        testing_config.proof_options.blowup_factor
    );
    println!(
        "   🏭 Production blowup: {}",
        production_config.proof_options.blowup_factor
    );
}

/// Test post-quantum security guarantees
#[tokio::test]
async fn test_post_quantum_security() {
    let config = ZkStarksEUtxoConfig::production();
    let processor = ZkStarksEUtxoProcessor::new(config).await.unwrap();

    let stats = processor.get_stark_anonymity_stats().await.unwrap();

    // Verify post-quantum properties
    assert!(stats.post_quantum_secure);
    assert_eq!(stats.proof_system, "ZK-STARKs");
    assert!(stats.security_level_bits >= 128); // Post-quantum security level
    assert_eq!(stats.max_anonymity_level, "quantum_resistant_maximum");

    println!("✅ Post-quantum security verified");
    println!("   🔒 Post-quantum secure: {}", stats.post_quantum_secure);
    println!("   🛡️ Security level: {} bits", stats.security_level_bits);
    println!("   📊 Anonymity level: {}", stats.max_anonymity_level);
}

/// Test STARK proof performance benchmarks
#[tokio::test]
async fn test_stark_proof_performance() {
    let config = ZkStarksEUtxoConfig::testing();
    let processor = ZkStarksEUtxoProcessor::new(config).await.unwrap();
    let mut rng = OsRng;

    // Benchmark STARK proof generation
    println!("🚀 Benchmarking STARK proof performance...");

    let mut generation_times = Vec::new();
    let mut verification_times = Vec::new();
    let mut proof_sizes = Vec::new();

    for i in 0..5 {
        let start = std::time::Instant::now();
        let proof = processor
            .create_generic_stark_proof(&format!("benchmark_{}", i), 42 + i as u64, &mut rng)
            .await
            .unwrap();
        let generation_time = start.elapsed();

        let start = std::time::Instant::now();
        let valid = processor.verify_stark_proof(&proof).await.unwrap();
        let verification_time = start.elapsed();

        assert!(valid);

        generation_times.push(generation_time);
        verification_times.push(verification_time);
        proof_sizes.push(proof.metadata.proof_size);
    }

    let avg_generation =
        generation_times.iter().sum::<std::time::Duration>() / generation_times.len() as u32;
    let avg_verification =
        verification_times.iter().sum::<std::time::Duration>() / verification_times.len() as u32;
    let avg_size = proof_sizes.iter().sum::<usize>() / proof_sizes.len();

    println!("📊 Performance Results:");
    println!("   ⚡ Average generation time: {:?}", avg_generation);
    println!("   🔍 Average verification time: {:?}", avg_verification);
    println!("   📏 Average proof size: {} bytes", avg_size);

    // Performance expectations for STARK proofs
    assert!(avg_generation.as_millis() < 10000); // Less than 10 seconds
    assert!(avg_verification.as_millis() < 1000); // Less than 1 second
    assert!(avg_size < 100000); // Less than 100KB
}

/// Test stealth address unlinkability with STARKs
#[tokio::test]
async fn test_stark_stealth_address_unlinkability() {
    let config = ZkStarksEUtxoConfig::testing();
    let processor = ZkStarksEUtxoProcessor::new(config).await.unwrap();
    let mut rng = OsRng;

    let recipient = "same_recipient_stark";

    // Create multiple stealth addresses for the same recipient
    let stealth_addrs = (0..5)
        .map(|_| {
            processor
                .create_stealth_address(recipient, &mut rng)
                .unwrap()
        })
        .collect::<Vec<_>>();

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

    println!("✅ STARK stealth addresses are properly unlinkable");
    println!(
        "📊 Generated {} unique stealth addresses",
        stealth_addrs.len()
    );
}

/// Test STARK anonymity statistics
#[tokio::test]
async fn test_stark_anonymity_statistics() {
    let config = ZkStarksEUtxoConfig::testing();
    let processor = ZkStarksEUtxoProcessor::new(config).await.unwrap();

    let stats = processor.get_stark_anonymity_stats().await.unwrap();

    // Verify statistics structure
    assert_eq!(stats.total_stark_utxos, 0);
    assert_eq!(stats.active_anonymity_sets, 0);
    assert_eq!(stats.used_nullifiers, 0);
    assert!(stats.stealth_addresses_enabled);
    assert!(stats.post_quantum_secure);
    assert_eq!(stats.proof_system, "ZK-STARKs");

    println!("📊 STARK Anonymity Statistics:");
    println!("   💎 Total STARKs UTXOs: {}", stats.total_stark_utxos);
    println!("   🎯 Anonymity sets: {}", stats.active_anonymity_sets);
    println!("   🔒 Used nullifiers: {}", stats.used_nullifiers);
    println!("   📏 Anonymity set size: {}", stats.anonymity_set_size);
    println!("   🛡️ Security level: {} bits", stats.security_level_bits);
    println!("   🔐 Post-quantum: {}", stats.post_quantum_secure);
}

/// Test block advancement with STARK system
#[tokio::test]
async fn test_stark_block_advancement() {
    let config = ZkStarksEUtxoConfig::testing();
    let processor = ZkStarksEUtxoProcessor::new(config).await.unwrap();

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

    println!("✅ STARK block advancement works correctly");
    println!("📦 Final block height: {}", final_block);
}

/// Test error handling with disabled features
#[tokio::test]
async fn test_stark_error_handling() {
    let mut config = ZkStarksEUtxoConfig::testing();

    // Test with disabled stealth addresses
    config.enable_stealth_addresses = false;
    let processor = ZkStarksEUtxoProcessor::new(config).await.unwrap();
    let mut rng = OsRng;

    let stealth_result = processor.create_stealth_address("test", &mut rng);
    assert!(stealth_result.is_err());
    assert!(stealth_result
        .unwrap_err()
        .to_string()
        .contains("not enabled"));

    println!("✅ STARK error handling works correctly");
}

/// Compare ZK-STARKs vs traditional zk-SNARKs features
#[tokio::test]
async fn test_stark_vs_snark_comparison() {
    let stark_config = ZkStarksEUtxoConfig::production();
    let stark_processor = ZkStarksEUtxoProcessor::new(stark_config).await.unwrap();

    let stark_stats = stark_processor.get_stark_anonymity_stats().await.unwrap();

    println!("🔬 ZK-STARKs vs zk-SNARKs Comparison:");
    println!("   📊 ZK-STARKs Features:");
    println!("      • No trusted setup required ✅");
    println!("      • Quantum resistant ✅");
    println!("      • Transparent ✅");
    println!("      • Larger proof sizes ⚠️");
    println!(
        "      • Post-quantum secure: {}",
        stark_stats.post_quantum_secure
    );
    println!(
        "      • Security level: {} bits",
        stark_stats.security_level_bits
    );

    println!("   📊 Traditional zk-SNARKs:");
    println!("      • Requires trusted setup ❌");
    println!("      • Not quantum resistant ❌");
    println!("      • Smaller proof sizes ✅");
    println!("      • Faster verification ✅");

    // Verify STARK advantages
    assert!(stark_stats.post_quantum_secure);
    assert!(stark_stats.security_level_bits >= 128);
    assert_eq!(stark_stats.proof_system, "ZK-STARKs");
}
