#!/bin/bash

# PolyTorus ContainerLab Test Environment Setup Script

set -e

echo "🚀 PolyTorus ContainerLab Test Environment Setup"
echo "================================================="

# Check if containerlab is installed
if ! command -v containerlab &> /dev/null; then
    echo "❌ containerlab is not installed. Please install it first:"
    echo "   sudo bash -c \"\$(curl -sL https://get.containerlab.dev)\""
    exit 1
fi

# Check if Docker is running
if ! docker info &> /dev/null; then
    echo "❌ Docker is not running. Please start Docker first."
    exit 1
fi

# Build Docker image
echo "🔨 Building PolyTorus Docker image..."
docker build -t polytorus:latest .

if [ $? -ne 0 ]; then
    echo "❌ Failed to build Docker image"
    exit 1
fi

echo "✅ Docker image built successfully"

# Deploy containerlab topology
echo "🌐 Deploying ContainerLab topology..."
sudo containerlab deploy -t containerlab.yml

if [ $? -ne 0 ]; then
    echo "❌ Failed to deploy containerlab topology"
    exit 1
fi

echo "✅ ContainerLab topology deployed successfully"

# Wait for nodes to initialize
echo "⏳ Waiting for nodes to initialize (30 seconds)..."
sleep 30

echo ""
echo "🎉 Setup complete! Your PolyTorus test network is ready."
echo ""
echo "📊 Network topology:"
echo "   - Genesis node:    localhost:17000 (P2P), localhost:18080 (Web)"
echo "   - Miner 1:         localhost:17001 (P2P), localhost:18081 (Web)"
echo "   - Miner 2:         localhost:17002 (P2P), localhost:18082 (Web)"
echo "   - Transaction node: localhost:17003 (P2P), localhost:18083 (Web)"
echo "   - Test client:     (for running commands)"
echo ""
echo "🧪 To run transaction tests:"
echo "   ./test_transactions.sh"
echo ""
echo "🔍 To monitor the network:"
echo "   sudo containerlab inspect -t containerlab.yml"
echo ""
echo "🛑 To stop the network:"
echo "   sudo containerlab destroy -t containerlab.yml"
