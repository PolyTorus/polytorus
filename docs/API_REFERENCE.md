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

### Configuration Files

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
