#!/bin/bash

# Polytorus clippy fixes automation script
# This script applies common clippy fixes automatically

set -e

echo "🔧 Starting automatic clippy fixes..."

# Function to fix format strings in a file
fix_format_strings() {
    local file="$1"
    if [ -f "$file" ]; then
        echo "  📝 Fixing format strings in $file"
        
        # Fix println! format strings - common patterns
        sed -i 's/println!("\([^"]*\){}", \([^)]*\))/println!("\1{\2}")/g' "$file"
        sed -i 's/println!("\([^"]*\){:?}", \([^)]*\))/println!("\1{\2:?}")/g' "$file"
        sed -i 's/format!("\([^"]*\){}", \([^)]*\))/format!("\1{\2}")/g' "$file"
        sed -i 's/format!("\([^"]*\){:?}", \([^)]*\))/format!("\1{\2:?}")/g' "$file"
        
        # More complex replacements for multiple variables
        # This is a simplified approach - manual fixes might be needed for complex cases
    fi
}

# Function to fix other common issues
fix_common_issues() {
    local file="$1"
    if [ -f "$file" ]; then
        echo "  🔧 Fixing common issues in $file"
        
        # Remove redundant tokio imports
        sed -i '/^use tokio;$/d' "$file"
        
        # Fix vec! to array where appropriate (simple cases)
        sed -i 's/vec!\[\([^]]*\)\]/[\1]/g' "$file"
        
        # Fix let unit values (simple pattern)
        sed -i 's/let _result = \([^;]*\);/\1;/g' "$file"
    fi
}

# Get list of Rust files with issues
echo "🔍 Finding Rust files..."
RUST_FILES=$(find . -name "*.rs" -not -path "./target/*" -not -path "./.git/*")

# Apply fixes to each file
for file in $RUST_FILES; do
    if [[ -f "$file" ]]; then
        fix_format_strings "$file"
        fix_common_issues "$file"
    fi
done

# Manual fixes for specific files mentioned in clippy output
echo "🎯 Applying specific fixes..."

# Fix test files - remove unused imports
if [ -f "tests/real_diamond_io_integration_tests.rs" ]; then
    echo "  🧪 Fixing test file imports"
    sed -i '/^use tokio;$/d' "tests/real_diamond_io_integration_tests.rs"
fi

if [ -f "tests/real_diamond_io_integration_tests_new.rs" ]; then
    echo "  🧪 Fixing new test file"
    sed -i '/^use tokio;$/d' "tests/real_diamond_io_integration_tests_new.rs"
    sed -i '/^use polytorus::diamond_io_integration_new::DiamondIOResult;$/d' "tests/real_diamond_io_integration_tests_new.rs"
    # Fix the useless comparison
    sed -i 's/assert!(evaluation_result\.execution_time_ms >= 0);/\/\/ execution_time_ms is always >= 0 for u64/g' "tests/real_diamond_io_integration_tests_new.rs"
fi

# Fix bool assertions in crypto module
if [ -f "src/crypto/real_diamond_io.rs" ]; then
    echo "  🔐 Fixing crypto module assertions"
    sed -i 's/assert_eq!(testing_config\.enable_disk_storage, false);/assert!(!testing_config.enable_disk_storage);/g' "src/crypto/real_diamond_io.rs"
    sed -i 's/assert_eq!(production_config\.enable_disk_storage, true);/assert!(production_config.enable_disk_storage);/g' "src/crypto/real_diamond_io.rs"
fi

echo "✅ Automatic fixes completed!"
echo "⚠️  Some complex format string issues may need manual fixing"
echo "🔍 Run 'make clippy' to check remaining issues"
