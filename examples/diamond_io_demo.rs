use polytorus::diamond_io_integration::{DiamondIOIntegration, DiamondIOConfig};
use polytorus::diamond_smart_contracts::DiamondContractEngine;
use polytorus::modular::{DiamondIOLayerBuilder, DiamondLayerTrait};
use num_bigint::BigUint;
use num_traits::Num;
use tracing::{info, error};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();
    
    info!("🚀 Starting PolyTorus Diamond IO Integration Demo");

    // Demo with different parameter sets
    info!("\n📊 Running demos with different parameter configurations...\n");

    // Demo 1: Dummy mode (safe for testing)
    info!("🧪 Demo 1: Dummy Mode Configuration");
    let dummy_config = DiamondIOConfig::dummy();
    demo_configuration(&dummy_config, "Dummy Mode").await?;

    // Demo 2: Testing parameters (medium security)
    info!("\n🔬 Demo 2: Testing Parameters Configuration");
    let testing_config = DiamondIOConfig::testing();
    demo_configuration(&testing_config, "Testing Parameters").await?;

    // Demo 3: Production parameters (high security, might be slower)
    info!("\n🏭 Demo 3: Production Parameters Configuration");
    let production_config = DiamondIOConfig::production();
    demo_configuration(&production_config, "Production Parameters").await?;

    // Demo 4: Original configuration
    let diamond_config = DiamondIOConfig {
        ring_dimension: 16,
        crt_depth: 2,
        crt_bits: 17,
        base_bits: 1,
        switched_modulus: BigUint::from_str_radix("123456789012345", 10)?,
        input_size: 4,
        level_width: 4,
        d: 2,
        hardcoded_key_sigma: 4.578,
        p_sigma: 4.578,
        trapdoor_sigma: Some(4.578),
        inputs: vec![true, false, true, false],
        dummy_mode: true, // Enable dummy mode to avoid tracing conflicts
    };

    info!("📋 Diamond IO Configuration:");
    info!("  Ring Dimension: {}", diamond_config.ring_dimension);
    info!("  CRT Depth: {}", diamond_config.crt_depth);
    info!("  Input Size: {}", diamond_config.input_size);

    // Demo 1: Basic Diamond IO Integration
    info!("\n🔐 Demo 4: Basic Diamond IO Operations");
    demo_basic_diamond_io(diamond_config.clone()).await?;

    // Demo 2: Smart Contract Engine
    info!("\n📜 Demo 5: Diamond Smart Contract Engine");
    demo_smart_contracts(diamond_config.clone()).await?;

    // Demo 3: Modular Layer Integration
    info!("\n🏗️  Demo 6: Modular Layer Integration");
    demo_modular_layer(diamond_config.clone()).await?;

    info!("\n✅ All demos completed successfully!");

    Ok(())
}

async fn demo_configuration(config: &DiamondIOConfig, name: &str) -> anyhow::Result<()> {
    info!("🔧 Testing {} configuration:", name);
    info!("  Ring Dimension: {}", config.ring_dimension);
    info!("  CRT Depth: {}", config.crt_depth);
    info!("  Input Size: {}", config.input_size);
    info!("  Dummy Mode: {}", config.dummy_mode);

    // Create integration
    let integration = DiamondIOIntegration::new(config.clone())?;
    
    // Test circuit creation
    let circuit = integration.create_demo_circuit();
    info!("  ✓ Demo circuit created: {} inputs, {} outputs", 
          circuit.num_input(), circuit.num_output());
    
    // Test different circuit types
    let and_circuit = integration.create_circuit("and_gate");
    info!("  ✓ AND circuit created: {} inputs, {} outputs", 
          and_circuit.num_input(), and_circuit.num_output());
    
    // Test obfuscation (may take longer with real parameters)
    let start_time = std::time::Instant::now();
    match integration.obfuscate_circuit(circuit).await {
        Ok(_) => {
            let obf_time = start_time.elapsed();
            info!("  ✓ Circuit obfuscation completed in {:?}", obf_time);
        },
        Err(e) => {
            info!("  ⚠️  Circuit obfuscation failed (expected with some parameters): {}", e);
        }
    }
    
    // Test evaluation
    let inputs = &config.inputs[..std::cmp::min(config.inputs.len(), config.input_size)];
    match integration.evaluate_circuit(inputs) {
        Ok(output) => {
            info!("  ✓ Circuit evaluation: {:?} -> {:?}", inputs, output);
        },
        Err(e) => {
            info!("  ⚠️  Circuit evaluation failed: {}", e);
        }
    }
    
    info!("  {} configuration test completed!\n", name);
    Ok(())
}

async fn demo_basic_diamond_io(config: DiamondIOConfig) -> anyhow::Result<()> {
    info!("Creating Diamond IO integration...");
    
    let integration = DiamondIOIntegration::new(config)?;
    
    // Create a demo circuit
    let circuit = integration.create_demo_circuit();
    info!("✓ Created demo circuit with {} inputs and {} outputs", 
        circuit.num_input(), 
        circuit.num_output()
    );

    // Encrypt some data
    let test_data = vec![true, false, true, false];
    info!("🔒 Encrypting data: {:?}", test_data);
    
    match integration.encrypt_data(&test_data) {
        Ok(_encrypted) => {
            info!("✓ Data encrypted successfully");
        }
        Err(e) => {
            error!("❌ Encryption failed: {}", e);
        }
    }

    // Note: Circuit obfuscation is commented out as it requires OpenFHE installation
    // info!("🔧 Obfuscating circuit...");
    // integration.obfuscate_circuit(circuit).await?;
    // info!("✓ Circuit obfuscated successfully");

    Ok(())
}

async fn demo_smart_contracts(config: DiamondIOConfig) -> anyhow::Result<()> {
    info!("Creating Diamond smart contract engine...");
    
    let mut engine = DiamondContractEngine::new(config)?;

    // Deploy different types of contracts
    let contracts = vec![
        ("and_gate", "AND Logic Gate", "and_gate"),
        ("or_gate", "OR Logic Gate", "or_gate"),
        ("xor_gate", "XOR Logic Gate", "xor_gate"),
        ("adder", "2-bit Adder", "adder"),
    ];

    for (id, name, description) in contracts {
        info!("📝 Deploying contract: {}", name);
        
        let contract_id = engine.deploy_contract(
            id.to_string(),
            name.to_string(),
            description.to_string(),
            "demo_user".to_string(),
            description,
        ).await?;
        
        info!("✓ Contract deployed with ID: {}", contract_id);

        // Test the contract
        let test_inputs = match description {
            "and_gate" | "or_gate" | "xor_gate" => vec![true, false, false, false],
            "adder" => vec![true, false, true, true], // 1 + 3 = 4 (binary: 01 + 11 = 100)
            _ => vec![true, false, false, false],
        };

        info!("🧪 Testing contract with inputs: {:?}", &test_inputs[..2.min(test_inputs.len())]);
        
        match engine.execute_contract(&contract_id, test_inputs, "test_user".to_string()).await {
            Ok(outputs) => {
                info!("✓ Contract execution successful, outputs: {:?}", outputs);
            }
            Err(e) => {
                error!("❌ Contract execution failed: {}", e);
            }
        }
    }

    // Show execution history
    let all_executions = engine.get_all_executions();
    info!("📊 Total executions: {}", all_executions.len());
    
    for execution in all_executions.iter().take(3) {
        info!("  - Contract: {}, Gas used: {}, Time: {:?}ms", 
            execution.contract_id,
            execution.gas_used,
            execution.execution_time.unwrap_or(0)
        );
    }

    Ok(())
}

async fn demo_modular_layer(config: DiamondIOConfig) -> anyhow::Result<()> {
    info!("Creating Diamond IO modular layer...");
    
    let mut layer = DiamondIOLayerBuilder::new()
        .with_diamond_config(config)
        .with_max_concurrent_executions(5)
        .with_obfuscation_enabled(false) // Disabled for demo
        .with_encryption_enabled(true)
        .with_gas_limit(1_000_000)
        .build()?;

    // Start the layer
    info!("🏗️  Starting Diamond IO layer...");
    layer.start_layer().await?;
    
    // Check health
    let is_healthy = layer.health_check().await?;
    info!("💚 Layer health check: {}", if is_healthy { "HEALTHY" } else { "UNHEALTHY" });

    // Deploy contracts through the layer
    info!("📝 Deploying contracts through modular layer...");
    
    let contract_id = layer.deploy_contract(
        "modular_and".to_string(),
        "Modular AND Gate".to_string(),
        "and_gate".to_string(),
        "modular_user".to_string(),
        "and_gate",
    ).await?;
    
    info!("✓ Contract deployed: {}", contract_id);

    // Execute the contract
    info!("🧪 Executing contract through modular layer...");
    let result = layer.execute_contract(
        &contract_id,
        vec![true, true, false, false],
        "test_executor".to_string(),
    ).await?;
    
    info!("✓ Execution result: {:?}", result);

    // Get layer statistics
    let stats = layer.get_stats().await;
    info!("📊 Layer Statistics:");
    info!("  - Total contracts: {}", stats.total_contracts);
    info!("  - Total executions: {}", stats.total_executions);
    info!("  - Successful executions: {}", stats.successful_executions);
    info!("  - Failed executions: {}", stats.failed_executions);
    info!("  - Average execution time: {}ms", stats.average_execution_time_ms);

    // List all contracts
    let contracts = layer.list_contracts().await;
    info!("📋 Deployed contracts:");
    for contract in contracts {
        info!("  - {}: {} (obfuscated: {})", 
            contract.id, 
            contract.name, 
            contract.is_obfuscated
        );
    }

    // Test encryption
    info!("🔒 Testing data encryption through layer...");
    let test_data = vec![true, false, true, true];
    match layer.encrypt_data(test_data.clone()).await {
        Ok(_encrypted_data) => {
            info!("✓ Data encrypted successfully");
        }
        Err(e) => {
            error!("❌ Encryption failed: {}", e);
        }
    }

    // Stop the layer
    info!("🛑 Stopping Diamond IO layer...");
    layer.stop_layer().await?;
    
    Ok(())
}
