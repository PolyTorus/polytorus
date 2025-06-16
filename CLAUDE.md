# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

PolyTorus is a cutting-edge modular blockchain platform designed for the post-quantum era. It features a revolutionary modular architecture with separate layers for consensus, execution, settlement, and data availability, along with Diamond IO integration for indistinguishability obfuscation.

## Essential Commands

### Build & Development
```bash
# Standard build (requires OpenFHE)
cargo build --release

# Development build
cargo build

# Run comprehensive tests
cargo test

# Run library-only tests (recommended during development)
cargo test --lib

# Run specific test modules
cargo test diamond_io --nocapture  # Diamond IO tests
cargo test modular --lib           # Modular architecture tests
cargo test cli_tests               # CLI functionality tests
```

### Code Quality & Linting
```bash
# Zero dead code policy enforcement
cargo check --lib
cargo clippy --lib -- -D warnings -D clippy::all

# Complete quality check pipeline
./scripts/quality_check.sh

# Format code
cargo fmt

# Security audit
cargo audit
```

### Diamond IO Integration
```bash
# Test Diamond IO functionality
cargo test diamond -- --nocapture

# Run Diamond IO demo with all configurations
cargo run --example diamond_io_demo

# Performance benchmarks
cargo run --example diamond_io_performance_test
```

### Modular Architecture
```bash
# Start modular blockchain with default config
./target/release/polytorus modular start

# Start with custom configuration
./target/release/polytorus modular start config/modular.toml

# Check modular system status
./target/release/polytorus modular state
./target/release/polytorus modular layers
```

### Wallet & Mining Operations
```bash
# Create quantum-resistant wallet
./target/release/polytorus createwallet FNDSA

# Create traditional ECDSA wallet
./target/release/polytorus createwallet ECDSA

# Mine blocks using modular architecture
./target/release/polytorus modular mine <address>

# List wallet addresses
./target/release/polytorus listaddresses
```

### Multi-Node Simulation
```bash
# Start 4-node simulation
./scripts/simulate.sh local --nodes 4 --duration 300

# Test transaction propagation
./scripts/test_complete_propagation.sh

# Monitor transactions in real-time
cargo run --example transaction_monitor
```

### Kani Verification
```bash
# Install and setup Kani
make kani-install
make kani-setup

# Run verification suite
make kani-verify

# Quick verification for development
make kani-quick

# Specific verification categories
make kani-crypto       # Cryptographic verification
make kani-blockchain   # Blockchain verification
make kani-modular      # Modular architecture verification
```

## Architecture Overview

### Core Modular Architecture
The project is built on a revolutionary modular design with these primary layers:

1. **Execution Layer** (`src/modular/execution.rs`)
   - WASM-based smart contract execution
   - Gas metering and resource management
   - State transition execution with rollback capabilities

2. **Settlement Layer** (`src/modular/settlement.rs`)
   - Optimistic rollup processing with fraud proofs
   - Batch transaction settlement and challenge period management
   - Validator stake management and slashing

3. **Consensus Layer** (`src/modular/consensus.rs`)
   - Pluggable consensus mechanisms (PoW, designed for PoS)
   - Block validation and chain management
   - Network finality guarantees

4. **Data Availability Layer** (`src/modular/data_availability.rs`)
   - Distributed data storage and retrieval
   - Configurable data retention policies
   - Network-based data availability proofs

5. **Unified Orchestrator** (`src/modular/unified_orchestrator.rs`)
   - Coordinates communication between all layers
   - Event-driven architecture with message bus
   - Layer communication and state synchronization

### Diamond IO Privacy Layer
Advanced indistinguishability obfuscation integrated throughout the modular architecture:

- **Circuit Obfuscation**: Transform smart contracts into indistinguishable programs
- **Homomorphic Evaluation**: Execute obfuscated circuits on encrypted data
- **Multiple Security Modes**: Dummy (testing), Testing (development), Production (maximum security)
- **E2E Privacy**: Complete obfuscation from contract creation to execution

### Network Architecture
Sophisticated P2P networking with modern protocols:

- **Priority Message Queue**: Advanced message prioritization with rate limiting
- **Peer Management**: Comprehensive peer tracking, health monitoring, and blacklisting
- **Network Topology**: Real-time network health and topology analysis
- **Bootstrap Node Support**: Automated peer discovery and connection management

## Development Guidelines

### Code Quality Standards
The project maintains a **zero dead code policy**:

- All code must be actively used (no `#[allow(dead_code)]`)
- Zero compiler warnings allowed
- Comprehensive test coverage (100+ tests)
- Strict Clippy compliance

### Testing Architecture
- **Unit Tests**: Located alongside source files (`*_tests.rs`)
- **Integration Tests**: In `/tests` directory
- **CLI Tests**: Comprehensive 25+ test functions in `src/command/cli_tests.rs`
- **Kani Verification**: Formal verification in `/kani-verification`
- **Property-Based Tests**: Using criterion for benchmarks

### Configuration Management
Configuration files are in `/config`:
- `modular.toml` - Modular architecture settings
- `diamond_io.toml` - Diamond IO configuration
- `polytorus.toml` - General blockchain settings
- `docker-node.toml` - Docker deployment configuration

### Dependencies & Build Requirements

**Essential Dependencies:**
- **Rust**: 1.87 nightly or later
- **OpenFHE**: MachinaIO fork with `feat/improve_determinant` branch (must be at `/usr/local`)
- **System Libraries**: `cmake`, `libgmp-dev`, `libntl-dev`, `libboost-all-dev`

**OpenFHE Installation:**
```bash
# Automated installation
sudo ./scripts/install_openfhe.sh

# Set required environment variables
export OPENFHE_ROOT=/usr/local
export LD_LIBRARY_PATH=/usr/local/lib:$LD_LIBRARY_PATH
export PKG_CONFIG_PATH=/usr/local/lib/pkgconfig:$PKG_CONFIG_PATH
```

### Directory Structure
```
src/
├── modular/              # Primary modular architecture (NEW)
│   ├── consensus.rs      # Consensus layer implementation
│   ├── execution.rs      # Execution layer with WASM engine
│   ├── settlement.rs     # Settlement layer with rollups
│   ├── data_availability.rs # Data availability layer
│   └── unified_orchestrator.rs # Layer coordination
├── diamond_io_integration.rs # Diamond IO privacy layer
├── blockchain/           # Legacy blockchain (maintained for compatibility)
├── crypto/              # Cryptographic primitives (ECDSA, FN-DSA)
├── network/             # P2P networking with priority queues
├── smart_contract/      # WASM smart contract engine
├── command/             # CLI implementation with extensive tests
└── webserver/           # HTTP API endpoints
```

### Testing Best Practices
Always run tests in this order during development:
1. `cargo test --lib` - Fast library tests
2. `cargo clippy --lib -- -D warnings` - Code quality
3. `cargo test` - Full test suite
4. `./scripts/quality_check.sh` - Complete quality pipeline

### Diamond IO Integration Notes
Diamond IO has three operational modes:
- **Dummy Mode**: Safe for development, no real obfuscation
- **Testing Mode**: Real parameters with medium security
- **Production Mode**: High-security parameters for live deployment

Always test Diamond IO functionality with:
```bash
cargo test diamond_io_with_production_params -- --nocapture
```

### Common Pitfalls to Avoid
1. **OpenFHE Dependencies**: Ensure OpenFHE is properly installed at system level
2. **Dead Code**: Never use `#[allow(dead_code)]` - create methods that use all fields
3. **Test Isolation**: Use proper cleanup in tests, especially for file system operations
4. **Async Safety**: Be careful with shared state in async contexts
5. **Configuration Validation**: Always validate TOML configurations before use

### Performance Considerations
- Modular architecture allows independent optimization of each layer
- Diamond IO operations scale with security parameters (ring dimension)
- P2P networking includes bandwidth management and rate limiting
- WASM execution includes gas metering for resource control

### Documentation Standards
- All public APIs must have rustdoc comments with examples
- Integration tests should include detailed comments explaining scenarios
- Configuration files should be well-documented with examples
- CLI help text should be comprehensive and user-friendly


You have to write tests for the code you've written.
TEST when you think you're done, and make a sound when you're really done.