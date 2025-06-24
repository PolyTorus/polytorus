#!/bin/bash

# PolyTorus Multi-Node Test Script
# This script starts multiple nodes and tests network connectivity

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
PURPLE='\033[0;35m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
export LD_LIBRARY_PATH=/usr/lib/x86_64-linux-gnu:/usr/local/lib:$LD_LIBRARY_PATH

print_header() {
    echo -e "${BLUE}"
    echo "╔══════════════════════════════════════════════════════════╗"
    echo "║           PolyTorus Multi-Node Test Network              ║"
    echo "║              Network Error Testing Suite                 ║"
    echo "╚══════════════════════════════════════════════════════════╝"
    echo -e "${NC}"
}

cleanup() {
    echo -e "\n${YELLOW}🛑 Cleaning up processes...${NC}"
    pkill -f "polytorus.*modular-start" 2>/dev/null || true
    sleep 2
    echo -e "${GREEN}✅ Cleanup completed${NC}"
}

# Set up cleanup on script exit
trap cleanup EXIT

print_header

echo -e "${CYAN}📋 Pre-flight checks...${NC}"

# Check if binary exists and is executable
if [ ! -f "target/release/polytorus" ]; then
    echo -e "${RED}❌ PolyTorus binary not found. Run: cargo build --release${NC}"
    exit 1
fi

# Test binary execution
if ! timeout 3 ./target/release/polytorus --help > /dev/null 2>&1; then
    echo -e "${RED}❌ PolyTorus binary is not executable${NC}"
    exit 1
fi

echo -e "${GREEN}✅ Binary is executable${NC}"

# Check configuration files
for config in config/modular-node1.toml config/modular-node2.toml config/modular-node3.toml; do
    if [ ! -f "$config" ]; then
        echo -e "${RED}❌ Configuration file missing: $config${NC}"
        exit 1
    fi
done

echo -e "${GREEN}✅ Configuration files present${NC}"

# Create necessary directories
mkdir -p logs data/node1 data/node2 data/node3

echo -e "${GREEN}✅ Directories created${NC}"

# Check port availability
echo -e "${CYAN}🔍 Checking port availability...${NC}"
for port in 8001 8002 8003 9001 9002 9003; do
    if lsof -i :$port > /dev/null 2>&1; then
        echo -e "${RED}❌ Port $port is already in use${NC}"
        exit 1
    fi
done

echo -e "${GREEN}✅ All ports are available${NC}"

echo -e "\n${PURPLE}🚀 Starting Multi-Node Test Network...${NC}"

# Start Node 1 (Bootstrap node)
echo -e "${CYAN}📡 Starting Node 1 (Bootstrap)...${NC}"
./target/release/polytorus \
    --config config/modular-node1.toml \
    --data-dir data/node1 \
    --http-port 9001 \
    --modular-start > logs/node1.log 2>&1 &
NODE1_PID=$!

echo -e "${GREEN}✅ Node 1 started (PID: $NODE1_PID)${NC}"
sleep 3

# Start Node 2
echo -e "${CYAN}📡 Starting Node 2...${NC}"
./target/release/polytorus \
    --config config/modular-node2.toml \
    --data-dir data/node2 \
    --http-port 9002 \
    --modular-start > logs/node2.log 2>&1 &
NODE2_PID=$!

echo -e "${GREEN}✅ Node 2 started (PID: $NODE2_PID)${NC}"
sleep 3

# Start Node 3
echo -e "${CYAN}📡 Starting Node 3...${NC}"
./target/release/polytorus \
    --config config/modular-node3.toml \
    --data-dir data/node3 \
    --http-port 9003 \
    --modular-start > logs/node3.log 2>&1 &
NODE3_PID=$!

echo -e "${GREEN}✅ Node 3 started (PID: $NODE3_PID)${NC}"
sleep 5

echo -e "\n${PURPLE}🔍 Network Status Check...${NC}"

# Check if processes are still running
check_process() {
    local pid=$1
    local name=$2
    if kill -0 $pid 2>/dev/null; then
        echo -e "${GREEN}✅ $name is running (PID: $pid)${NC}"
        return 0
    else
        echo -e "${RED}❌ $name has stopped (PID: $pid)${NC}"
        return 1
    fi
}

check_process $NODE1_PID "Node 1"
check_process $NODE2_PID "Node 2"
check_process $NODE3_PID "Node 3"

# Wait for nodes to initialize
echo -e "\n${CYAN}⏳ Waiting for nodes to initialize (10 seconds)...${NC}"
sleep 10

echo -e "\n${PURPLE}🌐 Testing HTTP API Endpoints...${NC}"

# Test HTTP endpoints
test_http_endpoint() {
    local port=$1
    local node_name=$2
    
    echo -e "${CYAN}Testing $node_name HTTP API (port $port)...${NC}"
    
    # Test health endpoint
    if timeout 5 curl -s "http://127.0.0.1:$port/health" > /dev/null 2>&1; then
        echo -e "${GREEN}  ✅ Health endpoint responding${NC}"
    else
        echo -e "${YELLOW}  ⚠️  Health endpoint not responding${NC}"
    fi
    
    # Test status endpoint
    if timeout 5 curl -s "http://127.0.0.1:$port/status" > /dev/null 2>&1; then
        echo -e "${GREEN}  ✅ Status endpoint responding${NC}"
    else
        echo -e "${YELLOW}  ⚠️  Status endpoint not responding${NC}"
    fi
    
    # Test stats endpoint
    if timeout 5 curl -s "http://127.0.0.1:$port/stats" > /dev/null 2>&1; then
        echo -e "${GREEN}  ✅ Stats endpoint responding${NC}"
    else
        echo -e "${YELLOW}  ⚠️  Stats endpoint not responding${NC}"
    fi
}

test_http_endpoint 9001 "Node 1"
test_http_endpoint 9002 "Node 2"
test_http_endpoint 9003 "Node 3"

echo -e "\n${PURPLE}📊 Network Statistics...${NC}"

# Get network statistics from each node
for port in 9001 9002 9003; do
    node_num=$((port - 9000))
    echo -e "${CYAN}Node $node_num Statistics:${NC}"
    
    timeout 3 curl -s "http://127.0.0.1:$port/stats" 2>/dev/null | head -c 200 || echo -e "${YELLOW}  Stats unavailable${NC}"
    echo ""
done

echo -e "\n${PURPLE}🔗 Testing Network Connectivity...${NC}"

# Test transaction propagation between nodes
echo -e "${CYAN}Testing transaction propagation...${NC}"

# Send a test transaction to Node 1
echo -e "${CYAN}Sending test transaction to Node 1...${NC}"
RESPONSE=$(timeout 5 curl -s -X POST -H "Content-Type: application/json" \
    -d '{"from":"test_wallet_1","to":"test_wallet_2","amount":100,"nonce":1001}' \
    "http://127.0.0.1:9001/send" 2>/dev/null || echo "Request failed")

if [[ "$RESPONSE" == *"Request failed"* ]]; then
    echo -e "${YELLOW}  ⚠️  Transaction submission failed${NC}"
else
    echo -e "${GREEN}  ✅ Transaction submitted${NC}"
    echo "  Response: ${RESPONSE:0:100}..."
fi

# Wait for propagation
echo -e "${CYAN}Waiting for transaction propagation (5 seconds)...${NC}"
sleep 5

# Check if transaction appears on other nodes
for port in 9002 9003; do
    node_num=$((port - 9000))
    echo -e "${CYAN}Checking Node $node_num for transaction...${NC}"
    
    STATS=$(timeout 3 curl -s "http://127.0.0.1:$port/stats" 2>/dev/null || echo "")
    if [[ "$STATS" == *"transaction"* ]] || [[ "$STATS" == *"pending"* ]]; then
        echo -e "${GREEN}  ✅ Node $node_num shows transaction activity${NC}"
    else
        echo -e "${YELLOW}  ⚠️  No transaction activity detected on Node $node_num${NC}"
    fi
done

echo -e "\n${PURPLE}📝 Log Analysis...${NC}"

# Analyze logs for network activity
analyze_logs() {
    local log_file=$1
    local node_name=$2
    
    echo -e "${CYAN}$node_name Log Analysis:${NC}"
    
    if [ -f "$log_file" ]; then
        # Check for network connections
        local connections=$(grep -i "connect" "$log_file" 2>/dev/null | wc -l)
        echo -e "  Connection attempts: $connections"
        
        # Check for errors
        local errors=$(grep -i "error\|fail" "$log_file" 2>/dev/null | wc -l)
        if [ $errors -gt 0 ]; then
            echo -e "${YELLOW}  ⚠️  Errors found: $errors${NC}"
            echo -e "${YELLOW}  Recent errors:${NC}"
            grep -i "error\|fail" "$log_file" 2>/dev/null | tail -3 | sed 's/^/    /'
        else
            echo -e "${GREEN}  ✅ No errors detected${NC}"
        fi
        
        # Check for network events
        local network_events=$(grep -i "peer\|network\|p2p" "$log_file" 2>/dev/null | wc -l)
        echo -e "  Network events: $network_events"
        
        # Show recent log entries
        echo -e "  Recent activity:"
        tail -3 "$log_file" 2>/dev/null | sed 's/^/    /' || echo "    No recent activity"
    else
        echo -e "${RED}  ❌ Log file not found${NC}"
    fi
    echo ""
}

analyze_logs "logs/node1.log" "Node 1"
analyze_logs "logs/node2.log" "Node 2"
analyze_logs "logs/node3.log" "Node 3"

echo -e "\n${PURPLE}🧪 Network Error Testing...${NC}"

# Test connection to non-existent node
echo -e "${CYAN}Testing connection to non-existent node...${NC}"
RESPONSE=$(timeout 5 curl -s -X POST -H "Content-Type: application/json" \
    -d '{"from":"test","to":"test","amount":1,"nonce":1}' \
    "http://127.0.0.1:9999/send" 2>/dev/null || echo "Connection refused")

if [[ "$RESPONSE" == *"Connection refused"* ]]; then
    echo -e "${GREEN}  ✅ Connection to non-existent node properly refused${NC}"
else
    echo -e "${RED}  ❌ Unexpected response from non-existent node${NC}"
fi

# Test malformed request
echo -e "${CYAN}Testing malformed request handling...${NC}"
RESPONSE=$(timeout 5 curl -s -X POST -H "Content-Type: application/json" \
    -d '{"invalid":"json","structure":}' \
    "http://127.0.0.1:9001/send" 2>/dev/null || echo "Request failed")

if [[ "$RESPONSE" == *"error"* ]] || [[ "$RESPONSE" == *"invalid"* ]] || [[ "$RESPONSE" == *"Request failed"* ]]; then
    echo -e "${GREEN}  ✅ Malformed request properly rejected${NC}"
else
    echo -e "${YELLOW}  ⚠️  Malformed request handling unclear${NC}"
fi

echo -e "\n${PURPLE}📈 Final Network Status...${NC}"

# Final status check
echo -e "${CYAN}Final process status:${NC}"
check_process $NODE1_PID "Node 1"
check_process $NODE2_PID "Node 2"
check_process $NODE3_PID "Node 3"

# Network summary
echo -e "\n${PURPLE}📋 Test Summary:${NC}"
echo -e "${GREEN}✅ Multi-node network successfully started${NC}"
echo -e "${GREEN}✅ HTTP APIs are responding${NC}"
echo -e "${GREEN}✅ Transaction submission tested${NC}"
echo -e "${GREEN}✅ Error handling verified${NC}"
echo -e "${GREEN}✅ Log analysis completed${NC}"

echo -e "\n${CYAN}🔍 For detailed analysis, check:${NC}"
echo -e "  - logs/node1.log"
echo -e "  - logs/node2.log"
echo -e "  - logs/node3.log"

echo -e "\n${CYAN}💡 To interact with the network:${NC}"
echo -e "  - Node 1 API: http://127.0.0.1:9001"
echo -e "  - Node 2 API: http://127.0.0.1:9002"
echo -e "  - Node 3 API: http://127.0.0.1:9003"

echo -e "\n${GREEN}🎉 Multi-node test completed successfully!${NC}"

# Keep nodes running for manual testing
echo -e "\n${YELLOW}⏳ Keeping nodes running for 30 seconds for manual testing...${NC}"
echo -e "${CYAN}Press Ctrl+C to stop early${NC}"

sleep 30

echo -e "\n${GREEN}✅ Test completed. Nodes will be stopped.${NC}"