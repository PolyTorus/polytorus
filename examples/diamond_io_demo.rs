use num_bigint::BigUint;
use num_traits::Num;
use polytorus::diamond_io_integration::{DiamondIOConfig, DiamondIOIntegration};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=== Diamond IO Integration Demo ===");

    // Test different configurations
    println!("\n1. Testing Dummy Mode (Fast)");
    test_diamond_io_mode("Dummy", DiamondIOConfig::dummy()).await?;

    println!("\n2. Testing Testing Mode (Moderate)");
    test_diamond_io_mode("Testing", DiamondIOConfig::testing()).await?;

    println!("\n3. Testing Production Mode (Secure - using dummy for speed)");
    let mut production_config = DiamondIOConfig::production();
    production_config.dummy_mode = true; // Use dummy mode for demo speed
    test_diamond_io_mode("Production", production_config).await?;

    println!("\n4. Custom Configuration Test");
    let custom_config = DiamondIOConfig {
        ring_dimension: 32,
        crt_depth: 6,
        crt_bits: 32,
        base_bits: 5,
        switched_modulus: BigUint::from_str_radix("274877906943", 10).unwrap(),
        input_size: 4,
        level_width: 2,
        d: 3,
        hardcoded_key_sigma: 1.0,
        p_sigma: 1.0,
        trapdoor_sigma: Some(4.578),
        dummy_mode: true,
    };
    test_diamond_io_mode("Custom", custom_config).await?;

    println!("\n5. E2E Obfuscation and Evaluation Test");
    test_e2e_obfuscation_evaluation().await?;

    println!("\n6. Performance Comparison");
    test_performance_comparison().await?;

    println!("\n=== Demo Complete ===");
    Ok(())
}

async fn test_diamond_io_mode(mode_name: &str, config: DiamondIOConfig) -> anyhow::Result<()> {
    println!("Testing {} Mode:", mode_name);
    println!("  Ring dimension: {}", config.ring_dimension);
    println!("  CRT depth: {}", config.crt_depth);
    println!("  Input size: {}", config.input_size);
    println!("  Dummy mode: {}", config.dummy_mode);

    let integration = DiamondIOIntegration::new(config)?;
    let circuit = integration.create_demo_circuit();

    println!(
        "  Circuit created - Inputs: {}, Outputs: {}",
        circuit.num_input(),
        circuit.num_output()
    );

    // Test evaluation with sample inputs
    let inputs = vec![true, false, true, false];
    let truncated_inputs = &inputs[..std::cmp::min(inputs.len(), integration.config().input_size)];

    let start = std::time::Instant::now();
    match integration.evaluate_circuit(truncated_inputs).await {
        Ok(output) => {
            let elapsed = start.elapsed();
            println!("  Evaluation successful in {:?}", elapsed);
            println!("  Input: {:?}", truncated_inputs);
            println!("  Output: {:?}", output);
        }
        Err(e) => {
            println!("  Evaluation failed: {}", e);
        }
    }

    Ok(())
}

async fn test_e2e_obfuscation_evaluation() -> anyhow::Result<()> {
    println!("E2E Obfuscation and Evaluation Test:");

    let config = DiamondIOConfig::dummy(); // Use dummy mode for speed
    let integration = DiamondIOIntegration::new(config)?;

    // Create a demo circuit
    let circuit = integration.create_demo_circuit();
    println!(
        "  Created circuit with {} inputs and {} outputs",
        circuit.num_input(),
        circuit.num_output()
    );

    // Obfuscate the circuit
    println!("  Starting obfuscation...");
    let obf_start = std::time::Instant::now();
    match integration.obfuscate_circuit(circuit).await {
        Ok(_) => {
            let obf_time = obf_start.elapsed();
            println!("  Obfuscation completed in {:?}", obf_time);

            // Evaluate the obfuscated circuit
            println!("  Starting evaluation...");
            let inputs = vec![true, false];
            let eval_start = std::time::Instant::now();

            match integration.evaluate_circuit(&inputs).await {
                Ok(outputs) => {
                    let eval_time = eval_start.elapsed();
                    println!("  Evaluation completed in {:?}", eval_time);
                    println!("  Total time: {:?}", obf_time + eval_time);
                    println!("  Input: {:?}", inputs);
                    println!("  Output: {:?}", outputs);
                }
                Err(e) => {
                    println!("  Evaluation failed: {}", e);
                }
            }
        }
        Err(e) => {
            println!("  Obfuscation failed: {}", e);
        }
    }

    Ok(())
}

async fn test_performance_comparison() -> anyhow::Result<()> {
    println!("Performance Comparison:");

    let configs = vec![
        ("Dummy (16)", DiamondIOConfig::dummy()),
        (
            "Testing (128)",
            DiamondIOConfig {
                ring_dimension: 128,
                crt_depth: 8,
                crt_bits: 35,
                base_bits: 6,
                switched_modulus: BigUint::from_str_radix("549755813887", 10).unwrap(),
                input_size: 16,
                level_width: 4,
                d: 4,
                hardcoded_key_sigma: 2.0,
                p_sigma: 2.0,
                trapdoor_sigma: Some(4.578),
                dummy_mode: true, // Use dummy mode for demo
            },
        ),
    ];

    for (name, config) in configs {
        println!("  Testing {}", name);
        let integration = DiamondIOIntegration::new(config)?;
        let circuit = integration.create_demo_circuit();

        // Measure obfuscation time
        let obf_start = std::time::Instant::now();
        let _ = integration.obfuscate_circuit(circuit).await;
        let obf_time = obf_start.elapsed();

        // Measure evaluation time
        let inputs = vec![true, false];
        let eval_start = std::time::Instant::now();
        let _ = integration.evaluate_circuit(&inputs).await;
        let eval_time = eval_start.elapsed();

        println!("    Obfuscation: {:?}", obf_time);
        println!("    Evaluation: {:?}", eval_time);
        println!("    Total: {:?}", obf_time + eval_time);
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_demo_functionality() {
        let config = DiamondIOConfig::dummy();
        let integration = DiamondIOIntegration::new(config).unwrap();

        let circuit = integration.create_demo_circuit();
        assert!(circuit.num_input() > 0);
        assert!(circuit.num_output() > 0);

        let inputs = vec![true, false];
        let result = integration.evaluate_circuit(&inputs).await;
        assert!(result.is_ok());
    }
}
