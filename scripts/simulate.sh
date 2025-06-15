#!/bin/bash

# PolyTorus Multi-Node Simulation Manager
# Provides easy commands to run various simulation scenarios

set -e

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m'

# Script directory
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_DIR="$(dirname "$SCRIPT_DIR")"

print_header() {
    echo -e "${BLUE}"
    echo "╔══════════════════════════════════════════════╗"
    echo "║        PolyTorus Multi-Node Simulator        ║"
    echo "║              Transaction Testing             ║"
    echo "╚══════════════════════════════════════════════╝"
    echo -e "${NC}"
}

print_help() {
    print_header
    echo -e "${CYAN}Usage: $0 <command> [options]${NC}"
    echo ""
    echo -e "${YELLOW}Commands:${NC}"
    echo -e "  ${GREEN}local${NC}     - Run simulation on local machine"
    echo -e "  ${GREEN}docker${NC}    - Run simulation with Docker Compose"
    echo -e "  ${GREEN}rust${NC}      - Run Rust-based multi-node simulation"
    echo -e "  ${GREEN}status${NC}    - Check running simulation status"
    echo -e "  ${GREEN}stop${NC}      - Stop all running simulations"
    echo -e "  ${GREEN}clean${NC}     - Clean up all simulation data"
    echo -e "  ${GREEN}logs${NC}      - Show simulation logs"
    echo -e "  ${GREEN}help${NC}      - Show this help message"
    echo ""
    echo -e "${YELLOW}Local Options:${NC}"
    echo -e "  --nodes <N>       Number of nodes (default: 4)"
    echo -e "  --duration <S>    Simulation duration in seconds (default: 300)"  
    echo -e "  --interval <MS>   Transaction interval in milliseconds (default: 5000)"
    echo -e "  --base-port <P>   Base HTTP port (default: 9000)"
    echo -e "  --p2p-port <P>    Base P2P port (default: 8000)"
    echo ""
    echo -e "${YELLOW}Examples:${NC}"
    echo -e "  $0 local --nodes 6 --duration 600"
    echo -e "  $0 docker"
    echo -e "  $0 rust --nodes 3 --interval 3000"
    echo -e "  $0 status"
    echo -e "  $0 logs"
}

check_dependencies() {
    local missing_deps=()
    
    # Check for required tools
    if ! command -v cargo &> /dev/null; then
        missing_deps+=("cargo (Rust)")
    fi
    
    if [[ "$1" == "docker" ]] && ! command -v docker &> /dev/null; then
        missing_deps+=("docker")
    fi
    
    if [[ "$1" == "docker" ]] && ! command -v docker-compose &> /dev/null; then
        missing_deps+=("docker-compose")
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
        exit 1
    fi
}

build_project() {
    echo -e "${BLUE}🔨 Building PolyTorus...${NC}"
    cd "$PROJECT_DIR"
    
    if cargo build --release; then
        echo -e "${GREEN}✅ Build successful${NC}"
    else
        echo -e "${RED}❌ Build failed${NC}"
        exit 1
    fi
}

run_local_simulation() {
    local nodes=4
    local duration=300
    local interval=5000
    local base_port=9000
    local p2p_port=8000
    
    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --nodes)
                nodes="$2"
                shift 2
                ;;
            --duration)
                duration="$2"
                shift 2
                ;;
            --interval)
                interval="$2"
                shift 2
                ;;
            --base-port)
                base_port="$2"
                shift 2
                ;;
            --p2p-port)
                p2p_port="$2"
                shift 2
                ;;
            *)
                echo -e "${RED}Unknown option: $1${NC}"
                exit 1
                ;;
        esac
    done
    
    print_header
    echo -e "${CYAN}🚀 Starting Local Multi-Node Simulation${NC}"
    echo -e "   Nodes: $nodes"
    echo -e "   Duration: ${duration}s"
    echo -e "   TX Interval: ${interval}ms"
    echo -e "   Base Port: $base_port"
    echo -e "   P2P Port: $p2p_port"
    echo ""
    
    check_dependencies "local"
    build_project
    
    # Run local simulation script
    "$SCRIPT_DIR/multi_node_simulation.sh" "$nodes" "$base_port" "$p2p_port" "$duration"
}

run_docker_simulation() {
    print_header
    echo -e "${CYAN}🐳 Starting Docker Multi-Node Simulation${NC}"
    
    check_dependencies "docker"
    
    cd "$PROJECT_DIR"
    
    echo -e "${BLUE}📦 Building Docker images...${NC}"
    if docker-compose build; then
        echo -e "${GREEN}✅ Docker images built successfully${NC}"
    else
        echo -e "${RED}❌ Docker build failed${NC}"
        exit 1
    fi
    
    echo -e "${BLUE}🚀 Starting containers...${NC}"
    docker-compose up --remove-orphans
}

run_rust_simulation() {
    local nodes=4
    local duration=300
    local interval=5000
    
    # Parse arguments
    while [[ $# -gt 0 ]]; do
        case $1 in
            --nodes)
                nodes="$2"
                shift 2
                ;;
            --duration)
                duration="$2"
                shift 2
                ;;
            --interval)
                interval="$2"
                shift 2
                ;;
            *)
                echo -e "${RED}Unknown option: $1${NC}"
                exit 1
                ;;
        esac
    done
    
    print_header
    echo -e "${CYAN}🦀 Starting Rust Multi-Node Simulation${NC}"
    echo -e "   Nodes: $nodes"
    echo -e "   Duration: ${duration}s"
    echo -e "   TX Interval: ${interval}ms"
    echo ""
    
    check_dependencies "rust"
    build_project
    
    cd "$PROJECT_DIR"
    cargo run --example multi_node_simulation -- \
        --nodes "$nodes" \
        --duration "$duration" \
        --interval "$interval"
}

show_status() {
    print_header
    echo -e "${CYAN}📊 Simulation Status${NC}"
    echo ""
    
    # Check for running processes
    if pgrep -f "polytorus" > /dev/null; then
        echo -e "${GREEN}✅ PolyTorus processes running:${NC}"
        pgrep -f "polytorus" | while read -r pid; do
            ps -p "$pid" -o pid,ppid,cmd --no-headers
        done
    else
        echo -e "${YELLOW}⚠️  No PolyTorus processes found${NC}"
    fi
    
    echo ""
    
    # Check Docker containers
    if command -v docker &> /dev/null; then
        if docker ps --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}" | grep -q "polytorus"; then
            echo -e "${GREEN}✅ Docker containers running:${NC}"
            docker ps --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}" | grep "polytorus"
        else
            echo -e "${YELLOW}⚠️  No PolyTorus Docker containers found${NC}"
        fi
    fi
    
    echo ""
    
    # Check for API endpoints
    echo -e "${BLUE}🌐 Checking API endpoints:${NC}"
    for port in {9000..9005}; do
        if curl -s --connect-timeout 2 "http://127.0.0.1:$port/status" > /dev/null 2>&1; then
            echo -e "   ✅ Node API responding on port $port"
        fi
    done
}

stop_simulation() {
    print_header
    echo -e "${CYAN}🛑 Stopping All Simulations${NC}"
    
    # Stop shell script processes
    if [[ -f "/tmp/polytorus_pids.txt" ]]; then
        echo -e "${BLUE}Stopping shell script processes...${NC}"
        while read -r pid; do
            if kill -0 "$pid" 2>/dev/null; then
                echo -e "   Stopping process $pid"
                kill "$pid" 2>/dev/null || true
            fi
        done < "/tmp/polytorus_pids.txt"
        rm -f "/tmp/polytorus_pids.txt"
    fi
    
    # Stop all polytorus processes
    if pgrep -f "polytorus" > /dev/null; then
        echo -e "${BLUE}Stopping PolyTorus processes...${NC}"
        pkill -f "polytorus" || true
    fi
    
    # Stop Docker containers
    if command -v docker-compose &> /dev/null && [[ -f "$PROJECT_DIR/docker-compose.yml" ]]; then
        echo -e "${BLUE}Stopping Docker containers...${NC}"
        cd "$PROJECT_DIR"
        docker-compose down --remove-orphans
    fi
    
    echo -e "${GREEN}✅ All simulations stopped${NC}"
}

clean_data() {
    print_header
    echo -e "${CYAN}🧹 Cleaning Simulation Data${NC}"
    
    # Stop everything first
    stop_simulation
    
    # Clean data directories
    if [[ -d "$PROJECT_DIR/data/simulation" ]]; then
        echo -e "${BLUE}Removing simulation data...${NC}"
        rm -rf "$PROJECT_DIR/data/simulation"
        echo -e "   ✅ Simulation data removed"
    fi
    
    # Clean Docker volumes
    if command -v docker &> /dev/null; then
        echo -e "${BLUE}Cleaning Docker volumes...${NC}"
        docker volume ls -q | grep -E "(polytorus|simulation)" | xargs -r docker volume rm || true
    fi
    
    # Clean logs
    if [[ -d "$PROJECT_DIR/logs" ]]; then
        echo -e "${BLUE}Cleaning logs...${NC}"
        find "$PROJECT_DIR/logs" -name "*.log" -delete || true
    fi
    
    echo -e "${GREEN}✅ Cleanup completed${NC}"
}

show_logs() {
    print_header
    echo -e "${CYAN}📋 Simulation Logs${NC}"
    echo ""
    
    # Show log files if they exist
    if [[ -d "$PROJECT_DIR/data/simulation" ]]; then
        echo -e "${BLUE}Available log files:${NC}"
        find "$PROJECT_DIR/data/simulation" -name "*.log" -type f | while read -r log_file; do
            file_size=$(du -h "$log_file" | cut -f1)
            echo -e "   📄 $log_file ($file_size)"
        done
        
        echo ""
        echo -e "${YELLOW}Recent log entries:${NC}"
        find "$PROJECT_DIR/data/simulation" -name "*.log" -type f -exec tail -n 5 {} \; -exec echo "" \;
    else
        echo -e "${YELLOW}No simulation logs found${NC}"
    fi
    
    # Show Docker logs if containers are running
    if command -v docker &> /dev/null; then
        docker ps --format "{{.Names}}" | grep -E "polytorus" | while read -r container; do
            echo -e "${BLUE}Docker logs for $container:${NC}"
            docker logs --tail 10 "$container" 2>/dev/null || true
            echo ""
        done
    fi
}

# Main command handling
case "${1:-help}" in
    local)
        shift
        run_local_simulation "$@"
        ;;
    docker)
        run_docker_simulation
        ;;
    rust)
        shift
        run_rust_simulation "$@"
        ;;
    status)
        show_status
        ;;
    stop)
        stop_simulation
        ;;
    clean)
        clean_data
        ;;
    logs)
        show_logs
        ;;
    help|--help|-h)
        print_help
        ;;
    *)
        echo -e "${RED}Unknown command: $1${NC}"
        echo ""
        print_help
        exit 1
        ;;
esac
