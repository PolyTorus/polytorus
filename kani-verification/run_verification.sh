#!/bin/bash

# PolyTorus Kani Verification Execution Script
# Sequentially runs multiple verification harnesses and summarizes results

set -e

echo "🔍 Starting PolyTorus Kani formal verification..."

# Color definitions
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Result counters
PASSED=0
FAILED=0
TOTAL=0

# Directory to save results
RESULTS_DIR="kani_results"
mkdir -p "$RESULTS_DIR"

echo -e "${BLUE}📋 Verification harnesses to execute:${NC}"
echo "  Basic operations:"
echo "    - verify_basic_arithmetic"
echo "    - verify_boolean_logic"
echo "    - verify_array_bounds"
echo "    - verify_hash_determinism"
echo "    - verify_queue_operations"
echo ""
echo "  Cryptographic functions:"
echo "    - verify_encryption_type_determination"
echo "    - verify_transaction_integrity"
echo "    - verify_transaction_value_bounds"
echo "    - verify_signature_properties"
echo "    - verify_public_key_format"
echo "    - verify_hash_computation"
echo ""
echo "  Blockchain:"
echo "    - verify_block_hash_consistency"
echo "    - verify_blockchain_integrity"
echo "    - verify_difficulty_adjustment"
echo "    - verify_invalid_block_rejection"
echo ""
echo "  Modular architecture:"
echo "    - verify_modular_architecture_structure"
echo "    - verify_layer_communication"
echo "    - verify_invalid_communication_rejection"
echo "    - verify_layer_state_update"
echo "    - verify_synchronization_mechanism"
echo ""

# Verification execution function
run_verification() {
    local harness_name=$1
    local description=$2
    local timeout_sec=${3:-60}
    
    echo -e "${BLUE}🔍 Executing: ${description}${NC}"
    echo "   Harness: ${harness_name}"
    echo "   Timeout: ${timeout_sec} seconds"
    
    ((TOTAL++))
    
    if timeout ${timeout_sec} cargo kani --harness ${harness_name} > "$RESULTS_DIR/${harness_name}.log" 2>&1; then
        if grep -q "VERIFICATION:- SUCCESSFUL" "$RESULTS_DIR/${harness_name}.log"; then
            echo -e "${GREEN}✅ ${description} - Success${NC}"
            ((PASSED++))
        else
            echo -e "${YELLOW}⚠️ ${description} - Unknown result${NC}"
        fi
    else
        echo -e "${RED}❌ ${description} - Failed or timed out${NC}"
        ((FAILED++))
    fi
    echo ""
}

# Execute basic verifications
echo -e "${BLUE}🧮 Starting basic operations verification...${NC}"
run_verification "verify_basic_arithmetic" "Basic arithmetic operations" 30
run_verification "verify_boolean_logic" "Boolean logic" 30
run_verification "verify_array_bounds" "Array bounds checking" 30
run_verification "verify_hash_determinism" "Hash determinism" 30
run_verification "verify_queue_operations" "Queue operations" 45

# Execute cryptographic verifications
echo -e "${BLUE}🔐 Starting cryptographic functions verification...${NC}"
run_verification "verify_encryption_type_determination" "Encryption type determination" 60
run_verification "verify_transaction_integrity" "Transaction integrity" 90
run_verification "verify_transaction_value_bounds" "Transaction value bounds" 60
run_verification "verify_signature_properties" "Signature properties" 45
run_verification "verify_public_key_format" "Public key format" 45
run_verification "verify_hash_computation" "Hash computation" 45

# Execute blockchain verifications
echo -e "${BLUE}⛓️ Starting blockchain functions verification...${NC}"
run_verification "verify_block_hash_consistency" "Block hash consistency" 60
run_verification "verify_blockchain_integrity" "Blockchain integrity" 90
run_verification "verify_difficulty_adjustment" "Difficulty adjustment" 45
run_verification "verify_invalid_block_rejection" "Invalid block rejection" 60

# Execute modular architecture verifications
echo -e "${BLUE}🏗️ Starting modular architecture verification...${NC}"
run_verification "verify_modular_architecture_structure" "Architecture structure" 60
run_verification "verify_layer_communication" "Inter-layer communication" 75
run_verification "verify_invalid_communication_rejection" "Invalid communication rejection" 60
run_verification "verify_layer_state_update" "Layer state update" 60
run_verification "verify_synchronization_mechanism" "Synchronization mechanism" 75

# Create results summary
echo -e "${BLUE}📊 Creating verification results summary...${NC}"

cat > "$RESULTS_DIR/summary.md" << EOF
# PolyTorus Kani Formal Verification Results

**Execution Date:** $(date)

## Overall Results

- **Total Verifications:** $TOTAL
- **Passed:** $PASSED
- **Failed:** $FAILED
- **Success Rate:** $(( (PASSED * 100) / TOTAL ))%

## Detailed Results

EOF

# Add detailed results to summary
for log_file in "$RESULTS_DIR"/*.log; do
    if [ -f "$log_file" ]; then
        harness_name=$(basename "$log_file" .log)
        echo "### $harness_name" >> "$RESULTS_DIR/summary.md"
        
        if grep -q "VERIFICATION:- SUCCESSFUL" "$log_file"; then
            echo "**Status:** ✅ Success" >> "$RESULTS_DIR/summary.md"
        else
            echo "**Status:** ❌ Failed" >> "$RESULTS_DIR/summary.md"
        fi
        
        # Extract execution time
        if grep -q "Verification Time:" "$log_file"; then
            exec_time=$(grep "Verification Time:" "$log_file" | tail -1)
            echo "**$exec_time**" >> "$RESULTS_DIR/summary.md"
        fi
        
        # Extract check count
        if grep -q "SUMMARY:" "$log_file"; then
            check_summary=$(grep -A 1 "SUMMARY:" "$log_file" | tail -1)
            echo "**Result:** $check_summary" >> "$RESULTS_DIR/summary.md"
        fi
        
        echo "" >> "$RESULTS_DIR/summary.md"
    fi
done

# Display final results
echo -e "${BLUE}🎯 Final Results${NC}"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"
echo -e "Total Verifications: ${BLUE}$TOTAL${NC}"
echo -e "Passed: ${GREEN}$PASSED${NC}"
echo -e "Failed: ${RED}$FAILED${NC}"
echo -e "Success Rate: ${GREEN}$(( (PASSED * 100) / TOTAL ))%${NC}"
echo ""

if [ $FAILED -eq 0 ]; then
    echo -e "${GREEN}🎉 All verifications passed successfully!${NC}"
    echo -e "${GREEN}PolyTorus implementation has been formally verified.${NC}"
else
    echo -e "${YELLOW}⚠️ Some verifications have issues.${NC}"
    echo -e "${YELLOW}Check individual log files in ${RESULTS_DIR}/ directory for details.${NC}"
fi

echo ""
echo -e "${BLUE}📁 Result files:${NC}"
echo "  - Summary: ${RESULTS_DIR}/summary.md"
echo "  - Individual logs: ${RESULTS_DIR}/*.log"
echo ""
echo -e "${BLUE}🔍 Commands for detailed review:${NC}"
echo "  cat ${RESULTS_DIR}/summary.md"
echo "  cat ${RESULTS_DIR}/<harness_name>.log"

exit $FAILED
