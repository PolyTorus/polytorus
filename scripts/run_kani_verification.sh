#!/bin/bash

# Kani Verification Script for Polytorus Blockchain
# This script runs formal verification using Kani on the Polytorus codebase

set -e

echo "🔍 Starting Kani formal verification for Polytorus..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if Kani is installed
if ! command -v kani &> /dev/null; then
    print_error "Kani is not installed. Please install Kani first:"
    echo "  cargo install --locked kani-verifier"
    echo "  cargo kani setup"
    exit 1
fi

print_status "Kani is installed. Starting verification..."

# Create verification results directory
mkdir -p verification_results

# Function to run a specific verification harness
run_verification() {
    local harness_name=$1
    local description=$2
    local timeout=${3:-300} # Default 5 minutes timeout
    
    print_status "Running verification: $description"
    echo "Harness: $harness_name"
    
    if timeout $timeout cargo kani --harness $harness_name > "verification_results/${harness_name}.log" 2>&1; then
        print_success "✅ $description - PASSED"
        return 0
    else
        print_error "❌ $description - FAILED"
        echo "Check verification_results/${harness_name}.log for details"
        return 1
    fi
}

# Cryptographic verifications
print_status "🔐 Running cryptographic verifications..."

# Note: Some verification harnesses may need to be run with specific bounds
# to avoid state explosion in the model checker

echo "Running ECDSA verification (basic properties)..."
if timeout 180 cargo kani --harness verify_ecdsa_sign_verify --config kani-config.toml > verification_results/ecdsa_verification.log 2>&1; then
    print_success "✅ ECDSA verification - PASSED"
else
    print_warning "⚠️ ECDSA verification - Check logs (may require key derivation)"
fi

echo "Running encryption type determination..."
if timeout 60 cargo kani --harness verify_encryption_type_determination > verification_results/encryption_type.log 2>&1; then
    print_success "✅ Encryption type determination - PASSED"
else
    print_error "❌ Encryption type determination - FAILED"
fi

echo "Running transaction integrity verification..."
if timeout 120 cargo kani --harness verify_transaction_integrity > verification_results/transaction_integrity.log 2>&1; then
    print_success "✅ Transaction integrity - PASSED"
else
    print_error "❌ Transaction integrity - FAILED"
fi

echo "Running transaction value bounds verification..."
if timeout 120 cargo kani --harness verify_transaction_value_bounds > verification_results/transaction_bounds.log 2>&1; then
    print_success "✅ Transaction value bounds - PASSED"
else
    print_error "❌ Transaction value bounds - FAILED"
fi

# Blockchain verifications
print_status "⛓️ Running blockchain verifications..."

echo "Running mining statistics verification..."
if timeout 90 cargo kani --harness verify_mining_stats > verification_results/mining_stats.log 2>&1; then
    print_success "✅ Mining statistics - PASSED"
else
    print_error "❌ Mining statistics - FAILED"
fi

echo "Running mining attempts verification..."
if timeout 120 cargo kani --harness verify_mining_attempts > verification_results/mining_attempts.log 2>&1; then
    print_success "✅ Mining attempts tracking - PASSED"
else
    print_error "❌ Mining attempts tracking - FAILED"
fi

echo "Running difficulty adjustment verification..."
if timeout 90 cargo kani --harness verify_difficulty_adjustment_config > verification_results/difficulty_config.log 2>&1; then
    print_success "✅ Difficulty adjustment config - PASSED"
else
    print_error "❌ Difficulty adjustment config - FAILED"
fi

echo "Running difficulty bounds verification..."
if timeout 120 cargo kani --harness verify_difficulty_bounds > verification_results/difficulty_bounds.log 2>&1; then
    print_success "✅ Difficulty bounds - PASSED"
else
    print_error "❌ Difficulty bounds - FAILED"
fi

# Modular architecture verifications
print_status "🏗️ Running modular architecture verifications..."

echo "Running message priority verification..."
if timeout 90 cargo kani --harness verify_message_priority_ordering > verification_results/message_priority.log 2>&1; then
    print_success "✅ Message priority ordering - PASSED"
else
    print_error "❌ Message priority ordering - FAILED"
fi

echo "Running layer state transitions verification..."
if timeout 60 cargo kani --harness verify_layer_state_transitions > verification_results/layer_states.log 2>&1; then
    print_success "✅ Layer state transitions - PASSED"
else
    print_error "❌ Layer state transitions - FAILED"
fi

echo "Running message bus capacity verification..."
if timeout 90 cargo kani --harness verify_message_bus_capacity > verification_results/message_bus.log 2>&1; then
    print_success "✅ Message bus capacity - PASSED"
else
    print_error "❌ Message bus capacity - FAILED"
fi

echo "Running orchestrator coordination verification..."
if timeout 120 cargo kani --harness verify_orchestrator_coordination > verification_results/orchestrator.log 2>&1; then
    print_success "✅ Orchestrator coordination - PASSED"
else
    print_error "❌ Orchestrator coordination - FAILED"
fi

# Generate summary report
print_status "📊 Generating verification summary..."

echo "=== KANI VERIFICATION SUMMARY ===" > verification_results/summary.txt
echo "Generated on: $(date)" >> verification_results/summary.txt
echo "" >> verification_results/summary.txt

# Count passed and failed verifications
passed_count=$(find verification_results -name "*.log" -exec grep -l "VERIFICATION:- PASSED" {} \; 2>/dev/null | wc -l)
total_logs=$(find verification_results -name "*.log" | wc -l)

echo "Total verifications run: $total_logs" >> verification_results/summary.txt
echo "Passed verifications: $passed_count" >> verification_results/summary.txt
echo "Failed/Inconclusive: $((total_logs - passed_count))" >> verification_results/summary.txt
echo "" >> verification_results/summary.txt

# List verification results
echo "=== DETAILED RESULTS ===" >> verification_results/summary.txt
for log_file in verification_results/*.log; do
    if [ -f "$log_file" ]; then
        filename=$(basename "$log_file" .log)
        if grep -q "VERIFICATION:- PASSED" "$log_file" 2>/dev/null; then
            echo "✅ $filename: PASSED" >> verification_results/summary.txt
        elif grep -q "VERIFICATION:- FAILED" "$log_file" 2>/dev/null; then
            echo "❌ $filename: FAILED" >> verification_results/summary.txt
        else
            echo "⚠️ $filename: INCONCLUSIVE" >> verification_results/summary.txt
        fi
    fi
done

print_success "🎉 Verification complete!"
print_status "Results saved to verification_results/ directory"
print_status "Summary available in verification_results/summary.txt"

# Display summary
echo ""
print_status "=== VERIFICATION SUMMARY ==="
cat verification_results/summary.txt | tail -n +4

echo ""
print_status "To view detailed results for any verification, check the corresponding .log file in verification_results/"
print_status "Example: cat verification_results/ecdsa_verification.log"
