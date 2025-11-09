# Memory Profiling Examples

This document provides practical examples of using the Liaison FMI memory profiling tools for various scenarios.

## Table of Contents

- [Quick Start](#quick-start)
- [Development Workflow](#development-workflow)
- [CI/CD Integration](#cicd-integration)
- [Performance Optimization](#performance-optimization)
- [Memory Leak Detection](#memory-leak-detection)
- [Comparing Implementations](#comparing-implementations)
- [Advanced Scenarios](#advanced-scenarios)

## Quick Start

### First Time Setup

```bash
cd benchmarks

# Run interactive setup
./quick_start.sh

# Or manually check dependencies
make check-deps

# Install missing dependencies
make install-deps
```

### Your First Profile

```bash
# Quick profile (recommended for first run)
make quick-profile

# View results
make show-latest

# Set as baseline for future comparisons
make baseline
```

## Development Workflow

### Before Making Changes

Establish a baseline before implementing memory optimizations:

```bash
# Full profile of current implementation
make profile

# Set as baseline
make baseline

# Note the peak memory usage
make show-latest
```

### After Making Changes

Compare your changes with the baseline:

```bash
# Profile the new code
make profile

# Compare with baseline
make compare

# If better, update baseline
make baseline
```

### Quick Iteration Cycle

During active development, use quick mode:

```bash
# Make code changes
vim ../liaison-server/src/fmu_loader.rs

# Quick check
make dev-check

# This runs: quick-profile + analyze
```

## CI/CD Integration

### GitHub Actions Example

Add to `.github/workflows/memory-profile.yml`:

```yaml
name: Memory Profile

on:
  pull_request:
    branches: [ main ]
  schedule:
    - cron: '0 2 * * *'  # Daily at 2 AM

jobs:
  profile:
    runs-on: ubuntu-latest

    steps:
      - uses: actions/checkout@v3

      - name: Install Rust
        uses: actions-rust-lang/setup-rust-toolchain@v1

      - name: Install dependencies
        run: |
          sudo apt-get update
          sudo apt-get install -y valgrind
          pip3 install matplotlib

      - name: Run profiling
        run: |
          cd benchmarks
          make ci-profile

      - name: Upload results
        uses: actions/upload-artifact@v3
        with:
          name: memory-profile-results
          path: benchmarks/results/

      - name: Check for regressions
        run: |
          cd benchmarks
          python3 check_regression.py results/latest/ --threshold 20
```

### GitLab CI Example

Add to `.gitlab-ci.yml`:

```yaml
memory-profile:
  stage: test
  image: rust:latest
  before_script:
    - apt-get update && apt-get install -y valgrind python3-pip
    - pip3 install matplotlib
  script:
    - cd benchmarks
    - make ci-profile
  artifacts:
    paths:
      - benchmarks/results/
    expire_in: 1 week
  only:
    - merge_requests
    - main
```

### Jenkins Pipeline Example

```groovy
pipeline {
    agent any

    stages {
        stage('Memory Profile') {
            steps {
                sh '''
                    cd benchmarks
                    make install-deps
                    make ci-profile
                '''
            }
        }

        stage('Analyze') {
            steps {
                sh '''
                    cd benchmarks
                    make full-analysis
                '''
            }
        }

        stage('Archive') {
            steps {
                archiveArtifacts artifacts: 'benchmarks/results/**/*'
            }
        }
    }
}
```

## Performance Optimization

### Finding Memory Hotspots

1. **Run detailed profiling:**

```bash
./memory_profile.sh --detailed
```

2. **Examine massif output:**

```bash
# View detailed Massif report
ms_print results/latest/server/massif.out | less

# Look for the largest allocations
ms_print results/latest/server/massif.out | grep "^-" | head -20
```

3. **Analyze specific snapshots:**

```bash
# Extract peak snapshot
ms_print results/latest/server/massif.out | sed -n '/peak/,/^-/p'
```

### Optimizing Specific Components

**Before optimization:**

```bash
# Profile current implementation
make server-profile

# Note peak memory
PEAK_BEFORE=$(grep "Peak memory" results/latest/server/peak_memory.txt)
echo "Before: $PEAK_BEFORE"
```

**After optimization:**

```bash
# Profile optimized implementation
make server-profile

# Compare
PEAK_AFTER=$(grep "Peak memory" results/latest/server/peak_memory.txt)
echo "After: $PEAK_AFTER"

# Detailed comparison
make compare
```

### Memory Usage Breakdown

Understand where memory is allocated:

```bash
# Run with detailed frequency
valgrind --tool=massif \
    --detailed-freq=1 \
    --massif-out-file=detailed.out \
    cargo test --test memory_test --release test_single_instance_lifecycle

# View allocation tree
ms_print detailed.out | less
# Use '/' to search for your function names
```

## Memory Leak Detection

### Basic Leak Check

```bash
# Run leak detection test
make test-leak

# Look for "definitely lost" or "indirectly lost"
```

### Comprehensive Leak Testing

```bash
# Run all leak detection tests
cd ..
valgrind --leak-check=full \
    --show-leak-kinds=all \
    --track-origins=yes \
    --log-file=benchmarks/leak_report.txt \
    cargo test --test memory_test --release

# Analyze report
less benchmarks/leak_report.txt
```

### Regression Testing for Leaks

```bash
# Create baseline
make profile
cp results/latest/client/massif.out results/baseline_leak.out

# After changes, compare
make profile
python3 compare_leaks.py results/baseline_leak.out results/latest/client/massif.out
```

### Custom Leak Test

Add to `tests/memory_test.rs`:

```rust
#[test]
fn test_my_component_leak() {
    let mut tracker = MemoryTracker::new();

    tracker.snapshot("Initial");

    // Run 100 iterations of create/destroy
    for i in 0..100 {
        let component = MyComponent::new();
        component.do_work();
        drop(component);

        if i % 10 == 0 {
            tracker.snapshot(&format!("After {} iterations", i));
        }
    }

    tracker.snapshot("Final");
    tracker.print_summary();

    // Memory should be stable
    assert!(tracker.memory_growth_mb() < 5.0);
}
```

## Comparing Implementations

### Compare Rust vs C++ Implementation

```bash
# Profile Rust implementation
cd benchmarks
make profile
cp -r results/latest results/rust_impl

# Switch to C++ branch (if applicable)
git checkout cpp-implementation

# Profile C++ implementation
make profile
cp -r results/latest results/cpp_impl

# Compare
python3 analyze_memory.py results/rust_impl --compare results/cpp_impl

# Generate comparison report
python3 -c "
from analyze_memory import MemoryAnalyzer, MemoryComparator
from pathlib import Path

rust = MemoryAnalyzer(Path('results/rust_impl'))
cpp = MemoryAnalyzer(Path('results/cpp_impl'))

rust.load_profiles()
cpp.load_profiles()

comparator = MemoryComparator(cpp, rust)
comparator.print_comparison()
"
```

### Compare Different Optimization Strategies

```bash
# Strategy 1: Pool allocations
git checkout feature/pool-allocations
make profile
mkdir -p results/comparisons
cp -r results/latest results/comparisons/pool_strategy

# Strategy 2: Arena allocations
git checkout feature/arena-allocations
make profile
cp -r results/latest results/comparisons/arena_strategy

# Strategy 3: Lazy initialization
git checkout feature/lazy-init
make profile
cp -r results/latest results/comparisons/lazy_strategy

# Compare all
for dir in results/comparisons/*/; do
    echo "=== $(basename $dir) ==="
    python3 analyze_memory.py "$dir"
done
```

## Advanced Scenarios

### Multi-Instance Profiling

```bash
# Create custom test for your scenario
cat > tests/custom_multi_instance.rs << 'EOF'
#[test]
fn test_50_concurrent_fmus() {
    let mut tracker = MemoryTracker::new();

    tracker.snapshot("Start");

    // Create 50 instances
    let instances: Vec<_> = (0..50)
        .map(|i| create_fmu_instance(i))
        .collect();

    tracker.snapshot("50 instances created");

    // Run simulation
    for step in 0..1000 {
        for instance in &instances {
            instance.do_step();
        }

        if step % 100 == 0 {
            tracker.snapshot(&format!("Step {}", step));
        }
    }

    tracker.snapshot("Simulation complete");
    tracker.print_summary();
}
EOF

# Profile it
valgrind --tool=massif \
    cargo test --test custom_multi_instance --release
```

### Time-Series Memory Analysis

Track memory usage over extended periods:

```bash
# Run profiling every hour
cat > continuous_profile.sh << 'EOF'
#!/bin/bash
for i in {1..24}; do
    echo "Hour $i profiling..."
    cd benchmarks
    make quick-profile
    sleep 3600
done
EOF

chmod +x continuous_profile.sh
./continuous_profile.sh &

# Analyze trend
ls -1 results/*/server/peak_memory.txt | while read f; do
    echo "$(dirname $(dirname $f)): $(cat $f)"
done
```

### Custom Analysis Script

Create domain-specific analysis:

```python
#!/usr/bin/env python3
# custom_analysis.py

from pathlib import Path
import json

def analyze_fmu_memory(results_dir):
    """Analyze memory usage per FMU instance"""

    # Parse test logs
    log_file = results_dir / "client" / "test.log"
    if not log_file.exists():
        return

    instances = {}
    with open(log_file) as f:
        for line in f:
            if "MEMORY" in line:
                # Parse your custom log format
                # Example: [MEMORY] Instance 5 - 12.34 MB
                parts = line.split()
                instance_id = parts[3]
                memory_mb = float(parts[5])

                if instance_id not in instances:
                    instances[instance_id] = []
                instances[instance_id].append(memory_mb)

    # Analyze per-instance
    print("Memory usage per FMU instance:")
    for instance_id, measurements in instances.items():
        avg = sum(measurements) / len(measurements)
        peak = max(measurements)
        print(f"  Instance {instance_id}: avg={avg:.2f}MB, peak={peak:.2f}MB")

if __name__ == '__main__':
    import sys
    analyze_fmu_memory(Path(sys.argv[1]))
```

### Automated Reporting

Generate weekly memory reports:

```bash
# weekly_report.sh
#!/bin/bash

WEEK=$(date +%Y-W%U)
REPORT_DIR="reports/$WEEK"
mkdir -p "$REPORT_DIR"

# Run profiling
cd benchmarks
make profile

# Generate all reports
make full-analysis

# Copy to report directory
cp -r results/latest/* "$REPORT_DIR/"

# Generate summary
cat > "$REPORT_DIR/README.md" << EOF
# Weekly Memory Profile Report - $WEEK

## Summary
$(cat results/latest/SUMMARY.md)

## Trend Analysis
$(python3 analyze_trend.py results/latest)

## Recommendations
- Review any memory growth > 5%
- Check for new leaks
- Update baseline if improved
EOF

# Send email (optional)
if command -v mail &> /dev/null; then
    mail -s "Weekly Memory Report - $WEEK" \
        team@example.com < "$REPORT_DIR/README.md"
fi
```

### Integration with Monitoring Systems

Send metrics to Prometheus/Grafana:

```bash
# Export metrics
cat > export_metrics.sh << 'EOF'
#!/bin/bash

RESULT_DIR="$1"
METRICS_FILE="memory_metrics.prom"

# Extract peak memory
SERVER_PEAK=$(grep -oP '\d+\.\d+' "$RESULT_DIR/server/peak_memory.txt" | head -1)
CLIENT_PEAK=$(grep -oP '\d+\.\d+' "$RESULT_DIR/client/peak_memory.txt" | head -1)

# Write Prometheus format
cat > "$METRICS_FILE" << PROM
# HELP liaison_server_peak_memory_mb Server peak memory in MB
# TYPE liaison_server_peak_memory_mb gauge
liaison_server_peak_memory_mb $SERVER_PEAK

# HELP liaison_client_peak_memory_mb Client peak memory in MB
# TYPE liaison_client_peak_memory_mb gauge
liaison_client_peak_memory_mb $CLIENT_PEAK
PROM

# Push to Pushgateway
if [ -n "$PUSHGATEWAY_URL" ]; then
    curl -X POST "$PUSHGATEWAY_URL/metrics/job/liaison_memory" \
        --data-binary "@$METRICS_FILE"
fi
EOF
```

## Troubleshooting Examples

### High Memory Usage Investigation

```bash
# 1. Profile with detailed snapshots
./memory_profile.sh --detailed

# 2. Find the peak snapshot
ms_print results/latest/server/massif.out | grep -A 50 "peak"

# 3. Analyze call stack at peak
ms_print results/latest/server/massif.out | grep "0x" | head -20

# 4. Run specific tests in isolation
cargo test --test memory_test --release test_large_data_transfer -- --nocapture

# 5. Check for external library leaks
valgrind --leak-check=full --show-leak-kinds=all \
    --trace-children=yes \
    cargo test --test memory_test --release
```

### Unexpected Memory Growth

```bash
# Compare multiple runs
for i in {1..5}; do
    make quick-profile
    sleep 10
done

# Analyze variance
ls -1 results/*/server/peak_memory.txt | while read f; do
    cat "$f"
done | python3 -c "
import sys
values = [float(line.split()[0]) for line in sys.stdin]
import statistics
print(f'Mean: {statistics.mean(values):.2f}')
print(f'Stdev: {statistics.stdev(values):.2f}')
print(f'Min: {min(values):.2f}')
print(f'Max: {max(values):.2f}')
"
```

## Tips and Best Practices

1. **Always use release builds** for profiling
2. **Profile on representative hardware** - CI environments may differ
3. **Warm up before measuring** - first run may have different characteristics
4. **Run multiple iterations** - single runs can be misleading
5. **Keep baselines up-to-date** - update after verified improvements
6. **Document significant changes** - note why memory usage changed
7. **Profile early and often** - catch issues before they compound
8. **Use version control** - tag commits with profiling results

## Further Reading

- [Valgrind Massif Manual](https://valgrind.org/docs/manual/ms-manual.html)
- [Rust Performance Book](https://nnethercote.github.io/perf-book/)
- [Memory Profiling Best Practices](https://easyperf.net/blog/2019/08/02/Perf-measurement-environment-on-Linux)
