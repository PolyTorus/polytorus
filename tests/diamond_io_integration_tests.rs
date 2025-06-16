use polytorus::diamond_io_integration_new::{
    DiamondIOConfig,
    DiamondIOIntegration,
};

#[tokio::test]
async fn test_basic_integration() {
    let config = DiamondIOConfig::testing();

    let integration = DiamondIOIntegration::new(config);
    assert!(integration.is_ok());

    let integration = integration.unwrap();
    let circuit = integration.create_demo_circuit();

    // Verify circuit has inputs and outputs
    assert!(circuit.num_input() > 0);
    assert!(circuit.num_output() > 0);
}

#[tokio::test]
async fn test_circuit_execution() {
    let config = DiamondIOConfig::testing();
    let integration = DiamondIOIntegration::new(config).unwrap();
    let circuit = integration.create_demo_circuit();

    let result = integration.obfuscate_circuit(circuit).await;
    assert!(result.is_ok());

    // Test evaluation after obfuscation
    let inputs = vec![true, false, true, false];
    let result = integration.execute_circuit_detailed(&inputs).await;
    assert!(result.is_ok());
    
    let result = result.unwrap();
    assert!(result.success);
    assert!(!result.outputs.is_empty());
}

#[tokio::test]
async fn test_circuit_evaluation() {
    let config = DiamondIOConfig::testing();
    let integration = DiamondIOIntegration::new(config).unwrap();

    let inputs = vec![true, false, true, true];
    let outputs = integration.execute_circuit_detailed(&inputs).await;
    assert!(outputs.is_ok());

    let outputs = outputs.unwrap();
    assert!(outputs.success);
    assert!(!outputs.outputs.is_empty());
}

#[tokio::test]
async fn test_contract_obfuscation() {
    let config = DiamondIOConfig::testing();
    let mut integration = DiamondIOIntegration::new(config).unwrap();
    let circuit = integration.create_demo_circuit();

    // Test obfuscation directory setting (no-op)
    integration.set_obfuscation_dir("test_contract_obfuscation".to_string());

    // Test circuit obfuscation
    let obfuscation_result = integration.obfuscate_circuit(circuit).await;
    assert!(obfuscation_result.is_ok());

    let inputs = vec![true, false, true, false];
    let outputs = integration.execute_circuit_detailed(&inputs).await;
    assert!(outputs.is_ok());
}

#[tokio::test]
async fn test_simple_circuit_operations() {
    let config = DiamondIOConfig::testing();
    let integration = DiamondIOIntegration::new(config).unwrap();
    let circuit = integration.create_demo_circuit();

    let obfuscation_result = integration.obfuscate_circuit(circuit).await;
    assert!(obfuscation_result.is_ok());

    // Test evaluation after obfuscation to verify functionality
    let inputs = vec![true, false, true, false];
    let result = integration.execute_circuit_detailed(&inputs).await;
    assert!(result.is_ok());
    let result = result.unwrap();
    assert!(result.success);
    // Check that execution time is recorded
    assert!(result.execution_time_ms < 1000000); // Reasonable upper bound
}

#[tokio::test]
async fn test_dummy_mode_performance() {
    let config = DiamondIOConfig::dummy();
    let integration = DiamondIOIntegration::new(config).unwrap();
    let circuit = integration.create_demo_circuit();

    let obfuscation_result = integration.obfuscate_circuit(circuit).await;
    assert!(obfuscation_result.is_ok());

    // Test evaluation to verify dummy mode works
    let inputs = vec![true, false, true, false];
    let result = integration.execute_circuit_detailed(&inputs).await;
    assert!(result.is_ok());
    let result = result.unwrap();
    assert!(result.success);
}
