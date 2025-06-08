# PolyTorus TPS Benchmark Analysis

## Overview
This document explains how to analyze TPS (Transactions Per Second) benchmark results for the PolyTorus blockchain.

## Benchmark Types

### 1. TPS Throughput Benchmark (`benchmark_tps`)
**Purpose**: Measure real-world TPS performance including mining and validation

**Measurement Items**:
- Processing speed with 10, 25, 50 transactions
- Complete process of block creation, mining, and validation
- Optimized performance with low difficulty settings

**Key Metrics**:
- Mean processing time
- Standard deviation
- Throughput (Transactions/second)

### 2. Pure Transaction Processing (`benchmark_pure_transaction_processing`)
**Purpose**: Measure pure transaction processing speed without mining

**Measurement Items**:
- Processing speed with 50, 100, 500 transactions
- Transaction creation and basic validation only
- Theoretical maximum TPS

### 3. Concurrent TPS (`benchmark_concurrent_tps`)
**Purpose**: Measure TPS performance in multi-threaded environments

**Measurement Items**:
- Parallel processing with 2, 4 threads
- Processing efficiency between threads
- Scalability evaluation

## Analysis Methods

### 1. HTML Report Review
```bash
# View detailed results in browser
firefox target/criterion/report/index.html
```

### 2. TPS Calculation
```
Effective TPS = Number of Transactions / Processing Time (seconds)
```

### 3. Performance Comparison
- **Baseline TPS**: Pure transaction processing results
- **Real-world TPS**: TPS throughput results  
- **Concurrent efficiency**: (Parallel TPS / Single-thread TPS) / Thread count

## Optimization Guidelines

### High TPS Approaches
1. **Transaction Validation Optimization**
   - Parallel signature verification
   - High-speed UTXO lookup

2. **Mining Efficiency Improvement**
   - Adaptive difficulty adjustment
   - Hash calculation optimization

3. **Memory Usage Reduction**
   - Efficient data structures
   - Caching strategies

4. **Parallel Processing Enhancement**
   - Lock-free data structures
   - Worker pool utilization

## Benchmark Execution Commands

```bash
# Execute all TPS benchmarks
./benchmark_tps.sh

# Execute individual benchmarks
cargo bench --bench blockchain_bench benchmark_tps
cargo bench --bench blockchain_bench benchmark_pure_transaction_processing
cargo bench --bench blockchain_bench benchmark_concurrent_tps

# Comparative benchmarks (save baseline)
cargo bench --bench blockchain_bench -- --save-baseline before_optimization

# Compare after optimization
cargo bench --bench blockchain_bench -- --baseline before_optimization
```

## Expected Values and Benchmark Targets

### Development Environment TPS Targets
- **Pure transaction processing**: 1,000+ TPS
- **With mining (low difficulty)**: 100+ TPS  
- **Production scenario**: 10-50 TPS

### Performance Improvement Indicators
- 10%+ improvement indicates significant enhancement
- Regression detection: 5%+ decrease triggers warning

## Troubleshooting

### Common Issues
1. **Memory Shortage**: OOM with large transaction volumes
2. **CPU Usage**: High load during mining processes
3. **I/O Waiting**: Database operation bottlenecks

### Solutions
```bash
# Memory usage limit
ulimit -m 2097152  # 2GB limit

# CPU usage monitoring
htop &
cargo bench --bench blockchain_bench

# I/O monitoring
iotop &
```
