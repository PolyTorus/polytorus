//! Genesis Block Creation and Chain Initialization
//!
//! This module handles the creation of genesis blocks and initialization
//! of the blockchain with predefined accounts, allocations, and configuration.

use std::{
    collections::HashMap,
    time::{SystemTime, UNIX_EPOCH},
};

use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};

use crate::{
    blockchain::block::{BuildingBlock, FinalizedBlock},
    crypto::{
        transaction::Transaction,
        wallets::{Wallet, WalletManager},
    },
    modular::storage::{ModularStorage, StorageLayer},
};

/// Genesis configuration for chain initialization
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisConfig {
    /// Chain ID for the network
    pub chain_id: String,
    /// Network name
    pub network_name: String,
    /// Initial timestamp (0 for current time)
    pub timestamp: u64,
    /// Initial difficulty
    pub difficulty: u32,
    /// Gas limit for genesis block
    pub gas_limit: u64,
    /// Extra data for genesis block
    pub extra_data: String,
    /// Initial account allocations
    pub allocations: HashMap<String, GenesisAllocation>,
    /// Validator configuration
    pub validators: Vec<ValidatorConfig>,
    /// Governance configuration
    pub governance: GovernanceConfig,
    /// Protocol parameters
    pub protocol_params: ProtocolParams,
}

/// Initial allocation for an account
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenesisAllocation {
    /// Account balance
    pub balance: u64,
    /// Account nonce
    pub nonce: u64,
    /// Account code (for contracts)
    pub code: Option<String>,
    /// Account storage
    pub storage: HashMap<String, String>,
}

/// Validator configuration for genesis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidatorConfig {
    /// Validator address
    pub address: String,
    /// Validator stake
    pub stake: u64,
    /// Validator public key
    pub public_key: String,
    /// Validator commission rate
    pub commission_rate: f64,
}

/// Governance configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GovernanceConfig {
    /// Voting period in blocks
    pub voting_period: u64,
    /// Minimum quorum for proposals
    pub min_quorum: f64,
    /// Minimum stake to propose
    pub min_proposal_stake: u64,
    /// Treasury allocation
    pub treasury_allocation: u64,
}

/// Protocol parameters
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProtocolParams {
    /// Block time in milliseconds
    pub block_time: u64,
    /// Maximum block size
    pub max_block_size: usize,
    /// Maximum gas per block
    pub max_gas_per_block: u64,
    /// Base fee per gas
    pub base_fee_per_gas: u64,
    /// Fee burn rate
    pub fee_burn_rate: f64,
}

impl Default for GenesisConfig {
    fn default() -> Self {
        let mut allocations = HashMap::new();

        // Default allocations for testnet
        allocations.insert(
            "polytorus1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq9yf5ce".to_string(),
            GenesisAllocation {
                balance: 1_000_000_000_000_000, // 1M tokens
                nonce: 0,
                code: None,
                storage: HashMap::new(),
            },
        );

        Self {
            chain_id: "polytorus-testnet-1".to_string(),
            network_name: "PolyTorus Testnet".to_string(),
            timestamp: 0, // Will use current time
            difficulty: 4,
            gas_limit: 8_000_000,
            extra_data: "PolyTorus Genesis Block".to_string(),
            allocations,
            validators: vec![ValidatorConfig {
                address: "polytorus1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqq9yf5ce".to_string(),
                stake: 100_000_000, // 100K tokens
                public_key: "genesis_validator_pubkey".to_string(),
                commission_rate: 0.05, // 5%
            }],
            governance: GovernanceConfig {
                voting_period: 100800,           // ~1 week at 6s block time
                min_quorum: 0.33,                // 33%
                min_proposal_stake: 10_000,      // 10K tokens
                treasury_allocation: 50_000_000, // 50K tokens
            },
            protocol_params: ProtocolParams {
                block_time: 6000,            // 6 seconds
                max_block_size: 1024 * 1024, // 1MB
                max_gas_per_block: 8_000_000,
                base_fee_per_gas: 1,
                fee_burn_rate: 0.5, // 50% of fees burned
            },
        }
    }
}

/// Genesis block creator
pub struct GenesisCreator {
    config: GenesisConfig,
    storage: Option<ModularStorage>,
}

impl GenesisCreator {
    /// Create a new genesis creator
    pub fn new(config: GenesisConfig) -> Self {
        Self {
            config,
            storage: None,
        }
    }

    /// Create genesis creator with default configuration
    pub fn with_default_config() -> Self {
        Self::new(GenesisConfig::default())
    }

    /// Create genesis creator with custom configuration
    pub fn with_config(config: GenesisConfig) -> Self {
        Self::new(config)
    }

    /// Set storage for genesis creation
    pub fn with_storage(mut self, storage: ModularStorage) -> Self {
        self.storage = Some(storage);
        self
    }

    /// Create the genesis block
    pub async fn create_genesis_block(&self) -> Result<FinalizedBlock> {
        log::info!("Creating genesis block for chain: {}", self.config.chain_id);

        // Use current timestamp if not specified
        let _timestamp = if self.config.timestamp == 0 {
            SystemTime::now().duration_since(UNIX_EPOCH)?.as_secs()
        } else {
            self.config.timestamp
        };

        // Create genesis transactions for initial allocations
        let mut genesis_transactions = Vec::new();

        // First transaction must be coinbase
        let coinbase_tx =
            Transaction::new_coinbase("genesis".to_string(), "Genesis Block".to_string())?;
        genesis_transactions.push(coinbase_tx);

        for (address, allocation) in &self.config.allocations {
            if allocation.balance > 0 {
                // Create a special genesis transaction
                let genesis_tx = Transaction::new_genesis_allocation(
                    address.clone(),
                    allocation.balance,
                    allocation.nonce,
                );
                genesis_transactions.push(genesis_tx);
            }
        }

        // Create validator setup transactions
        for validator in &self.config.validators {
            let validator_tx = Transaction::new_validator_registration(
                validator.address.clone(),
                validator.stake,
                validator.public_key.clone(),
                validator.commission_rate,
            );
            genesis_transactions.push(validator_tx);
        }

        // Create governance setup transaction
        let governance_tx = Transaction::new_governance_setup(self.config.governance.clone());
        genesis_transactions.push(governance_tx);

        // Create protocol parameters transaction
        let protocol_tx = Transaction::new_protocol_setup(self.config.protocol_params.clone());
        genesis_transactions.push(protocol_tx);

        // Build the genesis block
        let building_block = BuildingBlock::new_building(
            genesis_transactions,
            "0000000000000000000000000000000000000000000000000000000000000000".to_string(), // No previous hash
            0, // Height 0
            self.config.difficulty as usize,
        );

        // Mine the genesis block
        let mined_block = building_block.mine()?;

        // Validate the mined block
        let validated_block = mined_block.validate()?;

        // Finalize the block
        let finalized_block = validated_block.finalize();

        log::info!(
            "Genesis block created: {} at height {}",
            finalized_block.get_hash(),
            finalized_block.get_height()
        );

        Ok(finalized_block)
    }

    /// Initialize the blockchain with genesis block
    pub async fn initialize_chain(&self, storage: &ModularStorage) -> Result<FinalizedBlock> {
        // Check if genesis block already exists
        if (storage.get_block_by_height(0).await?).is_some() {
            return Err(anyhow!("Genesis block already exists"));
        }

        // Create genesis block
        let genesis_block = self.create_genesis_block().await?;

        // Store genesis block
        storage.store_block(&genesis_block)?;
        storage
            .update_best_block(genesis_block.get_hash(), 0)
            .await?;

        // Initialize state from genesis allocations
        self.initialize_genesis_state(storage, &genesis_block)
            .await?;

        log::info!(
            "Blockchain initialized with genesis block: {}",
            genesis_block.get_hash()
        );
        Ok(genesis_block)
    }

    /// Create initial wallets from genesis configuration
    pub async fn create_genesis_wallets(
        &self,
        wallet_manager: &WalletManager,
    ) -> Result<Vec<String>> {
        let mut created_addresses = Vec::new();

        for (address, allocation) in &self.config.allocations {
            if allocation.balance > 0 {
                // Create wallet for this address
                let wallet = Wallet::new_with_address(address.clone());
                wallet_manager.add_wallet(address.clone(), wallet).await?;
                created_addresses.push(address.clone());

                log::info!(
                    "Created genesis wallet: {} with balance: {}",
                    address,
                    allocation.balance
                );
            }
        }

        Ok(created_addresses)
    }

    /// Validate genesis configuration
    pub fn validate_config(&self) -> Result<()> {
        // Validate chain ID
        if self.config.chain_id.is_empty() {
            return Err(anyhow!("Chain ID cannot be empty"));
        }

        // Validate allocations
        let total_supply: u64 = self
            .config
            .allocations
            .values()
            .map(|alloc| alloc.balance)
            .sum();

        if total_supply == 0 {
            return Err(anyhow!("Total supply cannot be zero"));
        }

        // Validate validators
        if self.config.validators.is_empty() {
            return Err(anyhow!("At least one validator required"));
        }

        for validator in &self.config.validators {
            if validator.stake == 0 {
                return Err(anyhow!("Validator stake cannot be zero"));
            }

            if validator.commission_rate < 0.0 || validator.commission_rate > 1.0 {
                return Err(anyhow!(
                    "Invalid commission rate: {}",
                    validator.commission_rate
                ));
            }
        }

        // Validate governance parameters
        if self.config.governance.min_quorum < 0.0 || self.config.governance.min_quorum > 1.0 {
            return Err(anyhow!(
                "Invalid minimum quorum: {}",
                self.config.governance.min_quorum
            ));
        }

        // Validate protocol parameters
        if self.config.protocol_params.block_time == 0 {
            return Err(anyhow!("Block time cannot be zero"));
        }

        if self.config.protocol_params.max_block_size == 0 {
            return Err(anyhow!("Max block size cannot be zero"));
        }

        log::info!("Genesis configuration validated successfully");
        Ok(())
    }

    /// Export genesis configuration to JSON
    pub fn export_config(&self) -> Result<String> {
        Ok(serde_json::to_string_pretty(&self.config)?)
    }

    /// Import genesis configuration from JSON
    pub fn import_config(json_data: &str) -> Result<Self> {
        let config: GenesisConfig = serde_json::from_str(json_data)?;
        Ok(Self::new(config))
    }

    /// Initialize genesis state in storage
    async fn initialize_genesis_state(
        &self,
        storage: &ModularStorage,
        _genesis_block: &FinalizedBlock,
    ) -> Result<()> {
        // Store initial account states
        for (address, allocation) in &self.config.allocations {
            // Store account balance and nonce
            storage
                .store_account_state(address, allocation.balance, allocation.nonce)
                .await?;

            // Store contract code if present
            if let Some(code) = &allocation.code {
                storage.store_contract_code(address, code).await?;
            }

            // Store contract storage if present
            for (key, value) in &allocation.storage {
                storage.store_contract_storage(address, key, value).await?;
            }
        }

        // Store validator information
        for validator in &self.config.validators {
            storage
                .store_validator_info(
                    &validator.address,
                    validator.stake,
                    &validator.public_key,
                    validator.commission_rate,
                )
                .await?;
        }

        // Store governance configuration
        storage
            .store_governance_config(&self.config.governance)
            .await?;

        // Store protocol parameters
        storage
            .store_protocol_params(&self.config.protocol_params)
            .await?;

        log::info!("Genesis state initialized in storage");
        Ok(())
    }

    /// Get the genesis configuration
    pub fn get_config(&self) -> &GenesisConfig {
        &self.config
    }

    /// Update genesis configuration
    pub fn update_config(&mut self, config: GenesisConfig) {
        self.config = config;
    }
}

/// Utility functions for genesis creation
/// Create a testnet genesis configuration
pub fn create_testnet_genesis() -> GenesisConfig {
    let mut config = GenesisConfig {
        chain_id: "polytorus-testnet-1".to_string(),
        network_name: "PolyTorus Testnet".to_string(),
        difficulty: 2, // Lower difficulty for testnet
        ..Default::default()
    };

    // Add more test accounts
    config.allocations.insert(
        "polytorus1test1qqqqqqqqqqqqqqqqqqqqqqqqqqqqqq8yf5ce".to_string(),
        GenesisAllocation {
            balance: 100_000_000,
            nonce: 0,
            code: None,
            storage: HashMap::new(),
        },
    );

    config.allocations.insert(
        "polytorus1test2qqqqqqqqqqqqqqqqqqqqqqqqqqqqqq8yf5ce".to_string(),
        GenesisAllocation {
            balance: 100_000_000,
            nonce: 0,
            code: None,
            storage: HashMap::new(),
        },
    );

    config
}

/// Create a mainnet genesis configuration
pub fn create_mainnet_genesis() -> GenesisConfig {
    let mut config = GenesisConfig {
        chain_id: "polytorus-mainnet-1".to_string(),
        network_name: "PolyTorus Mainnet".to_string(),
        difficulty: 6, // Higher difficulty for mainnet
        ..Default::default()
    };

    // Mainnet would have different initial allocations
    config.allocations.clear();
    config.allocations.insert(
        "polytorus1mainnet1qqqqqqqqqqqqqqqqqqqqqqqqqqqqq8yf5ce".to_string(),
        GenesisAllocation {
            balance: 21_000_000_000_000_000, // 21M tokens total supply
            nonce: 0,
            code: None,
            storage: HashMap::new(),
        },
    );

    config
}

#[cfg(test)]
mod tests {
    use tempfile::TempDir;

    use super::*;

    #[tokio::test]
    async fn test_genesis_creation() {
        let config = GenesisConfig::default();
        let creator = GenesisCreator::new(config);

        let result = creator.validate_config();
        assert!(result.is_ok());

        let genesis_block = creator.create_genesis_block().await.unwrap();
        assert_eq!(genesis_block.get_height(), 0);
        assert!(!genesis_block.get_hash().is_empty());
    }

    #[tokio::test]
    async fn test_chain_initialization() {
        let temp_dir = TempDir::new().unwrap();
        let storage = ModularStorage::new_with_path(temp_dir.path()).unwrap();

        let config = create_testnet_genesis();
        let creator = GenesisCreator::new(config);

        let genesis_block = creator.initialize_chain(&storage).await.unwrap();
        assert_eq!(genesis_block.get_height(), 0);

        // Verify genesis block was stored
        let stored_block = storage.get_block_by_height(0).await.unwrap();
        assert!(stored_block.is_some());
    }

    #[test]
    fn test_config_validation() {
        let mut config = GenesisConfig::default();
        let creator = GenesisCreator::new(config.clone());
        assert!(creator.validate_config().is_ok());

        // Test invalid chain ID
        config.chain_id = "".to_string();
        let creator = GenesisCreator::new(config.clone());
        assert!(creator.validate_config().is_err());

        // Reset and test invalid validator
        config = GenesisConfig::default();
        config.validators[0].commission_rate = 1.5; // Invalid rate > 1.0
        let creator = GenesisCreator::new(config);
        assert!(creator.validate_config().is_err());
    }

    #[test]
    fn test_config_serialization() {
        let config = create_testnet_genesis();
        let creator = GenesisCreator::new(config);

        let json = creator.export_config().unwrap();
        assert!(!json.is_empty());

        let imported_creator = GenesisCreator::import_config(&json).unwrap();
        assert_eq!(creator.config.chain_id, imported_creator.config.chain_id);
    }

    #[test]
    fn test_testnet_vs_mainnet_config() {
        let testnet = create_testnet_genesis();
        let mainnet = create_mainnet_genesis();

        assert_ne!(testnet.chain_id, mainnet.chain_id);
        assert!(testnet.difficulty < mainnet.difficulty);
        assert!(testnet.allocations.len() > mainnet.allocations.len());
    }
}
