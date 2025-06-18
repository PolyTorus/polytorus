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

### Core Modular Architecture Implementation Status
The project features a sophisticated modular design with the following layers and their current implementation status:

1. **Consensus Layer** (`src/modular/consensus.rs`) - **✅ FULLY IMPLEMENTED**
   - Complete Proof-of-Work consensus mechanism
   - Comprehensive block validation (structure, PoW, timestamps, transactions)
   - Transaction validation with signature verification
   - Validator management and mining capabilities
   - **Test Coverage**: 6 comprehensive test functions
   - **Status**: Production-ready with robust validation

2. **Data Availability Layer** (`src/modular/data_availability.rs`) - **✅ FULLY IMPLEMENTED**
   - Real Merkle tree construction and proof verification
   - Data storage with metadata and integrity checks
   - Network-aware data distribution simulation
   - Comprehensive verification with caching and replication tracking
   - **Test Coverage**: 15 extensive test functions (best coverage)
   - **Status**: Most sophisticated implementation with real cryptographic proofs

3. **Settlement Layer** (`src/modular/settlement.rs`) - **✅ FULLY IMPLEMENTED**
   - Optimistic rollup processing with real fraud proof verification
   - Batch transaction settlement with integrity verification
   - Challenge processing with time-based expiration
   - Settlement history tracking and penalty system
   - **Test Coverage**: 13 comprehensive test functions
   - **Status**: Working optimistic rollup settlement with re-execution

4. **Execution Layer** (`src/modular/execution.rs`) - **⚠️ PARTIALLY IMPLEMENTED**
   - Dual transaction processing (account-based + eUTXO)
   - Smart contract execution engine integration
   - State management with rollback capabilities
   - Gas metering and resource management
   - **Test Coverage**: ❌ No dedicated unit tests (major gap)
   - **Status**: Good architecture but lacks direct validation

5. **Unified Orchestrator** (`src/modular/unified_orchestrator.rs`) - **⚠️ BASIC IMPLEMENTATION**
   - Event-driven architecture with 17 event types
   - Layer coordination and message passing framework
   - Performance metrics and health monitoring
   - Network integration capabilities
   - **Test Coverage**: ❌ No dedicated tests (significant gap)
   - **Status**: Well-designed architecture but needs integration validation

### Diamond IO Privacy Layer - **✅ IMPLEMENTED**
Advanced indistinguishability obfuscation integrated throughout the modular architecture:

- **Circuit Obfuscation**: Transform smart contracts into indistinguishable programs
- **Homomorphic Evaluation**: Execute obfuscated circuits on encrypted data  
- **Multiple Security Modes**: Dummy (testing), Testing (development), Production (maximum security)
- **E2E Privacy**: Complete obfuscation from contract creation to execution
- **Integration Status**: Working Diamond IO demos and performance tests available
- **Test Coverage**: Multiple integration test files and examples

### Network Architecture - **✅ IMPLEMENTED**
Sophisticated P2P networking with modern protocols:

- **Priority Message Queue**: Advanced message prioritization with rate limiting
- **Peer Management**: Comprehensive peer tracking, health monitoring, and blacklisting  
- **Network Topology**: Real-time network health and topology analysis
- **Bootstrap Node Support**: Automated peer discovery and connection management
- **Integration Status**: Working multi-node simulation and P2P examples
- **Test Coverage**: P2P tests and simulation scripts available

## Development Guidelines

### Code Quality Standards
The project maintains a **zero dead code policy**:

- All code must be actively used (no `#[allow(dead_code)]`)
- Zero compiler warnings allowed
- Comprehensive test coverage (100+ tests)
- Strict Clippy compliance

### Testing Architecture - **Current Status**
- **Unit Tests**: Located alongside source files (`*_tests.rs`)
  - ✅ **Data Availability**: 15 comprehensive tests
  - ✅ **Settlement Layer**: 13 comprehensive tests  
  - ✅ **Consensus Layer**: 6 comprehensive tests
  - ❌ **Execution Layer**: No dedicated unit tests (needs improvement)
  - ❌ **Unified Orchestrator**: No integration tests (needs improvement)
- **Integration Tests**: In `/tests` directory (Diamond IO, ERC20, EUTXO)
- **CLI Tests**: Comprehensive 25+ test functions in `src/command/cli_tests.rs`
- **Kani Verification**: Formal verification in `/kani-verification`
- **Property-Based Tests**: Using criterion for benchmarks

### Critical Testing Gaps
1. **Execution Layer**: Needs unit tests for transaction processing and state management
2. **Unified Orchestrator**: Needs integration tests showing layer coordination
3. **End-to-End**: Missing full system integration tests

### Configuration Management
Configuration files are in `/config`:
- `modular.toml` - Modular architecture settings
- `diamond_io.toml` - Diamond IO configuration
- `polytorus.toml` - General blockchain settings
- `docker-node.toml` - Docker deployment configuration

### Dependencies & Build Requirements

**Essential Dependencies:**
- **Rust**: 1.82+ (not 1.87 - that was incorrect)
- **OpenFHE**: MachinaIO fork with `exp/reimpl_trapdoor` branch (must be at `/usr/local`)
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
├── modular/              # Primary modular architecture - CORE IMPLEMENTATION
│   ├── consensus.rs      # ✅ FULLY IMPLEMENTED - PoW consensus with validation
│   ├── execution.rs      # ⚠️ PARTIALLY IMPLEMENTED - missing unit tests
│   ├── settlement.rs     # ✅ FULLY IMPLEMENTED - optimistic rollups with fraud proofs
│   ├── data_availability.rs # ✅ FULLY IMPLEMENTED - Merkle proofs & verification
│   ├── unified_orchestrator.rs # ⚠️ BASIC - needs integration tests
│   ├── traits.rs         # ✅ COMPLETE - well-defined interfaces
│   ├── storage.rs        # ✅ IMPLEMENTED - modular storage layer
│   ├── message_bus.rs    # ✅ IMPLEMENTED - inter-layer communication
│   └── network.rs        # ✅ IMPLEMENTED - modular network integration
├── diamond_io_integration.rs # ✅ IMPLEMENTED - privacy layer integration
├── blockchain/           # ✅ LEGACY - maintained for compatibility
├── crypto/              # ✅ IMPLEMENTED - ECDSA, FN-DSA, Verkle trees
├── network/             # ✅ IMPLEMENTED - P2P with priority queues & health monitoring
├── smart_contract/      # ✅ IMPLEMENTED - WASM engine with ERC20 support
├── command/             # ✅ IMPLEMENTED - comprehensive CLI with 25+ tests
└── webserver/           # ✅ IMPLEMENTED - HTTP API endpoints
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

### Current Development Priorities

**Immediate Actions Needed:**
1. **Add Unit Tests for Execution Layer** (`src/modular/execution.rs`)
   - Test transaction processing functionality
   - Test state management and rollback capabilities  
   - Test gas metering and resource management

2. **Add Integration Tests for Unified Orchestrator** (`src/modular/unified_orchestrator.rs`)
   - Test layer coordination and message passing
   - Test event-driven architecture with real layers
   - Test performance metrics and health monitoring

3. **End-to-End Integration Tests**
   - Test all modular layers working together
   - Test complete transaction flow through all layers
   - Test error handling and recovery scenarios

### Common Pitfalls to Avoid
1. **OpenFHE Dependencies**: Ensure OpenFHE is properly installed at system level
2. **Dead Code**: Never use `#[allow(dead_code)]` - create methods that use all fields
3. **Test Isolation**: Use proper cleanup in tests, especially for file system operations
4. **Async Safety**: Be careful with shared state in async contexts
5. **Configuration Validation**: Always validate TOML configurations before use
6. **Testing Gaps**: Don't assume implementation works without comprehensive tests

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