use polytorus::diamond_io_integration::{DiamondIOConfig, DiamondIOIntegration};
use num_bigint::BigUint;
use num_traits::Num;

#[tokio::test]
async fn test_basic_integration() {
    let config = DiamondIOConfig {
        ring_dimension: 16,
        crt_depth: 4,
        crt_bits: 30,
        base_bits: 4,
        switched_modulus: BigUint::from_str_radix("17592454479871", 10).unwrap(),
        input_size: 2,
        level_width: 2,
        d: 2,
        hardcoded_key_sigma: 4.578,
        p_sigma: 4.578,
        trapdoor_sigma: Some(4.578),
        dummy_mode: true, // Enable dummy mode for testing
    };

    let integration = DiamondIOIntegration::new(config);
    assert!(integration.is_ok());
    
    let integration = integration.unwrap();
    let circuit = integration.create_demo_circuit();
    
    // Verify circuit has inputs and outputs
    assert!(circuit.num_input() > 0);
    assert!(circuit.num_output() > 0);
    
    println!("Basic integration test passed");
}

#[tokio::test]
async fn test_dummy_mode_obfuscation_and_evaluation() {
    let config = DiamondIOConfig::dummy();
    let integration = DiamondIOIntegration::new(config).unwrap();
    
    let circuit = integration.create_demo_circuit();
    
    // Test obfuscation in dummy mode
    let obfuscation_result = integration.obfuscate_circuit(circuit).await;
    assert!(obfuscation_result.is_ok());
    
    // Test evaluation in dummy mode
    let inputs = vec![true, false];
    let evaluation_result = integration.evaluate_circuit(&inputs).await;
    assert!(evaluation_result.is_ok());
    
    let outputs = evaluation_result.unwrap();
    assert!(!outputs.is_empty());
    
    println!("Dummy mode obfuscation and evaluation test passed");
}

#[tokio::test]
async fn test_smart_contract_with_diamond_io() {
    let config = DiamondIOConfig::dummy();
    let mut integration = DiamondIOIntegration::new(config).unwrap();
    integration.set_obfuscation_dir("test_contract_obfuscation".to_string());
    
    let circuit = integration.create_demo_circuit();
    
    // Obfuscate the circuit
    let obfuscation_result = integration.obfuscate_circuit(circuit).await;
    assert!(obfuscation_result.is_ok());
    
    // Test contract execution
    let inputs = vec![true, false];
    let outputs = integration.evaluate_circuit(&inputs).await;
    assert!(outputs.is_ok());
    
    println!("Smart contract integration test passed");
}

#[tokio::test]
async fn test_modular_layer_integration() {
    let config = DiamondIOConfig::dummy();
    let integration = DiamondIOIntegration::new(config).unwrap();
    
    // Test encryption functionality
    let test_data = vec![true, false, true, false];
    let encryption_result = integration.encrypt_data(&test_data);
    assert!(encryption_result.is_ok());
    
    let encrypted_data = encryption_result.unwrap();
    assert!(!encrypted_data.is_empty());
    assert_eq!(encrypted_data.len(), test_data.len());
    
    println!("Modular layer integration test passed");
}

#[tokio::test]
async fn test_config_variations() {
    // Test default config
    let default_config = DiamondIOConfig::default();
    let default_integration = DiamondIOIntegration::new(default_config);
    assert!(default_integration.is_ok());
    
    // Test dummy config
    let dummy_config = DiamondIOConfig::dummy();
    let dummy_integration = DiamondIOIntegration::new(dummy_config);
    assert!(dummy_integration.is_ok());
    
    // Test testing config
    let testing_config = DiamondIOConfig::testing();
    let testing_integration = DiamondIOIntegration::new(testing_config);
    assert!(testing_integration.is_ok());
    
    // Test production config (should work but may be slow)
    let production_config = DiamondIOConfig {
        ring_dimension: 64, // Smaller than real production for testing
        crt_depth: 8,
        crt_bits: 35,
        base_bits: 6,
        switched_modulus: BigUint::from_str_radix("549755813887", 10).unwrap(),
        input_size: 8,
        level_width: 4,
        d: 4,
        hardcoded_key_sigma: 2.0,
        p_sigma: 2.0,
        trapdoor_sigma: Some(4.578),
        dummy_mode: true, // Use dummy mode for testing speed
    };
    let production_integration = DiamondIOIntegration::new(production_config);
    assert!(production_integration.is_ok());
    
    println!("Config variations test passed");
}

#[tokio::test]
async fn test_performance_comparison() {
    use std::time::Instant;
    
    // Test dummy mode performance
    let dummy_config = DiamondIOConfig::dummy();
    let dummy_integration = DiamondIOIntegration::new(dummy_config).unwrap();
    
    let circuit = dummy_integration.create_demo_circuit();
    
    let start = Instant::now();
    let obfuscation_result = dummy_integration.obfuscate_circuit(circuit).await;
    let dummy_obfuscation_time = start.elapsed();
    assert!(obfuscation_result.is_ok());
    
    let start = Instant::now();
    let inputs = vec![true, false];
    let evaluation_result = dummy_integration.evaluate_circuit(&inputs).await;
    let dummy_evaluation_time = start.elapsed();
    assert!(evaluation_result.is_ok());
    
    println!("Dummy mode - Obfuscation: {:?}, Evaluation: {:?}", 
             dummy_obfuscation_time, dummy_evaluation_time);
    
    // Verify dummy mode is fast (should be under 1ms for basic operations)
    assert!(dummy_obfuscation_time.as_millis() < 100);
    assert!(dummy_evaluation_time.as_millis() < 100);
    
    println!("Performance comparison test passed");
}
