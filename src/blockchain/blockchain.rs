//! Blockchain

use crate::blockchain::block::*;
use crate::config::DataContext;
use crate::crypto::traits::CryptoProvider;
use crate::crypto::transaction::*;
use crate::smart_contract::{ContractEngine, ContractState, SmartContract};
use crate::smart_contract::types::ContractExecution;
use crate::Result;
use bincode::{deserialize, serialize};
use failure::format_err;
use sled;
use std::collections::HashMap;
use std::time::SystemTime;

const GENESIS_COINBASE_DATA: &str =
    "The Times 03/Jan/2009 Chancellor on brink of second bailout for banks";

/// Blockchain implements interactions with a DB
#[derive(Debug, Clone)]
pub struct Blockchain {
    pub tip: String,
    pub db: sled::Db,
    pub context: DataContext,
}

/// BlockchainIterator is used to iterate over blockchain blocks
pub struct BlockchainIterator<'a> {
    current_hash: String,
    bc: &'a Blockchain,
}

impl Blockchain {
    pub fn new() -> Result<Blockchain> {
        Self::new_with_context(DataContext::default())
    }

    /// NewBlockchain creates a new Blockchain db
    pub fn new_with_context(context: DataContext) -> Result<Blockchain> {
        info!("open blockchain");

        let db = sled::open(context.blocks_dir())?;
        let hash = match db.get("LAST")? {
            Some(l) => l.to_vec(),
            None => Vec::new(),
        };
        info!("Found block database");
        let lasthash = if hash.is_empty() {
            String::new()
        } else {
            String::from_utf8(hash.to_vec())?
        };
        Ok(Blockchain {
            tip: lasthash,
            db,
            context,
        })
    }

    pub fn create_blockchain(address: String) -> Result<Blockchain> {
        Self::create_blockchain_with_context(address, DataContext::default())
    }

    /// CreateBlockchain creates a new blockchain DB
    pub fn create_blockchain_with_context(
        address: String,
        context: DataContext,
    ) -> Result<Blockchain> {
        info!("Creating new blockchain");

        let db_path = context.blocks_dir();
        std::fs::remove_dir_all(&db_path).ok();
        let db = sled::open(db_path)?;
        debug!("Creating new block database");
        let cbtx = Transaction::new_coinbase(address, String::from(GENESIS_COINBASE_DATA))?;
        let genesis: Block = Block::new_genesis_block(cbtx);
        db.insert(genesis.get_hash(), serialize(&genesis)?)?;
        db.insert("LAST", genesis.get_hash().as_bytes())?;
        let bc = Blockchain {
            tip: genesis.get_hash(),
            db,
            context,
        };
        bc.db.flush()?;
        Ok(bc)
    }    /// MineBlock mines a new block with the provided transactions
    pub fn mine_block(&mut self, transactions: Vec<Transaction>) -> Result<Block> {
        info!("mine a new block");

        for tx in &transactions {
            if !self.verify_transacton(tx)? {
                return Err(format_err!("ERROR: Invalid transaction"));
            }
        }

        // Execute smart contract transactions before adding to block
        let mut processed_transactions = transactions;
        for tx in &mut processed_transactions {
            if tx.is_contract_transaction() {
                if let Err(e) = self.execute_contract_transaction(tx) {
                    warn!("Contract execution failed for tx {}: {}", tx.id, e);
                    // In a real implementation, you might want to handle this differently
                    // For now, we'll continue with the transaction as-is
                }
            }
        }

        let lasthash = self.db.get("LAST")?.unwrap();
        let prev_hash = String::from_utf8(lasthash.to_vec())?;
        let prev_block = self.get_block(&prev_hash)?;
        let current_timestamp = SystemTime::now()
            .duration_since(SystemTime::UNIX_EPOCH)?
            .as_millis();
        let new_difficulty = Block::adjust_difficulty(&prev_block, current_timestamp);

        let newblock = Block::new_block(
            processed_transactions,
            prev_hash,
            self.get_best_height()? + 1,
            new_difficulty,
        )?;
        self.db.insert(newblock.get_hash(), serialize(&newblock)?)?;
        self.db.insert("LAST", newblock.get_hash().as_bytes())?;
        self.db.flush()?;

        self.tip = newblock.get_hash();
        Ok(newblock)
    }

    /// Iterator returns a BlockchainIterat
    pub fn iter(&self) -> BlockchainIterator {
        BlockchainIterator {
            current_hash: self.tip.clone(),
            bc: self,
        }
    }

    /// FindUTXO finds and returns all unspent transaction outputs
    pub fn find_UTXO(&self) -> HashMap<String, TXOutputs> {
        let mut utxos: HashMap<String, TXOutputs> = HashMap::new();
        let mut spend_txos: HashMap<String, Vec<i32>> = HashMap::new();

        for block in self.iter() {
            for tx in block.get_transaction() {
                for index in 0..tx.vout.len() {
                    if let Some(ids) = spend_txos.get(&tx.id) {
                        if ids.contains(&(index as i32)) {
                            continue;
                        }
                    }

                    match utxos.get_mut(&tx.id) {
                        Some(v) => {
                            v.outputs.push(tx.vout[index].clone());
                        }
                        None => {
                            utxos.insert(
                                tx.id.clone(),
                                TXOutputs {
                                    outputs: vec![tx.vout[index].clone()],
                                },
                            );
                        }
                    }
                }

                if !tx.is_coinbase() {
                    for i in &tx.vin {
                        match spend_txos.get_mut(&i.txid) {
                            Some(v) => {
                                v.push(i.vout);
                            }
                            None => {
                                spend_txos.insert(i.txid.clone(), vec![i.vout]);
                            }
                        }
                    }
                }
            }
        }

        utxos
    }

    /// FindTransaction finds a transaction by its ID
    pub fn find_transacton(&self, id: &str) -> Result<Transaction> {
        for b in self.iter() {
            for tx in b.get_transaction() {
                if tx.id == id {
                    return Ok(tx.clone());
                }
            }
        }
        Err(format_err!("Transaction is not found"))
    }

    fn get_prev_TXs(&self, tx: &Transaction) -> Result<HashMap<String, Transaction>> {
        let mut prev_TXs = HashMap::new();
        for vin in &tx.vin {
            let prev_TX = self.find_transacton(&vin.txid)?;
            prev_TXs.insert(prev_TX.id.clone(), prev_TX);
        }
        Ok(prev_TXs)
    }

    /// SignTransaction signs inputs of a Transaction
    pub fn sign_transacton(
        &self,
        tx: &mut Transaction,
        private_key: &[u8],
        crypto: &dyn CryptoProvider,
    ) -> Result<()> {
        let prev_TXs = self.get_prev_TXs(tx)?;
        tx.sign(private_key, prev_TXs, crypto)?;
        Ok(())
    }

    /// VerifyTransaction verifies transaction input signatures
    pub fn verify_transacton(&self, tx: &Transaction) -> Result<bool> {
        if tx.is_coinbase() {
            return Ok(true);
        }
        let prev_TXs = self.get_prev_TXs(tx)?;
        tx.verify(prev_TXs)
    }

    /// AddBlock saves the block into the blockchain
    pub fn add_block(&mut self, block: Block) -> Result<()> {
        let data = serialize(&block)?;
        if let Some(_) = self.db.get(block.get_hash())? {
            return Ok(());
        }
        self.db.insert(block.get_hash(), data)?;

        let lastheight = self.get_best_height()?;
        if block.get_height() > lastheight {
            self.db.insert("LAST", block.get_hash().as_bytes())?;
            self.tip = block.get_hash();
            self.db.flush()?;
        }
        Ok(())
    }

    // GetBlock finds a block by its hash and returns it
    pub fn get_block(&self, block_hash: &str) -> Result<Block> {
        let data = self.db.get(block_hash)?.unwrap();
        let block = deserialize(&data)?;
        Ok(block)
    }

    /// GetBestHeight returns the height of the latest block
    pub fn get_best_height(&self) -> Result<i32> {
        let lasthash = if let Some(h) = self.db.get("LAST")? {
            h
        } else {
            return Ok(-1);
        };
        let last_data = self.db.get(lasthash)?.unwrap();
        let last_block: Block = deserialize(&last_data)?;
        Ok(last_block.get_height())
    }

    /// GetBlockHashes returns a list of hashes of all the blocks in the chain
    pub fn get_block_hashs(&self) -> Vec<String> {
        let mut list = Vec::new();
        for b in self.iter() {
            list.push(b.get_hash());
        }
        list
    }

    /// Execute a smart contract transaction
    fn execute_contract_transaction(&self, tx: &Transaction) -> Result<()> {
        if let Some(contract_data) = tx.get_contract_data() {
            let contract_state_path = self.context.data_dir().join("contracts");
            let contract_state = ContractState::new(contract_state_path.to_str().unwrap())?;
            let engine = ContractEngine::new(contract_state)?;

            match &contract_data.tx_type {                ContractTransactionType::Deploy { bytecode, constructor_args, gas_limit: _ } => {
                    info!("Deploying smart contract from transaction {}", tx.id);
                      // Extract deployer address from transaction inputs
                    let deployer_address = if let Some(input) = tx.vin.first() {
                        // Convert public key to wallet address properly
                        use crate::crypto::wallets::hash_pub_key;
                        use bitcoincash_addr::{Address, Scheme, HashType};
                        
                        let mut pub_key_hash = input.pub_key.clone();
                        hash_pub_key(&mut pub_key_hash);
                        
                        let address = Address {
                            body: pub_key_hash,
                            scheme: Scheme::Base58,
                            hash_type: HashType::Script,
                            ..Default::default()
                        };
                        
                        // Create base address without encryption suffix for simplicity
                        address.encode().unwrap_or_else(|_| "unknown_deployer".to_string())
                    } else {
                        "unknown_deployer".to_string()
                    };
                    
                    // Create contract instance
                    let contract = SmartContract::new(
                        bytecode.clone(),
                        deployer_address,
                        constructor_args.clone(),
                        None, // ABI not provided in this simple implementation
                    )?;

                    // Deploy the contract
                    engine.deploy_contract(&contract)?;
                    info!("Contract deployed at address: {}", contract.get_address());
                }
                ContractTransactionType::Call { 
                    contract_address, 
                    function_name, 
                    arguments, 
                    gas_limit, 
                    value 
                } => {
                    info!("Calling smart contract {} function {}", contract_address, function_name);
                    
                    let execution = ContractExecution {
                        contract_address: contract_address.clone(),
                        function_name: function_name.clone(),
                        arguments: arguments.clone(),
                        gas_limit: *gas_limit,
                        caller: "caller".to_string(), // In a real implementation, extract from transaction
                        value: *value,
                    };

                    let result = engine.execute_contract(execution)?;
                    info!("Contract call result: success={}, gas_used={}", result.success, result.gas_used);
                    
                    if !result.logs.is_empty() {
                        info!("Contract logs: {:?}", result.logs);
                    }
                }
            }
        }
        Ok(())
    }

    /// Get smart contract state
    pub fn get_contract_state(&self, contract_address: &str) -> Result<std::collections::HashMap<String, Vec<u8>>> {
        let contract_state_path = self.context.data_dir().join("contracts");
        let contract_state = ContractState::new(contract_state_path.to_str().unwrap())?;
        let engine = ContractEngine::new(contract_state)?;
        engine.get_contract_state(contract_address)
    }

    /// List all deployed contracts
    pub fn list_contracts(&self) -> Result<Vec<crate::smart_contract::types::ContractMetadata>> {
        let contract_state_path = self.context.data_dir().join("contracts");
        let contract_state = ContractState::new(contract_state_path.to_str().unwrap())?;
        let engine = ContractEngine::new(contract_state)?;
        engine.list_contracts()
    }

    /// List deployed contracts with optional limit
    pub fn list_contracts_with_limit(&self, limit: Option<usize>) -> Result<Vec<crate::smart_contract::types::ContractMetadata>> {
        let contract_state_path = self.context.data_dir().join("contracts");
        let contract_state = ContractState::new(contract_state_path.to_str().unwrap())?;
        contract_state.list_contracts_with_limit(limit)
    }
}

impl Iterator for BlockchainIterator<'_> {
    type Item = Block;

    fn next(&mut self) -> Option<Self::Item> {
        if let Ok(encoded_block) = self.bc.db.get(&self.current_hash) {
            return match encoded_block {
                Some(b) => {
                    if let Ok(block) = deserialize::<Block>(&b) {
                        self.current_hash = block.get_prev_hash();
                        Some(block)
                    } else {
                        None
                    }
                }
                None => None,
            };
        }
        None
    }
}
