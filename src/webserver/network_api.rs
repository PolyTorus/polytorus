//! Network Management API
//!
//! RESTful API endpoints for network health monitoring, peer management,
//! and message queue statistics using Actix-web.

use std::sync::Arc;

use actix_web::{delete, get, post, web, HttpResponse, Result as ActixResult};
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::network::{NetworkCommand, PeerId};

/// Network health response
#[derive(Debug, Serialize, Deserialize)]
pub struct NetworkHealthResponse {
    pub status: String,
    pub total_nodes: usize,
    pub healthy_peers: usize,
    pub degraded_peers: usize,
    pub unhealthy_peers: usize,
    pub average_latency_ms: u64,
    pub network_diameter: usize,
}

/// Peer information response
#[derive(Debug, Serialize, Deserialize)]
pub struct PeerInfoResponse {
    pub peer_id: String,
    pub address: String,
    pub health: String,
    pub last_seen: String,
    pub connection_time: String,
    pub latency_ms: u64,
    pub messages_sent: u64,
    pub messages_received: u64,
    pub bytes_sent: u64,
    pub bytes_received: u64,
}

/// Message queue statistics response
#[derive(Debug, Serialize, Deserialize)]
pub struct MessageQueueStatsResponse {
    pub critical_queue_size: usize,
    pub high_queue_size: usize,
    pub normal_queue_size: usize,
    pub low_queue_size: usize,
    pub total_messages_processed: u64,
    pub total_messages_dropped: u64,
    pub average_processing_time_ms: u64,
    pub bandwidth_usage_mbps: f64,
}

/// Blacklist request
#[derive(Debug, Deserialize)]
pub struct BlacklistRequest {
    pub peer_id: String,
    pub reason: String,
}

/// Network API state
pub struct NetworkApiState {
    pub network_command_tx: mpsc::UnboundedSender<NetworkCommand>,
}

impl NetworkApiState {
    pub fn new(network_command_tx: mpsc::UnboundedSender<NetworkCommand>) -> Self {
        Self { network_command_tx }
    }
}

/// Get network health information
#[get("/api/network/health")]
pub async fn get_network_health(
    state: web::Data<Arc<NetworkApiState>>,
) -> ActixResult<HttpResponse> {
    // Send command to get network health
    if state
        .network_command_tx
        .send(NetworkCommand::GetNetworkHealth)
        .is_err()
    {
        return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Failed to communicate with network node"
        })));
    }

    // For now, return simulated data
    // In a real implementation, you would wait for the response through a channel
    let response = NetworkHealthResponse {
        status: "healthy".to_string(),
        total_nodes: 10,
        healthy_peers: 8,
        degraded_peers: 2,
        unhealthy_peers: 0,
        average_latency_ms: 45,
        network_diameter: 3,
    };

    Ok(HttpResponse::Ok().json(response))
}

/// Get peer information
#[get("/api/network/peer/{peer_id}")]
pub async fn get_peer_info(
    path: web::Path<String>,
    state: web::Data<Arc<NetworkApiState>>,
) -> ActixResult<HttpResponse> {
    let peer_id = path.into_inner();

    // Parse peer ID
    let peer_id_parsed = match uuid::Uuid::parse_str(&peer_id) {
        Ok(id) => PeerId(id),
        Err(_) => {
            return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid peer ID format"
            })));
        }
    };

    // Send command to get peer info
    if state
        .network_command_tx
        .send(NetworkCommand::GetPeerInfo(peer_id_parsed))
        .is_err()
    {
        return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Failed to communicate with network node"
        })));
    }

    // Simulated response
    let response = PeerInfoResponse {
        peer_id: peer_id.clone(),
        address: "192.168.1.100:8080".to_string(),
        health: "healthy".to_string(),
        last_seen: "2024-12-15T10:30:00Z".to_string(),
        connection_time: "2024-12-15T09:00:00Z".to_string(),
        latency_ms: 25,
        messages_sent: 1247,
        messages_received: 1156,
        bytes_sent: 2048576,
        bytes_received: 1875432,
    };

    Ok(HttpResponse::Ok().json(response))
}

/// Get message queue statistics
#[get("/api/network/queue/stats")]
pub async fn get_message_queue_stats(
    state: web::Data<Arc<NetworkApiState>>,
) -> ActixResult<HttpResponse> {
    // Send command to get queue stats
    if state
        .network_command_tx
        .send(NetworkCommand::GetMessageQueueStats)
        .is_err()
    {
        return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Failed to communicate with network node"
        })));
    }

    // Simulated response
    let response = MessageQueueStatsResponse {
        critical_queue_size: 0,
        high_queue_size: 5,
        normal_queue_size: 23,
        low_queue_size: 12,
        total_messages_processed: 1247,
        total_messages_dropped: 3,
        average_processing_time_ms: 2,
        bandwidth_usage_mbps: 1.2,
    };

    Ok(HttpResponse::Ok().json(response))
}

/// Blacklist a peer
#[post("/api/network/blacklist")]
pub async fn blacklist_peer(
    request: web::Json<BlacklistRequest>,
    state: web::Data<Arc<NetworkApiState>>,
) -> ActixResult<HttpResponse> {
    // Parse peer ID
    let peer_id = match uuid::Uuid::parse_str(&request.peer_id) {
        Ok(id) => PeerId(id),
        Err(_) => {
            return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid peer ID format"
            })));
        }
    };

    // Send blacklist command
    if state
        .network_command_tx
        .send(NetworkCommand::BlacklistPeer(
            peer_id,
            request.reason.clone(),
        ))
        .is_err()
    {
        return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Failed to communicate with network node"
        })));
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": format!("Peer {} blacklisted for: {}", request.peer_id, request.reason)
    })))
}

/// Unblacklist a peer
#[delete("/api/network/blacklist/{peer_id}")]
pub async fn unblacklist_peer(
    path: web::Path<String>,
    state: web::Data<Arc<NetworkApiState>>,
) -> ActixResult<HttpResponse> {
    let peer_id = path.into_inner();

    // Parse peer ID
    let peer_id_parsed = match uuid::Uuid::parse_str(&peer_id) {
        Ok(id) => PeerId(id),
        Err(_) => {
            return Ok(HttpResponse::BadRequest().json(serde_json::json!({
                "error": "Invalid peer ID format"
            })));
        }
    };

    // Send unblacklist command
    if state
        .network_command_tx
        .send(NetworkCommand::UnblacklistPeer(peer_id_parsed))
        .is_err()
    {
        return Ok(HttpResponse::InternalServerError().json(serde_json::json!({
            "error": "Failed to communicate with network node"
        })));
    }

    Ok(HttpResponse::Ok().json(serde_json::json!({
        "success": true,
        "message": format!("Peer {} removed from blacklist", peer_id)
    })))
}
