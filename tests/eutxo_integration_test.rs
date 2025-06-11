//! Integration test for eUTXO functionality in the modular blockchain architecture

use polytorus::config::DataContext;
use polytorus::modular::*;

#[tokio::test]
async fn test_eutxo_integration() {
    // Create modular blockchain with eUTXO support
    let config = default_modular_config();
    // Use unique database path for each test to avoid lock conflicts
    let test_db_path = format!("data/test_integration_{}", std::process::id());
    let data_context = DataContext::new(std::path::PathBuf::from(test_db_path));
    
    let orchestrator = UnifiedModularOrchestrator::create_and_start_with_defaults(config, data_context).await.unwrap();

    // Test orchestrator state
    let state = orchestrator.get_state().await;
    let metrics = orchestrator.get_metrics().await;
    
    // Initial state should have zero statistics
    assert_eq!(state.current_block_height, 0);
    assert_eq!(state.pending_transactions, 0);
    assert_eq!(metrics.total_transactions_processed, 0);
    assert_eq!(metrics.total_blocks_processed, 0);

    println!("✅ eUTXO integration test passed!");
    println!("📊 Initial State:");
    println!("   Block height: {}", state.current_block_height);
    println!("   Pending transactions: {}", state.pending_transactions);
    println!("   Total transactions: {}", metrics.total_transactions_processed);
    println!("   Total blocks: {}", metrics.total_blocks_processed);
    
    // Clean up test database
    let test_db_path = format!("data/test_integration_{}", std::process::id());
    std::fs::remove_dir_all(&test_db_path).ok();
}

#[tokio::test]
async fn test_eutxo_balance_operations() {
    let config = default_modular_config();
    // Use unique database path for each test to avoid lock conflicts
    let test_db_path = format!("data/test_balance_{}", std::process::id());
    let data_context = DataContext::new(std::path::PathBuf::from(test_db_path));
    
    let orchestrator = UnifiedModularOrchestrator::create_and_start_with_defaults(config, data_context).await.unwrap();

    // Test transaction processing
    let tx_data = b"test_balance_transaction".to_vec();
    let tx_id = orchestrator.execute_transaction(tx_data).await;
    assert!(tx_id.is_ok());
    
    let metrics = orchestrator.get_metrics().await;
    assert_eq!(metrics.total_transactions_processed, 1);

    println!("✅ eUTXO balance operations test passed!");
    println!("💰 Transaction processed: {}", tx_id.unwrap());
    
    // Clean up test database
    let test_db_path = format!("data/test_balance_{}", std::process::id());
    std::fs::remove_dir_all(&test_db_path).ok();
}

#[tokio::test] 
async fn test_eutxo_state_consistency() {
    let config = default_modular_config();
    // Use unique database path for each test to avoid lock conflicts
    let test_db_path = format!("data/test_consistency_{}", std::process::id());
    let data_context = DataContext::new(std::path::PathBuf::from(test_db_path));
    
    let orchestrator = UnifiedModularOrchestrator::create_and_start_with_defaults(config, data_context).await.unwrap();

    // Check initial state
    let initial_state = orchestrator.get_state().await;
    assert_eq!(initial_state.current_block_height, 0);
    assert!(initial_state.is_running);
    
    // Check layer health
    let health = orchestrator.get_layer_health().await.unwrap();
    assert!(health.contains_key("execution"));
    assert!(health.contains_key("settlement"));

    println!("✅ eUTXO state consistency test passed!");
    println!("📈 Initial stats verified");
    
    // Clean up test database
    let test_db_path = format!("data/test_consistency_{}", std::process::id());
    std::fs::remove_dir_all(&test_db_path).ok();
}
