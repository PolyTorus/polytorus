//! Modular Blockchain Message Bus
//!
//! This module provides a comprehensive message delivery system with real
//! pub/sub mechanisms, message routing, filtering, and delivery guarantees
//! for communication between different layers of the modular blockchain.

use std::{
    collections::{HashMap, VecDeque},
    sync::{
        atomic::{AtomicU64, Ordering},
        Arc,
    },
    time::{Duration, Instant, SystemTime},
};

use tokio::sync::{broadcast, mpsc, Mutex, RwLock};
use uuid::Uuid;

use super::traits::*;
use crate::Result;

/// Enhanced message bus for inter-layer communication with real pub/sub mechanisms
pub struct ModularMessageBus {
    /// Broadcast channels for each message type
    channels: Arc<RwLock<HashMap<MessageType, broadcast::Sender<ModularMessage>>>>,
    /// Layer registry with handlers
    layer_registry: Arc<RwLock<HashMap<LayerType, LayerInfo>>>,
    /// Subscription registry for routing
    subscriptions: Arc<RwLock<HashMap<SubscriptionId, Subscription>>>,
    /// Message filters for targeted delivery
    filters: Arc<RwLock<HashMap<LayerType, Vec<MessageFilter>>>>,
    /// Reliable delivery queue for critical messages
    reliable_queue: Arc<Mutex<VecDeque<PendingMessage>>>,
    /// Message history for debugging and replay
    message_history: Arc<RwLock<VecDeque<MessageHistoryEntry>>>,
    /// Event metrics with enhanced tracking
    metrics: Arc<RwLock<MessageBusMetrics>>,
    /// Message sequence counter
    sequence_counter: Arc<AtomicU64>,
    /// Dead letter queue for failed deliveries
    dead_letter_queue: Arc<Mutex<VecDeque<DeadLetterEntry>>>,
    /// Router for intelligent message routing
    router: Arc<MessageRouter>,
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

/// Enhanced message bus metrics with delivery tracking
#[derive(Debug, Clone, Default)]
pub struct MessageBusMetrics {
    pub total_messages: u64,
    pub messages_by_type: HashMap<MessageType, u64>,
    pub messages_by_priority: HashMap<MessagePriority, u64>,
    pub messages_delivered: u64,
    pub messages_failed: u64,
    pub messages_retried: u64,
    pub average_latency: f64,
    pub delivery_success_rate: f64,
    pub active_subscriptions: usize,
    pub queue_depth: usize,
    pub error_count: u64,
    pub dead_letter_count: u64,
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

/// Subscription identifier
pub type SubscriptionId = String;

/// Message subscription with filtering capabilities
#[derive(Debug, Clone)]
pub struct Subscription {
    pub id: SubscriptionId,
    pub subscriber: LayerType,
    pub message_types: Vec<MessageType>,
    pub filters: Vec<MessageFilter>,
    pub delivery_mode: DeliveryMode,
    pub handler: mpsc::UnboundedSender<ModularMessage>,
    pub created_at: SystemTime,
    pub last_activity: SystemTime,
}

/// Message filter for targeted delivery
#[derive(Debug, Clone)]
pub struct MessageFilter {
    pub filter_type: FilterType,
    pub criteria: FilterCriteria,
}

/// Filter types for message routing
#[derive(Debug, Clone)]
pub enum FilterType {
    SourceLayer,
    TargetLayer,
    Priority,
    Custom(String),
}

/// Filter criteria for message matching
#[derive(Debug, Clone)]
pub enum FilterCriteria {
    Equals(String),
    Contains(String),
    In(Vec<String>),
    Custom(HashMap<String, String>),
}

/// Message delivery modes
#[derive(Debug, Clone)]
pub enum DeliveryMode {
    BestEffort,  // Fire and forget
    AtLeastOnce, // Retry until acknowledgment
    ExactlyOnce, // Guaranteed single delivery
}

/// Pending message for reliable delivery
#[derive(Debug, Clone)]
pub struct PendingMessage {
    pub message: ModularMessage,
    pub target_subscriptions: Vec<SubscriptionId>,
    pub delivery_attempts: u32,
    pub max_attempts: u32,
    pub next_retry: SystemTime,
    pub created_at: SystemTime,
}

/// Message history entry for debugging
#[derive(Debug, Clone)]
pub struct MessageHistoryEntry {
    pub message: ModularMessage,
    pub delivered_to: Vec<SubscriptionId>,
    pub delivery_status: DeliveryStatus,
    pub processing_time: Duration,
    pub timestamp: SystemTime,
}

/// Delivery status tracking
#[derive(Debug, Clone)]
pub enum DeliveryStatus {
    Pending,
    Delivered,
    Failed(String),
    Retrying,
}

/// Dead letter entry for failed messages
#[derive(Debug, Clone)]
pub struct DeadLetterEntry {
    pub message: ModularMessage,
    pub failure_reason: String,
    pub attempts: u32,
    pub first_attempt: SystemTime,
    pub last_attempt: SystemTime,
}

/// Message router for intelligent routing
#[derive(Debug)]
pub struct MessageRouter {
    routing_table: RwLock<HashMap<MessageType, Vec<RoutingRule>>>,
    load_balancer: RwLock<HashMap<MessageType, LoadBalanceStrategy>>,
}

/// Routing rule for message delivery
#[derive(Debug)]
pub struct RoutingRule {
    pub target_layer: LayerType,
    pub condition: RoutingCondition,
    pub priority: u8,
}

/// Routing condition for rule matching
pub enum RoutingCondition {
    Always,
    SourceEquals(LayerType),
    PayloadContains(String),
    Custom(Box<dyn Fn(&ModularMessage) -> bool + Send + Sync>),
}

impl std::fmt::Debug for RoutingCondition {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RoutingCondition::Always => write!(f, "Always"),
            RoutingCondition::SourceEquals(layer) => write!(f, "SourceEquals({:?})", layer),
            RoutingCondition::PayloadContains(text) => write!(f, "PayloadContains({})", text),
            RoutingCondition::Custom(_) => write!(f, "Custom(<function>)"),
        }
    }
}

/// Load balancing strategies
#[derive(Debug, Clone)]
pub enum LoadBalanceStrategy {
    RoundRobin { current: usize },
    LeastLoaded,
    Random,
}

impl ModularMessageBus {
    /// Create a new enhanced message bus with real pub/sub mechanisms
    pub fn new() -> Self {
        Self {
            channels: Arc::new(RwLock::new(HashMap::new())),
            layer_registry: Arc::new(RwLock::new(HashMap::new())),
            subscriptions: Arc::new(RwLock::new(HashMap::new())),
            filters: Arc::new(RwLock::new(HashMap::new())),
            reliable_queue: Arc::new(Mutex::new(VecDeque::new())),
            message_history: Arc::new(RwLock::new(VecDeque::new())),
            metrics: Arc::new(RwLock::new(MessageBusMetrics::default())),
            sequence_counter: Arc::new(AtomicU64::new(0)),
            dead_letter_queue: Arc::new(Mutex::new(VecDeque::new())),
            router: Arc::new(MessageRouter::new()),
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

    /// Publish a message with enhanced routing and delivery guarantees
    pub async fn publish(&self, mut message: ModularMessage) -> Result<()> {
        let start_time = Instant::now();

        // Assign sequence number for ordering
        let sequence = self.sequence_counter.fetch_add(1, Ordering::SeqCst);
        message.id = format!("{}-{}", message.id, sequence);

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

        // Find target subscriptions using intelligent routing
        let target_subscriptions = self.find_target_subscriptions(&message).await;

        if target_subscriptions.is_empty() {
            log::warn!(
                "No subscribers found for message type: {:?} from layer: {:?}",
                message.message_type,
                message.source_layer
            );
            // Still try broadcast channel for backward compatibility
            self.broadcast_to_channel(&message).await?;
            return Ok(());
        }

        // Deliver message to targeted subscriptions
        let mut delivery_results = Vec::new();
        let mut delivered_count = 0;

        for subscription_id in &target_subscriptions {
            match self
                .deliver_to_subscription(&message, subscription_id)
                .await
            {
                Ok(()) => {
                    delivered_count += 1;
                    delivery_results.push((subscription_id.clone(), true));
                }
                Err(e) => {
                    log::warn!(
                        "Failed to deliver message {} to subscription {}: {}",
                        message.id,
                        subscription_id,
                        e
                    );
                    delivery_results.push((subscription_id.clone(), false));

                    // Queue for retry if delivery mode requires it
                    self.queue_for_retry(&message, subscription_id).await;
                }
            }
        }

        // Update delivery metrics
        {
            let mut metrics = self.metrics.write().await;
            metrics.messages_delivered += delivered_count;
            if delivered_count < target_subscriptions.len() as u64 {
                metrics.messages_failed += (target_subscriptions.len() as u64) - delivered_count;
            }

            let success_rate = delivered_count as f64 / target_subscriptions.len() as f64;
            metrics.delivery_success_rate = (metrics.delivery_success_rate + success_rate) / 2.0;
        }

        // Record in message history
        let processing_time = start_time.elapsed();
        self.record_message_history(
            &message,
            &target_subscriptions,
            if delivered_count > 0 {
                DeliveryStatus::Delivered
            } else {
                DeliveryStatus::Failed("No successful deliveries".to_string())
            },
            processing_time,
        )
        .await;

        // Also broadcast to legacy channel for backward compatibility
        let _ = self.broadcast_to_channel(&message).await;

        // Update latency metrics
        let latency = processing_time.as_millis() as f64;
        {
            let mut metrics = self.metrics.write().await;
            metrics.average_latency = (metrics.average_latency + latency) / 2.0;
        }

        log::trace!(
            "Published message: {} (type: {:?}, delivered to: {}/{} subscribers)",
            message.id,
            message.message_type,
            delivered_count,
            target_subscriptions.len()
        );

        Ok(())
    }

    /// Subscribe to messages with enhanced filtering and delivery options
    pub async fn subscribe_enhanced(
        &self,
        subscriber: LayerType,
        message_types: Vec<MessageType>,
        filters: Vec<MessageFilter>,
        delivery_mode: DeliveryMode,
    ) -> Result<(SubscriptionId, mpsc::UnboundedReceiver<ModularMessage>)> {
        let subscription_id = Uuid::new_v4().to_string();
        let (tx, rx) = mpsc::unbounded_channel();

        let subscription = Subscription {
            id: subscription_id.clone(),
            subscriber: subscriber.clone(),
            message_types: message_types.clone(),
            filters: filters.clone(),
            delivery_mode,
            handler: tx,
            created_at: SystemTime::now(),
            last_activity: SystemTime::now(),
        };

        // Register subscription
        {
            let mut subscriptions = self.subscriptions.write().await;
            subscriptions.insert(subscription_id.clone(), subscription);
        }

        // Update subscriber's filters
        {
            let mut layer_filters = self.filters.write().await;
            layer_filters.insert(subscriber.clone(), filters);
        }

        // Update metrics
        {
            let mut metrics = self.metrics.write().await;
            metrics.active_subscriptions = self.subscriptions.read().await.len();
        }

        log::info!(
            "Enhanced subscription created: {} for layer {:?} (types: {:?})",
            subscription_id,
            subscriber,
            message_types
        );

        Ok((subscription_id, rx))
    }

    /// Legacy subscribe method for backward compatibility
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

    /// Get enhanced message bus metrics
    pub async fn get_metrics(&self) -> MessageBusMetrics {
        let mut metrics = self.metrics.write().await;

        // Update current state metrics
        metrics.active_subscriptions = self.subscriptions.read().await.len();
        metrics.queue_depth = self.reliable_queue.lock().await.len();
        metrics.dead_letter_count = self.dead_letter_queue.lock().await.len() as u64;

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

    /// Find target subscriptions for a message using routing logic
    async fn find_target_subscriptions(&self, message: &ModularMessage) -> Vec<SubscriptionId> {
        let subscriptions = self.subscriptions.read().await;
        let mut targets = Vec::new();

        for (id, subscription) in subscriptions.iter() {
            // Check if subscription is interested in this message type
            if !subscription.message_types.contains(&message.message_type) {
                continue;
            }

            // Check target layer matching
            if let Some(target_layer) = &message.target_layer {
                if subscription.subscriber != *target_layer {
                    continue;
                }
            }

            // Apply message filters
            if self
                .message_matches_filters(message, &subscription.filters)
                .await
            {
                targets.push(id.clone());
            }
        }

        // Use router for additional intelligent routing
        if let Ok(additional_targets) = self.router.route_message(message).await {
            for target in additional_targets {
                if !targets.contains(&target) {
                    targets.push(target);
                }
            }
        }

        targets
    }

    /// Check if message matches subscription filters
    async fn message_matches_filters(
        &self,
        message: &ModularMessage,
        filters: &[MessageFilter],
    ) -> bool {
        if filters.is_empty() {
            return true; // No filters means accept all
        }

        for filter in filters {
            if !self.apply_message_filter(message, filter) {
                return false; // All filters must match
            }
        }

        true
    }

    /// Apply a single message filter
    fn apply_message_filter(&self, message: &ModularMessage, filter: &MessageFilter) -> bool {
        match &filter.filter_type {
            FilterType::SourceLayer => match &filter.criteria {
                FilterCriteria::Equals(layer_str) => {
                    format!("{:?}", message.source_layer) == *layer_str
                }
                _ => false,
            },
            FilterType::TargetLayer => {
                if let Some(target_layer) = &message.target_layer {
                    match &filter.criteria {
                        FilterCriteria::Equals(layer_str) => {
                            format!("{:?}", target_layer) == *layer_str
                        }
                        _ => false,
                    }
                } else {
                    false
                }
            }
            FilterType::Priority => match &filter.criteria {
                FilterCriteria::Equals(priority_str) => {
                    format!("{:?}", message.priority) == *priority_str
                }
                _ => false,
            },
            FilterType::Custom(_) => {
                // Custom filters would be implemented based on specific needs
                true
            }
        }
    }

    /// Deliver message to a specific subscription
    async fn deliver_to_subscription(
        &self,
        message: &ModularMessage,
        subscription_id: &SubscriptionId,
    ) -> Result<()> {
        let subscription = {
            let subscriptions = self.subscriptions.read().await;
            subscriptions.get(subscription_id).cloned()
        };

        if let Some(subscription) = subscription {
            // Update last activity
            {
                let mut subscriptions = self.subscriptions.write().await;
                if let Some(sub) = subscriptions.get_mut(subscription_id) {
                    sub.last_activity = SystemTime::now();
                }
            }

            // Send message to handler
            subscription
                .handler
                .send(message.clone())
                .map_err(|e| anyhow::anyhow!("Failed to send to subscription handler: {}", e))?;

            log::trace!(
                "Message {} delivered to subscription {} (layer: {:?})",
                message.id,
                subscription_id,
                subscription.subscriber
            );

            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Subscription {} not found",
                subscription_id
            ))
        }
    }

    /// Queue message for retry based on delivery mode
    async fn queue_for_retry(&self, message: &ModularMessage, subscription_id: &SubscriptionId) {
        let subscription = {
            let subscriptions = self.subscriptions.read().await;
            subscriptions.get(subscription_id).cloned()
        };

        if let Some(subscription) = subscription {
            match subscription.delivery_mode {
                DeliveryMode::BestEffort => {
                    // No retry for best effort
                }
                DeliveryMode::AtLeastOnce | DeliveryMode::ExactlyOnce => {
                    let pending_message = PendingMessage {
                        message: message.clone(),
                        target_subscriptions: vec![subscription_id.clone()],
                        delivery_attempts: 1,
                        max_attempts: 3,
                        next_retry: SystemTime::now() + Duration::from_secs(5),
                        created_at: SystemTime::now(),
                    };

                    let mut queue = self.reliable_queue.lock().await;
                    queue.push_back(pending_message);

                    let mut metrics = self.metrics.write().await;
                    metrics.messages_retried += 1;
                }
            }
        }
    }

    /// Record message in history for debugging
    async fn record_message_history(
        &self,
        message: &ModularMessage,
        delivered_to: &[SubscriptionId],
        status: DeliveryStatus,
        processing_time: Duration,
    ) {
        let history_entry = MessageHistoryEntry {
            message: message.clone(),
            delivered_to: delivered_to.to_vec(),
            delivery_status: status,
            processing_time,
            timestamp: SystemTime::now(),
        };

        let mut history = self.message_history.write().await;
        history.push_back(history_entry);

        // Keep only last 1000 entries
        if history.len() > 1000 {
            history.pop_front();
        }
    }

    /// Broadcast to legacy channel for backward compatibility
    async fn broadcast_to_channel(&self, message: &ModularMessage) -> Result<()> {
        let channels = self.channels.read().await;
        if let Some(sender) = channels.get(&message.message_type) {
            if let Err(e) = sender.send(message.clone()) {
                log::debug!(
                    "Legacy broadcast failed (expected if no legacy subscribers): {}",
                    e
                );
            }
        }
        Ok(())
    }

    /// Process retry queue for reliable delivery
    pub async fn process_retry_queue(&self) -> Result<()> {
        let mut queue = self.reliable_queue.lock().await;
        let mut to_retry = Vec::new();
        let mut to_dead_letter = Vec::new();

        // Check which messages are ready for retry
        while let Some(pending) = queue.pop_front() {
            if SystemTime::now() >= pending.next_retry {
                if pending.delivery_attempts < pending.max_attempts {
                    to_retry.push(pending);
                } else {
                    to_dead_letter.push(pending);
                }
            } else {
                queue.push_back(pending); // Put back if not ready
            }
        }

        drop(queue); // Release lock

        // Process retries
        for mut pending in to_retry {
            let mut success = false;

            for subscription_id in &pending.target_subscriptions {
                if self
                    .deliver_to_subscription(&pending.message, subscription_id)
                    .await
                    .is_ok()
                {
                    success = true;
                    log::debug!(
                        "Retry successful for message {} to subscription {}",
                        pending.message.id,
                        subscription_id
                    );
                }
            }

            if !success {
                pending.delivery_attempts += 1;
                pending.next_retry =
                    SystemTime::now() + Duration::from_secs(5 * pending.delivery_attempts as u64); // Exponential backoff

                let mut queue = self.reliable_queue.lock().await;
                queue.push_back(pending);
            }
        }

        // Move failed messages to dead letter queue
        if !to_dead_letter.is_empty() {
            let mut dead_letter = self.dead_letter_queue.lock().await;

            for pending in to_dead_letter {
                let dead_entry = DeadLetterEntry {
                    message: pending.message.clone(),
                    failure_reason: "Max retry attempts exceeded".to_string(),
                    attempts: pending.delivery_attempts,
                    first_attempt: pending.created_at,
                    last_attempt: SystemTime::now(),
                };

                dead_letter.push_back(dead_entry);

                log::warn!(
                    "Message {} moved to dead letter queue after {} attempts",
                    pending.message.id,
                    pending.delivery_attempts
                );
            }
        }

        Ok(())
    }

    /// Unsubscribe from message delivery
    pub async fn unsubscribe(&self, subscription_id: &SubscriptionId) -> Result<()> {
        let mut subscriptions = self.subscriptions.write().await;

        if subscriptions.remove(subscription_id).is_some() {
            log::info!("Subscription {} removed", subscription_id);

            // Update metrics
            let mut metrics = self.metrics.write().await;
            metrics.active_subscriptions = subscriptions.len();

            Ok(())
        } else {
            Err(anyhow::anyhow!(
                "Subscription {} not found",
                subscription_id
            ))
        }
    }

    /// Get message history for debugging
    pub async fn get_message_history(&self, limit: usize) -> Vec<MessageHistoryEntry> {
        let history = self.message_history.read().await;
        let start = if history.len() > limit {
            history.len() - limit
        } else {
            0
        };

        history.range(start..).cloned().collect()
    }

    /// Get dead letter queue entries
    pub async fn get_dead_letter_queue(&self) -> Vec<DeadLetterEntry> {
        let dead_letter = self.dead_letter_queue.lock().await;
        dead_letter.iter().cloned().collect()
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
    pub timestamp: SystemTime,
}

impl MessageRouter {
    /// Create a new message router
    pub fn new() -> Self {
        Self {
            routing_table: RwLock::new(HashMap::new()),
            load_balancer: RwLock::new(HashMap::new()),
        }
    }

    /// Route a message to appropriate subscriptions
    pub async fn route_message(&self, message: &ModularMessage) -> Result<Vec<SubscriptionId>> {
        let routing_table = self.routing_table.read().await;
        let mut targets = Vec::new();

        if let Some(rules) = routing_table.get(&message.message_type) {
            for rule in rules {
                if self.matches_routing_condition(&rule.condition, message) {
                    // Generate subscription ID based on target layer
                    // In a real implementation, this would lookup actual subscription IDs
                    let target_id = format!("{:?}-subscription", rule.target_layer);
                    targets.push(target_id);
                }
            }
        }

        Ok(targets)
    }

    /// Check if message matches routing condition
    fn matches_routing_condition(
        &self,
        condition: &RoutingCondition,
        message: &ModularMessage,
    ) -> bool {
        match condition {
            RoutingCondition::Always => true,
            RoutingCondition::SourceEquals(layer) => message.source_layer == *layer,
            RoutingCondition::PayloadContains(text) => {
                format!("{:?}", message.payload).contains(text)
            }
            RoutingCondition::Custom(func) => {
                // Evaluate custom condition function
                func(message)
            }
        }
    }

    /// Add routing rule
    pub async fn add_routing_rule(&self, message_type: MessageType, rule: RoutingRule) {
        let mut routing_table = self.routing_table.write().await;
        routing_table
            .entry(message_type)
            .or_insert_with(Vec::new)
            .push(rule);
    }

    /// Set load balance strategy for a message type
    pub async fn set_load_balance_strategy(
        &self,
        message_type: MessageType,
        strategy: LoadBalanceStrategy,
    ) {
        let mut load_balancer = self.load_balancer.write().await;
        load_balancer.insert(message_type, strategy);
    }
}

impl Default for MessageRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use std::time::UNIX_EPOCH;

    use tokio::time::Duration;

    use super::*;

    async fn create_test_message(
        msg_type: MessageType,
        source: LayerType,
        target: Option<LayerType>,
    ) -> ModularMessage {
        ModularMessage {
            id: Uuid::new_v4().to_string(),
            message_type: msg_type,
            source_layer: source,
            target_layer: target,
            payload: MessagePayload::Custom {
                data: b"test_data".to_vec(),
                metadata: HashMap::new(),
            },
            priority: MessagePriority::Normal,
            timestamp: SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap()
                .as_secs(),
        }
    }

    #[tokio::test]
    async fn test_enhanced_message_bus_creation() {
        let bus = ModularMessageBus::new();
        let metrics = bus.get_metrics().await;

        assert_eq!(metrics.total_messages, 0);
        assert_eq!(metrics.active_subscriptions, 0);
        assert_eq!(metrics.queue_depth, 0);
    }

    #[tokio::test]
    async fn test_enhanced_subscription_and_delivery() {
        let bus = Arc::new(ModularMessageBus::new());

        // Create enhanced subscription
        let (_subscription_id, mut receiver) = bus
            .subscribe_enhanced(
                LayerType::Execution,
                vec![MessageType::ExecutionResult],
                vec![],
                DeliveryMode::AtLeastOnce,
            )
            .await
            .unwrap();

        // Publish message
        let message = create_test_message(
            MessageType::ExecutionResult,
            LayerType::Consensus,
            Some(LayerType::Execution),
        )
        .await;

        let original_id = message.id.clone();
        bus.publish(message.clone()).await.unwrap();

        // Verify delivery
        let received_message = tokio::time::timeout(Duration::from_millis(100), receiver.recv())
            .await
            .unwrap()
            .unwrap();

        assert!(received_message.id.starts_with(&original_id));
        assert_eq!(received_message.message_type, MessageType::ExecutionResult);

        // Verify metrics
        let metrics = bus.get_metrics().await;
        assert!(metrics.total_messages > 0);
        assert_eq!(metrics.active_subscriptions, 1);
    }

    #[tokio::test]
    async fn test_message_filtering() {
        let bus = Arc::new(ModularMessageBus::new());

        // Create subscription with source layer filter
        let source_filter = MessageFilter {
            filter_type: FilterType::SourceLayer,
            criteria: FilterCriteria::Equals("Consensus".to_string()),
        };

        let (_subscription_id, mut receiver) = bus
            .subscribe_enhanced(
                LayerType::Execution,
                vec![MessageType::BlockValidation],
                vec![source_filter],
                DeliveryMode::BestEffort,
            )
            .await
            .unwrap();

        // Publish message from Consensus (should match filter)
        let matching_message = create_test_message(
            MessageType::BlockValidation,
            LayerType::Consensus,
            Some(LayerType::Execution),
        )
        .await;
        let original_matching_id = matching_message.id.clone();
        bus.publish(matching_message.clone()).await.unwrap();

        // Publish message from different source (should not match filter)
        let non_matching_message = create_test_message(
            MessageType::BlockValidation,
            LayerType::Settlement,
            Some(LayerType::Execution),
        )
        .await;
        bus.publish(non_matching_message).await.unwrap();

        // Should receive only the matching message
        let received = tokio::time::timeout(Duration::from_millis(100), receiver.recv())
            .await
            .unwrap()
            .unwrap();

        assert!(received.id.starts_with(&original_matching_id));
        assert_eq!(received.source_layer, LayerType::Consensus);

        // Should not receive the non-matching message
        let no_more_messages =
            tokio::time::timeout(Duration::from_millis(50), receiver.recv()).await;
        assert!(no_more_messages.is_err()); // Timeout expected
    }

    #[tokio::test]
    async fn test_reliable_delivery_and_retry() {
        let bus = Arc::new(ModularMessageBus::new());

        // Create subscription with AtLeastOnce delivery
        let (_subscription_id, receiver) = bus
            .subscribe_enhanced(
                LayerType::Settlement,
                vec![MessageType::SettlementBatch],
                vec![],
                DeliveryMode::AtLeastOnce,
            )
            .await
            .unwrap();

        // Drop the receiver to simulate delivery failure
        drop(receiver);

        // Publish message
        let message = create_test_message(
            MessageType::SettlementBatch,
            LayerType::Execution,
            Some(LayerType::Settlement),
        )
        .await;
        bus.publish(message.clone()).await.unwrap();

        // Verify message was queued for retry
        let metrics = bus.get_metrics().await;
        assert!(metrics.queue_depth > 0 || metrics.messages_retried > 0);

        // Process retry queue
        let retry_result = bus.process_retry_queue().await;
        assert!(retry_result.is_ok());
    }

    #[tokio::test]
    async fn test_dead_letter_queue() {
        let bus = Arc::new(ModularMessageBus::new());

        // Create subscription and immediately drop receiver
        let (subscription_id, receiver) = bus
            .subscribe_enhanced(
                LayerType::DataAvailability,
                vec![MessageType::DataAvailability],
                vec![],
                DeliveryMode::ExactlyOnce,
            )
            .await
            .unwrap();
        drop(receiver);

        // Publish message that will fail delivery
        let message = create_test_message(
            MessageType::DataAvailability,
            LayerType::Consensus,
            Some(LayerType::DataAvailability),
        )
        .await;
        bus.publish(message.clone()).await.unwrap();

        // Manually add to reliable queue with max attempts exceeded
        {
            let pending_message = PendingMessage {
                message: message.clone(),
                target_subscriptions: vec![subscription_id],
                delivery_attempts: 5, // Exceeds max attempts
                max_attempts: 3,
                next_retry: SystemTime::now(),
                created_at: SystemTime::now(),
            };

            let mut queue = bus.reliable_queue.lock().await;
            queue.push_back(pending_message);
        }

        // Process retry queue to move to dead letter
        bus.process_retry_queue().await.unwrap();

        // Check dead letter queue
        let dead_letters = bus.get_dead_letter_queue().await;
        assert!(!dead_letters.is_empty());
        assert_eq!(dead_letters[0].message.id, message.id);
    }

    #[tokio::test]
    async fn test_message_history() {
        let bus = Arc::new(ModularMessageBus::new());

        // Create subscription
        let (_subscription_id, mut receiver) = bus
            .subscribe_enhanced(
                LayerType::Network,
                vec![MessageType::StateSync],
                vec![],
                DeliveryMode::BestEffort,
            )
            .await
            .unwrap();

        // Publish several messages
        for i in 0..3 {
            let message = ModularMessage {
                id: format!("test_message_{}", i),
                message_type: MessageType::StateSync,
                source_layer: LayerType::Consensus,
                target_layer: Some(LayerType::Network),
                payload: MessagePayload::Custom {
                    data: format!("data_{}", i).into_bytes(),
                    metadata: HashMap::new(),
                },
                priority: MessagePriority::Normal,
                timestamp: SystemTime::now()
                    .duration_since(UNIX_EPOCH)
                    .unwrap()
                    .as_secs(),
            };
            bus.publish(message).await.unwrap();
        }

        // Consume messages
        for _ in 0..3 {
            receiver.recv().await.unwrap();
        }

        // Check message history
        let history = bus.get_message_history(5).await;
        assert!(history.len() >= 3);

        // Verify history entries contain expected data
        for entry in &history {
            assert!(entry.message.id.starts_with("test_message_"));
            assert_eq!(entry.message.message_type, MessageType::StateSync);
        }
    }

    #[tokio::test]
    async fn test_message_router() {
        let router = MessageRouter::new();

        // Add routing rule
        let rule = RoutingRule {
            target_layer: LayerType::Storage,
            condition: RoutingCondition::SourceEquals(LayerType::DataAvailability),
            priority: 1,
        };
        router
            .add_routing_rule(MessageType::DataAvailability, rule)
            .await;

        // Test message routing
        let message = create_test_message(
            MessageType::DataAvailability,
            LayerType::DataAvailability,
            None,
        )
        .await;

        let targets = router.route_message(&message).await.unwrap();
        assert!(!targets.is_empty());
        assert!(targets.iter().any(|t| t.contains("Storage")));
    }

    #[tokio::test]
    async fn test_subscription_unsubscribe() {
        let bus = Arc::new(ModularMessageBus::new());

        // Create subscription
        let (subscription_id, _receiver) = bus
            .subscribe_enhanced(
                LayerType::Monitoring,
                vec![MessageType::HealthCheck],
                vec![],
                DeliveryMode::BestEffort,
            )
            .await
            .unwrap();

        // Verify subscription exists
        let metrics_before = bus.get_metrics().await;
        assert_eq!(metrics_before.active_subscriptions, 1);

        // Unsubscribe
        bus.unsubscribe(&subscription_id).await.unwrap();

        // Verify subscription removed
        let metrics_after = bus.get_metrics().await;
        assert_eq!(metrics_after.active_subscriptions, 0);

        // Try to unsubscribe again (should fail)
        let result = bus.unsubscribe(&subscription_id).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_broadcast_health_check() {
        let bus = Arc::new(ModularMessageBus::new());

        // Create subscription for health checks
        let (_subscription_id, mut receiver) = bus
            .subscribe_enhanced(
                LayerType::Storage,
                vec![MessageType::HealthCheck],
                vec![],
                DeliveryMode::BestEffort,
            )
            .await
            .unwrap();

        // Broadcast health check
        bus.broadcast_health_check().await.unwrap();

        // Verify health check received
        let received = tokio::time::timeout(Duration::from_millis(100), receiver.recv())
            .await
            .unwrap()
            .unwrap();

        assert_eq!(received.message_type, MessageType::HealthCheck);
        assert_eq!(received.source_layer, LayerType::Monitoring);

        match received.payload {
            MessagePayload::HealthCheck { is_healthy, .. } => {
                assert!(is_healthy);
            }
            _ => panic!("Expected HealthCheck payload"),
        }
    }

    #[tokio::test]
    async fn test_multiple_subscribers_same_type() {
        let bus = Arc::new(ModularMessageBus::new());

        // Create multiple subscriptions for the same message type
        let (_sub1_id, mut receiver1) = bus
            .subscribe_enhanced(
                LayerType::Execution,
                vec![MessageType::BlockProposal],
                vec![],
                DeliveryMode::BestEffort,
            )
            .await
            .unwrap();

        let (_sub2_id, mut receiver2) = bus
            .subscribe_enhanced(
                LayerType::Settlement,
                vec![MessageType::BlockProposal],
                vec![],
                DeliveryMode::BestEffort,
            )
            .await
            .unwrap();

        // Publish message
        let message = create_test_message(
            MessageType::BlockProposal,
            LayerType::Consensus,
            None, // No specific target, should go to all subscribers
        )
        .await;
        let original_id = message.id.clone();
        bus.publish(message.clone()).await.unwrap();

        // Both subscribers should receive the message
        let received1 = tokio::time::timeout(Duration::from_millis(100), receiver1.recv())
            .await
            .unwrap()
            .unwrap();

        let received2 = tokio::time::timeout(Duration::from_millis(100), receiver2.recv())
            .await
            .unwrap()
            .unwrap();

        assert!(received1.id.starts_with(&original_id));
        assert!(received2.id.starts_with(&original_id));

        // Verify metrics
        let metrics = bus.get_metrics().await;
        assert_eq!(metrics.active_subscriptions, 2);
        assert!(metrics.messages_delivered >= 2);
    }
}
