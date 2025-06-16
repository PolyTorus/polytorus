//! Verkle Tree implementation for efficient state commitment and proofs

use std::fmt;

use ark_ec::{CurveGroup, PrimeGroup};
use ark_ed_on_bls12_381::{EdwardsAffine, EdwardsProjective, Fr};
#[cfg(test)]
use ark_ff::One;
use ark_ff::{PrimeField, Zero};
use ark_serialize::{CanonicalDeserialize, CanonicalSerialize};
use ark_std::{collections::BTreeMap, vec::Vec};
use blake3;
use serde::{Deserialize, Deserializer, Serialize, Serializer};
use tiny_keccak::{Hasher, Keccak};

/// Width of the Verkle tree (number of children per node)
pub const VERKLE_WIDTH: usize = 256;

/// Maximum depth of the Verkle tree
pub const MAX_VERKLE_DEPTH: usize = 32;

/// Elliptic curve point used in Verkle tree
#[derive(Clone, Debug, PartialEq, Eq)]
pub struct VerklePoint(pub EdwardsProjective);

impl VerklePoint {
    pub fn new(point: EdwardsProjective) -> Self {
        VerklePoint(point)
    }

    pub fn identity() -> Self {
        VerklePoint(EdwardsProjective::zero())
    }

    pub fn generator() -> Self {
        VerklePoint(<EdwardsProjective as PrimeGroup>::generator())
    }

    pub fn add(&self, other: &VerklePoint) -> VerklePoint {
        VerklePoint(self.0 + other.0)
    }

    pub fn scalar_mul(&self, scalar: &Fr) -> VerklePoint {
        VerklePoint(self.0 * scalar)
    }

    pub fn to_affine(&self) -> EdwardsAffine {
        self.0.into_affine()
    }
}

impl Serialize for VerklePoint {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        let mut bytes = Vec::new();
        self.0
            .serialize_compressed(&mut bytes)
            .map_err(serde::ser::Error::custom)?;
        serializer.serialize_bytes(&bytes)
    }
}

impl<'de> Deserialize<'de> for VerklePoint {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let bytes: Vec<u8> = Vec::deserialize(deserializer)?;
        let point = EdwardsProjective::deserialize_compressed(&bytes[..])
            .map_err(serde::de::Error::custom)?;
        Ok(VerklePoint(point))
    }
}

/// Verkle tree node
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum VerkleNode {
    /// Internal node with children commitments
    Internal {
        commitment: VerklePoint,
        children: BTreeMap<u8, Box<VerkleNode>>,
    },
    /// Leaf node with key-value pairs
    Leaf {
        commitment: VerklePoint,
        values: BTreeMap<Vec<u8>, Vec<u8>>,
    },
    /// Empty node
    Empty,
}

impl VerkleNode {
    /// Get the commitment of this node
    pub fn get_commitment(&self) -> VerklePoint {
        match self {
            VerkleNode::Internal { commitment, .. } => commitment.clone(),
            VerkleNode::Leaf { commitment, .. } => commitment.clone(),
            VerkleNode::Empty => VerklePoint::identity(),
        }
    }

    /// Check if node is empty
    pub fn is_empty(&self) -> bool {
        matches!(self, VerkleNode::Empty)
    }
}

/// Verkle tree structure
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VerkleTree {
    root: VerkleNode,
    /// Generator points for polynomial commitment
    generators: Vec<VerklePoint>,
}

impl VerkleTree {
    /// Create a new empty Verkle tree
    pub fn new() -> Self {
        let generators = Self::generate_generators();
        VerkleTree {
            root: VerkleNode::Empty,
            generators,
        }
    }

    /// Generate random generator points for the polynomial commitment scheme
    fn generate_generators() -> Vec<VerklePoint> {
        let mut generators = Vec::with_capacity(VERKLE_WIDTH + 1);
        let base_generator = VerklePoint::generator();

        // Use a deterministic seed for reproducible generators
        let seed = b"verkle_tree_generators_polytorus_blockchain_2025";
        let mut hasher = Keccak::v256();
        hasher.update(seed);
        let mut hash = [0u8; 32];
        hasher.finalize(&mut hash);

        // Generate base point
        generators.push(base_generator.clone());

        // Generate additional points by hashing
        for i in 1..=VERKLE_WIDTH {
            let mut next_hasher = Keccak::v256();
            next_hasher.update(&hash);
            next_hasher.update(&i.to_le_bytes());
            next_hasher.finalize(&mut hash);

            // Convert hash to field element
            let scalar = Fr::from_le_bytes_mod_order(&hash);
            let point = base_generator.scalar_mul(&scalar);
            generators.push(point);
        }

        generators
    }

    /// Insert a key-value pair into the tree
    pub fn insert(&mut self, key: &[u8], value: &[u8]) -> Result<(), VerkleError> {
        let key_hash = self.hash_key(key);
        self.root = self.insert_recursive(&self.root, &key_hash, key, value, 0)?;
        Ok(())
    }

    /// Get a value from the tree
    pub fn get(&self, key: &[u8]) -> Option<Vec<u8>> {
        let key_hash = self.hash_key(key);
        self.get_recursive(&self.root, &key_hash, key, 0)
    }

    /// Delete a key from the tree
    pub fn delete(&mut self, key: &[u8]) -> Result<Option<Vec<u8>>, VerkleError> {
        let key_hash = self.hash_key(key);
        let (new_root, deleted_value) = self.delete_recursive(&self.root, &key_hash, key, 0)?;
        self.root = new_root;
        Ok(deleted_value)
    }

    /// Get the root commitment
    pub fn get_root_commitment(&self) -> VerklePoint {
        self.root.get_commitment()
    }

    /// Generate a proof for a key
    pub fn generate_proof(&self, key: &[u8]) -> Result<VerkleProof, VerkleError> {
        let key_hash = self.hash_key(key);
        let mut path = Vec::new();
        let value = self.generate_proof_recursive(&self.root, &key_hash, key, 0, &mut path)?;

        Ok(VerkleProof {
            key: key.to_vec(),
            value,
            path,
            root_commitment: self.get_root_commitment(),
        })
    }

    /// Verify a proof
    pub fn verify_proof(&self, proof: &VerkleProof) -> bool {
        // Reconstruct the path and verify commitments
        let key_hash = self.hash_key(&proof.key);
        self.verify_proof_recursive(
            &proof.path,
            &key_hash,
            &proof.key,
            &proof.value,
            0,
            &proof.root_commitment,
        )
    }

    /// Hash a key to determine its path in the tree
    fn hash_key(&self, key: &[u8]) -> Vec<u8> {
        blake3::hash(key).as_bytes().to_vec()
    }

    /// Recursive insertion
    fn insert_recursive(
        &self,
        node: &VerkleNode,
        key_hash: &[u8],
        key: &[u8],
        value: &[u8],
        depth: usize,
    ) -> Result<VerkleNode, VerkleError> {
        if depth >= MAX_VERKLE_DEPTH {
            return Err(VerkleError::MaxDepthExceeded);
        }

        match node {
            VerkleNode::Empty => {
                // Create new leaf
                let mut values = BTreeMap::new();
                values.insert(key.to_vec(), value.to_vec());
                let commitment = self.compute_leaf_commitment(&values)?;
                Ok(VerkleNode::Leaf { commitment, values })
            }
            VerkleNode::Leaf { values, .. } => {
                let mut new_values = values.clone();
                new_values.insert(key.to_vec(), value.to_vec());
                let commitment = self.compute_leaf_commitment(&new_values)?;
                Ok(VerkleNode::Leaf {
                    commitment,
                    values: new_values,
                })
            }
            VerkleNode::Internal { children, .. } => {
                let child_index = key_hash[depth];
                let child = children
                    .get(&child_index)
                    .map(|c| c.as_ref())
                    .unwrap_or(&VerkleNode::Empty);

                let new_child = self.insert_recursive(child, key_hash, key, value, depth + 1)?;

                let mut new_children = children.clone();
                new_children.insert(child_index, Box::new(new_child));

                let commitment = self.compute_internal_commitment(&new_children)?;
                Ok(VerkleNode::Internal {
                    commitment,
                    children: new_children,
                })
            }
        }
    }

    /// Recursive get
    #[allow(clippy::only_used_in_recursion)]
    fn get_recursive(
        &self,
        node: &VerkleNode,
        key_hash: &[u8],
        key: &[u8],
        depth: usize,
    ) -> Option<Vec<u8>> {
        match node {
            VerkleNode::Empty => None,
            VerkleNode::Leaf { values, .. } => values.get(key).cloned(),
            VerkleNode::Internal { children, .. } => {
                if depth >= key_hash.len() {
                    return None;
                }
                let child_index = key_hash[depth];
                if let Some(child) = children.get(&child_index) {
                    self.get_recursive(child, key_hash, key, depth + 1)
                } else {
                    None
                }
            }
        }
    }

    /// Recursive delete
    fn delete_recursive(
        &self,
        node: &VerkleNode,
        key_hash: &[u8],
        key: &[u8],
        depth: usize,
    ) -> Result<(VerkleNode, Option<Vec<u8>>), VerkleError> {
        match node {
            VerkleNode::Empty => Ok((VerkleNode::Empty, None)),
            VerkleNode::Leaf { values, .. } => {
                let mut new_values = values.clone();
                let deleted_value = new_values.remove(key);

                if new_values.is_empty() {
                    Ok((VerkleNode::Empty, deleted_value))
                } else {
                    let commitment = self.compute_leaf_commitment(&new_values)?;
                    Ok((
                        VerkleNode::Leaf {
                            commitment,
                            values: new_values,
                        },
                        deleted_value,
                    ))
                }
            }
            VerkleNode::Internal { children, .. } => {
                if depth >= key_hash.len() {
                    return Ok((node.clone(), None));
                }

                let child_index = key_hash[depth];
                if let Some(child) = children.get(&child_index) {
                    let (new_child, deleted_value) =
                        self.delete_recursive(child, key_hash, key, depth + 1)?;

                    let mut new_children = children.clone();
                    if new_child.is_empty() {
                        new_children.remove(&child_index);
                    } else {
                        new_children.insert(child_index, Box::new(new_child));
                    }

                    if new_children.is_empty() {
                        Ok((VerkleNode::Empty, deleted_value))
                    } else {
                        let commitment = self.compute_internal_commitment(&new_children)?;
                        Ok((
                            VerkleNode::Internal {
                                commitment,
                                children: new_children,
                            },
                            deleted_value,
                        ))
                    }
                } else {
                    Ok((node.clone(), None))
                }
            }
        }
    }

    /// Generate proof recursively
    #[allow(clippy::only_used_in_recursion)]
    fn generate_proof_recursive(
        &self,
        node: &VerkleNode,
        key_hash: &[u8],
        key: &[u8],
        depth: usize,
        path: &mut Vec<ProofNode>,
    ) -> Result<Option<Vec<u8>>, VerkleError> {
        match node {
            VerkleNode::Empty => {
                path.push(ProofNode::Empty);
                Ok(None)
            }
            VerkleNode::Leaf { commitment, values } => {
                path.push(ProofNode::Leaf {
                    commitment: commitment.clone(),
                    values: values.clone(),
                });
                Ok(values.get(key).cloned())
            }
            VerkleNode::Internal {
                commitment,
                children,
            } => {
                if depth >= key_hash.len() {
                    return Err(VerkleError::InvalidProof);
                }

                let child_index = key_hash[depth];
                let mut sibling_commitments = BTreeMap::new();

                // Collect sibling commitments for proof
                for (&index, child) in children.iter() {
                    if index != child_index {
                        sibling_commitments.insert(index, child.get_commitment());
                    }
                }

                path.push(ProofNode::Internal {
                    commitment: commitment.clone(),
                    child_index,
                    sibling_commitments,
                });

                if let Some(child) = children.get(&child_index) {
                    self.generate_proof_recursive(child, key_hash, key, depth + 1, path)
                } else {
                    self.generate_proof_recursive(
                        &VerkleNode::Empty,
                        key_hash,
                        key,
                        depth + 1,
                        path,
                    )
                }
            }
        }
    }

    /// Verify proof recursively
    fn verify_proof_recursive(
        &self,
        path: &[ProofNode],
        key_hash: &[u8],
        key: &[u8],
        expected_value: &Option<Vec<u8>>,
        depth: usize,
        expected_commitment: &VerklePoint,
    ) -> bool {
        if depth >= path.len() {
            return false;
        }

        match &path[depth] {
            ProofNode::Empty => {
                expected_value.is_none() && expected_commitment == &VerklePoint::identity()
            }
            ProofNode::Leaf { commitment, values } => {
                if commitment != expected_commitment {
                    return false;
                }

                let actual_value = values.get(key).cloned();
                actual_value == *expected_value
            }
            ProofNode::Internal {
                commitment,
                child_index,
                sibling_commitments,
            } => {
                if commitment != expected_commitment {
                    return false;
                }

                if depth >= key_hash.len() {
                    return false;
                }

                let expected_child_index = key_hash[depth];
                if *child_index != expected_child_index {
                    return false;
                }

                // Get the child commitment from the next level
                if depth + 1 < path.len() {
                    let child_commitment = match &path[depth + 1] {
                        ProofNode::Empty => VerklePoint::identity(),
                        ProofNode::Leaf { commitment, .. } => commitment.clone(),
                        ProofNode::Internal { commitment, .. } => commitment.clone(),
                    };

                    // Verify that the internal commitment is correctly computed
                    let mut all_children = sibling_commitments.clone();
                    all_children.insert(*child_index, child_commitment.clone());

                    if let Ok(computed_commitment) =
                        self.compute_internal_commitment_from_map(&all_children)
                    {
                        if computed_commitment != *commitment {
                            return false;
                        }
                    } else {
                        return false;
                    }

                    // Recursively verify the child
                    self.verify_proof_recursive(
                        path,
                        key_hash,
                        key,
                        expected_value,
                        depth + 1,
                        &child_commitment,
                    )
                } else {
                    false
                }
            }
        }
    }

    /// Compute commitment for a leaf node
    fn compute_leaf_commitment(
        &self,
        values: &BTreeMap<Vec<u8>, Vec<u8>>,
    ) -> Result<VerklePoint, VerkleError> {
        if values.is_empty() {
            return Ok(VerklePoint::identity());
        }

        // Create a polynomial from the key-value pairs
        let mut commitment = VerklePoint::identity();

        for (key, value) in values.iter() {
            // Hash key-value pair to create coefficient
            let mut hasher = blake3::Hasher::new();
            hasher.update(key);
            hasher.update(value);
            let hash = hasher.finalize();
            let coefficient = Fr::from_le_bytes_mod_order(hash.as_bytes());

            // Add to commitment using first generator
            let contribution = self.generators[0].scalar_mul(&coefficient);
            commitment = commitment.add(&contribution);
        }

        Ok(commitment)
    }

    /// Compute commitment for an internal node
    fn compute_internal_commitment(
        &self,
        children: &BTreeMap<u8, Box<VerkleNode>>,
    ) -> Result<VerklePoint, VerkleError> {
        self.compute_internal_commitment_from_map(
            &children
                .iter()
                .map(|(&k, v)| (k, v.get_commitment()))
                .collect(),
        )
    }

    /// Compute commitment from a map of child commitments
    fn compute_internal_commitment_from_map(
        &self,
        children: &BTreeMap<u8, VerklePoint>,
    ) -> Result<VerklePoint, VerkleError> {
        if children.is_empty() {
            return Ok(VerklePoint::identity());
        }

        let mut commitment = VerklePoint::identity();

        // Create polynomial commitment from child commitments
        for (&index, child_commitment) in children.iter() {
            if index as usize >= self.generators.len() {
                return Err(VerkleError::InvalidChildIndex);
            }

            // Each child commitment is multiplied by its corresponding generator
            let generator = &self.generators[index as usize + 1]; // +1 to skip the base generator

            // For simplicity, we hash the child commitment to get a scalar
            // In a real implementation, you would extract a scalar from the commitment properly
            let child_affine = child_commitment.to_affine();
            let mut hasher = blake3::Hasher::new();
            let mut child_bytes = Vec::new();
            child_affine.serialize_compressed(&mut child_bytes).unwrap();
            hasher.update(&child_bytes);
            let hash = hasher.finalize();
            let scalar = Fr::from_le_bytes_mod_order(hash.as_bytes());
            let contribution = generator.scalar_mul(&scalar);
            commitment = commitment.add(&contribution);
        }

        Ok(commitment)
    }
}

impl Default for VerkleTree {
    fn default() -> Self {
        Self::new()
    }
}

/// Proof node types
#[derive(Clone, Debug, Serialize, Deserialize)]
pub enum ProofNode {
    Empty,
    Leaf {
        commitment: VerklePoint,
        values: BTreeMap<Vec<u8>, Vec<u8>>,
    },
    Internal {
        commitment: VerklePoint,
        child_index: u8,
        sibling_commitments: BTreeMap<u8, VerklePoint>,
    },
}

/// Verkle tree proof
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct VerkleProof {
    pub key: Vec<u8>,
    pub value: Option<Vec<u8>>,
    pub path: Vec<ProofNode>,
    pub root_commitment: VerklePoint,
}

impl VerkleProof {
    /// Get the size of the proof in bytes
    pub fn size(&self) -> usize {
        bincode::serialize(self).map(|data| data.len()).unwrap_or(0)
    }
}

/// Verkle tree errors
#[derive(Debug, Clone)]
pub enum VerkleError {
    MaxDepthExceeded,
    InvalidProof,
    InvalidChildIndex,
    SerializationError(String),
    ComputationError(String),
}

impl fmt::Display for VerkleError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            VerkleError::MaxDepthExceeded => write!(f, "Maximum tree depth exceeded"),
            VerkleError::InvalidProof => write!(f, "Invalid proof"),
            VerkleError::InvalidChildIndex => write!(f, "Invalid child index"),
            VerkleError::SerializationError(msg) => write!(f, "Serialization error: {}", msg),
            VerkleError::ComputationError(msg) => write!(f, "Computation error: {}", msg),
        }
    }
}

impl std::error::Error for VerkleError {}

#[cfg(test)]
mod tests {
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

        // Test update
        assert!(tree.insert(b"key1", b"updated_value1").is_ok());
        assert_eq!(tree.get(b"key1"), Some(b"updated_value1".to_vec()));
    }

    #[test]
    fn test_verkle_tree_deletion() {
        let mut tree = VerkleTree::new();

        tree.insert(b"key1", b"value1").unwrap();
        tree.insert(b"key2", b"value2").unwrap();

        // Test deletion
        let deleted = tree.delete(b"key1").unwrap();
        assert_eq!(deleted, Some(b"value1".to_vec()));
        assert_eq!(tree.get(b"key1"), None);
        assert_eq!(tree.get(b"key2"), Some(b"value2".to_vec()));

        // Test deleting non-existent key
        let deleted = tree.delete(b"nonexistent").unwrap();
        assert_eq!(deleted, None);
    }

    #[test]
    fn test_verkle_proof_generation_and_verification() {
        let mut tree = VerkleTree::new();

        tree.insert(b"key1", b"value1").unwrap();
        tree.insert(b"key2", b"value2").unwrap();
        tree.insert(b"key3", b"value3").unwrap();

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
    fn test_verkle_tree_commitment_consistency() {
        let mut tree1 = VerkleTree::new();
        let mut tree2 = VerkleTree::new();

        // Insert same data in different order
        tree1.insert(b"key1", b"value1").unwrap();
        tree1.insert(b"key2", b"value2").unwrap();

        tree2.insert(b"key2", b"value2").unwrap();
        tree2.insert(b"key1", b"value1").unwrap();

        // Root commitments should be the same
        assert_eq!(tree1.get_root_commitment().0, tree2.get_root_commitment().0);
    }

    #[test]
    fn test_verkle_point_operations() {
        let point1 = VerklePoint::generator();
        let point2 = VerklePoint::generator();
        let scalar = Fr::one();

        // Test scalar multiplication
        let scaled = point1.scalar_mul(&scalar);
        assert_eq!(scaled.0, point1.0);

        // Test addition
        let sum = point1.add(&point2);
        let expected = VerklePoint(point1.0 + point2.0);
        assert_eq!(sum.0, expected.0);
    }

    #[test]
    fn test_verkle_tree_large_dataset() {
        let mut tree = VerkleTree::new();

        // Insert many key-value pairs
        for i in 0..100 {
            let key = format!("key_{:04}", i);
            let value = format!("value_{:04}", i);
            tree.insert(key.as_bytes(), value.as_bytes()).unwrap();
        }

        // Verify all can be retrieved
        for i in 0..100 {
            let key = format!("key_{:04}", i);
            let expected_value = format!("value_{:04}", i);
            assert_eq!(
                tree.get(key.as_bytes()),
                Some(expected_value.as_bytes().to_vec())
            );
        }

        // Test proofs for random keys
        let proof = tree.generate_proof(b"key_0050").unwrap();
        assert!(tree.verify_proof(&proof));
        assert_eq!(proof.value, Some(b"value_0050".to_vec()));
    }
}
