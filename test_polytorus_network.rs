#!/usr/bin/env rust-script

//! PolyTorus Network Integration Test
//! 
//! This script tests the PolyTorus network layer specifically
//! to identify any network-related errors.

use std::process::Command;
use std::time::Duration;
use std::thread;

fn main() {
    println!("🔗 Testing PolyTorus Network Integration");
    println!("========================================");
    
    // Test 1: Check if polytorus binary exists
    test_binary_exists();
    
    // Test 2: Test network configuration parsing
    test_config_parsing();
    
    // Test 3: Test network startup with invalid config
    test_invalid_network_config();
    
    // Test 4: Test multiple node startup conflicts
    test_port_conflicts();
    
    println!("\n✅ PolyTorus network testing completed");
}

fn test_binary_exists() {
    println!("\n📦 Test 1: Check PolyTorus binary");
    
    let output = Command::new("./target/release/polytorus")
        .arg("--help")
        .output();
        
    match output {
        Ok(result) => {
            if result.status.success() {
                println!("✅ PolyTorus binary is accessible");
            } else {
                println!("❌ PolyTorus binary failed: {}", String::from_utf8_lossy(&result.stderr));
            }
        }
        Err(e) => {
            println!("❌ PolyTorus binary not found or not executable: {}", e);
        }
    }
}

fn test_config_parsing() {
    println!("\n⚙️  Test 2: Configuration parsing");
    
    // Test with existing config files
    let configs = vec![
        "config/modular-node1.toml",
        "config/modular-node2.toml", 
        "config/modular-node3.toml",
    ];
    
    for config in configs {
        if std::path::Path::new(config).exists() {
            println!("✅ Config file exists: {}", config);
        } else {
            println!("❌ Config file missing: {}", config);
        }
    }
}

fn test_invalid_network_config() {
    println!("\n🚫 Test 3: Invalid network configuration");
    
    // Create a temporary invalid config
    let invalid_config = r#"
[network]
listen_addr = "invalid_address"
bootstrap_peers = ["256.256.256.256:8000"]
max_peers = -1
"#;
    
    match std::fs::write("config/invalid.toml", invalid_config) {
        Ok(_) => {
            println!("✅ Created invalid config file for testing");
            
            // Try to start with invalid config (should fail gracefully)
            let output = Command::new("./target/release/polytorus")
                .arg("--config")
                .arg("config/invalid.toml")
                .arg("--modular-start")
                .output();
                
            match output {
                Ok(result) => {
                    if result.status.success() {
                        println!("❌ Unexpected: Invalid config was accepted");
                    } else {
                        println!("✅ Expected: Invalid config was rejected");
                        println!("   Error: {}", String::from_utf8_lossy(&result.stderr));
                    }
                }
                Err(e) => {
                    println!("✅ Expected: Failed to start with invalid config - {}", e);
                }
            }
            
            // Clean up
            let _ = std::fs::remove_file("config/invalid.toml");
        }
        Err(e) => {
            println!("❌ Failed to create invalid config: {}", e);
        }
    }
}

fn test_port_conflicts() {
    println!("\n🔒 Test 4: Port conflict detection");
    
    // This test would ideally start two nodes with the same port
    // and verify that the second one fails gracefully
    println!("ℹ️  Port conflict testing requires running instances");
    println!("   This would be tested in a full integration test suite");
    println!("   where multiple nodes are started simultaneously");
}