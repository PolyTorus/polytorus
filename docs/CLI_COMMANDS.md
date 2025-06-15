# PolyTorus CLI Commands

## Overview
Comprehensive CLI commands for operating PolyTorus blockchain, including modular architecture management and multi-node simulation capabilities.

## Core Commands

### `modular`
Modular blockchain management commands

```bash
# Start modular node
polytorus modular start --config config/modular.toml

# Check layer status
polytorus modular status

# Display execution layer status
polytorus modular execution status

# Display settlement layer status
polytorus modular settlement status

# Display consensus layer status
polytorus modular consensus status

# Display data availability layer status
polytorus modular data-availability status
```

### `layers`
Layer-specific operation commands

```bash
# Execute transaction on execution layer
polytorus layers execution execute-tx --tx-file transaction.json

# Submit settlement batch
polytorus layers settlement submit-batch --batch-file batch.json

# Submit fraud proof
polytorus layers settlement submit-challenge --challenge-file challenge.json

# Store data
polytorus layers data-availability store --data-file data.bin

# Retrieve data
polytorus layers data-availability retrieve --hash <HASH>
```

## Multi-Node Simulation Commands

### Global Options for Multi-Node Operations
```bash
polytorus [GLOBAL_OPTIONS] <COMMAND> [COMMAND_OPTIONS]

Global Options:
  --config, -c <FILE>      Configuration file path
  --data-dir <DIR>         Data directory path
  --http-port <PORT>       HTTP API server port
  --p2p-port <PORT>        P2P network port
  --verbose, -v            Enable verbose logging
  --help, -h               Show help information
```

### Node Management
```bash
# Start node with custom configuration
polytorus --config ./data/simulation/node-0/config.toml \
          --data-dir ./data/simulation/node-0 \
          --http-port 9000 \
          --modular-start

# Start multiple nodes for simulation
for i in {0..3}; do
  polytorus --config ./data/simulation/node-$i/config.toml \
            --data-dir ./data/simulation/node-$i \
            --http-port $((9000+i)) \
            --modular-start &
done
```

### Simulation Scripts
```bash
# Start multi-node simulation (via script)
./scripts/simulate.sh local --nodes 4 --duration 300

# Test complete transaction propagation
./scripts/test_complete_propagation.sh

# Monitor simulation status
./scripts/simulate.sh status

# Stop simulation
./scripts/simulate.sh stop

# Clean up simulation environment
./scripts/simulate.sh clean
```

### Transaction Monitoring
```bash
# Real-time transaction monitoring tool
cargo run --example transaction_monitor

# Multi-node statistics script
cargo run --example multi_node_simulation
```

### `config`
Configuration management commands

```bash
# Generate modular configuration
polytorus config generate-modular --output config/modular.toml

# Validate configuration
polytorus config validate --config config/modular.toml

# Display layer-specific configuration
polytorus config show-layer --layer execution
polytorus config show-layer --layer consensus
polytorus config show-layer --layer settlement
polytorus config show-layer --layer data-availability
```

## Configuration File Example

### `config/modular.toml`
```toml
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
retention_period = 604800
max_data_size = 1048576

[data_availability.network_config]
listen_addr = "0.0.0.0:7000"
bootstrap_peers = []
max_peers = 50
```

## Usage Examples

### 1. Starting a Modular Node
```bash
# Generate configuration file
polytorus config generate-modular --output config/modular.toml

# Start node
polytorus modular start --config config/modular.toml
```

### 2. Transaction Execution
```bash
# Create transaction file
cat > transaction.json << EOF
{
  "to": "recipient_address",
  "value": 100,
  "gas_limit": 21000
}
EOF

# Execute transaction
polytorus layers execution execute-tx --tx-file transaction.json
```

### 3. Layer Status Monitoring
```bash
# Check overall status
polytorus modular status

# Check execution layer details
polytorus layers execution status

# Check settlement history
polytorus layers settlement history --limit 10
```

### 4. Data Storage and Retrieval
```bash
# Store data
echo "Hello, Modular Blockchain!" > data.txt
polytorus layers data-availability store --data-file data.txt

# Retrieve data (using hash returned from above command)
polytorus layers data-availability retrieve --hash abc123...
```

## Error Handling

### Common Errors
- `Layer not responding`: Layer is not responding
- `Invalid configuration`: Configuration file is invalid
- `Gas limit exceeded`: Gas limit exceeded
- `Challenge period expired`: Challenge period has expired

### Debug Options
```bash
# Verbose logging
RUST_LOG=debug polytorus modular start --config config/modular.toml

# Layer-specific logging
RUST_LOG=polytorus::modular::execution=trace polytorus modular start
```

## Performance Monitoring

### Metrics Check
```bash
# Per-layer performance
polytorus modular metrics --layer execution
polytorus modular metrics --layer consensus
polytorus modular metrics --layer settlement
polytorus modular metrics --layer data-availability

# Overall statistics
polytorus modular statistics
```

## Developer Features

### Test Environment Setup
```bash
# Generate test configuration
polytorus config generate-modular --test --output config/test-modular.toml

# Initialize test data
polytorus modular init-test --config config/test-modular.toml
```

### Profiling
```bash
# Enable performance profiling
polytorus modular start --config config/modular.toml --profile

# Monitor memory usage
polytorus modular memory-usage --interval 5s
```
