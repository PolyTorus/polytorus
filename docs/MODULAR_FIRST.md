# PolyTorus Modular Architecture Guide

## Overview

PolyTorus has transitioned to a **modular-first architecture** as the primary system. This document explains the new architecture and how to use it.

## Architecture Layers

### 1. Execution Layer (`src/modular/execution.rs`)
**Purpose**: Transaction processing and smart contract execution
- WASM-based smart contract runtime
- Gas metering and resource management
- State transition validation
- Transaction batching and optimization

### 2. Settlement Layer (`src/modular/settlement.rs`)
**Purpose**: Optimistic rollup processing and finality
- Batch transaction settlement
- Challenge period management
- Fraud proof verification
- Validator stake management

### 3. Consensus Layer (`src/modular/consensus.rs`)
**Purpose**: Block validation and chain consensus
- Pluggable consensus mechanisms
- Block structure validation
- Validator set management
- Network finality guarantees

### 4. Data Availability Layer (`src/modular/data_availability.rs`)
**Purpose**: Distributed data storage and retrieval
- Data availability proofs
- Configurable retention policies
- Network-based storage
- Efficient data sampling

### 5. Orchestrator (`src/modular/orchestrator.rs`)
**Purpose**: Layer coordination and management
- Event-driven architecture
- Inter-layer communication
- State synchronization
- Configuration management

## Quick Start Commands

### Basic Usage
```bash
# Start modular blockchain (recommended)
polytorus modular start

# Start with custom config
polytorus modular start config/modular.toml

# Create quantum-resistant wallet
polytorus createwallet FNDSA

# Mine using modular architecture
polytorus modular mine <address>

# Check system status
polytorus modular state
polytorus modular layers
```

### Legacy Commands
Legacy commands are still available but marked with `[LEGACY]`:
```bash
# Legacy blockchain creation
polytorus createblockchain <address>  # [LEGACY]

# Legacy mining
polytorus startminer <port> <address>  # [LEGACY]
```

## Configuration

The modular system uses `config/modular.toml` for configuration:

```toml
[execution]
gas_limit = 8000000
gas_price = 1

[consensus]
block_time = 10000  # 10 seconds
difficulty = 4

[settlement]
challenge_period = 100  # blocks
batch_size = 100

[data_availability]
retention_period = 604800  # 7 days
max_data_size = 1048576   # 1MB
```

## Migration from Legacy

If you're using legacy commands, consider migrating to modular:

### Before (Legacy)
```bash
polytorus createblockchain my_address
polytorus startminer 3000 my_address
```

### After (Modular)
```bash
polytorus createwallet FNDSA  # Get address
polytorus modular start
polytorus modular mine my_address
```

## Benefits of Modular Architecture

1. **Separation of Concerns**: Each layer has a specific responsibility
2. **Pluggability**: Easy to swap consensus mechanisms or execution engines
3. **Scalability**: Layers can be optimized independently
4. **Testing**: Individual layers can be tested in isolation
5. **Future-Proof**: Easy to add new features without breaking existing code

## Development

When developing new features, consider which layer they belong to:
- **Transaction logic** → Execution Layer
- **Consensus changes** → Consensus Layer
- **Data storage** → Data Availability Layer
- **Settlement logic** → Settlement Layer
- **Cross-layer coordination** → Orchestrator

## Next Steps

1. Try the modular commands: `polytorus modular --help`
2. Read the configuration guide: `docs/CONFIGURATION.md`
3. Explore smart contracts: `docs/SMART_CONTRACTS.md`
4. Check the API reference: `docs/API_REFERENCE.md`
