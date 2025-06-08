#!/bin/bash

# TPS Benchmark Results Analyzer

echo "=== PolyTorus TPS Benchmark Results Analyzer ==="
echo

# Check if criterion results exist
if [ ! -d "target/criterion" ]; then
    echo "Error: No benchmark results found. Please run benchmarks first:"
    echo "  ./benchmark_tps.sh"
    exit 1
fi

echo "Analyzing TPS benchmark results..."
echo

# Function to extract TPS from criterion results
analyze_tps_results() {
    local benchmark_name=$1
    local description=$2
    
    if [ -d "target/criterion/$benchmark_name" ]; then
        echo "=== $description ==="
        
        # Find the latest results directory
        latest_dir=$(find target/criterion/$benchmark_name -name "report" -type d | head -1)
        
        if [ -n "$latest_dir" ]; then
            echo "Results directory: $latest_dir"
            
            # Look for JSON files with measurement data
            json_files=$(find target/criterion/$benchmark_name -name "*.json" 2>/dev/null)
            
            if [ -n "$json_files" ]; then
                echo "Raw measurement files found:"
                echo "$json_files" | while read file; do
                    echo "  - $(basename $file)"
                done
            fi
            
            # Check for HTML report
            html_report="$latest_dir/index.html"
            if [ -f "$html_report" ]; then
                echo "HTML Report: $html_report"
                echo "Open with: firefox $html_report"
            fi
        else
            echo "No detailed results found for $benchmark_name"
        fi
        echo
    else
        echo "No results found for $benchmark_name"
        echo
    fi
}

# Analyze each TPS benchmark
analyze_tps_results "tps_throughput" "TPS with Mining and Validation"
analyze_tps_results "pure_transaction_tps" "Pure Transaction Processing TPS"
analyze_tps_results "concurrent_tps" "Concurrent/Parallel TPS"

# Generate summary report
echo "=== TPS Summary Report ==="
echo "Generated on: $(date)"
echo

# Check for main criterion report
if [ -f "target/criterion/report/index.html" ]; then
    echo "Main Criterion Report: target/criterion/report/index.html"
    echo "This contains comprehensive results for all benchmarks."
    echo
fi

# TPS calculation helper
echo "=== TPS Calculation Helper ==="
echo "To calculate TPS from benchmark results:"
echo "  TPS = Number_of_Transactions / Time_in_Seconds"
echo
echo "For example:"
echo "  - 50 transactions in 2.5 seconds = 20 TPS"
echo "  - 100 transactions in 1.8 seconds = 55.6 TPS"
echo

# Performance optimization suggestions
echo "=== Performance Optimization Suggestions ==="
echo "Based on typical blockchain TPS bottlenecks:"
echo
echo "1. **Transaction Validation**:"
echo "   - Parallelize signature verification"
echo "   - Optimize UTXO lookups"
echo "   - Cache frequently accessed data"
echo
echo "2. **Block Mining**:"
echo "   - Adjust difficulty for target block time"
echo "   - Use efficient hashing algorithms"
echo "   - Implement adaptive difficulty"
echo
echo "3. **Concurrency**:"
echo "   - Process independent transactions in parallel"
echo "   - Use lock-free data structures"
echo "   - Implement efficient worker pools"
echo
echo "4. **Storage**:"
echo "   - Optimize database operations"
echo "   - Use appropriate indexing"
echo "   - Consider in-memory caching"
echo

# Comparison with other blockchains
echo "=== Blockchain TPS Comparison Reference ==="
echo "For context, here are typical TPS values:"
echo "  - Bitcoin: ~7 TPS"
echo "  - Ethereum: ~15 TPS"
echo "  - Solana: ~65,000 TPS (claimed)"
echo "  - Polygon: ~7,000 TPS"
echo "  - BSC: ~160 TPS"
echo
echo "Note: These are theoretical/peak values and real-world performance varies."
echo

echo "=== Next Steps ==="
echo "1. Review HTML reports for detailed analysis"
echo "2. Compare results with baseline measurements"
echo "3. Identify bottlenecks using profiling tools"
echo "4. Implement optimizations and re-benchmark"
echo

echo "TPS analysis complete!"
