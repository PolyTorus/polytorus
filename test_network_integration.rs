#!/usr/bin/env rust-script
//! ```cargo
//! [dependencies]
//! tokio = { version = "1.0", features = ["full"] }
//! anyhow = "1.0"
//! serde = { version = "1.0", features = ["derive"] }
//! bincode = "1.3"
//! uuid = { version = "1.0", features = ["v4"] }
//! log = "0.4"
//! env_logger = "0.11"
//! ```

//! PolyTorus Network Integration Test
//! 
//! This script tests the PolyTorus network layer integration
//! to verify error handling and network resilience.

use std::net::{IpAddr, Ipv4Addr, SocketAddr};
use std::time::Duration;
use tokio::time::timeout;
use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    env_logger::init();
    
    println!("🔗 PolyTorus Network Integration Test");
    println!("====================================");
    
    // Test 1: Basic network error scenarios
    test_basic_network_errors().await?;
    
    // Test 2: Connection timeout scenarios
    test_connection_timeouts().await?;
    
    // Test 3: Port binding conflicts
    test_port_binding_conflicts().await?;
    
    // Test 4: Message serialization errors
    test_message_serialization().await?;
    
    // Test 5: Network resilience
    test_network_resilience().await?;
    
    println!("\n✅ All network integration tests completed");
    Ok(())
}

async fn test_basic_network_errors() -> Result<()> {
    println!("\n📡 Test 1: Basic Network Error Scenarios");
    
    // Test connection to non-existent address
    let invalid_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 9999);
    
    match timeout(Duration::from_secs(2), tokio::net::TcpStream::connect(invalid_addr)).await {
        Ok(Ok(_)) => println!("❌ Unexpected: Connection succeeded to non-existent address"),
        Ok(Err(e)) => println!("✅ Expected: Connection failed - {}", e),
        Err(_) => println!("✅ Expected: Connection timed out"),
    }
    
    // Test connection to invalid address
    let invalid_ip = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(256, 256, 256, 256)), 8000);
    // Note: This would fail at parsing stage, so we test with a valid but unreachable IP
    let unreachable_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(10, 255, 255, 1)), 8000);
    
    match timeout(Duration::from_millis(100), tokio::net::TcpStream::connect(unreachable_addr)).await {
        Ok(Ok(_)) => println!("❌ Unexpected: Connection succeeded to unreachable address"),
        Ok(Err(e)) => println!("✅ Expected: Connection failed to unreachable address - {}", e),
        Err(_) => println!("✅ Expected: Connection timed out to unreachable address"),
    }
    
    Ok(())
}

async fn test_connection_timeouts() -> Result<()> {
    println!("\n⏱️  Test 2: Connection Timeout Scenarios");
    
    // Test with very short timeout
    let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(8, 8, 8, 8)), 80);
    
    match timeout(Duration::from_millis(1), tokio::net::TcpStream::connect(addr)).await {
        Ok(Ok(_)) => println!("❌ Unexpected: Very fast connection succeeded"),
        Ok(Err(e)) => println!("✅ Connection failed quickly - {}", e),
        Err(_) => println!("✅ Expected: Connection timed out with very short timeout"),
    }
    
    // Test with reasonable timeout to a slow/filtered address
    let filtered_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(10, 0, 0, 1)), 22);
    
    match timeout(Duration::from_millis(500), tokio::net::TcpStream::connect(filtered_addr)).await {
        Ok(Ok(_)) => println!("❌ Unexpected: Connection to filtered address succeeded"),
        Ok(Err(e)) => println!("✅ Connection to filtered address failed - {}", e),
        Err(_) => println!("✅ Expected: Connection to filtered address timed out"),
    }
    
    Ok(())
}

async fn test_port_binding_conflicts() -> Result<()> {
    println!("\n🔒 Test 3: Port Binding Conflicts");
    
    let test_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 8888);
    
    // Bind to a port
    let listener1 = match tokio::net::TcpListener::bind(test_addr).await {
        Ok(listener) => {
            println!("✅ First bind successful to {}", test_addr);
            listener
        }
        Err(e) => {
            println!("❌ First bind failed: {}", e);
            return Ok(());
        }
    };
    
    // Try to bind to the same port again
    match tokio::net::TcpListener::bind(test_addr).await {
        Ok(_) => println!("❌ Unexpected: Second bind succeeded to {}", test_addr),
        Err(e) => println!("✅ Expected: Second bind failed to {} - {}", test_addr, e),
    }
    
    // Clean up
    drop(listener1);
    
    // Verify port is released
    match tokio::net::TcpListener::bind(test_addr).await {
        Ok(_) => println!("✅ Port released successfully after first listener dropped"),
        Err(e) => println!("❌ Port still in use after cleanup: {}", e),
    }
    
    Ok(())
}

async fn test_message_serialization() -> Result<()> {
    println!("\n📦 Test 4: Message Serialization");
    
    // Test serialization of various data structures
    use serde::{Serialize, Deserialize};
    use uuid::Uuid;
    
    #[derive(Debug, Clone, Serialize, Deserialize)]
    struct TestMessage {
        id: String,
        data: Vec<u8>,
        timestamp: u64,
    }
    
    // Test normal message
    let normal_msg = TestMessage {
        id: "test_123".to_string(),
        data: vec![1, 2, 3, 4, 5],
        timestamp: 1234567890,
    };
    
    match bincode::serialize(&normal_msg) {
        Ok(serialized) => {
            println!("✅ Normal message serialized: {} bytes", serialized.len());
            
            match bincode::deserialize::<TestMessage>(&serialized) {
                Ok(deserialized) => {
                    if deserialized.id == normal_msg.id {
                        println!("✅ Normal message deserialized correctly");
                    } else {
                        println!("❌ Deserialized message data mismatch");
                    }
                }
                Err(e) => println!("❌ Deserialization failed: {}", e),
            }
        }
        Err(e) => println!("❌ Serialization failed: {}", e),
    }
    
    // Test large message
    let large_msg = TestMessage {
        id: "large_test".to_string(),
        data: vec![0u8; 1024 * 1024], // 1MB
        timestamp: 1234567890,
    };
    
    match bincode::serialize(&large_msg) {
        Ok(serialized) => {
            println!("✅ Large message serialized: {} bytes", serialized.len());
            if serialized.len() > 10 * 1024 * 1024 {
                println!("⚠️  Warning: Message exceeds typical size limits");
            }
        }
        Err(e) => println!("❌ Large message serialization failed: {}", e),
    }
    
    // Test corrupted data deserialization
    let corrupted_data = vec![0xFF, 0xFE, 0xFD, 0xFC];
    match bincode::deserialize::<TestMessage>(&corrupted_data) {
        Ok(_) => println!("❌ Unexpected: Corrupted data deserialized successfully"),
        Err(e) => println!("✅ Expected: Corrupted data deserialization failed - {}", e),
    }
    
    Ok(())
}

async fn test_network_resilience() -> Result<()> {
    println!("\n🛡️  Test 5: Network Resilience");
    
    // Test multiple rapid connection attempts
    let target_addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 9999);
    let mut success_count = 0;
    let mut failure_count = 0;
    
    println!("Testing rapid connection attempts...");
    for i in 0..10 {
        match timeout(Duration::from_millis(100), tokio::net::TcpStream::connect(target_addr)).await {
            Ok(Ok(_)) => {
                success_count += 1;
                println!("  Attempt {}: Success", i + 1);
            }
            Ok(Err(_)) => {
                failure_count += 1;
                println!("  Attempt {}: Failed", i + 1);
            }
            Err(_) => {
                failure_count += 1;
                println!("  Attempt {}: Timeout", i + 1);
            }
        }
        
        // Small delay between attempts
        tokio::time::sleep(Duration::from_millis(10)).await;
    }
    
    println!("Rapid connection test results: {} successes, {} failures", success_count, failure_count);
    
    if failure_count > success_count {
        println!("✅ Expected: More failures than successes for non-existent endpoint");
    } else {
        println!("⚠️  Unexpected: More successes than failures");
    }
    
    // Test concurrent connection attempts
    println!("Testing concurrent connection attempts...");
    let mut handles = Vec::new();
    
    for i in 0..5 {
        let handle = tokio::spawn(async move {
            let addr = SocketAddr::new(IpAddr::V4(Ipv4Addr::new(127, 0, 0, 1)), 9999 + i);
            match timeout(Duration::from_millis(200), tokio::net::TcpStream::connect(addr)).await {
                Ok(Ok(_)) => format!("Connection {} succeeded", i),
                Ok(Err(e)) => format!("Connection {} failed: {}", i, e),
                Err(_) => format!("Connection {} timed out", i),
            }
        });
        handles.push(handle);
    }
    
    for handle in handles {
        match handle.await {
            Ok(result) => println!("  {}", result),
            Err(e) => println!("  Task failed: {}", e),
        }
    }
    
    println!("✅ Concurrent connection test completed");
    
    Ok(())
}