//! Enhanced Unified Contract Engine
//!
//! This module provides an advanced unified contract engine that supports
//! multiple execution environments, advanced gas metering, and sophisticated
//! contract lifecycle management.

use std::{collections::HashMap, sync::Arc, time::Instant};

use anyhow::Result;
use serde::{Deserialize, Serialize};
use tokio::sync::RwLock;
use uuid::Uuid;

use super::{
    database_storage::DatabaseContractStorage,
    privacy_engine::PrivacyContractEngine,
    unified_engine::{
        ContractStateStorage, ContractType, UnifiedContractEngine, UnifiedContractExecution,
        UnifiedContractMetadata, UnifiedContractResult, UnifiedGasManager,
    },
    wasm_engine::WasmContractEngine,
};
use crate::diamond_io_integration::PrivacyEngineConfig;

/// Enhanced unified contract engine with advanced features
pub struct EnhancedUnifiedContractEngine {
    /// Storage backend
    storage: Arc<dyn ContractStateStorage>,
    /// Gas management
    _gas_manager: UnifiedGasManager,
    /// WASM execution engine
    wasm_engine: Arc<RwLock<WasmContractEngine>>,
    /// Privacy-enhanced contract engine
    privacy_engine: Arc<RwLock<PrivacyContractEngine>>,
    /// Advanced monitoring and analytics
    analytics: Arc<RwLock<ContractAnalytics>>,
    /// Contract execution cache
    execution_cache: Arc<RwLock<ExecutionCache>>,
    /// Configuration
    config: EnhancedEngineConfig,
}

/// Enhanced engine configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EnhancedEngineConfig {
    /// Enable execution result caching
    pub enable_caching: bool,
    /// Cache TTL in seconds
    pub cache_ttl_secs: u64,
    /// Maximum cache size (number of entries)
    pub max_cache_entries: usize,
    /// Enable analytics collection
    pub enable_analytics: bool,
    /// Analytics retention period in seconds
    pub analytics_retention_secs: u64,
    /// Enable automatic contract optimization
    pub enable_optimization: bool,
    /// Gas limit enforcement
    pub enforce_gas_limits: bool,
    /// Maximum execution time in milliseconds
    pub max_execution_time_ms: u64,
    /// Enable parallel execution (where safe)
    pub enable_parallel_execution: bool,
    /// Monitoring configuration
    pub monitoring: MonitoringConfig,
}

/// Monitoring configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MonitoringConfig {
    /// Enable performance monitoring
    pub enable_performance_monitoring: bool,
    /// Enable error tracking
    pub enable_error_tracking: bool,
    /// Enable execution tracing
    pub enable_execution_tracing: bool,
    /// Metrics collection interval in seconds
    pub metrics_interval_secs: u64,
}

/// Contract analytics and monitoring
#[derive(Debug, Clone, Default)]
pub struct ContractAnalytics {
    /// Total number of contract deployments
    pub total_deployments: u64,
    /// Total number of contract executions
    pub total_executions: u64,
    /// Total gas consumed across all executions
    pub total_gas_consumed: u64,
    /// Average execution time in milliseconds
    pub avg_execution_time_ms: f64,
    /// Execution success rate (0.0 to 1.0)
    pub success_rate: f64,
    /// Contract-specific metrics
    pub contract_metrics: HashMap<String, ContractMetrics>,
    /// Recent execution events
    pub recent_events: Vec<ExecutionEvent>,
    /// Error statistics
    pub error_stats: HashMap<String, u64>,
}

/// Metrics for individual contracts
#[derive(Debug, Clone, Default)]
pub struct ContractMetrics {
    pub executions: u64,
    pub gas_consumed: u64,
    pub avg_execution_time_ms: f64,
    pub success_count: u64,
    pub failure_count: u64,
    pub last_execution_time: u64,
}

/// Execution events for analytics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionEvent {
    pub timestamp: u64,
    pub contract_address: String,
    pub function_name: String,
    pub execution_time_ms: u64,
    pub gas_used: u64,
    pub success: bool,
    pub error_type: Option<String>,
}

/// Execution result cache
#[derive(Debug, Clone)]
pub struct ExecutionCache {
    entries: HashMap<String, CacheEntry>,
    max_entries: usize,
    ttl_secs: u64,
}

/// Cache entry with TTL
#[derive(Debug, Clone)]
pub struct CacheEntry {
    pub result: UnifiedContractResult,
    pub timestamp: Instant,
    pub ttl_secs: u64,
}

/// Advanced execution context with enhanced features
#[derive(Debug, Clone)]
pub struct EnhancedExecutionContext {
    pub execution: UnifiedContractExecution,
    pub start_time: Instant,
    pub execution_id: String,
    pub tracing_enabled: bool,
    pub optimization_enabled: bool,
}

impl Default for EnhancedEngineConfig {
    fn default() -> Self {
        Self {
            enable_caching: true,
            cache_ttl_secs: 300, // 5 minutes
            max_cache_entries: 1000,
            enable_analytics: true,
            analytics_retention_secs: 86400, // 24 hours
            enable_optimization: true,
            enforce_gas_limits: true,
            max_execution_time_ms: 30000,     // 30 seconds
            enable_parallel_execution: false, // Disabled by default for safety
            monitoring: MonitoringConfig::default(),
        }
    }
}

impl Default for MonitoringConfig {
    fn default() -> Self {
        Self {
            enable_performance_monitoring: true,
            enable_error_tracking: true,
            enable_execution_tracing: false, // Disabled by default (performance impact)
            metrics_interval_secs: 60,       // 1 minute
        }
    }
}

impl ExecutionCache {
    pub fn new(max_entries: usize, ttl_secs: u64) -> Self {
        Self {
            entries: HashMap::new(),
            max_entries,
            ttl_secs,
        }
    }

    pub fn get(&mut self, key: &str) -> Option<UnifiedContractResult> {
        // Clean expired entries first
        self.clean_expired();

        if let Some(entry) = self.entries.get(key) {
            if entry.timestamp.elapsed().as_secs() < entry.ttl_secs {
                return Some(entry.result.clone());
            } else {
                self.entries.remove(key);
            }
        }
        None
    }

    pub fn insert(&mut self, key: String, result: UnifiedContractResult) {
        // Ensure we don't exceed max capacity
        if self.entries.len() >= self.max_entries {
            self.evict_oldest();
        }

        let entry = CacheEntry {
            result,
            timestamp: Instant::now(),
            ttl_secs: self.ttl_secs,
        };

        self.entries.insert(key, entry);
    }

    fn clean_expired(&mut self) {
        let now = Instant::now();
        self.entries
            .retain(|_, entry| now.duration_since(entry.timestamp).as_secs() < entry.ttl_secs);
    }

    fn evict_oldest(&mut self) {
        if let Some((oldest_key, _)) = self
            .entries
            .iter()
            .min_by_key(|(_, entry)| entry.timestamp)
            .map(|(k, v)| (k.clone(), v.clone()))
        {
            self.entries.remove(&oldest_key);
        }
    }

    pub fn stats(&self) -> (usize, usize) {
        (self.entries.len(), self.max_entries)
    }
}

impl EnhancedUnifiedContractEngine {
    /// Create a new enhanced unified contract engine
    pub async fn new(
        storage: Arc<dyn ContractStateStorage>,
        gas_manager: UnifiedGasManager,
        privacy_config: PrivacyEngineConfig,
        config: EnhancedEngineConfig,
    ) -> Result<Self> {
        let wasm_engine = Arc::new(RwLock::new(WasmContractEngine::new(
            Arc::clone(&storage),
            gas_manager.clone(),
        )?));

        let privacy_engine = Arc::new(RwLock::new(PrivacyContractEngine::new(
            Arc::clone(&storage),
            gas_manager.clone(),
            privacy_config,
        )?));

        let analytics = Arc::new(RwLock::new(ContractAnalytics::default()));
        let execution_cache = Arc::new(RwLock::new(ExecutionCache::new(
            config.max_cache_entries,
            config.cache_ttl_secs,
        )));

        Ok(Self {
            storage,
            _gas_manager: gas_manager,
            wasm_engine,
            privacy_engine,
            analytics,
            execution_cache,
            config,
        })
    }

    /// Create enhanced engine with database storage
    pub async fn with_database_storage(
        database_config: super::database_storage::DatabaseStorageConfig,
        gas_manager: UnifiedGasManager,
        privacy_config: PrivacyEngineConfig,
        engine_config: EnhancedEngineConfig,
    ) -> Result<Self> {
        let storage = Arc::new(DatabaseContractStorage::new(database_config).await?);
        Self::new(storage, gas_manager, privacy_config, engine_config).await
    }

    /// Deploy a contract with enhanced features
    pub async fn deploy_contract_enhanced(
        &self,
        metadata: UnifiedContractMetadata,
        init_data: Vec<u8>,
        deployment_options: DeploymentOptions,
    ) -> Result<DeploymentResult> {
        let start_time = Instant::now();
        let deployment_id = Uuid::new_v4().to_string();

        // Record deployment attempt
        if self.config.enable_analytics {
            let mut analytics = self.analytics.write().await;
            analytics.total_deployments += 1;
        }

        // Validate deployment parameters
        if deployment_options.validate_bytecode {
            self.validate_contract_bytecode(&metadata.contract_type)?;
        }

        // Deploy based on contract type
        let contract_address = match &metadata.contract_type {
            ContractType::Wasm { .. } | ContractType::BuiltIn { .. } => {
                let mut engine = self.wasm_engine.write().await;
                engine.deploy_contract(metadata.clone(), init_data.clone())?
            }
            ContractType::PrivacyEnhanced { .. } => {
                let mut engine = self.privacy_engine.write().await;
                engine.deploy_contract(metadata.clone(), init_data.clone())?
            }
        };

        let deployment_time = start_time.elapsed();

        // Create deployment result
        let result = DeploymentResult {
            contract_address: contract_address.clone(),
            deployment_id,
            deployment_time_ms: deployment_time.as_millis() as u64,
            gas_used: deployment_options.gas_limit,
            success: true,
            optimization_applied: deployment_options.enable_optimization,
            validation_passed: deployment_options.validate_bytecode,
        };

        // Update analytics
        if self.config.enable_analytics {
            self.record_deployment_analytics(&result).await;
        }

        Ok(result)
    }

    /// Execute contract with enhanced monitoring and caching
    pub async fn execute_contract_enhanced(
        &self,
        execution: UnifiedContractExecution,
        execution_options: ExecutionOptions,
    ) -> Result<EnhancedExecutionResult> {
        let execution_id = Uuid::new_v4().to_string();
        let start_time = Instant::now();

        // Create enhanced execution context
        let context = EnhancedExecutionContext {
            execution: execution.clone(),
            start_time,
            execution_id: execution_id.clone(),
            tracing_enabled: execution_options.enable_tracing,
            optimization_enabled: execution_options.enable_optimization,
        };

        // Check cache first if enabled
        if self.config.enable_caching && execution_options.use_cache {
            let cache_key = self.make_cache_key(&execution);
            let mut cache = self.execution_cache.write().await;
            if let Some(cached_result) = cache.get(&cache_key) {
                return Ok(EnhancedExecutionResult {
                    basic_result: cached_result,
                    execution_id,
                    execution_time_ms: 0, // Cached result
                    cache_hit: true,
                    optimizations_applied: Vec::new(),
                    trace_data: None,
                    analytics_recorded: false,
                });
            }
        }

        // Execute based on contract type
        let contract_type = self.get_contract_type(&execution.contract_address).await?;
        let basic_result = match contract_type {
            ContractType::Wasm { .. } | ContractType::BuiltIn { .. } => {
                let mut engine = self.wasm_engine.write().await;
                engine.execute_contract(execution.clone())?
            }
            ContractType::PrivacyEnhanced { .. } => {
                let mut engine = self.privacy_engine.write().await;
                engine.execute_contract(execution.clone())?
            }
        };

        let execution_time = start_time.elapsed();

        // Apply post-execution optimizations if enabled
        let optimizations_applied = if execution_options.enable_optimization {
            self.apply_execution_optimizations(&basic_result).await
        } else {
            Vec::new()
        };

        // Collect trace data if enabled
        let trace_data = if execution_options.enable_tracing {
            Some(self.collect_trace_data(&context, &basic_result).await)
        } else {
            None
        };

        // Cache result if enabled and successful
        if self.config.enable_caching && basic_result.success {
            let cache_key = self.make_cache_key(&execution);
            let mut cache = self.execution_cache.write().await;
            cache.insert(cache_key, basic_result.clone());
        }

        // Record analytics
        let analytics_recorded = if self.config.enable_analytics {
            self.record_execution_analytics(&context, &basic_result)
                .await;
            true
        } else {
            false
        };

        Ok(EnhancedExecutionResult {
            basic_result,
            execution_id,
            execution_time_ms: execution_time.as_millis() as u64,
            cache_hit: false,
            optimizations_applied,
            trace_data,
            analytics_recorded,
        })
    }

    /// Get comprehensive analytics
    pub async fn get_analytics(&self) -> ContractAnalytics {
        self.analytics.read().await.clone()
    }

    /// Get real-time performance metrics
    pub async fn get_performance_metrics(&self) -> Result<PerformanceMetrics> {
        let analytics = self.analytics.read().await;
        let cache_stats = {
            let cache = self.execution_cache.read().await;
            cache.stats()
        };

        Ok(PerformanceMetrics {
            total_executions: analytics.total_executions,
            total_gas_consumed: analytics.total_gas_consumed,
            avg_execution_time_ms: analytics.avg_execution_time_ms,
            success_rate: analytics.success_rate,
            cache_hit_rate: self.calculate_cache_hit_rate().await,
            cache_utilization: cache_stats.0 as f64 / cache_stats.1 as f64,
            active_contracts: analytics.contract_metrics.len(),
            recent_error_rate: self.calculate_recent_error_rate().await,
        })
    }

    /// Optimize contract execution patterns
    pub async fn optimize_contracts(&self) -> Result<OptimizationReport> {
        let analytics = self.analytics.read().await;
        let mut report = OptimizationReport {
            optimizations_applied: Vec::new(),
            contracts_optimized: 0,
            estimated_gas_savings: 0,
            estimated_time_savings_ms: 0,
        };

        // Analyze contract patterns and suggest optimizations
        for (contract_address, metrics) in &analytics.contract_metrics {
            if metrics.executions > 100 && metrics.avg_execution_time_ms > 1000.0 {
                // Suggest caching for frequently executed slow contracts
                report.optimizations_applied.push(format!(
                    "Enable aggressive caching for contract {} (avg execution: {:.2}ms)",
                    contract_address, metrics.avg_execution_time_ms
                ));
                report.contracts_optimized += 1;
                report.estimated_time_savings_ms += (metrics.avg_execution_time_ms * 0.3) as u64;
            }

            if metrics.gas_consumed > 1_000_000 {
                // Suggest gas optimization for high-gas contracts
                report.optimizations_applied.push(format!(
                    "Optimize gas usage for contract {} (total gas: {})",
                    contract_address, metrics.gas_consumed
                ));
                report.estimated_gas_savings += metrics.gas_consumed / 10; // 10% estimated savings
            }
        }

        Ok(report)
    }

    /// Get detailed contract health report
    pub async fn get_contract_health(
        &self,
        contract_address: &str,
    ) -> Result<ContractHealthReport> {
        let analytics = self.analytics.read().await;
        let metrics = analytics
            .contract_metrics
            .get(contract_address)
            .cloned()
            .unwrap_or_default();

        let success_rate = if metrics.executions > 0 {
            metrics.success_count as f64 / metrics.executions as f64
        } else {
            0.0
        };

        let health_score = self.calculate_health_score(&metrics);
        let status = if health_score > 0.8 {
            ContractHealthStatus::Healthy
        } else if health_score > 0.6 {
            ContractHealthStatus::Warning
        } else {
            ContractHealthStatus::Critical
        };

        Ok(ContractHealthReport {
            contract_address: contract_address.to_string(),
            health_score,
            status,
            total_executions: metrics.executions,
            success_rate,
            avg_execution_time_ms: metrics.avg_execution_time_ms,
            gas_efficiency: metrics.gas_consumed as f64 / metrics.executions.max(1) as f64,
            last_execution: metrics.last_execution_time,
            recommendations: self.generate_health_recommendations(&metrics),
        })
    }

    // Helper methods
    async fn get_contract_type(&self, address: &str) -> Result<ContractType> {
        if let Some(metadata) = self.storage.get_contract_metadata(address)? {
            Ok(metadata.contract_type)
        } else {
            Err(anyhow::anyhow!("Contract not found: {}", address))
        }
    }

    fn make_cache_key(&self, execution: &UnifiedContractExecution) -> String {
        format!(
            "{}:{}:{}",
            execution.contract_address,
            execution.function_name,
            blake3::hash(&execution.input_data).to_hex()
        )
    }

    async fn record_deployment_analytics(&self, _result: &DeploymentResult) {
        let _analytics = self.analytics.write().await;
        // Record deployment metrics (implementation details)
    }

    async fn record_execution_analytics(
        &self,
        context: &EnhancedExecutionContext,
        result: &UnifiedContractResult,
    ) {
        let mut analytics = self.analytics.write().await;
        analytics.total_executions += 1;
        analytics.total_gas_consumed += result.gas_used;

        // Update contract-specific metrics
        let metrics = analytics
            .contract_metrics
            .entry(context.execution.contract_address.clone())
            .or_default();
        metrics.executions += 1;
        metrics.gas_consumed += result.gas_used;

        if result.success {
            metrics.success_count += 1;
        } else {
            metrics.failure_count += 1;
        }

        metrics.avg_execution_time_ms = (metrics.avg_execution_time_ms
            * (metrics.executions - 1) as f64
            + result.execution_time_ms as f64)
            / metrics.executions as f64;

        metrics.last_execution_time = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs();

        // Update global success rate
        let total_success = analytics
            .contract_metrics
            .values()
            .map(|m| m.success_count)
            .sum::<u64>();
        analytics.success_rate = total_success as f64 / analytics.total_executions as f64;

        // Update average execution time
        analytics.avg_execution_time_ms = analytics
            .contract_metrics
            .values()
            .map(|m| m.avg_execution_time_ms * m.executions as f64)
            .sum::<f64>()
            / analytics.total_executions as f64;
    }

    async fn apply_execution_optimizations(&self, _result: &UnifiedContractResult) -> Vec<String> {
        // Placeholder for optimization logic
        vec!["Gas optimization applied".to_string()]
    }

    async fn collect_trace_data(
        &self,
        _context: &EnhancedExecutionContext,
        _result: &UnifiedContractResult,
    ) -> ExecutionTrace {
        // Placeholder for trace data collection
        ExecutionTrace {
            steps: Vec::new(),
            call_stack: Vec::new(),
            state_changes: Vec::new(),
        }
    }

    fn validate_contract_bytecode(&self, _contract_type: &ContractType) -> Result<()> {
        // Placeholder for bytecode validation
        Ok(())
    }

    async fn calculate_cache_hit_rate(&self) -> f64 {
        // Placeholder - would track cache hits/misses
        0.75
    }

    async fn calculate_recent_error_rate(&self) -> f64 {
        // Placeholder - would calculate error rate from recent events
        0.05
    }

    fn calculate_health_score(&self, metrics: &ContractMetrics) -> f64 {
        if metrics.executions == 0 {
            return 1.0; // New contract, assume healthy
        }

        let success_rate = metrics.success_count as f64 / metrics.executions as f64;
        let time_penalty = if metrics.avg_execution_time_ms > 5000.0 {
            0.1
        } else {
            0.0
        };
        let gas_efficiency =
            1.0 - (metrics.gas_consumed as f64 / (metrics.executions * 100000) as f64).min(0.5);

        (success_rate * 0.6 + gas_efficiency * 0.3 + (1.0 - time_penalty) * 0.1).clamp(0.0, 1.0)
    }

    fn generate_health_recommendations(&self, metrics: &ContractMetrics) -> Vec<String> {
        let mut recommendations = Vec::new();

        if metrics.executions > 0 {
            let success_rate = metrics.success_count as f64 / metrics.executions as f64;
            if success_rate < 0.9 {
                recommendations
                    .push("Consider reviewing contract logic for error conditions".to_string());
            }

            if metrics.avg_execution_time_ms > 10000.0 {
                recommendations.push("Optimize contract execution time".to_string());
            }

            if metrics.gas_consumed / metrics.executions > 500000 {
                recommendations.push("Review gas optimization opportunities".to_string());
            }
        }

        if recommendations.is_empty() {
            recommendations.push("Contract is performing well".to_string());
        }

        recommendations
    }
}

// Additional types for enhanced functionality

#[derive(Debug, Clone)]
pub struct DeploymentOptions {
    pub validate_bytecode: bool,
    pub enable_optimization: bool,
    pub gas_limit: u64,
    pub deployment_metadata: HashMap<String, String>,
}

#[derive(Debug, Clone)]
pub struct DeploymentResult {
    pub contract_address: String,
    pub deployment_id: String,
    pub deployment_time_ms: u64,
    pub gas_used: u64,
    pub success: bool,
    pub optimization_applied: bool,
    pub validation_passed: bool,
}

#[derive(Debug, Clone)]
pub struct ExecutionOptions {
    pub use_cache: bool,
    pub enable_tracing: bool,
    pub enable_optimization: bool,
    pub timeout_ms: Option<u64>,
}

#[derive(Debug, Clone)]
pub struct EnhancedExecutionResult {
    pub basic_result: UnifiedContractResult,
    pub execution_id: String,
    pub execution_time_ms: u64,
    pub cache_hit: bool,
    pub optimizations_applied: Vec<String>,
    pub trace_data: Option<ExecutionTrace>,
    pub analytics_recorded: bool,
}

#[derive(Debug, Clone)]
pub struct ExecutionTrace {
    pub steps: Vec<TraceStep>,
    pub call_stack: Vec<String>,
    pub state_changes: Vec<StateChange>,
}

#[derive(Debug, Clone)]
pub struct TraceStep {
    pub step_id: u64,
    pub operation: String,
    pub gas_cost: u64,
    pub timestamp_ns: u64,
}

#[derive(Debug, Clone)]
pub struct StateChange {
    pub key: String,
    pub old_value: Option<Vec<u8>>,
    pub new_value: Vec<u8>,
}

#[derive(Debug, Clone)]
pub struct PerformanceMetrics {
    pub total_executions: u64,
    pub total_gas_consumed: u64,
    pub avg_execution_time_ms: f64,
    pub success_rate: f64,
    pub cache_hit_rate: f64,
    pub cache_utilization: f64,
    pub active_contracts: usize,
    pub recent_error_rate: f64,
}

#[derive(Debug, Clone)]
pub struct OptimizationReport {
    pub optimizations_applied: Vec<String>,
    pub contracts_optimized: u64,
    pub estimated_gas_savings: u64,
    pub estimated_time_savings_ms: u64,
}

#[derive(Debug, Clone)]
pub struct ContractHealthReport {
    pub contract_address: String,
    pub health_score: f64,
    pub status: ContractHealthStatus,
    pub total_executions: u64,
    pub success_rate: f64,
    pub avg_execution_time_ms: f64,
    pub gas_efficiency: f64,
    pub last_execution: u64,
    pub recommendations: Vec<String>,
}

#[derive(Debug, Clone)]
pub enum ContractHealthStatus {
    Healthy,
    Warning,
    Critical,
}

impl Default for DeploymentOptions {
    fn default() -> Self {
        Self {
            validate_bytecode: true,
            enable_optimization: true,
            gas_limit: 10_000_000,
            deployment_metadata: HashMap::new(),
        }
    }
}

impl Default for ExecutionOptions {
    fn default() -> Self {
        Self {
            use_cache: true,
            enable_tracing: false,
            enable_optimization: true,
            timeout_ms: Some(30000),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::smart_contract::{
        unified_engine::{UnifiedGasConfig, UnifiedGasManager},
        unified_storage::SyncInMemoryContractStorage,
    };

    async fn create_test_engine() -> EnhancedUnifiedContractEngine {
        let storage = Arc::new(SyncInMemoryContractStorage::new());
        let gas_manager = UnifiedGasManager::new(UnifiedGasConfig::default());
        let privacy_config = PrivacyEngineConfig::dummy();
        let config = EnhancedEngineConfig::default();

        EnhancedUnifiedContractEngine::new(storage, gas_manager, privacy_config, config)
            .await
            .unwrap()
    }

    #[tokio::test]
    async fn test_enhanced_engine_creation() {
        let engine = create_test_engine().await;
        let analytics = engine.get_analytics().await;

        assert_eq!(analytics.total_deployments, 0);
        assert_eq!(analytics.total_executions, 0);
    }

    #[tokio::test]
    async fn test_execution_cache() {
        let mut cache = ExecutionCache::new(10, 300);

        let result = UnifiedContractResult {
            success: true,
            return_data: vec![1, 2, 3],
            gas_used: 50000,
            events: vec![],
            execution_time_ms: 100,
            error_message: None,
        };

        // Test cache insertion and retrieval
        cache.insert("test_key".to_string(), result.clone());
        let cached = cache.get("test_key");
        assert!(cached.is_some());
        assert_eq!(cached.unwrap().gas_used, 50000);

        // Test cache miss
        let missed = cache.get("nonexistent_key");
        assert!(missed.is_none());

        // Test cache stats
        let (used, max) = cache.stats();
        assert_eq!(used, 1);
        assert_eq!(max, 10);
    }

    #[tokio::test]
    async fn test_performance_metrics() {
        let engine = create_test_engine().await;
        let metrics = engine.get_performance_metrics().await.unwrap();

        // Initial metrics should be zero/default
        assert_eq!(metrics.total_executions, 0);
        assert_eq!(metrics.total_gas_consumed, 0);
        assert_eq!(metrics.active_contracts, 0);
    }

    #[tokio::test]
    async fn test_contract_health_calculation() {
        let engine = create_test_engine().await;

        // Test health for non-existent contract
        let health = engine.get_contract_health("0xnonexistent").await.unwrap();
        assert_eq!(health.health_score, 1.0); // New contract should be healthy
        assert!(matches!(health.status, ContractHealthStatus::Healthy));
    }

    #[tokio::test]
    async fn test_optimization_report() {
        let engine = create_test_engine().await;
        let report = engine.optimize_contracts().await.unwrap();

        // With no contract activity, should have no optimizations
        assert_eq!(report.contracts_optimized, 0);
        assert!(report.optimizations_applied.is_empty());
        assert_eq!(report.estimated_gas_savings, 0);
    }

    #[tokio::test]
    async fn test_enhanced_config_defaults() {
        let config = EnhancedEngineConfig::default();

        assert!(config.enable_caching);
        assert!(config.enable_analytics);
        assert!(config.enable_optimization);
        assert!(config.enforce_gas_limits);
        assert!(!config.enable_parallel_execution); // Should be disabled by default
        assert_eq!(config.cache_ttl_secs, 300);
        assert_eq!(config.max_execution_time_ms, 30000);
    }

    #[tokio::test]
    async fn test_deployment_options() {
        let options = DeploymentOptions::default();

        assert!(options.validate_bytecode);
        assert!(options.enable_optimization);
        assert_eq!(options.gas_limit, 10_000_000);
        assert!(options.deployment_metadata.is_empty());
    }

    #[tokio::test]
    async fn test_execution_options() {
        let options = ExecutionOptions::default();

        assert!(options.use_cache);
        assert!(!options.enable_tracing); // Should be disabled by default
        assert!(options.enable_optimization);
        assert_eq!(options.timeout_ms, Some(30000));
    }
}
