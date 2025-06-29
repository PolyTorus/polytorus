use polytorus::diamond_io_integration_new::{PrivacyEngineConfig, PrivacyEngineIntegration};

#[tokio::test]
async fn test_basic_integration() {
    let config = PrivacyEngineConfig::testing();

    let integration = PrivacyEngineIntegration::new(config);
    assert!(integration.is_ok());

    let integration = integration.unwrap();
    let circuit = integration.create_demo_circuit();

    // Verify circuit has inputs and outputs
    assert!(circuit.num_input() > 0);
    assert!(circuit.num_output() > 0);
}

#[tokio::test]
async fn test_circuit_execution() {
    let config = PrivacyEngineConfig::testing();
    let mut integration = PrivacyEngineIntegration::new(config).unwrap();

    // Set a unique obfuscation directory for this test
    integration.set_obfuscation_dir("test_circuit_execution_obfuscation".to_string());

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
    // Initialize logging
    env_logger::init();

    // Check environment variables
    println!("Environment diagnostics:");
    println!("OPENFHE_ROOT: {:?}", std::env::var("OPENFHE_ROOT"));
    println!("LD_LIBRARY_PATH: {:?}", std::env::var("LD_LIBRARY_PATH"));
    println!("PKG_CONFIG_PATH: {:?}", std::env::var("PKG_CONFIG_PATH"));

    // Check if OpenFHE libraries are accessible
    let lib_paths = vec![
        "/usr/local/lib/libOPENFHEcore.so",
        "/usr/local/lib/libOPENFHEpke.so",
        "/usr/local/lib/libOPENFHEbinfhe.so",
    ];

    for lib_path in &lib_paths {
        if std::path::Path::new(lib_path).exists() {
            println!("✓ Found library: {lib_path}");
        } else {
            println!("✗ Missing library: {lib_path}");
        }
    }

    // Check if we're in CI environment
    let is_ci = std::env::var("CI").is_ok() || std::env::var("GITHUB_ACTIONS").is_ok();
    println!("Running in CI environment: {is_ci}");

    let config = if is_ci && std::env::var("FORCE_OPENFHE_CI").is_err() {
        println!("Using dummy mode for CI environment (set FORCE_OPENFHE_CI=1 to override)");
        PrivacyEngineConfig::dummy()
    } else {
        println!("Using real OpenFHE mode");
        PrivacyEngineConfig::testing()
    };
    println!("Created config: {config:?}");

    // Try to create the integration with detailed error handling
    println!("Attempting to create PrivacyEngineIntegration...");
    let integration = match PrivacyEngineIntegration::new(config) {
        Ok(integration) => {
            println!("✓ Successfully created PrivacyEngineIntegration");
            integration
        }
        Err(e) => {
            eprintln!("\n=== PrivacyEngineIntegration::new FAILED ===");
            eprintln!("Failed to create PrivacyEngineIntegration: {e:?}");
            eprintln!("Error message: {e}");
            let mut source = e.source();
            let mut level = 0;
            while let Some(err) = source {
                eprintln!("  Error source level {level}: {err}");
                source = err.source();
                level += 1;
            }
            eprintln!("=== END PrivacyEngineIntegration::new ERROR ===\n");
            panic!("Failed to create PrivacyEngineIntegration: {e}");
        }
    };

    // Create and obfuscate the circuit first
    let circuit = integration.create_demo_circuit();
    println!(
        "Created demo circuit with {} inputs and {} outputs",
        circuit.num_input(),
        circuit.num_output()
    );

    println!("Obfuscating circuit...");
    let obfuscation_result = integration.obfuscate_circuit(circuit).await;
    if let Err(ref e) = obfuscation_result {
        eprintln!("Circuit obfuscation failed with error: {e:?}");
        panic!("Failed to obfuscate circuit: {e}");
    }
    println!("✓ Circuit obfuscation successful");

    let inputs = vec![true, false, true, true];
    println!("Testing circuit evaluation with inputs: {inputs:?}");

    let outputs = integration.execute_circuit_detailed(&inputs).await;

    // Print error details if the result is an error
    if let Err(ref e) = outputs {
        eprintln!("Circuit evaluation failed with error: {e:?}");
        eprintln!("Error message: {e}");
        eprintln!("Error chain:");
        let mut source = e.source();
        let mut level = 0;
        while let Some(err) = source {
            eprintln!("  {level}: {err}");
            source = err.source();
            level += 1;
        }
    } else {
        println!("✓ Circuit evaluation successful: {outputs:?}");
    }

    // More detailed assertion with full error information
    if let Err(ref e) = outputs {
        eprintln!("\n=== DETAILED ERROR ANALYSIS ===");
        eprintln!("Main error: {e}");
        eprintln!("Debug representation: {e:?}");

        // Check if it's an OpenFHE-related error
        let error_string = format!("{e:?}");
        if error_string.contains("OpenFHE") {
            eprintln!("This appears to be an OpenFHE-related error");
        }
        if error_string.contains("library") {
            eprintln!("This appears to be a library linking error");
        }
        if error_string.contains("symbol") {
            eprintln!("This appears to be a symbol resolution error");
        }

        eprintln!("=== END ERROR ANALYSIS ===\n");

        // Panic with detailed message
        panic!(
            "Circuit evaluation failed with error: {e}\nDebug: {e:?}\nFull error chain has been printed above."
        );
    }

    assert!(
        outputs.is_ok(),
        "Circuit evaluation failed: {:?}",
        outputs.as_ref().err()
    );

    let outputs = outputs.unwrap();
    assert!(outputs.success);
    assert!(!outputs.outputs.is_empty());
}

#[tokio::test]
async fn test_simple_circuit_operations() {
    let config = PrivacyEngineConfig::testing();
    let mut integration = PrivacyEngineIntegration::new(config).unwrap();

    // Set a unique obfuscation directory for this test
    integration.set_obfuscation_dir("test_simple_circuit_operations_obfuscation".to_string());

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
    let config = PrivacyEngineConfig::dummy();
    let integration = PrivacyEngineIntegration::new(config).unwrap();
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
