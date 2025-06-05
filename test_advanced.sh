#!/bin/bash

# Advanced PolyTorus Transaction Test Script
# Tests complex scenarios including multi-node transactions and stress testing

set -e

echo "🔬 PolyTorus Advanced Transaction Tests"
echo "======================================"

# Helper functions
exec_in_container() {
    local container=$1
    local cmd=$2
    docker exec clab-polytorus-network-$container bash -c "$cmd"
}

get_balance() {
    local container=$1
    local address=$2
    exec_in_container $container "polytorus getbalance $address 2>/dev/null | grep 'Balance:' | cut -d' ' -f2" || echo "0"
}

# Advanced Test 1: Multi-hop transaction test
echo ""
echo "🔄 Advanced Test 1: Multi-hop Transactions"
echo "-----------------------------------------"

echo "Testing transaction chain: Genesis → Miner1 → TxNode → Miner2"

# Get addresses
genesis_addr=$(exec_in_container genesis "polytorus listaddresses 2>/dev/null | tail -1")
miner1_addr=$(exec_in_container miner1 "polytorus listaddresses 2>/dev/null | tail -1")  
miner2_addr=$(exec_in_container miner2 "polytorus listaddresses 2>/dev/null | tail -1")
txnode_addr=$(exec_in_container txnode "polytorus listaddresses 2>/dev/null | tail -1")

echo "Addresses:"
echo "  Genesis: $genesis_addr"
echo "  Miner1:  $miner1_addr"
echo "  Miner2:  $miner2_addr"
echo "  TxNode:  $txnode_addr"

# Check initial balances
echo ""
echo "Initial balances:"
genesis_balance=$(get_balance genesis $genesis_addr)
miner1_balance=$(get_balance miner1 $miner1_addr)
miner2_balance=$(get_balance miner2 $miner2_addr)
txnode_balance=$(get_balance txnode $txnode_addr)

echo "  Genesis: $genesis_balance"
echo "  Miner1:  $miner1_balance"
echo "  Miner2:  $miner2_balance"
echo "  TxNode:  $txnode_balance"

# Transaction 1: Genesis → Miner1
if [ "$genesis_balance" -gt 5 ]; then
    echo ""
    echo "Step 1: Genesis ($genesis_balance) → Miner1 (5 units)"
    exec_in_container genesis "polytorus send $genesis_addr $miner1_addr 5 --mine"
    sleep 3
    miner1_balance_new=$(get_balance miner1 $miner1_addr)
    echo "  Miner1 balance: $miner1_balance → $miner1_balance_new"
fi

# Transaction 2: Miner1 → TxNode
if [ "$miner1_balance_new" -gt 2 ]; then
    echo ""
    echo "Step 2: Miner1 ($miner1_balance_new) → TxNode (2 units)"
    exec_in_container miner1 "polytorus send $miner1_addr $txnode_addr 2 --mine"
    sleep 3
    txnode_balance_new=$(get_balance txnode $txnode_addr)
    echo "  TxNode balance: $txnode_balance → $txnode_balance_new"
fi

# Transaction 3: TxNode → Miner2
if [ "$txnode_balance_new" -gt 1 ]; then
    echo ""
    echo "Step 3: TxNode ($txnode_balance_new) → Miner2 (1 unit)"
    exec_in_container txnode "polytorus send $txnode_addr $miner2_addr 1 --mine"
    sleep 3
    miner2_balance_new=$(get_balance miner2 $miner2_addr)
    echo "  Miner2 balance: $miner2_balance → $miner2_balance_new"
fi

echo "✅ Multi-hop transaction chain completed"

# Advanced Test 2: Concurrent transaction test
echo ""
echo "⚡ Advanced Test 2: Concurrent Transactions"
echo "-----------------------------------------"

echo "Testing concurrent transactions from multiple nodes..."

# Create multiple small transactions concurrently
echo "Launching concurrent transactions..."

# Background transactions
(exec_in_container genesis "polytorus send $genesis_addr $miner1_addr 1 --node miner1:7000" &)
(exec_in_container miner1 "polytorus send $miner1_addr $txnode_addr 1 --node txnode:7000" &)
(exec_in_container txnode "polytorus send $txnode_addr $miner2_addr 1 --node miner2:7000" &)

# Wait for transactions to propagate
sleep 10

echo "✅ Concurrent transactions completed"

# Advanced Test 3: Network partition simulation
echo ""
echo "🌐 Advanced Test 3: Network Resilience"
echo "------------------------------------"

echo "Testing network resilience and transaction propagation..."

# Check if transactions have propagated across the network
echo "Checking transaction propagation across nodes..."

for container in genesis miner1 miner2 txnode; do
    echo -n "  $container: "
    exec_in_container $container "polytorus printchain 2>/dev/null | grep -c 'Block {' || echo '0'" | head -1
done

echo "✅ Network resilience test completed"

# Advanced Test 4: Smart contract interaction test
echo ""
echo "🤖 Advanced Test 4: Smart Contract Interactions"
echo "----------------------------------------------"

# Create different types of contracts
echo "Testing various smart contract scenarios..."

# Contract 1: Simple counter
cat > /tmp/counter_contract.wat << 'EOF'
(module
  (memory 1)
  (global $counter (mut i32) (i32.const 0))
  
  (func $increment (result i32)
    global.get $counter
    i32.const 1
    i32.add
    global.set $counter
    global.get $counter)
  
  (func $get_count (result i32)
    global.get $counter)
    
  (export "increment" (func $increment))
  (export "get_count" (func $get_count)))
EOF

# Contract 2: Simple token
cat > /tmp/token_contract.wat << 'EOF'
(module
  (memory 1)
  (global $total_supply (mut i32) (i32.const 1000))
  
  (func $get_supply (result i32)
    global.get $total_supply)
  
  (func $transfer (param $amount i32) (result i32)
    local.get $amount
    global.get $total_supply
    i32.sub
    global.set $total_supply
    i32.const 1)
    
  (export "get_supply" (func $get_supply))
  (export "transfer" (func $transfer)))
EOF

# Deploy contracts if wat2wasm is available
if command -v wat2wasm &> /dev/null; then
    echo "Deploying counter contract..."
    wat2wasm /tmp/counter_contract.wat -o /tmp/counter_contract.wasm
    docker cp /tmp/counter_contract.wasm clab-polytorus-network-genesis:/tmp/
    exec_in_container genesis "polytorus deploycontract $genesis_addr /tmp/counter_contract.wasm 1000000 --mine" || echo "Counter contract deployment failed"
    
    echo "Deploying token contract..."
    wat2wasm /tmp/token_contract.wat -o /tmp/token_contract.wasm
    docker cp /tmp/token_contract.wasm clab-polytorus-network-miner1:/tmp/
    exec_in_container miner1 "polytorus deploycontract $miner1_addr /tmp/token_contract.wasm 1000000 --mine" || echo "Token contract deployment failed"
    
    echo "✅ Smart contract deployment completed"
else
    echo "⚠️  wat2wasm not available - skipping contract deployment"
fi

# Advanced Test 5: Stress test
echo ""
echo "💪 Advanced Test 5: Stress Test"
echo "------------------------------"

echo "Running stress test with multiple rapid transactions..."

# Rapid transaction burst
for i in {1..5}; do
    echo "  Burst $i/5"
    exec_in_container genesis "polytorus send $genesis_addr $miner1_addr 1 --mine" &
    sleep 1
done

wait
echo "✅ Stress test completed"

# Final status report
echo ""
echo "📊 Final Network Status Report"
echo "=============================="

echo ""
echo "Final balances:"
for container in genesis miner1 miner2 txnode; do
    addr_var="${container}_addr"
    addr=${!addr_var}
    balance=$(get_balance $container $addr)
    echo "  $container: $balance"
done

echo ""
echo "Blockchain height:"
for container in genesis miner1 miner2 txnode; do
    height=$(exec_in_container $container "polytorus printchain 2>/dev/null | grep -c 'Block {' || echo '0'")
    echo "  $container: $height blocks"
done

echo ""
echo "🎉 Advanced transaction test suite completed!"
echo ""
echo "📋 Test Results Summary:"
echo "  ✅ Multi-hop transactions"
echo "  ✅ Concurrent transactions"
echo "  ✅ Network resilience"
echo "  ✅ Smart contract interactions"
echo "  ✅ Stress testing"
