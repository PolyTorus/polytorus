#!/bin/bash

echo "🚀 Complete Transaction Propagation Test"
echo "========================================"

# Test 1: Node 0 -> Node 1
echo "Test 1: Node 0 -> Node 1"
echo "Step 1: Sending to Node 0 /send endpoint..."
curl -s -X POST -H "Content-Type: application/json" \
  -d '{"from":"wallet_node-0","to":"wallet_node-1","amount":100,"nonce":2001}' \
  "http://127.0.0.1:9000/send" | head -c 200
echo ""

echo "Step 2: Sending to Node 1 /transaction endpoint..."
curl -s -X POST -H "Content-Type: application/json" \
  -d '{"from":"wallet_node-0","to":"wallet_node-1","amount":100,"nonce":2001}' \
  "http://127.0.0.1:9001/transaction" | head -c 200
echo ""

# Test 2: Node 1 -> Node 2
echo "Test 2: Node 1 -> Node 2"
echo "Step 1: Sending to Node 1 /send endpoint..."
curl -s -X POST -H "Content-Type: application/json" \
  -d '{"from":"wallet_node-1","to":"wallet_node-2","amount":200,"nonce":2002}' \
  "http://127.0.0.1:9001/send" | head -c 200
echo ""

echo "Step 2: Sending to Node 2 /transaction endpoint..."
curl -s -X POST -H "Content-Type: application/json" \
  -d '{"from":"wallet_node-1","to":"wallet_node-2","amount":200,"nonce":2002}' \
  "http://127.0.0.1:9002/transaction" | head -c 200
echo ""

# Test 3: Node 2 -> Node 3
echo "Test 3: Node 2 -> Node 3"
echo "Step 1: Sending to Node 2 /send endpoint..."
curl -s -X POST -H "Content-Type: application/json" \
  -d '{"from":"wallet_node-2","to":"wallet_node-3","amount":300,"nonce":2003}' \
  "http://127.0.0.1:9002/send" | head -c 200
echo ""

echo "Step 2: Sending to Node 3 /transaction endpoint..."
curl -s -X POST -H "Content-Type: application/json" \
  -d '{"from":"wallet_node-2","to":"wallet_node-3","amount":300,"nonce":2003}' \
  "http://127.0.0.1:9003/transaction" | head -c 200
echo ""

# Test 4: Node 3 -> Node 0
echo "Test 4: Node 3 -> Node 0"
echo "Step 1: Sending to Node 3 /send endpoint..."
curl -s -X POST -H "Content-Type: application/json" \
  -d '{"from":"wallet_node-3","to":"wallet_node-0","amount":400,"nonce":2004}' \
  "http://127.0.0.1:9003/send" | head -c 200
echo ""

echo "Step 2: Sending to Node 0 /transaction endpoint..."
curl -s -X POST -H "Content-Type: application/json" \
  -d '{"from":"wallet_node-3","to":"wallet_node-0","amount":400,"nonce":2004}' \
  "http://127.0.0.1:9000/transaction" | head -c 200
echo ""

echo "✅ Complete propagation tests completed!"
echo ""
echo "📊 Checking final statistics..."
for port in 9000 9001 9002 9003; do
  node_num=$((port - 9000))
  echo "Node $node_num (port $port):"
  timeout 3 curl -s "http://127.0.0.1:$port/stats" 2>/dev/null | head -c 200 || echo "  Stats unavailable"
  echo ""
done
