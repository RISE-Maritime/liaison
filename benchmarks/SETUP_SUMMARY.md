# Memory Profiling Tools - Setup Summary

This document provides a summary of the memory profiling tools created for the Liaison FMI Rust implementation.

## Created Files

### Core Tools

1. **memory_profile.sh** (executable)
   - Location: `/workspace/benchmarks/memory_profile.sh`
   - Main profiling script using Valgrind Massif
   - Profiles both server and client library
   - Generates organized results with peak memory reports

2. **analyze_memory.py** (executable)
   - Location: `/workspace/benchmarks/analyze_memory.py`
   - Python script for analyzing Massif output
   - Generates reports, comparisons, plots, and CSV exports
   - Detects memory regressions

3. **memory_test.rs**
   - Location: `/workspace/tests/memory_test.rs`
   - Rust test suite for memory stress testing
   - Multiple test scenarios for different use cases
   - Built-in memory leak detection

### Supporting Files

4. **README.md**
   - Location: `/workspace/benchmarks/README.md`
   - Comprehensive documentation
   - Usage instructions and examples
   - Troubleshooting guide

5. **EXAMPLES.md**
   - Location: `/workspace/benchmarks/EXAMPLES.md`
   - Practical usage examples
   - CI/CD integration examples
   - Advanced scenarios and workflows

6. **Makefile**
   - Location: `/workspace/benchmarks/Makefile`
   - Convenient make targets for common operations
   - Simplifies workflow with `make profile`, `make analyze`, etc.

7. **quick_start.sh** (executable)
   - Location: `/workspace/benchmarks/quick_start.sh`
   - Interactive guide for first-time users
   - Walks through setup and first profiling run

8. **profile_config.example.sh**
   - Location: `/workspace/benchmarks/profile_config.example.sh`
   - Example configuration file
   - Advanced options for customization

9. **.gitignore**
   - Location: `/workspace/benchmarks/.gitignore`
   - Excludes profiling results and temporary files
   - Keeps repository clean

10. **results/** directory
    - Location: `/workspace/benchmarks/results/`
    - Stores all profiling results
    - Organized by timestamp

## Quick Start

### Installation

```bash
cd /workspace/benchmarks

# Check dependencies
make check-deps

# Install missing dependencies (Ubuntu/Debian)
make install-deps

# Or run interactive setup
./quick_start.sh
```

### Basic Usage

```bash
# Quick profiling (5-10 seconds)
make quick-profile

# Full profiling (30-60 seconds)
make profile

# Analyze latest results
make analyze

# Set baseline for comparisons
make baseline

# Compare with baseline
make compare

# Generate plots
make plot
```

## Tool Features

### memory_profile.sh

**Features:**
- Automatic build with profiling symbols
- Valgrind Massif heap profiling
- Server and client profiling
- Peak memory extraction
- Comparison with previous runs
- Organized result storage

**Usage:**
```bash
./memory_profile.sh [--server-only|--client-only|--quick|--detailed|--compare PATH]
```

**Output:**
- `results/TIMESTAMP/SUMMARY.md` - High-level summary
- `results/TIMESTAMP/server/massif.out` - Server Massif data
- `results/TIMESTAMP/client/massif.out` - Client Massif data
- Peak memory reports and logs

### analyze_memory.py

**Features:**
- Parse Massif output files
- Generate detailed reports
- Compare multiple runs
- Export to CSV
- Generate plots (requires matplotlib)
- Detect memory regressions

**Usage:**
```bash
python3 analyze_memory.py results/TIMESTAMP/ [--compare DIR] [--export-csv] [--plot]
```

**Output:**
- `ANALYSIS.md` - Detailed analysis report
- `COMPARISON.md` - Comparison report (with --compare)
- `csv/` - CSV exports (with --export-csv)
- `memory_usage.png` - Memory plots (with --plot)

### memory_test.rs

**Test Scenarios:**
1. `test_memory_baseline` - Verify tracking works
2. `test_single_instance_lifecycle` - Single FMU lifecycle
3. `test_multiple_instances` - Multiple concurrent instances
4. `test_concurrent_instances` - Multi-threaded instances
5. `memory_stress` - Stress test with rapid alloc/dealloc
6. `test_memory_leak_detection` - Detect memory leaks
7. `test_large_data_transfer` - Large data scenarios
8. `test_operation_performance` - Operation performance

**Usage:**
```bash
# Run all tests
cargo test --test memory_test --release -- --nocapture

# Run specific test
cargo test --test memory_test --release test_memory_leak_detection -- --nocapture

# Profile with Valgrind
valgrind --tool=massif cargo test --test memory_test --release
```

## Common Workflows

### Development Workflow

1. **Before making changes:**
   ```bash
   make profile
   make baseline
   ```

2. **After making changes:**
   ```bash
   make profile
   make compare
   ```

3. **Quick iteration:**
   ```bash
   make dev-check  # Runs quick-profile + analyze
   ```

### CI/CD Integration

Add to your CI pipeline:

```bash
cd benchmarks
make ci-profile
```

Or use the examples in `EXAMPLES.md` for GitHub Actions, GitLab CI, or Jenkins.

### Memory Leak Investigation

```bash
# Run leak detection tests
make test-leak

# Or specific test with full leak check
valgrind --leak-check=full \
    cargo test --test memory_test --release test_memory_leak_detection
```

### Performance Optimization

```bash
# Profile current state
make profile
make baseline

# Make optimizations
# ... edit code ...

# Compare improvement
make profile
make compare

# If better, update baseline
make baseline
```

## Make Targets Reference

| Target | Description |
|--------|-------------|
| `make help` | Show all available targets |
| `make profile` | Run full memory profiling |
| `make quick-profile` | Run quick profiling (faster) |
| `make server-profile` | Profile server only |
| `make client-profile` | Profile client only |
| `make test` | Run memory tests |
| `make test-massif` | Run tests with Massif |
| `make test-leak` | Run leak detection |
| `make analyze` | Analyze latest results |
| `make compare` | Compare with baseline |
| `make plot` | Generate plots |
| `make baseline` | Set current as baseline |
| `make export-csv` | Export to CSV |
| `make clean` | Remove results (keep baseline) |
| `make clean-all` | Remove all results |
| `make install-deps` | Install dependencies |
| `make check-deps` | Check dependencies |
| `make ci-profile` | CI/CD profiling |

## Requirements

### Required
- valgrind (with massif)
- cargo (Rust toolchain)
- python3

### Optional
- matplotlib (for plots): `pip3 install matplotlib`
- heaptrack (for additional profiling)
- ms_print (usually comes with valgrind)

### Installation (Ubuntu/Debian)

```bash
sudo apt-get update
sudo apt-get install -y valgrind build-essential python3 python3-pip
pip3 install matplotlib
```

## Directory Structure

```
benchmarks/
├── README.md                      # Main documentation
├── EXAMPLES.md                    # Usage examples
├── SETUP_SUMMARY.md              # This file
├── Makefile                       # Make targets
├── .gitignore                     # Git ignore rules
├── memory_profile.sh             # Main profiling script
├── analyze_memory.py             # Analysis script
├── quick_start.sh                # Interactive setup
├── profile_config.example.sh     # Configuration example
└── results/                       # Profiling results
    ├── .gitkeep
    ├── baseline/                  # Baseline for comparisons
    └── YYYYMMDD_HHMMSS/          # Timestamped results
        ├── SUMMARY.md
        ├── ANALYSIS.md
        ├── COMPARISON.md
        ├── server/
        │   ├── massif.out
        │   ├── massif_report.txt
        │   ├── peak_memory.txt
        │   └── server.log
        └── client/
            ├── massif.out
            ├── massif_report.txt
            ├── peak_memory.txt
            └── test.log

tests/
└── memory_test.rs                 # Memory test suite
```

## Example Output

### Profiling Output

```
[INFO] Starting memory profiling for Liaison FMI
[INFO] Results will be saved to: ./results/20231109_143022
[INFO] Checking dependencies...
[SUCCESS] All required dependencies found
[INFO] Building project in release mode...
[SUCCESS] Build completed successfully
[INFO] Profiling liaison-server...
[INFO] Running Massif profiling on server...
[SUCCESS] Server profiling complete. Peak memory: 52.34 MB
[INFO] Profiling client library via memory tests...
[SUCCESS] Client profiling complete. Peak memory: 124.67 MB
[INFO] Generating summary report...
[SUCCESS] Summary report generated: ./results/20231109_143022/SUMMARY.md
[SUCCESS] Memory profiling complete!
```

### Analysis Output

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

### Test Output

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

## Typical Memory Usage

For reference, typical peak memory usage:

- **Server (single FMU):** 30-60 MB
- **Client library:** 50-150 MB
- **Multiple instances (10):** 200-500 MB

## Troubleshooting

### Valgrind not found
```bash
sudo apt-get install valgrind
```

### Memory measurements show N/A
- Only works on Linux (uses /proc/self/status)
- On other platforms, rely on Valgrind measurements

### Tests timeout
- Use `--quick` mode
- Reduce iterations in memory_test.rs
- Increase timeout values

### matplotlib not available
```bash
pip3 install matplotlib
```

## Next Steps

1. **Run your first profile:**
   ```bash
   cd /workspace/benchmarks
   ./quick_start.sh
   ```

2. **Explore examples:**
   ```bash
   less EXAMPLES.md
   ```

3. **Set up CI/CD:**
   - See EXAMPLES.md for GitHub Actions/GitLab CI examples

4. **Customize configuration:**
   ```bash
   cp profile_config.example.sh profile_config.sh
   # Edit profile_config.sh as needed
   ```

5. **Integrate into workflow:**
   - Run profiling before/after major changes
   - Set baselines for important milestones
   - Monitor memory trends over time

## Support

For questions or issues:
1. Check README.md for detailed documentation
2. Review EXAMPLES.md for practical examples
3. Run `make help` for available commands
4. Check Valgrind documentation: https://valgrind.org/docs/manual/ms-manual.html

## License

Same as the main Liaison FMI project (MIT).
