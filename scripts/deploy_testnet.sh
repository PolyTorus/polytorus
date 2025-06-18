#!/bin/bash

# PolyTorus Private Testnet Deployment Script
# このスクリプトは即座にプライベートテストネットを展開します

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m'

# Configuration
NUM_NODES=${1:-4}
BASE_HTTP_PORT=${2:-9000}
BASE_P2P_PORT=${3:-8000}
NETWORK_NAME=${4:-"polytorus-testnet"}

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

print_header() {
    echo -e "${BLUE}"
    echo "╔══════════════════════════════════════════════════════════╗"
    echo "║               PolyTorus Testnet Deployer                ║"
    echo "║              Private Network Deployment                 ║"
    echo "╚══════════════════════════════════════════════════════════╝"
    echo -e "${NC}"
}

print_status() {
    echo -e "${GREEN}[INFO]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

cleanup() {
    echo -e "\n${YELLOW}🧹 Cleaning up...${NC}"
    
    # Kill background processes
    if [[ -f "/tmp/polytorus_testnet_pids.txt" ]]; then
        while read -r pid; do
            if kill -0 "$pid" 2>/dev/null; then
                print_status "Stopping node process ${pid}"
                kill "$pid" 2>/dev/null || true
            fi
        done < "/tmp/polytorus_testnet_pids.txt"
        rm -f "/tmp/polytorus_testnet_pids.txt"
    fi
    
    echo -e "${GREEN}✅ Cleanup completed${NC}"
}

# Setup cleanup on script exit
trap cleanup EXIT INT TERM

check_dependencies() {
    print_status "Checking dependencies..."
    
    # Check Rust
    if ! command -v cargo &> /dev/null; then
        print_error "Rust/Cargo not found. Please install Rust."
        exit 1
    fi
    
    # Check OpenFHE
    if [[ ! -d "/usr/local/include/openfhe" ]]; then
        print_warning "OpenFHE not found. Some features may not work."
        read -p "Continue anyway? (y/N): " -n 1 -r
        echo
        if [[ ! $REPLY =~ ^[Yy]$ ]]; then
            exit 1
        fi
    fi
    
    print_status "✅ Dependencies check passed"
}

build_project() {
    print_status "Building PolyTorus..."
    
    cd "$PROJECT_DIR"
    
    # Clean build
    cargo clean > /dev/null 2>&1
    
    # Build release
    if ! cargo build --release; then
        print_error "Failed to build PolyTorus"
        exit 1
    fi
    
    print_status "✅ Build completed"
}

create_network_config() {
    print_status "Creating network configuration..."
    
    # Create config directory
    mkdir -p "$PROJECT_DIR/config/testnet"
    
    # Generate bootstrap peers list
    local bootstrap_peers=""
    for ((i=1; i<=NUM_NODES; i++)); do
        local p2p_port=$((BASE_P2P_PORT + i - 1))
        if [[ $i -gt 1 ]]; then
            bootstrap_peers+=", "
        fi
        bootstrap_peers+="\"127.0.0.1:$p2p_port\""
    done
    
    # Create node configurations
    for ((i=1; i<=NUM_NODES; i++)); do
        local p2p_port=$((BASE_P2P_PORT + i - 1))
        local http_port=$((BASE_HTTP_PORT + i - 1))
        
        cat > "$PROJECT_DIR/config/testnet/node$i.toml" << EOF
# PolyTorus Node $i Configuration
[network]
listen_addr = "0.0.0.0:$p2p_port"
bootstrap_peers = [$bootstrap_peers]
max_peers = 50

[consensus]
block_time = 10000          # 10 seconds
difficulty = 4              # Low difficulty for testing
max_block_size = 1048576    # 1MB

[execution]
gas_limit = 8000000
gas_price = 1

[settlement]
challenge_period = 100      # 100 blocks
batch_size = 100           # 100 transactions per batch
min_validator_stake = 1000

[data_availability]
retention_period = 604800   # 7 days
max_data_size = 1048576    # 1MB

[diamond_io]
security_mode = "testing"   # testing mode for testnet

[logging]
level = "info"
EOF
        
        print_status "Created config for node $i (HTTP: $http_port, P2P: $p2p_port)"
    done
    
    print_status "✅ Network configuration created"
}

start_nodes() {
    print_status "Starting $NUM_NODES nodes..."
    
    # Clear PID file
    > "/tmp/polytorus_testnet_pids.txt"
    
    # Start nodes
    for ((i=1; i<=NUM_NODES; i++)); do
        local http_port=$((BASE_HTTP_PORT + i - 1))
        local data_dir="$PROJECT_DIR/data/testnet/node$i"
        local config_file="$PROJECT_DIR/config/testnet/node$i.toml"
        
        # Create data directory
        mkdir -p "$data_dir"
        
        print_status "Starting node $i..."
        
        # Start node in background
        "$PROJECT_DIR/target/release/polytorus" \
            --config "$config_file" \
            --data-dir "$data_dir" \
            --http-port "$http_port" \
            --modular-start > "$data_dir/node.log" 2>&1 &
        
        local pid=$!
        echo "$pid" >> "/tmp/polytorus_testnet_pids.txt"
        
        print_status "Node $i started (PID: $pid, HTTP: $http_port)"
        
        # Wait a bit between node starts
        sleep 2
    done
    
    print_status "✅ All nodes started"
}

wait_for_nodes() {
    print_status "Waiting for nodes to initialize..."
    
    local ready_nodes=0
    local max_attempts=30
    local attempt=0
    
    while [[ $ready_nodes -lt $NUM_NODES && $attempt -lt $max_attempts ]]; do
        ready_nodes=0
        
        for ((i=1; i<=NUM_NODES; i++)); do
            local http_port=$((BASE_HTTP_PORT + i - 1))
            
            if curl -s "http://localhost:$http_port/api/health" > /dev/null 2>&1; then
                ((ready_nodes++))
            fi
        done
        
        print_status "Ready nodes: $ready_nodes/$NUM_NODES (attempt $((attempt+1))/$max_attempts)"
        
        if [[ $ready_nodes -lt $NUM_NODES ]]; then
            sleep 2
            ((attempt++))
        fi
    done
    
    if [[ $ready_nodes -eq $NUM_NODES ]]; then
        print_status "✅ All nodes are ready"
        return 0
    else
        print_error "Timeout: Only $ready_nodes/$NUM_NODES nodes are ready"
        return 1
    fi
}

create_wallets() {
    print_status "Creating wallets for each node..."
    
    for ((i=1; i<=NUM_NODES; i++)); do
        local data_dir="$PROJECT_DIR/data/testnet/node$i"
        
        print_status "Creating wallet for node $i..."
        
        "$PROJECT_DIR/target/release/polytorus" \
            --createwallet \
            --data-dir "$data_dir" > /dev/null 2>&1
        
        # Get address
        local address=$("$PROJECT_DIR/target/release/polytorus" \
            --listaddresses \
            --data-dir "$data_dir" 2>/dev/null | head -n1)
        
        print_status "Node $i wallet address: $address"
    done
    
    print_status "✅ Wallets created"
}

test_network() {
    print_status "Testing network functionality..."
    
    # Test API endpoints
    local test_port=$BASE_HTTP_PORT
    
    print_status "Testing health endpoint..."
    if curl -s "http://localhost:$test_port/api/health" | grep -q "healthy"; then
        print_status "✅ Health check passed"
    else
        print_warning "❌ Health check failed"
    fi
    
    print_status "Testing network status..."
    if curl -s "http://localhost:$test_port/api/network/status" > /dev/null; then
        print_status "✅ Network status accessible"
    else
        print_warning "❌ Network status failed"
    fi
    
    print_status "Testing modular status..."
    if "$PROJECT_DIR/target/release/polytorus" \
        --modular-status \
        --data-dir "$PROJECT_DIR/data/testnet/node1" > /dev/null 2>&1; then
        print_status "✅ Modular status check passed"
    else
        print_warning "❌ Modular status check failed"
    fi
}

print_network_info() {
    echo -e "\n${CYAN}╔══════════════════════════════════════════════════════════╗${NC}"
    echo -e "${CYAN}║                    TESTNET DEPLOYED                     ║${NC}"
    echo -e "${CYAN}╚══════════════════════════════════════════════════════════╝${NC}"
    echo -e "\n${GREEN}🎉 PolyTorus Private Testnet is now running!${NC}\n"
    
    echo -e "${YELLOW}Network Information:${NC}"
    echo -e "  Name: $NETWORK_NAME"
    echo -e "  Nodes: $NUM_NODES"
    echo -e "  Architecture: Modular (Consensus + Settlement + Execution + DA)"
    echo -e "  Privacy: Diamond IO enabled (testing mode)"
    echo -e ""
    
    echo -e "${YELLOW}Node Endpoints:${NC}"
    for ((i=1; i<=NUM_NODES; i++)); do
        local http_port=$((BASE_HTTP_PORT + i - 1))
        local p2p_port=$((BASE_P2P_PORT + i - 1))
        echo -e "  Node $i: HTTP http://localhost:$http_port | P2P :$p2p_port"
    done
    echo -e ""
    
    echo -e "${YELLOW}API Examples:${NC}"
    echo -e "  Health Check:    curl http://localhost:$BASE_HTTP_PORT/api/health"
    echo -e "  Network Status:  curl http://localhost:$BASE_HTTP_PORT/api/network/status"
    echo -e "  Statistics:      curl http://localhost:$BASE_HTTP_PORT/api/stats"
    echo -e "  Peers:           curl http://localhost:$BASE_HTTP_PORT/api/network/peers"
    echo -e ""
    
    echo -e "${YELLOW}CLI Commands:${NC}"
    echo -e "  Node Status:     ./target/release/polytorus --modular-status --data-dir data/testnet/node1"
    echo -e "  List Addresses:  ./target/release/polytorus --listaddresses --data-dir data/testnet/node1"
    echo -e "  Deploy ERC20:    ./target/release/polytorus --smart-contract-deploy erc20 --data-dir data/testnet/node1"
    echo -e ""
    
    echo -e "${YELLOW}Monitoring:${NC}"
    echo -e "  Logs:           tail -f data/testnet/node1/node.log"
    echo -e "  Live Stats:     cargo run --example transaction_monitor"
    echo -e ""
    
    echo -e "${YELLOW}Testing:${NC}"
    echo -e "  ERC20 Demo:     cargo run --example erc20_demo"
    echo -e "  Diamond IO:     cargo run --example diamond_io_demo"
    echo -e "  Multi-node:     cargo run --example multi_node_simulation"
    echo -e ""
    
    echo -e "${RED}To stop the testnet:${NC}"
    echo -e "  Press Ctrl+C or run: pkill -f polytorus"
    echo -e ""
    
    echo -e "${GREEN}🚀 Ready for testing! Documentation: docs/TESTNET_DEPLOYMENT.md${NC}"
}

# Main deployment flow
main() {
    print_header
    
    echo -e "${CYAN}Deployment Configuration:${NC}"
    echo -e "  Nodes: $NUM_NODES"
    echo -e "  HTTP Ports: $BASE_HTTP_PORT-$((BASE_HTTP_PORT + NUM_NODES - 1))"
    echo -e "  P2P Ports: $BASE_P2P_PORT-$((BASE_P2P_PORT + NUM_NODES - 1))"
    echo -e "  Network: $NETWORK_NAME"
    echo -e ""
    
    read -p "Continue with deployment? (Y/n): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Nn]$ ]]; then
        echo "Deployment cancelled."
        exit 0
    fi
    
    # Execute deployment steps
    check_dependencies
    build_project
    create_network_config
    start_nodes
    
    if wait_for_nodes; then
        create_wallets
        test_network
        print_network_info
        
        # Keep running until interrupted
        echo -e "${BLUE}Press Ctrl+C to stop the testnet...${NC}"
        while true; do
            sleep 10
            # Simple health check
            if ! curl -s "http://localhost:$BASE_HTTP_PORT/api/health" > /dev/null 2>&1; then
                print_warning "Primary node appears to be down"
            fi
        done
    else
        print_error "Failed to start all nodes properly"
        exit 1
    fi
}

# Run if called directly
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi