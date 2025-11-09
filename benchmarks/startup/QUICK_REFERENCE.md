# Liaison FMI Benchmarks - Quick Reference

## One-Liner: Run Everything

```bash
cd /workspace/benchmarks/startup && ./run_all_benchmarks.sh
```

## Prerequisites

```bash
# Build release binary first
cd /workspace && cargo build --release --bin liaison
```

## Individual Benchmarks

### Startup Performance
```bash
./run_startup_bench.sh
```
**Output:** `startup_results.json`

### Initialization Performance
```bash
./run_init_bench.sh
```
**Output:** `init_results.json`

### Binary Size Analysis
```bash
./binary_analysis.sh
```
**Output:** `binary_analysis_report.txt`

### Regression Check
```bash
./check_regression.py baseline_dir/ current_dir/
```
**Exit code:** 0 = pass, 1 = regression

## Common Options

```bash
# Custom FMU
./run_startup_bench.sh --fmu /path/to/custom.fmu

# More iterations
./run_init_bench.sh --iterations 200

# Skip build
./run_startup_bench.sh --no-build

# Custom binary
./run_startup_bench.sh --binary /custom/path/liaison
```

## Output Locations

All results saved to timestamped directory:
```
benchmark_results_YYYYMMDD_HHMMSS/
├── BENCHMARK_SUMMARY.md         # Complete report
├── startup_results.json         # Startup metrics
├── init_results.json            # Init metrics
├── binary_analysis.log          # Analysis output
└── binary_analysis_report.txt   # Size report
```

## Help

```bash
./run_startup_bench.sh --help
./run_init_bench.sh --help
./run_all_benchmarks.sh --help
```

## Documentation

- **README.md** - Comprehensive documentation
- **USAGE_GUIDE.md** - Step-by-step instructions
- **BENCHMARKS_CREATED.md** - Creation summary

## Typical Workflow

```bash
# 1. Build
cd /workspace
cargo build --release

# 2. Baseline
cd benchmarks/startup
./run_all_benchmarks.sh
mv benchmark_results_* baseline/

# 3. Make changes & rebuild
cd /workspace
# ... your changes ...
cargo build --release

# 4. New benchmark
cd benchmarks/startup
./run_all_benchmarks.sh

# 5. Check regression
./check_regression.py baseline/ benchmark_results_*/

# 6. Review
cat benchmark_results_*/BENCHMARK_SUMMARY.md
```

## Troubleshooting Quick Fixes

```bash
# Binary not found
cargo build --release --bin liaison

# Permission denied
chmod +x *.sh *.py

# FMU not found
cp /path/to/your.fmu /workspace/BouncingBallLiaison.fmu
```
