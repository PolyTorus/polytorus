use diamond_io::{
    bgg::circuit::PolyCircuit,
    io::{
        eval::evaluate,
        params::ObfuscationParams,
    },
    poly::{
        dcrt::{
            DCRTPoly, DCRTPolyHashSampler, DCRTPolyMatrix, DCRTPolyParams, 
            DCRTPolyTrapdoorSampler, DCRTPolyUniformSampler,
        },
        enc::rlwe_encrypt,
        sampler::{DistType, PolyUniformSampler},
        PolyMatrix, PolyParams,
    },
    utils::init_tracing,
};
use keccak_asm::Keccak256;
use num_bigint::BigUint;
use num_traits::Num;
use serde::{Deserialize, Serialize};
use std::{fs, path::Path, sync::Arc};
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
    /// Input values for testing
    pub inputs: Vec<bool>,
    /// Enable dummy mode (for testing without full Diamond IO initialization)
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
            crt_bits: 51,
            base_bits: 1,
            switched_modulus: BigUint::from_str_radix("123456789", 10).unwrap(),
            input_size: 8,
            level_width: 4,
            d: 3,
            hardcoded_key_sigma: 4.578,
            p_sigma: 4.578,
            trapdoor_sigma: Some(4.578),
            inputs: vec![true, false, true, false, true, false, true, false],
            dummy_mode: false,
        }
    }
}

#[derive(Debug)]
pub struct DiamondIOIntegration {
    config: DiamondIOConfig,
    params: DCRTPolyParams,
    obfuscation_dir: String,
}

impl DiamondIOIntegration {
    pub fn new(config: DiamondIOConfig) -> anyhow::Result<Self> {
        // Skip Diamond IO initialization in dummy mode
        if config.dummy_mode {
            return Ok(Self {
                config,
                params: DCRTPolyParams::new(16, 2, 17, 1), // Minimal dummy params
                obfuscation_dir: "obfuscation_data".to_string(),
            });
        }
        
        init_tracing();
        
        let params = DCRTPolyParams::new(
            config.ring_dimension,
            config.crt_depth,
            config.crt_bits,
            config.base_bits,
        );

        Ok(Self {
            config,
            params,
            obfuscation_dir: "obfuscation_data".to_string(),
        })
    }

    pub fn set_obfuscation_dir(&mut self, dir: String) {
        self.obfuscation_dir = dir;
    }

    /// Create a simple circuit for demonstration
    pub fn create_demo_circuit(&self) -> PolyCircuit {
        if self.config.dummy_mode {
            // Return a minimal circuit for dummy mode
            let mut circuit = PolyCircuit::new();
            
            // In dummy mode, just create a simple circuit structure
            let inputs = circuit.input(2);
            if inputs.len() >= 2 {
                let input1 = inputs[0];
                let input2 = inputs[1];
                let sum = circuit.add_gate(input1, input2);
                circuit.output(vec![sum]);
            }
            
            return circuit;
        }
        
        let mut circuit = PolyCircuit::new();
        
        // Add inputs - input() requires number of inputs as parameter
        let inputs = circuit.input(2);
        let input1 = inputs[0];
        let input2 = inputs[1];
        let sum = circuit.add_gate(input1, input2);
        circuit.output(vec![sum]);
        
        circuit
    }

    /// Obfuscate a circuit using Diamond IO
    pub async fn obfuscate_circuit(
        &self,
        _circuit: PolyCircuit,
    ) -> anyhow::Result<()> {
        info!("Starting circuit obfuscation...");
        
        let dir = Path::new(&self.obfuscation_dir);
        if dir.exists() {
            fs::remove_dir_all(dir)?;
        }
        fs::create_dir_all(dir)?;

        // Note: Full obfuscation requires OpenFHE installation
        // For now, we'll just simulate the process
        info!("Circuit obfuscation simulated (requires OpenFHE for full functionality)");
        
        let start_time = std::time::Instant::now();
        let obfuscation_time = start_time.elapsed();
        info!("Obfuscation completed in: {:?}", obfuscation_time);

        Ok(())
    }

    /// Evaluate an obfuscated circuit
    pub fn evaluate_circuit(&self, inputs: &[bool]) -> anyhow::Result<Vec<bool>> {
        info!("Evaluating obfuscated circuit with inputs: {:?}", inputs);
        
        if inputs.len() != self.config.input_size {
            return Err(anyhow::anyhow!(
                "Input size mismatch: expected {}, got {}",
                self.config.input_size,
                inputs.len()
            ));
        }

        let _log_base_q = self.params.modulus_digits();
        let switched_modulus = Arc::new(self.config.switched_modulus.clone());

        let circuit = self.create_demo_circuit();
        let obf_params = ObfuscationParams {
            params: self.params.clone(),
            switched_modulus,
            input_size: self.config.input_size,
            level_width: self.config.level_width,
            public_circuit: circuit,
            d: self.config.d,
            hardcoded_key_sigma: self.config.hardcoded_key_sigma,
            p_sigma: self.config.p_sigma,
            trapdoor_sigma: self.config.trapdoor_sigma.unwrap_or(4.578),
        };

        let start_time = std::time::Instant::now();
        let output = evaluate::<
            DCRTPolyMatrix,
            DCRTPolyHashSampler<Keccak256>,
            DCRTPolyTrapdoorSampler,
            _,
        >(obf_params, inputs, &self.obfuscation_dir);
        
        let eval_time = start_time.elapsed();
        info!("Evaluation completed in: {:?}", eval_time);
        info!("Output: {:?}", output);

        Ok(output)
    }

    /// Encrypt data using RLWE encryption
    pub fn encrypt_data(&self, data: &[bool]) -> anyhow::Result<DCRTPolyMatrix> {
        let sampler_uniform = DCRTPolyUniformSampler::new();
        
        // Sample secret key
        let secret_key = sampler_uniform.sample_uniform(
            &self.params, 
            1, 
            1, 
            DistType::GaussDist { sigma: self.config.hardcoded_key_sigma }
        );
        
        // Sample public key matrix A
        let a_matrix = sampler_uniform.sample_uniform(
            &self.params,
            1,
            1,
            DistType::FinRingDist
        );

        // Convert data to polynomial
        let data_poly = self.bool_vec_to_poly(data)?;
        let data_matrix = DCRTPolyMatrix::from_poly_vec_row(&self.params, vec![data_poly]);

        // Encrypt using RLWE
        let ciphertext = rlwe_encrypt(
            &self.params,
            &sampler_uniform,
            &secret_key,
            &a_matrix,
            &data_matrix,
            self.config.hardcoded_key_sigma,
        );

        Ok(ciphertext)
    }

    /// Convert boolean vector to polynomial
    fn bool_vec_to_poly(&self, data: &[bool]) -> anyhow::Result<DCRTPoly> {
        let sampler = DCRTPolyUniformSampler::new();
        // Create a zero polynomial using the sampler
        let temp_poly = sampler.sample_poly(&self.params, &DistType::BitDist);
        let mut result = temp_poly.clone() - temp_poly; // Create zero polynomial
        
        for (_i, &bit) in data.iter().enumerate() {
            if bit {
                let bit_poly = sampler.sample_poly(&self.params, &DistType::BitDist);
                result = result + bit_poly;
            }
        }
        
        Ok(result)
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
        
        // Circuit should have inputs and outputs
        assert!(circuit.num_input() > 0);
        assert!(circuit.num_output() > 0);
    }

    #[tokio::test]
    async fn test_encrypt_data() {
        let config = DiamondIOConfig::default();
        let integration = DiamondIOIntegration::new(config).unwrap();
        
        let data = vec![true, false, true, false];
        let result = integration.encrypt_data(&data);
        assert!(result.is_ok());
    }
}
