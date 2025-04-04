<div align="center">
    <h1>PolyTorus</h1>
    <p>A Quantum-Resistant Blockchain Implementation</P>
</div>

PolyTorus is a blockchain project designed to withstand quantum cryptography attacks by Implementing quantum-resistant cryptography algorithms alongside traditional ones.

## FEATURES
* Quantum-Resistant Cryptography: Implements FN-DSA for quantum-resistant digital signatures.
* Dual Cryptography Support: Use both ECDSA (traditional) and FN-DSA encryption methods.
* Complete Blockchain Implementation:
    * Block creation and mining with proof-of-work.
    * Transaction management with UTXO model.
    * Wallet creation and management.
    * Blockchain state persistence using sled database.
* Nwtworking Capabilities:
    * Peer-to-peer networking using TCP.
    * Message broadcasting and handling.
* Web Interface
* CLI Interface

## Goals
* Develop a quantum-resistant public blockchain
* Utilize quantum-resistant cryptography such as Falcon
* Implement a secure and efficient network and wallet
* Explore novel consensus algorithms
* Conduct formal verification of the blockchain's security

## Installation
### Prerequisites
* Rust 1.56 or later
* Cargo

### Building from Source
1. Clone the repository:
```bash
git clone https://github.com/PolyTorus/polytorus.git
cd polytorus
```

2. Build the project:
```bash
cargo build --release
```

3. Run the project:
```bash
./target/realease/polytorus 
```

## Usage
### Basic Commands
* Create a new wallet:
```bash
cargo run createwallet [ECDSA | FNDSA]
```

* List all wallet addresses:
```bash
cargo run listaddresses
```

* Create a new Blockchain with genesis block:
```bash
cargo run createblockchain <address>
```

* Check balance of an address:
```bash
cargo run balance <address>
```

* Print all blocks in the chain
```bash
cargo run printchain
```

* Reindex UTXO set:
```bash
cargo run reindex
```

## Pull Request

In this project, `rustfmt` and `clippy` will be run at PR merge time, and unified code will be added to the `main` branch. Therefore, you are free to use your own code formatter and linter.
When building a PR, it may be easier for others to help if you issue an Issue first. Please consider submitting an Issue first.
Other rules are not yet strictly defined. We are also looking for people to decide the rules for CONTRIBUTING!

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
