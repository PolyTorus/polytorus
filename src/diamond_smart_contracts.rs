use crate::diamond_io_integration::{DiamondIOIntegration, DiamondIOConfig};
use diamond_io::bgg::circuit::PolyCircuit;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use anyhow::Result;
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiamondContract {
    pub id: String,
    pub name: String,
    pub description: String,
    pub config: DiamondIOConfig,
    pub circuit: Option<String>, // Serialized circuit
    pub is_obfuscated: bool,
    pub creation_time: u64,
    pub owner: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContractExecution {
    pub contract_id: String,
    pub inputs: Vec<bool>,
    pub outputs: Option<Vec<bool>>,
    pub execution_time: Option<u64>,
    pub gas_used: u64,
    pub timestamp: u64,
    pub executor: String,
}

#[derive(Debug)]
pub struct DiamondContractEngine {
    contracts: HashMap<String, DiamondContract>,
    executions: Vec<ContractExecution>,
    diamond_io: DiamondIOIntegration,
}

impl DiamondContractEngine {
    pub fn new(config: DiamondIOConfig) -> Result<Self> {
        let diamond_io = DiamondIOIntegration::new(config)?;
        
        Ok(Self {
            contracts: HashMap::new(),
            executions: Vec::new(),
            diamond_io,
        })
    }

    /// Deploy a new Diamond IO powered smart contract
    pub async fn deploy_contract(
        &mut self,
        id: String,
        name: String,
        description: String,
        owner: String,
        circuit_description: &str,
    ) -> Result<String> {
        info!("Deploying Diamond contract: {}", name);

        // Create a circuit based on description (not stored due to serialization issues)
        let _circuit = self.create_circuit_from_description(circuit_description)?;
        
        let contract = DiamondContract {
            id: id.clone(),
            name,
            description,
            config: self.diamond_io.config().clone(),
            circuit: None, // Cannot serialize PolyCircuit directly
            is_obfuscated: false,
            creation_time: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs(),
            owner,
        };

        self.contracts.insert(id.clone(), contract);
        info!("Contract {} deployed successfully", id);
        
        Ok(id)
    }

    /// Obfuscate a deployed contract
    pub async fn obfuscate_contract(&mut self, contract_id: &str) -> Result<()> {
        // Get contract information first (not the mutable reference)
        let (description, config) = {
            let contract = self.contracts.get(contract_id)
                .ok_or_else(|| anyhow::anyhow!("Contract not found: {}", contract_id))?;
            
            if contract.is_obfuscated {
                warn!("Contract {} is already obfuscated", contract_id);
                return Ok(());
            }
            (contract.description.clone(), contract.config.clone())
        };

        info!("Obfuscating contract: {}", contract_id);

        // Since we cannot serialize/deserialize PolyCircuit, recreate it
        let circuit = self.create_circuit_from_description(&description)?;

        // Set obfuscation directory specific to this contract
        let mut diamond_io = DiamondIOIntegration::new(config)?;
        diamond_io.set_obfuscation_dir(format!("obfuscation_data_{}", contract_id));

        // Obfuscate the circuit
        diamond_io.obfuscate_circuit(circuit).await?;

        // Now update the contract
        if let Some(contract) = self.contracts.get_mut(contract_id) {
            contract.is_obfuscated = true;
        }
        
        info!("Contract {} obfuscated successfully", contract_id);

        Ok(())
    }

    /// Execute a contract with given inputs
    pub async fn execute_contract(
        &mut self,
        contract_id: &str,
        inputs: Vec<bool>,
        executor: String,
    ) -> Result<Vec<bool>> {
        let contract = self.contracts.get(contract_id)
            .ok_or_else(|| anyhow::anyhow!("Contract not found: {}", contract_id))?;

        info!("Executing contract: {} with inputs: {:?}", contract_id, inputs);

        let start_time = std::time::Instant::now();
        
        // Check if inputs match expected size
        if inputs.len() != contract.config.input_size {
            return Err(anyhow::anyhow!(
                "Input size mismatch for contract {}: expected {}, got {}",
                contract_id,
                contract.config.input_size,
                inputs.len()
            ));
        }

        // Create Diamond IO instance for this contract
        let mut diamond_io = DiamondIOIntegration::new(contract.config.clone())?;
        diamond_io.set_obfuscation_dir(format!("obfuscation_data_{}", contract_id));

        let outputs = if contract.is_obfuscated {
            // Execute obfuscated circuit
            diamond_io.evaluate_circuit(&inputs).await?
        } else {
            // Execute plain circuit (for testing/development)
            self.execute_plain_circuit(contract, &inputs)?
        };

        let execution_time = start_time.elapsed().as_millis() as u64;
        let gas_used = self.calculate_gas_usage(&inputs, &outputs, execution_time);

        // Record execution
        let execution = ContractExecution {
            contract_id: contract_id.to_string(),
            inputs,
            outputs: Some(outputs.clone()),
            execution_time: Some(execution_time),
            gas_used,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)?
                .as_secs(),
            executor,
        };

        self.executions.push(execution);
        
        info!(
            "Contract {} executed successfully in {}ms, gas used: {}",
            contract_id, execution_time, gas_used
        );

        Ok(outputs)
    }

    /// Get contract information
    pub fn get_contract(&self, contract_id: &str) -> Option<&DiamondContract> {
        self.contracts.get(contract_id)
    }

    /// List all contracts
    pub fn list_contracts(&self) -> Vec<&DiamondContract> {
        self.contracts.values().collect()
    }

    /// Get execution history for a contract
    pub fn get_execution_history(&self, contract_id: &str) -> Vec<&ContractExecution> {
        self.executions
            .iter()
            .filter(|exec| exec.contract_id == contract_id)
            .collect()
    }

    /// Get all executions
    pub fn get_all_executions(&self) -> &[ContractExecution] {
        &self.executions
    }

    /// Encrypt data using Diamond IO
    pub fn encrypt_data(&self, data: &[bool]) -> Result<String> {
        let _encrypted = self.diamond_io.encrypt_data(data)?;
        // Cannot serialize BaseMatrix<DCRTPoly>, return placeholder
        Ok("encrypted_data_placeholder".to_string())
    }

    /// Create a circuit from textual description
    fn create_circuit_from_description(&self, description: &str) -> Result<PolyCircuit> {
        info!("Creating circuit from description: {}", description);
        
        let mut circuit = PolyCircuit::new();
        
        // Parse simple circuit descriptions
        match description.to_lowercase().as_str() {
            "and_gate" => {
                let inputs = circuit.input(2);
                let input1 = inputs[0];
                let input2 = inputs[1];
                let and_result = circuit.and_gate(input1, input2);
                circuit.output(vec![and_result]);
            }
            "or_gate" => {
                let inputs = circuit.input(2);
                let input1 = inputs[0];
                let input2 = inputs[1];
                // OR = input1 + input2 - input1 * input2
                let sum = circuit.add_gate(input1, input2);
                let product = circuit.mul_gate(input1, input2);
                let or_result = circuit.sub_gate(sum, product);
                circuit.output(vec![or_result]);
            }
            "xor_gate" => {
                let inputs = circuit.input(2);
                let input1 = inputs[0];
                let input2 = inputs[1];
                // XOR = input1 + input2 - 2 * input1 * input2
                let sum = circuit.add_gate(input1, input2);
                let product = circuit.mul_gate(input1, input2);
                let double_product = circuit.add_gate(product, product);
                let xor_result = circuit.sub_gate(sum, double_product);
                circuit.output(vec![xor_result]);
            }
            "adder" => {
                // Simple 2-bit adder
                let inputs = circuit.input(4);
                let a0 = inputs[0];
                let a1 = inputs[1];
                let b0 = inputs[2];
                let b1 = inputs[3];
                
                // Sum bit 0: a0 XOR b0
                let sum0_temp = circuit.add_gate(a0, b0);
                let carry0_temp = circuit.mul_gate(a0, b0);
                let carry0_double = circuit.add_gate(carry0_temp, carry0_temp);
                let sum0 = circuit.sub_gate(sum0_temp, carry0_double);
                
                // Carry from bit 0
                let carry0 = carry0_temp;
                
                // Sum bit 1: a1 XOR b1 XOR carry0
                let sum1_temp1 = circuit.add_gate(a1, b1);
                let sum1_temp2 = circuit.add_gate(sum1_temp1, carry0);
                let product1 = circuit.mul_gate(a1, b1);
                let product2 = circuit.mul_gate(sum1_temp1, carry0);
                let product1_double = circuit.add_gate(product1, product1);
                let product2_double = circuit.add_gate(product2, product2);
                let products_sum = circuit.add_gate(product1_double, product2_double);
                let sum1 = circuit.sub_gate(sum1_temp2, products_sum);
                
                circuit.output(vec![sum0, sum1]);
            }
            _ => {
                // Default: simple echo circuit
                let inputs = circuit.input(1);
                let input = inputs[0];
                circuit.output(vec![input]);
            }
        }
        
        Ok(circuit)
    }

    /// Execute a plain (non-obfuscated) circuit for testing
    fn execute_plain_circuit(
        &self,
        contract: &DiamondContract,
        inputs: &[bool],
    ) -> Result<Vec<bool>> {
        // For demonstration, we'll implement basic logic gates
        match contract.description.to_lowercase().as_str() {
            "and_gate" => {
                if inputs.len() < 2 {
                    return Err(anyhow::anyhow!("AND gate requires 2 inputs"));
                }
                Ok(vec![inputs[0] && inputs[1]])
            }
            "or_gate" => {
                if inputs.len() < 2 {
                    return Err(anyhow::anyhow!("OR gate requires 2 inputs"));
                }
                Ok(vec![inputs[0] || inputs[1]])
            }
            "xor_gate" => {
                if inputs.len() < 2 {
                    return Err(anyhow::anyhow!("XOR gate requires 2 inputs"));
                }
                Ok(vec![inputs[0] ^ inputs[1]])
            }
            "adder" => {
                if inputs.len() < 4 {
                    return Err(anyhow::anyhow!("Adder requires 4 inputs"));
                }
                let a = (inputs[1] as u8) << 1 | (inputs[0] as u8);
                let b = (inputs[3] as u8) << 1 | (inputs[2] as u8);
                let sum = a + b;
                Ok(vec![
                    (sum & 1) != 0,        // bit 0
                    ((sum >> 1) & 1) != 0, // bit 1
                ])
            }
            _ => {
                // Echo circuit
                Ok(inputs.to_vec())
            }
        }
    }

    /// Calculate gas usage based on execution parameters
    fn calculate_gas_usage(
        &self,
        inputs: &[bool],
        outputs: &[bool],
        execution_time_ms: u64,
    ) -> u64 {
        let base_gas = 21000; // Base transaction cost
        let input_gas = inputs.len() as u64 * 100; // Gas per input
        let output_gas = outputs.len() as u64 * 50; // Gas per output
        let time_gas = execution_time_ms / 10; // Time-based gas
        
        base_gas + input_gas + output_gas + time_gas
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn get_test_config() -> DiamondIOConfig {
        DiamondIOConfig {
            ring_dimension: 16,
            crt_depth: 2,
            crt_bits: 17,
            base_bits: 1,
            input_size: 2,
            level_width: 4,
            d: 2,
            ..Default::default()
        }
    }

    #[tokio::test]
    async fn test_contract_deployment() {
        let config = get_test_config();
        let mut engine = DiamondContractEngine::new(config).unwrap();
        
        let contract_id = engine.deploy_contract(
            "test_and".to_string(),
            "Test AND Gate".to_string(),
            "and_gate".to_string(),
            "alice".to_string(),
            "and_gate",
        ).await.unwrap();
        
        assert_eq!(contract_id, "test_and");
        assert!(engine.get_contract(&contract_id).is_some());
    }

    #[tokio::test]
    async fn test_contract_execution() {
        let config = get_test_config();
        let mut engine = DiamondContractEngine::new(config).unwrap();
        
        let contract_id = engine.deploy_contract(
            "test_and".to_string(),
            "Test AND Gate".to_string(),
            "and_gate".to_string(),
            "alice".to_string(),
            "and_gate",
        ).await.unwrap();
        
        // Test AND gate
        let result = engine.execute_contract(
            &contract_id,
            vec![true, false],
            "bob".to_string(),
        ).await.unwrap();
        
        assert_eq!(result, vec![false]);
        
        let result = engine.execute_contract(
            &contract_id,
            vec![true, true],
            "charlie".to_string(),
        ).await.unwrap();
        
        assert_eq!(result, vec![true]);
    }

    #[tokio::test]
    async fn test_execution_history() {
        let config = get_test_config();
        let mut engine = DiamondContractEngine::new(config).unwrap();
        
        let contract_id = engine.deploy_contract(
            "test_or".to_string(),
            "Test OR Gate".to_string(),
            "or_gate".to_string(),
            "alice".to_string(),
            "or_gate",
        ).await.unwrap();
        
        // Execute multiple times
        engine.execute_contract(&contract_id, vec![true, false], "bob".to_string()).await.unwrap();
        engine.execute_contract(&contract_id, vec![false, false], "charlie".to_string()).await.unwrap();
        
        let history = engine.get_execution_history(&contract_id);
        assert_eq!(history.len(), 2);
    }
}
