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

#### Get Peer Information
```http
GET /network/peers
```

**Response:**
```json
{
  "peer_count": 8,
  "peers": [
    {
      "address": "192.168.1.100:8333",
      "version": "1.0.0",
      "connected_time": 3600,
      "last_seen": 1672531200000
    }
  ]
}
```

#### Add Peer
```http
POST /network/peers/add
```

**Request Body:**
```json
{
  "address": "192.168.1.200:8333"
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
