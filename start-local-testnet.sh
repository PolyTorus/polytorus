#!/bin/bash

# PolyTorus Local Testnet Startup Script
# This script helps users quickly set up and run a local testnet

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
TESTNET_NAME="polytorus-local-testnet"
TOPOLOGY_FILE="testnet-local.yml"
DOCKER_IMAGE="polytorus:testnet"

print_header() {
    echo -e "${BLUE}"
    echo "╔════════════════════════════════════════════════════════════╗"
    echo "║                PolyTorus Local Testnet                     ║"
    echo "║              Quick Setup & Management                      ║"
    echo "╚════════════════════════════════════════════════════════════╝"
    echo -e "${NC}"
}

print_usage() {
    echo -e "${CYAN}Usage: $0 [COMMAND]${NC}"
    echo ""
    echo -e "${YELLOW}Commands:${NC}"
    echo -e "  ${GREEN}start${NC}      - Start the local testnet"
    echo -e "  ${GREEN}stop${NC}       - Stop the local testnet"
    echo -e "  ${GREEN}restart${NC}    - Restart the local testnet"
    echo -e "  ${GREEN}status${NC}     - Show testnet status"
    echo -e "  ${GREEN}logs${NC}       - Show container logs"
    echo -e "  ${GREEN}clean${NC}      - Clean up all data and containers"
    echo -e "  ${GREEN}build${NC}      - Build Docker image"
    echo -e "  ${GREEN}wallet${NC}     - Create a new wallet"
    echo -e "  ${GREEN}send${NC}       - Send a test transaction"
    echo -e "  ${GREEN}api${NC}        - Test API endpoints"
    echo -e "  ${GREEN}cli${NC}        - Start interactive CLI"
    echo -e "  ${GREEN}help${NC}       - Show this help"
    echo ""
    echo -e "${YELLOW}Quick Start:${NC}"
    echo -e "  1. $0 build          # Build the Docker image"
    echo -e "  2. $0 start          # Start the testnet"
    echo -e "  3. $0 cli            # Use interactive CLI"
    echo ""
    echo -e "${YELLOW}Access Points:${NC}"
    echo -e "  API Gateway:   http://localhost:9020"
    echo -e "  Bootstrap:     http://localhost:9000"
    echo -e "  Miner 1:       http://localhost:9001"
    echo -e "  Miner 2:       http://localhost:9002"
    echo -e "  Validator:     http://localhost:9003"
}

check_dependencies() {
    local missing_deps=()
    
    if ! command -v containerlab &> /dev/null; then
        missing_deps+=("containerlab")
    fi
    
    if ! command -v docker &> /dev/null; then
        missing_deps+=("docker")
    fi
    
    if ! command -v python3 &> /dev/null; then
        missing_deps+=("python3")
    fi
    
    if [[ ${#missing_deps[@]} -gt 0 ]]; then
        echo -e "${RED}❌ Missing dependencies:${NC}"
        for dep in "${missing_deps[@]}"; do
            echo -e "   - $dep"
        done
        echo ""
        echo -e "${YELLOW}Please install the missing dependencies:${NC}"
        echo -e "  ContainerLab: bash -c \"\$(curl -sL https://get.containerlab.dev)\""
        echo -e "  Docker: https://docs.docker.com/get-docker/"
        exit 1
    fi
}

build_image() {
    echo -e "${BLUE}🔨 Building PolyTorus testnet Docker image...${NC}"
    
    if docker build -f Dockerfile.testnet -t "$DOCKER_IMAGE" .; then
        echo -e "${GREEN}✅ Docker image built successfully${NC}"
    else
        echo -e "${RED}❌ Docker build failed${NC}"
        exit 1
    fi
}

prepare_environment() {
    echo -e "${BLUE}📁 Preparing testnet environment...${NC}"
    
    # Create data directories
    mkdir -p testnet-data/{bootstrap,miner-1,miner-2,validator,api-gateway}
    
    # Create logs directories
    for node in bootstrap miner-1 miner-2 validator api-gateway; do
        mkdir -p "testnet-data/$node/logs"
    done
    
    # Ensure configuration file exists
    if [[ ! -f "config/testnet.toml" ]]; then
        echo -e "${YELLOW}⚠️  Configuration file not found, using default${NC}"
    fi
    
    echo -e "${GREEN}✅ Environment prepared${NC}"
}

start_testnet() {
    echo -e "${BLUE}🚀 Starting PolyTorus local testnet...${NC}"
    
    check_dependencies
    prepare_environment
    
    # Check if image exists
    if ! docker image inspect "$DOCKER_IMAGE" > /dev/null 2>&1; then
        echo -e "${YELLOW}⚠️  Docker image not found, building...${NC}"
        build_image
    fi
    
    # Deploy ContainerLab topology
    if containerlab deploy --topo "$TOPOLOGY_FILE"; then
        echo -e "${GREEN}✅ Testnet started successfully!${NC}"
        echo ""
        echo -e "${CYAN}🌐 Access your testnet:${NC}"
        echo -e "   API Gateway:   ${YELLOW}http://localhost:9020${NC}"
        echo -e "   Bootstrap:     ${YELLOW}http://localhost:9000${NC}"
        echo -e "   Miner 1:       ${YELLOW}http://localhost:9001${NC}"
        echo -e "   Miner 2:       ${YELLOW}http://localhost:9002${NC}"
        echo -e "   Validator:     ${YELLOW}http://localhost:9003${NC}"
        echo ""
        echo -e "${PURPLE}💡 Tip: Use '$0 status' to check node health${NC}"
        echo -e "${PURPLE}💡 Tip: Use '$0 cli' for interactive commands${NC}"
    else
        echo -e "${RED}❌ Failed to start testnet${NC}"
        exit 1
    fi
}

stop_testnet() {
    echo -e "${BLUE}🛑 Stopping PolyTorus local testnet...${NC}"
    
    if containerlab destroy --topo "$TOPOLOGY_FILE"; then
        echo -e "${GREEN}✅ Testnet stopped successfully${NC}"
    else
        echo -e "${YELLOW}⚠️  Some containers may still be running${NC}"
        
        # Force stop containers
        echo -e "${BLUE}🔧 Force stopping containers...${NC}"
        docker ps --filter "label=containerlab" --filter "name=clab-$TESTNET_NAME" -q | xargs -r docker stop
        docker ps -a --filter "label=containerlab" --filter "name=clab-$TESTNET_NAME" -q | xargs -r docker rm
        
        echo -e "${GREEN}✅ Containers force stopped${NC}"
    fi
}

restart_testnet() {
    echo -e "${BLUE}🔄 Restarting PolyTorus local testnet...${NC}"
    stop_testnet
    sleep 5
    start_testnet
}

show_status() {
    echo -e "${BLUE}📊 PolyTorus Local Testnet Status${NC}"
    echo -e "=================================="
    
    # Check ContainerLab topology
    if containerlab inspect --topo "$TOPOLOGY_FILE" > /dev/null 2>&1; then
        echo -e "${GREEN}✅ ContainerLab topology is running${NC}"
        
        echo -e "\n${CYAN}📡 Node Status:${NC}"
        
        # Check individual nodes
        local nodes=(
            "bootstrap:9000"
            "miner-1:9001"  
            "miner-2:9002"
            "validator:9003"
            "api-gateway:9020"
        )
        
        for node_info in "${nodes[@]}"; do
            IFS=':' read -r name port <<< "$node_info"
            
            if curl -s --connect-timeout 3 "http://localhost:$port/health" > /dev/null 2>&1 || \
               curl -s --connect-timeout 3 "http://localhost:$port/" > /dev/null 2>&1; then
                echo -e "   ✅ $name (port $port): Online"
            else
                echo -e "   ❌ $name (port $port): Offline"
            fi
        done
        
        # Show container status
        echo -e "\n${CYAN}🐳 Container Status:${NC}"
        docker ps --filter "label=containerlab" --filter "name=clab-$TESTNET_NAME" \
            --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}" | grep -v "NAMES" | \
            while read -r line; do
                echo -e "   📦 $line"
            done
        
    else
        echo -e "${RED}❌ Testnet is not running${NC}"
        echo -e "${YELLOW}💡 Start it with: $0 start${NC}"
    fi
}

show_logs() {
    echo -e "${BLUE}📋 Container Logs${NC}"
    echo -e "=================="
    
    local containers=$(docker ps --filter "label=containerlab" --filter "name=clab-$TESTNET_NAME" --format "{{.Names}}")
    
    if [[ -z "$containers" ]]; then
        echo -e "${YELLOW}⚠️  No running containers found${NC}"
        return
    fi
    
    echo -e "${CYAN}Available containers:${NC}"
    echo "$containers" | nl -v1 -w2 -s'. '
    
    echo -e "\n${YELLOW}Enter container number to view logs (or 'all' for all):${NC}"
    read -r choice
    
    if [[ "$choice" == "all" ]]; then
        echo "$containers" | while read -r container; do
            echo -e "\n${CYAN}--- Logs for $container ---${NC}"
            docker logs --tail 20 "$container"
        done
    elif [[ "$choice" =~ ^[0-9]+$ ]]; then
        local container=$(echo "$containers" | sed -n "${choice}p")
        if [[ -n "$container" ]]; then
            echo -e "\n${CYAN}--- Logs for $container ---${NC}"
            docker logs --follow "$container"
        else
            echo -e "${RED}❌ Invalid selection${NC}"
        fi
    else
        echo -e "${RED}❌ Invalid input${NC}"
    fi
}

clean_testnet() {
    echo -e "${BLUE}🧹 Cleaning up testnet data...${NC}"
    
    # Stop testnet first
    stop_testnet
    
    # Remove data directories
    if [[ -d "testnet-data" ]]; then
        echo -e "${YELLOW}⚠️  This will delete all testnet data. Continue? (y/N)${NC}"
        read -r confirm
        if [[ "$confirm" =~ ^[Yy]$ ]]; then
            rm -rf testnet-data
            echo -e "${GREEN}✅ Testnet data cleaned${NC}"
        else
            echo -e "${YELLOW}❌ Cleanup cancelled${NC}"
        fi
    fi
    
    # Remove Docker image
    echo -e "${YELLOW}Remove Docker image as well? (y/N)${NC}"
    read -r confirm
    if [[ "$confirm" =~ ^[Yy]$ ]]; then
        docker rmi "$DOCKER_IMAGE" 2>/dev/null || true
        echo -e "${GREEN}✅ Docker image removed${NC}"
    fi
}

create_wallet() {
    echo -e "${BLUE}👛 Creating new wallet...${NC}"
    
    if python3 scripts/testnet_manager.py --create-wallet; then
        echo -e "${GREEN}✅ Wallet created successfully${NC}"
    else
        echo -e "${RED}❌ Failed to create wallet${NC}"
        echo -e "${YELLOW}💡 Make sure the testnet is running: $0 start${NC}"
    fi
}

send_test_transaction() {
    echo -e "${BLUE}💸 Sending test transaction...${NC}"
    
    if python3 scripts/testnet_manager.py --test-transactions 1; then
        echo -e "${GREEN}✅ Test transaction sent${NC}"
    else
        echo -e "${RED}❌ Failed to send transaction${NC}"
        echo -e "${YELLOW}💡 Make sure you have wallets with balance${NC}"
    fi
}

test_api_endpoints() {
    echo -e "${BLUE}🔧 Testing API endpoints...${NC}"
    
    local api_url="http://localhost:9020"
    
    # Check if API gateway is running
    if curl -s --connect-timeout 3 "$api_url/health" > /dev/null 2>&1; then
        echo -e "${GREEN}✅ API Gateway is running${NC}"
        echo -e "${CYAN}🔗 Base URL: $api_url${NC}"
        echo ""
        
        echo -e "${YELLOW}Testing endpoints:${NC}"
        
        # Test network status
        echo -e "  📊 Network status:"
        curl -s "$api_url/network/status" | head -c 100
        echo "..."
        
        # Test wallet list
        echo -e "\n  👛 Wallet list:"
        curl -s "$api_url/wallet/list" | head -c 100
        echo "..."
        
        echo -e "\n\n${CYAN}Available endpoints:${NC}"
        echo -e "  GET  $api_url/network/status"
        echo -e "  GET  $api_url/wallet/list"
        echo -e "  POST $api_url/wallet/create"
        echo -e "  GET  $api_url/balance/<address>"
        echo -e "  POST $api_url/transaction/send"
        
    else
        echo -e "${RED}❌ API Gateway is not running${NC}"
        echo -e "${YELLOW}💡 Start the testnet first: $0 start${NC}"
    fi
}

start_cli() {
    echo -e "${BLUE}🎮 Starting interactive CLI...${NC}"
    
    if [[ -f "scripts/testnet_manager.py" ]]; then
        python3 scripts/testnet_manager.py --interactive
    else
        echo -e "${RED}❌ CLI script not found${NC}"
    fi
}

# Main command handling
case "${1:-help}" in
    start)
        start_testnet
        ;;
    stop)
        stop_testnet
        ;;
    restart)
        restart_testnet
        ;;
    status)
        show_status
        ;;
    logs)
        show_logs
        ;;
    clean)
        clean_testnet
        ;;
    build)
        build_image
        ;;
    wallet)
        create_wallet
        ;;
    send)
        send_test_transaction
        ;;
    api)
        test_api_endpoints
        ;;
    cli)
        start_cli
        ;;
    help|--help|-h)
        print_header
        print_usage
        ;;
    *)
        echo -e "${RED}Unknown command: $1${NC}"
        echo ""
        print_usage
        exit 1
        ;;
esac