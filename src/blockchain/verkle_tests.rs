//! Verkle Tree integration tests for blockchain

use crate::blockchain::block::*;
use crate::blockchain::types::{block_states, network};
use crate::crypto::transaction::*;
use crate::crypto::verkle_tree::*;
use std::collections::HashMap;

#[cfg(test)]
mod verkle_integration_tests {
    use super::*;

    fn create_test_transaction(from: &str, to: &str, amount: i64) -> Transaction {
        Transaction::new(
            from.to_string(),
            to.to_string(),
            amount,
            10, // fee
            vec![TxInput::new("prev_hash".to_string(), 0, "sig".to_string())],
            vec![TxOutput::new(amount, to.to_string())],
        ).unwrap()
    }

    #[test]
    fn test_verkle_tree_in_block_creation() {
        // Create test transactions
        let tx1 = create_test_transaction("alice", "bob", 100);
        let tx2 = create_test_transaction("bob", "charlie", 50);
        let tx3 = create_test_transaction("charlie", "dave", 25);
        
        let transactions = vec![tx1, tx2, tx3];
        
        // Create a building block
        let mut block: BuildingBlock<network::Mainnet> = Block::new_building(
            transactions.clone(),
            "prev_hash".to_string(),
            1,
            4,
        );
        
        // Build the Verkle tree
        let tree = block.get_or_build_verkle_tree().unwrap();
        
        // Verify the tree is not empty
        assert!(!tree.get_root_commitment().0.is_zero());
        
        // Get the root commitment
        let root_commitment = block.get_verkle_root_commitment().unwrap();
        assert!(!root_commitment.is_empty());
    }

    #[test]
    fn test_verkle_proof_generation_and_verification() {
        // Create test transactions
        let tx1 = create_test_transaction("alice", "bob", 100);
        let tx2 = create_test_transaction("bob", "charlie", 50);
        
        let transactions = vec![tx1.clone(), tx2.clone()];
        
        // Create a building block
        let mut block: BuildingBlock<network::Mainnet> = Block::new_building(
            transactions,
            "prev_hash".to_string(),
            1,
            4,
        );
        
        // Generate proof for the first transaction
        let proof = block.generate_transaction_proof(0).unwrap();
        
        // Verify the proof
        assert!(block.verify_transaction_proof(&proof));
        
        // Generate proof for the second transaction
        let proof2 = block.generate_transaction_proof(1).unwrap();
        
        // Verify the second proof
        assert!(block.verify_transaction_proof(&proof2));
        
        // Verify that the proofs are different
        assert_ne!(proof.key, proof2.key);
    }

    #[test]
    fn test_verkle_tree_consistency_across_block_states() {
        // Create test transactions
        let tx1 = create_test_transaction("alice", "bob", 100);
        let tx2 = create_test_transaction("bob", "charlie", 50);
        
        let transactions = vec![tx1, tx2];
        
        // Create building block
        let mut building_block: BuildingBlock<network::Mainnet> = Block::new_building(
            transactions,
            "prev_hash".to_string(),
            1,
            1, // Use low difficulty for faster mining
        );
        
        // Get the Verkle tree root commitment before mining
        let original_commitment = building_block.get_verkle_root_commitment().unwrap();
        
        // Mine the block
        let mined_block = building_block.mine().unwrap();
        
        // Verify the commitment is preserved
        assert_eq!(mined_block.verkle_root_commitment.unwrap(), original_commitment);
        
        // Validate the block
        let mut validated_block = mined_block.validate().unwrap();
        
        // Verify the commitment is still preserved
        assert_eq!(validated_block.verkle_root_commitment.as_ref().unwrap(), &original_commitment);
        
        // Generate a proof on the validated block
        let proof = validated_block.generate_transaction_proof(0).unwrap();
        assert!(validated_block.verify_transaction_proof(&proof));
        
        // Finalize the block
        let finalized_block = validated_block.finalize();
        
        // Verify the commitment is still preserved
        assert_eq!(finalized_block.verkle_root_commitment.as_ref().unwrap(), &original_commitment);
    }

    #[test]
    fn test_verkle_tree_with_empty_transactions() {
        // Create a block with no transactions
        let mut block: BuildingBlock<network::Mainnet> = Block::new_building(
            vec![],
            "prev_hash".to_string(),
            1,
            4,
        );
        
        // Build the Verkle tree
        let tree = block.get_or_build_verkle_tree().unwrap();
        
        // Verify the tree has identity root for empty tree
        let root_commitment = tree.get_root_commitment();
        // The root should be the identity element for empty tree
        assert_eq!(root_commitment.0, VerklePoint::identity().0);
    }

    #[test]
    fn test_verkle_tree_with_large_number_of_transactions() {
        // Create many test transactions
        let mut transactions = Vec::new();
        for i in 0..100 {
            let tx = create_test_transaction(
                &format!("user_{}", i),
                &format!("user_{}", i + 1),
                i as i64 + 1,
            );
            transactions.push(tx);
        }
        
        // Create a building block
        let mut block: BuildingBlock<network::Mainnet> = Block::new_building(
            transactions,
            "prev_hash".to_string(),
            1,
            4,
        );
        
        // Build the Verkle tree
        let tree = block.get_or_build_verkle_tree().unwrap();
        
        // Verify the tree is not empty
        assert!(!tree.get_root_commitment().0.is_zero());
        
        // Generate proofs for several transactions
        for i in [0, 25, 50, 75, 99] {
            let proof = block.generate_transaction_proof(i).unwrap();
            assert!(block.verify_transaction_proof(&proof));
        }
    }

    #[test]
    fn test_verkle_tree_deterministic_commitment() {
        // Create same transactions in two different blocks
        let tx1 = create_test_transaction("alice", "bob", 100);
        let tx2 = create_test_transaction("bob", "charlie", 50);
        
        let transactions = vec![tx1.clone(), tx2.clone()];
        
        // Create two identical blocks
        let mut block1: BuildingBlock<network::Mainnet> = Block::new_building(
            transactions.clone(),
            "prev_hash".to_string(),
            1,
            4,
        );
        
        let mut block2: BuildingBlock<network::Mainnet> = Block::new_building(
            transactions,
            "prev_hash".to_string(),
            1,
            4,
        );
        
        // Get commitments from both blocks
        let commitment1 = block1.get_verkle_root_commitment().unwrap();
        let commitment2 = block2.get_verkle_root_commitment().unwrap();
        
        // Commitments should be identical for identical transaction sets
        assert_eq!(commitment1, commitment2);
    }

    #[test]
    fn test_verkle_proof_size_efficiency() {
        // Create test transactions
        let tx1 = create_test_transaction("alice", "bob", 100);
        let tx2 = create_test_transaction("bob", "charlie", 50);
        let tx3 = create_test_transaction("charlie", "dave", 25);
        
        let transactions = vec![tx1, tx2, tx3];
        
        // Create a building block
        let mut block: BuildingBlock<network::Mainnet> = Block::new_building(
            transactions,
            "prev_hash".to_string(),
            1,
            4,
        );
        
        // Generate proof for a transaction
        let proof = block.generate_transaction_proof(0).unwrap();
        
        // Check proof size (should be reasonably small)
        let proof_size = proof.size();
        assert!(proof_size > 0);
        println!("Verkle proof size: {} bytes", proof_size);
        
        // Proof should be reasonably compact (less than 10KB for small trees)
        assert!(proof_size < 10_000);
    }

    #[test]
    fn test_verkle_tree_invalid_proof_rejection() {
        // Create test transactions
        let tx1 = create_test_transaction("alice", "bob", 100);
        let tx2 = create_test_transaction("bob", "charlie", 50);
        
        let transactions = vec![tx1, tx2];
        
        // Create building block
        let mut block: BuildingBlock<network::Mainnet> = Block::new_building(
            transactions,
            "prev_hash".to_string(),
            1,
            4,
        );
        
        // Generate a valid proof
        let mut proof = block.generate_transaction_proof(0).unwrap();
        
        // Verify the original proof is valid
        assert!(block.verify_transaction_proof(&proof));
        
        // Tamper with the proof
        proof.key = b"tampered_key".to_vec();
        
        // Verify the tampered proof is rejected
        assert!(!block.verify_transaction_proof(&proof));
    }
}

#[cfg(test)]
mod verkle_tree_unit_tests {
    use super::*;

    #[test]
    fn test_verkle_tree_basic_operations() {
        let mut tree = VerkleTree::new();
        
        // Test insertion
        assert!(tree.insert(b"key1", b"value1").is_ok());
        assert!(tree.insert(b"key2", b"value2").is_ok());
        
        // Test retrieval
        assert_eq!(tree.get(b"key1"), Some(b"value1".to_vec()));
        assert_eq!(tree.get(b"key2"), Some(b"value2".to_vec()));
        assert_eq!(tree.get(b"nonexistent"), None);
    }

    #[test]
    fn test_verkle_tree_proof_generation() {
        let mut tree = VerkleTree::new();
        
        tree.insert(b"key1", b"value1").unwrap();
        tree.insert(b"key2", b"value2").unwrap();
        
        // Generate proof for existing key
        let proof = tree.generate_proof(b"key1").unwrap();
        assert_eq!(proof.key, b"key1");
        assert_eq!(proof.value, Some(b"value1".to_vec()));
        assert!(tree.verify_proof(&proof));
        
        // Generate proof for non-existing key
        let proof_nonexistent = tree.generate_proof(b"nonexistent").unwrap();
        assert_eq!(proof_nonexistent.key, b"nonexistent");
        assert_eq!(proof_nonexistent.value, None);
        assert!(tree.verify_proof(&proof_nonexistent));
    }

    #[test]
    fn test_verkle_tree_commitment_changes() {
        let mut tree = VerkleTree::new();
        
        // Get initial commitment (should be identity)
        let initial_commitment = tree.get_root_commitment();
        
        // Insert a key-value pair
        tree.insert(b"key1", b"value1").unwrap();
        let commitment_after_insert = tree.get_root_commitment();
        
        // Commitment should change after insertion
        assert_ne!(initial_commitment.0, commitment_after_insert.0);
        
        // Insert another key-value pair
        tree.insert(b"key2", b"value2").unwrap();
        let commitment_after_second_insert = tree.get_root_commitment();
        
        // Commitment should change again
        assert_ne!(commitment_after_insert.0, commitment_after_second_insert.0);
        
        // Delete a key
        tree.delete(b"key1").unwrap();
        let commitment_after_delete = tree.get_root_commitment();
        
        // Commitment should change after deletion
        assert_ne!(commitment_after_second_insert.0, commitment_after_delete.0);
    }
}
