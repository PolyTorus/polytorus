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

## Install

This project uses `Cargo`. After cloning the project, you can run it by doing `cargo run`.

## Usage

- Create Wallet
```bash
cargo run createwallet
```

- Create Blockchain
```bash
cargo run createblockchain <address>
```

## Pull Request

In this project, `rustfmt` and `clippy` will be run at PR merge time, and unified code will be added to the `main` branch. Therefore, you are free to use your own code formatter and linter.
When building a PR, it may be easier for others to help if you issue an Issue first. Please consider submitting an Issue first.
Other rules are not yet strictly defined. We are also looking for people to decide the rules for CONTRIBUTING!

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.
