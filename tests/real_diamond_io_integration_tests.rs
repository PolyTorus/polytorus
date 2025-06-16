//! Integration tests for real Diamond IO privacy features
//!
//! These tests verify the complete integration between PolyTorus privacy features
//! and the real Diamond IO library from MachinaIO.

use std::collections::HashMap;

use polytorus::crypto::privacy::{
    PedersenCommitment,
    UtxoValidityProof,
};
use polytorus::crypto::real_diamond_io::{
    RealDiamondIOConfig,
    RealDiamondIOProof,
    RealDiamondIOProvider,
    SerializableDiamondIOResult,
};
use tokio;

#[tokio::test]
async fn test_real_diamond_io_provider_creation() {
    let config = RealDiamondIOConfig::testing();

    // Create provider
    let provider = RealDiamondIOProvider::new(config)
        .await
        .expect("Failed to create Diamond IO provider");

    // Check initial statistics
    let stats = provider.get_statistics();
    assert_eq!(stats.active_circuits, 0);
    assert_eq!(stats.security_level, 64);
    assert_eq!(stats.max_circuits, 10);
}

#[tokio::test]
async fn test_circuit_creation_and_evaluation() {
    let config = RealDiamondIOConfig::testing();
    let mut provider = RealDiamondIOProvider::new(config)
        .await
        .expect("Failed to create Diamond IO provider");

    // Create test proof
    let test_proof = UtxoValidityProof {
        commitment_proof: vec![1, 2, 3, 4, 5],
        range_proof: vec![6, 7, 8, 9, 10],
        nullifier: vec![11, 12, 13, 14, 15],
        params_hash: vec![16, 17, 18, 19, 20],
    };

    // Create circuit
    let circuit = provider
        .create_privacy_circuit("test_circuit".to_string(), &test_proof)
        .await
        .expect("Failed to create privacy circuit");

    // Verify circuit properties
    assert_eq!(circuit.circuit_id, "test_circuit");
    assert_eq!(circuit.metadata.input_size, 4);
    assert_eq!(circuit.metadata.security_level, 64);
    assert_eq!(circuit.metadata.complexity, "privacy_circuit");

    // Test circuit evaluation
    let test_inputs = vec![true, false, true, true];
    let evaluation_result = provider
        .evaluate_circuit(&circuit, test_inputs.clone())
        .await
        .expect("Failed to evaluate circuit");

    // Verify evaluation result
    assert!(!evaluation_result.outputs.is_empty());
}

#[tokio::test]
async fn test_privacy_proof_creation() {
    let config = RealDiamondIOConfig::testing();
    let mut provider = RealDiamondIOProvider::new(config)
        .await
        .expect("Failed to create Diamond IO provider");

    // Create test proof
    let test_proof = UtxoValidityProof {
        commitment_proof: vec![1, 2, 3, 4],
        range_proof: vec![5, 6, 7, 8],
        nullifier: vec![9, 10, 11, 12],
        params_hash: vec![13, 14, 15, 16],
    };

    // Create privacy proof
    let diamond_proof = provider
        .create_privacy_proof("test_proof".to_string(), test_proof.clone())
        .await
        .expect("Failed to create privacy proof");

    // Verify proof structure
    assert_eq!(diamond_proof.circuit_id, "test_proof");
    assert_eq!(
        diamond_proof.base_proof.commitment_proof,
        test_proof.commitment_proof
    );
    assert!(!diamond_proof.evaluation_result.outputs.is_empty());
    assert!(!diamond_proof.performance_metrics.is_empty());
}

#[tokio::test]
async fn test_proof_serialization() {
    let test_base_proof = UtxoValidityProof {
        commitment_proof: vec![1, 2, 3],
        range_proof: vec![4, 5, 6],
        nullifier: vec![7, 8, 9],
        params_hash: vec![10, 11, 12],
    };

    let test_evaluation_result = SerializableDiamondIOResult {
        outputs: vec![true, false, true],
        execution_time: 123.45,
        circuit_id: "test".to_string(),
        metadata: HashMap::new(),
    };

    let diamond_proof = RealDiamondIOProof {
        base_proof: test_base_proof,
        circuit_id: "test".to_string(),
        evaluation_result: test_evaluation_result,
        params_commitment: PedersenCommitment {
            commitment: vec![13, 14, 15],
            blinding_factor: vec![16, 17, 18],
        },
        performance_metrics: {
            let mut metrics = HashMap::new();
            metrics.insert("security_level".to_string(), 64.0);
            metrics
        },
    };

    // Test JSON serialization
    let json_serialized =
        serde_json::to_string(&diamond_proof).expect("Failed to serialize proof to JSON");
    assert!(!json_serialized.is_empty());

    let json_deserialized: RealDiamondIOProof =
        serde_json::from_str(&json_serialized).expect("Failed to deserialize proof from JSON");

    assert_eq!(json_deserialized.circuit_id, "test");
    assert_eq!(
        json_deserialized.evaluation_result.outputs,
        vec![true, false, true]
    );
    assert_eq!(json_deserialized.evaluation_result.execution_time, 123.45);
}

#[tokio::test]
async fn test_config_levels() {
    let testing_config = RealDiamondIOConfig::testing();
    let production_config = RealDiamondIOConfig::production();

    // Verify configuration differences
    assert!(testing_config.security_level <= production_config.security_level);
    assert!(testing_config.max_circuits <= production_config.max_circuits);
    assert_eq!(testing_config.proof_system, "dummy");
    assert_eq!(production_config.proof_system, "groth16");
}
