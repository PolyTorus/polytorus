#!/bin/bash

# Multi-Node Simulation Script for PolyTorus
# This script helps manage multiple node instances for testing

set -e

# Configuration
NUM_NODES=${1:-4}
BASE_PORT=${2:-9000}
BASE_P2P_PORT=${3:-8000}
SIMULATION_TIME=${4:-300}  # 5 minutes default

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo -e "${BLUE}🎭 PolyTorus Multi-Node Simulation${NC}"
echo -e "${BLUE}===================================${NC}"
echo -e "📊 Configuration:"
echo -e "   Nodes: ${NUM_NODES}"
echo -e "   Base Port: ${BASE_PORT}"
echo -e "   Base P2P Port: ${BASE_P2P_PORT}"
echo -e "   Simulation Time: ${SIMULATION_TIME}s"
echo ""

# Cleanup function
cleanup() {
    echo -e "\n${YELLOW}🧹 Cleaning up...${NC}"
    
    # Kill all background processes
    if [[ -f "/tmp/polytorus_pids.txt" ]]; then
        while read -r pid; do
            if kill -0 "$pid" 2>/dev/null; then
                echo -e "   Stopping process ${pid}"
                kill "$pid" 2>/dev/null || true
            fi
        done < "/tmp/polytorus_pids.txt"
        rm -f "/tmp/polytorus_pids.txt"
    fi
    
    # Clean up data directories
    if [[ -d "./data/simulation" ]]; then
        echo -e "   Cleaning up data directories"
        rm -rf "./data/simulation"
    fi
    
    echo -e "${GREEN}✅ Cleanup completed${NC}"
    exit 0
}

# Set up trap for cleanup
trap cleanup SIGINT SIGTERM EXIT

# Create data directories
echo -e "${BLUE}📁 Creating data directories...${NC}"
mkdir -p "./data/simulation"

# Generate node configurations
echo -e "${BLUE}⚙️  Generating node configurations...${NC}"
for ((i=0; i<NUM_NODES; i++)); do
    NODE_ID="node-$i"
    PORT=$((BASE_PORT + i))
    P2P_PORT=$((BASE_P2P_PORT + i))
    DATA_DIR="./data/simulation/$NODE_ID"
    
    mkdir -p "$DATA_DIR"
    
    # Create node-specific config file
    CONFIG_FILE="$DATA_DIR/config.toml"
    cat > "$CONFIG_FILE" << EOF
# Node $i Configuration
[execution]
gas_limit = 8000000
gas_price = 1

[consensus]
block_time = 10000
difficulty = 4
max_block_size = 1048576

[network]
listen_addr = "127.0.0.1:$P2P_PORT"
bootstrap_peers = [
EOF
    
    # Add bootstrap peers (previous nodes)
    for ((j=0; j<i; j++)); do
        PEER_PORT=$((BASE_P2P_PORT + j))
        echo "    \"127.0.0.1:$PEER_PORT\"," >> "$CONFIG_FILE"
    done
    
    cat >> "$CONFIG_FILE" << EOF
]
max_peers = 50
connection_timeout = 10
ping_interval = 30

[storage]
data_dir = "$DATA_DIR"
max_cache_size = 1073741824
sync_interval = 60

[logging]
level = "INFO"
output = "console"
EOF
    
    echo -e "   ✅ Node $i config created (port: $PORT, p2p: $P2P_PORT)"
done

# Start nodes
echo -e "\n${BLUE}🚀 Starting nodes...${NC}"
> "/tmp/polytorus_pids.txt"  # Clear PID file

for ((i=0; i<NUM_NODES; i++)); do
    NODE_ID="node-$i"
    PORT=$((BASE_PORT + i))
    P2P_PORT=$((BASE_P2P_PORT + i))
    DATA_DIR="./data/simulation/$NODE_ID"
    
    echo -e "   Starting ${NODE_ID} (HTTP: $PORT, P2P: $P2P_PORT)"
    
    # Start node in background
    POLYTORUS_CONFIG="$DATA_DIR/config.toml" \
    POLYTORUS_DATA_DIR="$DATA_DIR" \
    POLYTORUS_HTTP_PORT="$PORT" \
    cargo run --release -- \
        --config "$DATA_DIR/config.toml" \
        --data-dir "$DATA_DIR" \
        --http-port "$PORT" \
        --modular-start \
        > "./data/simulation/$NODE_ID.log" 2>&1 &
    
    NODE_PID=$!
    echo "$NODE_PID" >> "/tmp/polytorus_pids.txt"
    echo -e "   📡 Node $i started (PID: $NODE_PID)"
    
    # Small delay to avoid port conflicts
    sleep 2
done

# Wait for network to stabilize
echo -e "\n${YELLOW}⏳ Waiting for network to stabilize (10s)...${NC}"
sleep 10

# Check node status
echo -e "\n${BLUE}📊 Checking node status...${NC}"
for ((i=0; i<NUM_NODES; i++)); do
    PORT=$((BASE_PORT + i))
    
    # Try to get status (if HTTP API is available)
    if curl -s "http://127.0.0.1:$PORT/status" > /dev/null 2>&1; then
        echo -e "   ✅ Node $i (port $PORT) is responding"
    else
        echo -e "   ⚠️  Node $i (port $PORT) may still be starting up"
    fi
done

# Start transaction simulation
echo -e "\n${BLUE}💸 Starting transaction simulation...${NC}"
echo -e "   Running for ${SIMULATION_TIME} seconds"
echo -e "   Monitor logs: tail -f ./data/simulation/node-*.log"
echo -e "   Node APIs available at:"
for ((i=0; i<NUM_NODES; i++)); do
    PORT=$((BASE_PORT + i))
    echo -e "     Node $i: http://127.0.0.1:$PORT"
done

# Simple transaction generator
TRANSACTION_COUNT=0
START_TIME=$(date +%s)

while true; do
    CURRENT_TIME=$(date +%s)
    ELAPSED=$((CURRENT_TIME - START_TIME))
    
    if [[ $ELAPSED -ge $SIMULATION_TIME ]]; then
        break
    fi
    
    # Generate random transaction
    FROM_NODE=$((RANDOM % NUM_NODES))
    TO_NODE=$(((RANDOM % (NUM_NODES - 1) + FROM_NODE + 1) % NUM_NODES))
    AMOUNT=$((100 + RANDOM % 900))
    
    FROM_PORT=$((BASE_PORT + FROM_NODE))
    
    # Submit transaction (mock API call)
    TRANSACTION_DATA="{\"from\":\"wallet_node-$FROM_NODE\",\"to\":\"wallet_node-$TO_NODE\",\"amount\":$AMOUNT,\"nonce\":$TRANSACTION_COUNT}"
    
    if curl -s -X POST \
        -H "Content-Type: application/json" \
        -d "$TRANSACTION_DATA" \
        "http://127.0.0.1:$FROM_PORT/transaction" > /dev/null 2>&1; then
        echo -e "   💸 TX $TRANSACTION_COUNT: Node $FROM_NODE -> Node $TO_NODE (${AMOUNT})"
    else
        echo -e "   ❌ Failed to submit TX $TRANSACTION_COUNT"
    fi
    
    TRANSACTION_COUNT=$((TRANSACTION_COUNT + 1))
    
    # Progress report every 10 transactions
    if [[ $((TRANSACTION_COUNT % 10)) -eq 0 ]]; then
        echo -e "   📊 Progress: ${TRANSACTION_COUNT} transactions, ${ELAPSED}/${SIMULATION_TIME}s elapsed"
    fi
    
    sleep 5  # Transaction interval
done

echo -e "\n${GREEN}🎯 Simulation completed!${NC}"
echo -e "   Total transactions: ${TRANSACTION_COUNT}"
echo -e "   Duration: ${SIMULATION_TIME} seconds"

# Final statistics
echo -e "\n${BLUE}📈 Final Statistics:${NC}"
for ((i=0; i<NUM_NODES; i++)); do
    PORT=$((BASE_PORT + i))
    echo -e "   Node $i (port $PORT):"
    
    # Try to get final stats
    if curl -s "http://127.0.0.1:$PORT/stats" 2>/dev/null; then
        echo ""
    else
        echo -e "     Status: Running (no HTTP API stats available)"
    fi
done

# Show log files
echo -e "\n${BLUE}📋 Log files created:${NC}"
for ((i=0; i<NUM_NODES; i++)); do
    LOG_FILE="./data/simulation/node-$i.log"
    if [[ -f "$LOG_FILE" ]]; then
        LOG_SIZE=$(du -h "$LOG_FILE" | cut -f1)
        echo -e "   $LOG_FILE ($LOG_SIZE)"
    fi
done

echo -e "\n${YELLOW}💡 Tip: Check log files for detailed node activity${NC}"
echo -e "${YELLOW}💡 Data persisted in ./data/simulation/${NC}"

# Keep running until user interrupts
echo -e "\n${BLUE}🔄 Nodes still running. Press Ctrl+C to stop.${NC}"
while true; do
    sleep 1
done
