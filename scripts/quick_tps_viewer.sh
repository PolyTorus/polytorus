#!/bin/bash

# Quick TPS Results Viewer

echo "=== PolyTorus TPS Results Quick Viewer ==="
echo

# Check for existing criterion results
if [ -d "target/criterion" ]; then
    echo "Criterion benchmark results found!"
    echo
    
    # List available benchmarks
    echo "Available benchmark results:"
    find target/criterion -maxdepth 1 -type d -name "*tps*" -o -name "*transaction*" -o -name "*block*" | sort
    echo
    
    # Quick stats if available
    echo "Quick Statistics:"
    
    # Look for TPS-related results
    for dir in target/criterion/*/; do
        if [[ "$dir" == *"tps"* ]] || [[ "$dir" == *"transaction"* ]]; then
            benchmark_name=$(basename "$dir")
            echo "  $benchmark_name:"
            
            # Look for latest measurement files
            latest_files=$(find "$dir" -name "*.json" -newer "$dir" 2>/dev/null | head -3)
            if [ -n "$latest_files" ]; then
                echo "    - Has measurement data"
            else
                echo "    - No measurement data found"
            fi
            
            # Check for HTML reports
            if [ -d "$dir/report" ]; then
                echo "    - HTML report available"
            fi
        fi
    done
    echo
    
    # Main report link
    main_report="target/criterion/report/index.html"
    if [ -f "$main_report" ]; then
        echo "Main HTML Report: $main_report"
        echo "View with: firefox $main_report"
        echo
    fi
    
    # Suggest analysis
    echo "For detailed analysis, run:"
    echo "  ./analyze_tps.sh"
    
else
    echo "No benchmark results found yet."
    echo
    echo "To run TPS benchmarks:"
    echo "  ./benchmark_tps.sh          # Full TPS benchmark suite"
    echo "  ./simple_tps_test.sh        # Quick TPS test"
    echo
    echo "Individual benchmarks:"
    echo "  cargo bench --bench blockchain_bench benchmark_tps"
    echo "  cargo bench --bench blockchain_bench benchmark_pure_transaction_processing"
    echo "  cargo bench --bench blockchain_bench benchmark_concurrent_tps"
fi

# Check if any benchmark is currently running
if pgrep -f "blockchain_bench" > /dev/null; then
    echo
    echo "⚠️  Benchmark appears to be currently running..."
    echo "   Process: $(pgrep -f blockchain_bench)"
    echo "   You may need to wait for completion or stop it with:"
    echo "   pkill -f blockchain_bench"
fi

echo
echo "Quick TPS viewer complete!"
