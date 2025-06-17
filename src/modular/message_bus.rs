//! Modular Blockchain Message Bus
//!
//! This module provides a flexible message bus system for communication
//! between different layers of the modular blockchain.

use std::{collections::HashMap, sync::Arc};

use tokio::sync::{broadcast, mpsc, RwLock};

use super::traits::*;
use crate::Result;

/// Message bus for inter-layer communication
pub struct ModularMessageBus {
    /// Broadcast channels for each message type
    channels: Arc<RwLock<HashMap<MessageType, broadcast::Sender<ModularMessage>>>>,
    /// Layer registry
    layer_registry: Arc<RwLock<HashMap<LayerType, LayerInfo>>>,
    /// Event metrics
    metrics: Arc<RwLock<MessageBusMetrics>>,
}

/// Message types for routing
#[derive(Debug, Clone, Hash, Eq, PartialEq)]
pub enum MessageType {
    BlockProposal,
    BlockValidation,
    ExecutionResult,
    SettlementBatch,
    DataAvailability,
    HealthCheck,
    Challenge,
    StateSync,
    Custom(String),
}

/// Modular message wrapper
#[derive(Debug, Clone)]
pub struct ModularMessage {
    pub id: String,
    pub message_type: MessageType,
    pub source_layer: LayerType,
    pub target_layer: Option<LayerType>,
    pub payload: MessagePayload,
    pub priority: MessagePriority,
    pub timestamp: u64,
}

/// Message payload types
#[derive(Debug, Clone)]
pub enum MessagePayload {
    BlockProposal {
        block: Box<crate::blockchain::block::Block>,
        proposer_id: String,
    },
    BlockValidation {
        block_hash: Hash,
        is_valid: bool,
        validator_id: String,
    },
    ExecutionResult {
        result: ExecutionResult,
        execution_time: u64,
    },
    SettlementBatch {
        batch: ExecutionBatch,
        priority: u8,
    },
    DataAvailability {
        hash: Hash,
        size: usize,
        operation: DataOperation,
    },
    HealthCheck {
        metrics: LayerMetrics,
        is_healthy: bool,
    },
    Challenge {
        challenge: SettlementChallenge,
        challenger_id: String,
    },
    StateSync {
        state_root: Hash,
        height: u64,
    },
    Custom {
        data: Vec<u8>,
        metadata: HashMap<String, String>,
    },
}

/// Data operation types
#[derive(Debug, Clone)]
pub enum DataOperation {
    Store,
    Retrieve,
    Verify,
}

/// Message priority levels
#[derive(
    Debug, Clone, Ord, PartialOrd, Eq, PartialEq, Hash, serde::Serialize, serde::Deserialize,
)]
pub enum MessagePriority {
    Critical = 0,
    High = 1,
    Normal = 2,
    Low = 3,
}

/// Layer information for registry
#[derive(Debug, Clone)]
pub struct LayerInfo {
    pub layer_type: LayerType,
    pub layer_id: String,
    pub capabilities: Vec<String>,
    pub health_status: HealthStatus,
    pub message_handler: Option<mpsc::UnboundedSender<ModularMessage>>,
}

/// Health status of a layer
#[derive(Debug, Clone)]
pub enum HealthStatus {
    Healthy,
    Degraded,
    Unhealthy,
    Unknown,
}

/// Message bus metrics
#[derive(Debug, Clone, Default)]
pub struct MessageBusMetrics {
    pub total_messages: u64,
    pub messages_by_type: HashMap<MessageType, u64>,
    pub messages_by_priority: HashMap<MessagePriority, u64>,
    pub average_latency: f64,
    pub error_count: u64,
}

/// Layer type enumeration (extended)
#[derive(Debug, Clone, Hash, Eq, PartialEq, serde::Serialize, serde::Deserialize)]
pub enum LayerType {
    Execution,
    Settlement,
    Consensus,
    DataAvailability,
    Network,
    Storage,
    Monitoring,
    Custom(String),
}

/// Layer performance metrics (extended)
#[derive(Debug, Clone)]
pub struct LayerMetrics {
    pub throughput: f64,
    pub latency: u64,
    pub error_rate: f64,
    pub resource_usage: f64,
    pub queue_depth: usize,
    pub connections: usize,
}

impl ModularMessageBus {
    /// Create a new message bus
    pub fn new() -> Self {
        Self {
            channels: Arc::new(RwLock::new(HashMap::new())),
            layer_registry: Arc::new(RwLock::new(HashMap::new())),
            metrics: Arc::new(RwLock::new(MessageBusMetrics::default())),
        }
    }

    /// Register a layer with the message bus
    pub async fn register_layer(&self, layer_info: LayerInfo) -> Result<()> {
        let layer_type = layer_info.layer_type.clone();
        let mut registry = self.layer_registry.write().await;
        registry.insert(layer_info.layer_type.clone(), layer_info);
        log::info!("Layer registered with message bus: {:?}", layer_type);
        Ok(())
    }

    /// Create a broadcast channel for a message type
    pub async fn create_channel(
        &self,
        message_type: MessageType,
    ) -> Result<broadcast::Receiver<ModularMessage>> {
        let mut channels = self.channels.write().await;

        if let Some(sender) = channels.get(&message_type) {
            Ok(sender.subscribe())
        } else {
            let (sender, receiver) = broadcast::channel(1000); // Buffer size
            channels.insert(message_type, sender);
            Ok(receiver)
        }
    }

    /// Publish a message to the bus
    pub async fn publish(&self, message: ModularMessage) -> Result<()> {
        let start_time = std::time::Instant::now();

        // Update metrics
        {
            let mut metrics = self.metrics.write().await;
            metrics.total_messages += 1;
            *metrics
                .messages_by_type
                .entry(message.message_type.clone())
                .or_insert(0) += 1;
            *metrics
                .messages_by_priority
                .entry(message.priority.clone())
                .or_insert(0) += 1;
        }

        // Send to appropriate channel
        let channels = self.channels.read().await;
        if let Some(sender) = channels.get(&message.message_type) {
            if let Err(e) = sender.send(message.clone()) {
                log::warn!("Failed to send message: {}", e);
                let mut metrics = self.metrics.write().await;
                metrics.error_count += 1;
                return Err(anyhow::anyhow!("Message send failed: {}", e));
            }
        } else {
            log::warn!(
                "No channel found for message type: {:?}",
                message.message_type
            );
            return Err(anyhow::anyhow!("No channel for message type"));
        }

        // Update latency metrics
        let latency = start_time.elapsed().as_millis() as f64;
        {
            let mut metrics = self.metrics.write().await;
            metrics.average_latency = (metrics.average_latency + latency) / 2.0;
        }

        log::trace!(
            "Published message: {} (type: {:?}, priority: {:?})",
            message.id,
            message.message_type,
            message.priority
        );
        Ok(())
    }

    /// Subscribe to messages of a specific type
    pub async fn subscribe(
        &self,
        message_type: MessageType,
    ) -> Result<broadcast::Receiver<ModularMessage>> {
        self.create_channel(message_type).await
    }

    /// Get layer information
    pub async fn get_layer_info(&self, layer_type: &LayerType) -> Option<LayerInfo> {
        let registry = self.layer_registry.read().await;
        registry.get(layer_type).cloned()
    }

    /// Update layer health status
    pub async fn update_layer_health(
        &self,
        layer_type: LayerType,
        health_status: HealthStatus,
    ) -> Result<()> {
        let mut registry = self.layer_registry.write().await;
        if let Some(layer_info) = registry.get_mut(&layer_type) {
            layer_info.health_status = health_status;
            log::debug!("Updated health status for layer {:?}", layer_type);
        }
        Ok(())
    }

    /// Get message bus metrics
    pub async fn get_metrics(&self) -> MessageBusMetrics {
        let metrics = self.metrics.read().await;
        metrics.clone()
    }

    /// Get all registered layers
    pub async fn get_registered_layers(&self) -> Vec<LayerInfo> {
        let registry = self.layer_registry.read().await;
        registry.values().cloned().collect()
    }

    /// Broadcast health check request
    pub async fn broadcast_health_check(&self) -> Result<()> {
        let message = ModularMessage {
            id: uuid::Uuid::new_v4().to_string(),
            message_type: MessageType::HealthCheck,
            source_layer: LayerType::Monitoring,
            target_layer: None, // Broadcast to all
            payload: MessagePayload::HealthCheck {
                metrics: LayerMetrics {
                    throughput: 0.0,
                    latency: 0,
                    error_rate: 0.0,
                    resource_usage: 0.0,
                    queue_depth: 0,
                    connections: 0,
                },
                is_healthy: true,
            },
            priority: MessagePriority::Normal,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        };

        self.publish(message).await
    }
}

impl Default for ModularMessageBus {
    fn default() -> Self {
        Self::new()
    }
}

/// Message builder for convenience
pub struct MessageBuilder {
    message_type: Option<MessageType>,
    source_layer: Option<LayerType>,
    target_layer: Option<LayerType>,
    payload: Option<MessagePayload>,
    priority: MessagePriority,
}

impl MessageBuilder {
    pub fn new() -> Self {
        Self {
            message_type: None,
            source_layer: None,
            target_layer: None,
            payload: None,
            priority: MessagePriority::Normal,
        }
    }

    pub fn message_type(mut self, message_type: MessageType) -> Self {
        self.message_type = Some(message_type);
        self
    }

    pub fn source_layer(mut self, layer: LayerType) -> Self {
        self.source_layer = Some(layer);
        self
    }

    pub fn target_layer(mut self, layer: LayerType) -> Self {
        self.target_layer = Some(layer);
        self
    }

    pub fn payload(mut self, payload: MessagePayload) -> Self {
        self.payload = Some(payload);
        self
    }

    pub fn priority(mut self, priority: MessagePriority) -> Self {
        self.priority = priority;
        self
    }

    pub fn build(self) -> Result<ModularMessage> {
        Ok(ModularMessage {
            id: uuid::Uuid::new_v4().to_string(),
            message_type: self
                .message_type
                .ok_or_else(|| anyhow::anyhow!("Message type is required"))?,
            source_layer: self
                .source_layer
                .ok_or_else(|| anyhow::anyhow!("Source layer is required"))?,
            target_layer: self.target_layer,
            payload: self
                .payload
                .ok_or_else(|| anyhow::anyhow!("Payload is required"))?,
            priority: self.priority,
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        })
    }
}

impl Default for MessageBuilder {
    fn default() -> Self {
        Self::new()
    }
}

/// Simple message bus for layer communication
pub struct MessageBus {
    sender: broadcast::Sender<MessageBusMessage>,
}

impl Default for MessageBus {
    fn default() -> Self {
        Self::new()
    }
}

impl MessageBus {
    pub fn new() -> Self {
        let (sender, _) = broadcast::channel(1000);
        Self { sender }
    }

    pub async fn send(&self, message: MessageBusMessage) -> Result<()> {
        let _ = self.sender.send(message);
        Ok(())
    }

    pub fn subscribe(&self) -> broadcast::Receiver<MessageBusMessage> {
        self.sender.subscribe()
    }
}

/// Simple message structure for layer communication
#[derive(Debug, Clone)]
pub struct MessageBusMessage {
    pub layer_type: String,
    pub message: serde_json::Value,
    pub timestamp: std::time::SystemTime,
}
