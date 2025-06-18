# PolyTorus Modular Blockchain - Network Operations Guide

## Overview

This guide provides comprehensive instructions for operating the PolyTorus modular blockchain network, including multi-node deployments, P2P networking, and data availability layer testing.

## Prerequisites

### System Requirements
- **Rust**: 1.87 nightly or later
- **OpenFHE**: MachinaIO fork with `feat/improve_determinant` branch
- **System Libraries**: `cmake`, `libgmp-dev`, `libntl-dev`, `libboost-all-dev`
- **Operating System**: Linux/macOS/WSL2

### Environment Setup
```bash
# Set required environment variables
export OPENFHE_ROOT=/usr/local
export LD_LIBRARY_PATH=/usr/local/lib:$LD_LIBRARY_PATH
export PKG_CONFIG_PATH=/usr/local/lib/pkgconfig:$PKG_CONFIG_PATH
```

## Building the Project

### Standard Build
```bash
# Development build
cargo build

# Release build (recommended for testing)
cargo build --release
```

### Testing
```bash
# Run library tests
cargo test --lib

# Run data availability tests
cargo test data_availability --lib -- --nocapture

# Run complete test suite
cargo test

# Quality checks
cargo clippy --lib -- -D warnings
cargo fmt
```

## Configuration Files

### Node Configuration Templates

The project includes pre-configured node templates in the `config/` directory:

- `modular-node1.toml` - Bootstrap node (127.0.0.1:7001)
- `modular-node2.toml` - Peer node (127.0.0.1:7002)
- `modular-node3.toml` - Peer node (127.0.0.1:7003)

### Creating Custom Node Configurations

```toml
# Example: config/custom-node.toml
[execution]
gas_limit = 8000000
gas_price = 1

[execution.wasm_config]
max_memory_pages = 256
max_stack_size = 65536
gas_metering = true

[settlement]
challenge_period = 100
batch_size = 100
min_validator_stake = 1000

[consensus]
block_time = 10000
difficulty = 4
max_block_size = 1048576

[data_availability]
retention_period = 604800  # 7 days
max_data_size = 1048576    # 1MB

[data_availability.network_config]
listen_addr = "127.0.0.1:7004"
bootstrap_peers = ["127.0.0.1:7001"]
max_peers = 50
```

## Single Node Operations

### Starting a Single Node
```bash
# Start with default configuration
./target/release/polytorus --modular-start

# Start with custom configuration and data directory
./target/release/polytorus \
  --config config/modular-node1.toml \
  --data-dir data/node1 \
  --modular-start

# Start with HTTP API enabled
./target/release/polytorus \
  --config config/modular-node1.toml \
  --data-dir data/node1 \
  --http-port 9001 \
  --modular-start
```

### Node Status and Management
```bash
# Check node status
./target/release/polytorus --modular-status

# View configuration
./target/release/polytorus --modular-config

# Initialize modular architecture
./target/release/polytorus --modular-init
```

## Multi-Node Network Operations

### Manual Multi-Node Setup

#### Step 1: Start Bootstrap Node
```bash
# Terminal 1 - Bootstrap Node
./target/release/polytorus \
  --config config/modular-node1.toml \
  --data-dir data/node1 \
  --modular-start
```

#### Step 2: Start Peer Nodes
```bash
# Terminal 2 - Peer Node 2
./target/release/polytorus \
  --config config/modular-node2.toml \
  --data-dir data/node2 \
  --modular-start

# Terminal 3 - Peer Node 3
./target/release/polytorus \
  --config config/modular-node3.toml \
  --data-dir data/node3 \
  --modular-start
```

### Automated Multi-Node Testing

#### Using the Network Test Script
```bash
# Make script executable
chmod +x test_network.sh

# Run 3-node network test
./test_network.sh
```

#### Script Functionality
- Starts 3 nodes with bootstrap configuration
- Runs for 30 seconds to establish connections
- Collects logs from all nodes
- Automatically shuts down all nodes

### Advanced Multi-Node Scenarios

#### 4-Node Network with Custom Ports
```bash
# Node 1 (Bootstrap)
./target/release/polytorus --config config/node-7001.toml --data-dir data/node1 --modular-start &

# Node 2
./target/release/polytorus --config config/node-7002.toml --data-dir data/node2 --modular-start &

# Node 3
./target/release/polytorus --config config/node-7003.toml --data-dir data/node3 --modular-start &

# Node 4
./target/release/polytorus --config config/node-7004.toml --data-dir data/node4 --modular-start &
```

## Network Monitoring and Diagnostics

### Log Analysis
```bash
# Real-time node monitoring
tail -f logs/node1.log

# Search for network events
grep "P2P\|network\|peer" logs/node1.log

# Check for errors
grep "ERROR\|WARN" logs/*.log
```

### Network Health Checks
```bash
# Check network status
./target/release/polytorus --network-status

# View connected peers
./target/release/polytorus --network-peers

# Network health information
./target/release/polytorus --network-health

# Message queue statistics
./target/release/polytorus --network-queue-stats
```

### Process Management
```bash
# Check running nodes
ps aux | grep polytorus

# Stop all nodes
pkill -f "polytorus.*modular"

# Monitor system resources
htop -p $(pgrep -f polytorus)
```

## Data Availability Layer Operations

### Testing Data Storage and Retrieval
```bash
# Run data availability tests
cargo test data_availability --lib -- --nocapture

# Test specific functionality
cargo test merkle_proof_generation_and_verification --lib -- --nocapture
cargo test replication_status_tracking --lib -- --nocapture
```

### Data Verification Features
The data availability layer includes:
- **Real Merkle Proof Generation**: Actual merkle tree construction
- **Comprehensive Data Verification**: Hash validation, checksum integrity
- **Network Replication Tracking**: Distributed availability verification
- **Verification Caching**: Performance optimization for repeated checks

## Wallet and Transaction Operations

### Wallet Management
```bash
# Create new wallet
./target/release/polytorus --createwallet

# List wallet addresses
./target/release/polytorus --listaddresses

# Check balance
./target/release/polytorus --getbalance <ADDRESS>
```

### Mining Operations
```bash
# Mine blocks using modular architecture
./target/release/polytorus modular mine <address>

# Start mining with specific configuration
./target/release/polytorus \
  --config config/mining-node.toml \
  --data-dir data/miner \
  --modular-start
```

## Smart Contract Operations

### ERC20 Token Management
```bash
# Deploy ERC20 contract
./target/release/polytorus --erc20-deploy "MyToken,MTK,18,1000000,owner_address"

# Transfer tokens
./target/release/polytorus --erc20-transfer "contract_address,recipient,amount"

# Check balance
./target/release/polytorus --erc20-balance "contract_address,address"

# List all contracts
./target/release/polytorus --erc20-list
```

### Smart Contract Deployment
```bash
# Deploy custom contract
./target/release/polytorus --smart-contract-deploy path/to/contract.wasm

# Call contract function
./target/release/polytorus --smart-contract-call contract_address
```

## Troubleshooting

### Common Issues and Solutions

#### 1. Node Startup Failures
```bash
# Check configuration file syntax
cat config/modular-node1.toml

# Verify data directory permissions
ls -la data/

# Check port availability
netstat -tuln | grep :7001
```

#### 2. P2P Connection Issues
```bash
# Check network configuration
./target/release/polytorus --modular-config

# Verify bootstrap peer connectivity
telnet 127.0.0.1 7001

# Check firewall settings
sudo ufw status
```

#### 3. Data Availability Errors
```bash
# Run diagnostic tests
cargo test data_availability --lib

# Check storage stats
grep "Storage stats" logs/node*.log

# Verify merkle proof functionality
cargo test merkle_proof --lib -- --nocapture
```

### Debug Logging
```bash
# Enable debug logging
RUST_LOG=debug ./target/release/polytorus --modular-start

# Module-specific logging
RUST_LOG=polytorus::modular::network=debug ./target/release/polytorus --modular-start

# Network-only logging
RUST_LOG=polytorus::modular::network=trace ./target/release/polytorus --modular-start
```

## Performance Optimization

### Resource Monitoring
```bash
# Monitor memory usage
ps -o pid,vsz,rss,comm -p $(pgrep polytorus)

# Check disk usage
du -sh data/

# Network bandwidth monitoring
iftop -i lo  # For localhost testing
```

### Configuration Tuning
```toml
# High-performance configuration
[data_availability]
retention_period = 86400     # 1 day for testing
max_data_size = 10485760     # 10MB

[data_availability.network_config]
max_peers = 100
```

## Security Considerations

### Network Security
- Use firewall rules to restrict access to P2P ports
- Configure bootstrap peers carefully in production
- Monitor for unusual network activity

### Data Integrity
- The data availability layer includes comprehensive verification
- Merkle proofs ensure data integrity across the network
- Checksums validate data during retrieval

### Access Control
- Wallet files are encrypted by default
- Smart contract execution is sandboxed
- Network communication uses secure channels

## Production Deployment

### Recommended Architecture
```
Internet
    |
Load Balancer (Port 80/443)
    |
+-- Node 1 (Bootstrap) - Port 7001
+-- Node 2 (Peer)      - Port 7002
+-- Node 3 (Peer)      - Port 7003
+-- Node 4 (Peer)      - Port 7004
```

### Deployment Checklist
- [ ] OpenFHE properly installed and configured
- [ ] Environment variables set correctly
- [ ] Configuration files validated
- [ ] Data directories with proper permissions
- [ ] Network ports accessible
- [ ] Monitoring and logging configured
- [ ] Backup and recovery procedures in place

## API Reference

### HTTP API Endpoints (when enabled)
```
GET  /status     - Node status information
GET  /stats      - Performance statistics
GET  /health     - Health check endpoint
POST /transaction - Submit transaction
POST /send       - Send transaction
```

### CLI Command Reference
```
--modular-start        Start modular blockchain with P2P network
--modular-status       Show modular system status
--modular-config       Show modular configuration
--createwallet         Create a new wallet
--listaddresses        List all wallet addresses
--network-status       Show network status
--network-peers        List connected peers
--erc20-deploy         Deploy ERC20 token contract
--erc20-list          List deployed contracts
```

## Support and Maintenance

### Log Rotation
```bash
# Rotate logs daily
logrotate -f polytorus-logrotate.conf
```

### Database Maintenance
```bash
# Cleanup old data
find data/ -name "*.log" -mtime +7 -delete

# Compact database
./target/release/polytorus --data-dir data/node1 --compact-db
```

### Updates and Upgrades
```bash
# Update to latest version
git pull origin main
cargo build --release

# Run migration if needed
./target/release/polytorus --migrate --data-dir data/node1
```

---

## Conclusion

This guide provides comprehensive instructions for operating the PolyTorus modular blockchain network. The platform's modular architecture allows for flexible deployment scenarios, from single-node testing to multi-node production environments.

For additional support or advanced configurations, refer to the project documentation in `/docs` or the test implementations in `/tests`.