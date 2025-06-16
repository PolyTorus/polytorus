//! ERC20 integration tests
//!
//! Tests for ERC20 token functionality integration with the blockchain

use std::time::{
    SystemTime,
    UNIX_EPOCH,
};

use polytorus::config::DataContext;
use polytorus::smart_contract::{
    ContractEngine,
    ContractState,
    ERC20Contract,
};
use polytorus::Result;

#[tokio::test]
async fn test_erc20_full_workflow() -> Result<()> {
    // Initialize the contract engine with a temporary directory
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let temp_dir = format!("./data/test_erc20_full_{}", timestamp);
    let data_context = DataContext::new(std::path::PathBuf::from(&temp_dir));
    data_context.ensure_directories()?;
    let state = ContractState::new(&data_context.contracts_db_path)?;
    let engine = ContractEngine::new(state)?;

    // Deploy an ERC20 contract
    let contract_address = engine.deploy_erc20_contract(
        "Test Token".to_string(),
        "TEST".to_string(),
        18,
        1000000,
        "alice".to_string(),
        "erc20_test".to_string(),
    )?;

    println!("Deployed ERC20 contract at: {}", contract_address);

    // Test contract info
    let info = engine.get_erc20_contract_info(&contract_address)?;
    assert!(info.is_some());
    let (name, symbol, decimals, total_supply) = info.unwrap();
    assert_eq!(name, "Test Token");
    assert_eq!(symbol, "TEST");
    assert_eq!(decimals, 18);
    assert_eq!(total_supply, 1000000);

    // Check initial balance
    let balance_result = engine.execute_erc20_contract(
        &contract_address,
        "balanceOf",
        "alice",
        vec!["alice".to_string()],
    )?;
    assert!(balance_result.success);
    let balance_str = String::from_utf8(balance_result.return_value)?;
    assert_eq!(balance_str, "1000000");

    // Test transfer
    let transfer_result = engine.execute_erc20_contract(
        &contract_address,
        "transfer",
        "alice",
        vec!["bob".to_string(), "100".to_string()],
    )?;
    assert!(transfer_result.success);

    // Check balances after transfer
    let alice_balance = engine.execute_erc20_contract(
        &contract_address,
        "balanceOf",
        "alice",
        vec!["alice".to_string()],
    )?;
    assert!(alice_balance.success);
    let alice_balance_str = String::from_utf8(alice_balance.return_value)?;
    assert_eq!(alice_balance_str, "999900");

    let bob_balance = engine.execute_erc20_contract(
        &contract_address,
        "balanceOf",
        "bob",
        vec!["bob".to_string()],
    )?;
    assert!(bob_balance.success);
    let bob_balance_str = String::from_utf8(bob_balance.return_value)?;
    assert_eq!(bob_balance_str, "100");

    // Test approval
    let approve_result = engine.execute_erc20_contract(
        &contract_address,
        "approve",
        "alice",
        vec!["charlie".to_string(), "200".to_string()],
    )?;
    assert!(approve_result.success);

    // Check allowance
    let allowance_result = engine.execute_erc20_contract(
        &contract_address,
        "allowance",
        "alice",
        vec!["alice".to_string(), "charlie".to_string()],
    )?;
    assert!(allowance_result.success);
    let allowance_str = String::from_utf8(allowance_result.return_value)?;
    assert_eq!(allowance_str, "200");

    // Test transferFrom
    let transfer_from_result = engine.execute_erc20_contract(
        &contract_address,
        "transferFrom",
        "charlie",
        vec!["alice".to_string(), "bob".to_string(), "50".to_string()],
    )?;
    assert!(transfer_from_result.success);

    // Check final balances
    let alice_final = engine.execute_erc20_contract(
        &contract_address,
        "balanceOf",
        "alice",
        vec!["alice".to_string()],
    )?;
    let alice_final_str = String::from_utf8(alice_final.return_value)?;
    assert_eq!(alice_final_str, "999850"); // 1000000 - 100 - 50

    let bob_final = engine.execute_erc20_contract(
        &contract_address,
        "balanceOf",
        "bob",
        vec!["bob".to_string()],
    )?;
    let bob_final_str = String::from_utf8(bob_final.return_value)?;
    assert_eq!(bob_final_str, "150"); // 100 + 50

    println!("✅ All ERC20 tests passed!");
    Ok(())
}

#[tokio::test]
async fn test_erc20_error_cases() -> Result<()> {
    // Initialize the contract engine with a temporary directory
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let temp_dir = format!("./data/test_erc20_error_{}", timestamp);
    let data_context = DataContext::new(std::path::PathBuf::from(&temp_dir));
    data_context.ensure_directories()?;
    let state = ContractState::new(&data_context.contracts_db_path)?;
    let engine = ContractEngine::new(state)?;

    // Deploy an ERC20 contract
    let contract_address = engine.deploy_erc20_contract(
        "Test Token".to_string(),
        "TEST".to_string(),
        18,
        1000,
        "alice".to_string(),
        "erc20_error_test".to_string(),
    )?;

    // Test insufficient balance transfer
    let transfer_result = engine.execute_erc20_contract(
        &contract_address,
        "transfer",
        "alice",
        vec!["bob".to_string(), "2000".to_string()], // More than balance
    )?;
    assert!(!transfer_result.success);

    // Test insufficient allowance transferFrom
    let approve_result = engine.execute_erc20_contract(
        &contract_address,
        "approve",
        "alice",
        vec!["charlie".to_string(), "100".to_string()],
    )?;
    assert!(approve_result.success);

    let transfer_from_result = engine.execute_erc20_contract(
        &contract_address,
        "transferFrom",
        "charlie",
        vec!["alice".to_string(), "bob".to_string(), "200".to_string()], // More than allowance
    )?;
    assert!(!transfer_from_result.success);

    // Test invalid function call
    let invalid_result = engine.execute_erc20_contract(
        &contract_address,
        "nonexistent_function",
        "alice",
        vec![],
    )?;
    assert!(!invalid_result.success);

    println!("✅ All ERC20 error case tests passed!");
    Ok(())
}

#[tokio::test]
async fn test_multiple_erc20_contracts() -> Result<()> {
    // Initialize the contract engine with a temporary directory
    let timestamp = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap()
        .as_millis();
    let temp_dir = format!("./data/test_erc20_multi_{}", timestamp);
    let data_context = DataContext::new(std::path::PathBuf::from(&temp_dir));
    data_context.ensure_directories()?;
    let state = ContractState::new(&data_context.contracts_db_path)?;
    let engine = ContractEngine::new(state)?;

    // Deploy multiple ERC20 contracts
    let contract1 = engine.deploy_erc20_contract(
        "Token One".to_string(),
        "TOK1".to_string(),
        18,
        1000000,
        "alice".to_string(),
        "erc20_tok1".to_string(),
    )?;

    let contract2 = engine.deploy_erc20_contract(
        "Token Two".to_string(),
        "TOK2".to_string(),
        8,
        500000,
        "bob".to_string(),
        "erc20_tok2".to_string(),
    )?;

    // List contracts
    let contracts = engine.list_erc20_contracts()?;
    assert_eq!(contracts.len(), 2);
    assert!(contracts.contains(&contract1));
    assert!(contracts.contains(&contract2));

    // Test each contract independently
    let tok1_info = engine.get_erc20_contract_info(&contract1)?.unwrap();
    assert_eq!(tok1_info.0, "Token One");
    assert_eq!(tok1_info.1, "TOK1");

    let tok2_info = engine.get_erc20_contract_info(&contract2)?.unwrap();
    assert_eq!(tok2_info.0, "Token Two");
    assert_eq!(tok2_info.1, "TOK2");

    println!("✅ Multiple ERC20 contracts test passed!");
    Ok(())
}

#[test]
fn test_erc20_standalone() {
    let mut contract = ERC20Contract::new(
        "Standalone Test".to_string(),
        "STAND".to_string(),
        18,
        1000000,
        "owner".to_string(),
    );

    // Test basic operations
    assert_eq!(contract.name(), "Standalone Test");
    assert_eq!(contract.symbol(), "STAND");
    assert_eq!(contract.decimals(), 18);
    assert_eq!(contract.total_supply(), 1000000);
    assert_eq!(contract.balance_of("owner"), 1000000);

    // Test transfer
    let transfer_result = contract.transfer("owner", "user1", 100).unwrap();
    assert!(transfer_result.success);
    assert_eq!(contract.balance_of("owner"), 999900);
    assert_eq!(contract.balance_of("user1"), 100);

    // Test approve and transferFrom
    let approve_result = contract.approve("owner", "user2", 200).unwrap();
    assert!(approve_result.success);
    assert_eq!(contract.allowance("owner", "user2"), 200);

    let transfer_from_result = contract
        .transfer_from("user2", "owner", "user1", 50)
        .unwrap();
    assert!(transfer_from_result.success);
    assert_eq!(contract.balance_of("owner"), 999850);
    assert_eq!(contract.balance_of("user1"), 150);
    assert_eq!(contract.allowance("owner", "user2"), 150);

    // Test mint
    let mint_result = contract.mint("user3", 500).unwrap();
    assert!(mint_result.success);
    assert_eq!(contract.balance_of("user3"), 500);
    assert_eq!(contract.total_supply(), 1000500);

    // Test burn
    let burn_result = contract.burn("user3", 200).unwrap();
    assert!(burn_result.success);
    assert_eq!(contract.balance_of("user3"), 300);
    assert_eq!(contract.total_supply(), 1000300);

    println!("✅ Standalone ERC20 test passed!");
}
