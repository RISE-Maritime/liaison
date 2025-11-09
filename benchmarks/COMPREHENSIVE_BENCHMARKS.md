# Comprehensive Benchmark Suite for Liaison FMI

This document describes the complete benchmark suite for evaluating the Rust implementation of Liaison FMI, comparing it against the C++ baseline, and tracking performance over time.

## Table of Contents

1. [Overview](#overview)
2. [Benchmark Categories](#benchmark-categories)
3. [Quick Start](#quick-start)
4. [Memory Profiling](#memory-profiling)
5. [Latency Benchmarks](#latency-benchmarks)
6. [Throughput Benchmarks](#throughput-benchmarks)
7. [Build Performance](#build-performance)
8. [Network Performance](#network-performance)
9. [Results Analysis](#results-analysis)
10. [CI/CD Integration](#cicd-integration)

## Overview

The Liaison FMI benchmark suite provides comprehensive performance measurement across multiple dimensions:

| Benchmark Type | What It Measures | Tools Used |
|----------------|------------------|------------|
| **Memory Profiling** | Heap usage, memory leaks, footprint | Valgrind Massif, Heaptrack |
| **FMI Latency** | Time per FMI function call | Hyperfine, Criterion |
| **Throughput** | Operations per second | Custom scripts |
| **Build Performance** | Compilation time, binary size | Hyperfine, cargo |
| **Network Performance** | Zenoh query round-trip time | Custom profiling |
| **Startup Performance** | Initialization time | Hyperfine |

For detailed performance comparison results, see [PERFORMANCE_COMPARISON.md](../PERFORMANCE_COMPARISON.md).

## Benchmark Categories

### 1. Memory Profiling

**Purpose:** Detect memory leaks, measure peak memory usage, track memory growth

**Documentation:** See [README.md](./README.md) for detailed memory profiling documentation

**Quick Commands:**
```bash
# Full memory profile
./memory_profile.sh

# Quick memory check
./memory_profile.sh --quick

# Compare with baseline
./memory_profile.sh --compare results/baseline/
```

**Key Metrics:**
- Peak memory usage (MB)
- Memory footprint per instance
- Memory leak detection
- Startup memory

### 2. Latency Benchmarks

**Purpose:** Measure time for individual FMI function calls

**Setup:**
```bash
# Build release binaries
cargo build --release

# Create test FMU
./target/release/liaison-server make-fmu tests/BouncingBall.fmu fmus/bench
```

**Run Benchmarks:**
```bash
# Using criterion (micro-benchmarks)
cargo bench --bench fmi_benchmarks

# Using hyperfine (end-to-end)
hyperfine --warmup 3 --runs 10 'fmpy simulate BouncingBallLiaison.fmu --stop-time 1.0'
```

**Key Metrics:**
- fmi3GetFloat64 latency (µs)
- fmi3SetFloat64 latency (µs)
- fmi3DoStep latency (µs)
- Initialization latency (ms)

### 3. Throughput Benchmarks

**Purpose:** Measure operations per second for sustained workloads

**Test Script:**
```bash
cat > benchmarks/scripts/throughput.sh << 'EOF'
#!/bin/bash
./target/release/liaison-server serve tests/BouncingBall.fmu fmus/bench &
SERVER_PID=$!
sleep 2

echo "Running throughput test..."
START=$(date +%s.%N)
fmpy simulate BouncingBallLiaison.fmu --stop-time 100.0 --output-interval 0.01 > /dev/null
END=$(date +%s.%N)

DURATION=$(echo "$END - $START" | bc)
STEPS=10000
THROUGHPUT=$(echo "scale=2; $STEPS / $DURATION" | bc)

echo "Duration: ${DURATION}s"
echo "Steps: $STEPS"
echo "Throughput: ${THROUGHPUT} steps/sec"

kill $SERVER_PID
EOF

chmod +x benchmarks/scripts/throughput.sh
./benchmarks/scripts/throughput.sh
```

**Key Metrics:**
- DoStep operations/second
- Concurrent instance throughput
- Query handling rate

### 4. Build Performance

**Purpose:** Track compilation time and binary size

**Benchmarks:**
```bash
# Clean build time
cargo clean
hyperfine --warmup 1 --runs 5 'cargo build --release'

# Incremental build time
echo "// Benchmark comment" >> liaison-fmi/src/lib.rs
hyperfine --warmup 1 --runs 5 'cargo build --release'
git checkout liaison-fmi/src/lib.rs

# Binary size
ls -lh target/release/liaison-server target/release/libliaisonfmu.so
```

**Key Metrics:**
- Clean build time (seconds)
- Incremental build time (seconds)
- Server binary size (MB)
- Client library size (MB)

### 5. Network Performance

**Purpose:** Measure Zenoh communication overhead

**Test Setup:**
```bash
# Start server with profiling
./target/release/liaison-server serve tests/BouncingBall.fmu fmus/bench &
SERVER_PID=$!
sleep 2

# Measure Zenoh query latency
# (Requires custom profiling tools or Zenoh metrics)
```

**Key Metrics:**
- Query round-trip time (µs)
- Serialization time (µs)
- Deserialization time (µs)
- Network bandwidth usage

### 6. Startup Performance

**Purpose:** Measure initialization and shutdown time

**Benchmarks:**
```bash
# Server startup time
hyperfine --warmup 2 --runs 10 \
  --setup 'rm -f /tmp/liaison_*' \
  --prepare 'sleep 0.5' \
  'timeout 2 ./target/release/liaison-server serve tests/BouncingBall.fmu fmus/bench'

# Client instantiation time
# (Measured via FMI function timing)
```

**Key Metrics:**
- Server startup time (ms)
- FMU extraction time (ms)
- Zenoh session init time (ms)
- Client instantiation time (ms)

## Quick Start

### Prerequisites

```bash
# Ubuntu/Debian
sudo apt-get install valgrind hyperfine build-essential

# Install Python dependencies
pip3 install matplotlib numpy

# Install Rust criterion
cargo install cargo-criterion
```

### Run All Benchmarks

```bash
# 1. Memory profiling
cd benchmarks
./memory_profile.sh --quick

# 2. Build performance
cargo clean
hyperfine 'cargo build --release'

# 3. Latency benchmarks
cargo bench

# 4. Generate reports
cd ..
python3 aggregate_results.py --compare

# 5. View results
cat benchmarks/reports/benchmark_results_*.md
```

### Quick Comparison

```bash
# Run aggregation with sample data
python3 aggregate_results.py

# View HTML report
xdg-open benchmarks/reports/benchmark_results_*.html

# View performance comparison
cat PERFORMANCE_COMPARISON.md
```

## Memory Profiling

See [README.md](./README.md) for comprehensive memory profiling documentation, including:

- Valgrind Massif profiling
- Heaptrack analysis
- Memory leak detection
- Custom memory scenarios
- CI/CD integration

**Quick Reference:**

```bash
# Full memory profile
./memory_profile.sh

# Analyze results
python3 analyze_memory.py results/latest/

# Compare with baseline
./memory_profile.sh --compare results/baseline/
```

## Latency Benchmarks

### Criterion Micro-benchmarks

Create criterion benchmarks in `liaison-fmi/benches/`:

```rust
// liaison-fmi/benches/fmi_benchmarks.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn benchmark_protobuf_encoding(c: &mut Criterion) {
    use liaison_fmi::proto::GetFloat64Input;

    c.bench_function("protobuf_encode_getfloat64", |b| {
        let input = GetFloat64Input {
            instance_index: 0,
            value_references: vec![0, 1, 2, 3, 4],
            n_values: 5,
        };

        b.iter(|| {
            use prost::Message;
            let mut buf = Vec::new();
            input.encode(&mut buf).unwrap();
            black_box(buf);
        });
    });
}

criterion_group!(benches, benchmark_protobuf_encoding);
criterion_main!(benches);
```

**Run:**
```bash
cargo bench --bench fmi_benchmarks
```

**Output:**
```
protobuf_encode_getfloat64
                        time:   [12.234 µs 12.456 µs 12.678 µs]
```

### End-to-End Latency

```bash
# Create benchmark script
cat > benchmarks/scripts/latency_test.sh << 'EOF'
#!/bin/bash
./target/release/liaison-server serve tests/BouncingBall.fmu fmus/bench &
SERVER_PID=$!
sleep 2

# Run simulation
fmpy simulate BouncingBallLiaison.fmu --stop-time 1.0 --output-interval 0.01

kill $SERVER_PID
EOF

chmod +x benchmarks/scripts/latency_test.sh

# Run with hyperfine
hyperfine --warmup 3 --runs 10 'benchmarks/scripts/latency_test.sh'
```

## Throughput Benchmarks

### Co-Simulation Throughput

```bash
#!/bin/bash
# benchmarks/scripts/cosim_throughput.sh

./target/release/liaison-server serve tests/BouncingBall.fmu fmus/bench &
SERVER_PID=$!
sleep 2

# Test different step sizes
for STEP_SIZE in 0.001 0.01 0.1; do
  STEPS=$(echo "10 / $STEP_SIZE" | bc)
  echo "Testing step size: $STEP_SIZE (${STEPS} steps)"

  START=$(date +%s.%N)
  fmpy simulate BouncingBallLiaison.fmu \
    --stop-time 10.0 \
    --output-interval $STEP_SIZE \
    --output /dev/null
  END=$(date +%s.%N)

  DURATION=$(echo "$END - $START" | bc)
  THROUGHPUT=$(echo "scale=2; $STEPS / $DURATION" | bc)

  echo "  Duration: ${DURATION}s"
  echo "  Throughput: ${THROUGHPUT} steps/sec"
  echo ""
done

kill $SERVER_PID
```

### Concurrent Instance Throughput

```bash
#!/bin/bash
# benchmarks/scripts/concurrent_throughput.sh

# Start multiple servers
for i in {1..10}; do
  ./target/release/liaison-server serve tests/BouncingBall.fmu fmus/bench$i &
  echo $! >> /tmp/server_pids.txt
done

sleep 5

# Run simulations in parallel
echo "Running 10 concurrent simulations..."
START=$(date +%s.%N)

for i in {1..10}; do
  fmpy simulate BouncingBallLiaison$i.fmu --stop-time 10.0 --output /dev/null &
done

wait

END=$(date +%s.%N)
DURATION=$(echo "$END - $START" | bc)
TOTAL_STEPS=$(echo "10 * 1000" | bc)  # 10 sims × 1000 steps each
THROUGHPUT=$(echo "scale=2; $TOTAL_STEPS / $DURATION" | bc)

echo "Total throughput: ${THROUGHPUT} steps/sec"

# Cleanup
while read pid; do kill $pid; done < /tmp/server_pids.txt
rm /tmp/server_pids.txt
```

## Build Performance

### Clean Build

```bash
cargo clean
hyperfine --warmup 1 --runs 5 \
  --export-markdown build_clean.md \
  'cargo build --release'
```

### Incremental Build

```bash
# Baseline build
cargo build --release

# Make small change
echo "// Benchmark" >> liaison-fmi/src/lib.rs

# Measure incremental rebuild
hyperfine --warmup 1 --runs 10 \
  --export-markdown build_incremental.md \
  --prepare 'git checkout liaison-fmi/src/lib.rs && echo "// Change" >> liaison-fmi/src/lib.rs' \
  'cargo build --release'

# Cleanup
git checkout liaison-fmi/src/lib.rs
```

### Binary Size Tracking

```bash
#!/bin/bash
# benchmarks/scripts/binary_size.sh

echo "Binary Size Report"
echo "=================="
echo ""

echo "Server Binary:"
ls -lh target/release/liaison-server | awk '{print "  Size: " $5}'
file target/release/liaison-server
echo ""

echo "Client Library:"
ls -lh target/release/libliaisonfmu.so | awk '{print "  Size: " $5}'
file target/release/libliaisonfmu.so
echo ""

echo "Stripped Versions:"
strip -s target/release/liaison-server -o /tmp/liaison-server-stripped
strip -s target/release/libliaisonfmu.so -o /tmp/libliaisonfmu-stripped.so
ls -lh /tmp/liaison-server-stripped | awk '{print "  Server: " $5}'
ls -lh /tmp/libliaisonfmu-stripped.so | awk '{print "  Library: " $5}'
rm /tmp/liaison-server-stripped /tmp/libliaisonfmu-stripped.so
```

## Network Performance

### Zenoh Query Latency

```bash
#!/bin/bash
# benchmarks/scripts/zenoh_latency.sh

./target/release/liaison-server serve tests/BouncingBall.fmu fmus/bench &
SERVER_PID=$!
sleep 2

# Custom profiling tool would go here
# This requires instrumenting the client library
# to measure Zenoh query round-trip time

echo "Zenoh latency profiling requires custom instrumentation"
echo "See PERFORMANCE_COMPARISON.md for estimated latencies"

kill $SERVER_PID
```

### Protobuf Serialization

```rust
// Add to benches/serialization.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};
use liaison_fmi::proto::*;
use prost::Message;

fn benchmark_serialization(c: &mut Criterion) {
    let mut group = c.benchmark_group("protobuf");

    // Small message (10 values)
    group.bench_function("encode_small", |b| {
        let msg = GetFloat64Input {
            instance_index: 0,
            value_references: vec![0; 10],
            n_values: 10,
        };
        b.iter(|| {
            let mut buf = Vec::new();
            msg.encode(&mut buf).unwrap();
            black_box(buf);
        });
    });

    // Large message (1000 values)
    group.bench_function("encode_large", |b| {
        let msg = GetFloat64Input {
            instance_index: 0,
            value_references: vec![0; 1000],
            n_values: 1000,
        };
        b.iter(|| {
            let mut buf = Vec::new();
            msg.encode(&mut buf).unwrap();
            black_box(buf);
        });
    });

    group.finish();
}

criterion_group!(benches, benchmark_serialization);
criterion_main!(benches);
```

## Results Analysis

### Using the Aggregation Script

The `aggregate_results.py` script collects results from all benchmarks:

```bash
# Generate sample data and reports
python3 aggregate_results.py

# Export all formats
python3 aggregate_results.py --format all

# Generate comparison charts
python3 aggregate_results.py --compare
```

**Output Files:**
- `benchmark_results_*.md` - Markdown summary tables
- `benchmark_results_*.csv` - CSV data for spreadsheets
- `benchmark_results_*.json` - JSON data for programmatic access
- `benchmark_results_*.html` - Interactive HTML report
- `comparison_chart_*.png` - Visual comparisons

### Manual Result Collection

To feed real data into the aggregation script:

```json
// benchmarks/results/latency/rust_latency.json
[
  {
    "name": "fmi3GetFloat64_10",
    "implementation": "rust",
    "metric": "latency",
    "value": 145.2,
    "unit": "µs",
    "timestamp": "2025-11-09T12:34:56",
    "metadata": {
      "platform": "linux",
      "cpu_cores": 8
    }
  }
]
```

Then run:
```bash
python3 aggregate_results.py
```

### Interpreting Results

See [PERFORMANCE_COMPARISON.md](../PERFORMANCE_COMPARISON.md) for:
- Performance verdict and recommendations
- Detailed analysis of each metric
- Trade-off analysis
- Optimization guide

**Quick Guidelines:**

| Metric | Good | Acceptable | Investigate |
|--------|------|------------|-------------|
| Latency difference | < 5% | 5-10% | > 10% |
| Memory difference | < 5% | 5-10% | > 10% |
| Throughput difference | < 5% | 5-10% | > 10% |
| Build time difference | < 2x | 2-3x | > 3x |

## CI/CD Integration

### GitHub Actions Workflow

```yaml
# .github/workflows/benchmarks.yml
name: Performance Benchmarks

on:
  push:
    branches: [main]
  pull_request:
  schedule:
    - cron: '0 2 * * 0'  # Weekly on Sunday

jobs:
  benchmark:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v4

      - name: Install Rust
        uses: actions-rust-lang/setup-rust-toolchain@v1

      - name: Install dependencies
        run: |
          sudo apt-get update
          sudo apt-get install -y valgrind hyperfine
          pip3 install matplotlib numpy

      - name: Build release
        run: cargo build --release

      - name: Run memory profiling
        run: |
          cd benchmarks
          ./memory_profile.sh --quick

      - name: Run build benchmarks
        run: |
          cargo clean
          hyperfine --warmup 1 --runs 3 'cargo build --release'

      - name: Generate reports
        run: |
          python3 aggregate_results.py --format all

      - name: Upload results
        uses: actions/upload-artifact@v3
        with:
          name: benchmark-results
          path: |
            benchmarks/results/
            benchmarks/reports/

      - name: Comment PR with results
        if: github.event_name == 'pull_request'
        uses: actions/github-script@v6
        with:
          script: |
            const fs = require('fs');
            const report = fs.readFileSync('benchmarks/reports/benchmark_results_latest.md', 'utf8');
            github.rest.issues.createComment({
              issue_number: context.issue.number,
              owner: context.repo.owner,
              repo: context.repo.repo,
              body: '## Benchmark Results\n\n' + report
            });
```

### Regression Detection

```bash
#!/bin/bash
# benchmarks/scripts/check_regression.sh

BASELINE="benchmarks/results/baseline.json"
CURRENT="benchmarks/results/current.json"

python3 << EOF
import json
import sys

with open("$BASELINE") as f:
    baseline = json.load(f)
with open("$CURRENT") as f:
    current = json.load(f)

regressions = []
for bench_name in baseline:
    base_val = baseline[bench_name]["mean"]
    curr_val = current.get(bench_name, {}).get("mean", 0)

    # Check for >10% regression
    if curr_val > base_val * 1.10:
        pct = ((curr_val / base_val) - 1) * 100
        regressions.append(f"{bench_name}: +{pct:.1f}%")

if regressions:
    print("❌ Performance regressions detected:")
    for reg in regressions:
        print(f"  - {reg}")
    sys.exit(1)
else:
    print("✅ No regressions")
    sys.exit(0)
EOF
```

## Best Practices

### 1. Consistent Environment

- Use dedicated hardware for benchmarking
- Disable CPU frequency scaling
- Close background applications
- Use the same OS and kernel version
- Document hardware specifications

### 2. Statistical Rigor

- Run multiple iterations (10+ for macro, 1000+ for micro)
- Report median and percentiles, not just mean
- Calculate standard deviation
- Use warm-up iterations
- Detect and handle outliers

### 3. Version Control

- Tag baseline versions
- Track benchmark results in git
- Document changes that affect performance
- Maintain historical data

### 4. Reproducibility

- Document exact commands used
- Record environment details
- Use fixed seeds for randomness
- Share raw data and scripts

## Troubleshooting

### High Variance

**Problem:** Results vary significantly between runs

**Solutions:**
```bash
# Disable CPU frequency scaling
sudo cpupower frequency-set -g performance

# Pin to specific cores
taskset -c 0-3 ./target/release/liaison-server serve ...

# Increase iterations
hyperfine --runs 50 ...
```

### Valgrind Slowness

**Problem:** Memory profiling takes too long

**Solutions:**
```bash
# Use quick mode
./memory_profile.sh --quick

# Profile specific components
./memory_profile.sh --server-only

# Use heaptrack instead (faster)
heaptrack ./target/release/liaison-server serve ...
```

### Missing Baseline Data

**Problem:** No baseline to compare against

**Solutions:**
```bash
# Create baseline from current version
mkdir -p benchmarks/results/baseline
python3 aggregate_results.py
cp benchmarks/reports/benchmark_results_*.json benchmarks/results/baseline/baseline.json
```

## Resources

- [PERFORMANCE_COMPARISON.md](../PERFORMANCE_COMPARISON.md) - Detailed analysis
- [Memory Profiling README](./README.md) - Memory-specific documentation
- [Criterion.rs Guide](https://bheisler.github.io/criterion.rs/book/)
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)

## Contributing

To contribute benchmarks:

1. Add benchmark code/scripts
2. Document methodology
3. Run benchmarks and collect data
4. Update this documentation
5. Submit PR with results

---

**Last Updated:** 2025-11-09
**Maintainer:** Liaison Development Team
