# PolyTorus API Reference

## Overview
This document provides a comprehensive reference for the PolyTorus blockchain API endpoints and their usage.

## Authentication
All API endpoints require authentication using API keys or JWT tokens (implementation dependent).

## Base URL
```
http://localhost:8000/api/v1
```

## Endpoints

### Blockchain Operations

#### Get Blockchain Information
```http
GET /blockchain/info
```

**Response:**
```json
{
  "height": 12345,
  "best_block_hash": "000abc123...",
  "difficulty": 4,
  "total_transactions": 54321,
  "network": "mainnet"
}
```

#### Get Block by Hash
```http
GET /blockchain/block/{hash}
```

**Parameters:**
- `hash` (string): Block hash

**Response:**
```json
{
  "hash": "000abc123...",
  "prev_hash": "000def456...",
  "height": 12345,
  "timestamp": 1672531200000,
  "difficulty": 4,
  "nonce": 123456,
  "transactions": [...]
}
```

#### Get Block by Height
```http
GET /blockchain/block/height/{height}
```

**Parameters:**
- `height` (integer): Block height

### Wallet Operations

#### Create Wallet
```http
POST /wallet/create
```

**Request Body:**
```json
{
  "name": "my_wallet",
  "password": "secure_password"
}
```

**Response:**
```json
{
  "address": "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa",
  "private_key_encrypted": "encrypted_private_key_data"
}
```

#### List Addresses
```http
GET /wallet/addresses
```

**Response:**
```json
{
  "addresses": [
    {
      "address": "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa",
      "balance": 1000000000,
      "label": "main_wallet"
    }
  ]
}
```

#### Get Balance
```http
GET /wallet/balance/{address}
```

**Parameters:**
- `address` (string): Wallet address

**Response:**
```json
{
  "address": "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa",
  "balance": 1000000000,
  "confirmed_balance": 900000000,
  "unconfirmed_balance": 100000000
}
```

### Transaction Operations

#### Send Transaction
```http
POST /transaction/send
```

**Request Body:**
```json
{
  "from": "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa",
  "to": "1BvBMSEYstWetqTFn5Au4m4GFg7xJaNVN2",
  "amount": 100000000,
  "fee": 1000000,
  "password": "wallet_password"
}
```

**Response:**
```json
{
  "transaction_id": "abc123def456...",
  "status": "pending",
  "fee": 1000000
}
```

#### Get Transaction
```http
GET /transaction/{txid}
```

**Parameters:**
- `txid` (string): Transaction ID

**Response:**
```json
{
  "txid": "abc123def456...",
  "block_hash": "000abc123...",
  "block_height": 12345,
  "confirmations": 6,
  "inputs": [...],
  "outputs": [...],
  "fee": 1000000,
  "timestamp": 1672531200000
}
```

### Mining Operations

#### Start Mining
```http
POST /mining/start
```

**Request Body:**
```json
{
  "address": "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa",
  "threads": 4
}
```

**Response:**
```json
{
  "status": "started",
  "mining_address": "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa",
  "threads": 4
}
```

#### Stop Mining
```http
POST /mining/stop
```

**Response:**
```json
{
  "status": "stopped"
}
```

#### Get Mining Status
```http
GET /mining/status
```

**Response:**
```json
{
  "is_mining": true,
  "hash_rate": 1000000,
  "blocks_mined": 5,
  "current_difficulty": 4,
  "mining_address": "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa"
}
```

### Network Operations

#### Get Network Health
```http
GET /network/health
```

**Response:**
```json
{
  "status": "healthy",
  "total_nodes": 25,
  "healthy_peers": 23,
  "degraded_peers": 2,
  "disconnected_peers": 0,
  "average_latency": 45,
  "network_version": "1.0.0"
}
```

#### Get Peer Information
```http
GET /network/peer/{peer_id}
```

**Parameters:**
- `peer_id` (string): Peer identifier (UUID format)

**Response:**
```json
{
  "peer_id": "550e8400-e29b-41d4-a716-446655440000",
  "address": "192.168.1.100:8333",
  "status": "connected",
  "health": "healthy",
  "last_seen": 1672531200000,
  "version": "1.0.0",
  "latency": 35
}
```

#### Get Message Queue Statistics
```http
GET /network/queue/stats
```

**Response:**
```json
{
  "critical_queue_size": 0,
  "high_queue_size": 5,
  "normal_queue_size": 12,
  "low_queue_size": 3,
  "total_messages": 20,
  "messages_per_second": 2.5,
  "bandwidth_usage": "75%",
  "rate_limit_status": "normal"
}
```

#### Blacklist Peer
```http
POST /network/blacklist
```

**Request Body:**
```json
{
  "peer_id": "550e8400-e29b-41d4-a716-446655440000",
  "reason": "Malicious behavior detected"
}
```

**Response:**
```json
{
  "success": true,
  "message": "Peer 550e8400-e29b-41d4-a716-446655440000 blacklisted for: Malicious behavior detected"
}
```

#### Remove Peer from Blacklist
```http
DELETE /network/blacklist/{peer_id}
```

**Parameters:**
- `peer_id` (string): Peer identifier to remove from blacklist

**Response:**
```json
{
  "success": true,
  "message": "Peer 550e8400-e29b-41d4-a716-446655440000 removed from blacklist"
}
```

### Smart Contract Operations

#### Deploy Contract
```http
POST /contract/deploy
```

**Request Body:**
```json
{
  "code": "compiled_wasm_bytecode",
  "init_data": "initialization_data",
  "gas_limit": 1000000,
  "from": "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa"
}
```

**Response:**
```json
{
  "contract_address": "contract_address_hash",
  "transaction_id": "deployment_txid",
  "gas_used": 500000
}
```

#### Call Contract Function
```http
POST /contract/call
```

**Request Body:**
```json
{
  "contract_address": "contract_address_hash",
  "function": "transfer",
  "args": ["recipient_address", 1000],
  "gas_limit": 100000,
  "from": "1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa"
}
```

## Error Codes

### HTTP Status Codes
- `200 OK` - Request successful
- `400 Bad Request` - Invalid request parameters
- `401 Unauthorized` - Authentication required
- `404 Not Found` - Resource not found
- `500 Internal Server Error` - Server error

### Application Error Codes
```json
{
  "error": {
    "code": "INSUFFICIENT_BALANCE",
    "message": "Insufficient balance for transaction",
    "details": {
      "required": 1000000000,
      "available": 500000000
    }
  }
}
```

### Common Error Codes
- `INVALID_ADDRESS` - Invalid wallet address format
- `INSUFFICIENT_BALANCE` - Not enough funds
- `TRANSACTION_NOT_FOUND` - Transaction ID not found
- `BLOCK_NOT_FOUND` - Block hash or height not found
- `INVALID_SIGNATURE` - Transaction signature verification failed
- `NETWORK_ERROR` - P2P network communication error
- `CONTRACT_EXECUTION_FAILED` - Smart contract execution error

## Rate Limiting
API endpoints are rate-limited to prevent abuse:
- 100 requests per minute for general endpoints
- 10 requests per minute for mining operations
- 50 requests per minute for transaction operations

## WebSocket API
Real-time updates available via WebSocket connection:

```javascript
const ws = new WebSocket('ws://localhost:8000/ws');

ws.on('message', function(data) {
  const event = JSON.parse(data);
  // Handle events: new_block, new_transaction, mining_update
});
```

## SDK Examples

### JavaScript/Node.js
```javascript
const PolyTorusAPI = require('polytorus-sdk');

const client = new PolyTorusAPI('http://localhost:8000/api/v1');

// Send transaction
const result = await client.sendTransaction({
  from: 'sender_address',
  to: 'recipient_address',
  amount: 1000000000
});
```

### Python
```python
from polytorus import PolyTorusClient

client = PolyTorusClient('http://localhost:8000/api/v1')

# Get blockchain info
info = client.get_blockchain_info()
print(f"Current height: {info['height']}")
```

### Rust
```rust
use polytorus_sdk::PolyTorusClient;

#[tokio::main]
async fn main() {
    let client = PolyTorusClient::new("http://localhost:8000/api/v1");

    let balance = client.get_balance("address").await.unwrap();
    println!("Balance: {}", balance);
}
```

## Modular Execution Layer API

### Contract Engine Operations

#### Get Contract Engine
```rust
pub fn get_contract_engine(&self) -> Arc<Mutex<ContractEngine>>
```
Returns a reference to the contract execution engine for direct smart contract operations.

#### Execute Contract with Engine
```rust
pub fn execute_contract_with_engine(
    &self,
    contract_address: &str,
    function_name: &str,
    args: &[u8]
) -> Result<Vec<u8>>
```
Executes a contract function using the internal contract engine.

**Parameters:**
- `contract_address`: Target contract address
- `function_name`: Name of the function to call
- `args`: Function arguments as byte array

**Returns:** Function return value as byte array

#### Process Contract Transaction
```rust
pub fn process_contract_transaction(&self, tx: &Transaction) -> Result<TransactionReceipt>
```
Processes a complete contract transaction (deployment or function call).

### Account State Management

#### Get Account State from Storage
```rust
pub fn get_account_state_from_storage(&self, address: &str) -> Option<AccountState>
```
Retrieves account state from internal storage cache.

#### Set Account State in Storage
```rust
pub fn set_account_state_in_storage(&self, address: String, state: AccountState)
```
Updates account state in internal storage cache.

### Execution Context Management

#### Get Execution Context
```rust
pub fn get_execution_context(&self) -> Option<ExecutionContext>
```
Returns the current execution context with all state transition information.

#### Validate Execution Context
```rust
pub fn validate_execution_context(&self) -> Result<bool>
```
Validates the current execution context, checking:
- Context ID validity
- State root integrity
- Gas usage within limits
- Pending changes consistency

**ExecutionContext Structure:**
```rust
pub struct ExecutionContext {
    pub context_id: String,
    pub initial_state_root: Hash,
    pub pending_changes: HashMap<String, AccountState>,
    pub executed_txs: Vec<TransactionReceipt>,
    pub gas_used: u64,
}
```

### Transaction Processing

#### Add Transaction
```rust
pub fn add_transaction(&self, transaction: Transaction) -> Result<()>
```

#### Get Pending Transactions
```rust
pub fn get_pending_transactions(&self) -> Result<Vec<Transaction>>
```

#### Clear Transaction Pool
```rust
pub fn clear_transaction_pool(&self) -> Result<()>
```

## CLI API Reference

### Overview
PolyTorus provides a comprehensive command-line interface with modular architecture support, cryptographic wallet management, and blockchain operations.

### Command Structure
```bash
polytorus [GLOBAL_OPTIONS] <COMMAND> [COMMAND_OPTIONS]
```

### Global Options
- `--config, -c <FILE>`: Configuration file path
- `--verbose, -v`: Enable verbose output
- `--help, -h`: Show help information
- `--version, -V`: Show version information

### Commands

#### Modular Architecture Commands

**Start Modular Blockchain**
```bash
polytorus modular start [CONFIG_FILE]
```
- `CONFIG_FILE` (optional): Path to TOML configuration file
- Default: Uses built-in configuration

**Mine Blocks (Modular)**
```bash
polytorus modular mine <ADDRESS>
```
- `ADDRESS`: Mining reward address

**Check Modular State**
```bash
polytorus modular state
```

**View Layer Information**
```bash
polytorus modular layers
```

#### Wallet Management

**Create Wallet**
```bash
polytorus createwallet <TYPE> [OPTIONS]
```
- `TYPE`: Cryptographic type (`ECDSA` | `FNDSA`)
- `--name <NAME>`: Wallet name (optional)

**List Addresses**
```bash
polytorus listaddresses
```

**Get Balance**
```bash
polytorus getbalance <ADDRESS>
```

#### Traditional Blockchain Commands

**Start Node**
```bash
polytorus start-node [OPTIONS]
```
- `--port <PORT>`: Network port (default: 8333)

**Start Mining**
```bash
polytorus start-miner [OPTIONS]
```
- `--threads <COUNT>`: Mining threads (default: 4)
- `--address <ADDRESS>`: Mining reward address

**Print Chain**
```bash
polytorus print-chain
```

**Reindex Blockchain**
```bash
polytorus reindex
```

#### Web Server

**Start Web Server**
```bash
polytorus start-webserver [OPTIONS]
```
- `--port <PORT>`: Server port (default: 8080)
- `--bind <ADDRESS>`: Bind address (default: 127.0.0.1)

## Configuration Files

#### TOML Configuration Structure
```toml
[blockchain]
difficulty = 4
max_transactions_per_block = 1000

[network]
port = 8333
max_peers = 50

[modular]
enable_consensus_layer = true
enable_execution_layer = true
enable_settlement_layer = true
enable_data_availability_layer = true

[mining]
threads = 4
reward_address = "your_address_here"

[web]
port = 8080
bind_address = "127.0.0.1"
cors_enabled = true
```

#### Environment Configuration
```bash
# Environment variables
export POLYTORUS_CONFIG="/path/to/config.toml"
export POLYTORUS_DATA_DIR="/path/to/data"
export POLYTORUS_LOG_LEVEL="info"
```

### CLI Testing Commands

**Run All Tests**
```bash
cargo test
```

**Run CLI-Specific Tests**
```bash
cargo test cli_tests
```

**Run Configuration Tests**
```bash
cargo test test_configuration
```

**Run Wallet Tests**
```bash
cargo test test_wallet
```

**Run Modular Tests**
```bash
cargo test test_modular
```

### Error Handling

#### Common Error Codes
- `CONFIG_NOT_FOUND`: Configuration file not found
- `INVALID_ADDRESS`: Invalid wallet address format
- `INSUFFICIENT_FUNDS`: Insufficient balance for transaction
- `NETWORK_ERROR`: Network connectivity issues
- `VALIDATION_ERROR`: Transaction or block validation failed

#### Error Response Format
```json
{
  "error": {
    "code": "CONFIG_NOT_FOUND",
    "message": "Configuration file not found at specified path",
    "details": {
      "path": "/path/to/config.toml",
      "suggestion": "Create configuration file or use default settings"
    }
  }
}
```

### Examples

#### Complete Workflow Example
```bash
# 1. Create quantum-resistant wallet
polytorus createwallet FNDSA --name quantum-wallet

# 2. Start modular blockchain
polytorus modular start

# 3. Start mining to wallet address
polytorus modular mine 1A1zP1eP5QGefi2DMPTfTL5SLmv7DivfNa

# 4. Check blockchain state
polytorus modular state

# 5. Start web interface
polytorus start-webserver --port 8080
```

#### Configuration Testing Example
```bash
# Test configuration validation
echo '[blockchain]
difficulty = 4
[network]
port = 8333' > test-config.toml

# Start with custom configuration
polytorus modular start test-config.toml
```

## Multi-Node Simulation APIs

### Transaction Propagation

#### Send Transaction (Sender Node)
```http
POST /send
```

Records a transaction as sent from the current node.

**Request Body:**
```json
{
  "from": "wallet_node-0",
  "to": "wallet_node-1", 
  "amount": 100,
  "nonce": 1001
}
```

**Response:**
```json
{
  "status": "sent",
  "transaction_id": "8d705e89-50fb-4a34-bb0e-a8083bbcb40c",
  "message": "Transaction from wallet_node-0 to wallet_node-1 for 100 sent"
}
```

#### Receive Transaction (Receiver Node)
```http
POST /transaction
```

Records a transaction as received by the current node.

**Request Body:**
```json
{
  "from": "wallet_node-0",
  "to": "wallet_node-1",
  "amount": 100,
  "nonce": 1001
}
```

**Response:**
```json
{
  "status": "accepted",
  "transaction_id": "baf3ecb7-86dd-4523-9d8a-0eb90eb6da43", 
  "message": "Transaction from wallet_node-0 to wallet_node-1 for 100 accepted"
}
```

#### Get Node Statistics
```http
GET /stats
```

Returns transaction statistics for the current node.

**Response:**
```json
{
  "transactions_sent": 3,
  "transactions_received": 8,
  "timestamp": "2025-06-15T19:47:44.380841660+00:00",
  "node_id": "node-0"
}
```

#### Get Node Status
```http
GET /status
```

Returns the current status of the node.

**Response:**
```json
{
  "status": "running",
  "block_height": 0,
  "is_running": true,
  "total_transactions": 11,
  "total_blocks": 0,  "uptime": "0h 45m 32s"
}
```

#### Health Check
```http
GET /health
```

Simple health check endpoint for monitoring.

**Response:**
```json
{
  "status": "healthy",
  "timestamp": "2025-06-16T04:55:23.129845240+00:00"
}
```

### Complete Transaction Propagation Flow

The complete propagation ensures both sending and receiving nodes properly record transactions:

#### Setup Multi-Node Environment

**Quick Setup (Recommended):**
```bash
# 1. Build project
cargo build --release

# 2. Start simulation
./scripts/simulate.sh local --nodes 4 --duration 300

# 3. Wait for nodes to be ready
sleep 10

# 4. Verify all nodes are running
for port in 9000 9001 9002 9003; do
  curl -s "http://127.0.0.1:$port/health" || echo "Node on port $port not ready"
done
```

**Manual Setup:**
```bash
# Start nodes manually
./target/release/polytorus --config ./data/simulation/node-0/config.toml --data-dir ./data/simulation/node-0 --http-port 9000 --modular-start &
./target/release/polytorus --config ./data/simulation/node-1/config.toml --data-dir ./data/simulation/node-1 --http-port 9001 --modular-start &
./target/release/polytorus --config ./data/simulation/node-2/config.toml --data-dir ./data/simulation/node-2 --http-port 9002 --modular-start &
./target/release/polytorus --config ./data/simulation/node-3/config.toml --data-dir ./data/simulation/node-3 --http-port 9003 --modular-start &
```

#### Full Propagation Example

**Step-by-Step Transaction Flow:**
```bash
# Transaction: Node 0 → Node 1
echo "=== Testing Complete Transaction Propagation ==="
echo "Transaction: Node 0 sends 100 to Node 1"

# Step 1: Check initial statistics
echo "Initial statistics:"
echo "Node 0:" && curl -s http://127.0.0.1:9000/stats | jq '{transactions_sent, transactions_received}'
echo "Node 1:" && curl -s http://127.0.0.1:9001/stats | jq '{transactions_sent, transactions_received}'

# Step 2: Send transaction from Node 0
echo -e "\n🚀 Step 1: Recording send at Node 0..."
SEND_RESPONSE=$(curl -s -X POST http://127.0.0.1:9000/send \
  -H "Content-Type: application/json" \
  -d '{"from":"wallet_node-0","to":"wallet_node-1","amount":100,"nonce":1001}')
echo "Send response: $SEND_RESPONSE"

# Step 3: Record reception at Node 1
echo -e "\n📥 Step 2: Recording reception at Node 1..."
RECEIVE_RESPONSE=$(curl -s -X POST http://127.0.0.1:9001/transaction \
  -H "Content-Type: application/json" \
  -d '{"from":"wallet_node-0","to":"wallet_node-1","amount":100,"nonce":1001}')
echo "Receive response: $RECEIVE_RESPONSE"

# Step 4: Verify updated statistics
echo -e "\n📊 Step 3: Verifying updated statistics..."
echo "Node 0 (should show transactions_sent +1):"
curl -s http://127.0.0.1:9000/stats | jq '{transactions_sent, transactions_received}'

echo "Node 1 (should show transactions_received +1):"
curl -s http://127.0.0.1:9001/stats | jq '{transactions_sent, transactions_received}'

echo -e "\n✅ Complete propagation test completed!"
```

**Expected Output:**
```bash
=== Testing Complete Transaction Propagation ===
Transaction: Node 0 sends 100 to Node 1
Initial statistics:
Node 0:
{
  "transactions_sent": 0,
  "transactions_received": 0
}
Node 1:
{
  "transactions_sent": 0,
  "transactions_received": 0
}

🚀 Step 1: Recording send at Node 0...
Send response: {"status":"sent","transaction_id":"8d705e89-50fb-4a34-bb0e-a8083bbcb40c","message":"Transaction from wallet_node-0 to wallet_node-1 for 100 sent"}

📥 Step 2: Recording reception at Node 1...
Receive response: {"status":"accepted","transaction_id":"baf3ecb7-86dd-4523-9d8a-0eb90eb6da43","message":"Transaction from wallet_node-0 to wallet_node-1 for 100 accepted"}

📊 Step 3: Verifying updated statistics...
Node 0 (should show transactions_sent +1):
{
  "transactions_sent": 1,
  "transactions_received": 0
}
Node 1 (should show transactions_received +1):
{
  "transactions_sent": 0,
  "transactions_received": 1
}

✅ Complete propagation test completed!
```

#### Automated Testing Scripts

**Complete Propagation Test:**
```bash
# Run automated complete propagation test
./scripts/test_complete_propagation.sh

# Expected output:
# 🚀 Complete Transaction Propagation Test
# ========================================
# Test 1: Node 0 -> Node 1
# Step 1: Sending to Node 0 /send endpoint...
# Step 2: Sending to Node 1 /transaction endpoint...
# ...
# ✅ Complete propagation tests completed!
```

**Continuous Monitoring:**
```bash
# Real-time monitoring tool
cargo run --example transaction_monitor

# Expected output:
# ┌─────────┬────────────┬──────────┬──────────┬─────────────┬─────────────┐
# │ Node    │ Status     │ TX Sent  │ TX Recv  │ Block Height│ Last Update │
# ├─────────┼────────────┼──────────┼──────────┼─────────────┼─────────────┤
# │ node-0  │ 🟢 Online  │        3 │        8 │          0 │ 0s ago      │
# │ node-1  │ 🟢 Online  │        1 │       19 │          0 │ 0s ago      │
# ...
```

**Performance Testing:**
```bash
# Bulk transaction testing
for i in {1..10}; do
  echo "Transaction batch $i"
  curl -s -X POST http://127.0.0.1:9000/send \
    -H "Content-Type: application/json" \
    -d "{\"from\":\"wallet_node-0\",\"to\":\"wallet_node-1\",\"amount\":$((i*10)),\"nonce\":$((2000+i))}"
  
  curl -s -X POST http://127.0.0.1:9001/transaction \
    -H "Content-Type: application/json" \
    -d "{\"from\":\"wallet_node-0\",\"to\":\"wallet_node-1\",\"amount\":$((i*10)),\"nonce\":$((2000+i))}"
  
  sleep 1
done

# Check final statistics
echo "Final statistics after bulk test:"
curl -s http://127.0.0.1:9000/stats | jq
curl -s http://127.0.0.1:9001/stats | jq
```

#### Full Propagation Example
```bash
# Step 1: Send transaction from Node 0 to Node 1
curl -X POST http://127.0.0.1:9000/send \
  -H "Content-Type: application/json" \
  -d '{"from":"wallet_node-0","to":"wallet_node-1","amount":100,"nonce":1001}'

# Step 2: Record reception at Node 1
curl -X POST http://127.0.0.1:9001/transaction \
  -H "Content-Type: application/json" \
  -d '{"from":"wallet_node-0","to":"wallet_node-1","amount":100,"nonce":1001}'

# Step 3: Verify statistics
curl -s http://127.0.0.1:9000/stats | jq '.transactions_sent'     # Should increment
curl -s http://127.0.0.1:9001/stats | jq '.transactions_received' # Should increment
```

#### Monitoring Endpoints

**Multi-Node Status Overview**
```bash
# Check all nodes
for port in 9000 9001 9002 9003; do
  echo "Node port $port:"
  curl -s "http://127.0.0.1:$port/stats"
  echo ""
done
```

**Expected Output:**
```json
Node port 9000:
{"transactions_sent":3,"transactions_received":8,"timestamp":"2025-06-16T04:55:23.129845240+00:00","node_id":"node-0"}

Node port 9001:
{"transactions_sent":1,"transactions_received":19,"timestamp":"2025-06-16T04:55:23.129845240+00:00","node_id":"node-1"}
```

### Simulation Scripts Integration

#### Automated Testing
```bash
# Complete propagation test
./scripts/test_complete_propagation.sh

# Multi-node simulation with monitoring
./scripts/simulate.sh local --nodes 4 --duration 300

# Real-time monitoring
cargo run --example transaction_monitor
```

#### Docker Environment
```bash
# Docker Compose simulation
docker-compose up -d

# Check Docker container status
docker-compose ps

# View logs
docker-compose logs -f node-0
```

### Error Handling for Simulation APIs

#### Common Simulation Errors
- `CONNECTION_REFUSED`: Node not running or port unavailable
- `INVALID_JSON`: Malformed request body
- `TIMEOUT`: Node not responding within expected time
- `PORT_CONFLICT`: Multiple nodes attempting to bind to same port

#### Troubleshooting Guide
```bash
# Check if ports are available
netstat -tulpn | grep :900[0-3]

# Verify node processes
ps aux | grep polytorus

# Clean up zombie processes
pkill -f polytorus

# Restart simulation environment
./scripts/simulate.sh clean && ./scripts/simulate.sh local
```

### Performance Metrics

#### Transaction Throughput
- **Local Network**: 50-100 TPS per node
- **4-Node Setup**: 200-400 TPS aggregate
- **Docker Environment**: 30-60 TPS per container

#### Network Latency
- **Local Loopback**: < 1ms
- **Docker Bridge**: 1-5ms
- **Cross-Container**: 2-10ms

#### Resource Usage
- **Memory**: ~32MB per node
- **CPU**: 1-5% per node (idle)
- **Storage**: ~1MB per 1000 transactions

## Integration Examples

### Rust Application Integration
```rust
use reqwest::Client;
use serde_json::json;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = Client::new();
    
    // Send transaction
    let response = client
        .post("http://127.0.0.1:9000/send")
        .json(&json!({
            "from": "wallet_node-0",
            "to": "wallet_node-1", 
            "amount": 100,
            "nonce": 1001
        }))
        .send()
        .await?;
    
    println!("Send response: {}", response.text().await?);
    
    // Record reception
    let response = client
        .post("http://127.0.0.1:9001/transaction")
        .json(&json!({
            "from": "wallet_node-0",
            "to": "wallet_node-1",
            "amount": 100, 
            "nonce": 1001
        }))
        .send()
        .await?;
    
    println!("Receive response: {}", response.text().await?);
    
    Ok(())
}
```

### Python Integration
```python
import requests
import json
import time

def send_complete_transaction(sender_port, receiver_port, tx_data):
    """Send a complete transaction with propagation"""
    
    # Step 1: Record as sent
    send_response = requests.post(
        f"http://127.0.0.1:{sender_port}/send",
        json=tx_data
    )
    
    # Step 2: Record as received  
    receive_response = requests.post(
        f"http://127.0.0.1:{receiver_port}/transaction",
        json=tx_data
    )
    
    return send_response.json(), receive_response.json()

# Example usage
tx_data = {
    "from": "wallet_node-0",
    "to": "wallet_node-1",
    "amount": 100,
    "nonce": 1001
}

send_result, receive_result = send_complete_transaction(9000, 9001, tx_data)
print(f"Send: {send_result}")
print(f"Receive: {receive_result}")
```

### JavaScript/Node.js Integration
```javascript
const axios = require('axios');

async function sendCompleteTransaction(senderPort, receiverPort, txData) {
    try {
        // Step 1: Record as sent
        const sendResponse = await axios.post(
            `http://127.0.0.1:${senderPort}/send`,
            txData
        );
        
        // Step 2: Record as received
        const receiveResponse = await axios.post(
            `http://127.0.0.1:${receiverPort}/transaction`, 
            txData
        );
        
        return {
            sent: sendResponse.data,
            received: receiveResponse.data
        };
    } catch (error) {
        console.error('Transaction propagation failed:', error.message);
        throw error;
    }
}

// Example usage
const txData = {
    from: "wallet_node-0",
    to: "wallet_node-1",
    amount: 100,
    nonce: 1001
};

sendCompleteTransaction(9000, 9001, txData)
    .then(result => {
        console.log('Transaction propagated successfully:', result);
    })
    .catch(error => {
        console.error('Failed to propagate transaction:', error);
    });
```

---

*Last updated: June 16, 2025*
*For the latest updates and complete documentation, visit: [PolyTorus Documentation](docs/)*
  "data_dir": "./data/simulation/node-0"
}
```

#### Health Check
```http
GET /health
```

Simple health check endpoint.

**Response:**
```json
{
  "status": "healthy",
  "timestamp": "2025-06-15T19:44:09.146558523+00:00"
}
```

### Complete Propagation Flow

For a complete transaction propagation from Node A to Node B:

1. **Step 1**: POST to Node A's `/send` endpoint (records as sent)
2. **Step 2**: POST to Node B's `/transaction` endpoint (records as received)
3. **Step 3**: Check statistics via `/stats` on both nodes

**Example:**
```bash
# Node 0 → Node 1 transaction
curl -X POST http://127.0.0.1:9000/send \
  -H "Content-Type: application/json" \
  -d '{"from":"wallet_node-0","to":"wallet_node-1","amount":100,"nonce":1001}'

curl -X POST http://127.0.0.1:9001/transaction \
  -H "Content-Type: application/json" \
  -d '{"from":"wallet_node-0","to":"wallet_node-1","amount":100,"nonce":1001}'
```
