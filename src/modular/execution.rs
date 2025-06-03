//! Modular execution layer implementation
//!
//! This module implements the execution layer for the modular blockchain,
//! handling transaction execution and state management.

use super::traits::*;
use crate::blockchain::block::Block;
use crate::crypto::transaction::Transaction;
use crate::smart_contract::{ContractEngine, ContractState};
use crate::smart_contract::types::{ContractExecution, ContractDeployment};
use crate::config::DataContext;
use crate::Result;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Execution layer implementation
pub struct PolyTorusExecutionLayer {
    /// Contract execution engine
    contract_engine: Arc<Mutex<ContractEngine>>,
    /// Current state root
    state_root: Arc<Mutex<Hash>>,
    /// Account states
    account_states: Arc<Mutex<HashMap<String, AccountState>>>,
    /// Execution context
    execution_context: Arc<Mutex<Option<ExecutionContext>>>,
    /// Configuration
    config: ExecutionConfig,
}

/// Execution context for managing state transitions
#[derive(Debug, Clone)]
struct ExecutionContext {
    /// Context ID
    context_id: String,
    /// Initial state root
    initial_state_root: Hash,
    /// Pending state changes
    pending_changes: HashMap<String, AccountState>,
    /// Executed transactions
    executed_txs: Vec<TransactionReceipt>,
    /// Gas used in this context
    gas_used: u64,
}

impl PolyTorusExecutionLayer {
    /// Create a new execution layer
    pub fn new(data_context: DataContext, config: ExecutionConfig) -> Result<Self> {
        let contract_state_path = data_context.data_dir().join("contracts");
        let contract_state = ContractState::new(contract_state_path.to_str().unwrap())?;
        let contract_engine = ContractEngine::new(contract_state)?;

        Ok(Self {
            contract_engine: Arc::new(Mutex::new(contract_engine)),
            state_root: Arc::new(Mutex::new("genesis".to_string())),
            account_states: Arc::new(Mutex::new(HashMap::new())),
            execution_context: Arc::new(Mutex::new(None)),
            config,
        })
    }

    /// Execute a smart contract transaction
    fn execute_contract_transaction(&self, tx: &Transaction) -> Result<TransactionReceipt> {
        let mut events = Vec::new();
        let mut gas_used = 0;

        if let Some(contract_data) = tx.get_contract_data() {
            let engine = self.contract_engine.lock().unwrap();
            
            match &contract_data.tx_type {
                crate::crypto::transaction::ContractTransactionType::Deploy { 
                    bytecode, 
                    constructor_args, 
                    gas_limit 
                } => {
                    let deployment = ContractDeployment {
                        bytecode: bytecode.clone(),
                        constructor_args: constructor_args.clone(),
                        gas_limit: *gas_limit,
                    };
                    
                    // Create a simple contract and deploy it
                    let contract = crate::smart_contract::SmartContract::new(
                        deployment.bytecode,
                        "deployer".to_string(),
                        deployment.constructor_args,
                        None,
                    )?;
                    
                    engine.deploy_contract(&contract)?;
                    gas_used = deployment.gas_limit / 10; // Simple gas calculation
                    
                    // Create deployment event
                    events.push(Event {
                        contract: contract.get_address().to_string(),
                        data: b"Contract deployed".to_vec(),
                        topics: vec!["deployment".to_string()],
                    });
                }
                crate::crypto::transaction::ContractTransactionType::Call { 
                    contract_address, 
                    function_name, 
                    arguments, 
                    gas_limit, 
                    value 
                } => {
                    let execution = ContractExecution {
                        contract_address: contract_address.clone(),
                        function_name: function_name.clone(),
                        arguments: arguments.clone(),
                        gas_limit: *gas_limit,
                        caller: "caller".to_string(), // Extract from transaction
                        value: *value,
                    };
                    
                    let result = engine.execute_contract(execution)?;
                    gas_used = result.gas_used;
                    
                    // Create call event
                    events.push(Event {
                        contract: contract_address.clone(),
                        data: format!("Function {} called", function_name).into_bytes(),
                        topics: vec!["function_call".to_string(), function_name.clone()],
                    });
                }
            }
        }

        Ok(TransactionReceipt {
            tx_hash: tx.id.clone(),
            success: true,
            gas_used,
            events,
        })
    }

    /// Calculate new state root based on executed transactions
    fn calculate_state_root(&self, receipts: &[TransactionReceipt]) -> Hash {
        use crypto::digest::Digest;
        use crypto::sha2::Sha256;
        
        let mut hasher = Sha256::new();
        let current_root = self.state_root.lock().unwrap().clone();
        hasher.input(current_root.as_bytes());
        
        for receipt in receipts {
            hasher.input(receipt.tx_hash.as_bytes());
            hasher.input(&receipt.gas_used.to_le_bytes());
        }
        
        hasher.result_str()
    }
}

impl ExecutionLayer for PolyTorusExecutionLayer {
    fn execute_block(&self, block: &Block) -> Result<ExecutionResult> {
        let mut receipts = Vec::new();
        let mut total_gas_used = 0;
        let mut all_events = Vec::new();

        // Execute each transaction in the block
        for tx in block.get_transaction() {
            let receipt = if tx.is_contract_transaction() {
                self.execute_contract_transaction(tx)?
            } else {
                // Handle regular transactions
                TransactionReceipt {
                    tx_hash: tx.id.clone(),
                    success: true,
                    gas_used: 21000, // Base gas cost
                    events: vec![],
                }
            };

            total_gas_used += receipt.gas_used;
            all_events.extend(receipt.events.clone());
            receipts.push(receipt);

            // Check gas limit
            if total_gas_used > self.config.gas_limit {
                return Err(failure::format_err!("Block gas limit exceeded"));
            }
        }

        let new_state_root = self.calculate_state_root(&receipts);

        Ok(ExecutionResult {
            state_root: new_state_root,
            gas_used: total_gas_used,
            receipts,
            events: all_events,
        })
    }

    fn get_state_root(&self) -> Hash {
        self.state_root.lock().unwrap().clone()
    }

    fn verify_execution(&self, proof: &ExecutionProof) -> bool {
        // Simplified verification - in a real implementation, this would
        // verify the execution proof against the state transition
        !proof.state_proof.is_empty() && 
        !proof.execution_trace.is_empty() &&
        proof.input_state_root != proof.output_state_root
    }

    fn execute_transaction(&self, tx: &Transaction) -> Result<TransactionReceipt> {
        if tx.is_contract_transaction() {
            self.execute_contract_transaction(tx)
        } else {
            // Handle regular transaction
            Ok(TransactionReceipt {
                tx_hash: tx.id.clone(),
                success: true,
                gas_used: 21000,
                events: vec![],
            })
        }
    }

    fn get_account_state(&self, address: &str) -> Result<AccountState> {
        let account_states = self.account_states.lock().unwrap();
        
        Ok(account_states.get(address).cloned().unwrap_or(AccountState {
            balance: 0,
            nonce: 0,
            code_hash: None,
            storage_root: None,
        }))
    }

    fn begin_execution(&mut self) -> Result<()> {
        let context_id = uuid::Uuid::new_v4().to_string();
        let initial_state_root = self.get_state_root();
        
        let context = ExecutionContext {
            context_id,
            initial_state_root,
            pending_changes: HashMap::new(),
            executed_txs: Vec::new(),
            gas_used: 0,
        };
        
        *self.execution_context.lock().unwrap() = Some(context);
        Ok(())
    }

    fn commit_execution(&mut self) -> Result<Hash> {
        let mut ctx_lock = self.execution_context.lock().unwrap();
        
        if let Some(context) = ctx_lock.take() {
            // Apply pending changes to account states
            let mut account_states = self.account_states.lock().unwrap();
            for (address, state) in context.pending_changes {
                account_states.insert(address, state);
            }
            
            // Calculate new state root
            let new_state_root = self.calculate_state_root(&context.executed_txs);
            *self.state_root.lock().unwrap() = new_state_root.clone();
            
            Ok(new_state_root)
        } else {
            Err(failure::format_err!("No active execution context"))
        }
    }

    fn rollback_execution(&mut self) -> Result<()> {
        let mut ctx_lock = self.execution_context.lock().unwrap();
        
        if ctx_lock.is_some() {
            *ctx_lock = None;
            Ok(())
        } else {
            Err(failure::format_err!("No active execution context"))
        }
    }
}
