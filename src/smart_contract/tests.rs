// Test smart contract functionality

#[cfg(test)]
mod smart_contract_tests {
    use std::collections::HashMap;

    use tempfile::tempdir;

    use crate::smart_contract::{
        contract::SmartContract,
        engine::ContractEngine,
        state::ContractState,
        types::{ContractExecution, ContractMetadata},
    };

    #[test]
    fn test_contract_state_storage() {
        let temp_dir = tempdir().unwrap();
        let state = ContractState::new(temp_dir.path().to_str().unwrap()).unwrap();

        // Test storing and retrieving contract metadata
        let metadata = ContractMetadata {
            address: "test_contract".to_string(),
            creator: "test_owner".to_string(),
            bytecode_hash: "hash123".to_string(),
            created_at: 0,
            abi: None,
        };

        state.store_contract(&metadata).unwrap();
        let retrieved = state.get_contract("test_contract").unwrap();
        assert!(retrieved.is_some());
        assert_eq!(retrieved.unwrap().address, "test_contract");

        // Test storing and retrieving state
        state
            .set("test_contract", "key1", b"value1".as_ref())
            .unwrap();
        let value = state.get("test_contract", "key1").unwrap();
        assert!(value.is_some());
        assert_eq!(value.unwrap(), b"value1".to_vec());
    }

    #[test]
    fn test_contract_engine_creation() {
        let temp_dir = tempdir().unwrap();
        let state = ContractState::new(temp_dir.path().to_str().unwrap()).unwrap();
        let engine = ContractEngine::new(state);
        assert!(engine.is_ok());
    }

    #[test]
    fn test_contract_deployment() {
        let temp_dir = tempdir().unwrap();
        let state = ContractState::new(temp_dir.path().to_str().unwrap()).unwrap();
        let engine = ContractEngine::new(state).unwrap();

        // Create a test contract
        let contract = SmartContract::new(
            b"simple_wasm_bytecode".to_vec(),
            "owner".to_string(),
            vec![],
            None,
        )
        .unwrap();

        let result = engine.deploy_contract(&contract);
        assert!(result.is_ok());

        // Check if contract is stored
        let contracts = engine.list_contracts().unwrap();
        assert_eq!(contracts.len(), 1);
    }

    #[test]
    fn test_smart_contract_types() {
        // Test ContractExecution
        let execution = ContractExecution {
            contract_address: "test".to_string(),
            function_name: "main".to_string(),
            arguments: vec![],
            caller: "caller".to_string(),
            value: 0,
            gas_limit: 100000,
        };
        assert_eq!(execution.contract_address, "test");
        assert_eq!(execution.function_name, "main");

        // Test ContractMetadata
        let metadata = ContractMetadata {
            address: "contract1".to_string(),
            creator: "owner1".to_string(),
            bytecode_hash: "hash".to_string(),
            created_at: 123456,
            abi: None,
        };
        assert_eq!(metadata.address, "contract1");
        assert_eq!(metadata.creator, "owner1");
    }

    #[test]
    fn test_contract_state_changes() {
        let temp_dir = tempdir().unwrap();
        let state = ContractState::new(temp_dir.path().to_str().unwrap()).unwrap();

        // Test batch state changes
        let mut changes = HashMap::new();
        changes.insert("state:contract1:key1".to_string(), b"value1".to_vec());
        changes.insert("state:contract1:key2".to_string(), b"value2".to_vec());

        let result = state.apply_changes(&changes);
        assert!(result.is_ok());

        // Verify changes were applied
        let value1 = state.get("contract1", "key1").unwrap();
        let value2 = state.get("contract1", "key2").unwrap();
        assert!(value1.is_some());
        assert!(value2.is_some());
        assert_eq!(value1.unwrap(), b"value1".to_vec());
        assert_eq!(value2.unwrap(), b"value2".to_vec());
    }

    #[test]
    fn test_host_function_context_creation() {
        let temp_dir = tempdir().unwrap();
        let state = ContractState::new(temp_dir.path().to_str().unwrap()).unwrap();
        let engine = ContractEngine::new(state).unwrap();

        // Test that host functions can be created with execution context
        let execution = ContractExecution {
            contract_address: "test_contract".to_string(),
            function_name: "init".to_string(),
            arguments: vec![],
            gas_limit: 50000,
            caller: "test_caller".to_string(),
            value: 42,
        };

        // This should not panic or fail when creating the host context
        let result = engine.execute_contract(execution);
        assert!(result.is_ok(), "Host function context creation failed");

        let contract_result = result.unwrap();
        // Should execute successfully with host functions
        assert!(
            contract_result.success,
            "Contract execution with host functions failed"
        );
    }

    #[test]
    fn test_host_function_storage_operations() {
        let temp_dir = tempdir().unwrap();
        let state = ContractState::new(temp_dir.path().to_str().unwrap()).unwrap();
        let engine = ContractEngine::new(state).unwrap();

        // Pre-populate some storage data for the contract
        {
            let state_guard = engine.get_state().lock().unwrap();
            state_guard
                .set("test_contract", "counter", &[5, 0, 0, 0])
                .unwrap();
            state_guard.set("test_contract", "owner", b"alice").unwrap();
        }

        // Execute a contract that should have access to storage via host functions
        let execution = ContractExecution {
            contract_address: "test_contract".to_string(),
            function_name: "get".to_string(),
            arguments: vec![],
            gas_limit: 50000,
            caller: "test_caller".to_string(),
            value: 0,
        };

        let result = engine.execute_contract(execution).unwrap();
        assert!(result.success, "Storage operation failed");

        // The host functions are now active and can access the storage
        // The actual storage access depends on the WASM contract calling the host functions
    }

    #[test]
    fn test_host_function_caller_and_value() {
        let temp_dir = tempdir().unwrap();
        let state = ContractState::new(temp_dir.path().to_str().unwrap()).unwrap();
        let engine = ContractEngine::new(state).unwrap();

        // Test with specific caller and value
        let execution = ContractExecution {
            contract_address: "test_contract".to_string(),
            function_name: "main".to_string(), // Use available function
            arguments: vec![],
            gas_limit: 50000,
            caller: "specific_caller_address".to_string(),
            value: 1000,
        };

        let result = engine.execute_contract(execution).unwrap();
        assert!(result.success, "Caller/value test failed");

        // The get_caller and get_value host functions now have access to the actual values
        // The returned values would depend on the WASM contract actually calling these functions
    }

    #[test]
    fn test_host_function_logging() {
        let temp_dir = tempdir().unwrap();
        let state = ContractState::new(temp_dir.path().to_str().unwrap()).unwrap();
        let engine = ContractEngine::new(state).unwrap();

        // Execute a contract that might generate logs
        let execution = ContractExecution {
            contract_address: "logging_contract".to_string(),
            function_name: "init".to_string(), // Use available function
            arguments: vec![],
            gas_limit: 50000,
            caller: "test_caller".to_string(),
            value: 0,
        };

        let result = engine.execute_contract(execution).unwrap();
        assert!(result.success, "Logging test failed");

        // The logs field should be populated if the WASM contract calls the log host function
        // Since our test contract doesn't actually call log, logs might be empty
        // But the host function is available for use
    }

    #[test]
    fn test_actual_vs_dummy_host_functions() {
        let temp_dir = tempdir().unwrap();
        let state = ContractState::new(temp_dir.path().to_str().unwrap()).unwrap();
        let engine = ContractEngine::new(state).unwrap();

        // Test that the new host functions provide actual processing
        // vs the old dummy implementations

        // Store some test data
        {
            let state_guard = engine.get_state().lock().unwrap();
            state_guard
                .set("test_contract", "test_key", &[42, 0, 0, 0])
                .unwrap();
        }

        let execution = ContractExecution {
            contract_address: "test_contract".to_string(),
            function_name: "get".to_string(), // Use available function
            arguments: vec![],
            gas_limit: 50000,
            caller: "test_caller".to_string(),
            value: 999,
        };

        let result = engine.execute_contract(execution).unwrap();
        assert!(result.success, "Host function test failed");

        // Verify that the host functions have actual context
        // The execution should succeed with real host function implementations
        // rather than failing with dummy implementations
    }

    #[test]
    fn test_storage_persistence_with_wasm_calls() {
        let temp_dir = tempdir().unwrap();
        let state = ContractState::new(temp_dir.path().to_str().unwrap()).unwrap();
        let engine = ContractEngine::new(state).unwrap();

        let contract_address = "storage_test_contract";

        // Test 1: Write data using WASM contract storage_set host function
        let write_execution = ContractExecution {
            contract_address: contract_address.to_string(),
            function_name: "test_storage".to_string(),
            arguments: vec![],
            gas_limit: 100000,
            caller: "test_caller".to_string(),
            value: 0,
        };

        let write_result = engine.execute_contract(write_execution).unwrap();
        assert!(write_result.success, "Storage write operation failed");

        // The test_storage function should return success (1) from storage_set
        assert_eq!(
            write_result.return_value,
            vec![1, 0, 0, 0],
            "Storage set should return success"
        );

        // Verify state changes were tracked
        assert!(
            !write_result.state_changes.is_empty(),
            "State changes should be tracked"
        );

        // Test 2: Read data using WASM contract storage_get host function
        let read_execution = ContractExecution {
            contract_address: contract_address.to_string(),
            function_name: "read_counter".to_string(),
            arguments: vec![],
            gas_limit: 100000,
            caller: "test_caller".to_string(),
            value: 0,
        };

        let read_result = engine.execute_contract(read_execution).unwrap();
        assert!(read_result.success, "Storage read operation failed");

        // The read_counter function should return the length of data read (4 bytes)
        assert_eq!(
            read_result.return_value,
            vec![4, 0, 0, 0],
            "Should read 4 bytes of data"
        );

        // Test 3: Verify data persistence by reading directly from state
        {
            let state_guard = engine.get_state().lock().unwrap();
            let stored_value = state_guard.get(contract_address, "counter").unwrap();
            assert!(
                stored_value.is_some(),
                "Data should be persisted in storage"
            );

            let value = stored_value.unwrap();
            assert_eq!(
                value,
                vec![5, 0, 0, 0],
                "Stored value should be 5 in little endian"
            );
        }
    }

    #[test]
    fn test_storage_persistence_across_contract_calls() {
        let temp_dir = tempdir().unwrap();
        let state = ContractState::new(temp_dir.path().to_str().unwrap()).unwrap();
        let engine = ContractEngine::new(state).unwrap();

        let contract_address = "persistence_test_contract";

        // Step 1: Initialize storage with test data
        let init_execution = ContractExecution {
            contract_address: contract_address.to_string(),
            function_name: "storage_init".to_string(),
            arguments: vec![],
            gas_limit: 100000,
            caller: "test_caller".to_string(),
            value: 0,
        };

        let init_result = engine.execute_contract(init_execution).unwrap();
        assert!(init_result.success, "Storage initialization failed");
        assert_eq!(
            init_result.return_value,
            vec![1, 0, 0, 0],
            "Storage init should succeed"
        );

        // Step 2: In a separate contract call, read the stored data
        let read_execution = ContractExecution {
            contract_address: contract_address.to_string(),
            function_name: "storage_read".to_string(),
            arguments: vec![],
            gas_limit: 100000,
            caller: "test_caller".to_string(),
            value: 0,
        };

        let read_result = engine.execute_contract(read_execution).unwrap();
        assert!(read_result.success, "Storage read failed");

        // Should read 10 bytes ("test_value")
        assert_eq!(
            read_result.return_value,
            vec![10, 0, 0, 0],
            "Should read 10 bytes of test_value"
        );

        // Step 3: Verify the data is correctly persisted
        {
            let state_guard = engine.get_state().lock().unwrap();
            let stored_value = state_guard.get(contract_address, "test_key").unwrap();
            assert!(stored_value.is_some(), "test_key should exist in storage");

            let value = stored_value.unwrap();
            assert_eq!(
                value,
                b"test_value".to_vec(),
                "Stored value should be 'test_value'"
            );
        }
    }

    #[test]
    fn test_storage_error_handling() {
        let temp_dir = tempdir().unwrap();
        let state = ContractState::new(temp_dir.path().to_str().unwrap()).unwrap();
        let engine = ContractEngine::new(state).unwrap();

        // Test reading non-existent key
        {
            let state_guard = engine.get_state().lock().unwrap();
            let result = state_guard.get("test_contract", "nonexistent_key").unwrap();
            assert!(result.is_none(), "Non-existent key should return None");
        }

        // Test that storage operations work with proper error codes
        let execution = ContractExecution {
            contract_address: "error_test_contract".to_string(),
            function_name: "read_counter".to_string(), // Try to read before writing
            arguments: vec![],
            gas_limit: 100000,
            caller: "test_caller".to_string(),
            value: 0,
        };

        let result = engine.execute_contract(execution).unwrap();
        assert!(result.success, "Contract execution should succeed");

        // Reading non-existent key should return 0 (key not found)
        assert_eq!(
            result.return_value,
            vec![0, 0, 0, 0],
            "Reading non-existent key should return 0"
        );
    }
}
