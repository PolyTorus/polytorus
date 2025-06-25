//! Comprehensive integration tests for the unified smart contract engine
//!
//! These tests validate the complete integration between WASM engines, privacy engines,
//! storage systems, and advanced monitoring features.

use std::{collections::HashMap, sync::Arc, time::Duration};

use polytorus::{
    diamond_io_integration::PrivacyEngineConfig,
    smart_contract::{
        database_storage::{DatabaseContractStorage, DatabaseStorageConfig},
        enhanced_unified_engine::{
            DeploymentOptions, EnhancedEngineConfig, EnhancedUnifiedContractEngine,
            ExecutionOptions,
        },
        unified_engine::{
            ContractStateStorage, ContractType, UnifiedContractExecution, UnifiedContractMetadata,
            UnifiedGasConfig, UnifiedGasManager,
        },
        unified_manager::UnifiedContractManager,
        unified_storage::{SyncInMemoryContractStorage, UnifiedContractStorage},
    },
};
use tokio::time::timeout;

/// Test comprehensive unified engine functionality
#[tokio::test]
async fn test_comprehensive_unified_engine() {
    // Create enhanced engine with in-memory storage
    let storage = Arc::new(SyncInMemoryContractStorage::new());
    let gas_manager = UnifiedGasManager::new(UnifiedGasConfig::default());
    let privacy_config = PrivacyEngineConfig::dummy();
    let engine_config = EnhancedEngineConfig::default();

    let engine =
        EnhancedUnifiedContractEngine::new(storage, gas_manager, privacy_config, engine_config)
            .await
            .unwrap();

    // Test initial state
    let analytics = engine.get_analytics().await;
    assert_eq!(analytics.total_deployments, 0);
    assert_eq!(analytics.total_executions, 0);

    let metrics = engine.get_performance_metrics().await.unwrap();
    assert_eq!(metrics.total_executions, 0);
    assert_eq!(metrics.active_contracts, 0);

    // Test deployment with enhanced options
    let deployment_metadata = UnifiedContractMetadata {
        address: "0xenhanced123".to_string(),
        name: "Enhanced Test Contract".to_string(),
        description: "A test contract for enhanced engine".to_string(),
        contract_type: ContractType::BuiltIn {
            contract_name: "ERC20".to_string(),
            parameters: {
                let mut params = HashMap::new();
                params.insert("name".to_string(), "TestToken".to_string());
                params.insert("symbol".to_string(), "TTK".to_string());
                params.insert("decimals".to_string(), "18".to_string());
                params.insert("initial_supply".to_string(), "1000000".to_string());
                params
            },
        },
        deployment_tx: "0xdeploytx".to_string(),
        deployment_time: 1234567890,
        owner: "0xowner".to_string(),
        is_active: true,
    };

    let deployment_options = DeploymentOptions {
        validate_bytecode: true,
        enable_optimization: true,
        gas_limit: 5_000_000,
        deployment_metadata: HashMap::new(),
    };

    let deployment_result = engine
        .deploy_contract_enhanced(deployment_metadata, vec![], deployment_options)
        .await
        .unwrap();

    assert!(deployment_result.success);
    assert_eq!(deployment_result.contract_address, "0xenhanced123");
    assert!(deployment_result.optimization_applied);
    assert!(deployment_result.validation_passed);

    // Verify analytics were updated
    let analytics = engine.get_analytics().await;
    assert_eq!(analytics.total_deployments, 1);

    // Test enhanced execution
    let execution = UnifiedContractExecution {
        contract_address: "0xenhanced123".to_string(),
        function_name: "balance_of".to_string(),
        input_data: vec![0u8; 32], // 32 bytes for address parameter
        caller: "0xcaller".to_string(),
        value: 0,
        gas_limit: 100_000,
    };

    let execution_options = ExecutionOptions {
        use_cache: true,
        enable_tracing: false, // Disable for performance
        enable_optimization: true,
        timeout_ms: Some(10_000),
    };

    let execution_result = engine
        .execute_contract_enhanced(execution.clone(), execution_options)
        .await
        .unwrap();

    assert!(execution_result.basic_result.success);
    assert!(!execution_result.cache_hit); // First execution
    assert!(!execution_result.optimizations_applied.is_empty());
    assert!(execution_result.analytics_recorded);

    // Test cache functionality - execute same contract again
    let execution_options_cached = ExecutionOptions {
        use_cache: true,
        enable_tracing: false,
        enable_optimization: false, // Disable to test pure cache
        timeout_ms: Some(10_000),
    };

    let cached_result = engine
        .execute_contract_enhanced(execution.clone(), execution_options_cached)
        .await
        .unwrap();

    assert!(cached_result.basic_result.success);
    // Note: Cache may or may not hit depending on implementation details

    // Test performance metrics after execution
    let final_metrics = engine.get_performance_metrics().await.unwrap();
    assert!(final_metrics.total_executions > 0);
    assert_eq!(final_metrics.active_contracts, 1);

    // Test contract health report
    let health_report = engine.get_contract_health("0xenhanced123").await.unwrap();
    assert_eq!(health_report.contract_address, "0xenhanced123");
    assert!(health_report.health_score > 0.0);
    assert!(!health_report.recommendations.is_empty());

    // Test optimization report
    let optimization_report = engine.optimize_contracts().await.unwrap();
    // With minimal activity, should have few optimizations
    assert!(optimization_report.contracts_optimized <= 1);
}

/// Test database storage integration
#[tokio::test(flavor = "multi_thread")]
async fn test_database_storage_integration() {
    // Test database storage with memory fallback
    let db_config = DatabaseStorageConfig {
        postgres: None, // No actual database for tests
        redis: None,
        fallback_to_memory: true,
        connection_timeout_secs: 5,
        max_connections: 10,
        use_ssl: false,
    };

    let storage = DatabaseContractStorage::new(db_config).await.unwrap();
    let stats = storage.get_stats().await;

    // With fallback to memory and no real databases, should have zero connections
    assert_eq!(stats.postgres_connections, 0);
    assert_eq!(stats.redis_connections, 0);

    // Test storage operations through the database interface
    let metadata = UnifiedContractMetadata {
        address: "0xdbtest123".to_string(),
        name: "Database Test Contract".to_string(),
        description: "Testing database storage".to_string(),
        contract_type: ContractType::Wasm {
            bytecode: vec![1, 2, 3, 4, 5],
            abi: Some("test_abi".to_string()),
        },
        deployment_tx: "0xdbdeploy".to_string(),
        deployment_time: 1234567890,
        owner: "0xdbowner".to_string(),
        is_active: true,
    };

    // Test contract metadata storage
    storage.store_contract_metadata(&metadata).unwrap();
    let retrieved = storage.get_contract_metadata(&metadata.address).unwrap();
    assert!(retrieved.is_some());
    assert_eq!(retrieved.unwrap().name, metadata.name);

    // Test contract state operations
    storage
        .set_contract_state("0xdbtest123", "balance", &[1, 0, 0, 0, 0, 0, 0, 0])
        .unwrap();

    let balance = storage
        .get_contract_state("0xdbtest123", "balance")
        .unwrap();
    assert_eq!(balance, Some(vec![1, 0, 0, 0, 0, 0, 0, 0]));

    // Test contract listing
    let contracts = storage.list_contracts().unwrap();
    assert!(contracts.contains(&"0xdbtest123".to_string()));

    println!("Database storage integration test completed successfully");
}

/// Test unified contract manager with multiple engines
#[tokio::test]
async fn test_unified_manager_integration() {
    // Create unified manager with in-memory storage
    let manager = UnifiedContractManager::in_memory().unwrap();

    // Test initial state
    let engine_info = manager.get_engine_info().await;
    assert_eq!(engine_info.len(), 2); // WASM and Privacy engines

    let stats = manager.get_statistics().await.unwrap();
    assert_eq!(stats.total_contracts, 0);
    assert_eq!(stats.active_engines, 2);

    // Deploy ERC20 contract
    let erc20_address = manager
        .deploy_erc20(
            "Integration Token".to_string(),
            "ITK".to_string(),
            18,
            2_000_000,
            "0xintegration_owner".to_string(),
            "0xerc20_integration".to_string(),
        )
        .await
        .unwrap();

    assert_eq!(erc20_address, "0xerc20_integration");

    // Deploy privacy contract
    let privacy_address = manager
        .deploy_privacy_contract(
            "Privacy Integration Contract".to_string(),
            "Testing privacy engine integration".to_string(),
            "integration_circuit".to_string(),
            "0xprivacy_owner".to_string(),
            "0xprivacy_integration".to_string(),
            b"integration circuit description".to_vec(),
        )
        .await
        .unwrap();

    assert_eq!(privacy_address, "0xprivacy_integration");

    // Test contract execution - ERC20 balance query
    let erc20_execution = UnifiedContractExecution {
        contract_address: erc20_address.clone(),
        function_name: "balance_of".to_string(),
        input_data: {
            let mut data = vec![0u8; 32];
            data[..19].copy_from_slice(b"0xintegration_owner");
            data
        },
        caller: "0xquery_caller".to_string(),
        value: 0,
        gas_limit: 50_000,
    };

    let erc20_result = manager.execute_contract(erc20_execution).await.unwrap();
    assert!(erc20_result.success);
    assert!(!erc20_result.return_data.is_empty());

    // Test contract execution - Privacy contract info
    let privacy_execution = UnifiedContractExecution {
        contract_address: privacy_address.clone(),
        function_name: "get_info".to_string(),
        input_data: vec![],
        caller: "0xprivacy_caller".to_string(),
        value: 0,
        gas_limit: 100_000,
    };

    let privacy_result = manager.execute_contract(privacy_execution).await.unwrap();
    assert!(privacy_result.success);
    assert!(!privacy_result.return_data.is_empty());

    // Test gas estimation
    let gas_estimation_exec = UnifiedContractExecution {
        contract_address: erc20_address.clone(),
        function_name: "transfer".to_string(),
        input_data: vec![0u8; 64], // to address + amount
        caller: "0xgas_caller".to_string(),
        value: 0,
        gas_limit: 200_000,
    };

    let estimated_gas = manager.estimate_gas(&gas_estimation_exec).await.unwrap();
    assert!(estimated_gas > 0);
    assert!(estimated_gas < 200_000); // Should be reasonable

    // Test contract listing
    let all_contracts = manager.list_contracts().await.unwrap();
    assert_eq!(all_contracts.len(), 2);
    assert!(all_contracts.contains(&erc20_address));
    assert!(all_contracts.contains(&privacy_address));

    // Test listing by type
    let builtin_contracts = manager.list_contracts_by_type("builtin").await.unwrap();
    assert_eq!(builtin_contracts.len(), 1);
    assert!(builtin_contracts.contains(&erc20_address));

    let privacy_contracts = manager.list_contracts_by_type("privacy").await.unwrap();
    assert_eq!(privacy_contracts.len(), 1);
    assert!(privacy_contracts.contains(&privacy_address));

    // Test execution history
    let erc20_history = manager.get_execution_history(&erc20_address).await.unwrap();
    assert!(!erc20_history.is_empty());

    let privacy_history = manager
        .get_execution_history(&privacy_address)
        .await
        .unwrap();
    assert!(!privacy_history.is_empty());

    // Test final statistics
    let final_stats = manager.get_statistics().await.unwrap();
    assert_eq!(final_stats.total_contracts, 2);
    assert_eq!(final_stats.builtin_contracts, 1);
    assert_eq!(final_stats.privacy_contracts, 1);

    println!("Unified manager integration test completed successfully");
}

/// Test storage persistence and recovery
#[tokio::test(flavor = "multi_thread")]
async fn test_storage_persistence() {
    let temp_dir = tempfile::tempdir().unwrap();
    let storage_path = temp_dir.path().join("test_persistence.db");

    // Create storage and store some data
    {
        let storage = UnifiedContractStorage::new(&storage_path).unwrap();

        let metadata = UnifiedContractMetadata {
            address: "0xpersistent123".to_string(),
            name: "Persistent Contract".to_string(),
            description: "Testing persistence".to_string(),
            contract_type: ContractType::BuiltIn {
                contract_name: "TestContract".to_string(),
                parameters: HashMap::new(),
            },
            deployment_tx: "0xpersistenttx".to_string(),
            deployment_time: 1234567890,
            owner: "0xpersistentowner".to_string(),
            is_active: true,
        };

        storage.store_contract_metadata(&metadata).unwrap();
        storage
            .set_contract_state("0xpersistent123", "test_key", b"test_value")
            .unwrap();
        storage.flush().unwrap();
    }

    // Recreate storage and verify data persists
    {
        let storage = UnifiedContractStorage::new(&storage_path).unwrap();

        let retrieved = storage.get_contract_metadata("0xpersistent123").unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().name, "Persistent Contract");

        let state_value = storage
            .get_contract_state("0xpersistent123", "test_key")
            .unwrap();
        assert_eq!(state_value, Some(b"test_value".to_vec()));

        let contracts = storage.list_contracts().unwrap();
        assert!(contracts.contains(&"0xpersistent123".to_string()));
    }

    println!("Storage persistence test completed successfully");
}

/// Test concurrent operations and thread safety
#[tokio::test]
async fn test_concurrent_operations() {
    let manager = Arc::new(UnifiedContractManager::in_memory().unwrap());
    let mut handles = vec![];

    // Deploy multiple contracts concurrently
    for i in 0..5 {
        let manager_clone = Arc::clone(&manager);
        let handle = tokio::spawn(async move {
            let address = format!("0xconcurrent{i:03}");
            let result = manager_clone
                .deploy_erc20(
                    format!("Concurrent Token {i}"),
                    format!("CT{i}"),
                    18,
                    1_000_000,
                    "0xconcurrent_owner".to_string(),
                    address.clone(),
                )
                .await;
            (i, address, result)
        });
        handles.push(handle);
    }

    // Wait for all deployments to complete
    let results = futures::future::join_all(handles).await;

    for result in results {
        let (i, expected_address, deploy_result) = result.unwrap();
        assert!(
            deploy_result.is_ok(),
            "Deployment {i} failed: {deploy_result:?}"
        );
        assert_eq!(deploy_result.unwrap(), expected_address);
    }

    // Execute contracts concurrently
    let mut execution_handles = vec![];
    for i in 0..5 {
        let manager_clone = Arc::clone(&manager);
        let handle = tokio::spawn(async move {
            let address = format!("0xconcurrent{i:03}");
            let execution = UnifiedContractExecution {
                contract_address: address,
                function_name: "balance_of".to_string(),
                input_data: vec![0u8; 32],
                caller: format!("0xcaller{i}"),
                value: 0,
                gas_limit: 50_000,
            };

            manager_clone.execute_contract(execution).await
        });
        execution_handles.push(handle);
    }

    // Wait for all executions
    let execution_results = futures::future::join_all(execution_handles).await;

    for (i, result) in execution_results.into_iter().enumerate() {
        let exec_result = result.unwrap();
        assert!(exec_result.is_ok(), "Execution {i} failed: {exec_result:?}");
        assert!(exec_result.unwrap().success);
    }

    // Verify final state
    let contracts = manager.list_contracts().await.unwrap();
    assert_eq!(contracts.len(), 5);

    let stats = manager.get_statistics().await.unwrap();
    assert_eq!(stats.total_contracts, 5);
    assert_eq!(stats.builtin_contracts, 5);

    println!("Concurrent operations test completed successfully");
}

/// Test error handling and recovery
#[tokio::test]
async fn test_error_handling() {
    let manager = UnifiedContractManager::in_memory().unwrap();

    // Test execution on non-existent contract
    let invalid_execution = UnifiedContractExecution {
        contract_address: "0xnonexistent".to_string(),
        function_name: "some_function".to_string(),
        input_data: vec![],
        caller: "0xcaller".to_string(),
        value: 0,
        gas_limit: 50_000,
    };

    let result = manager.execute_contract(invalid_execution).await;
    assert!(result.is_err());

    // Test gas estimation on non-existent contract
    let invalid_gas_estimation = UnifiedContractExecution {
        contract_address: "0xnonexistent".to_string(),
        function_name: "test".to_string(),
        input_data: vec![],
        caller: "0xcaller".to_string(),
        value: 0,
        gas_limit: 50_000,
    };

    let gas_result = manager.estimate_gas(&invalid_gas_estimation).await;
    // Should fallback to base gas calculation
    assert!(gas_result.is_ok());
    assert!(gas_result.unwrap() > 0);

    // Test contract metadata retrieval for non-existent contract
    let metadata_result = manager.get_contract("0xnonexistent").await.unwrap();
    assert!(metadata_result.is_none());

    // Test execution history for non-existent contract
    let history_result = manager
        .get_execution_history("0xnonexistent")
        .await
        .unwrap();
    assert!(history_result.is_empty());

    println!("Error handling test completed successfully");
}

/// Test performance under load
#[tokio::test]
async fn test_performance_under_load() {
    let manager = Arc::new(UnifiedContractManager::in_memory().unwrap());

    // Deploy a test contract
    let contract_address = manager
        .deploy_erc20(
            "Load Test Token".to_string(),
            "LTT".to_string(),
            18,
            10_000_000,
            "0xload_owner".to_string(),
            "0xload_test".to_string(),
        )
        .await
        .unwrap();

    let start_time = std::time::Instant::now();
    let num_operations = 100;
    let mut handles = vec![];

    // Execute many operations concurrently
    for i in 0..num_operations {
        let manager_clone = Arc::clone(&manager);
        let address = contract_address.clone();
        let handle = tokio::spawn(async move {
            let execution = UnifiedContractExecution {
                contract_address: address,
                function_name: "balance_of".to_string(),
                input_data: {
                    let mut data = vec![0u8; 32];
                    data[0] = (i % 256) as u8; // Vary the input slightly
                    data
                },
                caller: format!("0xload_caller_{i}"),
                value: 0,
                gas_limit: 50_000,
            };

            manager_clone.execute_contract(execution).await
        });
        handles.push(handle);
    }

    // Wait for completion with timeout
    let results = timeout(Duration::from_secs(30), futures::future::join_all(handles))
        .await
        .expect("Operations timed out");

    let execution_time = start_time.elapsed();

    // Verify all operations succeeded
    let mut successful_operations = 0;
    for result in results {
        if let Ok(Ok(exec_result)) = result {
            if exec_result.success {
                successful_operations += 1;
            }
        }
    }

    assert_eq!(successful_operations, num_operations);

    let ops_per_second = num_operations as f64 / execution_time.as_secs_f64();
    println!(
        "Performance test: {} operations in {:.2}s ({:.2} ops/sec)",
        num_operations,
        execution_time.as_secs_f64(),
        ops_per_second
    );

    // Performance should be reasonable (at least 10 ops/sec)
    assert!(
        ops_per_second > 10.0,
        "Performance too low: {ops_per_second:.2} ops/sec"
    );

    println!("Performance under load test completed successfully");
}
