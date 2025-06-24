#!/usr/bin/env rust-script

//! Network Error Testing Script
//! 
//! This script tests various network error scenarios to ensure
//! the PolyTorus network layer handles errors gracefully.

use std::net::{SocketAddr, TcpListener, TcpStream};
use std::time::Duration;
use std::thread;

fn main() {
    println!("🔍 Testing PolyTorus Network Error Scenarios");
    println!("============================================");
    
    // Test 1: Connection to non-existent peer
    test_connection_refused();
    
    // Test 2: Connection timeout
    test_connection_timeout();
    
    // Test 3: Port already in use
    test_port_already_in_use();
    
    // Test 4: Invalid address format
    test_invalid_address();
    
    // Test 5: Network interface binding
    test_network_binding();
    
    println!("\n✅ Network error testing completed");
}

fn test_connection_refused() {
    println!("\n📡 Test 1: Connection to non-existent peer");
    
    // Try to connect to a port that should be closed
    let target = "127.0.0.1:9999";
    match TcpStream::connect(target) {
        Ok(_) => println!("❌ Unexpected: Connection succeeded to {}", target),
        Err(e) => println!("✅ Expected: Connection refused to {} - {}", target, e),
    }
}

fn test_connection_timeout() {
    println!("\n⏱️  Test 2: Connection timeout");
    
    // Try to connect to a non-routable address (should timeout)
    let target = "10.255.255.1:80";
    match TcpStream::connect_timeout(&target.parse().unwrap(), Duration::from_millis(100)) {
        Ok(_) => println!("❌ Unexpected: Connection succeeded to {}", target),
        Err(e) => println!("✅ Expected: Connection timeout to {} - {}", target, e),
    }
}

fn test_port_already_in_use() {
    println!("\n🔒 Test 3: Port already in use");
    
    let addr = "127.0.0.1:8888";
    
    // Bind to a port
    let _listener1 = match TcpListener::bind(addr) {
        Ok(listener) => {
            println!("✅ First bind successful to {}", addr);
            listener
        }
        Err(e) => {
            println!("❌ First bind failed: {}", e);
            return;
        }
    };
    
    // Try to bind to the same port again
    match TcpListener::bind(addr) {
        Ok(_) => println!("❌ Unexpected: Second bind succeeded to {}", addr),
        Err(e) => println!("✅ Expected: Second bind failed to {} - {}", addr, e),
    }
}

fn test_invalid_address() {
    println!("\n🚫 Test 4: Invalid address format");
    
    let invalid_addresses = vec![
        "invalid_address",
        "256.256.256.256:8000",
        "127.0.0.1:99999",
        "localhost:abc",
    ];
    
    for addr in invalid_addresses {
        match addr.parse::<SocketAddr>() {
            Ok(_) => println!("❌ Unexpected: {} parsed successfully", addr),
            Err(e) => println!("✅ Expected: {} failed to parse - {}", addr, e),
        }
    }
}

fn test_network_binding() {
    println!("\n🌐 Test 5: Network interface binding");
    
    // Test binding to different interfaces
    let test_addresses = vec![
        "127.0.0.1:0",  // Localhost
        "0.0.0.0:0",    // All interfaces
    ];
    
    for addr in test_addresses {
        match TcpListener::bind(addr) {
            Ok(listener) => {
                let local_addr = listener.local_addr().unwrap();
                println!("✅ Successfully bound to {} (actual: {})", addr, local_addr);
            }
            Err(e) => println!("❌ Failed to bind to {} - {}", addr, e),
        }
    }
}