# 🌐 PolyTorus Local Testnet Guide

Welcome to PolyTorus Local Testnet! This guide helps you set up and run a complete blockchain testnet on your local machine using ContainerLab.

## 📋 Prerequisites

Before you begin, ensure you have the following installed:

- **Docker** - Container runtime
- **ContainerLab** - Network topology orchestrator  
- **Python 3** - For CLI tools
- **curl** - For API testing

### Quick Installation

```bash
# Install ContainerLab
bash -c "$(curl -sL https://get.containerlab.dev)"

# Install Docker (Ubuntu/Debian)
curl -fsSL https://get.docker.com -o get-docker.sh
sudo sh get-docker.sh

# Verify installations
containerlab version
docker --version
python3 --version
```

## 🚀 Quick Start

### 1. Build and Start the Testnet

```bash
# Clone the PolyTorus repository
git clone https://github.com/PolyTorus/polytorus
cd polytorus

# Build the Docker image
./start-local-testnet.sh build

# Start the testnet
./start-local-testnet.sh start
```

### 2. Access the Testnet

Once started, you can access your testnet through multiple interfaces:

| Service | URL | Description |
|---------|-----|-------------|
| **Web UI** | http://localhost:3000 | Interactive web interface |
| **Block Explorer** | http://localhost:8080 | View blocks and transactions |
| **API Gateway** | http://localhost:9020 | REST API access |
| **Bootstrap Node** | http://localhost:9000 | Main blockchain node |
| **Miner 1** | http://localhost:9001 | First mining node |
| **Miner 2** | http://localhost:9002 | Second mining node |
| **Validator** | http://localhost:9003 | Validation node |

### 3. Create Your First Wallet

```bash
# Create a new wallet
./start-local-testnet.sh wallet

# Or use the interactive CLI
./start-local-testnet.sh cli
```

### 4. Send Your First Transaction

Open the Web UI at http://localhost:3000 and:

1. Select a wallet from the dropdown
2. Enter a recipient address
3. Specify the amount to send
4. Click "Send Transaction"

## 🛠️ Management Commands

The `start-local-testnet.sh` script provides comprehensive management:

```bash
# Core operations
./start-local-testnet.sh start      # Start the testnet
./start-local-testnet.sh stop       # Stop the testnet
./start-local-testnet.sh restart    # Restart the testnet
./start-local-testnet.sh status     # Check status

# Development tools
./start-local-testnet.sh build      # Build Docker image
./start-local-testnet.sh logs       # View container logs
./start-local-testnet.sh clean      # Clean all data

# User operations
./start-local-testnet.sh wallet     # Create new wallet
./start-local-testnet.sh send       # Send test transaction
./start-local-testnet.sh web        # Open web interface
./start-local-testnet.sh cli        # Interactive CLI
```

## 🎮 Interactive CLI

The testnet includes a powerful Python-based CLI for advanced operations:

```bash
# Start interactive mode
./start-local-testnet.sh cli

# Available commands in CLI:
polytest> help                    # Show all commands
polytest> status                  # Network status
polytest> wallets                 # List wallets
polytest> create-wallet           # Create new wallet
polytest> balance <address>       # Check balance
polytest> send <from> <to> <amount>  # Send transaction
polytest> transactions            # Recent transactions
polytest> stats                   # Blockchain statistics
```

## 📊 Network Architecture

Your local testnet consists of 6 containers:

```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│  Bootstrap  │────│   Miner 1   │────│   Miner 2   │
│    :9000    │    │    :9001    │    │    :9002    │
└─────────────┘    └─────────────┘    └─────────────┘
       │                   │                   │
       └───────────────────┼───────────────────┘
                           │
       ┌─────────────┐    ┌─────────────┐    ┌─────────────┐
       │  Validator  │    │User Interface│   │  Explorer   │
       │    :9003    │    │    :3000    │    │    :8080    │
       └─────────────┘    └─────────────┘    └─────────────┘
```

### Node Types

- **Bootstrap**: Genesis node, network entry point
- **Miner 1 & 2**: Active mining nodes with PoW consensus
- **Validator**: Transaction validation and network health
- **User Interface**: Web UI and API gateway for users
- **Explorer**: Block explorer and network monitoring

## 🌐 Web Interface Features

The Web UI (http://localhost:3000) provides:

### Dashboard
- Real-time network status
- Blockchain statistics (block height, transactions, difficulty)
- Node health monitoring

### Wallet Management
- View all available wallets
- Check wallet balances
- Create new wallets

### Transaction Operations
- Send transactions between wallets
- Real-time transaction tracking
- Transaction history viewer

### Mining Control
- View mining status
- Control mining operations (future feature)

## 🔧 API Usage

The API Gateway (http://localhost:9020) exposes REST endpoints:

### Wallet Operations
```bash
# Create wallet
curl -X POST http://localhost:9020/wallet/create

# List wallets
curl http://localhost:9020/wallet/list

# Get balance
curl http://localhost:9020/balance/<address>
```

### Transaction Operations
```bash
# Send transaction
curl -X POST http://localhost:9020/transaction/send \
  -H "Content-Type: application/json" \
  -d '{
    "from": "sender_address",
    "to": "recipient_address", 
    "amount": 10.5,
    "gasPrice": 1
  }'

# Get transaction status
curl http://localhost:9020/transaction/status/<hash>

# Recent transactions
curl http://localhost:9020/transaction/recent
```

### Network Information
```bash
# Network status
curl http://localhost:9020/network/status

# Latest block
curl http://localhost:9020/block/latest

# Specific block
curl http://localhost:9020/block/<hash>
```

## 📈 Monitoring and Debugging

### Real-time Monitoring

```bash
# Check overall status
./start-local-testnet.sh status

# Watch container logs
./start-local-testnet.sh logs

# Monitor specific node
docker logs -f clab-polytorus-local-testnet-miner-1
```

### Network Statistics

The CLI provides detailed statistics:

```bash
./start-local-testnet.sh cli
polytest> stats
```

### Block Explorer

Visit http://localhost:8080 to:
- Browse all blocks
- View transaction details
- Monitor network health
- Analyze mining statistics

## 🔧 Configuration

### Testnet Configuration

The testnet uses `config/testnet.toml` for settings:

```toml
[consensus]
block_time = 10000          # 10 seconds
difficulty = 2              # Low for testing
max_block_size = 1048576    # 1MB

[testnet]
network_id = "polytorus-local-testnet"
chain_id = 31337
initial_supply = 1000000000 # 1B tokens

[testnet.prefunded_accounts]
"test_account_1" = 1000000  # 1M tokens
"test_account_2" = 500000   # 500K tokens
"test_account_3" = 100000   # 100K tokens
```

### Node-Specific Settings

Each node type has optimized settings:

- **Bootstrap**: High connectivity, API enabled
- **Miners**: Mining enabled, moderate connectivity
- **Validator**: Validation only, no mining
- **Interface**: API gateway, web UI enabled
- **Explorer**: Historical data, monitoring enabled

## 🧪 Testing Scenarios

### Basic Transaction Flow

1. **Create Wallets**: Generate sender and receiver wallets
2. **Check Balances**: Verify initial balances
3. **Send Transaction**: Transfer tokens between wallets
4. **Verify Transaction**: Check transaction status and balances
5. **Monitor Blocks**: Watch new blocks being mined

### Automated Testing

```bash
# Send 5 test transactions
python3 scripts/testnet_manager.py --test-transactions 5

# Interactive testing
python3 scripts/testnet_manager.py --interactive
```

### Load Testing

Create multiple wallets and generate transaction load:

```python
# Example: Generate 100 transactions
for i in range(100):
    # Create transaction
    # Send via API
    # Monitor confirmation
```

## 🛡️ Security Considerations

This testnet is designed for **local development only**:

- **Low Security**: Uses test keys and simplified consensus
- **No Persistence**: Data is lost when containers stop
- **Network Isolation**: Runs in isolated Docker network
- **Resource Limits**: Optimized for local resource usage

**⚠️ Never use testnet wallets or keys in production!**

## 🔄 Troubleshooting

### Common Issues

#### ContainerLab Not Starting
```bash
# Check ContainerLab installation
containerlab version

# Verify Docker is running
docker ps

# Check file permissions
chmod +x start-local-testnet.sh
```

#### Nodes Not Responding
```bash
# Check node status
./start-local-testnet.sh status

# View container logs
./start-local-testnet.sh logs

# Restart if needed
./start-local-testnet.sh restart
```

#### Web Interface Not Loading
```bash
# Check if container is running
docker ps | grep user-interface

# Check port availability
netstat -tulpn | grep :3000

# Try direct container access
curl http://localhost:3000
```

#### API Calls Failing
```bash
# Test API gateway
curl http://localhost:9020/health

# Check node connectivity
curl http://localhost:9000/status

# Verify network connectivity
docker network ls
```

### Clean Reset

If you encounter persistent issues:

```bash
# Complete cleanup
./start-local-testnet.sh clean

# Rebuild everything
./start-local-testnet.sh build
./start-local-testnet.sh start
```

## 📚 Advanced Usage

### Custom Configuration

1. Modify `config/testnet.toml` for your needs
2. Update `testnet-local.yml` for topology changes
3. Rebuild the Docker image
4. Restart the testnet

### Integration with External Tools

The testnet exposes standard APIs that work with:

- **Web3 libraries**: For dApp development
- **Blockchain explorers**: Custom explorer integration
- **Monitoring tools**: Prometheus/Grafana compatible
- **Testing frameworks**: Automated test integration

### Development Workflow

1. **Local Development**: Code and test against local testnet
2. **Integration Testing**: Run automated test suites
3. **Performance Testing**: Load test with multiple nodes
4. **Deployment Preparation**: Test production configurations

## 🤝 Support and Community

- **Issues**: Report bugs in the GitHub repository
- **Documentation**: Check the main README.md
- **Community**: Join our Discord/Telegram
- **Updates**: Follow GitHub releases for updates

## 📄 License

This testnet setup is part of the PolyTorus project and follows the same license terms.

---

## 🎯 Next Steps

Now that your testnet is running:

1. **Explore the Web UI**: Familiarize yourself with the interface
2. **Try API Calls**: Test the REST API endpoints  
3. **Create a dApp**: Build your first decentralized application
4. **Run Load Tests**: Test performance with multiple transactions
5. **Experiment with Configuration**: Modify settings and observe changes

Happy testing with PolyTorus! 🚀