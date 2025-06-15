# Getting Started with PolyTorus

## Overview
PolyTorus is a modular blockchain platform built in Rust that supports smart contracts, dynamic difficulty adjustment, and a type-safe architecture. This guide will help you get started with setting up, running, and using PolyTorus.

## Prerequisites

### System Requirements
- **Operating System**: Linux, macOS, or Windows
- **Memory**: At least 4GB RAM (8GB recommended)
- **Storage**: At least 10GB free space
- **Network**: Internet connection for peer discovery

### Software Dependencies
- **Rust**: Version 1.70 or later
- **Git**: For cloning the repository
- **OpenSSL**: For cryptographic operations

### Installing Rust
```bash
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env
rustup update
```

### Installing Additional Dependencies

#### Ubuntu/Debian
```bash
sudo apt update
sudo apt install build-essential pkg-config libssl-dev git
```

#### macOS
```bash
brew install openssl pkg-config
```

#### Windows
Install Visual Studio Build Tools and Git for Windows.

## Installation

### Method 1: Clone and Build from Source
```bash
# Clone the repository
git clone https://github.com/quantumshiro/polytorus.git
cd polytorus

# Build the project
cargo build --release

# The binary will be available at target/release/polytorus
```

### Method 2: Install from Crates.io (when available)
```bash
cargo install polytorus
```

## Quick Start

### 1. Initialize Configuration
```bash
# Generate default configuration
./target/release/polytorus config generate --output config.toml

# Edit configuration as needed
nano config.toml
```

### 2. Create Your First Wallet
```bash
# Create a new wallet
./target/release/polytorus wallet create --name "my_wallet"

# List all addresses
./target/release/polytorus wallet list-addresses
```

### 3. Start the Node
```bash
# Start node in development mode
./target/release/polytorus node start --config config.toml --network development

# Start node in mainnet mode
./target/release/polytorus node start --config config.toml --network mainnet
```

### 4. Start Mining (Optional)
```bash
# Start mining to your wallet address
./target/release/polytorus mining start --address YOUR_WALLET_ADDRESS
```

## Multi-Node Development Environment

For testing and development, PolyTorus provides a comprehensive multi-node simulation environment:

### Quick Multi-Node Setup
```bash
# 1. Build the project first
cargo build --release

# 2. Start 4-node simulation environment (recommended)
./scripts/simulate.sh local --nodes 4 --duration 300

# 3. Test complete transaction propagation
./scripts/test_complete_propagation.sh

# 4. Monitor nodes in real-time
cargo run --example transaction_monitor
```

### Detailed Step-by-Step Guide

#### Step 1: Prepare Environment
```bash
# Build the project
cargo build --release

# Check available scripts
ls -la scripts/

# View simulation help
./scripts/simulate.sh --help
```

#### Step 2: Start Multi-Node Simulation
```bash
# Basic 4-node simulation (5 minutes)
./scripts/simulate.sh local

# Custom configuration example
./scripts/simulate.sh local --nodes 6 --duration 600 --interval 3000

# Check simulation status
./scripts/simulate.sh status
```

#### Step 3: Test Transaction Propagation
```bash
# Run complete propagation test
./scripts/test_complete_propagation.sh

# Expected output:
# ✅ Complete propagation tests completed!
# Node 0: transactions_sent > 0, transactions_received > 0
# Node 1: transactions_sent > 0, transactions_received > 0
# etc.
```

#### Step 4: Monitor and Verify
```bash
# Real-time monitoring
cargo run --example transaction_monitor

# Manual verification
for port in 9000 9001 9002 9003; do
  echo "Node port $port:"
  curl -s "http://127.0.0.1:$port/stats" | jq
done
```

#### Step 5: Cleanup
```bash
# Stop simulation
./scripts/simulate.sh stop

# Clean up data
./scripts/simulate.sh clean
```

### Manual Multi-Node Setup (Advanced)
```bash
# Build the project first
cargo build --release

# Create simulation directories
mkdir -p data/simulation/{node-0,node-1,node-2,node-3}

# Start multiple nodes manually on different ports
./target/release/polytorus --config ./data/simulation/node-0/config.toml --data-dir ./data/simulation/node-0 --http-port 9000 --modular-start &
./target/release/polytorus --config ./data/simulation/node-1/config.toml --data-dir ./data/simulation/node-1 --http-port 9001 --modular-start &
./target/release/polytorus --config ./data/simulation/node-2/config.toml --data-dir ./data/simulation/node-2 --http-port 9002 --modular-start &
./target/release/polytorus --config ./data/simulation/node-3/config.toml --data-dir ./data/simulation/node-3 --http-port 9003 --modular-start &

# Wait for nodes to start
sleep 10

# Verify nodes are running
for port in 9000 9001 9002 9003; do
  echo "Testing node on port $port:"
  curl -s "http://127.0.0.1:$port/health" || echo "Node not ready"
done
```

### Test Transaction Propagation (Manual)
```bash
# Test 1: Send transaction from Node 0 to Node 1
echo "Testing Node 0 -> Node 1 transaction..."

# Step 1: Record send at sender (Node 0)
curl -X POST http://127.0.0.1:9000/send \
  -H "Content-Type: application/json" \
  -d '{"from":"wallet_node-0","to":"wallet_node-1","amount":100,"nonce":1001}'

# Step 2: Record reception at receiver (Node 1)
curl -X POST http://127.0.0.1:9001/transaction \
  -H "Content-Type: application/json" \
  -d '{"from":"wallet_node-0","to":"wallet_node-1","amount":100,"nonce":1001}'

# Step 3: Verify statistics
echo "Node 0 stats (should show transactions_sent: 1):"
curl -s http://127.0.0.1:9000/stats | jq '.transactions_sent'

echo "Node 1 stats (should show transactions_received: 1):"
curl -s http://127.0.0.1:9001/stats | jq '.transactions_received'
```

### Docker-based Multi-Node Environment
```bash
# Start all nodes with Docker Compose
docker-compose up -d

# Check container status
docker-compose ps

# Expected output:
# NAME        IMAGE     COMMAND     SERVICE   CREATED    STATUS     PORTS
# node-0      ...       ...         node-0    ...        Up         0.0.0.0:9000->9000/tcp
# node-1      ...       ...         node-1    ...        Up         0.0.0.0:9001->9001/tcp

# View logs from specific node
docker-compose logs -f node-0

# Test Docker environment
curl http://localhost:9000/health
curl http://localhost:9001/health

# Stop all containers
docker-compose down
```

### Troubleshooting Common Issues

#### Port Already in Use
```bash
# Check what's using the ports
netstat -tulpn | grep :900[0-3]

# Kill conflicting processes
pkill -f polytorus

# Clean up zombie processes
./scripts/simulate.sh clean
```

#### Configuration Issues
```bash
# Verify configuration files exist
ls -la data/simulation/*/config.toml

# Check configuration syntax
./target/release/polytorus --config ./data/simulation/node-0/config.toml --help
```

#### Build Issues
```bash
# Clean build
cargo clean
cargo build --release

# Check Rust version
rustc --version  # Should be 1.70+
```

📚 **Detailed Guide**: [Multi-Node Simulation Documentation](MULTI_NODE_SIMULATION.md)

## Basic Operations

### Wallet Management

#### Create a New Wallet
```bash
polytorus wallet create --name "trading_wallet" --password
```

#### Import an Existing Wallet
```bash
polytorus wallet import --private-key "your_private_key" --name "imported_wallet"
```

#### Check Balance
```bash
polytorus wallet balance --address "your_wallet_address"
```

#### Send Transactions
```bash
polytorus transaction send \
  --from "sender_address" \
  --to "recipient_address" \
  --amount 1000000000 \
  --fee 1000000
```

### Blockchain Operations

#### Get Blockchain Information
```bash
polytorus blockchain info
```

#### Get Block Information
```bash
# By height
polytorus blockchain block --height 100

# By hash
polytorus blockchain block --hash "block_hash"
```

#### Print Blockchain
```bash
polytorus blockchain print
```

### Mining Operations

#### Start Mining
```bash
polytorus mining start --address "your_mining_address" --threads 4
```

#### Check Mining Status
```bash
polytorus mining status
```

#### Stop Mining
```bash
polytorus mining stop
```

## Configuration

### Configuration File Structure
```toml
[network]
port = 8333
bootstrap_peers = ["peer1.example.com:8333", "peer2.example.com:8333"]
max_peers = 50

[mining]
default_address = "your_mining_address"
threads = 4

[blockchain]
difficulty = 4
block_time_ms = 600000  # 10 minutes

[wallet]
data_dir = "./wallets"
default_wallet = "main_wallet"

[api]
enabled = true
port = 8000
cors_enabled = true
rate_limit = 100

[logging]
level = "info"
file = "./logs/polytorus.log"
```

### Environment Variables
```bash
export POLYTORUS_CONFIG_PATH="./config.toml"
export POLYTORUS_DATA_DIR="./data"
export POLYTORUS_LOG_LEVEL="debug"
export RUST_LOG="polytorus=debug"
```

## Smart Contracts

### Deploying a Smart Contract
```bash
# Compile WASM contract
polytorus contract compile --source contract.wat --output contract.wasm

# Deploy contract
polytorus contract deploy \
  --bytecode contract.wasm \
  --from "your_address" \
  --gas-limit 1000000
```

### Calling Contract Functions
```bash
polytorus contract call \
  --address "contract_address" \
  --function "transfer" \
  --args '["recipient", 1000]' \
  --from "your_address" \
  --gas-limit 100000
```

## Development Setup

### Running Tests
```bash
# Run all tests
cargo test

# Run specific test module
cargo test blockchain::tests

# Run tests with logging
RUST_LOG=debug cargo test -- --nocapture
```

### Running Examples
```bash
# Run difficulty adjustment example
cargo run --example difficulty_adjustment_example

# Run simple difficulty test
cargo run --example simple_difficulty_test
```

### Development Mode
```bash
# Start development node with reduced difficulty
polytorus node start --config config.toml --network development --dev-mode
```

## Web Interface

### Starting the Web Server
```bash
polytorus web start --port 8080
```

### Accessing the Web Interface
Open your browser and navigate to `http://localhost:8080`

Available endpoints:
- Dashboard: `/`
- Wallet: `/wallet`
- Blockchain Explorer: `/explorer`
- Mining Console: `/mining`
- Smart Contracts: `/contracts`

## API Usage

### REST API
The REST API is available at `http://localhost:8000/api/v1` when the web server is running.

Example API calls:
```bash
# Get blockchain info
curl http://localhost:8000/api/v1/blockchain/info

# Get wallet balance
curl http://localhost:8000/api/v1/wallet/balance/YOUR_ADDRESS

# Send transaction
curl -X POST http://localhost:8000/api/v1/transaction/send \
  -H "Content-Type: application/json" \
  -d '{
    "from": "sender_address",
    "to": "recipient_address",
    "amount": 1000000000,
    "fee": 1000000
  }'
```

### WebSocket API
```javascript
const ws = new WebSocket('ws://localhost:8000/ws');

ws.onmessage = function(event) {
  const data = JSON.parse(event.data);
  console.log('Received:', data);
};
```

## Troubleshooting

### Common Issues

#### Build Errors
```bash
# Update Rust toolchain
rustup update

# Clean build cache
cargo clean

# Rebuild
cargo build --release
```

#### Network Connection Issues
```bash
# Check firewall settings
sudo ufw status

# Test network connectivity
polytorus network test-connection --peer "peer_address:port"
```

#### Database Issues
```bash
# Reindex blockchain
polytorus blockchain reindex

# Reset database (warning: this will delete all data)
polytorus database reset --confirm
```

### Log Analysis
```bash
# View real-time logs
tail -f logs/polytorus.log

# Search for errors
grep -i error logs/polytorus.log

# View last 100 lines
tail -n 100 logs/polytorus.log
```

### Performance Tuning

#### Memory Optimization
```toml
[performance]
cache_size = 1000  # Number of blocks to cache
max_connections = 50
worker_threads = 4
```

#### Mining Optimization
```bash
# Set CPU affinity for mining
taskset -c 0-3 polytorus mining start --address "your_address"

# Adjust mining intensity
polytorus mining start --address "your_address" --intensity medium
```

## Next Steps

1. **Explore the Documentation**: Read the other documentation files for detailed information about specific features.

2. **Join the Community**:
   - GitHub: https://github.com/quantumshiro/polytorus
   - Discord: [Community Discord Server]
   - Telegram: [Community Telegram Group]

3. **Develop Applications**: Use the API and SDK to build applications on top of PolyTorus.

4. **Contribute**: Check out the contribution guidelines and help improve PolyTorus.

## Resources

- [API Reference](API_REFERENCE.md)
- [Smart Contracts Guide](SMART_CONTRACTS.md)
- [Modular Architecture](MODULAR_ARCHITECTURE.md)
- [CLI Commands](CLI_COMMANDS.md)
- [Difficulty Adjustment](DIFFICULTY_ADJUSTMENT.md)

## Support

If you encounter any issues or have questions:

1. Check the troubleshooting section above
2. Search existing GitHub issues
3. Create a new issue with detailed information
4. Join the community channels for help

Welcome to the PolyTorus ecosystem! 🚀
