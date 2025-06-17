#!/bin/bash

# Advanced clippy fixes for format strings
# This script handles complex format string replacements

set -e

echo "🔧 Starting advanced format string fixes..."

# Function to fix complex println patterns
fix_complex_println() {
    local file="$1"
    echo "  📝 Processing: $file"
    
    # Create temporary file
    local tmp_file=$(mktemp)
    
    # Process file line by line
    while IFS= read -r line; do
        # Fix single variable println patterns
        if [[ $line =~ println!\(\"([^\"]*)\{\}\",[[:space:]]*([^)]+)\) ]]; then
            format_str="${BASH_REMATCH[1]}"
            var_name="${BASH_REMATCH[2]// /}"
            new_line="        println!(\"${format_str}{${var_name}}\");"
            echo "$new_line" >> "$tmp_file"
        # Fix debug format patterns
        elif [[ $line =~ println!\(\"([^\"]*)\{\:\?\}\",[[:space:]]*([^)]+)\) ]]; then
            format_str="${BASH_REMATCH[1]}"
            var_name="${BASH_REMATCH[2]// /}"
            new_line="        println!(\"${format_str}{${var_name}:?}\");"
            echo "$new_line" >> "$tmp_file"
        # Fix format! patterns
        elif [[ $line =~ format!\(\"([^\"]*)\{\}\",[[:space:]]*([^)]+)\) ]]; then
            format_str="${BASH_REMATCH[1]}"
            var_name="${BASH_REMATCH[2]// /}"
            new_line="    let ${var_name%% *}_formatted = format!(\"${format_str}{${var_name}}\");"
            echo "$new_line" >> "$tmp_file"
        else
            echo "$line" >> "$tmp_file"
        fi
    done < "$file"
    
    # Replace original with fixed version
    mv "$tmp_file" "$file"
}

# Simple sed-based fixes for common patterns
fix_simple_patterns() {
    local file="$1"
    
    # Simple single-variable patterns
    sed -i 's/println!("\\([^"]*\\){}", \\([^)]*\\))/println!("\\1{\\2}")/g' "$file"
    sed -i 's/println!("\\([^"]*\\){:?}", \\([^)]*\\))/println!("\\1{\\2:?}")/g' "$file"
    sed -i 's/format!("\\([^"]*\\){}", \\([^)]*\\))/format!("\\1{\\2}")/g' "$file"
    sed -i 's/format!("\\([^"]*\\){:?}", \\([^)]*\\))/format!("\\1{\\2:?}")/g' "$file"
    
    # Remove redundant tokio imports
    sed -i '/^use tokio;$/d' "$file"
    
    # Fix vec! to arrays (simple cases only)
    sed -i 's/vec!\[true, false, true, false\]/[true, false, true, false]/g' "$file"
    
    # Fix let unit values
    sed -i 's/let _result = \\([^;]*\\);/\\1;/g' "$file"
}

# Apply fixes based on clippy output analysis
fix_specific_files() {
    echo "🎯 Applying specific file fixes..."
    
    # Fix examples with format issues
    for example in examples/*.rs; do
        if [[ -f "$example" ]]; then
            echo "  📁 Fixing: $example"
            fix_simple_patterns "$example"
        fi
    done
    
    # Fix test files
    for test in tests/*.rs; do
        if [[ -f "$test" ]]; then
            echo "  🧪 Fixing: $test"
            fix_simple_patterns "$test"
        fi
    done
    
    # Fix benchmark files
    for bench in benches/*.rs; do
        if [[ -f "$bench" ]]; then
            echo "  📊 Fixing: $bench"
            fix_simple_patterns "$bench"
        fi
    done
}

# Main execution
fix_specific_files

echo "✅ Format string fixes completed!"
echo "🔍 Running clippy to check remaining issues..."

# Test the fixes
if cargo clippy --all-targets --all-features -- -D warnings >/dev/null 2>&1; then
    echo "🎉 All clippy issues resolved!"
else
    echo "⚠️  Some issues remain - check output above"
fi
