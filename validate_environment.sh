#!/bin/bash

# PolyTorus ContainerLab Validation Script
# Validates the setup before running the actual tests

set -e

echo "✅ PolyTorus ContainerLab Environment Validation"
echo "==============================================="

# Colors for output
GREEN='\033[0;32m'
RED='\033[0;31m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Track validation results
validation_passed=true

# Function to check requirements
check_requirement() {
    local name=$1
    local command=$2
    local install_hint=$3
    
    echo -n "Checking $name... "
    if eval "$command" &>/dev/null; then
        echo -e "${GREEN}✓${NC}"
        return 0
    else
        echo -e "${RED}✗${NC}"
        echo -e "  ${YELLOW}Install hint: $install_hint${NC}"
        validation_passed=false
        return 1
    fi
}

echo "🔍 Checking system requirements..."
echo "----------------------------------"

# Check essential tools
check_requirement "Docker" "docker --version" "curl -fsSL https://get.docker.com | sh"
check_requirement "Rust/Cargo" "cargo --version" "curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh"

# Check Docker daemon
echo -n "Checking Docker daemon... "
if docker info &>/dev/null; then
    echo -e "${GREEN}✓${NC}"
else
    echo -e "${RED}✗${NC}"
    echo -e "  ${YELLOW}Start Docker: sudo systemctl start docker${NC}"
    validation_passed=false
fi

echo ""
echo "📁 Checking project files..."
echo "----------------------------"

required_files=("Cargo.toml" "Dockerfile" "containerlab.yml" "src/main.rs")
for file in "${required_files[@]}"; do
    echo -n "Checking $file... "
    if [ -f "$file" ]; then
        echo -e "${GREEN}✓${NC}"
    else
        echo -e "${RED}✗${NC}"
        validation_passed=false
    fi
done

echo ""
echo "🚀 Checking Rust compilation..."
echo "------------------------------"

echo -n "Testing compilation... "
if cargo check --quiet 2>/dev/null; then
    echo -e "${GREEN}✓${NC}"
else
    echo -e "${RED}✗${NC}"
    validation_passed=false
fi

echo ""
echo "📋 Validation Summary"
echo "===================="

if [ "$validation_passed" = true ]; then
    echo -e "${GREEN}🎉 All validations passed!${NC}"
    echo ""
    echo "Ready to deploy PolyTorus ContainerLab environment."
    echo ""
    echo "Next steps:"
    echo "1. Run setup: ./setup_containerlab.sh"
    echo "2. Run tests: ./test_transactions.sh"  
    echo "3. Monitor:   ./monitor_network.sh"
else
    echo -e "${RED}❌ Some validations failed.${NC}"
    echo "Please fix the issues above before proceeding."
    exit 1
fi
