# Liaison FMI Benchmarks - Usage Guide

This guide provides step-by-step instructions for running the startup and initialization benchmarks.

## Prerequisites

Before running benchmarks, ensure you have:

1. **Rust toolchain installed**
   ```bash
   rustc --version
   cargo --version
   ```

2. **Build the project**
   ```bash
   cd /workspace
   cargo build --release --bin liaison
   ```

3. **FMU file available**
   - Default: `BouncingBallLiaison.fmu` in workspace root
   - Or specify custom FMU with `--fmu` option

4. **System tools** (usually pre-installed):
   - `bash`, `strip`, `size`, `readelf`, `nm`
   - `awk`, `grep`, `stat`

## Quick Start (3 Steps)

### Step 1: Build Release Binary

```bash
cd /workspace
cargo build --release --bin liaison
```

This creates:
- `/workspace/target/release/liaison` (server binary)
- `/workspace/target/release/libliaisonfmu.so` (FMU library)

### Step 2: Run All Benchmarks

```bash
cd /workspace/benchmarks/startup
./run_all_benchmarks.sh
```

This runs:
1. Binary size analysis
2. Startup performance tests
3. Initialization performance tests
4. Generates comprehensive report

### Step 3: View Results

```bash
# List results directories
ls -la benchmark_results_*/

# View summary report
cat benchmark_results_*/BENCHMARK_SUMMARY.md
```

## Individual Benchmark Usage

### Binary Size Analysis

Compares Rust vs C++ binary sizes and analyzes ELF sections.

```bash
cd /workspace/benchmarks/startup
./binary_analysis.sh
```

**Output:**
- Console output with detailed analysis
- `binary_analysis_report.txt` - Summary report

**Example output:**
```
=== Rust Release Binary (liaison) ===

Total Size: 12.34 MB
Debug Info: Stripped

Section Breakdown:
  .text (code):    8.50 MB
  .data:           0.50 MB
  .bss:            0.10 MB

=== Comparison: Rust Lib vs C++ Lib ===

Rust Lib:            10.00 MB
C++ Lib:              8.00 MB

Rust Lib is 2.00 MB larger (25.0%)
```

### Startup Performance Benchmark

Measures server startup, FMU loading, and request latency.

```bash
cd /workspace/benchmarks/startup
./run_startup_bench.sh
```

**Options:**
```bash
./run_startup_bench.sh \
  --binary target/release/liaison \
  --fmu BouncingBallLiaison.fmu \
  --iterations 10 \
  --no-build \
  --no-analysis
```

**What it measures:**
- Total startup time (process start to ready)
- FMU loading time
- Zenoh session initialization
- Cold start request latency
- Warm request latency
- Binary size
- Memory footprint

**Output:**
- Console report
- `startup_results.json` - JSON results

**Example JSON:**
```json
{
  "total_startup_ms": 1234.56,
  "fmu_loading_ms": 123.45,
  "zenoh_init_ms": 234.56,
  "first_request_ms": 50.00,
  "warm_request_ms": 5.00,
  "binary_size_mb": 12.34,
  "startup_memory_kb": 50000
}
```

### Initialization Performance Benchmark

Measures FMU instantiation and initialization steps.

```bash
cd /workspace/benchmarks/startup
./run_init_bench.sh
```

**Options:**
```bash
./run_init_bench.sh \
  --fmu BouncingBallLiaison.fmu \
  --iterations 100
```

**What it measures:**
- FMU instantiation time
- Configuration loading (XML parsing)
- Variable initialization
- Setup experiment
- Enter/exit initialization mode
- Resource cleanup
- Percentiles (P50, P95, P99)

**Output:**
- Console report with statistics
- `init_results.json` - JSON results

**Example JSON:**
```json
{
  "fmu_category": "Small",
  "fmu_size_mb": 5.50,
  "variable_count": 42,
  "average_metrics": {
    "fmu_instantiation_ms": 0.50,
    "config_loading_ms": 12.34,
    "variable_init_ms": 0.10,
    "setup_experiment_ms": 0.10,
    "enter_init_mode_ms": 0.05,
    "exit_init_mode_ms": 0.05,
    "cleanup_ms": 0.20,
    "total_init_ms": 13.34
  },
  "percentiles": {
    "p50_ms": 13.50,
    "p95_ms": 15.20,
    "p99_ms": 16.80
  },
  "iterations": 100
}
```

## Advanced Usage

### Testing Multiple FMU Sizes

Create a script to test different FMU categories:

```bash
#!/bin/bash

# Test small FMU (fast, many iterations)
./run_init_bench.sh --fmu small.fmu --iterations 1000
mv init_results.json init_results_small.json

# Test medium FMU (moderate iterations)
./run_init_bench.sh --fmu medium.fmu --iterations 100
mv init_results.json init_results_medium.json

# Test large FMU (few iterations)
./run_init_bench.sh --fmu large.fmu --iterations 10
mv init_results.json init_results_large.json
```

### Custom Binary Paths

Test different build configurations:

```bash
# Test debug build
RUST_BINARY=target/debug/liaison \
RUST_LIB=target/debug/libliaisonfmu.so \
./binary_analysis.sh

# Test size-optimized build
RUST_BINARY=target/release-size/liaison \
./binary_analysis.sh
```

### Regression Testing

Compare against baseline:

```bash
# Run baseline benchmarks
./run_all_benchmarks.sh
mv benchmark_results_* benchmark_baseline/

# Make changes, rebuild
# ... your changes ...
cargo build --release

# Run new benchmarks
./run_all_benchmarks.sh

# Check for regressions
./check_regression.py benchmark_baseline/ benchmark_results_*/
```

Exit codes:
- `0` - No regressions
- `1` - Regressions detected

### Continuous Integration

Example CI configuration:

```bash
#!/bin/bash
# ci_benchmark.sh

set -e

# Build
cargo build --release

# Run benchmarks
cd benchmarks/startup
./run_all_benchmarks.sh

# Check regressions
if [ -d "../../benchmark_baseline" ]; then
    ./check_regression.py ../../benchmark_baseline/ benchmark_results_*/
fi

# Archive results
tar czf benchmark_results.tar.gz benchmark_results_*/
```

## Interpreting Results

### Startup Performance

**Good performance:**
- Total startup < 2 seconds
- FMU loading < 500 ms
- Zenoh init < 500 ms
- Cold start < 100 ms
- Warm start < 10 ms

**Concerning:**
- Startup > 5 seconds (investigate why)
- Large gap between cold/warm (caching issues)
- Memory usage > 100 MB (memory leak?)

### Initialization Performance

**Good performance:**
- Small FMUs: Total init < 20 ms
- Medium FMUs: Total init < 100 ms
- Large FMUs: Total init < 500 ms
- P95/P50 ratio < 1.5 (consistent performance)

**Concerning:**
- P99 >> P95 (outliers, investigate)
- Config loading dominates (optimize XML parsing)
- Cleanup time high (resource leak?)

### Binary Size

**Typical sizes:**
- Rust release binary: 10-15 MB (stripped)
- Rust debug binary: 100-200 MB (with symbols)
- C++ library: 8-12 MB

**Optimization impact:**
- `strip=true`: Reduces 50-80%
- `opt-level='z'`: Reduces 10-20% (vs 'opt-level=3')
- `lto=true`: Reduces 5-15%
- `codegen-units=1`: Reduces 3-8%

## Troubleshooting

### Issue: Binary not found

```
Error: Binary not found at target/release/liaison
```

**Solution:**
```bash
cd /workspace
cargo build --release --bin liaison
```

### Issue: FMU not found

```
Error: FMU not found at BouncingBallLiaison.fmu
```

**Solution:**
```bash
# Use custom FMU path
./run_startup_bench.sh --fmu /path/to/your.fmu
```

Or copy FMU to workspace root:
```bash
cp /path/to/your.fmu /workspace/BouncingBallLiaison.fmu
```

### Issue: Permission denied

```
bash: ./run_all_benchmarks.sh: Permission denied
```

**Solution:**
```bash
chmod +x *.sh *.py
```

### Issue: Benchmark times out

**Solution:**
Reduce iterations or increase timeout:
```bash
./run_startup_bench.sh --iterations 5
```

### Issue: Inconsistent results

**Causes:**
- System load (other processes running)
- Thermal throttling (CPU overheating)
- Power management (laptop on battery)

**Solutions:**
```bash
# Close other applications
# Run multiple times, take median
# Use dedicated benchmark machine
# Disable CPU frequency scaling:
sudo cpupower frequency-set --governor performance
```

### Issue: Compilation errors

**For standalone benchmarks:**
```bash
cd benchmarks/startup

# Compile individually
rustc startup_bench.rs -o startup_bench -O
rustc init_bench.rs -o init_bench -O

# Or use Cargo
cargo build --release --bin startup_bench
cargo build --release --bin init_bench
```

## Performance Tips

### Optimize for Size

Edit `/workspace/Cargo.toml`:

```toml
[profile.release]
opt-level = 'z'      # Optimize for size
lto = true
codegen-units = 1
strip = true
panic = 'abort'      # Remove panic unwinding
```

Rebuild:
```bash
cargo build --release
```

Expected reduction: 20-40% smaller binary.

### Optimize for Startup Speed

1. **Use LTO** (already enabled)
2. **Reduce dependencies** (fewer crates)
3. **Lazy initialization** (defer work)
4. **Parallel loading** (load concurrently)
5. **Binary caching** (preload libraries)

### Profile Performance

```bash
# CPU profiling
perf record -g target/release/liaison serve BouncingBallLiaison.fmu test_server
perf report

# Flamegraph
cargo install flamegraph
cargo flamegraph --bin liaison -- serve BouncingBallLiaison.fmu test_server

# Memory profiling
valgrind --tool=massif target/release/liaison serve BouncingBallLiaison.fmu test_server
```

## Best Practices

1. **Baseline First**
   - Run benchmarks on clean main branch
   - Save as baseline for comparison
   - Use consistent hardware

2. **Multiple Iterations**
   - Startup: 10-20 iterations
   - Init: 100-200 iterations
   - Discard first run (warmup)

3. **Controlled Environment**
   - Close other applications
   - Disable background services
   - Use AC power (laptops)
   - Same time of day (temperature)

4. **Document Changes**
   - Note hardware specs
   - Record software versions
   - Track optimization changes
   - Explain regressions

5. **Automate**
   - Run in CI/CD
   - Track over time
   - Alert on regressions
   - Archive results

## Example Workflow

Complete example from start to finish:

```bash
# 1. Setup
cd /workspace
cargo build --release

# 2. Run baseline
cd benchmarks/startup
./run_all_benchmarks.sh
mv benchmark_results_* ../../benchmark_baseline

# 3. Make changes
cd /workspace
# ... edit code ...
cargo build --release

# 4. Run new benchmarks
cd benchmarks/startup
./run_all_benchmarks.sh

# 5. Compare
./check_regression.py ../../benchmark_baseline/ benchmark_results_*/

# 6. Analyze if regression
cat benchmark_results_*/BENCHMARK_SUMMARY.md

# 7. Archive
tar czf benchmarks_$(date +%Y%m%d).tar.gz benchmark_results_*/
```

## Next Steps

- Read [README.md](README.md) for detailed documentation
- Check [../../MIGRATION_GUIDE.md](../../MIGRATION_GUIDE.md) for optimization tips
- See [../../tests/README.md](../../tests/README.md) for integration tests
- Review results and optimize hotspots
- Set up CI/CD for automated benchmarking

## Support

For issues or questions:
1. Check troubleshooting section above
2. Review script comments for details
3. Examine JSON output for raw data
4. Profile with `perf` or `flamegraph`
5. Open GitHub issue with benchmark results
