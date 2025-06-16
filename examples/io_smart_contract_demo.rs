use anyhow::Result;
use polytorus::{
    diamond_io_integration_new::DiamondIOConfig,
    diamond_smart_contracts::DiamondContractEngine,
};

#[tokio::main]
async fn main() -> Result<()> {
    // トレーシングを初期化（一度だけ）
    tracing_subscriber::fmt::init();

    println!("=== Diamond IO Smart Contract iO Test ===\n");

    // 1. Create contract engine in dummy mode
    let dummy_config = DiamondIOConfig::dummy();
    println!("1. Contract Engine Test in Dummy Mode");
    let mut engine = DiamondContractEngine::new(dummy_config)?;

    // 2. Deploy AND gate contract
    let contract_id = engine
        .deploy_contract(
            "test_and_io".to_string(),
            "iO AND Gate".to_string(),
            "and_gate".to_string(),
            "test_user".to_string(),
            "and_gate",
        )
        .await?;

    println!("  Contract '{contract_id}' deployed");

    // 3. Obfuscate contract
    println!("  Obfuscating contract...");
    engine.obfuscate_contract(&contract_id).await?;

    let contract = engine.get_contract(&contract_id).unwrap();
    println!("  Obfuscation status: {}", contract.is_obfuscated);

    // 4. Execute obfuscated contract
    println!("  Executing obfuscated contract...");
    let inputs = vec![true, true, false, false, false, false, false, false];
    let result = engine
        .execute_contract(&contract_id, inputs.clone(), "executor".to_string())
        .await?;

    println!("  Input: {:?}", &inputs[0..2]);
    println!("  Output: {result:?}");
    println!("  AND(true, true) = {} (expected: true)", result[0]);

    // 5. Actual iO usage in test mode
    println!("\n2. Actual iO Usage in Test Mode");
    let testing_config = DiamondIOConfig::testing();
    let mut testing_engine = DiamondContractEngine::new(testing_config)?;

    let test_contract_id = testing_engine
        .deploy_contract(
            "test_xor_io".to_string(),
            "iO XOR Gate".to_string(),
            "xor_gate".to_string(),
            "test_user".to_string(),
            "xor_gate",
        )
        .await?;

    println!("  Test contract '{test_contract_id}' deployed");

    // Obfuscate in test mode
    println!("  Obfuscating contract in test mode...");
    testing_engine.obfuscate_contract(&test_contract_id).await?;

    // Execute in test mode
    let test_inputs = vec![
        true, false, false, false, false, false, false, false, false, false, false, false, false,
        false, false, false,
    ];
    let test_result = testing_engine
        .execute_contract(
            &test_contract_id,
            test_inputs.clone(),
            "test_executor".to_string(),
        )
        .await?;

    println!("  Input: {:?}", &test_inputs[0..2]);
    println!("  Output: {test_result:?}");
    println!("  XOR(true, false) = {} (expected: true)", test_result[0]);

    // 6. Check execution history
    let history = engine.get_execution_history(&contract_id);
    println!("\n3. Execution History:");
    println!("  Number of executions: {}", history.len());
    for (i, exec) in history.iter().enumerate() {
        println!(
            "  Execution {}: gas used = {}, execution time = {:?}ms",
            i + 1,
            exec.gas_used,
            exec.execution_time.unwrap_or(0)
        );
    }

    println!("\n=== iO Test Complete ===");
    println!("Successfully deployed, obfuscated, and executed smart contracts");
    println!("using Diamond IO's iO (indistinguishability obfuscation)!");

    Ok(())
}
