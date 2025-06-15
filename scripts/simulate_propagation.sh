#!/bin/bash
#
# Complete Transaction Propagation Simulator for PolyTorus
# This script simulates complete transaction propagation by calling both
# sender's /send endpoint and receiver's /transaction endpoint

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Configuration
NUM_NODES=4
SIMULATION_TIME=60  # 60 seconds for testing
BASE_PORT=9000
TX_INTERVAL=3       # 3 seconds between transactions

echo -e "${BLUE}╔══════════════════════════════════════════════╗${NC}"
echo -e "${BLUE}║        PolyTorus Complete Propagation        ║${NC}"
echo -e "${BLUE}║              Transaction Testing             ║${NC}"
echo -e "${BLUE}╚══════════════════════════════════════════════╝${NC}"
echo ""

# Start nodes first using the existing simulate.sh script
echo -e "${GREEN}🚀 Starting nodes with existing script...${NC}"
./scripts/simulate.sh local &
SIMULATE_PID=$!

# Wait for nodes to be ready
echo -e "${YELLOW}⏳ Waiting for nodes to start up (15s)...${NC}"
sleep 15

# Check if nodes are responding
echo -e "${BLUE}📊 Checking node readiness...${NC}"
all_ready=true
for ((i=0; i<NUM_NODES; i++)); do
    PORT=$((BASE_PORT + i))
    if curl -s "http://127.0.0.1:$PORT/health" > /dev/null 2>&1; then
        echo -e "   ✅ Node $i (port $PORT) is ready"
    else
        echo -e "   ❌ Node $i (port $PORT) is not responding"
        all_ready=false
    fi
done

if [ "$all_ready" = false ]; then
    echo -e "${RED}❌ Not all nodes are ready. Exiting...${NC}"
    kill $SIMULATE_PID 2>/dev/null
    exit 1
fi

echo ""
echo -e "${GREEN}💸 Starting Complete Transaction Propagation Simulation${NC}"
echo -e "   Duration: ${SIMULATION_TIME}s"
echo -e "   Transaction interval: ${TX_INTERVAL}s"
echo -e "   Propagation: Sender -> Receiver"
echo ""

# Transaction simulation loop
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
    TO_PORT=$((BASE_PORT + TO_NODE))
    
    # Transaction data
    TRANSACTION_DATA="{\"from\":\"wallet_node-$FROM_NODE\",\"to\":\"wallet_node-$TO_NODE\",\"amount\":$AMOUNT,\"nonce\":$TRANSACTION_COUNT}"
    
    # Step 1: Submit to sender node's /send endpoint (records as sent)
    SEND_SUCCESS=false
    if curl -s -X POST \
        -H "Content-Type: application/json" \
        -d "$TRANSACTION_DATA" \
        "http://127.0.0.1:$FROM_PORT/send" > /dev/null 2>&1; then
        SEND_SUCCESS=true
    fi
    
    # Step 2: Submit to receiver node's /transaction endpoint (records as received)
    RECV_SUCCESS=false
    if curl -s -X POST \
        -H "Content-Type: application/json" \
        -d "$TRANSACTION_DATA" \
        "http://127.0.0.1:$TO_PORT/transaction" > /dev/null 2>&1; then
        RECV_SUCCESS=true
    fi
    
    # Report transaction status
    if [[ "$SEND_SUCCESS" == true && "$RECV_SUCCESS" == true ]]; then
        echo -e "   💸 TX $TRANSACTION_COUNT: Node $FROM_NODE ➜ Node $TO_NODE (${AMOUNT}) ✅"
    elif [[ "$SEND_SUCCESS" == true ]]; then
        echo -e "   ⚠️  TX $TRANSACTION_COUNT: Node $FROM_NODE ➜ Node $TO_NODE (${AMOUNT}) - Send ✅, Recv ❌"
    elif [[ "$RECV_SUCCESS" == true ]]; then
        echo -e "   ⚠️  TX $TRANSACTION_COUNT: Node $FROM_NODE ➜ Node $TO_NODE (${AMOUNT}) - Send ❌, Recv ✅"
    else
        echo -e "   ❌ TX $TRANSACTION_COUNT: Node $FROM_NODE ➜ Node $TO_NODE (${AMOUNT}) - Both failed"
    fi
    
    TRANSACTION_COUNT=$((TRANSACTION_COUNT + 1))
    
    # Progress report every 5 transactions
    if [[ $((TRANSACTION_COUNT % 5)) -eq 0 ]]; then
        echo -e "   📊 Progress: ${TRANSACTION_COUNT} transactions, ${ELAPSED}/${SIMULATION_TIME}s elapsed"
    fi
    
    sleep $TX_INTERVAL
done

echo ""
echo -e "${GREEN}🎯 Complete Propagation Simulation completed!${NC}"
echo -e "   Total transactions: ${TRANSACTION_COUNT}"
echo -e "   Duration: ${SIMULATION_TIME} seconds"

# Final statistics
echo ""
echo -e "${BLUE}📈 Final Complete Propagation Statistics:${NC}"
for ((i=0; i<NUM_NODES; i++)); do
    PORT=$((BASE_PORT + i))
    echo -e "   Node $i (port $PORT):"
    
    # Get detailed stats
    STATS=$(curl -s "http://127.0.0.1:$PORT/stats" 2>/dev/null)
    if [[ $? -eq 0 && -n "$STATS" ]]; then
        TX_SENT=$(echo "$STATS" | grep -o '"transactions_sent":[0-9]*' | cut -d: -f2)
        TX_RECV=$(echo "$STATS" | grep -o '"transactions_received":[0-9]*' | cut -d: -f2)
        echo -e "     📤 Sent: ${TX_SENT:-0}, 📨 Received: ${TX_RECV:-0}"
    else
        echo -e "     Status: Running (stats unavailable)"
    fi
done

echo ""
echo -e "${YELLOW}💡 Complete propagation simulation completed!${NC}"
echo -e "${YELLOW}💡 Both TX Sent and TX Recv should now show non-zero values${NC}"
echo ""
echo -e "${BLUE}🔄 Nodes still running. Press Ctrl+C to stop the main simulation.${NC}"

# Keep monitoring until interrupted
while true; do
    sleep 5
    # Check if main simulation is still running
    if ! kill -0 $SIMULATE_PID 2>/dev/null; then
        echo -e "${YELLOW}Main simulation stopped. Exiting monitoring.${NC}"
        break
    fi
done
