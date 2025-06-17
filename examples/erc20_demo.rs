//! ERC20 Demo
//!
//! This example demonstrates how to use ERC20 tokens in the PolyTorus blockchain

use polytorus::{
    config::DataContext,
    smart_contract::{ContractEngine, ContractState},
    Result,
};

#[tokio::main]
async fn main() -> Result<()> {
    println!("🚀 ERC20 Token Demo - PolyTorus Blockchain");
    println!("=========================================");

    // Initialize the contract engine
    println!("📦 Initializing contract engine...");
    let timestamp = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let temp_dir = format!("./data/demo_erc20_{timestamp}");
    let data_context = DataContext::new(std::path::PathBuf::from(&temp_dir));
    data_context.ensure_directories()?;
    let state = ContractState::new(&data_context.contracts_db_path)?;
    let engine = ContractEngine::new(state)?;

    // Deploy a sample ERC20 token
    println!("\n🔧 Deploying ERC20 token contract...");
    let contract_address = engine.deploy_erc20_contract(
        "PolyTorus Token".to_string(),
        "POLY".to_string(),
        18,
        1_000_000_000, // 1 billion tokens
        "alice".to_string(),
        "erc20_poly".to_string(),
    )?;

    println!("✅ Contract deployed at: {contract_address}");

    // Get contract information
    println!("\n📄 Contract Information:");
    if let Some((name, symbol, decimals, total_supply)) =
        engine.get_erc20_contract_info(&contract_address)?
    {
        println!("  Name: {name}");
        println!("  Symbol: {symbol}");
        println!("  Decimals: {decimals}");
        println!("  Total Supply: {total_supply} tokens");
    }

    // Check initial balance
    println!("\n💰 Initial Balances:");
    let alice_balance = engine.execute_erc20_contract(
        &contract_address,
        "balanceOf",
        "alice",
        vec!["alice".to_string()],
    )?;
    if alice_balance.success {
        let balance = String::from_utf8_lossy(&alice_balance.return_value);
        println!("  Alice: {balance} POLY");
    }

    let bob_balance = engine.execute_erc20_contract(
        &contract_address,
        "balanceOf",
        "bob",
        vec!["bob".to_string()],
    )?;
    if bob_balance.success {
        let balance = String::from_utf8_lossy(&bob_balance.return_value);
        println!("  Bob: {balance} POLY");
    }

    // Perform a transfer
    println!("\n🔄 Transferring 1000 POLY from Alice to Bob...");
    let transfer_result = engine.execute_erc20_contract(
        &contract_address,
        "transfer",
        "alice",
        vec!["bob".to_string(), "1000".to_string()],
    )?;

    if transfer_result.success {
        println!("✅ Transfer successful!");
        for log in &transfer_result.logs {
            println!("  📝 {log}");
        }
    } else {
        println!(
            "❌ Transfer failed: {}",
            String::from_utf8_lossy(&transfer_result.return_value)
        );
    }

    // Check balances after transfer
    println!("\n💰 Balances after transfer:");
    let alice_balance = engine.execute_erc20_contract(
        &contract_address,
        "balanceOf",
        "alice",
        vec!["alice".to_string()],
    )?;
    if alice_balance.success {
        let balance = String::from_utf8_lossy(&alice_balance.return_value);
        println!("  Alice: {balance} POLY");
    }

    let bob_balance = engine.execute_erc20_contract(
        &contract_address,
        "balanceOf",
        "bob",
        vec!["bob".to_string()],
    )?;
    if bob_balance.success {
        let balance = String::from_utf8_lossy(&bob_balance.return_value);
        println!("  Bob: {balance} POLY");
    }

    // Demonstrate approval and transferFrom
    println!("\n🔐 Setting up approval...");
    let approve_result = engine.execute_erc20_contract(
        &contract_address,
        "approve",
        "alice",
        vec!["charlie".to_string(), "500".to_string()],
    )?;

    if approve_result.success {
        println!("✅ Alice approved Charlie to spend 500 POLY");
        for log in &approve_result.logs {
            println!("  📝 {log}");
        }
    }

    // Check allowance
    let allowance_result = engine.execute_erc20_contract(
        &contract_address,
        "allowance",
        "alice",
        vec!["alice".to_string(), "charlie".to_string()],
    )?;
    if allowance_result.success {
        let allowance = String::from_utf8_lossy(&allowance_result.return_value);
        println!("  Allowance: {allowance} POLY");
    }

    // Charlie transfers from Alice to Bob
    println!("\n🔄 Charlie transferring 300 POLY from Alice to Bob...");
    let transfer_from_result = engine.execute_erc20_contract(
        &contract_address,
        "transferFrom",
        "charlie",
        vec!["alice".to_string(), "bob".to_string(), "300".to_string()],
    )?;

    if transfer_from_result.success {
        println!("✅ TransferFrom successful!");
        for log in &transfer_from_result.logs {
            println!("  📝 {log}");
        }
    } else {
        println!(
            "❌ TransferFrom failed: {}",
            String::from_utf8_lossy(&transfer_from_result.return_value)
        );
    }

    // Final balances
    println!("\n💰 Final Balances:");
    let alice_balance = engine.execute_erc20_contract(
        &contract_address,
        "balanceOf",
        "alice",
        vec!["alice".to_string()],
    )?;
    if alice_balance.success {
        let balance = String::from_utf8_lossy(&alice_balance.return_value);
        println!("  Alice: {balance} POLY");
    }

    let bob_balance = engine.execute_erc20_contract(
        &contract_address,
        "balanceOf",
        "bob",
        vec!["bob".to_string()],
    )?;
    if bob_balance.success {
        let balance = String::from_utf8_lossy(&bob_balance.return_value);
        println!("  Bob: {balance} POLY");
    }

    // Check remaining allowance
    let allowance_result = engine.execute_erc20_contract(
        &contract_address,
        "allowance",
        "alice",
        vec!["alice".to_string(), "charlie".to_string()],
    )?;
    if allowance_result.success {
        let allowance = String::from_utf8_lossy(&allowance_result.return_value);
        println!("  Remaining allowance for Charlie: {allowance} POLY");
    }

    // Deploy another token to demonstrate multiple contracts
    println!("\n🔧 Deploying second ERC20 token...");
    let contract2_address = engine.deploy_erc20_contract(
        "Utility Token".to_string(),
        "UTIL".to_string(),
        8,          // Different decimals
        10_000_000, // 10 million tokens
        "dave".to_string(),
        "erc20_util".to_string(),
    )?;

    println!("✅ Second contract deployed at: {contract2_address}");

    // List all ERC20 contracts
    println!("\n📋 All deployed ERC20 contracts:");
    let contracts = engine.list_erc20_contracts()?;
    for (i, addr) in contracts.iter().enumerate() {
        println!("  {}. {}", i + 1, addr);
        if let Some((name, symbol, decimals, total_supply)) =
            engine.get_erc20_contract_info(addr)?
        {
            println!("     {name} ({symbol}) - {decimals} decimals, {total_supply} total supply");
        }
    }

    println!("\n🎉 ERC20 Demo completed successfully!");
    Ok(())
}
