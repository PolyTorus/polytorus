use polytorus::diamond_io_integration::{
    DiamondIOConfig,
    DiamondIOIntegration,
};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    println!("=== Diamond IO Performance Test ===");

    // Test different configurations for performance
    let configs = [
        ("Dummy Configuration", DiamondIOConfig::dummy()),
        ("Testing Configuration", DiamondIOConfig::testing()),
        ("Production Configuration", DiamondIOConfig::production()),
    ];

    for (name, config) in configs {
        println!("\n--- {} ---", name);
        test_performance(config).await?;
    }

    println!("\n=== Performance Test Complete ===");
    Ok(())
}

async fn test_performance(config: DiamondIOConfig) -> anyhow::Result<()> {
    let mut integration = DiamondIOIntegration::new(config)?;
    let circuit = integration.create_demo_circuit();

    // Test obfuscation performance
    let start = std::time::Instant::now();
    let result = integration.obfuscate_circuit(circuit).await?;
    let obfuscation_time = start.elapsed();

    println!("  Obfuscation time: {:?}", obfuscation_time);
    println!("  Success: {}", result.success);
    println!("  Execution time: {}ms", result.execution_time_ms);

    // Test evaluation performance
    let inputs = vec![true, false, true, false];
    let start = std::time::Instant::now();
    let eval_result = integration.evaluate_circuit(&inputs).await?;
    let evaluation_time = start.elapsed();

    println!("  Evaluation time: {:?}", evaluation_time);
    println!("  Evaluation success: {}", eval_result.success);
    println!("  Output count: {}", eval_result.outputs.len());

    Ok(())
}
