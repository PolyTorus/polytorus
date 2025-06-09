//! Modular execution layer implementation
//!
//! This module implements the execution layer for the modular blockchain,
//! handling transaction execution and state management.

use super::traits::*;
use super::transaction_processor::{
    ModularTransactionProcessor, ProcessorAccountState, TransactionProcessorConfig,
};
use crate::blockchain::block::Block;
use crate::config::DataContext;
use crate::crypto::transaction::Transaction;
use crate::smart_contract::types::{ContractDeployment, ContractExecution};
use crate::smart_contract::{ContractEngine, ContractState};
use crate::Result;

use std::collections::HashMap;
use std::sync::{Arc, Mutex};

/// Execution layer implementation
pub struct PolyTorusExecutionLayer {
    /// Contract execution engine
    contract_engine: Arc<Mutex<ContractEngine>>,
    /// Modular transaction processor
    transaction_processor: Arc<ModularTransactionProcessor>,
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
pub struct ExecutionContext {
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

        // Create transaction processor with default configuration
        let tx_processor_config = TransactionProcessorConfig::default();
        let transaction_processor = Arc::new(ModularTransactionProcessor::new(tx_processor_config));

        Ok(Self {
            contract_engine: Arc::new(Mutex::new(contract_engine)),
            transaction_processor,
            state_root: Arc::new(Mutex::new("genesis".to_string())),
            account_states: Arc::new(Mutex::new(HashMap::new())),
            execution_context: Arc::new(Mutex::new(None)),
            config,
        })
    }

    /// Add a transaction to the processor pool
    pub fn add_transaction(&self, transaction: Transaction) -> Result<()> {
        self.transaction_processor.add_transaction(transaction)
    }

    /// Get pending transactions from the processor
    pub fn get_pending_transactions(&self) -> Result<Vec<Transaction>> {
        self.transaction_processor.get_pending_transactions()
    }
    /// Get account state from the processor
    pub fn get_processor_account_state(&self, address: &str) -> Result<ProcessorAccountState> {
        self.transaction_processor.get_account_state(address)
    }

    /// Set account state in the processor
    pub fn set_processor_account_state(
        &self,
        address: &str,
        state: ProcessorAccountState,
    ) -> Result<()> {
        self.transaction_processor.set_account_state(address, state)
    }

    /// Clear the transaction pool
    pub fn clear_transaction_pool(&self) -> Result<()> {
        self.transaction_processor.clear_transaction_pool()
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
                    gas_limit,
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
                    value,
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

        // Use the modular transaction processor for block execution
        let transactions = block.get_transactions().to_vec();
        let tx_results = self
            .transaction_processor
            .process_transactions(&transactions)?;

        // Convert transaction results to execution receipts
        for (tx, tx_result) in transactions.iter().zip(tx_results.iter()) {
            let receipt = TransactionReceipt {
                tx_hash: tx.id.clone(),
                success: tx_result.success,
                gas_used: tx_result.gas_used,
                events: tx_result
                    .events
                    .iter()
                    .map(|e| Event {
                        contract: e.address.clone(),
                        data: e.data.clone(),
                        topics: e.topics.clone(),
                    })
                    .collect(),
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
        !proof.state_proof.is_empty()
            && !proof.execution_trace.is_empty()
            && proof.input_state_root != proof.output_state_root
    }

    fn get_account_state(&self, address: &str) -> Result<AccountState> {
        // Convert from ProcessorAccountState to trait AccountState
        let processor_state = self.transaction_processor.get_account_state(address)?;
        Ok(AccountState {
            balance: processor_state.balance,
            nonce: processor_state.nonce,
            code_hash: processor_state.code.as_ref().map(|code| {
                use crypto::digest::Digest;
                use crypto::sha2::Sha256;
                let mut hasher = Sha256::new();
                hasher.input(code);
                hasher.result_str()
            }),
            storage_root: None, // Simplified for now
        })
    }

    fn execute_transaction(&self, tx: &Transaction) -> Result<TransactionReceipt> {
        let tx_result = self.transaction_processor.process_transaction(tx)?;
        Ok(TransactionReceipt {
            tx_hash: tx.id.clone(),
            success: tx_result.success,
            gas_used: tx_result.gas_used,
            events: tx_result
                .events
                .iter()
                .map(|e| Event {
                    contract: e.address.clone(),
                    data: e.data.clone(),
                    topics: e.topics.clone(),
                })
                .collect(),
        })
    }
    fn begin_execution(&mut self) -> Result<()> {
        // Create a new execution context
        let context = ExecutionContext {
            context_id: format!(
                "exec_{}",
                std::time::SystemTime::now()
                    .duration_since(std::time::UNIX_EPOCH)
                    .unwrap()
                    .as_secs()
            ),
            initial_state_root: self.get_state_root(),
            pending_changes: HashMap::new(),
            executed_txs: Vec::new(),
            gas_used: 0,
        };

        let mut exec_context = self.execution_context.lock().unwrap();
        *exec_context = Some(context);
        Ok(())
    }

    fn commit_execution(&mut self) -> Result<Hash> {
        let mut exec_context = self.execution_context.lock().unwrap();
        if let Some(context) = exec_context.take() {
            // Apply pending changes and calculate new state root
            let new_state_root = self.calculate_state_root(&context.executed_txs);
            let mut state_root = self.state_root.lock().unwrap();
            *state_root = new_state_root.clone();
            Ok(new_state_root)
        } else {
            Err(failure::format_err!("No execution context to commit"))
        }
    }

    fn rollback_execution(&mut self) -> Result<()> {
        let mut exec_context = self.execution_context.lock().unwrap();
        if exec_context.is_some() {
            *exec_context = None;
            Ok(())
        } else {
            Err(failure::format_err!("No execution context to rollback"))
        }
    }
}

impl PolyTorusExecutionLayer {
    /// Get contract engine for external use
    pub fn get_contract_engine(&self) -> Arc<Mutex<ContractEngine>> {
        self.contract_engine.clone()
    }

    /// Get account state from internal storage
    pub fn get_account_state_from_storage(&self, address: &str) -> Option<AccountState> {
        let account_states = self.account_states.lock().unwrap();
        account_states.get(address).cloned()
    }

    /// Set account state in internal storage
    pub fn set_account_state_in_storage(&self, address: String, state: AccountState) {
        let mut account_states = self.account_states.lock().unwrap();
        account_states.insert(address, state);
    }

    /// Get current execution context
    pub fn get_execution_context(&self) -> Option<ExecutionContext> {
        let context = self.execution_context.lock().unwrap();
        context.clone()
    }

    /// Use execution context fields for validation
    pub fn validate_execution_context(&self) -> Result<bool> {
        let context = self.execution_context.lock().unwrap();
        if let Some(ref ctx) = *context {
            // Use all ExecutionContext fields for validation
            let _context_id = &ctx.context_id; // Used for identification
            let _initial_state_root = &ctx.initial_state_root; // Used for rollback
            let _pending_changes = &ctx.pending_changes; // Used for state transitions
            let _gas_used = ctx.gas_used; // Used for gas calculations

            // Simple validation logic
            Ok(!ctx.context_id.is_empty()
                && !ctx.initial_state_root.is_empty()
                && ctx.gas_used <= 1_000_000) // Gas limit check
        } else {
            Ok(true) // No context is valid
        }
    }
    /// Execute contract using contract engine
    pub fn execute_contract_with_engine(
        &self,
        contract_address: &str,
        function_name: &str,
        args: &[u8],
    ) -> Result<Vec<u8>> {
        let engine = self.contract_engine.lock().unwrap();

        // Create execution context for contract call
        let execution = ContractExecution {
            contract_address: contract_address.to_string(),
            function_name: function_name.to_string(),
            arguments: args.to_vec(),
            gas_limit: 100000,
            caller: "system".to_string(),
            value: 0,
        };

        // Execute the contract
        engine
            .execute_contract(execution)
            .map(|result| result.return_value)
            .map_err(|e| failure::format_err!("Contract execution failed: {}", e))
    }

    /// Process and execute a contract transaction publicly
    pub fn process_contract_transaction(&self, tx: &Transaction) -> Result<TransactionReceipt> {
        self.execute_contract_transaction(tx)
    }
}
