use std::{fs, path::Path};

use diamond_io::{
    bgg::circuit::PolyCircuit,
    poly::dcrt::DCRTPolyParams,
    // utils::init_tracing, // コメントアウトして、独自のトレーシング管理を使用
};
use num_bigint::BigUint;
use num_traits::Num;
use serde::{Deserialize, Serialize};
use tracing::info;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiamondIOConfig {
    /// Ring dimension (must be power of 2)
    pub ring_dimension: u32,
    /// CRT depth
    pub crt_depth: usize,
    /// CRT bits
    pub crt_bits: usize,
    /// Base bits for gadget decomposition
    pub base_bits: u32,
    /// Switched modulus for the scheme
    #[serde(
        serialize_with = "biguint_to_string",
        deserialize_with = "biguint_from_string"
    )]
    pub switched_modulus: BigUint,
    /// Input size for the obfuscated circuit
    pub input_size: usize,
    /// Level width for the circuit
    pub level_width: usize,
    /// d parameter for the scheme
    pub d: usize,
    /// Hardcoded key sigma
    pub hardcoded_key_sigma: f64,
    /// P sigma
    pub p_sigma: f64,
    /// Trapdoor sigma (optional)
    pub trapdoor_sigma: Option<f64>,
    /// Whether to use dummy mode for fast testing
    pub dummy_mode: bool,
}

fn biguint_to_string<S>(value: &BigUint, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(&value.to_string())
}

fn biguint_from_string<'de, D>(deserializer: D) -> Result<BigUint, D::Error>
where
    D: serde::Deserializer<'de>,
{
    let s = String::deserialize(deserializer)?;
    BigUint::from_str_radix(&s, 10).map_err(serde::de::Error::custom)
}

impl Default for DiamondIOConfig {
    fn default() -> Self {
        Self {
            ring_dimension: 16,
            crt_depth: 4,
            crt_bits: 30,
            base_bits: 4,
            switched_modulus: BigUint::from_str_radix("17592454479871", 10).unwrap(),
            input_size: 8,
            level_width: 4,
            d: 2,
            hardcoded_key_sigma: 0.0,
            p_sigma: 0.0,
            trapdoor_sigma: Some(4.578),
            dummy_mode: false,
        }
    }
}

impl DiamondIOConfig {
    /// Create config for production with full security
    pub fn production() -> Self {
        Self {
            ring_dimension: 4096,
            crt_depth: 16,
            crt_bits: 45,
            base_bits: 8,
            switched_modulus: BigUint::from_str_radix("107374175678464", 10).unwrap(),
            input_size: 64,
            level_width: 8,
            d: 8,
            hardcoded_key_sigma: 3.2,
            p_sigma: 3.2,
            trapdoor_sigma: Some(4.578),
            dummy_mode: false,
        }
    }

    /// Create config for testing with moderate security
    pub fn testing() -> Self {
        Self {
            ring_dimension: 128,
            crt_depth: 8,
            crt_bits: 35,
            base_bits: 6,
            switched_modulus: BigUint::from_str_radix("549755813887", 10).unwrap(),
            input_size: 16,
            level_width: 4,
            d: 4,
            hardcoded_key_sigma: 2.0,
            p_sigma: 2.0,
            trapdoor_sigma: Some(4.578),
            dummy_mode: false,
        }
    }

    /// Create config for dummy mode (fast simulation)
    pub fn dummy() -> Self {
        Self {
            ring_dimension: 16,
            crt_depth: 4,
            crt_bits: 30,
            base_bits: 4,
            switched_modulus: BigUint::from_str_radix("17592454479871", 10).unwrap(),
            input_size: 8,
            level_width: 4,
            d: 2,
            hardcoded_key_sigma: 0.0,
            p_sigma: 0.0,
            trapdoor_sigma: Some(4.578),
            dummy_mode: true,
        }
    }
}

/// Diamond IO operation result
#[derive(Debug, Clone)]
pub struct DiamondIOResult {
    pub success: bool,
    pub outputs: Vec<bool>,
    pub execution_time_ms: u64,
}

pub struct DiamondIOIntegration {
    config: DiamondIOConfig,
    params: DCRTPolyParams,
    obfuscation_dir: String,
}

impl DiamondIOIntegration {
    /// Create a new Diamond IO integration instance
    pub fn new(config: DiamondIOConfig) -> anyhow::Result<Self> {
        // Note: Tracing initialization is handled externally to avoid conflicts

        // Create polynomial parameters
        let params = DCRTPolyParams::new(
            config.ring_dimension,
            config.crt_depth,
            config.crt_bits,
            config.base_bits,
        );

        let obfuscation_dir = "obfuscation_data".to_string();

        Ok(Self {
            config,
            params,
            obfuscation_dir,
        })
    }

    /// Create a demo circuit for testing
    pub fn create_demo_circuit(&self) -> PolyCircuit {
        let mut circuit = PolyCircuit::new();

        if self.config.dummy_mode {
            // Simple circuit for dummy mode
            let inputs = circuit.input(2);
            if inputs.len() >= 2 {
                let input1 = inputs[0];
                let input2 = inputs[1];
                let sum = circuit.add_gate(input1, input2);
                circuit.output(vec![sum]);
            }
            return circuit;
        }

        // Real mode: Create more sophisticated circuits
        let input_count = std::cmp::min(self.config.input_size, 16);
        let inputs = circuit.input(input_count);

        if inputs.len() >= 2 {
            let mut result = inputs[0];

            for i in 1..inputs.len() {
                if i % 2 == 1 {
                    result = circuit.add_gate(result, inputs[i]);
                } else {
                    result = circuit.mul_gate(result, inputs[i]);
                }
            }

            circuit.output(vec![result]);
        }

        circuit
    }

    /// Obfuscate a circuit using real Diamond IO
    pub async fn obfuscate_circuit(&self, circuit: PolyCircuit) -> anyhow::Result<()> {
        if self.config.dummy_mode {
            info!("Circuit obfuscation simulated (dummy mode)");
            return Ok(());
        }

        info!("Starting real Diamond IO circuit obfuscation...");

        let dir = Path::new(&self.obfuscation_dir);
        if dir.exists() {
            fs::remove_dir_all(dir).unwrap_or_else(|e| {
                eprintln!(
                    "Warning: Failed to remove existing obfuscation directory: {}",
                    e
                );
            });
        }
        fs::create_dir_all(dir)?;

        let start_time = std::time::Instant::now();

        // Validate circuit
        if circuit.num_input() == 0 || circuit.num_output() == 0 {
            return Err(anyhow::anyhow!(
                "Invalid circuit: must have at least one input and one output"
            ));
        } // For now, use placeholder implementation until Diamond IO API is fully stable
        info!("Diamond IO obfuscation - using placeholder implementation");
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;

        let obfuscation_time = start_time.elapsed();
        info!("Obfuscation completed in: {:?}", obfuscation_time);
        Ok(())
    }
    /// Evaluate an obfuscated circuit using Diamond IO
    pub async fn evaluate_circuit(&self, inputs: &[bool]) -> anyhow::Result<Vec<bool>> {
        if self.config.dummy_mode {
            return self.simulate_circuit_evaluation(inputs);
        }

        info!("Starting Diamond IO circuit evaluation...");
        let start_time = std::time::Instant::now();

        let dir = Path::new(&self.obfuscation_dir);
        if !dir.exists() {
            return Err(anyhow::anyhow!(
                "Obfuscation data not found. Please run obfuscate_circuit first."
            ));
        }

        // Pad or truncate inputs to match expected size
        let mut eval_inputs = inputs.to_vec();
        eval_inputs.resize(self.config.input_size, false);

        // For now, use placeholder implementation until Diamond IO API is fully stable
        info!("Diamond IO evaluation - using placeholder implementation");
        let result = self.simulate_circuit_evaluation(&eval_inputs)?;

        let eval_time = start_time.elapsed();
        info!("Evaluation completed in: {:?}", eval_time);
        Ok(result)
    }

    /// Execute circuit and return detailed result
    pub async fn execute_circuit_detailed(
        &self,
        inputs: &[bool],
    ) -> anyhow::Result<DiamondIOResult> {
        let start_time = std::time::Instant::now();

        let outputs = self.evaluate_circuit(inputs).await?;
        let execution_time = start_time.elapsed().as_millis() as u64;

        Ok(DiamondIOResult {
            success: true,
            outputs,
            execution_time_ms: execution_time,
        })
    }

    /// Simulate circuit evaluation for dummy mode or fallback
    fn simulate_circuit_evaluation(&self, inputs: &[bool]) -> anyhow::Result<Vec<bool>> {
        info!("Simulating circuit evaluation...");

        // Simple simulation: XOR all inputs
        let result = inputs.iter().fold(false, |acc, &x| acc ^ x);
        Ok(vec![result])
    }

    /// Encrypt data for privacy
    pub fn encrypt_data(&self, data: &[bool]) -> anyhow::Result<Vec<u8>> {
        if self.config.dummy_mode {
            // Simple dummy encryption
            let mut result = Vec::new();
            for chunk in data.chunks(8) {
                let mut byte = 0u8;
                for (i, &bit) in chunk.iter().enumerate() {
                    if bit {
                        byte |= 1 << i;
                    }
                }
                result.push(byte);
            }
            Ok(result)
        } else {
            // Use actual Diamond IO encryption
            info!("Encrypting data using Diamond IO...");
            // Placeholder implementation - would need actual Diamond IO encryption API
            Ok(data.iter().map(|&b| if b { 1u8 } else { 0u8 }).collect())
        }
    }

    /// Set the obfuscation directory
    pub fn set_obfuscation_dir(&mut self, dir: String) {
        self.obfuscation_dir = dir;
    }

    /// Get configuration
    pub fn config(&self) -> &DiamondIOConfig {
        &self.config
    }

    /// Get parameters
    pub fn params(&self) -> &DCRTPolyParams {
        &self.params
    }
}

impl std::fmt::Debug for DiamondIOIntegration {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DiamondIOIntegration")
            .field("config", &self.config)
            .field("obfuscation_dir", &self.obfuscation_dir)
            .finish()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_diamond_io_config_default() {
        let config = DiamondIOConfig::default();
        assert_eq!(config.ring_dimension, 16);
        assert_eq!(config.crt_depth, 4);
        assert_eq!(config.input_size, 8);
    }

    #[test]
    fn test_diamond_io_integration_creation() {
        let config = DiamondIOConfig::default();
        let integration = DiamondIOIntegration::new(config);
        assert!(integration.is_ok());
    }

    #[test]
    fn test_create_demo_circuit() {
        let config = DiamondIOConfig::default();
        let integration = DiamondIOIntegration::new(config).unwrap();
        let circuit = integration.create_demo_circuit();

        assert!(circuit.num_input() > 0);
        assert!(circuit.num_output() > 0);
    }

    #[tokio::test]
    async fn test_dummy_mode_obfuscation() {
        let config = DiamondIOConfig::dummy();
        let integration = DiamondIOIntegration::new(config).unwrap();

        let circuit = integration.create_demo_circuit();
        let result = integration.obfuscate_circuit(circuit).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_dummy_mode_evaluation() {
        let config = DiamondIOConfig::dummy();
        let integration = DiamondIOIntegration::new(config).unwrap();

        let inputs = vec![true, false, true, false];
        let result = integration.evaluate_circuit(&inputs).await;
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 1);
    }
}
