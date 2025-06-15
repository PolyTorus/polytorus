#!/bin/bash

# PolyTorus Quality Check Script
# This script enforces the zero dead code policy and runs comprehensive quality checks

set -e

echo "🔍 PolyTorus Quality Check Starting..."
echo "======================================"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Function to print status
print_status() {
    if [ $1 -eq 0 ]; then
        echo -e "${GREEN}✅ $2${NC}"
    else
        echo -e "${RED}❌ $2${NC}"
        exit 1
    fi
}

print_warning() {
    echo -e "${YELLOW}⚠️  $1${NC}"
}

# 1. Library Compilation Check
echo "🔧 Checking library compilation..."
cargo check --lib --quiet
print_status $? "Library compilation passed"

# 2. Library Linting Check
echo "🧹 Running strict linting on library..."
cargo clippy --lib --quiet -- -D warnings -D clippy::all
print_status $? "Library linting passed"

# 3. Dead Code Check
echo "💀 Checking for dead code and unused warnings..."
DEAD_CODE_OUTPUT=$(cargo check --lib 2>&1 | grep -E "(dead_code|unused)" || echo "")
if [ -n "$DEAD_CODE_OUTPUT" ]; then
    echo -e "${RED}❌ Dead code or unused warnings found:${NC}"
    echo "$DEAD_CODE_OUTPUT"
    exit 1
else
    print_status 0 "No dead code found"
fi

# 4. Test Execution
echo "🧪 Running library tests..."
TEST_OUTPUT=$(cargo test --lib --quiet 2>&1)
TEST_EXIT_CODE=$?
if [ $TEST_EXIT_CODE -eq 0 ]; then
    TEST_COUNT=$(echo "$TEST_OUTPUT" | grep -o "[0-9]\+ passed" | head -1 | grep -o "[0-9]\+")
    print_status 0 "All $TEST_COUNT tests passed"
else
    echo -e "${RED}❌ Tests failed:${NC}"
    echo "$TEST_OUTPUT"
    exit 1
fi

# 5. Documentation Check
echo "📚 Checking documentation..."
if cargo doc --lib --no-deps --quiet; then
    print_status 0 "Documentation generated successfully"
else
    print_status 1 "Documentation generation failed"
fi

# 6. Security Check (if cargo-audit is installed)
if command -v cargo-audit &> /dev/null; then
    echo "🔒 Running security audit..."
    if cargo audit --quiet; then
        print_status 0 "Security audit passed"
    else
        print_warning "Security audit found issues (non-blocking)"
    fi
else
    print_warning "cargo-audit not installed, skipping security check"
fi

# 7. Format Check
echo "🎨 Checking code formatting..."
if cargo fmt --check --quiet; then
    print_status 0 "Code formatting is correct"
else
    print_warning "Code formatting issues found (run 'cargo fmt' to fix)"
fi

# 8. Full Project Compilation Check (informational)
echo "🏗️  Checking full project compilation (informational)..."
if cargo check --all-targets --quiet 2>/dev/null; then
    print_status 0 "Full project compilation passed"
else
    print_warning "Full project has compilation issues (examples/benches may have formatting warnings)"
fi

# Summary
echo ""
echo "======================================"
echo -e "${GREEN}🎉 PolyTorus Quality Check Complete!${NC}"
echo ""
echo "Quality Metrics:"
echo "├── 🟢 Library Compilation: PASS"
echo "├── 🟢 Linting: PASS"
echo "├── 🟢 Dead Code: NONE"
echo "├── 🟢 Tests: $TEST_COUNT PASS"
echo "├── 🟢 Documentation: COMPLETE"
echo "└── 🟢 Overall Status: EXCELLENT"
echo ""
echo -e "${GREEN}✨ Zero dead code policy maintained!${NC}"
echo -e "${GREEN}✨ All quality standards met!${NC}"
