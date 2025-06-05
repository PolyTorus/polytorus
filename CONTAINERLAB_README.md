# PolyTorus ContainerLab Test Environment

This directory contains the ContainerLab setup for testing PolyTorus blockchain in a multi-node environment.

## 🚀 Quick Start

1. **Prerequisites:**
   ```bash
   # Install ContainerLab
   sudo bash -c "$(curl -sL https://get.containerlab.dev)"
   
   # Ensure Docker is running
   sudo systemctl start docker
   ```

2. **Setup the test environment:**
   ```bash
   ./setup_containerlab.sh
   ```

3. **Run basic transaction tests:**
   ```bash
   ./test_transactions.sh
   ```

4. **Monitor the network:**
   ```bash
   ./monitor_network.sh
   ```

## 🏗️ Network Topology

The test network consists of 5 containers:

- **Genesis Node** (`genesis`): Creates the blockchain and genesis block
  - P2P Port: `localhost:17000`
  - Web Port: `localhost:18080`

- **Miner Node 1** (`miner1`): Mining node that connects to genesis
  - P2P Port: `localhost:17001`
  - Web Port: `localhost:18081`

- **Miner Node 2** (`miner2`): Second mining node
  - P2P Port: `localhost:17002`
  - Web Port: `localhost:18082`

- **Transaction Node** (`txnode`): Node for sending transactions
  - P2P Port: `localhost:17003`
  - Web Port: `localhost:18083`

- **Test Client** (`testclient`): Container for running test commands

## 🧪 Testing Scripts

### Basic Tests (`test_transactions.sh`)
- Wallet creation and address management
- Balance checking across nodes
- Simple transactions between nodes
- Basic smart contract deployment
- Network connectivity verification

### Advanced Tests (`test_advanced.sh`)
- Multi-hop transaction chains
- Concurrent transaction processing
- Network resilience testing
- Complex smart contract interactions
- Stress testing with rapid transactions

### Network Monitoring (`monitor_network.sh`)
- Real-time balance monitoring
- Container status checking
- Blockchain height tracking
- Recent activity logs
- Interactive monitoring modes

## 📝 Usage Examples

### Manual Transaction Testing

1. **Access a container:**
   ```bash
   docker exec -it clab-polytorus-network-genesis bash
   ```

2. **Create a wallet:**
   ```bash
   polytorus createwallet FNDSA
   ```

3. **Check balance:**
   ```bash
   polytorus getbalance <wallet-address>
   ```

4. **Send transaction:**
   ```bash
   polytorus send <from-address> <to-address> <amount> --mine
   ```

5. **Deploy smart contract:**
   ```bash
   polytorus deploycontract <wallet-address> <contract-file> <gas-limit> --mine
   ```

### Network Operations

1. **Check network status:**
   ```bash
   sudo containerlab inspect -t containerlab.yml
   ```

2. **View container logs:**
   ```bash
   docker logs clab-polytorus-network-genesis
   ```

3. **Stop the network:**
   ```bash
   sudo containerlab destroy -t containerlab.yml
   ```

## 🔧 Configuration

### Docker Image
The `Dockerfile` creates a Rust-based image with PolyTorus binary. It includes:
- Rust runtime environment
- SSL certificates for secure communication
- Dedicated `polytorus` user for security
- Exposed ports for P2P (7000) and Web API (8080)

### ContainerLab Topology
The `containerlab.yml` defines:
- 5-node network topology
- Inter-container networking
- Port mappings to host
- Bootstrap configurations
- Startup commands for each node

## 📊 Smart Contract Testing

### Sample Contracts
The testing scripts include sample WebAssembly contracts:

1. **Simple Add Contract:**
   ```wasm
   (module
     (func $add (param $a i32) (param $b i32) (result i32)
       local.get $a
       local.get $b
       i32.add)
     (export "add" (func $add)))
   ```

2. **Counter Contract:**
   ```wasm
   (module
     (memory 1)
     (global $counter (mut i32) (i32.const 0))
     (func $increment (result i32) ...)
     (export "increment" (func $increment)))
   ```

3. **Token Contract:**
   ```wasm
   (module
     (memory 1)
     (global $total_supply (mut i32) (i32.const 1000))
     (func $transfer (param $amount i32) (result i32) ...)
     (export "transfer" (func $transfer)))
   ```

## 🐛 Troubleshooting

### Common Issues

1. **Containers not starting:**
   ```bash
   # Check Docker status
   sudo systemctl status docker
   
   # Check container logs
   docker logs clab-polytorus-network-genesis
   ```

2. **Port conflicts:**
   ```bash
   # Check if ports are in use
   netstat -tulpn | grep -E "(17000|17001|17002|17003|18080|18081|18082|18083)"
   ```

3. **Wallet address issues:**
   ```bash
   # Recreate wallets if needed
   docker exec clab-polytorus-network-genesis polytorus createwallet FNDSA
   ```

4. **Network connectivity:**
   ```bash
   # Check container network
   docker network ls
   docker network inspect clab
   ```

### Reset Environment

To completely reset the test environment:
```bash
# Stop and remove containers
sudo containerlab destroy -t containerlab.yml

# Remove Docker images (optional)
docker rmi polytorus:latest

# Clean up blockchain data
sudo rm -rf /tmp/polytorus-*

# Restart from scratch
./setup_containerlab.sh
```

## 📈 Performance Monitoring

### Resource Usage
```bash
# Monitor container resources
docker stats

# Check individual container
docker exec clab-polytorus-network-genesis top
```

### Network Traffic
```bash
# Monitor network traffic
sudo tcpdump -i any port 7000

# Check container network stats
docker exec clab-polytorus-network-genesis netstat -i
```

## 🔐 Security Notes

- All containers run with non-root `polytorus` user
- Network communication is isolated within ContainerLab network
- Private keys are generated within each container
- No sensitive data is exposed to host by default

## 📚 Additional Resources

- [ContainerLab Documentation](https://containerlab.dev/)
- [PolyTorus CLI Reference](./docs/CLI_COMMANDS.md)
- [Smart Contract Development](./docs/SMART_CONTRACTS.md)
- [Network Configuration](./docs/CONFIGURATION.md)
