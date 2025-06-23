# PolyTorus Realistic Testnet Guide

## Overview

This guide explains how to use the enhanced ContainerLab topology for PolyTorus that simulates realistic network conditions with Autonomous System (AS) separation, geographic distribution, and various network constraints.

## Architecture

### Autonomous Systems

The testnet simulates four autonomous systems representing different global regions:

#### AS65001 - North America
- **Tier**: Tier-1 ISP infrastructure
- **Characteristics**: High bandwidth (1Gbps), low latency (10-50ms)
- **Nodes**:
  - `bootstrap-na`: Primary bootstrap node with 99.9% uptime
  - `miner-pool-na`: High-performance mining pool infrastructure
  - `exchange-na`: Exchange infrastructure with compliance requirements

#### AS65002 - Europe
- **Tier**: Datacenter/institutional infrastructure
- **Characteristics**: Good bandwidth (100-500Mbps), moderate latency (80-120ms to NA)
- **Nodes**:
  - `validator-institution-eu`: Institutional validator with GDPR compliance
  - `research-eu`: Academic research node with experimental features

#### AS65003 - Asia-Pacific
- **Tier**: Business ISP with mobile optimization
- **Characteristics**: Variable bandwidth (25-200Mbps), high latency (150-250ms to other regions)
- **Nodes**:
  - `miner-apac`: Regional miner with trans-Pacific connectivity
  - `mobile-backend-apac`: Mobile wallet backend with carrier-grade connectivity

#### AS65004 - Edge/Mobile
- **Tier**: Satellite and rural connectivity
- **Characteristics**: Limited bandwidth (2-25Mbps), very high latency (300-2000ms)
- **Nodes**:
  - `light-client-mobile`: Mobile light client for edge devices
  - `rural-satellite`: Rural node with satellite connectivity

### Network Characteristics

#### Latency Matrix
```
           NA    EU    APAC  EDGE
NA         10ms  100ms 180ms 50ms
EU         100ms 15ms  220ms 80ms
APAC       180ms 220ms 20ms  150ms
EDGE       50ms  80ms  150ms 100ms
```

#### Bandwidth Limits
- **Tier-1 (NA)**: 500Mbps - 1Gbps
- **Datacenter (EU)**: 100-500Mbps
- **Business (APAC)**: 25-200Mbps
- **Mobile/Satellite (EDGE)**: 2-25Mbps

#### Packet Loss
- **Fiber connections**: 0.01-0.1%
- **Wireless/cellular**: 0.1-1%
- **Satellite connections**: 1-2%

## Quick Start

### Prerequisites

1. **ContainerLab**: Install with `bash -c "$(curl -sL https://get.containerlab.dev)"`
2. **Docker**: Container runtime
3. **Rust/Cargo**: For building PolyTorus
4. **Linux Traffic Control (tc)**: For network impairments
5. **FRRouting (optional)**: For BGP simulation

### Basic Usage

1. **Start the realistic testnet**:
```bash
./scripts/realistic_testnet_simulation.sh
```

2. **Start with custom parameters**:
```bash
./scripts/realistic_testnet_simulation.sh 1800 200 15 false
# Duration: 30 minutes, 200 transactions, 15s interval, no chaos mode
```

3. **Enable chaos engineering**:
```bash
./scripts/realistic_testnet_simulation.sh 3600 500 10 true
# 1 hour simulation with chaos testing enabled
```

## Advanced Configuration

### Network Simulation Parameters

Edit `/home/shiro/workspace/polytorus/config/realistic-testnet.toml` to adjust:

#### Geographic Latency Settings
```toml
[network.latency_matrix]
north_america_to_europe = 100
north_america_to_asia_pacific = 180
europe_to_asia_pacific = 220
# Add jitter and packet loss per link
```

#### Regional Characteristics
```toml
[network.regions.north_america]
base_latency_ms = 10
jitter_ms = 2
bandwidth_mbps = 1000
packet_loss_percent = 0.01
connectivity_tier = "tier1_isp"
```

#### Node Type Definitions
```toml
[node_types.mining_pool]
description = "High-performance mining pool infrastructure"
min_uptime_percent = 99.5
min_bandwidth_mbps = 200
max_latency_ms = 20
required_connections = 15
```

### BGP Configuration

The testnet includes FRR routers for realistic BGP simulation:

#### Viewing BGP Status
```bash
# Check BGP neighbors
docker exec clab-polytorus-realistic-testnet-router-na vtysh -c "show ip bgp summary"

# View routing table
docker exec clab-polytorus-realistic-testnet-router-na vtysh -c "show ip route"

# Check BGP routes
docker exec clab-polytorus-realistic-testnet-router-na vtysh -c "show ip bgp"
```

#### BGP Communities
- `65001:100`: North America routes
- `65002:777`: GDPR protected routes (Europe)
- `65003:555`: Mobile optimized routes (APAC)
- `65004:999`: Satellite/low-bandwidth routes (Edge)

### Traffic Control Examples

#### Manual Network Impairment
```bash
# Add 200ms latency with 20ms jitter
docker exec clab-polytorus-realistic-testnet-miner-apac \
  tc qdisc add dev eth1 root netem delay 200ms 20ms

# Limit bandwidth to 10Mbps
docker exec clab-polytorus-realistic-testnet-rural-satellite \
  tc qdisc add dev eth1 root handle 1: tbf rate 10mbit burst 10kb latency 50ms

# Add packet loss
docker exec clab-polytorus-realistic-testnet-light-client-mobile \
  tc qdisc add dev eth1 root netem loss 1%
```

#### Network Partition Simulation
```bash
# Isolate APAC region
docker exec clab-polytorus-realistic-testnet-router-apac \
  tc qdisc add dev eth2 root netem loss 100%
docker exec clab-polytorus-realistic-testnet-router-apac \
  tc qdisc add dev eth3 root netem loss 100%

# Restore connectivity
docker exec clab-polytorus-realistic-testnet-router-apac \
  tc qdisc del dev eth2 root
docker exec clab-polytorus-realistic-testnet-router-apac \
  tc qdisc del dev eth3 root
```

## Monitoring & Observability

### Node Status Endpoints

Each node exposes HTTP APIs for monitoring:

```bash
# Bootstrap node status
curl http://localhost:9000/status

# Mining pool statistics
curl http://localhost:9001/stats

# Institutional validator metrics
curl http://localhost:9010/metrics
```

### Network Performance Monitoring

The simulation includes automated monitoring for:

- **Inter-AS connectivity**: Latency and reachability between regions
- **Bandwidth utilization**: Traffic patterns and congestion
- **Partition detection**: Network splits and healing
- **BGP convergence**: Routing table updates and stability

### Blockchain Metrics

Monitor blockchain-specific metrics:

- **Block propagation**: Time for blocks to reach all regions
- **Transaction latency**: End-to-end transaction confirmation time
- **Fork resolution**: Consensus behavior during network partitions
- **Mining distribution**: Hash rate distribution across regions

## Testing Scenarios

### 1. Geographic Distribution Testing

**Objective**: Validate blockchain performance across global regions

**Test Steps**:
1. Deploy full testnet
2. Generate transactions from each region
3. Monitor block propagation times
4. Measure transaction confirmation latency

**Expected Results**:
- Blocks propagate within 30-60 seconds globally
- Transaction finality varies by region (10s NA, 60s satellite)
- No consensus failures during normal operation

### 2. Network Partition Testing

**Objective**: Test consensus resilience during network splits

**Test Steps**:
1. Start testnet in normal operation
2. Simulate partition isolating one region
3. Monitor consensus behavior
4. Heal partition and observe recovery

**Expected Results**:
- Consensus continues in majority partition
- Minority partition stops producing blocks
- Recovery occurs within 5-10 minutes after healing

### 3. Performance Under Constraint Testing

**Objective**: Validate operation under bandwidth/latency constraints

**Test Steps**:
1. Deploy testnet with realistic constraints
2. Generate high transaction load
3. Monitor system performance
4. Identify bottlenecks and limitations

**Expected Results**:
- Graceful degradation under load
- Mobile/satellite nodes maintain connectivity
- Transaction throughput scales with network capacity

### 4. Compliance and Regulatory Testing

**Objective**: Test regulatory compliance features across jurisdictions

**Test Steps**:
1. Enable compliance mode on EU nodes
2. Generate cross-border transactions
3. Monitor compliance reporting
4. Validate data protection requirements

**Expected Results**:
- GDPR compliance maintained for EU data
- Cross-border transactions properly logged
- Regulatory reporting functions correctly

## Chaos Engineering

### Automated Chaos Testing

When chaos mode is enabled (`CHAOS_MODE=true`), the simulation includes:

#### Network Partitions
- **Timing**: After 10 minutes of operation
- **Duration**: 5 minutes
- **Scope**: Isolates APAC region from other AS
- **Recovery**: Gradual healing over 1 minute

#### Node Failures
- **Timing**: After 15 minutes of operation
- **Duration**: 5 minutes
- **Target**: EU research node (non-critical)
- **Recovery**: Automatic restart

#### Performance Degradation
- **Timing**: After 20 minutes of operation
- **Duration**: 10 minutes
- **Target**: Satellite connections (bandwidth reduction)
- **Recovery**: Gradual improvement

### Manual Chaos Injection

```bash
# Inject random packet loss
./scripts/inject_packet_loss.sh 2%

# Simulate DDoS on bootstrap node
./scripts/simulate_ddos.sh bootstrap-na

# Create bandwidth bottleneck
./scripts/limit_bandwidth.sh router-na 50mbit
```

## Performance Expectations

### Transaction Throughput
- **Global testnet**: 50-100 TPS sustained
- **Regional clusters**: 200-500 TPS
- **Single node**: 1000+ TPS

### Latency Expectations
- **Intra-region confirmation**: 10-30 seconds
- **Cross-region confirmation**: 60-120 seconds
- **Satellite confirmation**: 120-300 seconds

### Resource Usage
- **Memory**: 2-4GB per node
- **CPU**: 1-2 cores per node
- **Network**: 1-100Mbps per node (varies by tier)
- **Storage**: 1-10GB per node (depends on duration)

## Troubleshooting

### Common Issues

#### Nodes Not Starting
```bash
# Check container logs
docker logs clab-polytorus-realistic-testnet-bootstrap-na

# Verify network connectivity
docker exec clab-polytorus-realistic-testnet-bootstrap-na ping 10.1.0.1

# Check resource constraints
docker stats
```

#### BGP Not Converging
```bash
# Check FRR status
docker exec clab-polytorus-realistic-testnet-router-na vtysh -c "show ip bgp summary"

# Verify interface configuration
docker exec clab-polytorus-realistic-testnet-router-na ip addr show

# Restart BGP daemon
docker exec clab-polytorus-realistic-testnet-router-na vtysh -c "clear ip bgp *"
```

#### High Latency/Packet Loss
```bash
# Check traffic control configuration
docker exec clab-polytorus-realistic-testnet-miner-apac tc qdisc show

# Reset network impairments
docker exec clab-polytorus-realistic-testnet-miner-apac tc qdisc del dev eth1 root

# Verify routing
docker exec clab-polytorus-realistic-testnet-miner-apac ip route show
```

### Performance Optimization

#### For Development Testing
- Reduce latency values by 50%
- Increase bandwidth limits by 2x
- Disable packet loss simulation
- Use fewer chaos scenarios

#### For Production Simulation
- Use real-world latency measurements
- Implement time-zone based traffic patterns
- Enable full compliance monitoring
- Add economic incentive modeling

## Integration with CI/CD

### Automated Testing

```bash
# Quick smoke test (5 minutes)
./scripts/realistic_testnet_simulation.sh 300 50 5 false

# Full integration test (30 minutes)
./scripts/realistic_testnet_simulation.sh 1800 200 10 true

# Performance benchmark (2 hours)
./scripts/realistic_testnet_simulation.sh 7200 1000 5 false
```

### Test Metrics Collection

The simulation automatically collects:
- Block propagation times
- Transaction confirmation latencies
- Network partition recovery times
- Resource utilization statistics
- BGP convergence metrics

Results are stored in `./data/monitoring/` for analysis.

## Future Enhancements

### Planned Features
1. **Economic modeling**: Transaction fee markets across regions
2. **Regulatory simulation**: Country-specific compliance requirements
3. **Mobile optimization**: 5G and edge computing integration
4. **Quantum readiness**: Post-quantum cryptography testing
5. **Interoperability**: Cross-chain bridge simulation

### Research Applications
- Academic research on distributed consensus
- Economic analysis of global blockchain networks
- Regulatory compliance testing
- Network optimization research
- Security vulnerability assessment

This realistic testnet provides an excellent platform for validating PolyTorus performance under real-world conditions and preparing for global deployment.