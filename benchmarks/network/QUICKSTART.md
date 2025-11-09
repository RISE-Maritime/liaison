# Network Benchmarks Quick Start Guide

This guide will help you get started with the Liaison FMI network benchmarks in just a few minutes.

## Quick Start (5 minutes)

### Step 1: Build the Benchmarks

```bash
# From workspace root
cd /workspace
cargo build --release --manifest-path benchmarks/network/Cargo.toml
```

Or build from the benchmark directory:

```bash
cd /workspace/benchmarks/network
cargo build --release
```

### Step 2: Run Latency Benchmark

```bash
# Using the built-in test FMU
./target/release/latency_test ./BouncingBallLiaison.fmu test_responder
```

Expected output:
```
Starting Liaison FMI Network Latency Benchmarks
FMU: ./BouncingBallLiaison.fmu
Responder ID: test_responder
Iterations: 100
...
================================================================================
Latency Statistics: DoStep_0.01s
================================================================================
Samples:        100
Min:              2.134 ms
Max:             15.678 ms
Mean:             3.245 ms
Median (p50):     3.120 ms
p95:              5.432 ms
p99:              8.765 ms
Std Dev:          1.234 ms
================================================================================
```

### Step 3: Run Throughput Benchmark

```bash
./target/release/throughput_test ./BouncingBallLiaison.fmu test_responder
```

Expected output:
```
Starting Liaison FMI Network Throughput Benchmarks
FMU: ./BouncingBallLiaison.fmu
Responder ID: test_responder
Duration: 10 seconds
...
================================================================================
Throughput Statistics: DoStep_RPS
================================================================================
Duration:           10.00 s
Total Requests:     12543
Successful:         12543
Failed:             0
Success Rate:       100.00%
Requests/sec:       1254.30
Throughput:         0.245 MB/s
Avg Response:       3.245 ms
Total Data:         2.45 MB
================================================================================
```

### Step 4: Run Comparison (Optional)

```bash
./benchmarks/network/compare_network.sh ./BouncingBallLiaison.fmu test_responder
```

This will run both benchmarks and generate a comparison report in the `results/` directory.

## Common Use Cases

### Quick Performance Check

Run a fast benchmark with fewer iterations:

```bash
./target/release/latency_test ./BouncingBallLiaison.fmu test_responder --iterations 10
./target/release/throughput_test ./BouncingBallLiaison.fmu test_responder --duration 5
```

### Detailed Analysis

Run comprehensive benchmarks with more iterations:

```bash
./target/release/latency_test ./BouncingBallLiaison.fmu test_responder \
    --iterations 1000 \
    --format csv > latency_results.csv

./target/release/throughput_test ./BouncingBallLiaison.fmu test_responder \
    --duration 60 \
    --format csv > throughput_results.csv
```

### Test Specific Operations

Test only lifecycle operations:

```bash
./target/release/latency_test ./BouncingBallLiaison.fmu test_responder --test lifecycle
```

Test only data transfer rates:

```bash
./target/release/throughput_test ./BouncingBallLiaison.fmu test_responder --test transfer
```

### Concurrent Load Testing

Test with multiple concurrent clients:

```bash
./target/release/throughput_test ./BouncingBallLiaison.fmu test_responder \
    --clients 10 \
    --duration 30
```

## Understanding the Output

### Latency Metrics

- **Mean**: Average latency - should be low (< 10ms for localhost)
- **p95**: 95% of requests are faster than this - important for consistency
- **p99**: 99% of requests are faster than this - identifies worst-case performance
- **Std Dev**: Lower is better - indicates consistent performance

### Throughput Metrics

- **Requests/sec**: Higher is better - indicates system capacity
- **Success Rate**: Should be close to 100%
- **Throughput (MB/s)**: Data transfer rate
- **Avg Response**: Similar to latency mean

## Troubleshooting

### "Liaison binary not found"

Build the main liaison server:
```bash
cargo build --release
```

### "Connection refused" or timeout errors

The benchmark will start a server automatically. If you see connection errors:
1. Make sure no other liaison server is running
2. Check that the FMU file exists and is valid
3. Try with `--no-server` if you're running a server separately

### "No successful samples collected"

This usually means the FMU operations are failing:
1. Verify the FMU file is valid
2. Check the server logs
3. Try with a different FMU

### Poor performance

1. Make sure you're using release build (not debug)
2. Close unnecessary applications
3. Run on a system with adequate resources

## Next Steps

- Read the full [README.md](README.md) for detailed documentation
- Experiment with different test categories
- Try different output formats (CSV, JSON)
- Compare results across different FMU files
- Set up continuous benchmarking in CI/CD

## Example Workflows

### Development Workflow

```bash
# 1. Make code changes
# 2. Rebuild
cargo build --release

# 3. Quick smoke test
./target/release/latency_test ./test.fmu dev_responder --iterations 10

# 4. If good, run full benchmark
./benchmarks/network/compare_network.sh ./test.fmu dev_responder
```

### Performance Investigation

```bash
# 1. Identify slow operation
./target/release/latency_test ./test.fmu test --format csv > before.csv

# 2. Make optimization
# ... code changes ...

# 3. Rebuild and test
cargo build --release
./target/release/latency_test ./test.fmu test --format csv > after.csv

# 4. Compare results
diff before.csv after.csv
```

### Capacity Planning

```bash
# Test different concurrency levels
for clients in 1 2 4 8 16 32; do
    echo "Testing with $clients clients..."
    ./target/release/throughput_test ./prod.fmu test \
        --clients $clients \
        --duration 30 \
        --format csv >> capacity_results.csv
done
```

## Tips

1. **Warmup matters**: Always use warmup iterations for accurate results
2. **Multiple runs**: Run benchmarks multiple times and average results
3. **System state**: Ensure consistent system state between runs
4. **Documentation**: Keep notes of system configuration and results
5. **Version control**: Track benchmark results over time

## Getting Help

- Check the [README.md](README.md) for detailed documentation
- Review the main project documentation
- Open an issue on GitHub
- Contact the development team

Happy benchmarking!
