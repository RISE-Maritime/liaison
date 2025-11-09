# Liaison FMI Network Benchmarks

This directory contains comprehensive network performance benchmarks for the Liaison FMI Rust implementation. The benchmarks measure latency, throughput, and system capacity for FMI operations over Zenoh.

## Overview

The benchmark suite consists of three main components:

1. **Latency Test** (`latency_test.rs`) - Measures round-trip time for individual FMI operations
2. **Throughput Test** (`throughput_test.rs`) - Measures requests per second and data transfer rates
3. **Comparison Script** (`compare_network.sh`) - Automates running benchmarks and generating comparison reports

## Prerequisites

### Build Requirements

- Rust toolchain (1.70 or later)
- Cargo build system
- Zenoh runtime (automatically handled by dependencies)

### Test Requirements

- A valid FMU file (FMI 3.0 compatible)
- Sufficient system resources for concurrent testing
- Network connectivity (localhost is sufficient)

## Building the Benchmarks

Build the benchmark binaries in release mode for accurate performance measurements:

```bash
# Build both latency and throughput tests
cargo build --release --bin latency_test --bin throughput_test

# Or build from workspace root
cd /path/to/liaison
cargo build --release
```

The compiled binaries will be located at:
- `target/release/latency_test`
- `target/release/throughput_test`

## Running the Benchmarks

### Latency Test

The latency test measures round-trip time for FMI function calls.

#### Basic Usage

```bash
./target/release/latency_test <fmu_path> <responder_id>

# Example
./target/release/latency_test ./BouncingBallLiaison.fmu test_responder
```

#### Advanced Options

```bash
# Run with custom iterations
./target/release/latency_test ./BouncingBall.fmu test_responder --iterations 1000

# Run specific test category
./target/release/latency_test ./BouncingBall.fmu test_responder --test lifecycle
./target/release/latency_test ./BouncingBall.fmu test_responder --test data
./target/release/latency_test ./BouncingBall.fmu test_responder --test simulation

# Generate CSV output for data analysis
./target/release/latency_test ./BouncingBall.fmu test_responder --format csv

# Generate JSON output for programmatic processing
./target/release/latency_test ./BouncingBall.fmu test_responder --format json

# Use existing server (don't start a new one)
./target/release/latency_test ./BouncingBall.fmu test_responder --no-server

# Custom warmup iterations
./target/release/latency_test ./BouncingBall.fmu test_responder --warmup 20
```

#### Test Categories

**Lifecycle Operations:**
- `InstantiateCoSimulation` - FMU instantiation
- `EnterInitializationMode` - Initialization phase entry
- `ExitInitializationMode` - Initialization phase exit

**Data Operations:**
- `GetFloat64_N_values` - Get operations with varying payload sizes (1, 10, 100, 1000 values)
- `SetFloat64_N_values` - Set operations with varying payload sizes
- `GetInt32`, `GetBoolean`, `GetString` - Different data types

**Simulation Operations:**
- `DoStep_Xs` - Simulation step with varying step sizes (0.001s, 0.01s, 0.1s, 1.0s)

#### Output Metrics

For each test, the following statistics are collected:

- **Samples** - Number of successful measurements
- **Min** - Minimum latency (ms)
- **Max** - Maximum latency (ms)
- **Mean** - Average latency (ms)
- **Median (p50)** - 50th percentile (ms)
- **p95** - 95th percentile (ms)
- **p99** - 99th percentile (ms)
- **Std Dev** - Standard deviation (ms)

### Throughput Test

The throughput test measures system capacity and sustained performance.

#### Basic Usage

```bash
./target/release/throughput_test <fmu_path> <responder_id>

# Example
./target/release/throughput_test ./BouncingBallLiaison.fmu test_responder
```

#### Advanced Options

```bash
# Run with custom duration
./target/release/throughput_test ./BouncingBall.fmu test_responder --duration 60

# Test with multiple concurrent clients
./target/release/throughput_test ./BouncingBall.fmu test_responder --clients 10

# Run specific test category
./target/release/throughput_test ./BouncingBall.fmu test_responder --test rps
./target/release/throughput_test ./BouncingBall.fmu test_responder --test concurrent
./target/release/throughput_test ./BouncingBall.fmu test_responder --test transfer
./target/release/throughput_test ./BouncingBall.fmu test_responder --test sustained

# Generate CSV output
./target/release/throughput_test ./BouncingBall.fmu test_responder --format csv

# Use existing server
./target/release/throughput_test ./BouncingBall.fmu test_responder --no-server
```

#### Test Categories

**Request Rate (RPS):**
- `DoStep_RPS` - Requests per second for simulation steps
- `GetFloat64_RPS` - Requests per second for read operations
- `SetFloat64_RPS` - Requests per second for write operations

**Concurrent Clients:**
- `DoStep_Nclients` - Performance with 1, 2, 4, 8, 16 concurrent clients

**Data Transfer:**
- `Transfer_Nvalues` - Throughput with varying payload sizes (10, 100, 1000, 10000 values)

**Sustained Load:**
- `Sustained_Mixed_Load` - Realistic mixed workload (60% DoStep, 30% Get, 10% Set)

#### Output Metrics

For each test, the following metrics are collected:

- **Duration** - Test duration (seconds)
- **Total Requests** - Total number of requests attempted
- **Successful Requests** - Number of successful requests
- **Failed Requests** - Number of failed requests
- **Success Rate** - Percentage of successful requests
- **Requests/sec** - Average requests per second
- **Throughput** - Data throughput (MB/s)
- **Avg Response** - Average response time (ms)
- **Total Data** - Total data transferred (MB)

### Comparison Script

The comparison script automates running benchmarks and generating reports.

#### Basic Usage

```bash
./benchmarks/network/compare_network.sh <fmu_path> <responder_id>

# Example
./benchmarks/network/compare_network.sh ./BouncingBallLiaison.fmu test_responder
```

#### Advanced Options

```bash
# Custom iterations and duration
./benchmarks/network/compare_network.sh ./BouncingBall.fmu test_responder \
    --iterations 1000 \
    --duration 60

# Skip C++ benchmarks (Rust only)
./benchmarks/network/compare_network.sh ./BouncingBall.fmu test_responder --skip-cpp

# Custom output directory
./benchmarks/network/compare_network.sh ./BouncingBall.fmu test_responder \
    --output-dir ./my_benchmarks

# Generate JSON output
./benchmarks/network/compare_network.sh ./BouncingBall.fmu test_responder --format json
```

#### Output Files

The script generates timestamped output files:

```
results/
├── benchmark_20250109_120000_rust_latency.csv
├── benchmark_20250109_120000_rust_throughput.csv
├── benchmark_20250109_120000_cpp_latency.csv (if available)
├── benchmark_20250109_120000_cpp_throughput.csv (if available)
├── benchmark_20250109_120000_comparison.csv
└── benchmark_20250109_120000_comparison_report.txt
```

## Interpreting Results

### Latency Benchmarks

**Good Performance Indicators:**
- p95 < 10ms for most operations
- p99 < 50ms for most operations
- Low standard deviation (consistent performance)
- Min and median close together (few outliers)

**Performance Issues:**
- p99 > 100ms (high tail latency)
- Large standard deviation (inconsistent performance)
- Significant difference between min and p50 (many outliers)

### Throughput Benchmarks

**Good Performance Indicators:**
- High requests/second (>1000 for simple operations)
- Success rate close to 100%
- Linear scaling with concurrent clients (up to hardware limits)
- High throughput (>100 MB/s for large payloads)

**Performance Issues:**
- Low requests/second (<100)
- Success rate below 95%
- No performance improvement with concurrency
- High average response time

### Common Patterns

**Network Latency:**
- Baseline latency (min value) indicates network overhead
- Typical range: 0.5-5ms for localhost

**Payload Size Impact:**
- Larger payloads increase latency and reduce throughput
- Should scale approximately linearly with payload size

**Concurrency:**
- Performance should improve with concurrency up to CPU core count
- Degradation beyond this indicates resource contention

## Troubleshooting

### Server Fails to Start

**Problem:** Benchmark reports server startup failure

**Solutions:**
1. Ensure liaison server binary is built: `cargo build --release`
2. Check FMU file exists and is readable
3. Verify port 7447 (Zenoh default) is not in use
4. Check system logs for errors

### Low Performance

**Problem:** Benchmarks show unexpectedly low performance

**Solutions:**
1. Ensure release build is used (not debug)
2. Close unnecessary applications
3. Check system resources (CPU, memory)
4. Verify network configuration (localhost should be fast)
5. Increase warmup iterations to allow JIT optimization

### High Variance

**Problem:** Large standard deviation in results

**Solutions:**
1. Increase number of iterations for better statistical significance
2. Close background applications
3. Disable CPU frequency scaling (performance governor)
4. Run on dedicated hardware if possible

### Connection Errors

**Problem:** Zenoh connection failures

**Solutions:**
1. Check if server is running: `ps aux | grep liaison`
2. Verify responder ID matches between server and client
3. Check firewall settings
4. Ensure Zenoh is properly configured

## Visualization

### CSV Output

CSV files can be imported into spreadsheet software or analyzed with Python:

```python
import pandas as pd
import matplotlib.pyplot as plt

# Load latency data
df = pd.read_csv('benchmark_TIMESTAMP_rust_latency.csv')

# Plot p95 latencies
plt.figure(figsize=(12, 6))
plt.bar(df['test_name'], df['p95_ms'])
plt.xticks(rotation=45, ha='right')
plt.ylabel('p95 Latency (ms)')
plt.title('Latency Benchmark Results (p95)')
plt.tight_layout()
plt.savefig('latency_p95.png')

# Load throughput data
df = pd.read_csv('benchmark_TIMESTAMP_rust_throughput.csv')

# Plot requests per second
plt.figure(figsize=(12, 6))
plt.bar(df['test_name'], df['requests_per_second'])
plt.xticks(rotation=45, ha='right')
plt.ylabel('Requests/Second')
plt.title('Throughput Benchmark Results')
plt.tight_layout()
plt.savefig('throughput_rps.png')
```

### JSON Output

JSON files can be processed with `jq` or programming languages:

```bash
# Extract p95 latencies
cat benchmark_TIMESTAMP_rust_latency.json | jq '.[] | {test_name, p95_ms}'

# Find slowest operation
cat benchmark_TIMESTAMP_rust_latency.json | jq 'max_by(.p99_ms) | {test_name, p99_ms}'

# Calculate average throughput
cat benchmark_TIMESTAMP_rust_throughput.json | jq '[.[] | .requests_per_second] | add / length'
```

### Comparison Analysis

To compare Rust and C++ implementations:

```bash
# Run comparison
./benchmarks/network/compare_network.sh ./BouncingBall.fmu test_responder --format csv

# Process comparison CSV
python3 << EOF
import pandas as pd
import matplotlib.pyplot as plt

df = pd.read_csv('results/benchmark_TIMESTAMP_comparison.csv')

# Pivot for comparison
pivot = df.pivot_table(
    values='value',
    index='test_name',
    columns=['implementation', 'metric']
)

# Plot comparison
pivot.plot(kind='bar', figsize=(14, 7))
plt.xticks(rotation=45, ha='right')
plt.ylabel('Value')
plt.title('Rust vs C++ Performance Comparison')
plt.tight_layout()
plt.savefig('comparison.png')
EOF
```

## Performance Tuning

### System Configuration

For best benchmark results:

1. **CPU Governor:** Set to performance mode
   ```bash
   echo performance | sudo tee /sys/devices/system/cpu/cpu*/cpufreq/scaling_governor
   ```

2. **Network Tuning:** For network benchmarks
   ```bash
   sudo sysctl -w net.core.rmem_max=26214400
   sudo sysctl -w net.core.wmem_max=26214400
   ```

3. **Process Priority:** Run with higher priority
   ```bash
   sudo nice -n -10 ./target/release/latency_test ...
   ```

### Zenoh Configuration

Create a custom Zenoh configuration file for benchmarking:

```json
{
  "mode": "peer",
  "connect": {
    "endpoints": ["tcp/127.0.0.1:7447"]
  },
  "scouting": {
    "multicast": {
      "enabled": false
    }
  }
}
```

Use with benchmarks:
```bash
export ZENOH_CONFIG_FILE=/path/to/config.json
./target/release/latency_test ...
```

## Continuous Integration

To integrate benchmarks into CI/CD:

```yaml
# Example GitHub Actions workflow
name: Network Benchmarks

on:
  push:
    branches: [main]
  pull_request:

jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2

      - name: Build benchmarks
        run: cargo build --release --bin latency_test --bin throughput_test

      - name: Run benchmarks
        run: |
          ./benchmarks/network/compare_network.sh \
            ./BouncingBallLiaison.fmu \
            test_responder \
            --iterations 100 \
            --duration 10 \
            --format csv \
            --skip-cpp

      - name: Upload results
        uses: actions/upload-artifact@v2
        with:
          name: benchmark-results
          path: results/
```

## Best Practices

1. **Consistent Environment:** Run benchmarks on the same hardware for comparisons
2. **Multiple Runs:** Run benchmarks multiple times and average results
3. **Warmup:** Allow sufficient warmup iterations for JIT optimization
4. **Isolation:** Close unnecessary applications during benchmarking
5. **Documentation:** Record system configuration with results
6. **Version Control:** Track benchmark results over time
7. **Realistic Workloads:** Test with representative FMUs and usage patterns

## References

- [FMI 3.0 Standard](https://fmi-standard.org/)
- [Zenoh Documentation](https://zenoh.io/docs/)
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [Liaison FMI Project](https://github.com/RISE-Maritime/liaison)

## Support

For issues or questions:

1. Check the troubleshooting section above
2. Review the main project README
3. Open an issue on GitHub
4. Contact the development team

## License

This benchmark suite is part of the Liaison FMI project and is licensed under the MIT License.
