# Multi-Node Transaction Simulation & Complete Propagation

Multi-node transaction simulation functionality for the PolyTorus blockchain environment.
Supports **complete transaction propagation** with accurate tracking of both sending and receiving operations.

## 🎯 New Feature: Complete Transaction Propagation

### Overview
- **Sender API**: `/send` endpoint increments `tx_count` on sender nodes
- **Receiver API**: `/transaction` endpoint increments `rx_count` on receiver nodes  
- **Complete Tracking**: Each transaction is properly recorded on both sender and receiver sides

### Propagation Flow
```
Sender Node              Receiver Node
    ↓                         ↓
POST /send              POST /transaction
    ↓                         ↓
tx_count++              rx_count++
    ↓                         ↓
"Send Record"           "Receive Record"
```

## 🚀 Quick Start

### Method 1: Using Integrated Scripts (Recommended)

```bash
# Preparation: Build the project
cargo build --release

# Basic simulation (4 nodes, 5 minutes)
./scripts/simulate.sh local

# Complete propagation test (recommended)
./scripts/test_complete_propagation.sh

# Custom configuration simulation
./scripts/simulate.sh local --nodes 6 --duration 600 --interval 3000

# Check simulation status
./scripts/simulate.sh status

# Stop simulation and cleanup
./scripts/simulate.sh stop
./scripts/simulate.sh clean
```

### Method 2: Manual Complete Propagation Test

```bash
# Step 0: Verify nodes are running
for port in 9000 9001 9002 9003; do
  echo "Testing node on port $port:"
  curl -s "http://127.0.0.1:$port/health" && echo " ✅ Ready" || echo " ❌ Not ready"
done

# Step 1: Record send at sender node
echo "Step 1: Recording send at Node 0..."
curl -X POST -H "Content-Type: application/json" \
  -d '{"from":"wallet_node-0","to":"wallet_node-1","amount":100,"nonce":1001}' \
  "http://127.0.0.1:9000/send"

# Step 2: Record receive at receiver node  
echo "Step 2: Recording receive at Node 1..."
curl -X POST -H "Content-Type: application/json" \
  -d '{"from":"wallet_node-0","to":"wallet_node-1","amount":100,"nonce":1001}' \
  "http://127.0.0.1:9001/transaction"

# Step 3: Check statistics
echo "Step 3: Checking statistics..."
echo "Node 0 stats:" && curl -s "http://127.0.0.1:9000/stats" | jq
echo "Node 1 stats:" && curl -s "http://127.0.0.1:9001/stats" | jq
```

### Method 3: Real-time Monitoring

```bash
# Transaction monitoring tool (run in separate terminal)
cargo run --example transaction_monitor

# Node statistics check (loop execution)
while true; do
  clear
  echo "=== Node Statistics $(date) ==="
  for port in 9000 9001 9002 9003; do
    echo "Node port $port:" 
    curl -s "http://127.0.0.1:$port/stats" | jq '{transactions_sent, transactions_received, node_id}'
    echo ""
  done
  sleep 5
done
```

### Method 4: Docker Environment Execution

```bash
# Start with Docker Compose
docker-compose up -d

# Check container status
docker-compose ps

# Health check for each container
for port in 9000 9001 9002 9003; do
  echo "Testing Docker node on port $port:"
  curl -s "http://localhost:$port/health" && echo " ✅ Ready" || echo " ❌ Not ready"
done

# Check container logs
docker-compose logs -f node-0

# Complete propagation test (Docker environment)
curl -X POST http://localhost:9000/send \
  -H "Content-Type: application/json" \
  -d '{"from":"wallet_node-0","to":"wallet_node-1","amount":100,"nonce":1001}'

curl -X POST http://localhost:9001/transaction \
  -H "Content-Type: application/json" \
  -d '{"from":"wallet_node-0","to":"wallet_node-1","amount":100,"nonce":1001}'

# Stop
docker-compose down
```

## 🌐 HTTP API Endpoints

Each node provides the following HTTP APIs:

### Complete Propagation APIs

- `POST /send` - **Send Recording API** (used by sender nodes)
- `POST /transaction` - **Receive Recording API** (used by receiver nodes)
- `GET /stats` - **Statistics Information** (includes send/receive counters)
- `GET /status` - Node status
- `GET /health` - Health check

### API Usage Examples

```bash
# Complete transaction propagation example: Node 0 → Node 1

# Step 1: Record send at sender node (Node 0)
curl -X POST http://127.0.0.1:9000/send \
  -H "Content-Type: application/json" \
  -d '{"from":"wallet_node-0","to":"wallet_node-1","amount":100,"nonce":1001}'

# Step 2: Record receive at receiver node (Node 1)
curl -X POST http://127.0.0.1:9001/transaction \
  -H "Content-Type: application/json" \
  -d '{"from":"wallet_node-0","to":"wallet_node-1","amount":100,"nonce":1001}'

# Step 3: Check statistics
curl http://127.0.0.1:9000/stats  # Sender statistics
curl http://127.0.0.1:9001/stats  # Receiver statistics
```

### Response Examples

**Send Recording API (`/send`) Response:**
```json
{
  "status": "sent",
  "transaction_id": "8d705e89-50fb-4a34-bb0e-a8083bbcb40c",
  "message": "Transaction from wallet_node-0 to wallet_node-1 for 100 sent"
}
```

**Receive Recording API (`/transaction`) Response:**
```json
{
  "status": "accepted", 
  "transaction_id": "baf3ecb7-86dd-4523-9d8a-0eb90eb6da43",
  "message": "Transaction from wallet_node-0 to wallet_node-1 for 100 accepted"
}
```

**Statistics API (`/stats`) Response:**
```json
{
  "transactions_sent": 3,
  "transactions_received": 8,
  "timestamp": "2025-06-15T19:47:44.380841660+00:00",
  "node_id": "node-0"
}
```

## 📊 Monitoring and Debugging

### Real-time Monitoring

```bash
# Dedicated monitoring tool (displays in table format for better readability)
cargo run --example transaction_monitor

# Simple statistics check
curl -s http://127.0.0.1:9000/stats | jq '.'

# Batch check for all nodes statistics
for port in 9000 9001 9002 9003; do
  node_num=$((port - 9000))
  echo "Node $node_num: $(curl -s http://127.0.0.1:$port/stats)"
done
```

### Example Output

```
📊 Network Statistics - 2025-06-15 19:47:44 UTC
┌─────────┬────────┬──────────┬──────────┬────────────┬─────────────┐
│ Node    │ Status │ TX Sent  │ TX Recv  │ Block Height│ Last Update │
├─────────┼────────┼──────────┼──────────┼────────────┼─────────────┤
│ node-0  │ 🟢 Online  │        3 │        8 │          0 │ 0s ago      │
│ node-1  │ 🟢 Online  │        1 │       19 │          0 │ 0s ago      │
│ node-2  │ 🟢 Online  │        1 │       18 │          0 │ 0s ago      │
│ node-3  │ 🟢 Online  │        1 │       10 │          0 │ 0s ago      │
├─────────┼────────┼──────────┼──────────┼────────────┼─────────────┤
│ Total   │  4/4  ON │        6 │       55 │ N/A        │ Summary     │
└─────────┴────────┴──────────┴──────────┴────────────┴─────────────┘
```

## ⚙️ Configuration Options

### Simulation Settings

| Parameter | Default | Description |
|-----------|---------|-------------|
| `--nodes` | 4 | Number of nodes |
| `--duration` | 300 | Simulation duration (seconds) |
| `--interval` | 5000 | Transaction send interval (milliseconds) |
| `--base-port` | 9000 | HTTP API base port |
| `--p2p-port` | 8000 | P2P network base port |

### Node Configuration

Each node has its own configuration file:

```toml
[network]
listen_addr = "127.0.0.1:8000"
bootstrap_peers = ["127.0.0.1:8001", "127.0.0.1:8002"]
max_peers = 50

[storage]
data_dir = "./data/simulation/node-0"
max_cache_size = 1073741824

[logging]
level = "INFO"
output = "console"
```

## 📈 Performance Evaluation

### Complete Propagation Verification

```bash
# Execute complete propagation test
./scripts/test_complete_propagation.sh

# Expected results:
# - Each node has transactions_sent > 0
# - Each node has transactions_received > 0
# - Total sent and received counts match
```

### Metrics

- **TX Sent**: Number of sent transactions (**✅ Implemented**)
- **TX Recv**: Number of received transactions (**✅ Implemented**)
- **Network Latency**: Inter-node communication latency
- **Block Propagation**: Block propagation time  
- **API Response Time**: HTTP API response time

## 🔄 Available Scripts

### Main Scripts

```bash
# Integrated simulation management
./scripts/simulate.sh [local|docker|rust|status|stop|clean]

# Complete propagation test (recommended)
./scripts/test_complete_propagation.sh

# Individual node startup
./scripts/multi_node_simulation.sh [nodes] [base_port] [p2p_port] [duration]
```

### Monitoring & Analysis Scripts

```bash
# Real-time monitoring
cargo run --example transaction_monitor

# Statistics information check
for port in 9000 9001 9002 9003; do
  echo "Node $((port-9000)): $(curl -s http://127.0.0.1:$port/stats)"
done
```

## 🛠️ Troubleshooting

### Common Issues

1. **Port Conflict Error**
   ```bash
   # Check ports in use
   netstat -tulpn | grep :9000
   
   # Use different base port
   ./scripts/simulate.sh local --base-port 9100
   ```

2. **TX Sent Remains 0**
   ```bash
   # Cause: /send endpoint not being called
   # Solution: Use test_complete_propagation.sh
   ./scripts/test_complete_propagation.sh
   ```

3. **TX Recv Remains 0**
   ```bash
   # Cause: /transaction endpoint not being called
   # Solution: POST correctly to receiver node as well
   curl -X POST http://127.0.0.1:9001/transaction -d '{...}'
   ```

4. **Node Not Responding**
   ```bash
   # Health check
   curl http://127.0.0.1:9000/health
   
   # Process check
   ./scripts/simulate.sh status
   
   # Restart
   ./scripts/simulate.sh stop && ./scripts/simulate.sh local
   ```

### Debug Logs

```bash
# Check node logs
tail -f ./data/simulation/node-0.log

# Monitor all node logs
tail -f ./data/simulation/node-*.log

# Extract error logs
grep -i error ./data/simulation/node-*.log
```

## 📁 File Structure

```
scripts/
├── simulate.sh                    # Main simulation management
├── test_complete_propagation.sh   # Complete propagation test
├── multi_node_simulation.sh       # Individual simulation
└── analyze_tps.sh                 # Performance analysis

examples/
├── multi_node_simulation.rs       # Rust implementation
└── transaction_monitor.rs         # Monitoring tool

data/simulation/
├── node-0/
│   ├── config.toml
│   └── data/
├── node-1/
└── ...
```

## 🎯 Success Verification Methods

### Complete Propagation Verification Checklist

1. **✅ Node Startup Verification**
   ```bash
   curl http://127.0.0.1:9000/health
   ```

2. **✅ Send Record Verification**
   ```bash
   # Before sending
   curl -s http://127.0.0.1:9000/stats | jq '.transactions_sent'  # 0
   
   # Execute send
   curl -X POST http://127.0.0.1:9000/send -d '{...}'
   
   # After sending
   curl -s http://127.0.0.1:9000/stats | jq '.transactions_sent'  # 1
   ```

3. **✅ Receive Record Verification**
   ```bash
   # Before receiving
   curl -s http://127.0.0.1:9001/stats | jq '.transactions_received'
   
   # Execute receive
   curl -X POST http://127.0.0.1:9001/transaction -d '{...}'
   
   # After receiving
   curl -s http://127.0.0.1:9001/stats | jq '.transactions_received'  # +1
   ```

4. **✅ Complete Propagation Test**
   ```bash
   ./scripts/test_complete_propagation.sh
   # Result: All nodes should have transactions_sent > 0 AND transactions_received > 0
   ```

## 📝 Update History

- **2025-06-16**: Complete implementation and documentation update of multi-node simulation functionality
  - Complete transaction propagation functionality implemented and verified
  - Added `/send` endpoint (for send recording)
  - Modified `/transaction` endpoint (for receive recording)
  - Added `test_complete_propagation.sh` script and verified operation
  - Confirmed normal operation of both TX Sent / TX Recv across all nodes
  - Implemented integrated monitoring tool `transaction_monitor.rs`
  - Full containerization with Docker Compose environment
  - Performance testing and log analysis tools setup
  - Comprehensive documentation updates (this document, API_REFERENCE.md)
