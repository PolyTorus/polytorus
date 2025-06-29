use polytorus::diamond_io_integration_new::{PrivacyEngineConfig, PrivacyEngineIntegration};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=== Diamond IO Integration Demo ===");

    // Test different configurations
    println!("\n1. Testing Dummy Mode (Fast)");
    test_diamond_io_mode("Dummy", PrivacyEngineConfig::dummy()).await?;

    println!("\n2. Testing Testing Mode (Moderate)");
    test_diamond_io_mode("Testing", PrivacyEngineConfig::testing()).await?;

    println!("\n3. Testing Production Mode (Secure)");
    test_diamond_io_mode("Production", PrivacyEngineConfig::production()).await?;

    println!("\n4. E2E Obfuscation and Evaluation Test");
    test_e2e_obfuscation_evaluation().await?;

    println!("\n5. Performance Comparison");
    test_performance_comparison().await?;

    println!("\n=== Demo Complete ===");
    Ok(())
}

async fn test_diamond_io_mode(mode_name: &str, config: PrivacyEngineConfig) -> anyhow::Result<()> {
    println!("Testing {mode_name} Mode:");
    println!("  Ring dimension: {}", config.ring_dimension);
    println!("  CRT depth: {}", config.crt_depth);
    println!("  Base bits: {}", config.base_bits);
    println!("  Dummy mode: {}", config.dummy_mode);

    let integration = PrivacyEngineIntegration::new(config)?;
    let circuit = integration.create_demo_circuit();

    println!(
        "  Circuit created - Inputs: {}, Outputs: {}",
        circuit.num_input(),
        circuit.num_output()
    );

    // Test evaluation with sample inputs
    let inputs = [true, false, true, false];
    let truncated_inputs = &inputs[..std::cmp::min(inputs.len(), integration.config().input_size)];

    let start = std::time::Instant::now();
    match integration.execute_circuit_detailed(truncated_inputs).await {
        Ok(output) => {
            let elapsed = start.elapsed();
            println!("  Evaluation successful in {elapsed:?}");
            println!("  Output length: {}", output.outputs.len());
            println!("  Execution time: {}ms", output.execution_time_ms);
        }
        Err(e) => {
            println!("  Evaluation failed: {e}");
        }
    }

    Ok(())
}

async fn test_e2e_obfuscation_evaluation() -> anyhow::Result<()> {
    println!("Testing End-to-End Obfuscation and Evaluation:");

    let config = PrivacyEngineConfig::testing();
    let integration = PrivacyEngineIntegration::new(config)?;
    let circuit = integration.create_demo_circuit();

    println!(
        "  Circuit: {} inputs, {} outputs",
        circuit.num_input(),
        circuit.num_output()
    );

    // Test obfuscation
    let obf_start = std::time::Instant::now();
    match integration.obfuscate_circuit(circuit).await {
        Ok(_result) => {
            let obf_elapsed = obf_start.elapsed();
            println!("  Obfuscation successful in {obf_elapsed:?}");

            // Test evaluation after obfuscation
            let inputs = vec![true, false, true, true];
            let eval_start = std::time::Instant::now();

            match integration.execute_circuit_detailed(&inputs).await {
                Ok(eval_result) => {
                    let eval_elapsed = eval_start.elapsed();
                    println!("  Evaluation successful in {eval_elapsed:?}");
                    println!("  Evaluation outputs: {:?}", eval_result.outputs);
                    println!(
                        "  Evaluation execution time: {}ms",
                        eval_result.execution_time_ms
                    );
                }
                Err(e) => {
                    println!("  Evaluation failed: {e}");
                }
            }
        }
        Err(e) => {
            println!("  Obfuscation failed: {e}");
        }
    }

    Ok(())
}

async fn test_performance_comparison() -> anyhow::Result<()> {
    println!("Performance Comparison:");

    let configs = [
        ("Dummy Mode", PrivacyEngineConfig::dummy()),
        ("Testing Mode", PrivacyEngineConfig::testing()),
        ("Production Mode", PrivacyEngineConfig::production()),
    ];

    for (name, config) in configs {
        let integration = PrivacyEngineIntegration::new(config)?;
        let circuit = integration.create_demo_circuit();

        let start = std::time::Instant::now();

        // Run multiple operations
        for _ in 0..3 {
            let _ = integration.obfuscate_circuit(circuit.clone()).await;
        }

        let elapsed = start.elapsed();
        println!("  {} avg time: {:?}", name, elapsed / 3);
    }

    // Test with different input sizes
    println!("\nDifferent Input Size Performance:");
    for input_size in [2, 4, 8] {
        let config = PrivacyEngineConfig::testing();
        let integration = PrivacyEngineIntegration::new(config)?;

        let inputs = vec![true; input_size];
        let start = std::time::Instant::now();

        match integration.execute_circuit_detailed(&inputs).await {
            Ok(_) => {
                let elapsed = start.elapsed();
                println!("  {input_size} inputs: {elapsed:?}");
            }
            Err(e) => {
                println!("  {input_size} inputs failed: {e}");
            }
        }
    }

    Ok(())
}
