#!/bin/bash

echo "🚀 Starting PolyTorus Multi-Node Network Test"

# Clean up any existing processes
pkill -f "polytorus.*modular"
sleep 1

# Create data directories
mkdir -p data/node1 data/node2 data/node3

echo "📡 Starting Node 1 (Bootstrap)..."
RUST_LOG=debug ./target/release/polytorus --config config/modular-node1.toml --data-dir data/node1 --modular-start > logs/node1.log 2>&1 &
NODE1_PID=$!
sleep 3

echo "📡 Starting Node 2..."
RUST_LOG=debug ./target/release/polytorus --config config/modular-node2.toml --data-dir data/node2 --modular-start > logs/node2.log 2>&1 &
NODE2_PID=$!
sleep 3

echo "📡 Starting Node 3..."
RUST_LOG=debug ./target/release/polytorus --config config/modular-node3.toml --data-dir data/node3 --modular-start > logs/node3.log 2>&1 &
NODE3_PID=$!
sleep 5

echo "🔍 Checking network status..."
echo "Node 1 PID: $NODE1_PID"
echo "Node 2 PID: $NODE2_PID" 
echo "Node 3 PID: $NODE3_PID"

# Test network connectivity
echo "📊 Testing network for 30 seconds..."
sleep 30

echo "📝 Checking logs for errors..."
echo "=== Node 1 Logs ==="
tail -10 logs/node1.log

echo "=== Node 2 Logs ==="
tail -10 logs/node2.log

echo "=== Node 3 Logs ==="
tail -10 logs/node3.log

echo "🛑 Stopping all nodes..."
kill $NODE1_PID $NODE2_PID $NODE3_PID 2>/dev/null
sleep 2
pkill -f "polytorus.*modular" 2>/dev/null

echo "✅ Network test completed"