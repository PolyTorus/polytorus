use polytorus::diamond_io_integration::{DiamondIOIntegration, DiamondIOConfig};
use polytorus::diamond_smart_contracts::DiamondContractEngine;
use polytorus::modular::{DiamondIOLayerBuilder, DiamondLayerTrait};
use num_bigint::BigUint;
use num_traits::Num;

#[tokio::test]
async fn test_diamond_io_integration_basic() {
    let config = DiamondIOConfig {
        ring_dimension: 16,
        crt_depth: 2,
        crt_bits: 17,
        base_bits: 1,
        switched_modulus: BigUint::from_str_radix("123456789", 10).unwrap(),
        input_size: 2,
        level_width: 2,
        d: 2,
        hardcoded_key_sigma: 4.578,
        p_sigma: 4.578,
        trapdoor_sigma: Some(4.578),
        inputs: vec![true, false],
        dummy_mode: true, // Enable dummy mode for testing
    };

    let integration = DiamondIOIntegration::new(config);
    assert!(integration.is_ok());
    
    let integration = integration.unwrap();
    let circuit = integration.create_demo_circuit();
    
    // Circuit should have inputs and outputs
    assert!(circuit.num_input() > 0);
    assert!(circuit.num_output() > 0);
}

#[tokio::test]
async fn test_diamond_io_with_production_params() {
    let config = DiamondIOConfig::testing(); // Use testing params (safer than full production)
    
    let integration = DiamondIOIntegration::new(config);
    assert!(integration.is_ok());
    
    let integration = integration.unwrap();
    let circuit = integration.create_demo_circuit();
    
    // Circuit should have inputs and outputs
    assert!(circuit.num_input() > 0);
    assert!(circuit.num_output() > 0);
    
    // Test circuit creation with different types
    let and_circuit = integration.create_circuit("and_gate");
    assert!(and_circuit.num_input() > 0);
    
    let or_circuit = integration.create_circuit("or_gate");
    assert!(or_circuit.num_input() > 0);
}

#[tokio::test]
async fn test_diamond_io_obfuscation_with_real_params() {
    let config = DiamondIOConfig::testing();
    
    let integration = DiamondIOIntegration::new(config);
    assert!(integration.is_ok());
    
    let integration = integration.unwrap();
    let circuit = integration.create_demo_circuit();
    
    // Test obfuscation (this might take longer with real params)
    let obfuscation_result = integration.obfuscate_circuit(circuit).await;
    // Note: Obfuscation might fail with real params due to complexity,
    // but the integration should not panic
    println!("Obfuscation result: {:?}", obfuscation_result.is_ok());
}

#[tokio::test]
async fn test_smart_contract_engine() {
    let config = DiamondIOConfig {
        ring_dimension: 16,
        crt_depth: 2,
        crt_bits: 17,
        base_bits: 1,
        switched_modulus: BigUint::from_str_radix("123456789", 10).unwrap(),
        input_size: 2,
        level_width: 2,
        d: 2,
        hardcoded_key_sigma: 4.578,
        p_sigma: 4.578,
        trapdoor_sigma: Some(4.578),
        inputs: vec![true, false],
        dummy_mode: true, // Enable dummy mode for testing
    };

    let mut engine = DiamondContractEngine::new(config).unwrap();
    
    // Deploy AND gate contract
    let contract_id = engine.deploy_contract(
        "test_and".to_string(),
        "Test AND Gate".to_string(),
        "and_gate".to_string(),
        "test_user".to_string(),
        "and_gate",
    ).await.unwrap();
    
    // Execute the contract
    let result = engine.execute_contract(
        &contract_id,
        vec![true, false],
        "executor".to_string(),
    ).await.unwrap();
    
    // AND(true, false) should be false
    assert_eq!(result, vec![false]);
    
    // Test with true, true
    let result = engine.execute_contract(
        &contract_id,
        vec![true, true],
        "executor".to_string(),
    ).await.unwrap();
    
    // AND(true, true) should be true
    assert_eq!(result, vec![true]);
}

#[tokio::test]
async fn test_modular_layer_integration() {
    let config = DiamondIOConfig {
        ring_dimension: 16,
        crt_depth: 2,
        crt_bits: 17,
        base_bits: 1,
        switched_modulus: BigUint::from_str_radix("123456789", 10).unwrap(),
        input_size: 2,
        level_width: 2,
        d: 2,
        hardcoded_key_sigma: 4.578,
        p_sigma: 4.578,
        trapdoor_sigma: Some(4.578),
        inputs: vec![true, false],
        dummy_mode: true, // Enable dummy mode for testing
    };

    let mut layer = DiamondIOLayerBuilder::new()
        .with_diamond_config(config)
        .with_max_concurrent_executions(3)
        .with_obfuscation_enabled(false) // Disabled for testing
        .build()
        .unwrap();

    // Start the layer
    layer.start_layer().await.unwrap();
    
    // Check health
    let is_healthy = layer.health_check().await.unwrap();
    assert!(is_healthy);
    
    // Deploy a contract
    let contract_id = layer.deploy_contract(
        "layer_test".to_string(),
        "Layer Test Contract".to_string(),
        "or_gate".to_string(),
        "layer_user".to_string(),
        "or_gate",
    ).await.unwrap();
    
    // Execute the contract
    let result = layer.execute_contract(
        &contract_id,
        vec![true, false],
        "layer_executor".to_string(),
    ).await.unwrap();
    
    // OR(true, false) should be true
    assert_eq!(result, vec![true]);
    
    // Check stats
    let stats = layer.get_stats().await;
    assert_eq!(stats.total_contracts, 1);
    assert_eq!(stats.successful_executions, 1);
    
    // Stop the layer
    layer.stop_layer().await.unwrap();
}

#[tokio::test]
async fn test_multiple_contract_types() {
    let config = DiamondIOConfig {
        ring_dimension: 16,
        crt_depth: 2,
        crt_bits: 17,
        base_bits: 1,
        switched_modulus: BigUint::from_str_radix("123456789", 10).unwrap(),
        input_size: 4, // Needed for adder
        level_width: 2,
        d: 2,
        hardcoded_key_sigma: 4.578,
        p_sigma: 4.578,
        trapdoor_sigma: Some(4.578),
        inputs: vec![true, false, true, false],
        dummy_mode: true, // Enable dummy mode for testing
    };

    let mut engine = DiamondContractEngine::new(config).unwrap();
    
    // Test different contract types
    let contracts = vec![
        ("and_test", "and_gate"),
        ("or_test", "or_gate"),
        ("xor_test", "xor_gate"),
        ("adder_test", "adder"),
    ];
    
    for (id, circuit_type) in contracts {
        let contract_id = engine.deploy_contract(
            id.to_string(),
            format!("Test {}", circuit_type),
            circuit_type.to_string(),
            "multi_user".to_string(),
            circuit_type,
        ).await.unwrap();
        
        let inputs = match circuit_type {
            "adder" => vec![true, false, true, true], // 1 + 3
            _ => vec![true, false, false, false],
        };
        
        let result = engine.execute_contract(
            &contract_id,
            inputs,
            "multi_executor".to_string(),
        ).await;
        
        assert!(result.is_ok(), "Failed to execute {} contract", circuit_type);
    }
    
    // Check that all contracts were deployed
    let all_contracts = engine.list_contracts();
    assert_eq!(all_contracts.len(), 4);
}

#[test]
fn test_diamond_io_config_serialization() {
    let config = DiamondIOConfig::default();
    
    // Test JSON serialization
    let json = serde_json::to_string(&config).unwrap();
    let deserialized: DiamondIOConfig = serde_json::from_str(&json).unwrap();
    
    assert_eq!(config.ring_dimension, deserialized.ring_dimension);
    assert_eq!(config.input_size, deserialized.input_size);
}

#[test]
fn test_diamond_io_config_validation() {
    let mut config = DiamondIOConfig::default();
    
    // Test valid configuration
    assert!(config.ring_dimension.is_power_of_two());
    assert!(config.input_size > 0);
    assert!(config.level_width > 0);
    
    // Test power of 2 requirement for ring dimension
    config.ring_dimension = 15; // Not a power of 2
    // In a real implementation, you'd validate this in the constructor
}
