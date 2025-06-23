#!/bin/bash

# Enhanced ContainerLab Realistic Testnet Simulation
# Simulates global blockchain network with AS separation, geographic distribution,
# realistic latency/bandwidth constraints, and BGP-like routing

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
TOPOLOGY_FILE="containerlab-topology-enhanced.yml"
SIMULATION_DURATION=${1:-1800}    # 30 minutes default
NUM_TRANSACTIONS=${2:-200}        # Number of transactions to generate
TX_INTERVAL=${3:-15}              # Transaction interval in seconds
CHAOS_MODE=${4:-false}            # Enable chaos engineering

# Simulation parameters
ENABLE_PARTITION_TESTING=${ENABLE_PARTITION_TESTING:-true}
ENABLE_PERFORMANCE_MONITORING=${ENABLE_PERFORMANCE_MONITORING:-true}
ENABLE_BGP_MONITORING=${ENABLE_BGP_MONITORING:-true}

print_header() {
    echo -e "${BLUE}"
    echo "╔══════════════════════════════════════════════════════════════════════════════╗"
    echo "║           PolyTorus Realistic Testnet Simulation with AS Separation         ║"
    echo "║                    Global Network • BGP Routing • Chaos Testing             ║"
    echo "╚══════════════════════════════════════════════════════════════════════════════╝"
    echo -e "${NC}"
}

print_config() {
    echo -e "${CYAN}🌍 Global Network Simulation Configuration:${NC}"
    echo -e "   Duration: ${SIMULATION_DURATION}s ($(($SIMULATION_DURATION / 60)) minutes)"
    echo -e "   Transactions: ${NUM_TRANSACTIONS}"
    echo -e "   TX Interval: ${TX_INTERVAL}s"
    echo -e "   Chaos Mode: ${CHAOS_MODE}"
    echo -e "   Network Topology: Multi-AS with geographic distribution"
    echo ""
    echo -e "${PURPLE}🏗️  Network Architecture:${NC}"
    echo -e "   • AS65001 (North America): Bootstrap + Mining pools + Exchanges"
    echo -e "   • AS65002 (Europe): Institutional validators + Research nodes"  
    echo -e "   • AS65003 (Asia-Pacific): Mobile backends + IoT infrastructure"
    echo -e "   • AS65004 (Edge/Mobile): Light clients + Rural/satellite nodes"
    echo ""
    echo -e "${YELLOW}📊 Network Characteristics:${NC}"
    echo -e "   • Intra-region latency: 10-50ms"
    echo -e "   • Inter-region latency: 100-600ms" 
    echo -e "   • Bandwidth: 5Mbps (satellite) to 1Gbps (Tier-1)"
    echo -e "   • Packet loss: 0.01% (fiber) to 2% (satellite)"
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
    
    if ! command -v tc &> /dev/null; then
        missing_deps+=("tc (traffic control)")
    fi
    
    if ! command -v curl &> /dev/null; then
        missing_deps+=("curl")
    fi
    
    if ! command -v prometheus &> /dev/null; then
        echo -e "${YELLOW}⚠️  Prometheus not found - monitoring will be limited${NC}"
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
    echo -e "${BLUE}🔨 Building enhanced PolyTorus Docker image...${NC}"
    
    if docker build -t polytorus:latest .; then
        echo -e "${GREEN}✅ Docker image built successfully${NC}"
    else
        echo -e "${RED}❌ Docker build failed${NC}"
        exit 1
    fi
}

prepare_enhanced_environment() {
    echo -e "${BLUE}📁 Preparing realistic testnet environment...${NC}"
    
    # Create data directories for all nodes
    mkdir -p "./data/containerlab"
    
    # North America (AS65001)
    for node in bootstrap-na miner-pool-na exchange-na; do
        mkdir -p "./data/containerlab/$node"/{wallets,blockchain,contracts,modular_storage,logs}
    done
    
    # Europe (AS65002)  
    for node in validator-institution-eu research-eu; do
        mkdir -p "./data/containerlab/$node"/{wallets,blockchain,contracts,modular_storage,logs}
    done
    
    # Asia-Pacific (AS65003)
    for node in miner-apac mobile-backend-apac; do
        mkdir -p "./data/containerlab/$node"/{wallets,blockchain,contracts,modular_storage,logs}
    done
    
    # Edge/Mobile (AS65004)
    for node in light-client-mobile rural-satellite; do
        mkdir -p "./data/containerlab/$node"/{wallets,blockchain,contracts,modular_storage,logs}
    done
    
    # Create monitoring directories
    mkdir -p "./data/monitoring"/{prometheus,grafana,logs}
    
    # Prepare network configuration files
    prepare_network_configs
    
    echo -e "${GREEN}✅ Enhanced environment prepared${NC}"
}

prepare_network_configs() {
    echo -e "${BLUE}⚙️  Preparing network configuration files...${NC}"
    
    # Ensure FRR configurations exist
    if [[ ! -f "./config/frr/router-na/frr.conf" ]]; then
        echo -e "${YELLOW}⚠️  FRR configurations not found - creating basic configs${NC}"
        mkdir -p "./config/frr"/{router-na,router-eu,router-apac,router-edge}
        
        # Create basic FRR configs (simplified versions)
        for router in router-na router-eu router-apac router-edge; do
            cat > "./config/frr/$router/frr.conf" << EOF
frr version 8.0
frr defaults traditional
hostname $router
log syslog informational
service integrated-vtysh-config
line vty
end
EOF
        done
    fi
    
    # Create enhanced realistic testnet config if it doesn't exist
    if [[ ! -f "./config/realistic-testnet.toml" ]]; then
        echo -e "${YELLOW}⚠️  Realistic testnet config not found - using docker-node.toml${NC}"
        cp "./config/docker-node.toml" "./config/realistic-testnet.toml"
    fi
}

generate_mining_wallets() {
    echo -e "${BLUE}🔑 Generating mining wallets for global testnet...${NC}"
    
    # Create wallets for all mining nodes
    for miner in miner-pool-na miner-apac; do
        echo -e "   Creating wallet for: $miner"
        
        # Set data directory for this node
        export POLYTORUS_DATA_DIR="./data/containerlab/$miner"
        
        # Create wallet using Rust binary
        if cargo run --release -- --data-dir "$POLYTORUS_DATA_DIR" --createwallet; then
            echo -e "   ✅ Wallet created for $miner"
            
            # Get the wallet address
            WALLET_ADDRESS=$(cargo run --release -- --data-dir "$POLYTORUS_DATA_DIR" --listaddresses | tail -n 1 | grep -oE '[A-Za-z0-9]{25,}' | head -n 1)
            
            if [[ -n "$WALLET_ADDRESS" ]]; then
                echo -e "   📝 Mining address for $miner: $WALLET_ADDRESS"
                echo "$WALLET_ADDRESS" > "./data/containerlab/$miner/mining_address.txt"
            else
                echo -e "   ⚠️  Could not extract wallet address for $miner"
                echo "${miner}_default_address" > "./data/containerlab/$miner/mining_address.txt"
            fi
        else
            echo -e "   ⚠️  Failed to create wallet for $miner, using default address"
            echo "${miner}_default_address" > "./data/containerlab/$miner/mining_address.txt"
        fi
    done
    
    # Update topology file with actual mining addresses
    update_topology_with_addresses
}

update_topology_with_addresses() {
    echo -e "${BLUE}⚙️  Updating topology with mining addresses...${NC}"
    
    # Read mining addresses
    MINER_POOL_NA_ADDRESS=$(cat "./data/containerlab/miner-pool-na/mining_address.txt" 2>/dev/null || echo "miner_pool_na_default")
    MINER_APAC_ADDRESS=$(cat "./data/containerlab/miner-apac/mining_address.txt" 2>/dev/null || echo "miner_apac_default")
    
    # Update the topology file with real addresses
    sed -i "s/miner_pool_na_address/$MINER_POOL_NA_ADDRESS/g" "$TOPOLOGY_FILE"
    sed -i "s/miner_apac_address/$MINER_APAC_ADDRESS/g" "$TOPOLOGY_FILE"
    
    echo -e "   ✅ Topology updated with mining addresses"
    echo -e "   📝 NA Mining Pool: $MINER_POOL_NA_ADDRESS"
    echo -e "   📝 APAC Miner: $MINER_APAC_ADDRESS"
}

start_containerlab() {
    echo -e "${BLUE}🚀 Starting enhanced ContainerLab topology...${NC}"
    
    if containerlab deploy --topo "$TOPOLOGY_FILE"; then
        echo -e "${GREEN}✅ Enhanced ContainerLab topology deployed successfully${NC}"
    else
        echo -e "${RED}❌ Failed to deploy ContainerLab topology${NC}"
        exit 1
    fi
}

wait_for_nodes() {
    echo -e "${BLUE}⏳ Waiting for global nodes to start...${NC}"
    sleep 45  # Longer wait for complex topology
    
    echo -e "${BLUE}📊 Checking global node status...${NC}"
    
    # Check all nodes across different regions
    declare -A node_ports=(
        ["bootstrap-na"]=9000
        ["miner-pool-na"]=9001
        ["exchange-na"]=9002
        ["validator-institution-eu"]=9010
        ["research-eu"]=9011
        ["miner-apac"]=9020
        ["mobile-backend-apac"]=9021
        ["light-client-mobile"]=9030
        ["rural-satellite"]=9031
    )
    
    for node in "${!node_ports[@]}"; do
        port="${node_ports[$node]}"
        if curl -s --connect-timeout 5 "http://localhost:$port/status" > /dev/null 2>&1; then
            echo -e "   ✅ $node (port $port) is responding"
        else
            echo -e "   ⚠️  $node (port $port) may still be starting up"
        fi
    done
}

start_enhanced_monitoring() {
    echo -e "${BLUE}📊 Starting enhanced network monitoring...${NC}"
    
    # Start BGP monitoring
    if [[ "$ENABLE_BGP_MONITORING" == "true" ]]; then
        monitor_bgp_status &
        BGP_MONITOR_PID=$!
        echo "$BGP_MONITOR_PID" > /tmp/bgp_monitor.pid
    fi
    
    # Start network performance monitoring
    if [[ "$ENABLE_PERFORMANCE_MONITORING" == "true" ]]; then
        monitor_network_performance &
        PERF_MONITOR_PID=$!
        echo "$PERF_MONITOR_PID" > /tmp/perf_monitor.pid
    fi
    
    # Start blockchain monitoring
    monitor_blockchain_metrics &
    BLOCKCHAIN_MONITOR_PID=$!
    echo "$BLOCKCHAIN_MONITOR_PID" > /tmp/blockchain_monitor.pid
    
    # Start transaction generation
    generate_realistic_transactions &
    TX_GENERATOR_PID=$!
    echo "$TX_GENERATOR_PID" > /tmp/tx_generator.pid
    
    echo -e "${GREEN}✅ Enhanced monitoring started${NC}"
    echo -e "   BGP monitor PID: ${BGP_MONITOR_PID:-'disabled'}"
    echo -e "   Performance monitor PID: ${PERF_MONITOR_PID:-'disabled'}"
    echo -e "   Blockchain monitor PID: $BLOCKCHAIN_MONITOR_PID"
    echo -e "   Transaction generator PID: $TX_GENERATOR_PID"
}

monitor_bgp_status() {
    echo -e "${YELLOW}🌐 Starting BGP status monitoring...${NC}"
    
    while true; do
        sleep 120  # Check every 2 minutes
        
        echo -e "\n${CYAN}🛣️  BGP Status Report:${NC}"
        
        # Check BGP status on all routers
        for router in router-na router-eu router-apac router-edge; do
            if docker exec "clab-polytorus-realistic-testnet-$router" vtysh -c "show ip bgp summary" 2>/dev/null | head -20; then
                echo -e "   📡 $router: BGP operational"
            else
                echo -e "   ❌ $router: BGP issues detected"
            fi
        done
    done
}

monitor_network_performance() {
    echo -e "${YELLOW}📊 Starting network performance monitoring...${NC}"
    
    while true; do
        sleep 60  # Check every minute
        
        echo -e "\n${CYAN}🔍 Network Performance Report:${NC}"
        
        # Test inter-AS connectivity and latency
        test_inter_as_connectivity
        
        # Monitor bandwidth utilization
        monitor_bandwidth_usage
        
        # Check for network partitions
        detect_network_partitions
    done
}

test_inter_as_connectivity() {
    local timestamp=$(date '+%Y-%m-%d %H:%M:%S')
    echo -e "   ⚡ Inter-AS Connectivity Test ($timestamp):"
    
    # Test NA to EU latency
    if ping -c 3 -W 2 10.2.0.10 > /dev/null 2>&1; then
        latency=$(ping -c 3 10.2.0.10 2>/dev/null | tail -1 | awk '{print $4}' | cut -d'/' -f2)
        echo -e "     NA → EU: ${latency}ms (target: ~100ms)"
    else
        echo -e "     NA → EU: ❌ Connection failed"
    fi
    
    # Test NA to APAC latency
    if ping -c 3 -W 2 10.3.0.10 > /dev/null 2>&1; then
        latency=$(ping -c 3 10.3.0.10 2>/dev/null | tail -1 | awk '{print $4}' | cut -d'/' -f2)
        echo -e "     NA → APAC: ${latency}ms (target: ~180ms)"
    else
        echo -e "     NA → APAC: ❌ Connection failed"
    fi
}

monitor_bandwidth_usage() {
    echo -e "   📈 Bandwidth Utilization:"
    
    # Simple bandwidth monitoring (requires enhanced implementation)
    for interface in eth0 eth1; do
        if [[ -f "/sys/class/net/$interface/statistics/rx_bytes" ]]; then
            rx_bytes=$(cat "/sys/class/net/$interface/statistics/rx_bytes" 2>/dev/null || echo "0")
            tx_bytes=$(cat "/sys/class/net/$interface/statistics/tx_bytes" 2>/dev/null || echo "0")
            echo -e "     $interface: RX: ${rx_bytes} bytes, TX: ${tx_bytes} bytes"
        fi
    done
}

detect_network_partitions() {
    echo -e "   🔍 Partition Detection:"
    
    # Check if all regions can reach bootstrap node
    local regions=("eu" "apac" "edge")
    local partition_detected=false
    
    for region in "${regions[@]}"; do
        case $region in
            "eu") test_ip="10.2.0.10" ;;
            "apac") test_ip="10.3.0.10" ;;
            "edge") test_ip="10.4.0.10" ;;
        esac
        
        if ! ping -c 1 -W 2 "$test_ip" > /dev/null 2>&1; then
            echo -e "     ⚠️  Partition detected: $region region unreachable"
            partition_detected=true
        fi
    done
    
    if [[ "$partition_detected" == "false" ]]; then
        echo -e "     ✅ No partitions detected - all regions connected"
    fi
}

monitor_blockchain_metrics() {
    echo -e "${YELLOW}⛓️  Starting blockchain metrics monitoring...${NC}"
    
    while true; do
        sleep 45
        
        echo -e "\n${CYAN}⛓️  Blockchain Status Report:${NC}"
        
        # Check all blockchain nodes
        declare -A node_ports=(
            ["bootstrap-na"]=9000
            ["miner-pool-na"]=9001
            ["exchange-na"]=9002
            ["validator-institution-eu"]=9010
            ["research-eu"]=9011
            ["miner-apac"]=9020
            ["mobile-backend-apac"]=9021
            ["light-client-mobile"]=9030
            ["rural-satellite"]=9031
        )
        
        declare -A node_regions=(
            ["bootstrap-na"]="NA"
            ["miner-pool-na"]="NA"
            ["exchange-na"]="NA"
            ["validator-institution-eu"]="EU"
            ["research-eu"]="EU"
            ["miner-apac"]="APAC"
            ["mobile-backend-apac"]="APAC"
            ["light-client-mobile"]="EDGE"
            ["rural-satellite"]="EDGE"
        )
        
        for node in "${!node_ports[@]}"; do
            port="${node_ports[$node]}"
            region="${node_regions[$node]}"
            
            if RESPONSE=$(curl -s --connect-timeout 3 "http://localhost:$port/status" 2>/dev/null); then
                BLOCK_HEIGHT=$(echo "$RESPONSE" | grep -o '"block_height":[0-9]*' | cut -d':' -f2 | head -n1)
                echo -e "   📡 $node ($region): Block height ${BLOCK_HEIGHT:-'unknown'}"
            else
                echo -e "   ❌ $node ($region): Not responding"
            fi
        done
        
        # Get mining statistics
        for miner in miner-pool-na miner-apac; do
            port="${node_ports[$miner]}"
            region="${node_regions[$miner]}"
            if STATS=$(curl -s --connect-timeout 3 "http://localhost:$port/stats" 2>/dev/null); then
                echo -e "   ⛏️  $miner ($region): $STATS"
            fi
        done
    done
}

generate_realistic_transactions() {
    echo -e "${YELLOW}💸 Starting realistic transaction generation...${NC}"
    
    local tx_count=0
    local start_time=$(date +%s)
    
    # Define business hours for each region (UTC offsets)
    local na_business_start=14  # 9 AM EST (UTC-5)
    local na_business_end=22    # 5 PM EST
    local eu_business_start=8   # 9 AM CET (UTC+1)
    local eu_business_end=16    # 5 PM CET
    local apac_business_start=1 # 9 AM JST (UTC+9)
    local apac_business_end=9   # 5 PM JST
    
    while [[ $tx_count -lt $NUM_TRANSACTIONS ]]; do
        local current_time=$(date +%s)
        local elapsed=$((current_time - start_time))
        
        if [[ $elapsed -ge $SIMULATION_DURATION ]]; then
            break
        fi
        
        # Determine current UTC hour for business hours simulation
        local current_hour=$(date -u +%H)
        local activity_multiplier=1
        
        # Adjust transaction rate based on business hours
        if [[ $current_hour -ge $na_business_start && $current_hour -lt $na_business_end ]]; then
            activity_multiplier=3  # NA business hours
        elif [[ $current_hour -ge $eu_business_start && $current_hour -lt $eu_business_end ]]; then
            activity_multiplier=2  # EU business hours
        elif [[ $current_hour -ge $apac_business_start && $current_hour -lt $apac_business_end ]]; then
            activity_multiplier=2  # APAC business hours
        fi
        
        # Generate transactions based on realistic patterns
        generate_cross_border_payment $tx_count $activity_multiplier
        generate_defi_transaction $tx_count $activity_multiplier
        generate_microtransaction $tx_count $activity_multiplier
        
        tx_count=$((tx_count + 3))  # 3 transactions per cycle
        
        # Progress report
        if [[ $((tx_count % 15)) -eq 0 ]]; then
            echo -e "   📊 Progress: ${tx_count}/${NUM_TRANSACTIONS} transactions, ${elapsed}/${SIMULATION_DURATION}s elapsed"
        fi
        
        # Adjust sleep based on activity level
        local adjusted_interval=$((TX_INTERVAL / activity_multiplier))
        sleep $adjusted_interval
    done
    
    echo -e "${GREEN}✅ Realistic transaction generation completed: $tx_count transactions sent${NC}"
}

generate_cross_border_payment() {
    local tx_id=$1
    local multiplier=$2
    
    # Simulate cross-border payment from NA to EU
    local na_node="exchange-na"
    local eu_node="validator-institution-eu"
    local amount=$((1000 + RANDOM % 9000))  # $10-100 equivalent
    
    local tx_data="{\"type\":\"cross_border\",\"from\":\"$na_node\",\"to\":\"$eu_node\",\"amount\":$amount,\"nonce\":$tx_id,\"compliance_delay\":true}"
    
    submit_transaction_to_node "$na_node" 9002 "$tx_data" "Cross-border payment"
}

generate_defi_transaction() {
    local tx_id=$1
    local multiplier=$2
    
    # Simulate DeFi transaction in APAC region
    local from_node="mobile-backend-apac"
    local to_node="miner-apac"
    local amount=$((50 + RANDOM % 450))  # Smaller DeFi amounts
    
    local tx_data="{\"type\":\"defi\",\"from\":\"$from_node\",\"to\":\"$to_node\",\"amount\":$amount,\"nonce\":$((tx_id + 1)),\"gas_premium\":true}"
    
    submit_transaction_to_node "$from_node" 9021 "$tx_data" "DeFi transaction"
}

generate_microtransaction() {
    local tx_id=$1
    local multiplier=$2
    
    # Simulate microtransaction from mobile client
    local from_node="light-client-mobile"
    local to_node="bootstrap-na"
    local amount=$((1 + RANDOM % 50))  # Very small amounts
    
    local tx_data="{\"type\":\"micro\",\"from\":\"$from_node\",\"to\":\"$to_node\",\"amount\":$amount,\"nonce\":$((tx_id + 2)),\"low_priority\":true}"
    
    submit_transaction_to_node "$from_node" 9030 "$tx_data" "Microtransaction"
}

submit_transaction_to_node() {
    local node=$1
    local port=$2
    local tx_data=$3
    local tx_type=$4
    
    if curl -s -X POST \
        -H "Content-Type: application/json" \
        -d "$tx_data" \
        "http://localhost:$port/transaction" > /dev/null 2>&1; then
        echo -e "   💸 $tx_type: $node ($(echo "$tx_data" | jq -r '.amount' 2>/dev/null || echo 'unknown') satoshis)"
    fi
}

start_chaos_testing() {
    if [[ "$CHAOS_MODE" == "true" ]]; then
        echo -e "${PURPLE}🌪️  Starting chaos engineering tests...${NC}"
        
        # Simulate network partition after 10 minutes
        sleep 600 &&
        simulate_network_partition &
        
        # Simulate node failure after 15 minutes
        sleep 900 &&
        simulate_node_failure &
        
        # Simulate bandwidth degradation after 20 minutes
        sleep 1200 &&
        simulate_bandwidth_degradation &
        
        echo -e "${PURPLE}✅ Chaos testing scheduled${NC}"
    fi
}

simulate_network_partition() {
    echo -e "${PURPLE}🌪️  Simulating network partition: Isolating APAC region...${NC}"
    
    # Block traffic between APAC and other regions
    docker exec clab-polytorus-realistic-testnet-router-apac tc qdisc add dev eth2 root netem loss 100%
    docker exec clab-polytorus-realistic-testnet-router-apac tc qdisc add dev eth3 root netem loss 100%
    
    sleep 300  # 5 minutes partition
    
    echo -e "${PURPLE}🔄 Healing network partition...${NC}"
    
    # Restore connectivity gradually
    docker exec clab-polytorus-realistic-testnet-router-apac tc qdisc change dev eth2 root netem loss 50%
    docker exec clab-polytorus-realistic-testnet-router-apac tc qdisc change dev eth3 root netem loss 50%
    
    sleep 60
    
    docker exec clab-polytorus-realistic-testnet-router-apac tc qdisc del dev eth2 root
    docker exec clab-polytorus-realistic-testnet-router-apac tc qdisc del dev eth3 root
    
    echo -e "${GREEN}✅ Network partition healed${NC}"
}

simulate_node_failure() {
    echo -e "${PURPLE}🌪️  Simulating node failure: Taking down EU research node...${NC}"
    
    # Stop the research node
    docker stop clab-polytorus-realistic-testnet-research-eu
    
    sleep 300  # 5 minutes downtime
    
    echo -e "${PURPLE}🔄 Recovering failed node...${NC}"
    
    # Restart the node
    docker start clab-polytorus-realistic-testnet-research-eu
    
    echo -e "${GREEN}✅ Node recovery completed${NC}"
}

simulate_bandwidth_degradation() {
    echo -e "${PURPLE}🌪️  Simulating bandwidth degradation on satellite links...${NC}"
    
    # Reduce bandwidth on edge connections
    docker exec clab-polytorus-realistic-testnet-rural-satellite tc qdisc change dev eth1 root handle 1: netem delay 1000ms 200ms loss 5%
    
    sleep 600  # 10 minutes degradation
    
    echo -e "${PURPLE}🔄 Restoring normal bandwidth...${NC}"
    
    # Restore normal bandwidth
    docker exec clab-polytorus-realistic-testnet-rural-satellite tc qdisc change dev eth1 root handle 1: netem delay 600ms 100ms loss 2%
    
    echo -e "${GREEN}✅ Bandwidth restored${NC}"
}

show_final_statistics() {
    echo -e "\n${BLUE}📈 Final Global Network Statistics:${NC}"
    echo -e "======================================"
    
    # Show node statistics by region
    echo -e "\n${CYAN}🌍 Regional Node Status:${NC}"
    
    # North America
    echo -e "\n${YELLOW}🇺🇸 North America (AS65001):${NC}"
    for node in bootstrap-na miner-pool-na exchange-na; do
        show_node_stats "$node" "${node_ports[$node]}"
    done
    
    # Europe
    echo -e "\n${YELLOW}🇪🇺 Europe (AS65002):${NC}"
    for node in validator-institution-eu research-eu; do
        show_node_stats "$node" "${node_ports[$node]}"
    done
    
    # Asia-Pacific
    echo -e "\n${YELLOW}🌏 Asia-Pacific (AS65003):${NC}"
    for node in miner-apac mobile-backend-apac; do
        show_node_stats "$node" "${node_ports[$node]}"
    done
    
    # Edge/Mobile
    echo -e "\n${YELLOW}📱 Edge/Mobile (AS65004):${NC}"
    for node in light-client-mobile rural-satellite; do
        show_node_stats "$node" "${node_ports[$node]}"
    done
    
    # Network statistics
    echo -e "\n${CYAN}🌐 Network Performance Summary:${NC}"
    show_network_summary
    
    # BGP status
    if [[ "$ENABLE_BGP_MONITORING" == "true" ]]; then
        echo -e "\n${CYAN}🛣️  BGP Routing Status:${NC}"
        show_bgp_summary
    fi
    
    echo -e "\n${BLUE}📋 ContainerLab Container Status:${NC}"
    containerlab inspect --topo "$TOPOLOGY_FILE" || true
}

show_node_stats() {
    local node=$1
    local port=$2
    
    echo -e "   📡 $node:"
    
    if RESPONSE=$(curl -s --connect-timeout 5 "http://localhost:$port/status" 2>/dev/null); then
        echo -e "     Status: Online"
        echo -e "     Response: $RESPONSE"
    else
        echo -e "     Status: Offline or not responding"
    fi
}

show_network_summary() {
    echo -e "   🔍 Inter-region connectivity tests performed"
    echo -e "   📊 Bandwidth utilization monitored"
    echo -e "   🌪️  Chaos testing: ${CHAOS_MODE}"
    echo -e "   ⏱️  Total simulation time: $SIMULATION_DURATION seconds"
}

show_bgp_summary() {
    for router in router-na router-eu router-apac router-edge; do
        echo -e "   📡 $router:"
        if docker exec "clab-polytorus-realistic-testnet-$router" vtysh -c "show ip bgp summary" 2>/dev/null | tail -5; then
            echo -e "     BGP Status: Operational"
        else
            echo -e "     BGP Status: Issues detected"
        fi
    done
}

cleanup() {
    echo -e "\n${YELLOW}🧹 Cleaning up realistic testnet simulation...${NC}"
    
    # Stop all background processes
    for pid_file in /tmp/{bgp_monitor,perf_monitor,blockchain_monitor,tx_generator}.pid; do
        if [[ -f "$pid_file" ]]; then
            PID=$(cat "$pid_file")
            if kill -0 "$PID" 2>/dev/null; then
                kill "$PID" 2>/dev/null || true
            fi
            rm -f "$pid_file"
        fi
    done
    
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
    prepare_enhanced_environment
    generate_mining_wallets
    start_containerlab
    wait_for_nodes
    start_enhanced_monitoring
    
    if [[ "$CHAOS_MODE" == "true" ]]; then
        start_chaos_testing
    fi
    
    echo -e "\n${GREEN}🎯 Realistic testnet simulation running!${NC}"
    echo -e "${YELLOW}💡 Monitor nodes and network performance:${NC}"
    echo -e "   🇺🇸 NA Bootstrap: http://localhost:9000"
    echo -e "   🇺🇸 NA Mining Pool: http://localhost:9001"
    echo -e "   🇺🇸 NA Exchange: http://localhost:9002"
    echo -e "   🇪🇺 EU Validator: http://localhost:9010"
    echo -e "   🇪🇺 EU Research: http://localhost:9011"
    echo -e "   🌏 APAC Miner: http://localhost:9020"
    echo -e "   🌏 APAC Mobile: http://localhost:9021"
    echo -e "   📱 Light Client: http://localhost:9030"
    echo -e "   🛰️  Rural Satellite: http://localhost:9031"
    
    echo -e "\n${CYAN}Press Ctrl+C to stop the simulation...${NC}"
    
    # Wait for simulation duration
    sleep $SIMULATION_DURATION
    
    echo -e "\n${GREEN}🏁 Realistic testnet simulation completed!${NC}"
    show_final_statistics
}

# Check if running as source or executed
if [[ "${BASH_SOURCE[0]}" == "${0}" ]]; then
    main "$@"
fi