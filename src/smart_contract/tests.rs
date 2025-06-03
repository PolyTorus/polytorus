// Test smart contract functionality

#[cfg(test)]
mod smart_contract_tests {
    use crate::smart_contract::engine::ContractEngine;
    use crate::smart_contract::state::ContractState;
    use crate::smart_contract::types::{ContractExecution, ContractMetadata};
    use crate::smart_contract::contract::SmartContract;
    use std::collections::HashMap;
    use tempfile::tempdir;

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
        state.set("test_contract", "key1", &b"value1".to_vec()).unwrap();
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
        ).unwrap();

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
}
