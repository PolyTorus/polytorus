# PolyTorus Configuration Guide

## Overview
PolyTorus uses a flexible configuration system supporting both TOML files and environment variables for maximum deployment flexibility, especially in containerized environments.

## Latest Updates (June 2025)
- ✅ **Environment Variable Support** - Full configuration via environment variables
- ✅ **Docker Secrets Integration** - Secure secret management in Docker environments
- ✅ **Flexible Configuration** - TOML files, environment variables, and Docker secrets
- ✅ **Development vs Production** - Separate configurations for different environments
- ✅ **Database Configuration** - Support for PostgreSQL, Redis, and SQLite

## Configuration Methods

### 1. TOML Configuration Files
Traditional configuration file approach:

```bash
# Configuration file priority:
1. --config command line argument
2. POLYTORUS_CONFIG_PATH environment variable
3. ./config.toml in current directory
4. ~/.polytorus/config.toml in home directory
```

### 2. Environment Variables
Full configuration support via environment variables:

```bash
# Network configuration
export POLYTORUS_NETWORK_TYPE=mainnet
export POLYTORUS_NETWORK_PORT=8333
export POLYTORUS_NETWORK_BIND_ADDRESS=0.0.0.0

# Database configuration
export DATABASE_URL=postgres://user:pass@localhost/polytorus
export REDIS_URL=redis://localhost:6379

# Mining configuration
export POLYTORUS_MINING_ENABLED=true
export POLYTORUS_MINING_THREADS=4
```

### 3. Docker Secrets (Production)
Secure secret management in Docker environments:

```bash
# Docker secrets are automatically loaded from:
/run/secrets/database_password
/run/secrets/redis_password
/run/secrets/api_key
```

## Environment Variable Reference

### Database Configuration
```bash
# Primary database
DATABASE_URL=postgres://user:password@host:port/database
DATABASE_MAX_CONNECTIONS=10
DATABASE_MIN_CONNECTIONS=1

# Redis configuration
REDIS_URL=redis://host:port
REDIS_MAX_CONNECTIONS=10
REDIS_TIMEOUT=5

# SQLite fallback
SQLITE_DATABASE_PATH=./data/polytorus.db
```

### Network Configuration
```bash
POLYTORUS_NETWORK_TYPE=mainnet|testnet|development
POLYTORUS_NETWORK_PORT=8333
POLYTORUS_NETWORK_BIND_ADDRESS=0.0.0.0
POLYTORUS_NETWORK_MAX_PEERS=50
POLYTORUS_NETWORK_MIN_PEERS=3
```

### Mining Configuration
```bash
POLYTORUS_MINING_ENABLED=true|false
POLYTORUS_MINING_THREADS=4
POLYTORUS_MINING_DIFFICULTY_TARGET=0x1d00ffff
POLYTORUS_MINING_REWARD=50
```

### Logging Configuration
```bash
RUST_LOG=info|debug|trace
POLYTORUS_LOG_LEVEL=info
POLYTORUS_LOG_FILE=/var/log/polytorus.log
```

## Docker Configuration

### Development Environment
Create `.env` file for development:

```bash
# .env (development)
RUST_LOG=debug
DATABASE_URL=postgres://postgres:password@db:5432/polytorus_dev
REDIS_URL=redis://redis:6379
POLYTORUS_NETWORK_TYPE=development
POLYTORUS_MINING_ENABLED=true
```

### Production Environment
Create `.env.secrets` file for production:

```bash
# .env.secrets (production - never commit to git)
DATABASE_URL=postgres://user:secure_password@db:5432/polytorus
REDIS_URL=redis://:secure_password@redis:6379
POLYTORUS_API_KEY=your_secure_api_key
POLYTORUS_NETWORK_TYPE=mainnet
```

## Configuration File Location
By default, PolyTorus looks for configuration files in the following order:
1. `--config` command line argument
2. `POLYTORUS_CONFIG_PATH` environment variable
3. `./config.toml` in the current directory
4. `~/.polytorus/config.toml` in the user's home directory

## Complete Configuration Reference

### Basic Configuration Template
```toml
# PolyTorus Configuration File
# Generated on: 2025-06-05

[network]
# Network configuration
type = "mainnet"  # mainnet, testnet, development
port = 8333
bind_address = "0.0.0.0"
max_peers = 50
min_peers = 3
peer_discovery_timeout = 30
bootstrap_peers = [
    "node1.polytorus.network:8333",
    "node2.polytorus.network:8333",
    "node3.polytorus.network:8333"
]

[network.timeouts]
connection_timeout = 10
handshake_timeout = 30
ping_interval = 60
peer_timeout = 300

[blockchain]
# Blockchain parameters
data_dir = "./data/blockchain"
cache_size = 1000
reorg_limit = 100
checkpoint_interval = 1000

[blockchain.genesis]
# Genesis block configuration (only for new networks)
timestamp = 1672531200000
difficulty = 1
coinbase_reward = 5000000000
coinbase_address = "genesis_address"

[mining]
# Mining configuration
enabled = false
address = ""
threads = 0  # 0 = auto-detect
intensity = "medium"  # low, medium, high
cache_enabled = true
stats_interval = 10

[mining.difficulty]
# Difficulty adjustment parameters
base_difficulty = 4
min_difficulty = 1
max_difficulty = 32
adjustment_factor = 0.25
tolerance_percentage = 20.0
retarget_blocks = 10

[wallet]
# Wallet management
data_dir = "./data/wallets"
default_wallet = ""
encryption_enabled = true
backup_enabled = true
backup_interval = 3600

[wallet.security]
password_min_length = 8
session_timeout = 1800
max_failed_attempts = 5
lockout_duration = 300

[api]
# REST API configuration
enabled = true
port = 8000
bind_address = "127.0.0.1"
cors_enabled = true
cors_origins = ["*"]
rate_limit_enabled = true
rate_limit_requests = 100
rate_limit_window = 60

[api.authentication]
enabled = false
jwt_secret = "your_jwt_secret_here"
token_expiry = 3600

[websocket]
# WebSocket configuration
enabled = true
port = 8001
max_connections = 100
ping_interval = 30
pong_timeout = 10

[database]
# Database configuration
engine = "sled"  # sled, rocksdb
path = "./data/db"
cache_size = 64  # MB
compression = true
sync_writes = true

[database.maintenance]
auto_compact = true
compact_interval = 86400  # 24 hours
backup_enabled = true
backup_retention = 7  # days

[smart_contracts]
# Smart contract engine configuration
enabled = true
gas_limit_default = 1000000
gas_price_default = 1
max_contract_size = 1048576  # 1MB
execution_timeout = 30

[smart_contracts.wasm]
memory_limit = 268435456  # 256MB
stack_limit = 65536
fuel_limit = 1000000

[logging]
# Logging configuration
level = "info"  # trace, debug, info, warn, error
format = "full"  # full, compact, json
color = true
file_enabled = true
file_path = "./logs/polytorus.log"
file_rotation = "daily"
file_retention = 30

[logging.modules]
# Per-module log levels
blockchain = "info"
network = "info"
mining = "debug"
smart_contracts = "info"
api = "warn"

[performance]
# Performance tuning
worker_threads = 0  # 0 = auto-detect
blocking_threads = 512
max_memory_usage = 2147483648  # 2GB
gc_interval = 300

[security]
# Security settings
rpc_whitelist = ["127.0.0.1", "::1"]
max_request_size = 1048576  # 1MB
request_timeout = 30
ddos_protection = true

[monitoring]
# Monitoring and metrics
enabled = true
prometheus_enabled = true
prometheus_port = 9090
health_check_interval = 60
```

## Network-Specific Configurations

### Mainnet Configuration
```toml
[network]
type = "mainnet"
port = 8333
bootstrap_peers = [
    "mainnet-node1.polytorus.network:8333",
    "mainnet-node2.polytorus.network:8333"
]

[blockchain]
data_dir = "./data/mainnet"

[mining.difficulty]
base_difficulty = 16
min_difficulty = 4
max_difficulty = 256
```

### Testnet Configuration
```toml
[network]
type = "testnet"
port = 18333
bootstrap_peers = [
    "testnet-node1.polytorus.network:18333",
    "testnet-node2.polytorus.network:18333"
]

[blockchain]
data_dir = "./data/testnet"

[mining.difficulty]
base_difficulty = 4
min_difficulty = 1
max_difficulty = 32
```

### Development Configuration
```toml
[network]
type = "development"
port = 28333
bootstrap_peers = []
max_peers = 5

[blockchain]
data_dir = "./data/development"

[mining]
enabled = true
threads = 1

[mining.difficulty]
base_difficulty = 1
min_difficulty = 1
max_difficulty = 4

[logging]
level = "debug"
```

## Environment Variables

### Core Settings
```bash
# Configuration file path
export POLYTORUS_CONFIG_PATH="/path/to/config.toml"

# Data directory
export POLYTORUS_DATA_DIR="/path/to/data"

# Network type
export POLYTORUS_NETWORK="mainnet"

# Log level
export POLYTORUS_LOG_LEVEL="info"
export RUST_LOG="polytorus=debug"
```

### Mining Settings
```bash
# Mining configuration
export POLYTORUS_MINING_ENABLED="true"
export POLYTORUS_MINING_ADDRESS="your_mining_address"
export POLYTORUS_MINING_THREADS="4"
```

### API Settings
```bash
# API configuration
export POLYTORUS_API_ENABLED="true"
export POLYTORUS_API_PORT="8000"
export POLYTORUS_API_BIND="127.0.0.1"
```

## Configuration Validation

### Validate Configuration File
```bash
# Validate configuration syntax
polytorus config validate --config config.toml

# Show parsed configuration
polytorus config show --config config.toml

# Generate sample configuration
polytorus config generate --output sample-config.toml
```

### Configuration Errors and Solutions

#### Common Configuration Errors
```toml
# ERROR: Invalid port number
[network]
port = 99999  # Port must be between 1-65535

# ERROR: Invalid log level
[logging]
level = "verbose"  # Must be: trace, debug, info, warn, error

# ERROR: Invalid network type
[network]
type = "custom"  # Must be: mainnet, testnet, development
```

#### Fixing Configuration Issues
```bash
# Check configuration syntax
polytorus config validate --config config.toml

# Reset to default configuration
polytorus config generate --output config.toml --force

# Migrate old configuration format
polytorus config migrate --input old-config.toml --output new-config.toml
```

## Advanced Configuration

### Custom Network Configuration
```toml
[network.custom]
name = "private_network"
magic_bytes = [0x12, 0x34, 0x56, 0x78]
genesis_hash = "custom_genesis_hash"
port = 9333
bootstrap_peers = ["192.168.1.100:9333"]
```

### Load Balancing Configuration
```toml
[network.load_balancing]
enabled = true
strategy = "round_robin"  # round_robin, least_connections, random
health_check_interval = 30
max_retries = 3
```

### Backup Configuration
```toml
[backup]
enabled = true
interval = 3600  # seconds
retention_days = 30
compression = true
remote_backup = true

[backup.remote]
type = "s3"  # s3, ftp, sftp
endpoint = "s3.amazonaws.com"
bucket = "polytorus-backups"
access_key = "your_access_key"
secret_key = "your_secret_key"
```

### Cluster Configuration
```toml
[cluster]
enabled = true
node_id = "node_001"
cluster_name = "polytorus_cluster"
discovery_service = "consul://localhost:8500"
heartbeat_interval = 10
failover_timeout = 30
```

## Performance Tuning

### High-Performance Configuration
```toml
[performance]
# Optimize for high throughput
worker_threads = 16
blocking_threads = 1024
max_memory_usage = 8589934592  # 8GB

[database]
cache_size = 512  # MB
compression = false  # Disable for speed
sync_writes = false  # Async writes for performance

[mining]
threads = 8
intensity = "high"
cache_enabled = true
```

### Low-Resource Configuration
```toml
[performance]
# Optimize for low resource usage
worker_threads = 2
blocking_threads = 64
max_memory_usage = 536870912  # 512MB

[database]
cache_size = 16  # MB
compression = true  # Enable compression to save space

[network]
max_peers = 10
```

## Configuration Management

### Configuration Profiles
```bash
# Use different profiles for different environments
polytorus --config configs/development.toml node start
polytorus --config configs/staging.toml node start
polytorus --config configs/production.toml node start
```

### Configuration Templates
```bash
# Generate configuration for specific use cases
polytorus config generate --template mining --output mining-config.toml
polytorus config generate --template api-server --output api-config.toml
polytorus config generate --template full-node --output fullnode-config.toml
```

### Dynamic Configuration Updates
```bash
# Update configuration without restart (limited settings)
polytorus config update --key logging.level --value debug
polytorus config update --key api.rate_limit_requests --value 200

# Reload configuration
polytorus config reload
```

## Security Considerations

### Secure Configuration
```toml
[security]
# Enable security features
rpc_whitelist = ["127.0.0.1"]  # Restrict API access
max_request_size = 1048576     # Limit request size
ddos_protection = true         # Enable DDoS protection

[api.authentication]
enabled = true
jwt_secret = "your_secure_jwt_secret_here"

[wallet.security]
password_min_length = 12
session_timeout = 900  # 15 minutes
```

### File Permissions
```bash
# Set secure file permissions
chmod 600 config.toml
chmod 700 data/
chmod 700 logs/
```

## Monitoring Configuration

### Metrics and Monitoring
```toml
[monitoring]
enabled = true
prometheus_enabled = true
prometheus_port = 9090
metrics_interval = 10

[monitoring.alerts]
enabled = true
webhook_url = "https://your-webhook-url.com"
alert_threshold_cpu = 80
alert_threshold_memory = 80
alert_threshold_disk = 90
```

### Health Checks
```toml
[health_check]
enabled = true
port = 8080
endpoint = "/health"
interval = 30
timeout = 10
```

## Troubleshooting Configuration

### Configuration Debugging
```bash
# Enable configuration debugging
RUST_LOG=polytorus::config=debug polytorus node start

# Validate configuration with verbose output
polytorus config validate --config config.toml --verbose

# Show effective configuration (after environment variable overrides)
polytorus config show --effective
```

### Common Issues
1. **Port conflicts**: Ensure ports are not already in use
2. **File permissions**: Check that data directories are writable
3. **Network connectivity**: Verify bootstrap peers are reachable
4. **Resource limits**: Ensure system has sufficient resources

For more detailed troubleshooting, see the [Getting Started Guide](GETTING_STARTED.md#troubleshooting).
