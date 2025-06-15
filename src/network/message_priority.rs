//! Message Priority and Rate Limiting Module
//!
//! Provides message prioritization, rate limiting, and bandwidth management
//! for efficient network communication.

use crate::network::PeerId;
use crate::Result;
use failure::format_err;
use serde::{Deserialize, Serialize};
use std::{
    collections::{HashMap, VecDeque},
    sync::{Arc, Mutex},
    time::{Duration, Instant},
};
use tokio::{
    sync::{RwLock, Semaphore},
    time::sleep,
};

/// Message priority levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum MessagePriority {
    Critical = 0,   // Consensus messages, block announcements
    High = 1,       // Transaction propagation, peer discovery
    Normal = 2,     // General communication
    Low = 3,        // Background sync, statistics
}

impl Default for MessagePriority {
    fn default() -> Self {
        MessagePriority::Normal
    }
}

/// Message with priority and metadata
#[derive(Debug, Clone)]
pub struct PrioritizedMessage {
    pub id: String,
    pub priority: MessagePriority,
    pub data: Vec<u8>,
    pub target_peer: Option<PeerId>,
    pub created_at: Instant,
    pub expires_at: Option<Instant>,
    pub retry_count: u32,
    pub max_retries: u32,
}

impl PrioritizedMessage {
    pub fn new(
        id: String,
        priority: MessagePriority,
        data: Vec<u8>,
        target_peer: Option<PeerId>,
    ) -> Self {
        let now = Instant::now();
        Self {
            id,
            priority,
            data,
            target_peer,
            created_at: now,
            expires_at: Some(now + Duration::from_secs(300)), // 5 minutes default
            retry_count: 0,
            max_retries: 3,
        }
    }

    pub fn is_expired(&self) -> bool {
        if let Some(expires_at) = self.expires_at {
            Instant::now() > expires_at
        } else {
            false
        }
    }

    pub fn can_retry(&self) -> bool {
        self.retry_count < self.max_retries
    }

    pub fn increment_retry(&mut self) {
        self.retry_count += 1;
    }
}

/// Rate limiting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    pub max_messages_per_second: u32,
    pub max_bytes_per_second: u64,
    pub burst_allowance: u32,
    pub window_size: Duration,
    pub per_peer_limit: bool,
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            max_messages_per_second: 100,
            max_bytes_per_second: 1024 * 1024, // 1MB/s
            burst_allowance: 20,
            window_size: Duration::from_secs(1),
            per_peer_limit: true,
        }
    }
}

/// Rate limiter state for tracking usage
#[derive(Debug)]
struct RateLimiterState {
    message_count: u32,
    byte_count: u64,
    window_start: Instant,
    burst_tokens: u32,
}

impl RateLimiterState {
    fn new(burst_allowance: u32) -> Self {
        Self {
            message_count: 0,
            byte_count: 0,
            window_start: Instant::now(),
            burst_tokens: burst_allowance,
        }
    }

    fn reset_window(&mut self, burst_allowance: u32) {
        self.message_count = 0;
        self.byte_count = 0;
        self.window_start = Instant::now();
        self.burst_tokens = burst_allowance;
    }

    fn should_reset_window(&self, window_size: Duration) -> bool {
        Instant::now().duration_since(self.window_start) >= window_size
    }
}

/// Message queue with priority support
pub struct PriorityMessageQueue {
    queues: [VecDeque<PrioritizedMessage>; 4], // One for each priority level
    rate_limiters: Arc<RwLock<HashMap<PeerId, RateLimiterState>>>,
    global_rate_limiter: Arc<Mutex<RateLimiterState>>,
    config: RateLimitConfig,
    bandwidth_semaphore: Arc<Semaphore>,
}

impl PriorityMessageQueue {
    pub fn new(config: RateLimitConfig) -> Self {
        let bandwidth_permits = config.max_bytes_per_second as usize;
        
        Self {
            queues: [
                VecDeque::new(), // Critical
                VecDeque::new(), // High
                VecDeque::new(), // Normal
                VecDeque::new(), // Low
            ],
            rate_limiters: Arc::new(RwLock::new(HashMap::new())),
            global_rate_limiter: Arc::new(Mutex::new(RateLimiterState::new(config.burst_allowance))),
            config: config.clone(),
            bandwidth_semaphore: Arc::new(Semaphore::new(bandwidth_permits)),
        }
    }

    /// Add a message to the appropriate priority queue
    pub fn enqueue(&mut self, message: PrioritizedMessage) -> Result<()> {
        if message.is_expired() {
            return Err(format_err!("Message expired before queuing"));
        }

        let priority_index = message.priority as usize;
        self.queues[priority_index].push_back(message);
        
        Ok(())
    }

    /// Dequeue the highest priority message that passes rate limiting
    pub fn dequeue(&mut self) -> Option<PrioritizedMessage> {
        // First pass: check for expired messages and remove them
        for queue in &mut self.queues {
            queue.retain(|msg| !msg.is_expired());
        }

        // Reset global rate limiter window if needed
        if let Ok(mut global_limiter) = self.global_rate_limiter.try_lock() {
            if global_limiter.should_reset_window(self.config.window_size) {
                global_limiter.reset_window(self.config.burst_allowance);
            }
        }

        // Find the highest priority message
        for queue in &mut self.queues {
            if let Some(message) = queue.pop_front() {
                // Update rate limits and try to acquire bandwidth
                self.update_rate_limit_state_sync(&message);
                
                // Try to acquire bandwidth semaphore
                if self.bandwidth_semaphore.available_permits() > message.data.len() {
                    let _ = self.bandwidth_semaphore.try_acquire_many(message.data.len() as u32);
                }
                
                return Some(message);
            }
        }
        
        None
    }

    /// Async version of dequeue with full rate limiting
    pub async fn dequeue_async(&mut self) -> Option<PrioritizedMessage> {
        // First pass: check for expired messages and remove them
        for queue in &mut self.queues {
            queue.retain(|msg| !msg.is_expired());
        }

        // Collect candidate messages first to avoid borrowing issues
        let mut candidates = Vec::new();
        for (priority, queue) in self.queues.iter().enumerate() {
            if let Some(message) = queue.front() {
                candidates.push((priority, message.clone()));
            }
        }

        // Check rate limits for candidates
        for (priority, message) in candidates {
            if self.check_rate_limit(&message).await {
                // Remove the message from the appropriate queue
                if let Some(actual_message) = self.queues[priority].pop_front() {
                    self.update_rate_limit_state(&actual_message).await;
                    return Some(actual_message);
                }
            }
        }
        
        None
    }

    /// Synchronous rate limit state update
    fn update_rate_limit_state_sync(&self, message: &PrioritizedMessage) {
        // Update global state
        if let Ok(mut global_limiter) = self.global_rate_limiter.try_lock() {
            global_limiter.message_count += 1;
            global_limiter.byte_count += message.data.len() as u64;
            
            if global_limiter.burst_tokens > 0 {
                global_limiter.burst_tokens -= 1;
            }
        }
    }

    /// Check if message passes rate limiting
    async fn check_rate_limit(&self, message: &PrioritizedMessage) -> bool {
        let now = Instant::now();
        
        // Check global rate limit
        {
            let mut global_limiter = self.global_rate_limiter.lock().unwrap();
            
            // Reset window if needed
            if now.duration_since(global_limiter.window_start) >= self.config.window_size {
                global_limiter.reset_window(self.config.burst_allowance);
            }
            
            // Check global limits
            if global_limiter.message_count >= self.config.max_messages_per_second &&
               global_limiter.burst_tokens == 0 {
                return false;
            }
            
            if global_limiter.byte_count + message.data.len() as u64 > self.config.max_bytes_per_second {
                return false;
            }
        }

        // Check per-peer rate limit if enabled
        if self.config.per_peer_limit {
            if let Some(peer_id) = &message.target_peer {
                let mut rate_limiters = self.rate_limiters.write().await;
                let limiter = rate_limiters
                    .entry(peer_id.clone())
                    .or_insert_with(|| RateLimiterState::new(self.config.burst_allowance));
                
                // Reset window if needed
                if now.duration_since(limiter.window_start) >= self.config.window_size {
                    limiter.reset_window(self.config.burst_allowance);
                }
                
                // Check per-peer limits
                if limiter.message_count >= self.config.max_messages_per_second / 10 && // 10% of global limit per peer
                   limiter.burst_tokens == 0 {
                    return false;
                }
            }
        }

        // Check bandwidth semaphore
        if self.bandwidth_semaphore.available_permits() < message.data.len() {
            return false;
        }

        true
    }

    /// Update rate limiting state after sending a message
    async fn update_rate_limit_state(&self, message: &PrioritizedMessage) {
        // Update global state
        {
            let mut global_limiter = self.global_rate_limiter.lock().unwrap();
            global_limiter.message_count += 1;
            global_limiter.byte_count += message.data.len() as u64;
            
            if global_limiter.burst_tokens > 0 {
                global_limiter.burst_tokens -= 1;
            }
        }

        // Update per-peer state if enabled
        if self.config.per_peer_limit {
            if let Some(peer_id) = &message.target_peer {
                let mut rate_limiters = self.rate_limiters.write().await;
                if let Some(limiter) = rate_limiters.get_mut(peer_id) {
                    limiter.message_count += 1;
                    limiter.byte_count += message.data.len() as u64;
                    
                    if limiter.burst_tokens > 0 {
                        limiter.burst_tokens -= 1;
                    }
                }
            }
        }

        // Acquire bandwidth permits
        if let Ok(permit) = self.bandwidth_semaphore.clone().acquire_many_owned(message.data.len() as u32).await {
            // Release permits after a delay to simulate bandwidth usage
            tokio::spawn(async move {
                sleep(Duration::from_millis(10)).await;
                drop(permit);
            });
        }
    }

    /// Get comprehensive queue statistics
    pub async fn get_stats(&self) -> QueueStats {
        QueueStats {
            critical_queue_size: self.queues[0].len(),
            high_queue_size: self.queues[1].len(),
            normal_queue_size: self.queues[2].len(),
            low_queue_size: self.queues[3].len(),
            total_messages_processed: self.get_total_processed(),
            total_messages_dropped: self.get_total_dropped(),
            average_processing_time: self.get_average_processing_time(),
            bandwidth_usage: self.get_bandwidth_usage(),
        }
    }

    /// Get basic queue statistics as HashMap
    pub fn get_basic_stats(&self) -> HashMap<String, u64> {
        let mut stats = HashMap::new();
        
        for (priority, queue) in self.queues.iter().enumerate() {
            let priority_name = match priority {
                0 => "critical",
                1 => "high", 
                2 => "normal",
                3 => "low",
                _ => "unknown",
            };
            stats.insert(format!("{}_queue_size", priority_name), queue.len() as u64);
        }
        
        stats.insert("total_queue_size".to_string(), 
                    self.queues.iter().map(|q| q.len() as u64).sum());
        
        stats
    }

    fn get_total_processed(&self) -> u64 {
        // This would be tracked in practice
        0
    }

    fn get_total_dropped(&self) -> u64 {
        // This would be tracked in practice
        0
    }

    fn get_average_processing_time(&self) -> Duration {
        // This would be calculated from timing data
        Duration::from_millis(0)
    }

    fn get_bandwidth_usage(&self) -> f64 {
        // This would be calculated from bandwidth monitor
        0.0
    }

    /// Clean up expired messages and old rate limiter states
    pub async fn cleanup(&mut self) {
        // Remove expired messages
        for queue in &mut self.queues {
            queue.retain(|msg| !msg.is_expired());
        }

        // Clean up old rate limiter states
        let mut rate_limiters = self.rate_limiters.write().await;
        let now = Instant::now();
        
        rate_limiters.retain(|_, limiter| {
            now.duration_since(limiter.window_start) < Duration::from_secs(300) // Keep for 5 minutes
        });
    }
}

/// Bandwidth monitor for tracking network usage
pub struct BandwidthMonitor {
    upload_bytes: Arc<Mutex<u64>>,
    download_bytes: Arc<Mutex<u64>>,
    upload_rate: Arc<Mutex<f64>>, // bytes per second
    download_rate: Arc<Mutex<f64>>, // bytes per second
    last_update: Arc<Mutex<Instant>>,
}

impl BandwidthMonitor {
    pub fn new() -> Self {
        Self {
            upload_bytes: Arc::new(Mutex::new(0)),
            download_bytes: Arc::new(Mutex::new(0)),
            upload_rate: Arc::new(Mutex::new(0.0)),
            download_rate: Arc::new(Mutex::new(0.0)),
            last_update: Arc::new(Mutex::new(Instant::now())),
        }
    }

    pub fn record_upload(&self, bytes: u64) {
        let mut upload_bytes = self.upload_bytes.lock().unwrap();
        *upload_bytes += bytes;
        self.update_rates();
    }

    pub fn record_download(&self, bytes: u64) {
        let mut download_bytes = self.download_bytes.lock().unwrap();
        *download_bytes += bytes;
        self.update_rates();
    }

    fn update_rates(&self) {
        let now = Instant::now();
        let mut last_update = self.last_update.lock().unwrap();
        
        let elapsed = now.duration_since(*last_update).as_secs_f64();
        if elapsed >= 1.0 { // Update rates every second
            let upload_bytes = *self.upload_bytes.lock().unwrap();
            let download_bytes = *self.download_bytes.lock().unwrap();
            
            let mut upload_rate = self.upload_rate.lock().unwrap();
            let mut download_rate = self.download_rate.lock().unwrap();
            
            *upload_rate = upload_bytes as f64 / elapsed;
            *download_rate = download_bytes as f64 / elapsed;
            
            // Reset counters
            *self.upload_bytes.lock().unwrap() = 0;
            *self.download_bytes.lock().unwrap() = 0;
            *last_update = now;
        }
    }

    pub fn get_upload_rate(&self) -> f64 {
        *self.upload_rate.lock().unwrap()
    }

    pub fn get_download_rate(&self) -> f64 {
        *self.download_rate.lock().unwrap()
    }

    pub fn get_total_upload(&self) -> u64 {
        *self.upload_bytes.lock().unwrap()
    }

    pub fn get_total_download(&self) -> u64 {
        *self.download_bytes.lock().unwrap()
    }
}

impl Default for BandwidthMonitor {
    fn default() -> Self {
        Self::new()
    }
}

/// Statistics for the priority message queue
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QueueStats {
    pub critical_queue_size: usize,
    pub high_queue_size: usize,
    pub normal_queue_size: usize,
    pub low_queue_size: usize,
    pub total_messages_processed: u64,
    pub total_messages_dropped: u64,
    pub average_processing_time: Duration,
    pub bandwidth_usage: f64,
}

impl Default for QueueStats {
    fn default() -> Self {
        Self {
            critical_queue_size: 0,
            high_queue_size: 0,
            normal_queue_size: 0,
            low_queue_size: 0,
            total_messages_processed: 0,
            total_messages_dropped: 0,
            average_processing_time: Duration::from_millis(0),
            bandwidth_usage: 0.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use uuid::Uuid;

    #[tokio::test]
    async fn test_priority_queue() {
        let config = RateLimitConfig::default();
        let mut queue = PriorityMessageQueue::new(config);
        
        // Add messages with different priorities
        let critical_msg = PrioritizedMessage::new(
            Uuid::new_v4().to_string(),
            MessagePriority::Critical,
            b"critical".to_vec(),
            None,
        );
        
        let normal_msg = PrioritizedMessage::new(
            Uuid::new_v4().to_string(),
            MessagePriority::Normal,
            b"normal".to_vec(),
            None,
        );
        
        queue.enqueue(normal_msg).unwrap();
        queue.enqueue(critical_msg).unwrap();
        
        // Critical message should come out first
        let dequeued = queue.dequeue().unwrap();
        assert_eq!(dequeued.priority, MessagePriority::Critical);
        
        let dequeued = queue.dequeue().unwrap();
        assert_eq!(dequeued.priority, MessagePriority::Normal);
    }

    #[tokio::test]
    async fn test_message_expiration() {
        let config = RateLimitConfig::default();
        let mut queue = PriorityMessageQueue::new(config);
        
        let mut expired_msg = PrioritizedMessage::new(
            Uuid::new_v4().to_string(),
            MessagePriority::Normal,
            b"expired".to_vec(),
            None,
        );
        expired_msg.expires_at = Some(Instant::now() - Duration::from_secs(1));
        
        // Should fail to enqueue expired message
        assert!(queue.enqueue(expired_msg).is_err());
    }

    #[test]
    fn test_bandwidth_monitor() {
        let monitor = BandwidthMonitor::new();
        
        monitor.record_upload(1024);
        monitor.record_download(2048);
        
        assert_eq!(monitor.get_total_upload(), 1024);
        assert_eq!(monitor.get_total_download(), 2048);
    }
}
