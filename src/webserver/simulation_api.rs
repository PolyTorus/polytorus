//! Simulation API endpoints for multi-node testing

use std::sync::Arc;

use actix_web::{web, HttpResponse, Result};
use serde::{Deserialize, Serialize};
use tokio::sync::Mutex;
use uuid::Uuid;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionRequest {
    pub from: String,
    pub to: String,
    pub amount: u64,
    pub nonce: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransactionResponse {
    pub status: String,
    pub transaction_id: String,
    pub message: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeStatus {
    pub status: String,
    pub block_height: u64,
    pub is_running: bool,
    pub total_transactions: u64,
    pub total_blocks: u64,
    pub error_rate: f64,
    pub node_id: String,
    pub data_dir: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct NodeStats {
    pub transactions_sent: u64,
    pub transactions_received: u64,
    pub timestamp: String,
    pub node_id: String,
}

#[derive(Debug, Clone)]
pub struct SimulationState {
    pub node_id: String,
    pub data_dir: String,
    pub tx_count: Arc<Mutex<u64>>,
    pub rx_count: Arc<Mutex<u64>>,
}

impl SimulationState {
    pub fn new(node_id: String, data_dir: String) -> Self {
        Self {
            node_id,
            data_dir,
            tx_count: Arc::new(Mutex::new(0)),
            rx_count: Arc::new(Mutex::new(0)),
        }
    }
}

/// Get node status endpoint
pub async fn get_status(state: web::Data<SimulationState>) -> Result<HttpResponse> {
    let status = NodeStatus {
        status: "running".to_string(),
        block_height: 0, // TODO: Get actual block height
        is_running: true,
        total_transactions: *state.rx_count.lock().await,
        total_blocks: 0, // TODO: Get actual block count
        error_rate: 0.0,
        node_id: state.node_id.clone(),
        data_dir: state.data_dir.clone(),
    };

    Ok(HttpResponse::Ok().json(status))
}

/// Submit transaction endpoint (receives transaction from another node)
pub async fn submit_transaction(
    state: web::Data<SimulationState>,
    req: web::Json<TransactionRequest>,
) -> Result<HttpResponse> {
    // Increment received transaction count
    *state.rx_count.lock().await += 1;

    let response = TransactionResponse {
        status: "accepted".to_string(),
        transaction_id: Uuid::new_v4().to_string(),
        message: Some(format!(
            "Transaction from {} to {} for {} accepted",
            req.from, req.to, req.amount
        )),
    };

    println!(
        "� Transaction received on {}: {} -> {} ({})",
        state.node_id, req.from, req.to, req.amount
    );

    Ok(HttpResponse::Ok().json(response))
}

/// Send transaction endpoint (sends transaction from this node)
pub async fn send_transaction(
    state: web::Data<SimulationState>,
    req: web::Json<TransactionRequest>,
) -> Result<HttpResponse> {
    // Increment sent transaction count
    *state.tx_count.lock().await += 1;

    let response = TransactionResponse {
        status: "sent".to_string(),
        transaction_id: Uuid::new_v4().to_string(),
        message: Some(format!(
            "Transaction from {} to {} for {} sent",
            req.from, req.to, req.amount
        )),
    };

    println!(
        "📤 Transaction sent from {}: {} -> {} ({})",
        state.node_id, req.from, req.to, req.amount
    );

    Ok(HttpResponse::Ok().json(response))
}

/// Get node statistics endpoint
pub async fn get_stats(state: web::Data<SimulationState>) -> Result<HttpResponse> {
    let stats = NodeStats {
        transactions_sent: *state.tx_count.lock().await,
        transactions_received: *state.rx_count.lock().await,
        timestamp: chrono::Utc::now().to_rfc3339(),
        node_id: state.node_id.clone(),
    };

    Ok(HttpResponse::Ok().json(stats))
}

/// Health check endpoint
pub async fn health_check() -> Result<HttpResponse> {
    Ok(HttpResponse::Ok().json(serde_json::json!({
        "status": "healthy",
        "timestamp": chrono::Utc::now().to_rfc3339()
    })))
}
