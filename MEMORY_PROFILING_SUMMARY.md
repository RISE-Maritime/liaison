# Memory Profiling Tools - Complete Summary

## Overview

A comprehensive memory profiling toolkit has been created for the Liaison FMI Rust implementation. The toolkit includes scripts for automated profiling, test scenarios for stress testing and leak detection, and analysis tools for generating reports and visualizations.

## What Was Created

### 1. Memory Profiling Script (`benchmarks/memory_profile.sh`)

**Location:** `/workspace/benchmarks/memory_profile.sh`

**Purpose:** Main profiling script that orchestrates memory measurements using Valgrind Massif

**Features:**
- Profiles both liaison-server and client library
- Uses Valgrind Massif for heap profiling
- Automatic build with profiling symbols
- Extracts peak memory usage
- Generates organized results with timestamps
- Supports quick mode for faster iteration
- Can compare with previous runs

**Usage:**
```bash
cd benchmarks

# Full profiling (server + client)
./memory_profile.sh

# Quick profiling (faster, fewer iterations)
./memory_profile.sh --quick

# Server only
./memory_profile.sh --server-only

# Client only
./memory_profile.sh --client-only

# Compare with previous results
./memory_profile.sh --compare results/baseline/

# Show help
./memory_profile.sh --help
```

**Output Structure:**
```
results/YYYYMMDD_HHMMSS/
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

### 2. Memory Test Scenario (`tests/memory_test.rs`)

**Location:** `/workspace/tests/memory_test.rs`

**Purpose:** Comprehensive Rust test suite for memory stress testing and leak detection

**Test Scenarios:**
1. **test_memory_baseline** - Verifies memory tracking functionality
2. **test_single_instance_lifecycle** - Profiles single FMU instance lifecycle
3. **test_multiple_instances** - Tests multiple concurrent instances
4. **test_concurrent_instances** - Profiles multi-threaded instances
5. **memory_stress** - Stress test with rapid allocation/deallocation
6. **test_memory_leak_detection** - Detects potential memory leaks
7. **test_large_data_transfer** - Profiles large data transfer scenarios
8. **test_operation_performance** - Measures individual operation performance

**Key Features:**
- Built-in memory tracking using /proc/self/status (Linux)
- MemoryTracker utility for snapshot-based measurements
- Automatic leak detection with growth analysis
- Performance metrics collection
- Detailed output showing memory at each stage

**Usage:**
```bash
# Run all memory tests
cargo test --test memory_test --release -- --nocapture

# Run specific test
cargo test --test memory_test --release test_memory_leak_detection -- --nocapture

# Profile with Valgrind Massif
valgrind --tool=massif \
    --massif-out-file=massif.out \
    cargo test --test memory_test --release

# Profile with leak detection
valgrind --leak-check=full \
    --show-leak-kinds=all \
    cargo test --test memory_test --release
```

**Sample Output:**
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

### 3. Analysis Script (`benchmarks/analyze_memory.py`)

**Location:** `/workspace/benchmarks/analyze_memory.py`

**Purpose:** Parses profiling output, generates reports, creates visualizations, and highlights differences

**Features:**
- Parses Valgrind Massif output files
- Generates detailed markdown reports
- Compares multiple profiling runs
- Exports data to CSV format
- Creates memory usage plots (requires matplotlib)
- Detects significant memory changes
- Highlights regressions

**Usage:**
```bash
# Basic analysis
python3 analyze_memory.py results/20231109_143022/

# Compare with baseline
python3 analyze_memory.py results/current/ --compare results/baseline/

# Export to CSV
python3 analyze_memory.py results/latest/ --export-csv

# Generate plots
python3 analyze_memory.py results/latest/ --plot

# Full analysis with all features
python3 analyze_memory.py results/latest/ \
    --compare results/baseline/ \
    --export-csv \
    --plot \
    --verbose
```

**Output Files:**
- **ANALYSIS.md** - Detailed analysis with memory timelines
- **COMPARISON.md** - Comparison report (with --compare)
- **csv/** - CSV exports of memory data (with --export-csv)
- **memory_usage.png** - Memory usage plots (with --plot)

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

--- COMPARISON ---
Server: -3.11 MB (-5.9%)
Client: +3.22 MB (+2.6%)
======================================================================
```

### 4. Supporting Files

#### README.md (`benchmarks/README.md`)
- Comprehensive documentation
- Tool descriptions and usage
- Troubleshooting guide
- Best practices
- Integration examples

#### EXAMPLES.md (`benchmarks/EXAMPLES.md`)
- Practical usage examples
- Development workflow examples
- CI/CD integration (GitHub Actions, GitLab CI, Jenkins)
- Performance optimization workflows
- Memory leak detection examples
- Advanced scenarios

#### Makefile (`benchmarks/Makefile`)
- Convenient make targets for common operations
- Simplifies workflow
- Includes CI/CD targets

**Common targets:**
```bash
make help              # Show all available targets
make profile           # Run full memory profiling
make quick-profile     # Run quick profiling
make test              # Run memory tests
make analyze           # Analyze latest results
make compare           # Compare with baseline
make plot              # Generate plots
make baseline          # Set current as baseline
make clean             # Clean results
make install-deps      # Install dependencies
make check-deps        # Check dependencies
```

#### quick_start.sh (`benchmarks/quick_start.sh`)
- Interactive setup guide
- Walks through first-time usage
- Checks dependencies
- Offers to install missing tools
- Demonstrates basic workflows

#### profile_config.example.sh (`benchmarks/profile_config.example.sh`)
- Example configuration file
- Advanced options for customization
- Pre/post profiling hooks
- Platform-specific settings
- CI/CD configuration options

#### .gitignore (`benchmarks/.gitignore`)
- Excludes profiling results from git
- Keeps repository clean
- Preserves baseline results

## Installation and Setup

### Prerequisites

**Required:**
- valgrind (with massif)
- cargo (Rust toolchain)
- python3

**Optional:**
- matplotlib (for plots): `pip3 install matplotlib`
- heaptrack (for additional profiling)

### Installation

**On Ubuntu/Debian:**
```bash
cd benchmarks

# Automatic installation
make install-deps

# Or manual installation
sudo apt-get update
sudo apt-get install -y valgrind build-essential python3 python3-pip
pip3 install matplotlib
```

**Verification:**
```bash
make check-deps
```

## Quick Start Guide

### First Time Setup

```bash
cd /workspace/benchmarks

# Interactive setup
./quick_start.sh

# Or manual setup
make check-deps
make install-deps
```

### Run Your First Profile

```bash
# Quick profile (5-10 seconds)
make quick-profile

# View results
make show-latest

# Set as baseline for future comparisons
make baseline
```

### Basic Workflow

```bash
# 1. Before making changes
make profile
make baseline

# 2. Make your code changes
# ... edit code ...

# 3. Profile again
make profile

# 4. Compare
make compare

# 5. If improved, update baseline
make baseline
```

## Usage Examples

### Development Workflow

**Before optimization:**
```bash
cd benchmarks
make profile
make baseline
# Note: Baseline set at 124.67 MB peak memory
```

**After optimization:**
```bash
make profile
make compare
# Output shows: -15.23 MB (-12.2%) improvement
make baseline  # Update baseline
```

### Quick Iteration

```bash
# Make small change
vim ../liaison-server/src/fmu_loader.rs

# Quick check
make dev-check  # Runs quick-profile + analyze

# Review results
make show-latest
```

### Memory Leak Detection

```bash
# Run leak detection tests
make test-leak

# Or with specific test
valgrind --leak-check=full \
    --show-leak-kinds=all \
    cargo test --test memory_test --release test_memory_leak_detection
```

### CI/CD Integration

**GitHub Actions:**
```yaml
- name: Memory Profile
  run: |
    cd benchmarks
    make ci-profile

- name: Upload Results
  uses: actions/upload-artifact@v3
  with:
    name: memory-profile
    path: benchmarks/results/
```

**In Makefile:**
```bash
make ci-profile  # Optimized for CI environments
```

## How to Use the Tools

### Scenario 1: Regular Development

```bash
# Daily workflow
make quick-profile  # Fast check during development
make analyze       # Review results

# Weekly workflow
make profile       # Full comprehensive profile
make compare       # Compare with last week's baseline
make baseline      # Update if improved
```

### Scenario 2: Performance Investigation

```bash
# Step 1: Establish current state
./memory_profile.sh --detailed
make analyze

# Step 2: View detailed Massif output
ms_print results/latest/server/massif.out | less

# Step 3: Find hotspots
ms_print results/latest/server/massif.out | grep "peak" -A 50

# Step 4: Run specific tests
cargo test --test memory_test --release test_large_data_transfer -- --nocapture
```

### Scenario 3: Regression Testing

```bash
# Set baseline from main branch
git checkout main
make profile
cp -r results/latest results/baseline

# Test feature branch
git checkout feature/my-optimization
make profile

# Compare
python3 analyze_memory.py results/latest/ \
    --compare results/baseline/ \
    --plot \
    --export-csv
```

### Scenario 4: Continuous Monitoring

```bash
# Automated nightly profiling
cat > nightly_profile.sh << 'EOF'
#!/bin/bash
cd /workspace/benchmarks
make profile
make compare
# Email results or upload to monitoring system
EOF

chmod +x nightly_profile.sh
# Add to cron or CI schedule
```

## Interpreting Results

### Memory Usage Patterns

**Normal patterns:**
- Gradual increase during initialization
- Stable memory during steady-state operation
- Clean decrease after cleanup
- Small variations between runs (< 5%)

**Warning signs:**
- Continuous linear growth over time (leak)
- Large spikes without corresponding cleanup
- High variance between identical runs
- Memory not released after instance destruction

### Typical Memory Usage

For reference:
- **Server (single FMU):** 30-60 MB peak
- **Client library:** 50-150 MB peak
- **Multiple instances (10):** 200-500 MB peak

### Memory Leak Indicators

Suspect a leak if:
- Memory grows > 10 MB after repeated create/destroy cycles
- Final memory is > 5% higher than baseline after identical operations
- Valgrind reports "definitely lost" or "indirectly lost" blocks
- Memory growth is linear with iteration count

## Troubleshooting

### Common Issues

**Issue: Valgrind not found**
```bash
sudo apt-get install valgrind
```

**Issue: Memory measurements show N/A**
- Only works on Linux (uses /proc/self/status)
- On other platforms, rely on Valgrind measurements

**Issue: Tests timeout**
- Use `--quick` mode
- Reduce iterations in memory_test.rs
- Increase timeout values

**Issue: matplotlib not available**
```bash
pip3 install matplotlib
```

**Issue: Permission denied**
```bash
chmod +x benchmarks/*.sh benchmarks/*.py
```

## Advanced Usage

### Custom Configuration

```bash
cp profile_config.example.sh profile_config.sh
# Edit profile_config.sh with your settings
./memory_profile.sh  # Will use custom config
```

### Custom Test Scenarios

Add to `tests/memory_test.rs`:

```rust
#[test]
fn test_my_custom_scenario() {
    let mut tracker = MemoryTracker::new();

    tracker.snapshot("Start");

    // Your custom test code

    tracker.snapshot("End");
    tracker.print_summary();
}
```

### Integration with Monitoring

Export metrics to Prometheus/Grafana:

```bash
# Extract metrics
SERVER_PEAK=$(grep -oP '\d+\.\d+' results/latest/server/peak_memory.txt | head -1)

# Push to Pushgateway
curl -X POST $PUSHGATEWAY_URL/metrics/job/liaison_memory \
    --data-binary "liaison_server_peak_mb $SERVER_PEAK"
```

## Files Summary

### Core Tools
- `/workspace/benchmarks/memory_profile.sh` - Main profiling script
- `/workspace/benchmarks/analyze_memory.py` - Analysis and reporting
- `/workspace/tests/memory_test.rs` - Memory test suite

### Documentation
- `/workspace/benchmarks/README.md` - Main documentation
- `/workspace/benchmarks/EXAMPLES.md` - Usage examples
- `/workspace/benchmarks/SETUP_SUMMARY.md` - Setup guide

### Utilities
- `/workspace/benchmarks/Makefile` - Make targets
- `/workspace/benchmarks/quick_start.sh` - Interactive setup
- `/workspace/benchmarks/profile_config.example.sh` - Config template
- `/workspace/benchmarks/.gitignore` - Git ignore rules

### Results
- `/workspace/benchmarks/results/` - Profiling results directory

## Integration Points

### With Existing Build System
- Uses existing Cargo workspace
- Works with current test infrastructure
- Integrates with existing FMU test files

### With CI/CD
- GitHub Actions examples provided
- GitLab CI examples provided
- Jenkins pipeline examples provided
- Make target: `make ci-profile`

### With Development Workflow
- Quick mode for rapid iteration
- Baseline comparison for tracking improvements
- Detailed mode for deep investigation

## Best Practices

1. **Always use release builds** for meaningful measurements
2. **Run multiple iterations** to account for variance
3. **Establish baselines** before making changes
4. **Profile in isolation** - close other applications
5. **Use consistent test scenarios** for comparisons
6. **Document significant changes** in commit messages
7. **Update baseline** after verified improvements
8. **Profile early and often** - catch issues early

## Next Steps

1. **Get Started:**
   ```bash
   cd /workspace/benchmarks
   ./quick_start.sh
   ```

2. **Explore Examples:**
   ```bash
   less EXAMPLES.md
   ```

3. **Run First Profile:**
   ```bash
   make quick-profile
   make show-latest
   ```

4. **Set Baseline:**
   ```bash
   make baseline
   ```

5. **Integrate into Workflow:**
   - Run before/after major changes
   - Add to CI/CD pipeline
   - Monitor trends over time

## References

- Main Documentation: `/workspace/benchmarks/README.md`
- Usage Examples: `/workspace/benchmarks/EXAMPLES.md`
- Setup Guide: `/workspace/benchmarks/SETUP_SUMMARY.md`
- Valgrind Massif: https://valgrind.org/docs/manual/ms-manual.html
- Rust Performance: https://nnethercote.github.io/perf-book/

## Support

For questions or issues:
1. Check README.md for detailed documentation
2. Review EXAMPLES.md for practical examples
3. Run `make help` for available commands
4. Check script comments for inline documentation

---

**Created:** 2025-11-09
**Location:** `/workspace/benchmarks/`
**License:** MIT (same as main project)
