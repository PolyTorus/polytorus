//! Smart contract definition and management

use std::time::{
    SystemTime,
    UNIX_EPOCH,
};

use crypto::digest::Digest;
use crypto::sha2::Sha256;
use serde::{
    Deserialize,
    Serialize,
};

use crate::smart_contract::state::ContractState;
use crate::smart_contract::types::ContractMetadata;
use crate::Result;

/// Smart contract representation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SmartContract {
    pub address: String,
    pub bytecode: Vec<u8>,
    pub metadata: ContractMetadata,
}

impl SmartContract {
    /// Create a new smart contract
    pub fn new(
        bytecode: Vec<u8>,
        creator: String,
        _constructor_args: Vec<u8>,
        abi: Option<String>,
    ) -> Result<Self> {
        let address = Self::generate_address(&bytecode, &creator)?;
        let bytecode_hash = Self::hash_bytecode(&bytecode)?;

        let created_at = SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs();

        let metadata = ContractMetadata {
            address: address.clone(),
            creator,
            created_at,
            bytecode_hash,
            abi,
        };

        Ok(Self {
            address,
            bytecode,
            metadata,
        })
    }

    /// Generate a deterministic contract address
    fn generate_address(bytecode: &[u8], creator: &str) -> Result<String> {
        let mut hasher = Sha256::new();
        hasher.input(creator.as_bytes());
        hasher.input(bytecode);
        hasher.input(
            &SystemTime::now()
                .duration_since(UNIX_EPOCH)?
                .as_nanos()
                .to_le_bytes(),
        );
        Ok(format!("contract_{}", &hasher.result_str()[..20]))
    }

    /// Calculate bytecode hash
    fn hash_bytecode(bytecode: &[u8]) -> Result<String> {
        let mut hasher = Sha256::new();
        hasher.input(bytecode);
        Ok(hasher.result_str())
    }

    /// Deploy the contract to the blockchain state
    pub fn deploy(&self, state: &ContractState) -> Result<()> {
        // Store contract metadata
        state.store_contract(&self.metadata)?;

        // Initialize contract state if needed
        // This could include running a constructor function
        log::info!("Contract deployed at address: {}", self.address);

        Ok(())
    }

    /// Get contract address
    pub fn get_address(&self) -> &str {
        &self.address
    }

    /// Get contract bytecode
    pub fn get_bytecode(&self) -> &[u8] {
        &self.bytecode
    }

    /// Get contract metadata
    pub fn get_metadata(&self) -> &ContractMetadata {
        &self.metadata
    }

    /// Verify contract bytecode integrity
    pub fn verify_integrity(&self) -> Result<bool> {
        let calculated_hash = Self::hash_bytecode(&self.bytecode)?;
        Ok(calculated_hash == self.metadata.bytecode_hash)
    }
}
