# PolyTorus

[![Ask DeepWiki](https://deepwiki.com/badge.svg)](https://deepwiki.com/PolyTorus/polytorus)

<div align="center">
    <h1>🔗 PolyTorus</h1>
    <p><strong>A Cutting-Edge Modular Blockchain Platform</strong></p>    <p><em>Quantum-Resistant Era Ready • Flexible Cryptographic Wallets • Modular Architecture • WASM Smart Contracts</em></p>
</div>

PolyTorus is a revolutionary modular blockchain platform designed for the post-quantum era, offering unparalleled cryptographic flexibility and adaptability. It empowers users to choose between traditional ECDSA and quantum-resistant FN-DSA cryptography for their wallets, while implementing a sophisticated multi-layered architecture that cleanly separates consensus, execution, settlement, and data availability concerns, enabling unprecedented customization and optimization for diverse use cases in the quantum computing age.

## 🚀 Features

### Core Architecture
- **🏗️ Modular Architecture**: Cleanly separated layers for different blockchain functions with pluggable components
- **🔧 Smart Contracts**: High-performance WebAssembly (WASM) based smart contract execution engine
- **🤝 Multiple Consensus Mechanisms**: Support for various consensus algorithms including proof-of-work and future consensus protocols
- **🔐 Quantum-Resistant Cryptography**: Future-proof security with flexible cryptographic wallet options - users can choose between traditional ECDSA for current compatibility or quantum-resistant FN-DSA for post-quantum security
- **🌐 Advanced P2P Networking**: Robust peer-to-peer communication with TCP and modern networking protocols
- **💻 CLI Interface**: Comprehensive command-line tools for blockchain interaction and management
- **🌍 Web Interface**: RESTful HTTP API for external integrations and web applications

### Advanced Capabilities
- **🔄 UTXO Model**: Efficient unspent transaction output management
- **📦 State Management**: Sophisticated blockchain state handling with rollback capabilities
- **🧪 Comprehensive Testing**: Extensive test coverage across all modules
- **📊 Performance Monitoring**: Built-in metrics and monitoring capabilities
- **🔒 Wallet Management**: Secure wallet creation and transaction signing

## 🏛️ Architecture

PolyTorus implements a revolutionary modular blockchain architecture with the following layers:

### 1. **Execution Layer**
- Handles transaction processing and validation
- WASM-based smart contract execution environment
- Gas metering and resource management
- State transition execution

### 2. **Settlement Layer** 
- Manages transaction finality and state updates
- Batch processing and state commitment
- Cross-layer communication protocols
- Settlement guarantees and dispute resolution

### 3. **Consensus Layer**
- Coordinates agreement on block production and validation
- Pluggable consensus mechanisms
- Leader election and block proposal
- Network synchronization and fork resolution

### 4. **Data Availability Layer**
- Ensures data is available and verifiable across the network
- Data sampling and verification protocols
- Erasure coding and redundancy management
- Light client data availability proofs

## 🛠️ Getting Started

### Prerequisites

- **Rust**: 1.70 or later (recommended: latest stable)
- **Cargo**: Package manager (included with Rust)
- **Git**: For cloning the repository
- **System**: Linux, macOS, or Windows with WSL2

### Installation

1. **Clone the repository:**
```bash
git clone https://github.com/PolyTorus/polytorus.git
cd polytorus
```

2. **Build the project:**
```bash
# Development build
cargo build

# Optimized release build
cargo build --release
```

3. **Run comprehensive tests:**
```bash
cargo test
```

4. **Verify installation:**
```bash
./target/release/polytorus --help
```

### 🖥️ Usage

#### Command Line Interface

**Start a blockchain node:**
```bash
./target/release/polytorus start-node --port 8333
```

**Create a new wallet with cryptographic choice:**
```bash
# Create wallet with traditional ECDSA (current standard)
./target/release/polytorus create-wallet --name my-wallet --type ECDSA

# Create wallet with quantum-resistant FN-DSA (post-quantum ready)
./target/release/polytorus create-wallet --name my-quantum-wallet --type FNDSA
```

**List wallet addresses:**
```bash
./target/release/polytorus list-addresses
```

**Start mining:**
```bash
./target/release/polytorus start-miner --threads 4
```

**Print blockchain information:**
```bash
./target/release/polytorus print-chain
```

**Reindex blockchain:**
```bash
./target/release/polytorus reindex
```

#### Web Interface

**Start the web server:**
```bash
./target/release/polytorus start-webserver --port 8080
```

The web interface provides a RESTful API at `http://localhost:8080` with endpoints for:
- `/api/wallets` - Wallet management
- `/api/addresses` - Address listing
- `/api/blockchain` - Blockchain information
- `/api/mining` - Mining operations
- `/api/network` - Network status

## 📝 Smart Contracts

PolyTorus features a sophisticated WebAssembly (WASM) based smart contract platform that provides:

### Key Features
- **Sandboxed Execution**: Secure isolation with resource limits
- **Gas Metering**: Precise computation cost tracking
- **State Access**: Direct blockchain state interaction
- **Cryptographic Functions**: Built-in crypto operations
- **Memory Management**: Efficient WASM linear memory handling

### Example Smart Contracts

The repository includes production-ready example smart contracts:

- **`contracts/counter.wat`**: A simple counter contract demonstrating state management
- **`contracts/token.wat`**: A comprehensive token implementation with transfer and balance functionality

### Contract Development
```bash
# Compile a smart contract
wasmtime compile contract.wat -o contract.wasm

# Deploy via CLI (coming soon)
./target/release/polytorus deploy-contract contract.wasm
```

For comprehensive smart contract development documentation, see [SMART_CONTRACTS.md](SMART_CONTRACTS.md).

## 🧪 Development

### Project Structure

```
polytorus/
├── src/
│   ├── modular/          # 🏗️ Modular architecture implementation
│   │   ├── consensus.rs      # Consensus layer protocols
│   │   ├── execution.rs      # Transaction execution engine
│   │   ├── settlement.rs     # Settlement and finality
│   │   ├── data_availability.rs # Data availability protocols
│   │   └── orchestrator.rs   # Layer coordination
│   ├── smart_contract/   # 🔧 WASM smart contract engine
│   │   ├── engine.rs         # Contract execution engine
│   │   ├── state.rs          # Contract state management
│   │   └── types.rs          # Contract type definitions
│   ├── network/          # 🌐 P2P networking layer
│   │   ├── p2p.rs           # Peer-to-peer protocols
│   │   ├── manager.rs       # Network management
│   │   └── server.rs        # Network server
│   ├── crypto/           # 🔐 Cryptographic functions and wallets
│   │   ├── ecdsa.rs         # ECDSA implementation
│   │   ├── fndsa.rs         # Quantum-resistant FN-DSA
│   │   ├── wallets.rs       # Wallet management
│   │   └── transaction.rs   # Transaction signing
│   ├── blockchain/       # ⛓️ Core blockchain data structures
│   │   ├── block.rs         # Block structure
│   │   ├── blockchain.rs    # Blockchain management
│   │   └── utxoset.rs      # UTXO set management
│   ├── command/          # 💻 CLI command implementations
│   └── webserver/        # 🌍 HTTP API server
├── contracts/            # 📝 Example smart contracts
├── docs/                 # 📚 Comprehensive documentation
└── tests/               # 🧪 Integration tests
```

### 🧪 Testing

**Run all tests:**
```bash
cargo test
```

**Run specific test suites:**
```bash
# Modular architecture tests
cargo test modular

# Smart contract tests  
cargo test smart_contract

# Network layer tests
cargo test network

# Cryptography tests
cargo test crypto
```

**Run tests with output:**
```bash
cargo test -- --nocapture
```

**Performance benchmarks:**
```bash
cargo bench
```

### 🔧 Development Commands

**Check code formatting:**
```bash
cargo fmt --check
```

**Run linter:**
```bash
cargo clippy -- -D warnings
```

**Generate documentation:**
```bash
cargo doc --open
```

## 🎯 Goals

- **🛡️ Post-Quantum Ready**: Pioneer the quantum-resistant blockchain era with flexible cryptographic wallet architecture that allows seamless transition from traditional to quantum-resistant algorithms
- **🔑 Cryptographic Freedom**: Empower users to choose their preferred cryptographic security level - from current ECDSA compatibility to future-proof FN-DSA quantum resistance
- **🏗️ Modular Design**: Create a flexible, pluggable architecture for diverse use cases and future protocol upgrades
- **🔧 Smart Contracts**: Provide a secure and efficient WASM-based contract execution environment
- **🌐 Network Security**: Implement secure and efficient networking and wallet systems that scale with cryptographic evolution
- **⚡ Performance**: Explore novel consensus algorithms optimized for the post-quantum computing landscape
- **🔍 Formal Verification**: Conduct rigorous security verification of blockchain components across multiple cryptographic paradigms

## 🤝 Contributing

We welcome contributions from the community! Please follow these steps:

1. **Fork the repository** on GitHub
2. **Create a feature branch** (`git checkout -b feature/amazing-feature`)
3. **Make your changes** following our coding standards
4. **Add tests** for new functionality
5. **Ensure all tests pass** (`cargo test`)
6. **Format your code** (`cargo fmt`)
7. **Submit a pull request** with a clear description

### Code Standards
- Follow Rust naming conventions and best practices
- Add comprehensive documentation for public APIs
- Include unit tests for new functions
- Maintain backward compatibility when possible
- Run `rustfmt` and `clippy` before submitting

### Pull Request Guidelines
In this project, `rustfmt` and `clippy` will be run at PR merge time, and unified code will be added to the `main` branch. Therefore, you are free to use your own code formatter and linter during development.

When building a PR, it may be easier for others to help if you issue an Issue first. Please consider submitting an Issue before making significant changes.

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 📚 Documentation

### Core Documentation
- [📖 Modular Architecture](docs/MODULAR_ARCHITECTURE.md) - Detailed architecture overview
- [📝 Smart Contracts](SMART_CONTRACTS.md) - Smart contract development guide  
- [💻 CLI Commands](docs/CLI_COMMANDS.md) - Complete CLI reference *(coming soon)*

### Additional Resources
- [🚀 Getting Started Guide](docs/GETTING_STARTED.md) *(coming soon)*
- [🔧 API Reference](docs/API_REFERENCE.md) *(coming soon)*
- [🏗️ Developer Guide](docs/DEVELOPER_GUIDE.md) *(coming soon)*
- [🔐 Security Considerations](docs/SECURITY.md) *(coming soon)*

## 🆘 Support

- **Issues**: Report bugs and feature requests on [GitHub Issues](https://github.com/PolyTorus/polytorus/issues)
- **Discussions**: Join community discussions on [GitHub Discussions](https://github.com/PolyTorus/polytorus/discussions)
- **Documentation**: Comprehensive docs available in the `docs/` directory

---

<div align="center">
    <p><strong>Built with ❤️ using Rust</strong> | <strong>Powered by WebAssembly</strong> | <strong>Ready for the Post-Quantum Era</strong></p>
    <p><em>Empowering cryptographic choice for a quantum-safe future</em></p>
</div>
