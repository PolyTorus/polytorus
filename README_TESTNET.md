# 🏠 PolyTorus Local Testnet

**Your personal blockchain development environment**

The PolyTorus Local Testnet allows developers and users to run a complete blockchain network on their local machine using ContainerLab. Perfect for development, testing, and learning blockchain technology.

## ⚡ Quick Start

```bash
# 1. Start your testnet
./start-local-testnet.sh build
./start-local-testnet.sh start

# 2. Open web interface
./start-local-testnet.sh web

# 3. Create your first wallet
./start-local-testnet.sh wallet

# 4. Send transactions via CLI
./start-local-testnet.sh cli
```

## 🎯 What You Get

### 🌐 **Complete Blockchain Network**
- **6 Node Architecture**: Bootstrap, 2 Miners, Validator, User Interface, Explorer
- **Real Mining**: Actual Proof-of-Work consensus with configurable difficulty
- **Network Topology**: Realistic P2P connections using ContainerLab

### 💻 **User-Friendly Interfaces**
- **Web UI** (`:3000`): Beautiful interface for wallet management and transactions
- **Block Explorer** (`:8080`): View blocks, transactions, and network stats
- **REST API** (`:9020`): Full API access for dApp development
- **Interactive CLI**: Python-based command-line interface

### 🔧 **Developer Tools**
- **Hot Reloading**: Changes reflected immediately
- **Comprehensive Logging**: Debug with detailed container logs
- **API Testing**: curl-friendly REST endpoints
- **Load Testing**: Built-in transaction generation tools

## 📋 Prerequisites

- **Docker** - Container runtime
- **ContainerLab** - Network orchestration
- **Python 3** - CLI tools
- **curl** - API testing

```bash
# Quick install (Ubuntu/Debian)
bash -c "$(curl -sL https://get.containerlab.dev)"  # ContainerLab
curl -fsSL https://get.docker.com | sh               # Docker
```

## 🚀 Usage Examples

### Basic Operations

```bash
# Management
./start-local-testnet.sh start     # Start testnet
./start-local-testnet.sh stop      # Stop testnet  
./start-local-testnet.sh status    # Check status
./start-local-testnet.sh logs      # View logs

# User operations
./start-local-testnet.sh wallet    # Create wallet
./start-local-testnet.sh send      # Send test transaction
./start-local-testnet.sh web       # Open web UI
./start-local-testnet.sh cli       # Interactive CLI
```

### Interactive CLI

```bash
./start-local-testnet.sh cli

polytest> create-wallet              # Create new wallet
polytest> wallets                    # List all wallets
polytest> balance <address>          # Check balance
polytest> send <from> <to> <amount>  # Send transaction
polytest> transactions              # Recent transactions
polytest> stats                     # Network statistics
```

### API Examples

```bash
# Create wallet
curl -X POST http://localhost:9020/wallet/create

# Send transaction
curl -X POST http://localhost:9020/transaction/send \
  -H "Content-Type: application/json" \
  -d '{
    "from": "sender_address",
    "to": "recipient_address",
    "amount": 10.5,
    "gasPrice": 1
  }'

# Check balance
curl http://localhost:9020/balance/your_address

# Network status
curl http://localhost:9020/network/status
```

## 🏗️ Architecture

```
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│  Bootstrap  │────│   Miner 1   │────│   Miner 2   │
│   :9000     │    │   :9001     │    │   :9002     │
│  (Genesis)  │    │ (Mining)    │    │ (Mining)    │
└─────────────┘    └─────────────┘    └─────────────┘
       │                   │                   │
       └───────────────────┼───────────────────┘
                           │
┌─────────────┐    ┌─────────────┐    ┌─────────────┐
│  Validator  │    │User Interface│   │  Explorer   │
│   :9003     │    │   :3000     │    │   :8080     │
│(Validation) │    │ (Web UI)    │    │(Monitoring) │
└─────────────┘    └─────────────┘    └─────────────┘
```

### Node Functions

| Node | Port | Function |
|------|------|----------|
| **Bootstrap** | 9000 | Genesis node, network entry point |
| **Miner 1** | 9001 | Active mining, transaction processing |
| **Miner 2** | 9002 | Active mining, network redundancy |
| **Validator** | 9003 | Transaction validation, consensus |
| **User Interface** | 3000 | Web UI, API gateway |
| **Explorer** | 8080 | Block explorer, network monitoring |

## 🌐 Access Points

| Service | URL | Description |
|---------|-----|-------------|
| **Web UI** | http://localhost:3000 | Main user interface |
| **Block Explorer** | http://localhost:8080 | Blockchain explorer |
| **API Gateway** | http://localhost:9020 | REST API access |
| **Bootstrap API** | http://localhost:9000 | Core node API |
| **Miner 1 API** | http://localhost:9001 | Mining node API |
| **Miner 2 API** | http://localhost:9002 | Mining node API |
| **Validator API** | http://localhost:9003 | Validation node API |

## 🎮 Features

### Web Interface Features
- 👛 **Wallet Management**: Create, view, manage wallets
- 💸 **Send Transactions**: User-friendly transaction interface
- 📊 **Real-time Stats**: Block height, transactions, difficulty
- 🔍 **Network Status**: Live node health monitoring
- 📋 **Transaction History**: View all network transactions

### CLI Features
- 🖥️ **Interactive Mode**: Full-featured command-line interface
- 🔄 **Automated Testing**: Send bulk test transactions
- 📈 **Statistics**: Comprehensive network analytics
- 🛠️ **Development Tools**: Wallet creation, balance checking

### API Features
- 🔗 **REST Endpoints**: Full blockchain functionality via HTTP
- 📝 **JSON Responses**: Machine-readable data format
- 🔐 **Wallet Operations**: Create, list, check balances
- 💰 **Transaction Management**: Send, track, verify transactions
- 📊 **Network Information**: Status, blocks, statistics

## ⚙️ Configuration

The testnet is pre-configured for immediate use, but can be customized:

### Quick Settings (`config/testnet.toml`)
```toml
[consensus]
block_time = 10000          # 10 seconds
difficulty = 2              # Low for testing

[testnet]
chain_id = 31337
initial_supply = 1000000000 # 1B tokens
```

### Network Topology (`testnet-local.yml`)
- Modify node count
- Adjust resource limits
- Change network configuration
- Add custom containers

## 🧪 Testing Scenarios

### Basic Workflow
1. **Setup**: `./start-local-testnet.sh start`
2. **Create Wallets**: Use Web UI or CLI
3. **Fund Wallets**: Initial balances from genesis
4. **Send Transactions**: Between wallets
5. **Monitor**: Watch blocks being mined

### Load Testing
```bash
# Generate 100 test transactions
python3 scripts/testnet_manager.py --test-transactions 100

# Monitor performance
./start-local-testnet.sh status
```

### API Integration Testing
```bash
# Test all endpoints
curl http://localhost:9020/wallet/list
curl http://localhost:9020/network/status
curl http://localhost:9020/block/latest
```

## 🔧 Troubleshooting

### Common Issues

**Containers not starting?**
```bash
# Check dependencies
containerlab version
docker --version

# Check logs
./start-local-testnet.sh logs
```

**Web UI not loading?**
```bash
# Check container status
./start-local-testnet.sh status

# Restart if needed
./start-local-testnet.sh restart
```

**API calls failing?**
```bash
# Test connectivity
curl http://localhost:9020/health

# Check network
docker network ls
```

### Clean Reset
```bash
# Complete cleanup and restart
./start-local-testnet.sh clean
./start-local-testnet.sh build
./start-local-testnet.sh start
```

## 📚 Documentation

- **[Complete Guide](LOCAL_TESTNET_GUIDE.md)** - Detailed setup and usage
- **[API Reference](docs/API_REFERENCE.md)** - Full API documentation
- **[Configuration](docs/CONFIGURATION.md)** - Advanced configuration options
- **[Troubleshooting](docs/TROUBLESHOOTING.md)** - Common issues and solutions

## 🚀 Advanced Usage

### Custom Development
- **dApp Development**: Build against local testnet
- **Smart Contracts**: Deploy and test contracts
- **Performance Testing**: Load test your applications
- **Network Simulation**: Test network conditions

### Integration
- **CI/CD Integration**: Automated testing in pipelines
- **External Tools**: Connect monitoring and analytics
- **Custom Nodes**: Add specialized node types
- **Network Extensions**: Expand topology

## 🤝 Support

- **Issues**: [GitHub Issues](https://github.com/PolyTorus/polytorus/issues)
- **Discussions**: [GitHub Discussions](https://github.com/PolyTorus/polytorus/discussions)
- **Documentation**: [Full Documentation](https://docs.polytorus.org)
- **Community**: [Discord](https://discord.gg/polytorus)

## 📄 License

Licensed under the same terms as the main PolyTorus project.

---

## 🎯 Get Started Now!

```bash
git clone https://github.com/PolyTorus/polytorus
cd polytorus
./start-local-testnet.sh build
./start-local-testnet.sh start
./start-local-testnet.sh web
```

Your personal blockchain awaits! 🚀