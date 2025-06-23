#!/bin/bash

# ContainerLab Testnet Simulation with Mining
# This script sets up a complete testnet environment using ContainerLab

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m'

# Configuration
TOPOLOGY_FILE="containerlab-topology.yml"
SIMULATION_DURATION=${1:-600}  # 10 minutes default
NUM_TRANSACTIONS=${2:-50}      # Number of transactions to generate
TX_INTERVAL=${3:-10}           # Transaction interval in seconds

print_header() {
    echo -e "${BLUE}"
    echo "╔══════════════════════════════════════════════════════════════╗"
    echo "║           PolyTorus ContainerLab Testnet Simulation          ║"
    echo "║                    With Mining & Transactions                ║"
    echo "╚══════════════════════════════════════════════════════════════╝"
    echo -e "${NC}"
}

print_config() {
    echo -e "${CYAN}📊 Simulation Configuration:${NC}"
    echo -e "   Duration: ${SIMULATION_DURATION}s ($(($SIMULATION_DURATION / 60)) minutes)"
    echo -e "   Transactions: ${NUM_TRANSACTIONS}"
    echo -e "   TX Interval: ${TX_INTERVAL}s"
    echo -e "   Topology: 4 nodes (1 bootstrap + 2 miners + 1 validator)"
    echo ""
}

check_dependencies() {
    local missing_deps=()
    
    # Check for required tools
    if ! command -v containerlab &> /dev/null; then
        missing_deps+=("containerlab")
    fi
    
    if ! command -v docker &> /dev/null; then
        missing_deps+=("docker")
    fi
    
    if ! command -v cargo &> /dev/null; then
        missing_deps+=("cargo (Rust)")
    fi
    
    if ! command -v curl &> /dev/null; then
        missing_deps+=("curl")
    fi
    
    if [[ ${#missing_deps[@]} -gt 0 ]]; then
        echo -e "${RED}❌ Missing dependencies:${NC}"
        for dep in "${missing_deps[@]}"; do
            echo -e "   - $dep"
        done
        echo ""
        echo -e "${YELLOW}Please install the missing dependencies and try again.${NC}"
        echo -e "${YELLOW}To install ContainerLab: bash -c \"\$(curl -sL https://get.containerlab.dev)\"${NC}"
        exit 1
    fi
}

build_docker_image() {
    echo -e "${BLUE}🔨 Building PolyTorus Docker image...${NC}"
    
    if docker build -t polytorus:latest .; then
        echo -e "${GREEN}✅ Docker image built successfully${NC}"
    else
        echo -e "${RED}❌ Docker build failed${NC}"
        exit 1
    fi
}

prepare_environment() {
    echo -e "${BLUE}📁 Preparing simulation environment...${NC}"
    
    # Create data directories for ContainerLab
    mkdir -p "./data/containerlab"
    for i in {0..3}; do
        mkdir -p "./data/containerlab/node-$i"
        mkdir -p "./data/containerlab/node-$i/wallets"
        mkdir -p "./data/containerlab/node-$i/blockchain"
        mkdir -p "./data/containerlab/node-$i/contracts"
        mkdir -p "./data/containerlab/node-$i/modular_storage"
    done
    
    echo -e "${GREEN}✅ Environment prepared${NC}"
}

generate_mining_wallets() {
    echo -e "${BLUE}🔑 Generating mining wallets...${NC}"
    
    # Create wallets for miners
    for i in 1 2; do
        echo -e "   Creating wallet for miner node-$i..."
        
        # Set data directory for this node
        export POLYTORUS_DATA_DIR="./data/containerlab/node-$i"
        
        # Create wallet using Rust binary
        if cargo run --release -- --data-dir "$POLYTORUS_DATA_DIR" --createwallet; then
            echo -e "   ✅ Wallet created for node-$i"
            
            # Get the wallet address
            WALLET_ADDRESS=$(cargo run --release -- --data-dir "$POLYTORUS_DATA_DIR" --listaddresses | tail -n 1 | grep -oE '[A-Za-z0-9]{25,}' | head -n 1)
            
            if [[ -n "$WALLET_ADDRESS" ]]; then
                echo -e "   📝 Mining address for node-$i: $WALLET_ADDRESS"
                echo "$WALLET_ADDRESS" > "./data/containerlab/node-$i/mining_address.txt"
            else
                echo -e "   ⚠️  Could not extract wallet address for node-$i"
                echo "miner${i}_default_address" > "./data/containerlab/node-$i/mining_address.txt"
            fi
        else
            echo -e "   ⚠️  Failed to create wallet for node-$i, using default address"
            echo "miner${i}_default_address" > "./data/containerlab/node-$i/mining_address.txt"
        fi
    done
    
    # Create topology file with actual mining addresses
    update_topology_with_addresses
}

update_topology_with_addresses() {
    echo -e "${BLUE}⚙️  Updating topology with mining addresses...${NC}"
    
    # Read mining addresses
    MINER1_ADDRESS=$(cat "./data/containerlab/node-1/mining_address.txt" 2>/dev/null || echo "miner1_default")
    MINER2_ADDRESS=$(cat "./data/containerlab/node-2/mining_address.txt" 2>/dev/null || echo "miner2_default")
    
    # Update the topology file with real addresses
    sed -i "s/miner1_address_here/$MINER1_ADDRESS/g" "$TOPOLOGY_FILE"
    sed -i "s/miner2_address_here/$MINER2_ADDRESS/g" "$TOPOLOGY_FILE"
    
    echo -e "   ✅ Topology updated with mining addresses"
    echo -e "   📝 Miner 1: $MINER1_ADDRESS"
    echo -e "   📝 Miner 2: $MINER2_ADDRESS"
}

start_containerlab() {
    echo -e "${BLUE}🚀 Starting ContainerLab topology...${NC}"
    
    if containerlab deploy --topo "$TOPOLOGY_FILE"; then
        echo -e "${GREEN}✅ ContainerLab topology deployed successfully${NC}"
    else
        echo -e "${RED}❌ Failed to deploy ContainerLab topology${NC}"
        exit 1
    fi
}

wait_for_nodes() {
    echo -e "${BLUE}⏳ Waiting for nodes to start...${NC}"
    sleep 30
    
    echo -e "${BLUE}📊 Checking node status...${NC}"
    for i in {0..3}; do
        PORT=$((9000 + i))
        if curl -s --connect-timeout 5 "http://localhost:$PORT/status" > /dev/null 2>&1; then
            echo -e "   ✅ Node $i (port $PORT) is responding"
        else
            echo -e "   ⚠️  Node $i (port $PORT) may still be starting up"
        fi
    done
}

start_mining_simulation() {
    echo -e "${BLUE}⛏️  Starting mining simulation...${NC}"
    
    # Start background mining monitoring
    monitor_mining &
    MINING_MONITOR_PID=$!
    
    # Start transaction generation
    generate_transactions &
    TX_GENERATOR_PID=$!
    
    echo -e "${GREEN}✅ Mining simulation started${NC}"
    echo -e "   Mining monitor PID: $MINING_MONITOR_PID"
    echo -e "   Transaction generator PID: $TX_GENERATOR_PID"
    
    # Store PIDs for cleanup
    echo "$MINING_MONITOR_PID" > /tmp/mining_monitor.pid
    echo "$TX_GENERATOR_PID" > /tmp/tx_generator.pid
}

monitor_mining() {
    echo -e "${YELLOW}🔍 Starting mining monitor...${NC}"
    
    while true; do
        sleep 30
        
        echo -e "\n${CYAN}⛏️  Mining Status Report:${NC}"
        
        for i in {0..3}; do
            PORT=$((9000 + i))
            NODE_TYPE="validator"
            [[ $i -eq 1 || $i -eq 2 ]] && NODE_TYPE="miner"
            
            if RESPONSE=$(curl -s --connect-timeout 3 "http://localhost:$PORT/status" 2>/dev/null); then
                # Parse response for block height and other metrics
                BLOCK_HEIGHT=$(echo "$RESPONSE" | grep -o '"block_height":[0-9]*' | cut -d':' -f2 | head -n1)
                echo -e "   📡 Node $i ($NODE_TYPE): Block height ${BLOCK_HEIGHT:-'unknown'}"
            else
                echo -e "   ❌ Node $i ($NODE_TYPE): Not responding"
            fi
        done
        
        # Get mining statistics from miners
        for i in 1 2; do
            PORT=$((9000 + i))
            if STATS=$(curl -s --connect-timeout 3 "http://localhost:$PORT/stats" 2>/dev/null); then
                echo -e "   ⛏️  Miner $i stats: $STATS"
            fi
        done
    done
}

generate_transactions() {
    echo -e "${YELLOW}💸 Starting transaction generator...${NC}"
    
    local tx_count=0
    local start_time=$(date +%s)
    
    while [[ $tx_count -lt $NUM_TRANSACTIONS ]]; do
        local current_time=$(date +%s)
        local elapsed=$((current_time - start_time))
        
        if [[ $elapsed -ge $SIMULATION_DURATION ]]; then
            break
        fi
        
        # Generate random transaction
        local from_node=$((RANDOM % 4))
        local to_node=$(((RANDOM % 3 + from_node + 1) % 4))
        local amount=$((100 + RANDOM % 900))
        
        local from_port=$((9000 + from_node))
        local to_port=$((9000 + to_node))
        
        # Create transaction payload
        local tx_data="{\"from\":\"node-${from_node}\",\"to\":\"node-${to_node}\",\"amount\":$amount,\"nonce\":$tx_count}"
        
        # Submit transaction to sender node
        if curl -s -X POST \
            -H "Content-Type: application/json" \
            -d "$tx_data" \
            "http://localhost:$from_port/send" > /dev/null 2>&1; then
            echo -e "   💸 TX $tx_count: Node $from_node -> Node $to_node (${amount} satoshis)"
        else
            echo -e "   ❌ Failed to submit TX $tx_count"
        fi
        
        # Also propagate to receiver node
        curl -s -X POST \
            -H "Content-Type: application/json" \
            -d "$tx_data" \
            "http://localhost:$to_port/transaction" > /dev/null 2>&1
        
        tx_count=$((tx_count + 1))
        
        # Progress report
        if [[ $((tx_count % 10)) -eq 0 ]]; then
            echo -e "   📊 Progress: ${tx_count}/${NUM_TRANSACTIONS} transactions, ${elapsed}/${SIMULATION_DURATION}s elapsed"
        fi
        
        sleep $TX_INTERVAL
    done
    
    echo -e "${GREEN}✅ Transaction generation completed: $tx_count transactions sent${NC}"
}

show_final_statistics() {
    echo -e "\n${BLUE}📈 Final Network Statistics:${NC}"
    
    for i in {0..3}; do
        PORT=$((9000 + i))
        NODE_TYPE="validator"
        [[ $i -eq 1 || $i -eq 2 ]] && NODE_TYPE="miner"
        
        echo -e "\n   📡 Node $i ($NODE_TYPE):"
        
        if RESPONSE=$(curl -s --connect-timeout 5 "http://localhost:$PORT/status" 2>/dev/null); then
            echo -e "     Status: $RESPONSE"
        else
            echo -e "     Status: Not responding"
        fi
        
        if [[ $i -eq 1 || $i -eq 2 ]]; then
            if STATS=$(curl -s --connect-timeout 5 "http://localhost:$PORT/stats" 2>/dev/null); then
                echo -e "     Mining stats: $STATS"
            fi
        fi
    done
    
    echo -e "\n${BLUE}📋 ContainerLab Container Status:${NC}"
    containerlab inspect --topo "$TOPOLOGY_FILE" || true
}

cleanup() {
    echo -e "\n${YELLOW}🧹 Cleaning up simulation...${NC}"
    
    # Stop background processes
    if [[ -f "/tmp/mining_monitor.pid" ]]; then
        MINING_PID=$(cat /tmp/mining_monitor.pid)
        if kill -0 "$MINING_PID" 2>/dev/null; then
            kill "$MINING_PID" 2>/dev/null || true
        fi
        rm -f /tmp/mining_monitor.pid
    fi
    
    if [[ -f "/tmp/tx_generator.pid" ]]; then
        TX_PID=$(cat /tmp/tx_generator.pid)
        if kill -0 "$TX_PID" 2>/dev/null; then
            kill "$TX_PID" 2>/dev/null || true
        fi
        rm -f /tmp/tx_generator.pid
    fi
    
    # Destroy ContainerLab topology
    echo -e "${BLUE}🗑️  Destroying ContainerLab topology...${NC}"
    containerlab destroy --topo "$TOPOLOGY_FILE" || true
    
    echo -e "${GREEN}✅ Cleanup completed${NC}"
}

# Set up signal handlers
trap cleanup SIGINT SIGTERM EXIT

# Main execution
main() {
    print_header
    print_config
    
    check_dependencies
    build_docker_image
    prepare_environment
    generate_mining_wallets
    start_containerlab
    wait_for_nodes
    start_mining_simulation
    
    echo -e "\n${GREEN}🎯 Testnet simulation running!${NC}"
    echo -e "${YELLOW}💡 Monitor nodes at:${NC}"
    for i in {0..3}; do
        echo -e "   Node $i: http://localhost:$((9000 + i))"
    done
    
    echo -e "\n${CYAN}Press Ctrl+C to stop the simulation...${NC}"
    
    # Wait for simulation duration
    sleep $SIMULATION_DURATION
    
    echo -e "\n${GREEN}🏁 Simulation completed!${NC}"
    show_final_statistics
}

# Check if running as source or executed
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi