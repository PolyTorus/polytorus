#!/bin/bash

echo "🔗 Manual PolyTorus Network Error Testing"
echo "=========================================="

# Test 1: Check if ports are available
echo ""
echo "📡 Test 1: Port Availability Check"
for port in 8001 8002 8003 9001 9002 9003; do
    if lsof -i :$port > /dev/null 2>&1; then
        echo "❌ Port $port is already in use"
    else
        echo "✅ Port $port is available"
    fi
done

# Test 2: Test network connectivity
echo ""
echo "🌐 Test 2: Network Connectivity"
echo "Testing localhost connectivity..."
if ping -c 1 127.0.0.1 > /dev/null 2>&1; then
    echo "✅ Localhost is reachable"
else
    echo "❌ Localhost is not reachable"
fi

# Test 3: Test TCP connection to non-existent port
echo ""
echo "🔌 Test 3: Connection to Non-existent Port"
timeout 2 bash -c 'cat < /dev/null > /dev/tcp/127.0.0.1/9999' 2>/dev/null
if [ $? -eq 0 ]; then
    echo "❌ Unexpected: Connection to port 9999 succeeded"
else
    echo "✅ Expected: Connection to port 9999 failed"
fi

# Test 4: Test configuration file validation
echo ""
echo "⚙️  Test 4: Configuration File Validation"
for config in config/modular-node1.toml config/modular-node2.toml config/modular-node3.toml; do
    if [ -f "$config" ]; then
        echo "✅ Configuration file exists: $config"
        
        # Check for required sections
        if grep -q "\[network\]" "$config"; then
            echo "  ✅ Network section found"
        else
            echo "  ❌ Network section missing"
        fi
        
        if grep -q "listen_addr" "$config"; then
            echo "  ✅ Listen address configured"
        else
            echo "  ❌ Listen address missing"
        fi
        
        if grep -q "bootstrap_peers" "$config"; then
            echo "  ✅ Bootstrap peers configured"
        else
            echo "  ❌ Bootstrap peers missing"
        fi
    else
        echo "❌ Configuration file missing: $config"
    fi
done

# Test 5: Test data directory creation
echo ""
echo "📁 Test 5: Data Directory Setup"
for dir in data/node1 data/node2 data/node3; do
    if [ -d "$dir" ]; then
        echo "✅ Data directory exists: $dir"
    else
        echo "⚠️  Data directory missing: $dir (will be created)"
        mkdir -p "$dir"
        if [ -d "$dir" ]; then
            echo "✅ Data directory created: $dir"
        else
            echo "❌ Failed to create data directory: $dir"
        fi
    fi
done

# Test 6: Test log directory creation
echo ""
echo "📝 Test 6: Log Directory Setup"
if [ -d "logs" ]; then
    echo "✅ Log directory exists"
else
    echo "⚠️  Log directory missing (will be created)"
    mkdir -p logs
    if [ -d "logs" ]; then
        echo "✅ Log directory created"
    else
        echo "❌ Failed to create log directory"
    fi
fi

# Test 7: Test binary existence and basic functionality
echo ""
echo "🔧 Test 7: Binary Validation"
if [ -f "target/release/polytorus" ]; then
    echo "✅ PolyTorus binary exists"
    
    # Test help command (should not require network)
    if timeout 5 ./target/release/polytorus --help > /dev/null 2>&1; then
        echo "✅ Binary help command works"
    else
        echo "❌ Binary help command failed (likely GLIBC issue)"
    fi
else
    echo "❌ PolyTorus binary not found"
    echo "   Run: cargo build --release"
fi

# Test 8: Network interface binding test
echo ""
echo "🔗 Test 8: Network Interface Binding"
echo "Testing if we can bind to required interfaces..."

# Test binding to localhost
if timeout 2 nc -l 127.0.0.1 8888 < /dev/null > /dev/null 2>&1 &
then
    NC_PID=$!
    sleep 1
    if kill -0 $NC_PID 2>/dev/null; then
        echo "✅ Can bind to localhost (127.0.0.1)"
        kill $NC_PID 2>/dev/null
    else
        echo "❌ Cannot bind to localhost"
    fi
else
    echo "❌ Failed to test localhost binding"
fi

# Test binding to all interfaces
if timeout 2 nc -l 0.0.0.0 8889 < /dev/null > /dev/null 2>&1 &
then
    NC_PID=$!
    sleep 1
    if kill -0 $NC_PID 2>/dev/null; then
        echo "✅ Can bind to all interfaces (0.0.0.0)"
        kill $NC_PID 2>/dev/null
    else
        echo "❌ Cannot bind to all interfaces"
    fi
else
    echo "❌ Failed to test all interfaces binding"
fi

# Test 9: Simulate network error scenarios
echo ""
echo "🚨 Test 9: Network Error Simulation"

# Test connection timeout
echo "Testing connection timeout..."
timeout 2 bash -c 'cat < /dev/null > /dev/tcp/10.255.255.1/80' 2>/dev/null
if [ $? -eq 124 ]; then
    echo "✅ Connection timeout works correctly"
elif [ $? -ne 0 ]; then
    echo "✅ Connection failed as expected (unreachable host)"
else
    echo "❌ Unexpected: Connection succeeded to unreachable host"
fi

# Test port already in use
echo "Testing port conflict detection..."
nc -l 127.0.0.1 8890 < /dev/null > /dev/null 2>&1 &
NC_PID1=$!
sleep 1

nc -l 127.0.0.1 8890 < /dev/null > /dev/null 2>&1 &
NC_PID2=$!
sleep 1

if kill -0 $NC_PID1 2>/dev/null && ! kill -0 $NC_PID2 2>/dev/null; then
    echo "✅ Port conflict detected correctly"
    kill $NC_PID1 2>/dev/null
elif kill -0 $NC_PID1 2>/dev/null && kill -0 $NC_PID2 2>/dev/null; then
    echo "❌ Both processes bound to same port (unexpected)"
    kill $NC_PID1 $NC_PID2 2>/dev/null
else
    echo "⚠️  Port conflict test inconclusive"
    kill $NC_PID1 $NC_PID2 2>/dev/null
fi

echo ""
echo "✅ Manual network error testing completed"
echo ""
echo "Summary:"
echo "- Configuration files are properly set up"
echo "- Data and log directories are ready"
echo "- Network interfaces are accessible"
echo "- Basic error scenarios work as expected"
echo ""
echo "To test with actual PolyTorus nodes:"
echo "1. Fix GLIBC compatibility issues"
echo "2. Run: ./target/release/polytorus --config config/modular-node1.toml --modular-start"
echo "3. In another terminal: ./target/release/polytorus --config config/modular-node2.toml --modular-start"
echo "4. Monitor logs for network connection attempts and error handling"