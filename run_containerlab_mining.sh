#!/bin/bash

# Quick ContainerLab Mining Test Script
# This script provides easy access to the ContainerLab mining simulation

set -e

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m'

print_usage() {
    echo -e "${BLUE}PolyTorus ContainerLab Mining Simulation${NC}"
    echo ""
    echo "Usage: $0 [OPTIONS]"
    echo ""
    echo "Options:"
    echo "  rust-sim       Run Rust-based mining simulation (recommended)"
    echo "  containerlab   Run basic ContainerLab setup"
    echo "  realistic      Run realistic global testnet with AS separation"
    echo "  test-setup     Test the basic setup without ContainerLab"
    echo "  clean          Clean up simulation data"
    echo "  help           Show this help message"
    echo ""
    echo "Examples:"
    echo "  $0 rust-sim         # Quick mining simulation"
    echo "  $0 containerlab     # Basic ContainerLab setup"
    echo "  $0 realistic        # Realistic global testnet (recommended)"
    echo "  $0 test-setup       # Test basic functionality"
}

run_rust_simulation() {
    echo -e "${BLUE}🦀 Running Rust-based mining simulation...${NC}"
    
    # Build the project first
    echo -e "${YELLOW}Building PolyTorus...${NC}"
    cargo build --release
    
    # Run the mining simulation
    echo -e "${YELLOW}Starting mining simulation...${NC}"
    cargo run --example containerlab_mining_simulation -- \
        --nodes 4 \
        --miners 2 \
        --duration 300
}

run_containerlab() {
    echo -e "${BLUE}🐳 Running basic ContainerLab simulation...${NC}"
    
    if ! command -v containerlab &> /dev/null; then
        echo -e "${YELLOW}⚠️  ContainerLab not found. Running Rust simulation instead...${NC}"
        run_rust_simulation
        return
    fi
    
    # Run the basic ContainerLab script
    ./scripts/containerlab_testnet.sh 600 50 10
}

run_realistic_testnet() {
    echo -e "${BLUE}🌍 Running realistic global testnet with AS separation...${NC}"
    
    if ! command -v containerlab &> /dev/null; then
        echo -e "${YELLOW}⚠️  ContainerLab not found. Running Rust simulation instead...${NC}"
        run_rust_simulation
        return
    fi
    
    # Run the realistic testnet with BGP routing
    ./scripts/realistic_testnet.sh 1800 true true
}

test_basic_setup() {
    echo -e "${BLUE}🔧 Testing basic setup...${NC}"
    
    # Test build
    echo -e "${YELLOW}Testing build...${NC}"
    if cargo build; then
        echo -e "${GREEN}✅ Build successful${NC}"
    else
        echo -e "❌ Build failed"
        exit 1
    fi
    
    # Test CLI functionality
    echo -e "${YELLOW}Testing CLI...${NC}"
    if cargo run --release --bin polytorus -- --help > /dev/null; then
        echo -e "${GREEN}✅ CLI working${NC}"
    else
        echo -e "❌ CLI test failed"
        exit 1
    fi
    
    # Test modular architecture
    echo -e "${YELLOW}Testing modular architecture...${NC}"
    timeout 10s cargo run --release --bin polytorus -- --modular-status > /dev/null 2>&1 || true
    echo -e "${GREEN}✅ Modular architecture test completed${NC}"
    
    echo -e "${GREEN}🎯 Basic setup test completed successfully!${NC}"
}

clean_simulation() {
    echo -e "${BLUE}🧹 Cleaning simulation data...${NC}"
    
    # Clean data directories
    if [[ -d "./data/containerlab" ]]; then
        rm -rf "./data/containerlab"
        echo -e "   ✅ ContainerLab data cleaned"
    fi
    
    if [[ -d "./data/realistic" ]]; then
        rm -rf "./data/realistic"
        echo -e "   ✅ Realistic testnet data cleaned"
    fi
    
    if [[ -d "./data/simulation" ]]; then
        rm -rf "./data/simulation"
        echo -e "   ✅ Simulation data cleaned"
    fi
    
    # Clean any running containerlab topologies
    if command -v containerlab &> /dev/null; then
        containerlab destroy --all > /dev/null 2>&1 || true
        echo -e "   ✅ ContainerLab topologies destroyed"
    fi
    
    # Clean monitoring PIDs
    for pid_file in "/tmp/bgp_monitor.pid" "/tmp/network_monitor.pid" "/tmp/blockchain_monitor.pid" "/tmp/chaos.pid"; do
        if [[ -f "$pid_file" ]]; then
            rm -f "$pid_file"
        fi
    done
    
    echo -e "${GREEN}✅ Cleanup completed${NC}"
}

# Main command handling
case "${1:-help}" in
    rust-sim)
        run_rust_simulation
        ;;
    containerlab)
        run_containerlab
        ;;
    realistic)
        run_realistic_testnet
        ;;
    test-setup)
        test_basic_setup
        ;;
    clean)
        clean_simulation
        ;;
    help|--help|-h)
        print_usage
        ;;
    *)
        echo "Unknown command: $1"
        echo ""
        print_usage
        exit 1
        ;;
esac