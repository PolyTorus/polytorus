#!/bin/bash

# PolyTorus TPS (Transactions Per Second) Benchmark

echo "=== PolyTorus TPS Benchmark ==="
echo "This script measures the Transactions Per Second (TPS) performance of PolyTorus blockchain."
echo

# Clean previous results
echo "Cleaning previous benchmark results..."
rm -rf target/criterion/tps_throughput target/criterion/pure_transaction_tps target/criterion/concurrent_tps

echo
echo "Starting TPS benchmarks..."
echo "Note: This may take several minutes to complete."
echo

# Run only TPS-related benchmarks
echo "1. Running TPS throughput benchmark (with mining)..."
cargo bench --bench blockchain_bench -- benchmark_tps

echo
echo "2. Running pure transaction processing benchmark (no mining)..."
cargo bench --bench blockchain_bench -- benchmark_pure_transaction_processing

echo
echo "3. Running concurrent TPS benchmark..."
cargo bench --bench blockchain_bench -- benchmark_concurrent_tps

echo
echo "=== TPS Benchmark Results Summary ==="
echo
echo "Detailed results are available in:"
echo "  - target/criterion/tps_throughput/report/index.html"
echo "  - target/criterion/pure_transaction_tps/report/index.html"
echo "  - target/criterion/concurrent_tps/report/index.html"
echo
echo "Key metrics to analyze:"
echo "  1. TPS Throughput: Real-world TPS including mining and validation"
echo "  2. Pure Transaction TPS: Maximum theoretical transaction processing speed"
echo "  3. Concurrent TPS: Multi-threaded transaction processing performance"
echo

# Extract and display summary if criterion results exist
if [ -d "target/criterion" ]; then
    echo "Quick TPS Summary:"
    echo "=================="
    
    # Find the latest TPS results
    if [ -d "target/criterion/tps_throughput" ]; then
        echo "TPS with mining (latest results):"
        find target/criterion/tps_throughput -name "*.json" -exec grep -l "mean" {} \; | head -1 | xargs cat 2>/dev/null | grep -o '"mean":[0-9.]*' || echo "Results parsing requires manual inspection"
    fi
    
    if [ -d "target/criterion/pure_transaction_tps" ]; then
        echo "Pure transaction processing TPS:"
        echo "See detailed results in HTML reports"
    fi
    
    echo
    echo "For detailed analysis, open the HTML reports in your browser."
fi

echo
echo "=== Performance Optimization Tips ==="
echo "To improve TPS performance:"
echo "1. Reduce block difficulty for faster mining"
echo "2. Optimize transaction validation logic"
echo "3. Implement parallel transaction processing"
echo "4. Use more efficient data structures"
echo "5. Optimize database operations"
echo

echo "TPS benchmark complete!"
