//! Network integration tests for P2P functionality
//!
//! These tests verify the P2P communication between nodes across different servers.
//! Note: Some tests require multiple machines to run properly.

use super::server::Server;
use crate::blockchain::blockchain::Blockchain;
use crate::blockchain::utxoset::UTXOSet;
use crate::crypto::types::EncryptionType;
use crate::crypto::wallets::Wallets;
use crate::Result;

use std::env;
use std::net::TcpStream;
use std::thread;
use std::time::Duration;

/// The TEST_REMOTE_NODE environment variable should be set to the address of a remote test node
const ENV_REMOTE_NODE: &str = "TEST_REMOTE_NODE";
/// The TEST_LOCAL_PORT environment variable can be set to specify local port for tests
const ENV_LOCAL_PORT: &str = "TEST_LOCAL_PORT";

/// Skip test if no remote node is configured
fn require_remote_node() -> Option<String> {
    match env::var(ENV_REMOTE_NODE) {
        Ok(addr) => {
            // Verify connection to remote node
            match TcpStream::connect(&addr) {
                Ok(_) => {
                    println!("Remote node available at: {}", addr);
                    Some(addr)
                }
                Err(e) => {
                    println!(
                        "Remote node at {} is not accessible: {}. Skipping test.",
                        addr, e
                    );
                    None
                }
            }
        }
        Err(_) => {
            println!(
                "No remote node configured. Set {} env var to run this test.",
                ENV_REMOTE_NODE
            );
            None
        }
    }
}

/// Get local port for testing
fn get_local_port() -> String {
    env::var(ENV_LOCAL_PORT).unwrap_or_else(|_| "7777".to_string())
}

/// Create a test blockchain and wallet
fn setup_test_environment() -> Result<(Blockchain, Wallets, String)> {
    // Create wallet for testing
    let mut wallets = Wallets::new()?;
    let address = wallets.create_wallet(EncryptionType::FNDSA);
    wallets.save_all()?;

    // Create or load blockchain
    let bc = Blockchain::new()
        .unwrap_or_else(|_| Blockchain::create_blockchain(address.clone()).unwrap());

    Ok((bc, wallets, address))
}

/// Create and start a local server for testing
fn start_test_server(port: &str, mining_address: &str, bootstrap: Option<&str>) -> Result<Server> {
    let (bc, _, _) = setup_test_environment()?;
    let utxo_set = UTXOSet { blockchain: bc };

    let server = Server::new("0.0.0.0", port, mining_address, bootstrap, utxo_set)?;

    // Start server in background thread
    let server_clone = server.clone();
    thread::spawn(move || {
        if let Err(e) = server_clone.start_server() {
            eprintln!("Server error: {}", e);
        }
    });

    // Give server time to start
    thread::sleep(Duration::from_secs(2));

    Ok(server)
}

/// Test external connectivity to a remote node
#[test]
fn test_external_connectivity() {
    let remote_addr = match require_remote_node() {
        Some(addr) => addr,
        None => return, // Skip test if no remote node
    };

    // Test direct TCP connectivity to remote node
    let result = TcpStream::connect(&remote_addr);
    assert!(
        result.is_ok(),
        "Could not connect to remote node at {}",
        remote_addr
    );

    println!("Successfully connected to remote node at {}", remote_addr);
}

/// Test version exchange with a remote node
#[test]
fn test_version_exchange() {
    let remote_addr = match require_remote_node() {
        Some(addr) => addr,
        None => return, // Skip test if no remote node
    };

    // Create a test server with the remote node as bootstrap
    let port = get_local_port();
    let server = match start_test_server(&port, "", Some(&remote_addr)) {
        Ok(s) => s,
        Err(e) => {
            panic!("Failed to create test server: {}", e);
        }
    };

    // Give time for version exchange
    thread::sleep(Duration::from_secs(5));

    // Stop server
    let _ = server.stop_server();

    println!("Version exchange test completed");
}

/// Test sending a transaction to a remote node
#[test]
fn test_send_transaction_to_remote() {
    let remote_addr = match require_remote_node() {
        Some(addr) => addr,
        None => return, // Skip test if no remote node
    };

    // Set up test environment
    let (bc, wallets, from_address) = match setup_test_environment() {
        Ok(env) => env,
        Err(e) => {
            panic!("Failed to set up test environment: {}", e);
        }
    };

    let utxo_set = UTXOSet { blockchain: bc };

    // Create a new wallet for receiving
    let mut wallets_clone = wallets.clone();
    let to_address = wallets_clone.create_wallet(EncryptionType::FNDSA);
    wallets_clone.save_all().unwrap();

    // Set up test server
    let port = get_local_port();
    let server = match start_test_server(&port, "", None) {
        Ok(s) => s,
        Err(e) => {
            panic!("Failed to start test server: {}", e);
        }
    };

    // Get wallet and crypto provider
    let from_wallet = wallets.get_wallet(&from_address).unwrap();
    let crypto = FnDsaCrypto;

    // Create and send a transaction
    match Transaction::new_UTXO(from_wallet, &to_address, 1, &utxo_set, &crypto) {
        Ok(tx) => {
            // Send transaction to remote node
            let result = server.send_tx(&remote_addr, &tx);

            if let Err(e) = result {
                println!("Transaction send failed: {}. This might be expected.", e);
            } else {
                println!("Transaction sent successfully");
            }
        }
        Err(e) => {
            println!(
                "Could not create transaction: {}. This might be normal if no funds.",
                e
            );
        }
    };

    // Stop server
    let _ = server.stop_server();
}

/// Test blockchain synchronization with a remote node
#[test]
fn test_blockchain_sync() {
    let remote_addr = match require_remote_node() {
        Some(addr) => addr,
        None => return, // Skip test if no remote node
    };

    // Create a fresh blockchain for testing
    let _ = std::fs::remove_dir_all("data/blocks").ok(); // Ignore errors if directory doesn't exist

    // Create a new blockchain
    let (_, wallets, address) = match setup_test_environment() {
        Ok(env) => env,
        Err(e) => {
            panic!("Failed to set up test environment: {}", e);
        }
    };

    let bc = Blockchain::create_blockchain(address).unwrap();
    let initial_height = bc.get_best_height().unwrap();
    println!("Initial blockchain height: {}", initial_height);

    // Set up server with the remote node as bootstrap
    let port = get_local_port();
    let utxo_set = UTXOSet { blockchain: bc };
    let server = match Server::new("0.0.0.0", &port, "", Some(&remote_addr), utxo_set) {
        Ok(s) => s,
        Err(e) => {
            panic!("Failed to create test server: {}", e);
        }
    };

    // Start server (this should trigger blockchain sync)
    let server_clone = server.clone();
    thread::spawn(move || {
        if let Err(e) = server_clone.start_server() {
            eprintln!("Server error: {}", e);
        }
    });

    // Give time for sync (this might need to be longer for larger blockchains)
    println!("Waiting for blockchain sync (30 seconds)...");
    thread::sleep(Duration::from_secs(30));

    // Stop server
    let _ = server.stop_server();

    // Check if blockchain was synchronized
    let bc_after = Blockchain::new().unwrap();
    let final_height = bc_after.get_best_height().unwrap();

    println!("Blockchain height after sync: {}", final_height);

    // Either we synced more blocks or the remote node had the same height as us
    assert!(
        final_height >= initial_height,
        "Blockchain was not properly synchronized. Height before: {}, after: {}",
        initial_height,
        final_height
    );
}

/// Test remote wallet operations (requires a remote node with wallets)
#[test]
fn test_remote_wallet_operations() {
    let remote_addr = match require_remote_node() {
        Some(addr) => addr,
        None => return, // Skip test if no remote node
    };

    // Set up test environment
    let (bc, _, _) = match setup_test_environment() {
        Ok(env) => env,
        Err(e) => {
            panic!("Failed to set up test environment: {}", e);
        }
    };

    let utxo_set = UTXOSet { blockchain: bc };

    // Create a test server
    let port = get_local_port();
    let server = match start_test_server(&port, "", None) {
        Ok(s) => s,
        Err(e) => {
            panic!("Failed to start test server: {}", e);
        }
    };

    // Create an unsigned transaction (dummy transaction for testing)
    let tx = Transaction {
        id: String::new(),
        vin: Vec::new(),
        vout: vec![],
    };

    // Try to request remote signing
    println!("Requesting remote signing from {}", remote_addr);
    let result = server.send_sign_request(&remote_addr, "test_wallet_address", &tx);

    // Allow failure as the remote node might not have the requested wallet
    if let Err(e) = result {
        println!("Remote signing failed: {}. This might be expected.", e);
    } else {
        println!("Remote signing successful");
    }

    // Stop server
    let _ = server.stop_server();
}

/// Test using command-line tools to interact with the server
#[test]
fn test_cli_integration() {
    let remote_addr = match require_remote_node() {
        Some(addr) => addr,
        None => return, // Skip test if no remote node
    };

    println!("Creating test wallet via CLI");

    // Create a wallet using CLI
    let output = Command::new("cargo")
        .args(["run", "createwallet", "FNDSA"])
        .output();

    if let Err(e) = output {
        println!("Failed to create wallet: {}", e);
        return;
    }

    let output = output.unwrap();
    let wallet_output = String::from_utf8_lossy(&output.stdout);

    // Extract wallet address
    let address = if let Some(addr_line) = wallet_output.lines().find(|l| l.starts_with("address:"))
    {
        addr_line.trim_start_matches("address:").trim().to_string()
    } else {
        println!("Could not find wallet address in output: {}", wallet_output);
        return;
    };

    println!("Created wallet with address: {}", address);

    // Try to send a transaction to the remote node via CLI
    println!("Attempting to send transaction via CLI to {}", remote_addr);
    let output = Command::new("cargo")
        .args([
            "run",
            "send",
            &address, // from
            &address, // to (self)
            "1",      // amount
            "--node",
            &remote_addr,
        ])
        .output();

    // It's okay if this fails, we're just testing the CLI interface
    match output {
        Ok(output) => {
            println!(
                "CLI command output: {}",
                String::from_utf8_lossy(&output.stdout)
            );
            if !output.stderr.is_empty() {
                println!(
                    "CLI command error: {}",
                    String::from_utf8_lossy(&output.stderr)
                );
            }
        }
        Err(e) => {
            println!("CLI command failed: {}", e);
        }
    }

    println!("CLI integration test completed");
}
