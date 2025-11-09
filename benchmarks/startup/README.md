# Liaison FMI Startup and Initialization Benchmarks

This directory contains comprehensive performance benchmarks for the Liaison FMI Rust implementation, focusing on startup time, initialization, and binary size analysis.

## Overview

The benchmark suite measures:

1. **Startup Performance** - Server startup time, FMU loading, Zenoh initialization, and request latency
2. **Initialization Performance** - FMU instantiation, configuration loading, and resource cleanup
3. **Binary Size Analysis** - Comparison of Rust vs C++ binary sizes with detailed section breakdown

## Benchmark Components

### 1. Startup Benchmark (`startup_bench.rs`)

Measures the complete startup cycle of the liaison-server:

- **Total startup time**: From process start to server ready
- **FMU loading time**: Time to extract and parse FMU
- **Zenoh initialization time**: Time to establish Zenoh session
- **First request latency**: Cold start request handling
- **Warm request latency**: Subsequent request handling after warmup
- **Memory footprint**: RSS memory usage at startup

**Output**: `startup_results.json`

### 2. Initialization Benchmark (`init_bench.rs`)

Measures FMU initialization performance:

- **FMU instantiation**: Time to load shared library and create instance
- **Configuration loading**: modelDescription.xml parsing
- **Variable initialization**: Setting up initial variable states
- **Setup experiment**: FMI setupExperiment call
- **Enter/Exit init mode**: FMI initialization mode transitions
- **Resource cleanup**: Cleanup and deallocation time

**Features**:
- Runs multiple iterations for statistical significance
- Calculates P50, P95, P99 percentiles
- Categorizes FMUs by size (Small, Medium, Large)
- Tests with different FMU sizes

**Output**: `init_results.json`

### 3. Binary Analysis (`binary_analysis.sh`)

Comprehensive binary size analysis:

- Compares Rust vs C++ binary sizes
- Analyzes stripped vs unstripped binaries
- Breaks down by ELF sections (.text, .data, .bss, etc.)
- Shows debug symbol overhead
- Documents size trade-offs

**Output**: `binary_analysis_report.txt`

## Quick Start

### Run All Benchmarks

The simplest way to run all benchmarks:

```bash
cd benchmarks/startup
./run_all_benchmarks.sh
```

This will:
1. Build the release binary
2. Run binary size analysis
3. Run startup benchmarks
4. Run initialization benchmarks
5. Generate a comprehensive summary report

Results are saved to `benchmark_results_YYYYMMDD_HHMMSS/`.

### Run Individual Benchmarks

#### Startup Benchmark

```bash
./run_startup_bench.sh
```

Options:
- `--no-build`: Skip building the binary
- `--no-analysis`: Skip binary size analysis
- `--binary PATH`: Path to liaison binary (default: `target/release/liaison`)
- `--fmu PATH`: Path to FMU file (default: `BouncingBallLiaison.fmu`)
- `--iterations N`: Number of iterations (default: 10)

Example:
```bash
./run_startup_bench.sh --fmu MyCustom.fmu --iterations 20
```

#### Initialization Benchmark

```bash
./run_init_bench.sh
```

Options:
- `--fmu PATH`: Path to FMU file (default: `BouncingBallLiaison.fmu`)
- `--iterations N`: Number of iterations (default: 100)

Example:
```bash
./run_init_bench.sh --fmu LargeFMU.fmu --iterations 200
```

#### Binary Analysis

```bash
./binary_analysis.sh
```

Environment variables:
- `RUST_BINARY`: Path to Rust binary (default: `target/release/liaison`)
- `RUST_LIB`: Path to Rust library (default: `target/release/libliaisonfmu.so`)
- `CPP_LIB`: Path to C++ library (default: `binaries/x86_64-linux/libliaisonfmu.so`)
- `RUST_BINARY_DEBUG`: Path to debug binary (default: `target/debug/liaison`)

Example:
```bash
RUST_BINARY=target/release/liaison ./binary_analysis.sh
```

## Benchmark Results

### Sample Output

#### Startup Performance

```
=== Startup Performance Report ===
Total Startup Time:         1234.56 ms
FMU Loading Time:            123.45 ms
Zenoh Init Time:             234.56 ms
First Request (cold):         50.00 ms
Warm Request:                  5.00 ms
Binary Size:                  12.34 MB
Startup Memory (RSS):         50000 KB
===================================
```

#### Initialization Performance

```
=== Initialization Performance Report ===
FMU Instantiation:            0.50 ms
Config Loading:              12.34 ms
Variable Init:                0.10 ms
Setup Experiment:             0.10 ms
Enter Init Mode:              0.05 ms
Exit Init Mode:               0.05 ms
Cleanup:                      0.20 ms
FMU Size:                     5.50 MB
Variable Count:                  42
==========================================

=== Total Init Time Percentiles ===
P50: 13.50 ms
P95: 15.20 ms
P99: 16.80 ms
====================================
```

#### Binary Size Analysis

```
=== Rust Release Binary (liaison) ===

Total Size: 12.34 MB
Debug Info: Stripped

Section Breakdown:
  .text (code):    8.50 MB
  .data:           0.50 MB
  .bss:            0.10 MB
  Total sections:  9.10 MB

=== Comparison: Rust Lib vs C++ Lib ===

Rust Lib:            10.00 MB
C++ Lib:              8.00 MB

Rust Lib is 2.00 MB larger (25.0%)
```

## Understanding the Results

### Startup Performance

- **Total startup time** includes all initialization steps
- **Cold start** is significantly higher than warm due to:
  - CPU cache misses
  - Page faults for code/data
  - JIT compilation (if applicable)
  - Network connection establishment

### Initialization Performance

- **FMU instantiation** is dominated by:
  - Shared library loading (dlopen)
  - Symbol resolution
  - FMI function binding

- **Config loading** depends on:
  - modelDescription.xml size
  - XML parser performance
  - Number of variables

- **Percentiles** show performance consistency:
  - P50 (median): Typical case
  - P95: 95% of operations faster than this
  - P99: 99% of operations faster than this

### Binary Size

Rust binaries are typically larger due to:
- **Monomorphization**: Generic code instantiation
- **Standard library**: More functionality included
- **LLVM codegen**: Different optimization strategy
- **Debug symbols**: More detailed (even stripped)

Trade-offs:
- Larger binaries provide better performance
- Memory safety without runtime cost
- Better error messages and debugging
- Zero-cost abstractions

## Optimization Strategies

### Reduce Binary Size

1. **Use LTO** (already enabled):
   ```toml
   [profile.release]
   lto = true
   ```

2. **Optimize for size**:
   ```toml
   [profile.release]
   opt-level = 'z'  # Optimize for size
   ```

3. **Strip symbols** (already enabled):
   ```toml
   [profile.release]
   strip = true
   ```

4. **Reduce codegen units**:
   ```toml
   [profile.release]
   codegen-units = 1
   ```

5. **Abort on panic** (for embedded):
   ```toml
   [profile.release]
   panic = 'abort'
   ```

### Improve Startup Time

1. **Lazy initialization**: Defer work until needed
2. **Parallel loading**: Load FMU and Zenoh concurrently
3. **Cache parsed XML**: Reuse modelDescription parsing
4. **Preload shared libraries**: Use LD_PRELOAD
5. **Profile with `perf`**: Identify bottlenecks

### Improve Initialization Time

1. **Optimize XML parsing**: Use faster parser or binary format
2. **Reduce variable count**: Only expose necessary variables
3. **Cache compiled FMUs**: Avoid repeated extraction
4. **Use mmap for files**: Memory-mapped file I/O
5. **Profile with `flamegraph`**: Visualize performance

## Requirements

### Tools

- **Rust toolchain**: `rustc`, `cargo`
- **Build tools**: `gcc`, `strip`, `size`, `readelf`, `nm`
- **System tools**: `bash`, `awk`, `grep`, `stat`

### Dependencies

The benchmark binaries are standalone and only require:
- Standard Rust library
- `zip` crate (for FMU extraction)

They do NOT require:
- Zenoh runtime (uses mock/simulation)
- Full liaison-server dependencies

## Troubleshooting

### Binary not found

```bash
# Build the release binary first
cargo build --release --bin liaison
```

### FMU not found

```bash
# Specify FMU path
./run_startup_bench.sh --fmu path/to/your.fmu
```

### Permission denied

```bash
# Make scripts executable
chmod +x *.sh
```

### Benchmark times out

Increase timeout or reduce iterations:
```bash
./run_init_bench.sh --iterations 10
```

## Advanced Usage

### Custom FMU Testing

Test with multiple FMU sizes:

```bash
# Small FMU
./run_init_bench.sh --fmu small.fmu --iterations 1000

# Medium FMU
./run_init_bench.sh --fmu medium.fmu --iterations 100

# Large FMU
./run_init_bench.sh --fmu large.fmu --iterations 10
```

### Continuous Benchmarking

Integrate into CI/CD:

```bash
# Run benchmarks and fail if regression
./run_all_benchmarks.sh
python3 check_regression.py benchmark_results_*/
```

### Profiling

Combine with profiling tools:

```bash
# Profile startup
perf record -g ./startup_bench
perf report

# Generate flamegraph
cargo flamegraph --bin liaison

# Memory profiling
valgrind --tool=massif ./startup_bench
```

## Output Files

After running benchmarks, you'll find:

- `startup_results.json` - Startup benchmark JSON results
- `init_results.json` - Initialization benchmark JSON results
- `binary_analysis_report.txt` - Binary size analysis report
- `benchmark_results_*/` - Timestamped results directory
  - `BENCHMARK_SUMMARY.md` - Comprehensive markdown report
  - `binary_analysis.log` - Full binary analysis output
  - All JSON results

## Integration with CI/CD

Example GitHub Actions workflow:

```yaml
name: Performance Benchmarks

on: [push, pull_request]

jobs:
  benchmark:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - uses: actions-rs/toolchain@v1
        with:
          toolchain: stable
      - name: Run benchmarks
        run: |
          cd benchmarks/startup
          ./run_all_benchmarks.sh
      - name: Upload results
        uses: actions/upload-artifact@v2
        with:
          name: benchmark-results
          path: benchmarks/startup/benchmark_results_*
```

## Contributing

To add new benchmarks:

1. Create new `.rs` file in this directory
2. Add corresponding shell script runner
3. Update `run_all_benchmarks.sh` to include new benchmark
4. Document in this README

## License

Same as parent project (MIT).

## See Also

- [Main README](../../README.md) - Project overview
- [Migration Guide](../../MIGRATION_GUIDE.md) - C++ to Rust migration
- [Test Framework](../../tests/README.md) - Integration tests
