# Storage Systems Documentation

## Overview

The PolyTorus storage system provides multiple storage backends for smart contract state, metadata, and execution history. The system is designed for flexibility, performance, and enterprise-grade reliability.

## Storage Architecture

### Core Interface

All storage implementations adhere to the `ContractStateStorage` trait:

```rust
pub trait ContractStateStorage {
    // Contract metadata management
    fn store_contract_metadata(&self, metadata: &UnifiedContractMetadata) -> Result<()>;
    fn get_contract_metadata(&self, address: &str) -> Result<Option<UnifiedContractMetadata>>;
    
    // Contract state management
    fn set_contract_state(&self, contract: &str, key: &str, value: &[u8]) -> Result<()>;
    fn get_contract_state(&self, contract: &str, key: &str) -> Result<Option<Vec<u8>>>;
    fn delete_contract_state(&self, contract: &str, key: &str) -> Result<()>;
    
    // Contract discovery
    fn list_contracts(&self) -> Result<Vec<String>>;
    
    // Execution history
    fn store_execution(&self, execution: &ContractExecutionRecord) -> Result<()>;
    fn get_execution_history(&self, contract: &str) -> Result<Vec<ContractExecutionRecord>>;
}
```

## Storage Implementations

### 1. UnifiedContractStorage (Production Ready)

**Technology**: Sled embedded database with memory caching
**Use Case**: Production deployments requiring persistence without external dependencies

#### Features
- **Persistent Storage**: Survives application restarts
- **Memory Caching**: Async-aware LRU caching for performance
- **Multi-tree Architecture**: Separate trees for contracts, state, and history
- **Statistics Tracking**: Database size and operation metrics
- **Compaction**: Automatic database optimization

#### Configuration
```rust
let storage = UnifiedContractStorage::new("/path/to/database")?;

// Get storage statistics
let stats = storage.get_stats()?;
println!("Database size: {} bytes", stats.db_size_bytes);
println!("Contracts: {}", stats.contracts_count);
println!("State entries: {}", stats.state_entries);

// Manual compaction
storage.compact().await?;
```

#### Performance Characteristics
- **Read Performance**: O(log n) with memory cache acceleration
- **Write Performance**: O(log n) with write-ahead logging
- **Memory Usage**: Configurable cache with automatic eviction
- **Disk Usage**: Efficient compression and compaction

#### File Structure
```
database/
├── conf           # Database configuration
├── db/            # Main database files
│   ├── contracts  # Contract metadata tree
│   ├── contract_state # Contract state tree
│   └── execution_history # Execution history tree
├── snapshots/     # Point-in-time snapshots
└── logs/          # Write-ahead logs
```

### 2. DatabaseContractStorage (Enterprise Grade)

**Technology**: PostgreSQL + Redis with memory fallback
**Use Case**: Enterprise deployments requiring scalability and high availability

#### Features
- **PostgreSQL Integration**: Relational data with ACID guarantees
- **Redis Caching**: Sub-millisecond read performance
- **Connection Pooling**: Efficient resource utilization
- **Fallback Mechanism**: Automatic degradation to memory storage
- **Statistics Tracking**: Connection health and query metrics

#### Configuration
```rust
let config = DatabaseStorageConfig {
    postgres: Some(PostgresConfig {
        host: "localhost".to_string(),
        port: 5432,
        database: "polytorus".to_string(),
        username: "polytorus_user".to_string(),
        password: "secure_password".to_string(),
        schema: "smart_contracts".to_string(),
        max_connections: 20,
    }),
    redis: Some(RedisConfig {
        url: "redis://localhost:6379".to_string(),
        password: Some("redis_password".to_string()),
        database: 0,
        max_connections: 20,
        key_prefix: "polytorus:contracts:".to_string(),
        ttl_seconds: Some(3600), // 1 hour TTL
    }),
    fallback_to_memory: true,
    connection_timeout_secs: 30,
    max_connections: 20,
    use_ssl: true,
};

let storage = DatabaseContractStorage::new(config).await?;
```

#### Database Schema (PostgreSQL)

```sql
-- Contract metadata table
CREATE TABLE contracts (
    address VARCHAR(42) PRIMARY KEY,
    metadata JSONB NOT NULL,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW()
);

-- Contract state table
CREATE TABLE contract_state (
    id SERIAL PRIMARY KEY,
    contract_address VARCHAR(42) NOT NULL,
    state_key VARCHAR(255) NOT NULL,
    state_value BYTEA NOT NULL,
    created_at TIMESTAMP DEFAULT NOW(),
    updated_at TIMESTAMP DEFAULT NOW(),
    UNIQUE(contract_address, state_key)
);

-- Execution history table
CREATE TABLE execution_history (
    id SERIAL PRIMARY KEY,
    execution_id VARCHAR(36) NOT NULL,
    contract_address VARCHAR(42) NOT NULL,
    function_name VARCHAR(255) NOT NULL,
    caller VARCHAR(42) NOT NULL,
    timestamp BIGINT NOT NULL,
    gas_used BIGINT NOT NULL,
    success BOOLEAN NOT NULL,
    error_message TEXT,
    created_at TIMESTAMP DEFAULT NOW()
);

-- Indexes for performance
CREATE INDEX idx_contracts_address ON contracts(address);
CREATE INDEX idx_state_contract ON contract_state(contract_address);
CREATE INDEX idx_state_key ON contract_state(contract_address, state_key);
CREATE INDEX idx_history_contract ON execution_history(contract_address);
CREATE INDEX idx_history_timestamp ON execution_history(timestamp);
```

#### Redis Key Structure
```
polytorus:contracts:state:{contract}:{key}  # Contract state
polytorus:contracts:contract:{address}      # Contract metadata
polytorus:contracts:stats                   # Connection statistics
```

#### Performance Characteristics
- **PostgreSQL**: Excellent for complex queries and ACID compliance
- **Redis**: Sub-millisecond reads for frequently accessed data
- **Combined**: Best of both worlds with intelligent caching
- **Fallback**: Graceful degradation maintains availability

### 3. InMemoryContractStorage (Development)

**Technology**: HashMap-based with async/sync variants
**Use Case**: Development, testing, and lightweight deployments

#### Features
- **Zero Dependencies**: No external database required
- **Async/Sync Variants**: Compatible with different runtime environments
- **Thread Safety**: Proper synchronization for concurrent access
- **Fast Performance**: Direct memory access

#### Variants

1. **SyncInMemoryContractStorage**: Synchronous operations
```rust
let storage = SyncInMemoryContractStorage::new();
// Thread-safe with std::sync::RwLock
```

2. **InMemoryContractStorage**: Asynchronous operations
```rust
let storage = InMemoryContractStorage::new();
// Async-compatible with tokio::sync::RwLock
```

#### Performance Characteristics
- **Read/Write**: O(1) average case with HashMap
- **Memory Usage**: Linear with data size
- **Concurrency**: High with reader-writer locks
- **Persistence**: None (data lost on restart)

## Deployment Strategies

### Development Environment
```rust
// Quick setup for development
let manager = UnifiedContractManager::in_memory()?;
```

### Staging Environment
```rust
// Persistent storage for testing
let storage = Arc::new(UnifiedContractStorage::new("./staging_db")?);
let manager = UnifiedContractManager::new(
    storage,
    UnifiedGasManager::new(UnifiedGasConfig::default()),
    PrivacyEngineConfig::testing(),
)?;
```

### Production Environment
```rust
// Enterprise setup with database backend
let db_config = DatabaseStorageConfig::production();
let storage = Arc::new(DatabaseContractStorage::new(db_config).await?);
let manager = UnifiedContractManager::new(
    storage,
    UnifiedGasManager::new(UnifiedGasConfig::production()),
    PrivacyEngineConfig::production(),
)?;
```

## Performance Optimization

### Caching Strategy

1. **Multi-Level Caching**
   - L1: Memory cache (fastest)
   - L2: Redis cache (fast network)
   - L3: PostgreSQL (persistent)

2. **Cache Invalidation**
   - TTL-based expiration
   - Manual invalidation on updates
   - LRU eviction for memory management

3. **Cache Warming**
   - Preload frequently accessed contracts
   - Background cache population
   - Predictive caching based on patterns

### Database Optimization

1. **Connection Pooling**
   - Reuse existing connections
   - Configurable pool sizes
   - Health monitoring

2. **Query Optimization**
   - Proper indexing strategy
   - Query plan analysis
   - Batch operations where possible

3. **Partitioning**
   - Time-based partitioning for history
   - Hash partitioning for large datasets
   - Archive old data automatically

## Monitoring and Maintenance

### Health Monitoring

```rust
// Check storage health
let stats = storage.get_stats().await?;
println!("Storage Health Report:");
println!("- Total contracts: {}", stats.contracts_count);
println!("- State entries: {}", stats.state_entries);
println!("- History entries: {}", stats.history_entries);
println!("- Database size: {} MB", stats.db_size_bytes / 1024 / 1024);

// Database-specific metrics
if let Some(db_storage) = storage.as_any().downcast_ref::<DatabaseContractStorage>() {
    let connection_stats = db_storage.get_stats().await;
    println!("- PostgreSQL connections: {}", connection_stats.postgres_connections);
    println!("- Redis connections: {}", connection_stats.redis_connections);
    println!("- Cache hit rate: {:.2}%", 
        connection_stats.cache_hits as f64 / 
        (connection_stats.cache_hits + connection_stats.cache_misses) as f64 * 100.0
    );
}
```

### Backup and Recovery

#### Sled Database Backup
```bash
# Create backup
cp -r /path/to/database /path/to/backup/$(date +%Y%m%d_%H%M%S)

# Restore from backup
rm -rf /path/to/database
cp -r /path/to/backup/20240101_120000 /path/to/database
```

#### PostgreSQL Backup
```bash
# Create backup
pg_dump -h localhost -U polytorus_user polytorus > backup_$(date +%Y%m%d_%H%M%S).sql

# Restore from backup
psql -h localhost -U polytorus_user polytorus < backup_20240101_120000.sql
```

#### Redis Backup
```bash
# Redis persistence (RDB snapshots)
redis-cli BGSAVE

# Copy snapshot
cp /var/lib/redis/dump.rdb /backup/redis_$(date +%Y%m%d_%H%M%S).rdb
```

## Migration Guide

### From In-Memory to Sled

```rust
// Export from in-memory storage
let in_memory = SyncInMemoryContractStorage::new();
let contracts = in_memory.list_contracts()?;

// Create Sled storage
let sled_storage = UnifiedContractStorage::new("./migrated_db")?;

// Migrate contracts
for contract_address in contracts {
    // Migrate metadata
    if let Some(metadata) = in_memory.get_contract_metadata(&contract_address)? {
        sled_storage.store_contract_metadata(&metadata)?;
    }
    
    // Migrate state (implementation depends on state structure)
    // This would require iterating through all state keys
    
    // Migrate execution history
    let history = in_memory.get_execution_history(&contract_address)?;
    for execution in history {
        sled_storage.store_execution(&execution)?;
    }
}
```

### From Sled to PostgreSQL

```rust
// Read from Sled
let sled_storage = UnifiedContractStorage::new("./existing_db")?;
let contracts = sled_storage.list_contracts()?;

// Setup PostgreSQL
let pg_config = DatabaseStorageConfig::production();
let pg_storage = DatabaseContractStorage::new(pg_config).await?;

// Migrate all data
for contract_address in contracts {
    // Same migration pattern as above
}
```

## Security Considerations

### Access Control

1. **Database Permissions**
   - Principle of least privilege
   - Separate read/write users
   - Network access restrictions

2. **Connection Security**
   - SSL/TLS encryption
   - Certificate validation
   - Secure password storage

3. **Data Protection**
   - Encrypted storage at rest
   - Secure key management
   - Regular security audits

### Configuration Security

```rust
// Secure configuration loading
let config = DatabaseStorageConfig {
    postgres: Some(PostgresConfig {
        // Load from environment variables
        host: env::var("POSTGRES_HOST").unwrap_or_else(|_| "localhost".to_string()),
        username: env::var("POSTGRES_USER").expect("POSTGRES_USER not set"),
        password: env::var("POSTGRES_PASSWORD").expect("POSTGRES_PASSWORD not set"),
        // ... other settings
    }),
    // Enable SSL in production
    use_ssl: env::var("ENVIRONMENT") == Ok("production".to_string()),
    // ... other settings
};
```

## Troubleshooting

### Common Issues

1. **Connection Timeouts**
   - Check network connectivity
   - Verify database server status
   - Adjust timeout settings

2. **High Memory Usage**
   - Reduce cache sizes
   - Implement memory limits
   - Monitor for memory leaks

3. **Slow Queries**
   - Analyze query patterns
   - Add missing indexes
   - Optimize data access patterns

4. **Cache Misses**
   - Verify cache configuration
   - Check TTL settings
   - Monitor cache hit rates

### Debugging Tools

```rust
// Enable debug logging
env::set_var("RUST_LOG", "polytorus::smart_contract::database_storage=debug");

// Storage health check
let health = storage.health_check().await?;
if !health.is_healthy {
    eprintln!("Storage health issues: {:?}", health.issues);
}

// Performance profiling
let start = Instant::now();
let result = storage.get_contract_state("0x123", "balance")?;
println!("Query took: {:?}", start.elapsed());
```

## Best Practices

### Development
- Use in-memory storage for unit tests
- Use Sled for integration tests
- Mock external dependencies

### Staging
- Use production-like database setup
- Test migration procedures
- Validate backup/restore processes

### Production
- Enable all monitoring
- Use connection pooling
- Implement proper backup strategy
- Regular maintenance windows

### Performance
- Monitor cache hit rates (target >80%)
- Set appropriate TTL values
- Use batch operations for bulk data
- Regular database maintenance