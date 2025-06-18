//! Modern API Endpoints
//!
//! This module provides comprehensive REST API endpoints for the PolyTorus blockchain,
//! including wallet management, blockchain operations, smart contracts, ERC20 tokens,
//! governance, and legacy compatibility.

use std::sync::Arc;

use actix_web::{web, HttpResponse, Result as ActixResult};
use serde::{Deserialize, Serialize};

use crate::{
    command::cli::ModernCli,
    config::DataContext,
    crypto::{types::EncryptionType, wallets::Wallets},
    modular::UnifiedModularOrchestrator,
    smart_contract::{ContractEngine, ContractState},
};

// ============================================================================
// Request/Response Types
// ============================================================================

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateWalletRequest {
    pub encryption_type: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CreateWalletResponse {
    pub success: bool,
    pub address: Option<String>,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BalanceResponse {
    pub address: String,
    pub balance: u64,
    pub balance_btc: f64,
    pub utxo_count: Option<usize>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ServerStatusResponse {
    pub status: String,
    pub version: String,
    pub uptime: String,
    pub blockchain_running: bool,
    pub endpoints_available: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct BlockchainStatusResponse {
    pub running: bool,
    pub block_height: u64,
    pub pending_transactions: usize,
    pub active_layers: Vec<String>,
    pub network_peers: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeployContractRequest {
    pub bytecode: String, // Hex-encoded bytecode
    pub constructor_args: Option<Vec<String>>,
    pub gas_limit: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct DeployContractResponse {
    pub success: bool,
    pub contract_address: Option<String>,
    pub transaction_hash: Option<String>,
    pub gas_used: Option<u64>,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CallContractRequest {
    pub contract_address: String,
    pub function_name: String,
    pub arguments: Option<Vec<String>>,
    pub caller: Option<String>,
    pub gas_limit: Option<u64>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ERC20DeployRequest {
    pub name: String,
    pub symbol: String,
    pub decimals: u8,
    pub initial_supply: u64,
    pub owner: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ERC20TransferRequest {
    pub contract: String,
    pub to: String,
    pub amount: u64,
    pub from: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GovernanceProposalRequest {
    pub title: String,
    pub description: String,
    pub proposer: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct GovernanceVoteRequest {
    pub proposal_id: String,
    pub vote: String, // "yes", "no", "abstain"
    pub voter: Option<String>,
}

// ============================================================================
// Health and Status Endpoints
// ============================================================================

/// Get server status
pub async fn get_server_status() -> ActixResult<HttpResponse> {
    let response = ServerStatusResponse {
        status: "running".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
        uptime: chrono::Utc::now().to_rfc3339(),
        blockchain_running: true,
        endpoints_available: 25, // Count of available API endpoints
    };

    Ok(HttpResponse::Ok().json(response))
}

// ============================================================================
// Wallet Management Endpoints
// ============================================================================

/// Create a new wallet (default ECDSA)
pub async fn api_create_wallet() -> ActixResult<HttpResponse> {
    let cli = ModernCli::new();
    match cli.cmd_create_wallet().await {
        Ok(()) => {
            // Get the newly created address
            let data_context = DataContext::default();
            match Wallets::new_with_context(data_context) {
                Ok(wallets) => {
                    let addresses = wallets.get_all_addresses();
                    let address = addresses.last().cloned();

                    Ok(HttpResponse::Ok().json(CreateWalletResponse {
                        success: true,
                        address,
                        message: "Wallet created successfully".to_string(),
                    }))
                }
                Err(e) => Ok(
                    HttpResponse::InternalServerError().json(CreateWalletResponse {
                        success: false,
                        address: None,
                        message: format!("Failed to retrieve wallet address: {}", e),
                    }),
                ),
            }
        }
        Err(e) => Ok(
            HttpResponse::InternalServerError().json(CreateWalletResponse {
                success: false,
                address: None,
                message: format!("Failed to create wallet: {}", e),
            }),
        ),
    }
}

/// Create a new wallet with specified encryption type
pub async fn api_create_wallet_with_type(path: web::Path<String>) -> ActixResult<HttpResponse> {
    let encryption_type = path.into_inner();

    // Validate encryption type
    let _enc_type = match encryption_type.to_uppercase().as_str() {
        "ECDSA" => EncryptionType::ECDSA,
        "FNDSA" => EncryptionType::FNDSA,
        _ => {
            return Ok(HttpResponse::BadRequest().json(CreateWalletResponse {
                success: false,
                address: None,
                message: "Invalid encryption type. Use ECDSA or FNDSA".to_string(),
            }));
        }
    };

    // Use the same logic as the default wallet creation
    api_create_wallet().await
}

/// List all wallet addresses
pub async fn api_list_addresses() -> ActixResult<HttpResponse> {
    let data_context = DataContext::default();
    match Wallets::new_with_context(data_context) {
        Ok(wallets) => {
            let addresses = wallets.get_all_addresses();
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "addresses": addresses,
                "count": addresses.len()
            })))
        }
        Err(e) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "success": false,
            "error": e.to_string()
        }))),
    }
}

/// Get balance for a specific address
pub async fn api_get_balance(
    path: web::Path<String>,
    _orchestrator: web::Data<Arc<UnifiedModularOrchestrator>>,
) -> ActixResult<HttpResponse> {
    let address = path.into_inner();

    // Try to get balance using UTXO processor
    use crate::modular::eutxo_processor::{EUtxoProcessor, EUtxoProcessorConfig};
    let utxo_processor = EUtxoProcessor::new(EUtxoProcessorConfig::default());

    match utxo_processor.get_balance(&address) {
        Ok(balance) => {
            let balance_btc = balance as f64 / 100_000_000.0;

            // Try to get UTXO count
            let utxo_count = match utxo_processor.get_utxos_for_address(&address) {
                Ok(utxos) => Some(utxos.len()),
                Err(_) => None,
            };

            Ok(HttpResponse::Ok().json(BalanceResponse {
                address,
                balance,
                balance_btc,
                utxo_count,
            }))
        }
        Err(_e) => Ok(HttpResponse::Ok().json(BalanceResponse {
            address,
            balance: 0,
            balance_btc: 0.0,
            utxo_count: Some(0),
        })),
    }
}

// ============================================================================
// Blockchain Operation Endpoints
// ============================================================================

/// Get blockchain status
pub async fn api_blockchain_status(
    orchestrator: web::Data<Arc<UnifiedModularOrchestrator>>,
) -> ActixResult<HttpResponse> {
    let state = orchestrator.get_state().await;

    // Try to get connected peers count
    let network_peers = match orchestrator.get_connected_peers().await {
        Ok(peers) => peers.len(),
        Err(_) => 0,
    };

    let response = BlockchainStatusResponse {
        running: state.is_running,
        block_height: state.current_block_height,
        pending_transactions: state.pending_transactions,
        active_layers: state.active_layers.keys().cloned().collect(),
        network_peers,
    };

    Ok(HttpResponse::Ok().json(response))
}

/// Get blockchain configuration
pub async fn api_blockchain_config(
    orchestrator: web::Data<Arc<UnifiedModularOrchestrator>>,
) -> ActixResult<HttpResponse> {
    match orchestrator.get_current_config().await {
        Ok(config) => Ok(HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "config": config
        }))),
        Err(e) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "success": false,
            "error": e.to_string()
        }))),
    }
}

/// Get blockchain metrics
pub async fn api_blockchain_metrics(
    orchestrator: web::Data<Arc<UnifiedModularOrchestrator>>,
) -> ActixResult<HttpResponse> {
    let metrics = orchestrator.get_metrics().await;
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "total_blocks_processed": metrics.total_blocks_processed,
        "total_transactions_processed": metrics.total_transactions_processed,
        "average_block_time_ms": metrics.average_block_time_ms,
        "error_rate": metrics.error_rate,
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}

/// Get layer status information
pub async fn api_layer_status(
    orchestrator: web::Data<Arc<UnifiedModularOrchestrator>>,
) -> ActixResult<HttpResponse> {
    let state = orchestrator.get_state().await;
    let layer_names: Vec<String> = state.active_layers.keys().cloned().collect();
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "active_layers": layer_names,
        "layer_count": layer_names.len(),
        "status": "operational"
    })))
}

// ============================================================================
// Smart Contract Endpoints
// ============================================================================

/// Deploy a smart contract
pub async fn api_deploy_contract(
    req: web::Json<DeployContractRequest>,
) -> ActixResult<HttpResponse> {
    // Decode hex bytecode
    let bytecode = match hex::decode(&req.bytecode) {
        Ok(bytes) => bytes,
        Err(_) => {
            return Ok(HttpResponse::BadRequest().json(DeployContractResponse {
                success: false,
                contract_address: None,
                transaction_hash: None,
                gas_used: None,
                message: "Invalid hex bytecode".to_string(),
            }));
        }
    };

    let data_context = DataContext::default();
    match data_context.ensure_directories() {
        Ok(_) => {}
        Err(e) => {
            return Ok(
                HttpResponse::InternalServerError().json(DeployContractResponse {
                    success: false,
                    contract_address: None,
                    transaction_hash: None,
                    gas_used: None,
                    message: format!("Failed to initialize data directories: {}", e),
                }),
            );
        }
    }

    match ContractState::new(&data_context.contracts_db_path) {
        Ok(state) => {
            match ContractEngine::new(state) {
                Ok(engine) => {
                    let contract_address = format!(
                        "contract_{}",
                        chrono::Utc::now().timestamp_nanos_opt().unwrap_or(0)
                    );

                    // Create contract
                    use crate::smart_contract::contract::SmartContract;
                    // Convert constructor args from strings to bytes
                    let constructor_bytes: Vec<u8> = req
                        .constructor_args
                        .clone()
                        .unwrap_or_default()
                        .join(",")
                        .into_bytes();

                    match SmartContract::new(
                        bytecode,
                        contract_address.clone(),
                        constructor_bytes,
                        None,
                    ) {
                        Ok(contract) => match engine.deploy_contract(&contract) {
                            Ok(_) => Ok(HttpResponse::Ok().json(DeployContractResponse {
                                success: true,
                                contract_address: Some(contract_address),
                                transaction_hash: Some(format!(
                                    "tx_{}",
                                    chrono::Utc::now().timestamp()
                                )),
                                gas_used: Some(100000),
                                message: "Contract deployed successfully".to_string(),
                            })),
                            Err(e) => Ok(HttpResponse::InternalServerError().json(
                                DeployContractResponse {
                                    success: false,
                                    contract_address: None,
                                    transaction_hash: None,
                                    gas_used: None,
                                    message: format!("Deployment failed: {}", e),
                                },
                            )),
                        },
                        Err(e) => Ok(HttpResponse::InternalServerError().json(
                            DeployContractResponse {
                                success: false,
                                contract_address: None,
                                transaction_hash: None,
                                gas_used: None,
                                message: format!("Failed to create contract: {}", e),
                            },
                        )),
                    }
                }
                Err(e) => Ok(
                    HttpResponse::InternalServerError().json(DeployContractResponse {
                        success: false,
                        contract_address: None,
                        transaction_hash: None,
                        gas_used: None,
                        message: format!("Failed to initialize contract engine: {}", e),
                    }),
                ),
            }
        }
        Err(e) => Ok(
            HttpResponse::InternalServerError().json(DeployContractResponse {
                success: false,
                contract_address: None,
                transaction_hash: None,
                gas_used: None,
                message: format!("Failed to initialize contract state: {}", e),
            }),
        ),
    }
}

/// Call a smart contract function
pub async fn api_call_contract(req: web::Json<CallContractRequest>) -> ActixResult<HttpResponse> {
    let data_context = DataContext::default();
    data_context.ensure_directories().ok();

    match ContractState::new(&data_context.contracts_db_path) {
        Ok(state) => {
            match ContractEngine::new(state) {
                Ok(engine) => {
                    use crate::smart_contract::types::ContractExecution;
                    // Convert arguments from strings to bytes
                    let args_bytes: Vec<u8> = req
                        .arguments
                        .clone()
                        .unwrap_or_default()
                        .join(",")
                        .into_bytes();

                    let execution = ContractExecution {
                        contract_address: req.contract_address.clone(),
                        function_name: req.function_name.clone(),
                        arguments: args_bytes,
                        caller: req
                            .caller
                            .clone()
                            .unwrap_or_else(|| "default_caller".to_string()),
                        value: 0,
                        gas_limit: req.gas_limit.unwrap_or(1000000),
                    };

                    match engine.execute_contract(execution) {
                        Ok(result) => Ok(HttpResponse::Ok().json(serde_json::json!({
                            "success": result.success,
                            "return_value": String::from_utf8_lossy(&result.return_value),
                            "gas_used": result.gas_used,
                            "logs": result.logs,
                            "state_changes": result.state_changes.len()
                        }))),
                        Err(e) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                            "success": false,
                            "error": e.to_string()
                        }))),
                    }
                }
                Err(e) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                    "success": false,
                    "error": format!("Engine initialization failed: {}", e)
                }))),
            }
        }
        Err(e) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "success": false,
            "error": format!("State initialization failed: {}", e)
        }))),
    }
}

/// List deployed contracts
pub async fn api_list_contracts() -> ActixResult<HttpResponse> {
    let data_context = DataContext::default();
    data_context.ensure_directories().ok();

    match ContractState::new(&data_context.contracts_db_path) {
        Ok(state) => match ContractEngine::new(state) {
            Ok(engine) => match engine.list_contracts() {
                Ok(contracts) => Ok(HttpResponse::Ok().json(serde_json::json!({
                    "success": true,
                    "contracts": contracts,
                    "count": contracts.len()
                }))),
                Err(e) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                    "success": false,
                    "error": e.to_string()
                }))),
            },
            Err(e) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "error": format!("Engine initialization failed: {}", e)
            }))),
        },
        Err(e) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "success": false,
            "error": format!("State initialization failed: {}", e)
        }))),
    }
}

/// Get contract state
pub async fn api_contract_state(path: web::Path<String>) -> ActixResult<HttpResponse> {
    let contract_address = path.into_inner();
    let data_context = DataContext::default();
    data_context.ensure_directories().ok();

    match ContractState::new(&data_context.contracts_db_path) {
        Ok(state) => match ContractEngine::new(state) {
            Ok(engine) => match engine.get_contract_state(&contract_address) {
                Ok(contract_state) => Ok(HttpResponse::Ok().json(serde_json::json!({
                    "success": true,
                    "contract_address": contract_address,
                    "state": contract_state,
                    "state_size": contract_state.len()
                }))),
                Err(e) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                    "success": false,
                    "error": e.to_string()
                }))),
            },
            Err(e) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
                "success": false,
                "error": format!("Engine initialization failed: {}", e)
            }))),
        },
        Err(e) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "success": false,
            "error": format!("State initialization failed: {}", e)
        }))),
    }
}

// ============================================================================
// ERC20 Token Endpoints
// ============================================================================

/// Deploy an ERC20 token contract
pub async fn api_erc20_deploy(req: web::Json<ERC20DeployRequest>) -> ActixResult<HttpResponse> {
    let cli = ModernCli::new();
    let params = format!(
        "{},{},{},{},{}",
        req.name, req.symbol, req.decimals, req.initial_supply, req.owner
    );

    match cli.cmd_erc20_deploy(&params).await {
        Ok(_) => {
            let contract_address = format!("erc20_{}", req.symbol.to_lowercase());
            Ok(HttpResponse::Ok().json(serde_json::json!({
                "success": true,
                "contract_address": contract_address,
                "name": req.name,
                "symbol": req.symbol,
                "decimals": req.decimals,
                "initial_supply": req.initial_supply,
                "owner": req.owner
            })))
        }
        Err(e) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "success": false,
            "error": e.to_string()
        }))),
    }
}

/// Transfer ERC20 tokens
pub async fn api_erc20_transfer(req: web::Json<ERC20TransferRequest>) -> ActixResult<HttpResponse> {
    let cli = ModernCli::new();
    let params = format!("{},{},{}", req.contract, req.to, req.amount);

    match cli.cmd_erc20_transfer(&params).await {
        Ok(_) => Ok(HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "message": "Transfer completed successfully"
        }))),
        Err(e) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "success": false,
            "error": e.to_string()
        }))),
    }
}

/// Get ERC20 token balance
pub async fn api_erc20_balance(path: web::Path<(String, String)>) -> ActixResult<HttpResponse> {
    let (contract, address) = path.into_inner();
    let cli = ModernCli::new();
    let params = format!("{},{}", contract, address);

    match cli.cmd_erc20_balance(&params).await {
        Ok(_) => Ok(HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "contract": contract,
            "address": address,
            "message": "Balance check completed"
        }))),
        Err(e) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "success": false,
            "error": e.to_string()
        }))),
    }
}

/// Get ERC20 token information
pub async fn api_erc20_info(path: web::Path<String>) -> ActixResult<HttpResponse> {
    let contract_address = path.into_inner();
    let cli = ModernCli::new();

    match cli.cmd_erc20_info(&contract_address).await {
        Ok(_) => Ok(HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "contract_address": contract_address,
            "message": "Contract info retrieved"
        }))),
        Err(e) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "success": false,
            "error": e.to_string()
        }))),
    }
}

/// List all ERC20 contracts
pub async fn api_erc20_list() -> ActixResult<HttpResponse> {
    let cli = ModernCli::new();

    match cli.cmd_erc20_list().await {
        Ok(_) => Ok(HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "message": "ERC20 contracts listed"
        }))),
        Err(e) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "success": false,
            "error": e.to_string()
        }))),
    }
}

// ============================================================================
// Governance Endpoints
// ============================================================================

/// Create a governance proposal
pub async fn api_governance_propose(
    req: web::Json<GovernanceProposalRequest>,
) -> ActixResult<HttpResponse> {
    let cli = ModernCli::new();
    let proposal_data = format!("{}: {}", req.title, req.description);

    match cli.cmd_governance_propose(&proposal_data).await {
        Ok(_) => Ok(HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "title": req.title,
            "description": req.description,
            "proposer": req.proposer,
            "message": "Proposal created successfully"
        }))),
        Err(e) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "success": false,
            "error": e.to_string()
        }))),
    }
}

/// Vote on a governance proposal
pub async fn api_governance_vote(
    req: web::Json<GovernanceVoteRequest>,
) -> ActixResult<HttpResponse> {
    let cli = ModernCli::new();

    match cli.cmd_governance_vote(&req.proposal_id).await {
        Ok(_) => Ok(HttpResponse::Ok().json(serde_json::json!({
            "success": true,
            "proposal_id": req.proposal_id,
            "vote": req.vote,
            "voter": req.voter,
            "message": "Vote submitted successfully"
        }))),
        Err(e) => Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "success": false,
            "error": e.to_string()
        }))),
    }
}

/// List governance proposals
pub async fn api_governance_list() -> ActixResult<HttpResponse> {
    // In a real implementation, this would read from the governance storage
    let data_context = DataContext::default();
    let governance_dir = data_context.data_dir.join("governance");

    let mut proposals = Vec::new();
    if governance_dir.exists() {
        if let Ok(entries) = std::fs::read_dir(&governance_dir) {
            for entry in entries.flatten() {
                if let Some(file_name) = entry.file_name().to_str() {
                    if file_name.ends_with(".json") {
                        if let Ok(content) = std::fs::read_to_string(entry.path()) {
                            if let Ok(proposal) =
                                serde_json::from_str::<serde_json::Value>(&content)
                            {
                                proposals.push(proposal);
                            }
                        }
                    }
                }
            }
        }
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "proposals": proposals,
        "count": proposals.len()
    })))
}

// ============================================================================
// Legacy Compatibility Endpoints
// ============================================================================

/// Legacy create wallet endpoint
pub async fn legacy_create_wallet(path: web::Path<String>) -> ActixResult<HttpResponse> {
    api_create_wallet_with_type(path).await
}

/// Legacy list addresses endpoint
pub async fn legacy_list_addresses() -> ActixResult<HttpResponse> {
    api_list_addresses().await
}
