//! Integration test for eUTXO functionality in the modular blockchain architecture

use polytorus::config::DataContext;
use polytorus::modular::*;

#[test]
fn test_eutxo_integration() {
    // Create modular blockchain with eUTXO support
    let config = default_modular_config();
    // Use unique database path for each test to avoid lock conflicts
    let test_db_path = format!("data/test_integration_{}", std::process::id());
    let data_context = DataContext::new(std::path::PathBuf::from(test_db_path));
    
    let blockchain = ModularBlockchainBuilder::new()
        .with_config(config)
        .with_data_context(data_context)
        .build()
        .unwrap();

    // Test state info includes eUTXO statistics
    let state_info = blockchain.get_state_info().unwrap();
    
    // Initial state should have zero eUTXO statistics
    assert_eq!(state_info.eutxo_stats.total_utxos, 0);
    assert_eq!(state_info.eutxo_stats.unspent_utxos, 0);
    assert_eq!(state_info.eutxo_stats.total_value, 0);
    assert_eq!(state_info.eutxo_stats.eutxo_count, 0);

    println!("✅ eUTXO integration test passed!");
    println!("📊 Initial eUTXO Stats:");
    println!("   Total UTXOs: {}", state_info.eutxo_stats.total_utxos);
    println!("   Unspent UTXOs: {}", state_info.eutxo_stats.unspent_utxos);
    println!("   Total value: {}", state_info.eutxo_stats.total_value);
    println!("   eUTXO transactions: {}", state_info.eutxo_stats.eutxo_count);
    
    // Clean up test database
    let test_db_path = format!("data/test_integration_{}", std::process::id());
    std::fs::remove_dir_all(&test_db_path).ok();
}

#[test]
fn test_eutxo_balance_operations() {
    let config = default_modular_config();
    // Use unique database path for each test to avoid lock conflicts
    let test_db_path = format!("data/test_balance_{}", std::process::id());
    let data_context = DataContext::new(std::path::PathBuf::from(test_db_path));
    
    let blockchain = ModularBlockchainBuilder::new()
        .with_config(config)
        .with_data_context(data_context)
        .build()
        .unwrap();

    let test_address = "test_address";
    
    // Initial balance should be zero
    let initial_balance = blockchain.get_eutxo_balance(test_address);
    assert!(initial_balance.is_ok());
    assert_eq!(initial_balance.unwrap(), 0);

    println!("✅ eUTXO balance operations test passed!");
    println!("💰 Initial balance: 0");
    
    // Clean up test database
    let test_db_path = format!("data/test_balance_{}", std::process::id());
    std::fs::remove_dir_all(&test_db_path).ok();
}

#[test] 
fn test_eutxo_state_consistency() {
    let config = default_modular_config();
    // Use unique database path for each test to avoid lock conflicts
    let test_db_path = format!("data/test_consistency_{}", std::process::id());
    let data_context = DataContext::new(std::path::PathBuf::from(test_db_path));
    
    let blockchain = ModularBlockchainBuilder::new()
        .with_config(config)
        .with_data_context(data_context)
        .build()
        .unwrap();

    // Check initial state
    let initial_state = blockchain.get_state_info().unwrap();
    assert_eq!(initial_state.eutxo_stats.total_utxos, 0);

    println!("✅ eUTXO state consistency test passed!");
    println!("📈 Initial stats verified");
    
    // Clean up test database
    let test_db_path = format!("data/test_consistency_{}", std::process::id());
    std::fs::remove_dir_all(&test_db_path).ok();
}
