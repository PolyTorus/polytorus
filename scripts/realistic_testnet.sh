#!/bin/bash

# Realistic PolyTorus Testnet with AS Separation
# This script simulates a global blockchain network with realistic network conditions

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
TOPOLOGY_FILE="containerlab-topology-realistic.yml"
SIMULATION_DURATION=${1:-1800}  # 30 minutes default
CHAOS_ENABLED=${2:-true}        # Enable chaos engineering
MONITORING_ENABLED=${3:-true}   # Enable monitoring

print_header() {
    echo -e "${BLUE}"
    echo "╔══════════════════════════════════════════════════════════════════════╗"
    echo "║                  PolyTorus Realistic Global Testnet                  ║"
    echo "║                    With AS Separation & BGP Routing                  ║"
    echo "╚══════════════════════════════════════════════════════════════════════╝"
    echo -e "${NC}"
}

print_config() {
    echo -e "${CYAN}🌍 Global Network Configuration:${NC}"
    echo -e "   Duration: ${SIMULATION_DURATION}s ($(($SIMULATION_DURATION / 60)) minutes)"
    echo -e "   Chaos Engineering: ${CHAOS_ENABLED}"
    echo -e "   Monitoring: ${MONITORING_ENABLED}"
    echo ""
    echo -e "${YELLOW}📊 Network Architecture:${NC}"
    echo -e "   AS 65001 (North America): Bootstrap + Mining Pool"
    echo -e "   AS 65002 (Europe): Institutional + Research"
    echo -e "   AS 65003 (Asia Pacific): Mobile + IoT"
    echo -e "   AS 65004 (Edge/Mobile): Rural + Mobile Edge"
    echo ""
    echo -e "${PURPLE}🔗 Realistic Network Conditions:${NC}"
    echo -e "   Trans-Atlantic: 100ms latency, 0.01% loss"
    echo -e "   Trans-Pacific: 180ms latency, 0.02% loss"
    echo -e "   Satellite Links: 600ms latency, 2% loss"
    echo -e "   Mobile Connections: 80ms latency, 0.8% loss"
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
    
    if ! command -v tc &> /dev/null; then
        missing_deps+=("iproute2 (tc)")
    fi
    
    if ! command -v iperf3 &> /dev/null; then
        missing_deps+=("iperf3")
    fi
    
    if [[ ${#missing_deps[@]} -gt 0 ]]; then
        echo -e "${RED}❌ Missing dependencies:${NC}"
        for dep in "${missing_deps[@]}"; do
            echo -e "   - $dep"
        done
        exit 1
    fi
}

build_docker_images() {
    echo -e "${BLUE}🔨 Building Docker images...${NC}"
    
    # Build main PolyTorus image
    echo -e "   Building PolyTorus image..."
    docker build -t polytorus:latest . || {
        echo -e "${RED}❌ Failed to build PolyTorus image${NC}"
        exit 1
    }
    
    # Ensure FRRouting image is available
    echo -e "   Pulling FRRouting image..."
    docker pull frrouting/frr:latest || {
        echo -e "${RED}❌ Failed to pull FRRouting image${NC}"
        exit 1
    }
    
    echo -e "${GREEN}✅ Docker images ready${NC}"
}

prepare_environment() {
    echo -e "${BLUE}📁 Preparing realistic testnet environment...${NC}"
    
    # Create data directories for each region
    mkdir -p "./data/realistic"
    
    # Create regional data directories
    regions=("na-bootstrap" "na-mining" "eu-institutional" "eu-research" 
             "ap-mobile" "ap-iot" "edge-rural" "edge-mobile")
    
    for region in "${regions[@]}"; do
        mkdir -p "./data/realistic/$region"/{wallets,blockchain,contracts,modular_storage,logs}
        echo -e "   📁 Created data directory for $region"
    done
    
    # Ensure FRR configuration directories exist
    mkdir -p "./config/frr"
    
    echo -e "${GREEN}✅ Environment prepared${NC}"
}

generate_mining_wallets() {
    echo -e "${BLUE}🔑 Generating region-specific mining wallets...${NC}"
    
    # Mining nodes that need wallets
    miners=("na-mining" "eu-research" "ap-iot")
    
    for miner in "${miners[@]}"; do
        echo -e "   Creating wallet for $miner..."
        
        export POLYTORUS_DATA_DIR="./data/realistic/$miner"
        
        # Create wallet using Rust binary
        if timeout 30s cargo run --release --bin polytorus -- --data-dir "$POLYTORUS_DATA_DIR" --createwallet; then
            echo -e "   ✅ Wallet created for $miner"
            
            # Get the wallet address
            WALLET_ADDRESS=$(timeout 10s cargo run --release --bin polytorus -- --data-dir "$POLYTORUS_DATA_DIR" --listaddresses | tail -n 1 | grep -oE '[A-Za-z0-9]{25,}' | head -n 1)
            
            if [[ -n "$WALLET_ADDRESS" ]]; then
                echo -e "   📝 Mining address for $miner: $WALLET_ADDRESS"
                echo "$WALLET_ADDRESS" > "./data/realistic/$miner/mining_address.txt"
                
                # Update topology with real address
                sed -i "s/${miner}_address/$WALLET_ADDRESS/g" "$TOPOLOGY_FILE"
            else
                echo -e "   ⚠️  Using default address for $miner"
                echo "${miner}_default_address" > "./data/realistic/$miner/mining_address.txt"
            fi
        else
            echo -e "   ⚠️  Failed to create wallet for $miner, using default"
            echo "${miner}_default_address" > "./data/realistic/$miner/mining_address.txt"
        fi
    done
}

start_realistic_testnet() {
    echo -e "${BLUE}🚀 Starting realistic global testnet...${NC}"
    
    # Deploy ContainerLab topology
    if containerlab deploy --topo "$TOPOLOGY_FILE" --reconfigure; then
        echo -e "${GREEN}✅ ContainerLab topology deployed${NC}"
    else
        echo -e "${RED}❌ Failed to deploy topology${NC}"
        exit 1
    fi
}

configure_network_impairments() {
    echo -e "${BLUE}🌐 Configuring realistic network impairments...${NC}"
    
    # Wait for containers to be ready
    sleep 30
    
    # Configure traffic control for realistic conditions
    echo -e "   Configuring inter-AS latency and packet loss..."
    
    # These would be applied inside containers via their startup commands
    # The actual tc commands are in the containerlab topology file
    
    echo -e "${GREEN}✅ Network impairments configured${NC}"
}

wait_for_network_convergence() {
    echo -e "${BLUE}⏳ Waiting for BGP convergence and node startup...${NC}"
    
    # Wait longer for BGP convergence and international connections
    local wait_time=120
    echo -e "   Waiting ${wait_time} seconds for global network convergence..."
    
    for ((i=1; i<=wait_time; i++)); do
        if [[ $((i % 20)) -eq 0 ]]; then
            echo -e "   📊 Convergence progress: ${i}/${wait_time}s"
        fi
        sleep 1
    done
    
    echo -e "${BLUE}📊 Checking node status across regions...${NC}"
    
    # Check each regional node
    regions=(
        "9000:NA-Bootstrap"
        "9001:NA-Mining"
        "9002:EU-Institutional"
        "9003:EU-Research"
        "9004:AP-Mobile"
        "9005:AP-IoT"
        "9006:Edge-Rural"
        "9007:Edge-Mobile"
    )
    
    for region_info in "${regions[@]}"; do
        IFS=':' read -r port name <<< "$region_info"
        
        if curl -s --connect-timeout 5 "http://localhost:$port/status" > /dev/null 2>&1; then
            echo -e "   ✅ $name (port $port) is responding"
        else
            echo -e "   ⚠️  $name (port $port) may still be starting"
        fi
    done
}

start_monitoring() {
    if [[ "$MONITORING_ENABLED" == "true" ]]; then
        echo -e "${BLUE}📊 Starting comprehensive monitoring...${NC}"
        
        # Start BGP monitoring
        monitor_bgp_status &
        BGP_MONITOR_PID=$!
        
        # Start network performance monitoring
        monitor_network_performance &
        NETWORK_MONITOR_PID=$!
        
        # Start blockchain metrics monitoring
        monitor_blockchain_metrics &
        BLOCKCHAIN_MONITOR_PID=$!
        
        echo -e "   📈 BGP Monitor PID: $BGP_MONITOR_PID"
        echo -e "   🌐 Network Monitor PID: $NETWORK_MONITOR_PID"
        echo -e "   ⛓️  Blockchain Monitor PID: $BLOCKCHAIN_MONITOR_PID"
        
        # Store PIDs for cleanup
        echo "$BGP_MONITOR_PID" > /tmp/bgp_monitor.pid
        echo "$NETWORK_MONITOR_PID" > /tmp/network_monitor.pid
        echo "$BLOCKCHAIN_MONITOR_PID" > /tmp/blockchain_monitor.pid
    fi
}

monitor_bgp_status() {
    echo -e "${YELLOW}🔍 Starting BGP status monitoring...${NC}"
    
    while true; do
        sleep 60
        
        echo -e "\n${CYAN}📡 BGP Status Report:${NC}"
        
        # Check BGP status on each router
        routers=("router-na-east" "router-eu" "router-ap" "router-edge")
        
        for router in "${routers[@]}"; do
            if docker exec "clab-polytorus-realistic-testnet-$router" vtysh -c "show bgp summary" 2>/dev/null | head -n 10; then
                echo -e "   ✅ $router BGP operational"
            else
                echo -e "   ❌ $router BGP issues detected"
            fi
        done
    done
}

monitor_network_performance() {
    echo -e "${YELLOW}🌐 Starting network performance monitoring...${NC}"
    
    while true; do
        sleep 120
        
        echo -e "\n${CYAN}🚀 Network Performance Report:${NC}"
        
        # Test latency between regions
        echo -e "   📊 Inter-regional latency:"
        
        # Ping from NA to EU
        if NA_EU_LATENCY=$(docker exec clab-polytorus-realistic-testnet-node-na-bootstrap ping -c 3 172.100.2.20 2>/dev/null | tail -n 1 | cut -d'/' -f5); then
            echo -e "     NA → EU: ${NA_EU_LATENCY}ms"
        fi
        
        # Ping from EU to AP
        if EU_AP_LATENCY=$(docker exec clab-polytorus-realistic-testnet-node-eu-institutional ping -c 3 172.100.3.20 2>/dev/null | tail -n 1 | cut -d'/' -f5); then
            echo -e "     EU → AP: ${EU_AP_LATENCY}ms"
        fi
        
        # Ping to satellite link
        if SATELLITE_LATENCY=$(docker exec clab-polytorus-realistic-testnet-node-edge-rural ping -c 3 172.100.1.20 2>/dev/null | tail -n 1 | cut -d'/' -f5); then
            echo -e "     Satellite: ${SATELLITE_LATENCY}ms"
        fi
    done
}

monitor_blockchain_metrics() {
    echo -e "${YELLOW}⛓️  Starting blockchain metrics monitoring...${NC}"
    
    while true; do
        sleep 90
        
        echo -e "\n${CYAN}⛏️  Blockchain Status Report:${NC}"
        
        # Check each node's blockchain status
        regions=(
            "9000:NA-Bootstrap:exchange"
            "9001:NA-Mining:mining_pool"
            "9002:EU-Institutional:institutional"
            "9003:EU-Research:research"
            "9004:AP-Mobile:mobile_backend"
            "9005:AP-IoT:iot_infrastructure"
            "9006:Edge-Rural:light_client"
            "9007:Edge-Mobile:mobile_edge"
        )
        
        for region_info in "${regions[@]}"; do
            IFS=':' read -r port name type <<< "$region_info"
            
            if RESPONSE=$(curl -s --connect-timeout 3 "http://localhost:$port/status" 2>/dev/null); then
                # Extract metrics from response
                BLOCK_HEIGHT=$(echo "$RESPONSE" | grep -o '"block_height":[0-9]*' | cut -d':' -f2)
                echo -e "     $name ($type): Block ${BLOCK_HEIGHT:-'unknown'}"
            else
                echo -e "     $name ($type): Offline"
            fi
        done
    done
}

start_chaos_engineering() {
    if [[ "$CHAOS_ENABLED" == "true" ]]; then
        echo -e "${BLUE}🔥 Starting chaos engineering...${NC}"
        
        # Start chaos scenarios
        chaos_network_partitions &
        CHAOS_PID=$!
        echo "$CHAOS_PID" > /tmp/chaos.pid
        
        echo -e "   💥 Chaos Engineering PID: $CHAOS_PID"
    fi
}

chaos_network_partitions() {
    echo -e "${YELLOW}💥 Starting network partition chaos...${NC}"
    
    while true; do
        # Wait random time between 300-900 seconds (5-15 minutes)
        local wait_time=$((300 + RANDOM % 600))
        sleep $wait_time
        
        # Randomly select a partition scenario
        local scenarios=("transatlantic" "transpacific" "regional_isolation" "satellite_storm")
        local scenario=${scenarios[$RANDOM % ${#scenarios[@]}]}
        
        echo -e "\n${RED}💥 CHAOS: Simulating $scenario partition${NC}"
        
        case $scenario in
            "transatlantic")
                simulate_transatlantic_partition
                ;;
            "transpacific")
                simulate_transpacific_partition
                ;;
            "regional_isolation")
                simulate_regional_isolation
                ;;
            "satellite_storm")
                simulate_satellite_storm
                ;;
        esac
    done
}

simulate_transatlantic_partition() {
    echo -e "   🌊 Simulating transatlantic cable cut (300s)..."
    
    # Block traffic between NA and EU routers
    docker exec clab-polytorus-realistic-testnet-router-na-east iptables -A OUTPUT -d 172.100.2.0/24 -j DROP 2>/dev/null || true
    docker exec clab-polytorus-realistic-testnet-router-eu iptables -A OUTPUT -d 172.100.1.0/24 -j DROP 2>/dev/null || true
    
    sleep 300
    
    # Restore connectivity
    echo -e "   🔧 Restoring transatlantic connectivity..."
    docker exec clab-polytorus-realistic-testnet-router-na-east iptables -D OUTPUT -d 172.100.2.0/24 -j DROP 2>/dev/null || true
    docker exec clab-polytorus-realistic-testnet-router-eu iptables -D OUTPUT -d 172.100.1.0/24 -j DROP 2>/dev/null || true
}

simulate_satellite_storm() {
    echo -e "   🛰️  Simulating satellite interference (600s)..."
    
    # Increase latency and packet loss on edge nodes
    docker exec clab-polytorus-realistic-testnet-node-edge-rural tc qdisc change dev eth0 root netem delay 1200ms 200ms loss 10% 2>/dev/null || true
    
    sleep 600
    
    # Restore normal satellite conditions
    echo -e "   📡 Restoring normal satellite conditions..."
    docker exec clab-polytorus-realistic-testnet-node-edge-rural tc qdisc change dev eth0 root netem delay 600ms 100ms loss 2% 2>/dev/null || true
}

generate_realistic_transactions() {
    echo -e "${BLUE}💸 Starting realistic transaction generation...${NC}"
    
    # Start transaction generators for different patterns
    generate_business_transactions &
    generate_cross_border_transactions &
    generate_mobile_transactions &
    
    echo -e "${GREEN}✅ Transaction generators started${NC}"
}

generate_business_transactions() {
    local tx_count=0
    
    while true; do
        # Simulate business hours traffic patterns
        local current_hour=$(date +%H)
        local multiplier=1
        
        # Increase traffic during business hours (9-17)
        if [[ $current_hour -ge 9 && $current_hour -le 17 ]]; then
            multiplier=3
        fi
        
        # Generate transactions between institutional nodes
        for ((i=0; i<multiplier; i++)); do
            generate_single_transaction "9000" "9002" "$tx_count" "institutional"
            tx_count=$((tx_count + 1))
            sleep 30
        done
        
        sleep 60
    done
}

generate_cross_border_transactions() {
    local tx_count=0
    
    while true; do
        # Cross-border transactions (30% of traffic)
        if [[ $((RANDOM % 10)) -lt 3 ]]; then
            # Random cross-region transaction
            local regions=("9000" "9002" "9004")
            local from=${regions[$RANDOM % ${#regions[@]}]}
            local to=${regions[$RANDOM % ${#regions[@]}]}
            
            if [[ "$from" != "$to" ]]; then
                generate_single_transaction "$from" "$to" "$tx_count" "cross_border"
                tx_count=$((tx_count + 1))
            fi
        fi
        
        sleep 45
    done
}

generate_mobile_transactions() {
    local tx_count=0
    
    while true; do
        # Mobile and IoT transactions
        generate_single_transaction "9004" "9005" "$tx_count" "mobile_iot"
        generate_single_transaction "9006" "9007" "$tx_count" "edge"
        
        tx_count=$((tx_count + 2))
        sleep 20
    done
}

generate_single_transaction() {
    local from_port=$1
    local to_port=$2
    local tx_id=$3
    local tx_type=$4
    
    local amount=$((100 + RANDOM % 900))
    
    local tx_data="{\"from\":\"port_${from_port}\",\"to\":\"port_${to_port}\",\"amount\":$amount,\"type\":\"$tx_type\",\"nonce\":$tx_id}"
    
    if curl -s -X POST \
        -H "Content-Type: application/json" \
        -d "$tx_data" \
        "http://localhost:$to_port/transaction" > /dev/null 2>&1; then
        echo -e "   💸 TX $tx_id ($tx_type): Port $from_port -> $to_port (${amount} units)"
    fi
}

show_final_statistics() {
    echo -e "\n${BLUE}📈 Final Global Network Statistics:${NC}"
    echo -e "========================================"
    
    # BGP Summary
    echo -e "\n${CYAN}📡 BGP Routing Summary:${NC}"
    for router in "router-na-east" "router-eu" "router-ap" "router-edge"; do
        echo -e "\n   🌐 $router:"
        docker exec "clab-polytorus-realistic-testnet-$router" vtysh -c "show bgp summary" 2>/dev/null | tail -n 5 || echo "     BGP data unavailable"
    done
    
    # Node Performance Summary
    echo -e "\n${CYAN}⛓️  Blockchain Node Summary:${NC}"
    regions=(
        "9000:NA-Bootstrap"
        "9001:NA-Mining"
        "9002:EU-Institutional"
        "9003:EU-Research"
        "9004:AP-Mobile"
        "9005:AP-IoT"
        "9006:Edge-Rural"
        "9007:Edge-Mobile"
    )
    
    for region_info in "${regions[@]}"; do
        IFS=':' read -r port name <<< "$region_info"
        echo -e "\n   📊 $name (Port $port):"
        
        if STATUS=$(curl -s --connect-timeout 5 "http://localhost:$port/status" 2>/dev/null); then
            echo -e "     Status: $STATUS" | head -n 3
        else
            echo -e "     Status: Offline or unreachable"
        fi
    done
    
    # Network Quality Summary
    echo -e "\n${CYAN}🌐 Network Quality Summary:${NC}"
    echo -e "   📡 Simulated real-world conditions:"
    echo -e "     - Trans-Atlantic: ~100ms latency"
    echo -e "     - Trans-Pacific: ~180ms latency"
    echo -e "     - Satellite: ~600ms latency"
    echo -e "     - Mobile Edge: ~80ms latency"
    echo -e "   💥 Chaos scenarios executed during simulation"
    echo -e "   🔒 Compliance policies enforced per region"
}

cleanup() {
    echo -e "\n${YELLOW}🧹 Cleaning up realistic testnet...${NC}"
    
    # Stop monitoring processes
    for pid_file in "/tmp/bgp_monitor.pid" "/tmp/network_monitor.pid" "/tmp/blockchain_monitor.pid" "/tmp/chaos.pid"; do
        if [[ -f "$pid_file" ]]; then
            PID=$(cat "$pid_file")
            if kill -0 "$PID" 2>/dev/null; then
                kill "$PID" 2>/dev/null || true
            fi
            rm -f "$pid_file"
        fi
    done
    
    # Destroy ContainerLab topology
    echo -e "${BLUE}🗑️  Destroying realistic testnet topology...${NC}"
    containerlab destroy --topo "$TOPOLOGY_FILE" --cleanup || true
    
    echo -e "${GREEN}✅ Cleanup completed${NC}"
}

# Set up signal handlers
trap cleanup SIGINT SIGTERM EXIT

# Main execution
main() {
    print_header
    print_config
    
    check_dependencies
    build_docker_images
    prepare_environment
    generate_mining_wallets
    start_realistic_testnet
    configure_network_impairments
    wait_for_network_convergence
    start_monitoring
    start_chaos_engineering
    generate_realistic_transactions
    
    echo -e "\n${GREEN}🌍 Realistic global testnet is running!${NC}"
    echo -e "${YELLOW}💡 Regional APIs:${NC}"
    echo -e "   NA Bootstrap: http://localhost:9000"
    echo -e "   NA Mining: http://localhost:9001"
    echo -e "   EU Institutional: http://localhost:9002"
    echo -e "   EU Research: http://localhost:9003"
    echo -e "   AP Mobile: http://localhost:9004"
    echo -e "   AP IoT: http://localhost:9005"
    echo -e "   Edge Rural: http://localhost:9006"
    echo -e "   Edge Mobile: http://localhost:9007"
    
    echo -e "\n${CYAN}Press Ctrl+C to stop the testnet...${NC}"
    
    # Wait for simulation duration
    sleep $SIMULATION_DURATION
    
    echo -e "\n${GREEN}🏁 Realistic testnet simulation completed!${NC}"
    show_final_statistics
}

# Check if running as source or executed
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi