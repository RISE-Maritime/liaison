# Memory Profiling Tools for Liaison FMI

This directory contains tools and scripts for memory profiling the Liaison FMI Rust implementation. The suite includes profiling scripts, test scenarios, and analysis tools to measure memory usage, detect leaks, and compare performance across versions.

## Overview

The memory profiling suite consists of three main components:

1. **memory_profile.sh** - Shell script for automated memory profiling using Valgrind Massif
2. **memory_test.rs** - Rust test suite for memory stress testing and leak detection
3. **analyze_memory.py** - Python script for analyzing and visualizing profiling results

## Quick Start

### Prerequisites

Install required tools:

```bash
# On Ubuntu/Debian
sudo apt-get install valgrind build-essential

# Optional: For enhanced profiling
sudo apt-get install heaptrack

# Optional: For visualization
pip3 install matplotlib
```

### Basic Usage

1. **Run complete memory profiling:**
   ```bash
   cd benchmarks
   ./memory_profile.sh
   ```

2. **Run quick profiling (faster, fewer iterations):**
   ```bash
   ./memory_profile.sh --quick
   ```

3. **Profile only the server:**
   ```bash
   ./memory_profile.sh --server-only
   ```

4. **Profile only the client library:**
   ```bash
   ./memory_profile.sh --client-only
   ```

5. **Analyze results:**
   ```bash
   python3 analyze_memory.py results/<timestamp>/
   ```

## Detailed Documentation

### memory_profile.sh

The main profiling script that orchestrates memory measurements for both server and client components.

**Features:**
- Automatic build with profiling symbols
- Valgrind Massif heap profiling
- Peak memory usage extraction
- Startup memory footprint measurement
- Result organization and reporting

**Command-line Options:**

```
./memory_profile.sh [OPTIONS]

Options:
  --server-only     Profile only the liaison server
  --client-only     Profile only the client library tests
  --quick           Run quick profile (fewer iterations)
  --detailed        Generate detailed massif visualization
  --compare PATH    Compare with previous results at PATH
  --help            Show help message
```

**Output Structure:**

```
results/
└── YYYYMMDD_HHMMSS/
    ├── SUMMARY.md              # High-level summary
    ├── server/
    │   ├── massif.out          # Raw Massif data
    │   ├── massif_report.txt   # Human-readable report
    │   ├── peak_memory.txt     # Peak memory value
    │   ├── server.log          # Server output
    │   └── startup_memory.txt  # Startup footprint
    └── client/
        ├── massif.out
        ├── massif_report.txt
        ├── peak_memory.txt
        └── test.log
```

**Example Usage:**

```bash
# Run full profiling
./memory_profile.sh

# Quick check after code changes
./memory_profile.sh --quick --server-only

# Compare with baseline
./memory_profile.sh --compare results/baseline/
```

### memory_test.rs

Comprehensive Rust test suite for memory profiling and leak detection.

**Test Scenarios:**

1. **test_memory_baseline** - Verifies memory tracking functionality
2. **test_single_instance_lifecycle** - Profiles a single FMU instance lifecycle
3. **test_multiple_instances** - Tests memory usage with multiple concurrent instances
4. **test_concurrent_instances** - Profiles multi-threaded instance handling
5. **memory_stress** - Stress test with rapid allocation/deallocation cycles
6. **test_memory_leak_detection** - Detects potential memory leaks
7. **test_large_data_transfer** - Profiles large data transfer scenarios
8. **test_operation_performance** - Measures performance of individual operations

**Running Tests:**

```bash
# Run all memory tests
cargo test --test memory_test --release -- --nocapture

# Run specific test
cargo test --test memory_test --release test_memory_leak_detection -- --nocapture

# Profile with Valgrind
valgrind --tool=massif --massif-out-file=massif.out \
    cargo test --test memory_test --release -- --nocapture

# Profile with Heaptrack (if available)
heaptrack cargo test --test memory_test --release -- --nocapture
```

**Test Output:**

Each test provides detailed memory snapshots:

```
[MEMORY] Initial - 45.23 MB
[MEMORY] After instance creation - 47.89 MB
[MEMORY] After 100 steps - 48.12 MB
[MEMORY] After instance cleanup - 45.67 MB

========================================
Memory Profile Summary
========================================
Total duration: 2.345s

Memory usage by stage:
Stage                          Memory (MB)
---------------------------------------------
Initial                             45.23
After instance creation             47.89
After 100 steps                     48.12
After instance cleanup              45.67
---------------------------------------------
Total memory growth: 0.44 MB
========================================
```

**Memory Leak Detection:**

The leak detection test performs multiple create/destroy cycles and checks for memory growth:

```
========================================
Memory Leak Detection
========================================
Baseline memory: 45.23 MB
Final memory:    45.67 MB
Growth:          0.44 MB

OK: No significant memory leak detected
========================================
```

### analyze_memory.py

Python script for analyzing Massif output and generating reports.

**Features:**
- Parse Massif output files
- Generate summary statistics
- Create detailed markdown reports
- Export data to CSV
- Generate memory usage plots
- Compare multiple profiling runs
- Detect significant memory changes

**Command-line Options:**

```
python3 analyze_memory.py <results_dir> [OPTIONS]

Options:
  results_dir           Directory containing profiling results
  --compare DIR         Compare with another results directory
  --export-csv          Export results to CSV files
  --plot                Generate memory usage plots
  --verbose             Show detailed output
```

**Example Usage:**

```bash
# Basic analysis
python3 analyze_memory.py results/20231109_143022/

# Compare two runs
python3 analyze_memory.py results/current/ --compare results/baseline/

# Full analysis with exports
python3 analyze_memory.py results/20231109_143022/ --export-csv --plot --verbose
```

**Output Files:**

- **ANALYSIS.md** - Detailed analysis report with memory timelines
- **COMPARISON.md** - Comparison report (when using --compare)
- **csv/** - CSV exports of memory data (when using --export-csv)
- **memory_usage.png** - Memory usage plots (when using --plot)

**Sample Output:**

```
======================================================================
                    MEMORY PROFILE SUMMARY
======================================================================

--- SERVER MEMORY ---
Peak memory:       52.34 MB
Final memory:      48.12 MB
Snapshots:            100
Peak at:         8234.50 ms

--- CLIENT MEMORY ---
Peak memory:      124.67 MB
Final memory:     112.34 MB
Snapshots:            150
Peak at:        12456.30 ms

======================================================================
```

**Comparison Output:**

```
======================================================================
                  MEMORY PROFILE COMPARISON
======================================================================

--- SERVER ---
Baseline peak:      52.34 MB
Current peak:       49.23 MB
Difference:         -3.11 MB (-5.9%)

--- CLIENT ---
Baseline peak:     124.67 MB
Current peak:      127.89 MB
Difference:         +3.22 MB (+2.6%)

⚠️  Notable change
======================================================================
```

## Integration with Cargo

You can integrate memory tests into your regular test workflow:

```bash
# Add to your CI/CD pipeline
cargo test --test memory_test --release

# Run with Valgrind in CI
if command -v valgrind &> /dev/null; then
    valgrind --leak-check=full --error-exitcode=1 \
        cargo test --test memory_test --release
fi
```

## Interpreting Results

### Memory Usage Patterns

**Normal patterns:**
- Gradual increase during initialization
- Stable memory during steady-state operation
- Clean decrease after cleanup
- Small variations between runs (< 5%)

**Warning signs:**
- Continuous linear growth over time (indicates leak)
- Large spikes without corresponding cleanup
- High variance between identical runs
- Memory not released after instance destruction

### Peak Memory Guidelines

Typical peak memory usage for reference:

- **Server (single FMU):** 30-60 MB
- **Client library:** 50-150 MB (depending on FMU complexity)
- **Multiple instances (10):** 200-500 MB

### Memory Leak Indicators

A memory leak is suspected if:
- Memory grows > 10 MB after repeated create/destroy cycles
- Final memory is > 5% higher than baseline after identical operations
- Valgrind reports "definitely lost" or "indirectly lost" blocks

## Performance Considerations

### Profiling Overhead

Valgrind Massif adds significant overhead:
- 20-30x slowdown
- Use `--quick` mode for rapid iteration
- Use release builds for realistic measurements

### Best Practices

1. **Always use release builds** for meaningful measurements
2. **Run multiple iterations** to account for variance
3. **Establish baselines** before making changes
4. **Profile in isolation** - close other applications
5. **Use consistent test scenarios** for comparisons

## Troubleshooting

### Common Issues

**Issue: Valgrind not found**
```bash
sudo apt-get install valgrind
```

**Issue: Massif output not generated**
- Check disk space
- Ensure write permissions in results directory
- Verify Valgrind version (>= 3.15 recommended)

**Issue: Memory measurements show N/A**
- Only works on Linux (uses /proc/self/status)
- On other platforms, use Valgrind for measurements

**Issue: Tests timeout**
- Use `--quick` mode
- Reduce number of iterations in memory_test.rs
- Increase timeout values

## Advanced Usage

### Custom Memory Scenarios

Add custom tests to `memory_test.rs`:

```rust
#[test]
fn test_my_scenario() {
    let mut tracker = MemoryTracker::new();

    tracker.snapshot("Start");

    // Your test code here

    tracker.snapshot("End");
    tracker.print_summary();
}
```

### Continuous Monitoring

Set up automated profiling:

```bash
#!/bin/bash
# run_nightly_profiling.sh

cd /path/to/liaison-fmi
git pull
cd benchmarks

# Run profiling
./memory_profile.sh --compare results/baseline/

# Email results or upload to monitoring system
# ... your notification code ...
```

### Integration with CI/CD

Example GitHub Actions workflow:

```yaml
name: Memory Profile

on:
  pull_request:
    branches: [ main ]

jobs:
  profile:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v2
      - name: Install dependencies
        run: sudo apt-get install valgrind
      - name: Run memory profiling
        run: |
          cd benchmarks
          ./memory_profile.sh --quick
      - name: Upload results
        uses: actions/upload-artifact@v2
        with:
          name: memory-profile
          path: benchmarks/results/
```

## Contributing

When adding new profiling capabilities:

1. Add test scenarios to `memory_test.rs`
2. Update `memory_profile.sh` if new profiling tools are used
3. Extend `analyze_memory.py` for new metrics
4. Update this README with usage examples

## References

- [Valgrind Massif Manual](https://valgrind.org/docs/manual/ms-manual.html)
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [Memory Profiling in Rust](https://doc.rust-lang.org/book/ch15-00-smart-pointers.html)

## License

Same as the main Liaison FMI project (MIT).
