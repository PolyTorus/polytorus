//! Advanced smart contract integration tests

use crate::smart_contract::{
    contract::SmartContract, engine::ContractEngine, state::ContractState, types::ContractExecution,
};
use tempfile::TempDir;

#[cfg(test)]
pub mod advanced_contract_tests {
    use super::*;

    fn create_test_engine() -> (ContractEngine, TempDir) {
        let temp_dir = TempDir::new().unwrap();
        let state = ContractState::new(temp_dir.path().to_str().unwrap()).unwrap();
        let engine = ContractEngine::new(state).unwrap();
        (engine, temp_dir)
    }

    fn create_test_contract(address_hint: &str) -> SmartContract {
        SmartContract::new(
            vec![1, 2, 3, 4], // Placeholder bytecode
            "test_deployer".to_string(),
            vec![],                         // constructor args
            Some(address_hint.to_string()), // Use address hint as ABI for testing
        )
        .unwrap()
    }

    #[test]
    fn test_counter_contract_deployment() {
        let (engine, _temp_dir) = create_test_engine();

        // Create a counter contract
        let contract = create_test_contract("counter_test_001");

        // Deploy the contract
        let result = engine.deploy_contract(&contract);
        assert!(
            result.is_ok(),
            "Failed to deploy counter contract: {:?}",
            result
        );

        // Verify contract is listed
        let contracts = engine.list_contracts().unwrap();
        assert_eq!(contracts.len(), 1);
        assert!(
            contracts[0].address.contains("counter") || contracts[0].creator == "test_deployer"
        );
    }

    #[test]
    fn test_counter_contract_execution() {
        let (engine, _temp_dir) = create_test_engine();

        // Deploy counter contract
        let contract = SmartContract::new(
            vec![1, 2, 3, 4],                     // bytecode
            "test_deployer".to_string(),          // creator
            vec![],                               // constructor_args
            Some("counter_test_002".to_string()), // abi
        )
        .unwrap();

        engine.deploy_contract(&contract).unwrap();

        // Initialize the counter
        let init_execution = ContractExecution {
            contract_address: contract.get_address().to_string(),
            function_name: "init".to_string(),
            arguments: vec![],
            gas_limit: 50000,
            caller: "test_caller".to_string(),
            value: 0,
        };

        let result = engine.execute_contract(init_execution).unwrap();
        assert!(result.success, "Counter initialization failed");
        assert!(result.gas_used > 0, "No gas was consumed");

        // Increment the counter
        let increment_execution = ContractExecution {
            contract_address: contract.get_address().to_string(),
            function_name: "increment".to_string(),
            arguments: vec![],
            gas_limit: 50000,
            caller: "test_caller".to_string(),
            value: 0,
        };

        let result = engine.execute_contract(increment_execution).unwrap();
        assert!(result.success, "Counter increment failed");

        // The result should contain the incremented value (1)
        assert_eq!(result.return_value, vec![1, 0, 0, 0]); // i32 little endian
    }

    #[test]
    fn test_counter_contract_with_parameters() {
        let (engine, _temp_dir) = create_test_engine();

        // Deploy counter contract
        let contract = SmartContract::new(
            vec![1, 2, 3, 4],                     // bytecode
            "test_deployer".to_string(),          // creator
            vec![],                               // constructor_args
            Some("counter_test_003".to_string()), // abi
        )
        .unwrap();

        engine.deploy_contract(&contract).unwrap();

        // Initialize the counter
        let init_execution = ContractExecution {
            contract_address: contract.get_address().to_string(),
            function_name: "init".to_string(),
            arguments: vec![],
            gas_limit: 50000,
            caller: "test_caller".to_string(),
            value: 0,
        };
        engine.execute_contract(init_execution).unwrap();

        // Add a specific value to the counter
        let add_value = 5i32;
        let add_execution = ContractExecution {
            contract_address: contract.get_address().to_string(),
            function_name: "add".to_string(),
            arguments: add_value.to_le_bytes().to_vec(),
            gas_limit: 50000,
            caller: "test_caller".to_string(),
            value: 0,
        };

        let result = engine.execute_contract(add_execution).unwrap();
        assert!(result.success, "Counter add failed");

        // The result should contain the new value (5)
        assert_eq!(result.return_value, vec![5, 0, 0, 0]); // i32 little endian
    }

    #[test]
    fn test_token_contract_deployment() {
        let (engine, _temp_dir) = create_test_engine();

        // Create a token contract
        let contract = SmartContract::new(
            vec![1, 2, 3, 4], // Placeholder bytecode
            "test_deployer".to_string(),
            vec![],                             // constructor_args
            Some("token_test_001".to_string()), // abi
        )
        .unwrap();

        // Deploy the contract
        let result = engine.deploy_contract(&contract);
        assert!(
            result.is_ok(),
            "Failed to deploy token contract: {:?}",
            result
        );

        // Verify contract is listed
        let contracts = engine.list_contracts().unwrap();
        assert_eq!(contracts.len(), 1);
        assert!(contracts[0].address.starts_with("contract_"));
    }

    #[test]
    fn test_token_contract_initialization() {
        let (engine, _temp_dir) = create_test_engine();

        // Deploy token contract
        let contract = SmartContract::new(
            vec![1, 2, 3, 4],                   // bytecode
            "test_deployer".to_string(),        // creator
            vec![],                             // constructor_args
            Some("token_test_002".to_string()), // abi
        )
        .unwrap();

        engine.deploy_contract(&contract).unwrap();

        // Initialize the token with 1000 total supply
        let initial_supply = 1000i32;
        let init_execution = ContractExecution {
            contract_address: contract.get_address().to_string(),
            function_name: "init".to_string(),
            arguments: initial_supply.to_le_bytes().to_vec(),
            gas_limit: 50000,
            caller: "test_deployer".to_string(),
            value: 0,
        };

        let result = engine.execute_contract(init_execution).unwrap();
        assert!(result.success, "Token initialization failed");
        assert_eq!(result.return_value, vec![1, 0, 0, 0]); // Success return value

        // Check total supply
        let supply_execution = ContractExecution {
            contract_address: contract.get_address().to_string(),
            function_name: "total_supply".to_string(),
            arguments: vec![],
            gas_limit: 50000,
            caller: "test_caller".to_string(),
            value: 0,
        };

        let result = engine.execute_contract(supply_execution).unwrap();
        assert!(result.success, "Total supply check failed");
        assert_eq!(result.return_value, vec![232, 3, 0, 0]); // 1000 in little endian
    }

    #[test]
    fn test_token_contract_transfer() {
        println!("[test_token_contract_transfer] Starting test");
        let (engine, _temp_dir) = create_test_engine();

        // Deploy and initialize token contract
        let contract = SmartContract::new(
            vec![1, 2, 3, 4],                   // bytecode
            "test_deployer".to_string(),        // creator
            vec![],                             // constructor_args
            Some("token_test_003".to_string()), // abi
        )
        .unwrap();

        println!("[test_token_contract_transfer] Deploying contract");
        engine.deploy_contract(&contract).unwrap();
        println!("[test_token_contract_transfer] Contract deployed");

        // Initialize with 1000 tokens
        let initial_supply = 1000i32;
        let init_execution = ContractExecution {
            contract_address: contract.get_address().to_string(),
            function_name: "init".to_string(),
            arguments: initial_supply.to_le_bytes().to_vec(),
            gas_limit: 50000,
            caller: "test_deployer".to_string(),
            value: 0,
        };
        println!("[test_token_contract_transfer] Executing init");
        engine.execute_contract(init_execution).unwrap();
        println!("[test_token_contract_transfer] Init executed");

        // Transfer 100 tokens to another address
        let recipient = 12345i32; // Simple address representation
        let amount = 100i32;
        let mut transfer_args = Vec::new();
        transfer_args.extend_from_slice(&recipient.to_le_bytes());
        transfer_args.extend_from_slice(&amount.to_le_bytes());

        let transfer_execution = ContractExecution {
            contract_address: contract.get_address().to_string(),
            function_name: "transfer".to_string(),
            arguments: transfer_args,
            gas_limit: 50000,
            caller: "test_deployer".to_string(),
            value: 0,
        };

        println!("[test_token_contract_transfer] Executing transfer");
        let result = engine.execute_contract(transfer_execution).unwrap();
        println!("[test_token_contract_transfer] Transfer executed");
        assert!(result.success, "Token transfer failed");
        assert_eq!(result.return_value, vec![1, 0, 0, 0]); // Success return value
        println!("[test_token_contract_transfer] Test finished");
    }

    #[test]
    fn test_gas_limit_enforcement() {
        let (engine, _temp_dir) = create_test_engine();

        // Deploy a contract
        let contract = SmartContract::new(
            vec![1, 2, 3, 4],                 // bytecode
            "test_deployer".to_string(),      // creator
            vec![],                           // constructor_args
            Some("gas_test_001".to_string()), // abi
        )
        .unwrap();

        engine.deploy_contract(&contract).unwrap();

        // Try to execute with very low gas limit
        let execution = ContractExecution {
            contract_address: contract.get_address().to_string(),
            function_name: "init".to_string(),
            arguments: vec![],
            gas_limit: 1, // Very low gas limit
            caller: "test_caller".to_string(),
            value: 0,
        };

        let result = engine.execute_contract(execution).unwrap();
        // Should fail due to gas limit
        assert!(
            !result.success,
            "Execution should have failed due to gas limit"
        );
        assert!(result.gas_used > 1, "Gas usage should exceed limit");
    }

    #[test]
    fn test_contract_state_persistence() {
        let (engine, _temp_dir) = create_test_engine();

        // Deploy counter contract
        let contract = SmartContract::new(
            vec![1, 2, 3, 4],                   // bytecode
            "test_deployer".to_string(),        // creator
            vec![],                             // constructor_args
            Some("state_test_001".to_string()), // abi
        )
        .unwrap();

        engine.deploy_contract(&contract).unwrap();

        // Initialize and increment multiple times
        let init_execution = ContractExecution {
            contract_address: contract.get_address().to_string(),
            function_name: "init".to_string(),
            arguments: vec![],
            gas_limit: 50000,
            caller: "test_caller".to_string(),
            value: 0,
        };
        engine.execute_contract(init_execution).unwrap();

        // Increment 3 times
        for _ in 0..3 {
            let increment_execution = ContractExecution {
                contract_address: contract.get_address().to_string(),
                function_name: "increment".to_string(),
                arguments: vec![],
                gas_limit: 50000,
                caller: "test_caller".to_string(),
                value: 0,
            };
            engine.execute_contract(increment_execution).unwrap();
        }

        // Get final value
        let get_execution = ContractExecution {
            contract_address: contract.get_address().to_string(),
            function_name: "get".to_string(),
            arguments: vec![],
            gas_limit: 50000,
            caller: "test_caller".to_string(),
            value: 0,
        };

        let result = engine.execute_contract(get_execution).unwrap();
        assert!(result.success, "Get counter value failed");
        // Should be 3 after 3 increments
        assert_eq!(result.return_value, vec![3, 0, 0, 0]); // i32 little endian

        // Check that state changes were recorded
        assert!(
            !result.state_changes.is_empty(),
            "No state changes recorded"
        );
    }
}
