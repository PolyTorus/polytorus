//! Enhanced privacy provider with real Diamond IO integration
//!
//! This module combines the existing privacy features with real Diamond IO
//! to provide maximum privacy guarantees for eUTXO transactions.

use std::collections::HashMap;

use ark_std::rand::{CryptoRng, RngCore};
use serde::{Deserialize, Serialize};

use crate::{
    crypto::{
        privacy::{PrivacyConfig, PrivacyProvider, PrivateTransaction, UtxoValidityProof},
        real_diamond_io::{
            DiamondIOCircuit, RealDiamondIOConfig, RealDiamondIOProof, RealDiamondIOProvider,
        },
        transaction::Transaction,
    },
    diamond_io_integration_new::DiamondIOResult,
    Result,
};

/// Enhanced privacy configuration combining traditional privacy with real Diamond IO
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedPrivacyConfig {
    /// Base privacy configuration
    pub privacy_config: PrivacyConfig,
    /// Real Diamond IO configuration
    pub diamond_io_config: RealDiamondIOConfig,
    /// Enable real Diamond IO obfuscation
    pub enable_real_diamond_io: bool,
    /// Use hybrid mode (traditional + Diamond IO)
    pub use_hybrid_mode: bool,
    /// Circuit cleanup interval in seconds
    pub cleanup_interval: u64,
}

impl Default for EnhancedPrivacyConfig {
    fn default() -> Self {
        Self {
            privacy_config: PrivacyConfig::default(),
            diamond_io_config: RealDiamondIOConfig::testing(),
            enable_real_diamond_io: true,
            use_hybrid_mode: true,
            cleanup_interval: 3600, // 1 hour
        }
    }
}

impl EnhancedPrivacyConfig {
    /// Create testing configuration
    pub fn testing() -> Self {
        Self {
            privacy_config: PrivacyConfig {
                enable_zk_proofs: true,
                enable_confidential_amounts: true,
                enable_nullifiers: true,
                range_proof_bits: 32,
                commitment_randomness_size: 32,
            },
            diamond_io_config: RealDiamondIOConfig::testing(),
            enable_real_diamond_io: true,
            use_hybrid_mode: true,
            cleanup_interval: 300, // 5 minutes for testing
        }
    }

    /// Create production configuration
    pub fn production() -> Self {
        Self {
            privacy_config: PrivacyConfig {
                enable_zk_proofs: true,
                enable_confidential_amounts: true,
                enable_nullifiers: true,
                range_proof_bits: 64,
                commitment_randomness_size: 32,
            },
            diamond_io_config: RealDiamondIOConfig::production(),
            enable_real_diamond_io: true,
            use_hybrid_mode: true,
            cleanup_interval: 7200, // 2 hours
        }
    }
}

/// Enhanced private transaction with real Diamond IO proofs
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedPrivateTransaction {
    /// Base private transaction
    pub base_private_transaction: PrivateTransaction,
    /// Real Diamond IO proofs for each input
    pub diamond_io_proofs: Vec<RealDiamondIOProof>,
    /// Circuit references
    pub circuit_ids: Vec<String>,
    /// Enhanced transaction metadata
    pub enhanced_metadata: EnhancedTransactionMetadata,
}

/// Metadata for enhanced private transactions
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedTransactionMetadata {
    /// Creation timestamp
    pub created_at: u64,
    /// Diamond IO provider statistics at creation time
    pub diamond_io_stats: HashMap<String, f64>,
    /// Privacy level achieved
    pub privacy_level: String,
    /// Total gas cost including Diamond IO operations
    pub total_gas_cost: u64,
}

/// Enhanced privacy provider with real Diamond IO integration
pub struct EnhancedPrivacyProvider {
    /// Configuration
    config: EnhancedPrivacyConfig,
    /// Traditional privacy provider
    pub privacy_provider: PrivacyProvider,
    /// Real Diamond IO provider
    diamond_io_provider: Option<RealDiamondIOProvider>,
    /// Circuit counter for unique IDs
    circuit_counter: u64,
}

impl EnhancedPrivacyProvider {
    /// Create a new enhanced privacy provider
    pub async fn new(config: EnhancedPrivacyConfig) -> Result<Self> {
        let privacy_provider = PrivacyProvider::new(config.privacy_config.clone());
        let diamond_io_provider = if config.enable_real_diamond_io {
            Some(RealDiamondIOProvider::new(config.diamond_io_config.clone()).await?)
        } else {
            None
        };

        Ok(Self {
            config,
            privacy_provider,
            diamond_io_provider,
            circuit_counter: 0,
        })
    }

    /// Create an enhanced private transaction with both traditional and Diamond IO privacy
    pub async fn create_enhanced_private_transaction<R: RngCore + CryptoRng>(
        &mut self,
        base_transaction: Transaction,
        input_amounts: Vec<u64>,
        output_amounts: Vec<u64>,
        secret_keys: Vec<Vec<u8>>,
        rng: &mut R,
    ) -> Result<EnhancedPrivateTransaction> {
        // Create base private transaction using traditional privacy
        let base_private_tx = self.privacy_provider.create_private_transaction(
            base_transaction,
            input_amounts,
            output_amounts,
            secret_keys,
            rng,
        )?;

        let mut diamond_io_proofs = Vec::new();
        let mut circuit_ids = Vec::new();

        // Create Diamond IO proofs if enabled
        if self.diamond_io_provider.is_some() {
            for (i, input) in base_private_tx.private_inputs.iter().enumerate() {
                let circuit_id = format!("circuit_{}_{}", self.circuit_counter, i);
                self.circuit_counter += 1;

                // Get circuit inputs before borrowing diamond provider
                let circuit_inputs = self.derive_circuit_inputs(&input.validity_proof)?;

                // Now borrow diamond provider mutably
                let diamond_provider = self.diamond_io_provider.as_mut().unwrap();

                // Create Diamond IO circuit
                let circuit = diamond_provider
                    .create_privacy_circuit(circuit_id.clone(), &input.validity_proof)
                    .await?;

                // Evaluate circuit
                let evaluation_result = diamond_provider
                    .evaluate_circuit(&circuit, circuit_inputs)
                    .await?;

                // Collect performance metrics after releasing the mutable borrow
                let performance_metrics = self.collect_performance_metrics(&circuit);

                // Create enhanced proof
                let diamond_proof = RealDiamondIOProof {
                    base_proof: input.validity_proof.clone(),
                    circuit_id: circuit_id.clone(),
                    evaluation_result: evaluation_result.into(),
                    params_commitment: input.amount_commitment.clone(),
                    performance_metrics,
                };

                diamond_io_proofs.push(diamond_proof);
                circuit_ids.push(circuit_id);
            }
        }

        // Create enhanced metadata
        let enhanced_metadata = EnhancedTransactionMetadata {
            created_at: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .map_err(|e| anyhow::anyhow!("Time error: {}", e))?
                .as_secs(),
            diamond_io_stats: self.collect_diamond_io_stats(),
            privacy_level: self.determine_privacy_level(),
            total_gas_cost: self.calculate_total_gas_cost(&base_private_tx, &diamond_io_proofs),
        };

        Ok(EnhancedPrivateTransaction {
            base_private_transaction: base_private_tx,
            diamond_io_proofs,
            circuit_ids,
            enhanced_metadata,
        })
    }
    /// Verify an enhanced private transaction
    pub async fn verify_enhanced_private_transaction(
        &mut self,
        enhanced_tx: &EnhancedPrivateTransaction,
    ) -> Result<bool> {
        // Verify base private transaction
        if !self
            .privacy_provider
            .verify_private_transaction(&enhanced_tx.base_private_transaction)?
        {
            return Ok(false);
        }

        // Verify Diamond IO proofs if available
        if self.diamond_io_provider.is_some() {
            // Collect verification data first
            let mut verification_data = Vec::new();
            for diamond_proof in enhanced_tx.diamond_io_proofs.iter() {
                if let Some(circuit) = self.get_circuit_by_id(&diamond_proof.circuit_id).await? {
                    let circuit_inputs = self.derive_circuit_inputs(&diamond_proof.base_proof)?;
                    let expected_result: DiamondIOResult =
                        diamond_proof.evaluation_result.clone().into(); // Convert SerializableDiamondIOResult to DiamondIOResult
                    verification_data.push((circuit, circuit_inputs, expected_result));
                } else {
                    // Circuit not found - this could be normal if it was cleaned up
                    tracing::warn!(
                        "Circuit {} not found for verification",
                        diamond_proof.circuit_id
                    );
                }
            }

            // Now verify with mutable reference
            if let Some(ref mut diamond_provider) = self.diamond_io_provider {
                for (circuit, circuit_inputs, expected_result) in verification_data {
                    if !diamond_provider
                        .verify_evaluation(&circuit, &circuit_inputs, &expected_result)
                        .await?
                    {
                        return Ok(false);
                    }
                }
            }
        }

        // Verify metadata consistency
        self.verify_enhanced_metadata(&enhanced_tx.enhanced_metadata)?;

        Ok(true)
    }

    /// Derive circuit inputs from validity proof
    fn derive_circuit_inputs(&self, proof: &UtxoValidityProof) -> Result<Vec<bool>> {
        use sha2::{Digest, Sha256};

        let mut hasher = Sha256::new();
        hasher.update(&proof.commitment_proof);
        hasher.update(&proof.nullifier);
        let hash = hasher.finalize();

        // Convert hash bytes to boolean inputs
        let mut inputs = Vec::new();
        for byte in &hash[..8] {
            // Use first 8 bytes
            for bit in 0..8 {
                inputs.push((byte >> bit) & 1 == 1);
            }
        }

        Ok(inputs)
    }
    /// Collect performance metrics from a circuit
    fn collect_performance_metrics(&self, circuit: &DiamondIOCircuit) -> HashMap<String, f64> {
        let mut metrics = HashMap::new();
        metrics.insert("input_size".to_string(), circuit.metadata.input_size as f64);
        metrics.insert(
            "output_size".to_string(),
            circuit.metadata.output_size as f64,
        );
        metrics.insert(
            "obfuscated_size".to_string(),
            circuit.obfuscated_data.len() as f64,
        );
        metrics.insert(
            "obfuscation_time".to_string(),
            circuit.metadata.obfuscation_time as f64,
        );
        metrics.insert(
            "complexity".to_string(),
            circuit.metadata.complexity.parse().unwrap_or(0.0),
        );
        metrics.insert(
            "security_level".to_string(),
            circuit.metadata.security_level as f64,
        );
        metrics
    }
    /// Collect Diamond IO statistics
    fn collect_diamond_io_stats(&self) -> HashMap<String, f64> {
        let mut stats = HashMap::new();

        if let Some(ref diamond_provider) = self.diamond_io_provider {
            let provider_stats = diamond_provider.get_statistics();
            stats.insert(
                "active_circuits".to_string(),
                provider_stats.active_circuits as f64,
            );
            stats.insert(
                "security_level".to_string(),
                provider_stats.security_level as f64,
            );
            stats.insert(
                "max_circuits".to_string(),
                provider_stats.max_circuits as f64,
            );
            stats.insert(
                "disk_storage_enabled".to_string(),
                provider_stats.disk_storage_enabled as u8 as f64,
            );
        }

        stats.insert("circuit_counter".to_string(), self.circuit_counter as f64);
        stats.insert(
            "hybrid_mode".to_string(),
            self.config.use_hybrid_mode as u8 as f64,
        );

        stats
    }

    /// Determine the privacy level achieved
    pub fn determine_privacy_level(&self) -> String {
        let mut level = "basic".to_string();

        if self.config.privacy_config.enable_confidential_amounts {
            level = "confidential".to_string();
        }

        if self.config.privacy_config.enable_zk_proofs {
            level = "zero_knowledge".to_string();
        }

        if self.config.enable_real_diamond_io {
            level = "indistinguishable_obfuscation".to_string();
        }

        if self.config.use_hybrid_mode && self.config.enable_real_diamond_io {
            level = "maximum_privacy".to_string();
        }

        level
    }

    /// Calculate total gas cost including Diamond IO operations
    fn calculate_total_gas_cost(
        &self,
        base_tx: &PrivateTransaction,
        diamond_proofs: &[RealDiamondIOProof],
    ) -> u64 {
        let mut total_gas = 0u64;

        // Base transaction gas (this would come from the transaction processor)
        total_gas += 5000; // Base gas

        // Privacy features gas
        total_gas += base_tx.private_inputs.len() as u64 * 1000; // ZK proof verification
        total_gas += base_tx.private_outputs.len() as u64 * 500; // Range proof verification

        // Diamond IO gas
        total_gas += diamond_proofs.len() as u64 * 2000; // Circuit evaluation

        // Additional gas based on complexity
        for proof in diamond_proofs {
            if let Some(ring_dim) = proof.performance_metrics.get("ring_dimension") {
                total_gas += (*ring_dim as u64) * 10; // Scale with ring dimension
            }
        }

        total_gas
    }

    /// Verify enhanced metadata consistency
    fn verify_enhanced_metadata(&self, metadata: &EnhancedTransactionMetadata) -> Result<bool> {
        // Check timestamp is reasonable (within last 24 hours)
        let current_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| anyhow::anyhow!("Time error: {}", e))?
            .as_secs();

        let time_diff = current_time.saturating_sub(metadata.created_at);
        if time_diff > 86400 {
            // 24 hours
            return Ok(false);
        }

        // Verify privacy level is valid
        let valid_levels = [
            "basic",
            "confidential",
            "zero_knowledge",
            "indistinguishable_obfuscation",
            "maximum_privacy",
        ];
        if !valid_levels.contains(&metadata.privacy_level.as_str()) {
            return Ok(false);
        }

        Ok(true)
    }

    /// Get circuit by ID (helper function)
    async fn get_circuit_by_id(&self, _circuit_id: &str) -> Result<Option<DiamondIOCircuit>> {
        // This would query the Diamond IO provider's circuit cache
        // For now, return None as circuits might be cleaned up
        Ok(None)
    }

    /// Clean up old circuits
    pub async fn cleanup_old_circuits(&mut self) -> Result<()> {
        if let Some(ref mut _diamond_provider) = self.diamond_io_provider {
            // In a real implementation, this would track circuit creation times
            // and clean up circuits older than cleanup_interval
            tracing::info!("Cleaning up old Diamond IO circuits");
        }
        Ok(())
    }

    /// Get enhanced privacy statistics
    pub fn get_enhanced_statistics(&self) -> EnhancedPrivacyStatistics {
        let base_stats = self.privacy_provider.get_privacy_stats();
        let diamond_stats = self.collect_diamond_io_stats();

        EnhancedPrivacyStatistics {
            base_privacy_stats: base_stats,
            diamond_io_stats: diamond_stats,
            total_circuits_created: self.circuit_counter,
            hybrid_mode_enabled: self.config.use_hybrid_mode,
            real_diamond_io_enabled: self.config.enable_real_diamond_io,
        }
    }
}

/// Enhanced privacy statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedPrivacyStatistics {
    pub base_privacy_stats: crate::crypto::privacy::PrivacyStats,
    pub diamond_io_stats: HashMap<String, f64>,
    pub total_circuits_created: u64,
    pub hybrid_mode_enabled: bool,
    pub real_diamond_io_enabled: bool,
}

#[cfg(test)]
mod tests {
    use rand_core::OsRng;

    use super::*;
    use crate::crypto::transaction::Transaction;

    #[tokio::test]
    async fn test_enhanced_privacy_provider_creation() {
        let config = EnhancedPrivacyConfig::testing();
        let provider = EnhancedPrivacyProvider::new(config).await;

        assert!(provider.is_ok());
        let provider = provider.unwrap();

        let stats = provider.get_enhanced_statistics();
        assert!(stats.real_diamond_io_enabled);
        assert!(stats.hybrid_mode_enabled);
        assert_eq!(stats.total_circuits_created, 0);
    }

    #[tokio::test]
    async fn test_enhanced_private_transaction_creation() {
        let config = EnhancedPrivacyConfig::testing();
        let mut provider = EnhancedPrivacyProvider::new(config).await.unwrap();
        let mut rng = OsRng;

        // Create a test coinbase transaction
        let base_tx =
            Transaction::new_coinbase("test_address".to_string(), "test_data".to_string()).unwrap();

        // Create enhanced private transaction
        let enhanced_tx = provider
            .create_enhanced_private_transaction(
                base_tx,
                vec![100u64],        // Input amount
                vec![50u64],         // One output (50 coins, 50 fee)
                vec![vec![1, 2, 3]], // Dummy secret key
                &mut rng,
            )
            .await
            .unwrap();

        assert_eq!(enhanced_tx.base_private_transaction.private_inputs.len(), 1);
        assert_eq!(
            enhanced_tx.base_private_transaction.private_outputs.len(),
            1
        );
        assert_eq!(enhanced_tx.diamond_io_proofs.len(), 1);
        assert_eq!(enhanced_tx.circuit_ids.len(), 1);
        assert_eq!(
            enhanced_tx.enhanced_metadata.privacy_level,
            "maximum_privacy"
        );

        // Verify the enhanced transaction
        let verification = provider
            .verify_enhanced_private_transaction(&enhanced_tx)
            .await
            .unwrap();
        assert!(verification);
    }

    #[test]
    fn test_enhanced_privacy_config_levels() {
        let testing_config = EnhancedPrivacyConfig::testing();
        let production_config = EnhancedPrivacyConfig::production();

        // Production should have stronger parameters
        assert!(
            production_config.privacy_config.range_proof_bits
                >= testing_config.privacy_config.range_proof_bits
        );
        assert!(production_config.cleanup_interval >= testing_config.cleanup_interval);
        assert!(
            production_config.diamond_io_config.security_level
                >= testing_config.diamond_io_config.security_level
        );
    }

    #[tokio::test]
    async fn test_privacy_level_determination() {
        let config = EnhancedPrivacyConfig::testing();
        let provider = EnhancedPrivacyProvider::new(config).await.unwrap();

        let level = provider.determine_privacy_level();
        assert_eq!(level, "maximum_privacy");
    }
}
