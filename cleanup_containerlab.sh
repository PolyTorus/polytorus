#!/bin/bash

# PolyTorus ContainerLab Cleanup Script
# Safely destroys the test environment and cleans up resources

set -e

echo "🧹 PolyTorus ContainerLab Cleanup"
echo "================================="

# Function to ask for confirmation
confirm() {
    read -p "Are you sure you want to $1? (y/N): " -n 1 -r
    echo
    if [[ ! $REPLY =~ ^[Yy]$ ]]; then
        echo "Operation cancelled."
        exit 1
    fi
}

# Check if topology exists
if [ ! -f "containerlab.yml" ]; then
    echo "❌ containerlab.yml not found in current directory"
    exit 1
fi

# Stop ContainerLab topology
echo "🛑 Stopping ContainerLab topology..."
if sudo containerlab destroy -t containerlab.yml; then
    echo "✅ ContainerLab topology destroyed"
else
    echo "⚠️  Failed to destroy topology (may already be stopped)"
fi

# Ask about Docker cleanup
echo ""
echo "🐳 Docker Cleanup Options:"
echo "1. Remove PolyTorus Docker image only"
echo "2. Remove containers and image"
echo "3. Full cleanup (containers, image, and volumes)"
echo "4. Skip Docker cleanup"
echo ""
read -p "Select cleanup level (1-4): " cleanup_level

case $cleanup_level in
    1)
        echo "Removing PolyTorus Docker image..."
        docker rmi polytorus:latest || echo "Image already removed or doesn't exist"
        ;;
    2)
        echo "Removing containers and image..."
        # Remove any remaining containers
        docker ps -a | grep "clab-polytorus-network" | awk '{print $1}' | xargs -r docker rm -f
        docker rmi polytorus:latest || echo "Image already removed or doesn't exist"
        ;;
    3)
        confirm "perform full Docker cleanup (containers, images, volumes)"
        echo "Performing full cleanup..."
        # Remove containers
        docker ps -a | grep "clab-polytorus-network" | awk '{print $1}' | xargs -r docker rm -f
        # Remove image
        docker rmi polytorus:latest || echo "Image already removed or doesn't exist"
        # Remove related volumes
        docker volume ls | grep "clab-polytorus" | awk '{print $2}' | xargs -r docker volume rm
        # Clean up unused Docker resources
        docker system prune -f
        ;;
    4)
        echo "Skipping Docker cleanup"
        ;;
    *)
        echo "Invalid option, skipping Docker cleanup"
        ;;
esac

# Ask about file cleanup
echo ""
echo "📁 File Cleanup Options:"
echo "1. Keep all test files"
echo "2. Remove temporary test files only"
echo "3. Remove all generated files"
echo ""
read -p "Select file cleanup level (1-3): " file_cleanup

case $file_cleanup in
    1)
        echo "Keeping all files"
        ;;
    2)
        echo "Removing temporary files..."
        rm -f /tmp/simple_contract.wat /tmp/simple_contract.wasm
        rm -f /tmp/counter_contract.wat /tmp/counter_contract.wasm
        rm -f /tmp/token_contract.wat /tmp/token_contract.wasm
        rm -f /tmp/genesis_wallet.txt /tmp/miner*_wallet.txt /tmp/txnode_wallet.txt
        echo "✅ Temporary files removed"
        ;;
    3)
        confirm "remove all generated files (contracts, logs, etc.)"
        echo "Removing all generated files..."
        rm -f /tmp/simple_contract.* /tmp/counter_contract.* /tmp/token_contract.*
        rm -f /tmp/*_wallet.txt
        rm -rf /tmp/polytorus-*
        # Remove any local blockchain data directories
        find . -name "data" -type d -exec rm -rf {} + 2>/dev/null || true
        echo "✅ All generated files removed"
        ;;
    *)
        echo "Invalid option, keeping all files"
        ;;
esac

# Network cleanup
echo ""
echo "🌐 Checking for leftover networks..."
leftover_networks=$(docker network ls | grep "clab" | awk '{print $1}' || true)
if [ ! -z "$leftover_networks" ]; then
    echo "Found ContainerLab networks: $leftover_networks"
    read -p "Remove leftover ContainerLab networks? (y/N): " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        echo "$leftover_networks" | xargs -r docker network rm
        echo "✅ Leftover networks removed"
    fi
else
    echo "✅ No leftover networks found"
fi

# Final status check
echo ""
echo "🔍 Final Status Check:"
echo "----------------------"

# Check for remaining containers
remaining_containers=$(docker ps -a | grep "clab-polytorus" | wc -l)
if [ "$remaining_containers" -eq 0 ]; then
    echo "✅ No remaining PolyTorus containers"
else
    echo "⚠️  $remaining_containers PolyTorus containers still exist"
fi

# Check for Docker image
if docker images | grep -q "polytorus"; then
    echo "ℹ️  PolyTorus Docker image still exists"
else
    echo "✅ PolyTorus Docker image removed"
fi

# Check for running processes
if pgrep -f "polytorus" > /dev/null; then
    echo "⚠️  PolyTorus processes still running"
    echo "Running processes:"
    pgrep -f "polytorus" | xargs ps -p
else
    echo "✅ No PolyTorus processes running"
fi

echo ""
echo "🎉 Cleanup completed!"
echo ""
echo "💡 To start fresh:"
echo "   ./setup_containerlab.sh"
echo ""
echo "📋 What was cleaned:"
case $cleanup_level in
    1) echo "   - ContainerLab topology" ;;
    2) echo "   - ContainerLab topology and containers" ;;
    3) echo "   - ContainerLab topology, containers, and Docker resources" ;;
    4) echo "   - ContainerLab topology only" ;;
esac

case $file_cleanup in
    2) echo "   - Temporary test files" ;;
    3) echo "   - All generated files" ;;
esac
