//! Comprehensive RPC API for Modular Blockchain
//!
//! This module provides a complete JSON-RPC API for external clients
//! to interact with the modular blockchain, including wallet operations,
//! transaction submission, block queries, and network information.

use std::sync::Arc;

use actix_web::{
    middleware,
    web::{self, Data, Json},
    App, HttpResponse, HttpServer, Result as ActixResult,
};
use anyhow::{anyhow, Result};
use serde::{Deserialize, Serialize};
use serde_json::{json, Value};

use crate::{
    blockchain::block::FinalizedBlock,
    crypto::{transaction::Transaction, wallets::WalletManager},
    modular::{
        mempool::{MempoolStats, TransactionMempool, TransactionStatus},
        peer_discovery::PeerDiscoveryService,
        state_sync::{StateSynchronizer, SyncState},
        storage::ModularStorage,
        unified_orchestrator::UnifiedModularOrchestrator,
    },
};

/// JSON-RPC request structure
#[derive(Debug, Deserialize)]
pub struct JsonRpcRequest {
    pub jsonrpc: String,
    pub method: String,
    pub params: Option<Value>,
    pub id: Option<Value>,
}

/// JSON-RPC response structure
#[derive(Debug, Serialize)]
pub struct JsonRpcResponse {
    pub jsonrpc: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub result: Option<Value>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub error: Option<JsonRpcError>,
    pub id: Option<Value>,
}

/// JSON-RPC error structure
#[derive(Debug, Serialize)]
pub struct JsonRpcError {
    pub code: i32,
    pub message: String,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub data: Option<Value>,
}

/// Block information for API responses
#[derive(Debug, Serialize, Deserialize)]
pub struct BlockInfo {
    pub hash: String,
    pub height: u64,
    pub previous_hash: String,
    pub timestamp: u64,
    pub nonce: u64,
    pub difficulty: u32,
    pub transaction_count: usize,
    pub transactions: Vec<TransactionInfo>,
    pub size: usize,
}

/// Transaction information for API responses
#[derive(Debug, Serialize, Deserialize)]
pub struct TransactionInfo {
    pub hash: String,
    pub from: String,
    pub to: String,
    pub value: u64,
    pub fee: u64,
    pub gas_price: u64,
    pub status: TransactionStatus,
    pub block_hash: Option<String>,
    pub block_height: Option<u64>,
    pub transaction_index: Option<usize>,
}

/// Account information for API responses
#[derive(Debug, Serialize, Deserialize)]
pub struct AccountInfo {
    pub address: String,
    pub balance: u64,
    pub nonce: u64,
    pub transaction_count: u64,
}

/// Network information for API responses
#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkInfo {
    pub chain_id: String,
    pub network_name: String,
    pub protocol_version: String,
    pub best_block_height: u64,
    pub best_block_hash: String,
    pub peer_count: usize,
    pub sync_state: SyncState,
    pub is_mining: bool,
}

/// Node status information
#[derive(Debug, Serialize, Deserialize)]
pub struct NodeStatus {
    pub version: String,
    pub uptime: u64,
    pub network: NetworkInfo,
    pub mempool: MempoolStats,
    pub storage: StorageStats,
    pub peers: Vec<PeerInfo>,
}

/// Peer information for API responses
#[derive(Debug, Serialize, Deserialize)]
pub struct PeerInfo {
    pub id: String,
    pub address: String,
    pub connected_at: u64,
    pub best_height: u64,
    pub ping_ms: Option<u64>,
    pub capabilities: Vec<String>,
}

/// Storage statistics
#[derive(Debug, Serialize, Deserialize)]
pub struct StorageStats {
    pub blocks_count: u64,
    pub transactions_count: u64,
    pub accounts_count: u64,
    pub storage_size_bytes: u64,
}

/// Application state for RPC server
#[derive(Clone)]
pub struct RpcState {
    pub orchestrator: Arc<UnifiedModularOrchestrator>,
    pub storage: Arc<ModularStorage>,
    pub mempool: Arc<TransactionMempool>,
    pub wallet_manager: Arc<WalletManager>,
    pub synchronizer: Arc<StateSynchronizer>,
    pub peer_discovery: Arc<PeerDiscoveryService>,
}

/// RPC API server
pub struct RpcApiServer {
    state: RpcState,
    bind_address: String,
}

impl RpcApiServer {
    /// Create a new RPC API server
    pub fn new(
        orchestrator: Arc<UnifiedModularOrchestrator>,
        storage: Arc<ModularStorage>,
        mempool: Arc<TransactionMempool>,
        wallet_manager: Arc<WalletManager>,
        synchronizer: Arc<StateSynchronizer>,
        peer_discovery: Arc<PeerDiscoveryService>,
        bind_address: String,
    ) -> Self {
        let state = RpcState {
            orchestrator,
            storage,
            mempool,
            wallet_manager,
            synchronizer,
            peer_discovery,
        };

        Self {
            state,
            bind_address,
        }
    }

    /// Start the RPC server
    pub async fn start(self) -> Result<()> {
        log::info!("Starting RPC API server on {}", self.bind_address);

        HttpServer::new(move || {
            App::new()
                .app_data(Data::new(self.state.clone()))
                .wrap(middleware::Logger::default())
                .wrap(middleware::DefaultHeaders::new().add(("Access-Control-Allow-Origin", "*")))
                .service(
                    web::scope("/rpc")
                        .route("/", web::post().to(handle_rpc_request))
                        .route("/health", web::get().to(health_check))
                        .route("/status", web::get().to(get_node_status)),
                )
        })
        .bind(&self.bind_address)?
        .run()
        .await?;

        Ok(())
    }
}

/// Handle JSON-RPC requests
async fn handle_rpc_request(
    state: Data<RpcState>,
    request: Json<JsonRpcRequest>,
) -> ActixResult<HttpResponse> {
    let response = process_rpc_request(&state, &request).await;
    Ok(HttpResponse::Ok().json(response))
}

/// Process individual RPC requests
async fn process_rpc_request(state: &RpcState, request: &JsonRpcRequest) -> JsonRpcResponse {
    let id = request.id.clone();

    if request.jsonrpc != "2.0" {
        return JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(JsonRpcError {
                code: -32600,
                message: "Invalid JSON-RPC version".to_string(),
                data: None,
            }),
            id,
        };
    }

    let result = match request.method.as_str() {
        // Blockchain queries
        "eth_blockNumber" => get_block_number(state).await,
        "eth_getBlockByNumber" => get_block_by_number(state, &request.params).await,
        "eth_getBlockByHash" => get_block_by_hash(state, &request.params).await,
        "eth_getTransactionByHash" => get_transaction_by_hash(state, &request.params).await,
        "eth_getBalance" => get_balance(state, &request.params).await,
        "eth_getTransactionCount" => get_transaction_count(state, &request.params).await,

        // Transaction operations
        "eth_sendTransaction" => send_transaction(state, &request.params).await,
        "eth_sendRawTransaction" => send_raw_transaction(state, &request.params).await,
        "eth_estimateGas" => estimate_gas(state, &request.params).await,
        "eth_gasPrice" => get_gas_price(state).await,

        // Wallet operations
        "personal_newAccount" => create_account(state, &request.params).await,
        "personal_listAccounts" => list_accounts(state).await,
        "personal_unlockAccount" => unlock_account(state, &request.params).await,

        // Network information
        "net_version" => get_network_version(state).await,
        "net_peerCount" => get_peer_count(state).await,
        "net_listening" => get_listening_status(state).await,

        // Node information
        "web3_clientVersion" => get_client_version(state).await,
        "polytorus_nodeStatus" => get_node_status_rpc(state).await,
        "polytorus_syncStatus" => get_sync_status(state).await,
        "polytorus_mempoolStatus" => get_mempool_status(state).await,

        // Mining operations
        "miner_start" => start_mining(state, &request.params).await,
        "miner_stop" => stop_mining(state).await,
        "miner_setEtherbase" => set_mining_address(state, &request.params).await,

        // Custom PolyTorus methods
        "polytorus_getNetworkTopology" => get_network_topology(state).await,
        "polytorus_getPeers" => get_peers(state).await,
        "polytorus_addPeer" => add_peer(state, &request.params).await,
        "polytorus_removePeer" => remove_peer(state, &request.params).await,

        _ => Err(anyhow!("Method not found: {}", request.method)),
    };

    match result {
        Ok(value) => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: Some(value),
            error: None,
            id,
        },
        Err(e) => JsonRpcResponse {
            jsonrpc: "2.0".to_string(),
            result: None,
            error: Some(JsonRpcError {
                code: -32603,
                message: e.to_string(),
                data: None,
            }),
            id,
        },
    }
}

/// Get current block number
async fn get_block_number(state: &RpcState) -> Result<Value> {
    let height = state.storage.get_latest_block_height().await?;
    Ok(json!(format!("0x{:x}", height)))
}

/// Get block by number
async fn get_block_by_number(state: &RpcState, params: &Option<Value>) -> Result<Value> {
    let params = params
        .as_ref()
        .ok_or_else(|| anyhow!("Missing parameters"))?;
    let params_array = params
        .as_array()
        .ok_or_else(|| anyhow!("Invalid parameters"))?;

    if params_array.len() < 2 {
        return Err(anyhow!("Insufficient parameters"));
    }

    let block_number = parse_block_number(&params_array[0])?;
    let include_transactions = params_array[1].as_bool().unwrap_or(false);

    if let Some(block) = state.storage.get_block_by_height(block_number).await? {
        let block_info = convert_block_to_info(&block, include_transactions)?;
        Ok(serde_json::to_value(block_info)?)
    } else {
        Ok(Value::Null)
    }
}

/// Get block by hash
async fn get_block_by_hash(state: &RpcState, params: &Option<Value>) -> Result<Value> {
    let params = params
        .as_ref()
        .ok_or_else(|| anyhow!("Missing parameters"))?;
    let params_array = params
        .as_array()
        .ok_or_else(|| anyhow!("Invalid parameters"))?;

    if params_array.len() < 2 {
        return Err(anyhow!("Insufficient parameters"));
    }

    let block_hash = params_array[0]
        .as_str()
        .ok_or_else(|| anyhow!("Invalid block hash"))?;
    let include_transactions = params_array[1].as_bool().unwrap_or(false);

    if let Some(block) = state.storage.get_block_by_hash(block_hash).await? {
        let block_info = convert_block_to_info(&block, include_transactions)?;
        Ok(serde_json::to_value(block_info)?)
    } else {
        Ok(Value::Null)
    }
}

/// Get transaction by hash
async fn get_transaction_by_hash(state: &RpcState, params: &Option<Value>) -> Result<Value> {
    let params = params
        .as_ref()
        .ok_or_else(|| anyhow!("Missing parameters"))?;
    let params_array = params
        .as_array()
        .ok_or_else(|| anyhow!("Invalid parameters"))?;

    if params_array.is_empty() {
        return Err(anyhow!("Missing transaction hash"));
    }

    let tx_hash = params_array[0]
        .as_str()
        .ok_or_else(|| anyhow!("Invalid transaction hash"))?;

    // First check mempool
    if let Some(mempool_tx) = state.mempool.get_transaction(tx_hash).await {
        let tx_info = TransactionInfo {
            hash: mempool_tx.transaction.get_id(),
            from: mempool_tx.transaction.get_from().to_string(),
            to: mempool_tx.transaction.get_to().to_string(),
            value: mempool_tx.transaction.get_amount(),
            fee: mempool_tx.fee,
            gas_price: mempool_tx.gas_price,
            status: mempool_tx.status,
            block_hash: None,
            block_height: None,
            transaction_index: None,
        };
        return Ok(serde_json::to_value(tx_info)?);
    }

    // Then check storage
    // Implementation would query transaction from storage
    Ok(Value::Null)
}

/// Send a transaction
async fn send_transaction(state: &RpcState, params: &Option<Value>) -> Result<Value> {
    let params = params
        .as_ref()
        .ok_or_else(|| anyhow!("Missing parameters"))?;
    let tx_params = params
        .as_object()
        .ok_or_else(|| anyhow!("Invalid transaction parameters"))?;

    let from = tx_params
        .get("from")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Missing 'from' field"))?;
    let to = tx_params
        .get("to")
        .and_then(|v| v.as_str())
        .ok_or_else(|| anyhow!("Missing 'to' field"))?;
    let value = tx_params
        .get("value")
        .and_then(|v| v.as_str())
        .map(|s| u64::from_str_radix(s.trim_start_matches("0x"), 16))
        .transpose()?
        .unwrap_or(0);

    let transaction = Transaction::new(from.to_string(), to.to_string(), value);
    let fee = 1000; // Default fee
    let gas_price = 100; // Default gas price

    state
        .mempool
        .add_transaction(transaction.clone(), fee, gas_price)
        .await?;

    Ok(json!(transaction.get_id()))
}

/// Get account balance
async fn get_balance(_state: &RpcState, params: &Option<Value>) -> Result<Value> {
    let params = params
        .as_ref()
        .ok_or_else(|| anyhow!("Missing parameters"))?;
    let params_array = params
        .as_array()
        .ok_or_else(|| anyhow!("Invalid parameters"))?;

    if params_array.is_empty() {
        return Err(anyhow!("Missing address"));
    }

    let _address = params_array[0]
        .as_str()
        .ok_or_else(|| anyhow!("Invalid address"))?;

    // Implementation would query balance from state
    let balance = 1000000u64; // Placeholder
    Ok(json!(format!("0x{:x}", balance)))
}

/// Get peer count
async fn get_peer_count(state: &RpcState) -> Result<Value> {
    let peers = state.peer_discovery.get_known_nodes();
    Ok(json!(format!("0x{:x}", peers.len())))
}

/// Get node status
async fn get_node_status_rpc(state: &RpcState) -> Result<Value> {
    let height = state.storage.get_latest_block_height().await?;
    let mempool_stats = state.mempool.get_stats().await;
    let sync_state = state.synchronizer.get_sync_state();
    let peers = state.peer_discovery.get_known_nodes();

    let status = NodeStatus {
        version: "PolyTorus/1.0.0".to_string(),
        uptime: 0, // Implementation would track actual uptime
        network: NetworkInfo {
            chain_id: "polytorus-testnet".to_string(),
            network_name: "PolyTorus Testnet".to_string(),
            protocol_version: "1.0".to_string(),
            best_block_height: height,
            best_block_hash: "".to_string(), // Would get from storage
            peer_count: peers.len(),
            sync_state,
            is_mining: false, // Would check mining status
        },
        mempool: mempool_stats,
        storage: StorageStats {
            blocks_count: height,
            transactions_count: 0, // Would count from storage
            accounts_count: 0,     // Would count from storage
            storage_size_bytes: 0, // Would calculate actual size
        },
        peers: peers
            .into_iter()
            .map(|peer| PeerInfo {
                id: peer.node_id.to_string(),
                address: peer.address.to_string(),
                connected_at: peer.last_seen,
                best_height: peer.chain_height,
                ping_ms: peer.ping_ms,
                capabilities: if peer.capabilities.mining {
                    vec!["mining".to_string()]
                } else {
                    vec![]
                },
            })
            .collect(),
    };

    Ok(serde_json::to_value(status)?)
}

/// Health check endpoint
async fn health_check() -> ActixResult<HttpResponse> {
    Ok(HttpResponse::Ok().json(json!({"status": "healthy"})))
}

/// Node status endpoint
async fn get_node_status(state: Data<RpcState>) -> ActixResult<HttpResponse> {
    match get_node_status_rpc(&state).await {
        Ok(status) => Ok(HttpResponse::Ok().json(status)),
        Err(e) => Ok(HttpResponse::InternalServerError().json(json!({"error": e.to_string()}))),
    }
}

/// Helper functions
fn parse_block_number(value: &Value) -> Result<u64> {
    match value {
        Value::String(s) => {
            if s == "latest" || s == "pending" {
                Ok(u64::MAX) // Will be resolved to latest height
            } else if s == "earliest" {
                Ok(0)
            } else if let Some(stripped) = s.strip_prefix("0x") {
                Ok(u64::from_str_radix(stripped, 16)?)
            } else {
                Ok(s.parse()?)
            }
        }
        Value::Number(n) => Ok(n.as_u64().unwrap_or(0)),
        _ => Err(anyhow!("Invalid block number format")),
    }
}

fn convert_block_to_info(block: &FinalizedBlock, include_transactions: bool) -> Result<BlockInfo> {
    let transactions = if include_transactions {
        block
            .get_transactions()
            .iter()
            .map(|tx| TransactionInfo {
                hash: tx.get_id(),
                from: tx.get_from().to_string(),
                to: tx.get_to().to_string(),
                value: tx.get_amount(),
                fee: 0,       // Would need to store fee information
                gas_price: 0, // Would need to store gas price
                status: TransactionStatus::Validated,
                block_hash: Some(block.get_hash().to_string()),
                block_height: Some(block.get_height() as u64),
                transaction_index: None,
            })
            .collect()
    } else {
        Vec::new()
    };

    Ok(BlockInfo {
        hash: block.get_hash().to_string(),
        height: block.get_height() as u64,
        previous_hash: block.get_prev_hash().to_string(),
        timestamp: block.get_timestamp() as u64,
        nonce: block.get_nonce() as u64,
        difficulty: block.get_difficulty() as u32,
        transaction_count: block.get_transactions().len(),
        transactions,
        size: 0, // Would calculate actual block size
    })
}

// Placeholder implementations for missing methods
async fn send_raw_transaction(_state: &RpcState, _params: &Option<Value>) -> Result<Value> {
    Err(anyhow!("Method not implemented"))
}

async fn estimate_gas(_state: &RpcState, _params: &Option<Value>) -> Result<Value> {
    Ok(json!("0x5208")) // 21000 gas (basic transaction)
}

async fn get_gas_price(state: &RpcState) -> Result<Value> {
    let fee = state.mempool.estimate_fee().await;
    Ok(json!(format!("0x{:x}", fee)))
}

async fn create_account(_state: &RpcState, _params: &Option<Value>) -> Result<Value> {
    Err(anyhow!("Method not implemented"))
}

async fn list_accounts(_state: &RpcState) -> Result<Value> {
    Ok(json!([]))
}

async fn unlock_account(_state: &RpcState, _params: &Option<Value>) -> Result<Value> {
    Ok(json!(true))
}

async fn get_network_version(_state: &RpcState) -> Result<Value> {
    Ok(json!("1"))
}

async fn get_listening_status(_state: &RpcState) -> Result<Value> {
    Ok(json!(true))
}

async fn get_client_version(_state: &RpcState) -> Result<Value> {
    Ok(json!("PolyTorus/1.0.0"))
}

async fn get_sync_status(state: &RpcState) -> Result<Value> {
    let sync_state = state.synchronizer.get_sync_state();
    Ok(serde_json::to_value(sync_state)?)
}

async fn get_mempool_status(state: &RpcState) -> Result<Value> {
    let stats = state.mempool.get_stats().await;
    Ok(serde_json::to_value(stats)?)
}

async fn start_mining(_state: &RpcState, _params: &Option<Value>) -> Result<Value> {
    Ok(json!(true))
}

async fn stop_mining(_state: &RpcState) -> Result<Value> {
    Ok(json!(true))
}

async fn set_mining_address(_state: &RpcState, _params: &Option<Value>) -> Result<Value> {
    Ok(json!(true))
}

async fn get_network_topology(_state: &RpcState) -> Result<Value> {
    // Implementation would return actual network topology
    Ok(json!({}))
}

async fn get_peers(state: &RpcState) -> Result<Value> {
    let peers = state.peer_discovery.get_known_nodes();
    Ok(serde_json::to_value(peers)?)
}

async fn add_peer(_state: &RpcState, _params: &Option<Value>) -> Result<Value> {
    Ok(json!(true))
}

async fn remove_peer(_state: &RpcState, _params: &Option<Value>) -> Result<Value> {
    Ok(json!(true))
}

async fn get_transaction_count(_state: &RpcState, _params: &Option<Value>) -> Result<Value> {
    Ok(json!("0x0"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_block_number() {
        assert_eq!(parse_block_number(&json!("latest")).unwrap(), u64::MAX);
        assert_eq!(parse_block_number(&json!("earliest")).unwrap(), 0);
        assert_eq!(parse_block_number(&json!("0x10")).unwrap(), 16);
        assert_eq!(parse_block_number(&json!(42)).unwrap(), 42);
    }

    #[test]
    fn test_json_rpc_error() {
        let error = JsonRpcError {
            code: -32600,
            message: "Invalid Request".to_string(),
            data: None,
        };

        let json = serde_json::to_string(&error).unwrap();
        assert!(json.contains("Invalid Request"));
    }
}
