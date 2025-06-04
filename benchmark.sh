#!/bin/bash

# Polytorus Blockchain Benchmark Runner

echo "=== PolyTorus Blockchain Benchmark Suite ==="
echo

# Check if criterion is available
echo "1. Running Criterion benchmarks (advanced benchmarking)..."
echo "   This will generate detailed HTML reports in target/criterion/"
echo

# Run criterion benchmarks
cargo bench --bench blockchain_bench

echo
echo "=== Benchmark Results ==="
echo "Criterion HTML reports are available at: target/criterion/report/index.html"
echo

# Optional: Run stdlib benchmarks (requires nightly)
if rustc --version | grep -q nightly; then
    echo "2. Running standard library benchmarks (nightly required)..."
    echo
    RUSTFLAGS="--cfg bench" cargo +nightly test --release --test stdlib_bench -- --bench
else
    echo "2. Standard library benchmarks skipped (requires nightly Rust)"
    echo "   To run stdlib benchmarks:"
    echo "   rustup install nightly"
    echo "   RUSTFLAGS=\"--cfg bench\" cargo +nightly test --release --test stdlib_bench -- --bench"
fi

echo
echo "=== Performance Tips ==="
echo "- Use 'cargo bench' for quick benchmarks"
echo "- Use 'cargo bench -- --save-baseline <name>' to save baselines"
echo "- Use 'cargo bench -- --baseline <name>' to compare against baselines"
echo "- HTML reports provide detailed analysis and graphs"
echo

# Check if hyperfine is available for additional benchmarking
if command -v hyperfine &> /dev/null; then
    echo "3. Running example performance tests with hyperfine..."
    echo
    
    echo "Difficulty adjustment example:"
    hyperfine --warmup 3 --runs 10 'cargo run --release --example difficulty_adjustment'
    
    echo
    echo "Simple difficulty test:"
    hyperfine --warmup 3 --runs 10 'cargo run --release --example simple_difficulty_test'
else
    echo "3. Install 'hyperfine' for additional performance measurements:"
    echo "   cargo install hyperfine"
fi

echo
echo "=== Benchmark Complete ==="
