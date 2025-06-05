#!/bin/bash

# PolyTorus Network Monitoring Script
# Provides real-time monitoring of the containerlab network

set -e

echo "📊 PolyTorus Network Monitor"
echo "============================"

# Helper functions
exec_in_container() {
    local container=$1
    local cmd=$2
    docker exec clab-polytorus-network-$container bash -c "$cmd" 2>/dev/null || echo "N/A"
}

get_balance() {
    local container=$1
    local address=$2
    exec_in_container $container "polytorus getbalance $address 2>/dev/null | grep 'Balance:' | cut -d' ' -f2" || echo "0"
}

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Main monitoring loop
monitor_network() {
    local refresh_interval=${1:-5}
    
    # Get wallet addresses once
    echo "🔍 Discovering wallet addresses..."
    genesis_addr=$(exec_in_container genesis "polytorus listaddresses 2>/dev/null | tail -1")
    miner1_addr=$(exec_in_container miner1 "polytorus listaddresses 2>/dev/null | tail -1")
    miner2_addr=$(exec_in_container miner2 "polytorus listaddresses 2>/dev/null | tail -1")
    txnode_addr=$(exec_in_container txnode "polytorus listaddresses 2>/dev/null | tail -1")
    
    while true; do
        clear
        echo -e "${BLUE}📊 PolyTorus Network Monitor${NC}"
        echo -e "${BLUE}============================${NC}"
        echo "$(date)"
        echo "Refresh interval: ${refresh_interval}s (Press Ctrl+C to exit)"
        echo ""
        
        # Container status
        echo -e "${GREEN}🟢 Container Status:${NC}"
        for container in genesis miner1 miner2 txnode testclient; do
            if docker ps | grep -q "clab-polytorus-network-$container"; then
                status="${GREEN}RUNNING${NC}"
            else
                status="${RED}STOPPED${NC}"
            fi
            printf "  %-12s: %b\n" "$container" "$status"
        done
        echo ""
        
        # Network connectivity
        echo -e "${GREEN}🌐 Network Connectivity:${NC}"
        for container in genesis miner1 miner2 txnode; do
            # Test if container responds to commands
            response=$(exec_in_container $container "echo 'ok'" 2>/dev/null)
            if [ "$response" = "ok" ]; then
                status="${GREEN}CONNECTED${NC}"
            else
                status="${RED}DISCONNECTED${NC}"
            fi
            printf "  %-12s: %b\n" "$container" "$status"
        done
        echo ""
        
        # Wallet balances
        echo -e "${GREEN}💰 Wallet Balances:${NC}"
        printf "  %-12s: %s (Address: %s)\n" "Genesis" "$(get_balance genesis $genesis_addr)" "${genesis_addr:0:20}..."
        printf "  %-12s: %s (Address: %s)\n" "Miner1" "$(get_balance miner1 $miner1_addr)" "${miner1_addr:0:20}..."
        printf "  %-12s: %s (Address: %s)\n" "Miner2" "$(get_balance miner2 $miner2_addr)" "${miner2_addr:0:20}..."
        printf "  %-12s: %s (Address: %s)\n" "TxNode" "$(get_balance txnode $txnode_addr)" "${txnode_addr:0:20}..."
        echo ""
        
        # Blockchain height
        echo -e "${GREEN}⛓️  Blockchain Height:${NC}"
        for container in genesis miner1 miner2 txnode; do
            height=$(exec_in_container $container "polytorus printchain 2>/dev/null | grep -c 'Block {'" || echo "0")
            printf "  %-12s: %s blocks\n" "$container" "$height"
        done
        echo ""
        
        # Recent activity (check logs for recent activity)
        echo -e "${GREEN}📝 Recent Activity:${NC}"
        for container in genesis miner1 miner2; do
            recent_log=$(docker logs --tail 1 clab-polytorus-network-$container 2>/dev/null | head -1 || echo "No recent activity")
            if [ ${#recent_log} -gt 80 ]; then
                recent_log="${recent_log:0:77}..."
            fi
            printf "  %-12s: %s\n" "$container" "$recent_log"
        done
        echo ""
        
        # Port mapping
        echo -e "${GREEN}🔌 Port Mappings:${NC}"
        echo "  Genesis:     P2P=localhost:17000, Web=localhost:18080"
        echo "  Miner1:      P2P=localhost:17001, Web=localhost:18081"
        echo "  Miner2:      P2P=localhost:17002, Web=localhost:18082"
        echo "  TxNode:      P2P=localhost:17003, Web=localhost:18083"
        echo ""
        
        # Commands help
        echo -e "${YELLOW}💡 Quick Commands:${NC}"
        echo "  Test transactions:  ./test_transactions.sh"
        echo "  Advanced tests:     ./test_advanced.sh"
        echo "  Container logs:     docker logs clab-polytorus-network-<node>"
        echo "  Container shell:    docker exec -it clab-polytorus-network-<node> bash"
        echo "  Inspect topology:   sudo containerlab inspect -t containerlab.yml"
        
        sleep $refresh_interval
    done
}

# Interactive mode for specific monitoring
interactive_mode() {
    echo "🔧 Interactive Monitoring Mode"
    echo "Available commands:"
    echo "  1) Monitor specific node"
    echo "  2) Watch transaction flow"
    echo "  3) Monitor contract deployments"
    echo "  4) Network health check"
    echo "  5) Return to main monitor"
    echo ""
    read -p "Select option (1-5): " choice
    
    case $choice in
        1)
            echo "Available nodes: genesis, miner1, miner2, txnode, testclient"
            read -p "Enter node name: " node
            echo "Monitoring $node logs..."
            docker logs -f clab-polytorus-network-$node
            ;;
        2)
            echo "Watching transaction flow..."
            while true; do
                echo "=== Transaction Status $(date) ==="
                for container in genesis miner1 miner2 txnode; do
                    addr_var="${container}_addr"
                    if [ -n "${!addr_var}" ]; then
                        balance=$(get_balance $container ${!addr_var})
                        echo "$container: $balance"
                    fi
                done
                sleep 3
            done
            ;;
        3)
            echo "Monitoring contract deployments..."
            for container in genesis miner1 miner2 txnode; do
                echo "=== $container contracts ==="
                exec_in_container $container "polytorus listcontracts 2>/dev/null || echo 'No contracts'"
                echo ""
            done
            ;;
        4)
            echo "Network health check..."
            sudo containerlab inspect -t containerlab.yml
            ;;
        5)
            return
            ;;
        *)
            echo "Invalid option"
            ;;
    esac
}

# Main script
if [ "$1" = "--interactive" ] || [ "$1" = "-i" ]; then
    interactive_mode
elif [ "$1" = "--help" ] || [ "$1" = "-h" ]; then
    echo "Usage: $0 [options] [refresh_interval]"
    echo "Options:"
    echo "  -i, --interactive    Interactive monitoring mode"
    echo "  -h, --help          Show this help"
    echo "  refresh_interval    Monitor refresh interval in seconds (default: 5)"
    echo ""
    echo "Examples:"
    echo "  $0                  Start monitoring with 5s refresh"
    echo "  $0 2                Start monitoring with 2s refresh"
    echo "  $0 -i               Start in interactive mode"
else
    refresh_interval=${1:-5}
    monitor_network $refresh_interval
fi
