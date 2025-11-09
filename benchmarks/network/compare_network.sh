#!/bin/bash
#
# Network Benchmark Comparison Script for Liaison FMI
#
# This script runs network benchmarks for both Rust and C++ implementations
# (if available) and generates comparison reports.
#
# Usage:
#   ./compare_network.sh <fmu_path> <responder_id> [options]
#
# Options:
#   --iterations <N>        Number of iterations for latency tests (default: 100)
#   --duration <seconds>    Duration for throughput tests (default: 10)
#   --clients <N>           Max concurrent clients (default: 16)
#   --skip-rust             Skip Rust benchmarks
#   --skip-cpp              Skip C++ benchmarks
#   --output-dir <path>     Output directory for results (default: ./results)
#   --format <format>       Output format: human, csv, json (default: csv)
#   --help                  Show this help message
#
# Output:
#   Results are saved to the output directory with timestamps.
#   A comparison report is generated showing differences between implementations.
#

set -euo pipefail

# Default configuration
ITERATIONS=100
DURATION=10
CLIENTS=16
SKIP_RUST=false
SKIP_CPP=false
OUTPUT_DIR="./results"
FORMAT="csv"
WARMUP=10

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Print colored message
print_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Print usage information
print_usage() {
    cat << EOF
Network Benchmark Comparison Script for Liaison FMI

Usage:
  $0 <fmu_path> <responder_id> [options]

Options:
  --iterations <N>        Number of iterations for latency tests (default: 100)
  --duration <seconds>    Duration for throughput tests (default: 10)
  --clients <N>           Max concurrent clients (default: 16)
  --skip-rust             Skip Rust benchmarks
  --skip-cpp              Skip C++ benchmarks
  --output-dir <path>     Output directory for results (default: ./results)
  --format <format>       Output format: human, csv, json (default: csv)
  --help                  Show this help message

Examples:
  # Run all benchmarks with default settings
  $0 ./BouncingBall.fmu test_responder

  # Run with custom iterations and duration
  $0 ./BouncingBall.fmu test_responder --iterations 1000 --duration 60

  # Compare only Rust implementation (skip C++)
  $0 ./BouncingBall.fmu test_responder --skip-cpp

  # Generate JSON output
  $0 ./BouncingBall.fmu test_responder --format json

EOF
}

# Parse command-line arguments
if [ $# -lt 2 ]; then
    print_error "Missing required arguments"
    print_usage
    exit 1
fi

FMU_PATH="$1"
RESPONDER_ID="$2"
shift 2

while [ $# -gt 0 ]; do
    case "$1" in
        --iterations)
            ITERATIONS="$2"
            shift 2
            ;;
        --duration)
            DURATION="$2"
            shift 2
            ;;
        --clients)
            CLIENTS="$2"
            shift 2
            ;;
        --skip-rust)
            SKIP_RUST=true
            shift
            ;;
        --skip-cpp)
            SKIP_CPP=true
            shift
            ;;
        --output-dir)
            OUTPUT_DIR="$2"
            shift 2
            ;;
        --format)
            FORMAT="$2"
            shift 2
            ;;
        --help)
            print_usage
            exit 0
            ;;
        *)
            print_error "Unknown option: $1"
            print_usage
            exit 1
            ;;
    esac
done

# Validate inputs
if [ ! -f "$FMU_PATH" ]; then
    print_error "FMU file not found: $FMU_PATH"
    exit 1
fi

if [[ ! "$FORMAT" =~ ^(human|csv|json)$ ]]; then
    print_error "Invalid format: $FORMAT. Must be human, csv, or json"
    exit 1
fi

# Create output directory
mkdir -p "$OUTPUT_DIR"
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
OUTPUT_PREFIX="${OUTPUT_DIR}/benchmark_${TIMESTAMP}"

print_info "Liaison FMI Network Benchmark Comparison"
print_info "========================================="
print_info "FMU: $FMU_PATH"
print_info "Responder ID: $RESPONDER_ID"
print_info "Iterations: $ITERATIONS"
print_info "Duration: ${DURATION}s"
print_info "Max Clients: $CLIENTS"
print_info "Output Directory: $OUTPUT_DIR"
print_info "Output Format: $FORMAT"
echo

# Find workspace root
WORKSPACE_ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")/../.." && pwd)"
cd "$WORKSPACE_ROOT"

# Build benchmarks
print_info "Building benchmarks..."

if [ "$SKIP_RUST" = false ]; then
    print_info "Building Rust benchmarks..."
    if cargo build --release --bin latency_test --bin throughput_test 2>&1 | grep -q "error"; then
        print_error "Failed to build Rust benchmarks"
        SKIP_RUST=true
    else
        print_success "Rust benchmarks built successfully"
    fi
fi

if [ "$SKIP_CPP" = false ]; then
    print_info "Checking for C++ benchmarks..."
    if [ ! -f "./build/latency_benchmark" ] && [ ! -f "./build/throughput_benchmark" ]; then
        print_warning "C++ benchmarks not found. Skipping C++ tests."
        SKIP_CPP=true
    else
        print_success "C++ benchmarks found"
    fi
fi

# Function to run Rust latency benchmark
run_rust_latency() {
    print_info "Running Rust latency benchmark..."

    local output_file="${OUTPUT_PREFIX}_rust_latency.${FORMAT}"

    if ./target/release/latency_test \
        "$FMU_PATH" \
        "$RESPONDER_ID" \
        --iterations "$ITERATIONS" \
        --warmup "$WARMUP" \
        --format "$FORMAT" \
        > "$output_file" 2>&1; then
        print_success "Rust latency benchmark completed: $output_file"
        return 0
    else
        print_error "Rust latency benchmark failed"
        return 1
    fi
}

# Function to run Rust throughput benchmark
run_rust_throughput() {
    print_info "Running Rust throughput benchmark..."

    local output_file="${OUTPUT_PREFIX}_rust_throughput.${FORMAT}"

    if ./target/release/throughput_test \
        "$FMU_PATH" \
        "$RESPONDER_ID" \
        --duration "$DURATION" \
        --clients "$CLIENTS" \
        --format "$FORMAT" \
        > "$output_file" 2>&1; then
        print_success "Rust throughput benchmark completed: $output_file"
        return 0
    else
        print_error "Rust throughput benchmark failed"
        return 1
    fi
}

# Function to run C++ latency benchmark
run_cpp_latency() {
    print_info "Running C++ latency benchmark..."

    local output_file="${OUTPUT_PREFIX}_cpp_latency.${FORMAT}"

    if [ -f "./build/latency_benchmark" ]; then
        if ./build/latency_benchmark \
            "$FMU_PATH" \
            "$RESPONDER_ID" \
            "$ITERATIONS" \
            > "$output_file" 2>&1; then
            print_success "C++ latency benchmark completed: $output_file"
            return 0
        else
            print_error "C++ latency benchmark failed"
            return 1
        fi
    else
        print_warning "C++ latency benchmark not found"
        return 1
    fi
}

# Function to run C++ throughput benchmark
run_cpp_throughput() {
    print_info "Running C++ throughput benchmark..."

    local output_file="${OUTPUT_PREFIX}_cpp_throughput.${FORMAT}"

    if [ -f "./build/throughput_benchmark" ]; then
        if ./build/throughput_benchmark \
            "$FMU_PATH" \
            "$RESPONDER_ID" \
            "$DURATION" \
            "$CLIENTS" \
            > "$output_file" 2>&1; then
            print_success "C++ throughput benchmark completed: $output_file"
            return 0
        else
            print_error "C++ throughput benchmark failed"
            return 1
        fi
    else
        print_warning "C++ throughput benchmark not found"
        return 1
    fi
}

# Run benchmarks
RUST_LATENCY_SUCCESS=false
RUST_THROUGHPUT_SUCCESS=false
CPP_LATENCY_SUCCESS=false
CPP_THROUGHPUT_SUCCESS=false

if [ "$SKIP_RUST" = false ]; then
    echo
    print_info "=== Running Rust Benchmarks ==="
    run_rust_latency && RUST_LATENCY_SUCCESS=true
    sleep 2
    run_rust_throughput && RUST_THROUGHPUT_SUCCESS=true
fi

if [ "$SKIP_CPP" = false ]; then
    echo
    print_info "=== Running C++ Benchmarks ==="
    run_cpp_latency && CPP_LATENCY_SUCCESS=true
    sleep 2
    run_cpp_throughput && CPP_THROUGHPUT_SUCCESS=true
fi

# Generate comparison report
echo
print_info "=== Generating Comparison Report ==="

REPORT_FILE="${OUTPUT_PREFIX}_comparison_report.txt"

cat > "$REPORT_FILE" << EOF
Liaison FMI Network Benchmark Comparison Report
================================================

Generated: $(date)
FMU: $FMU_PATH
Responder ID: $RESPONDER_ID

Configuration:
- Iterations (latency): $ITERATIONS
- Duration (throughput): ${DURATION}s
- Max Concurrent Clients: $CLIENTS
- Output Format: $FORMAT

Results:
--------
EOF

# Add Rust results to report
if [ "$RUST_LATENCY_SUCCESS" = true ]; then
    echo >> "$REPORT_FILE"
    echo "Rust Latency Benchmark: SUCCESS" >> "$REPORT_FILE"
    echo "Output: ${OUTPUT_PREFIX}_rust_latency.${FORMAT}" >> "$REPORT_FILE"
else
    echo >> "$REPORT_FILE"
    echo "Rust Latency Benchmark: FAILED or SKIPPED" >> "$REPORT_FILE"
fi

if [ "$RUST_THROUGHPUT_SUCCESS" = true ]; then
    echo >> "$REPORT_FILE"
    echo "Rust Throughput Benchmark: SUCCESS" >> "$REPORT_FILE"
    echo "Output: ${OUTPUT_PREFIX}_rust_throughput.${FORMAT}" >> "$REPORT_FILE"
else
    echo >> "$REPORT_FILE"
    echo "Rust Throughput Benchmark: FAILED or SKIPPED" >> "$REPORT_FILE"
fi

# Add C++ results to report
if [ "$CPP_LATENCY_SUCCESS" = true ]; then
    echo >> "$REPORT_FILE"
    echo "C++ Latency Benchmark: SUCCESS" >> "$REPORT_FILE"
    echo "Output: ${OUTPUT_PREFIX}_cpp_latency.${FORMAT}" >> "$REPORT_FILE"
else
    echo >> "$REPORT_FILE"
    echo "C++ Latency Benchmark: FAILED or SKIPPED" >> "$REPORT_FILE"
fi

if [ "$CPP_THROUGHPUT_SUCCESS" = true ]; then
    echo >> "$REPORT_FILE"
    echo "C++ Throughput Benchmark: SUCCESS" >> "$REPORT_FILE"
    echo "Output: ${OUTPUT_PREFIX}_cpp_throughput.${FORMAT}" >> "$REPORT_FILE"
else
    echo >> "$REPORT_FILE"
    echo "C++ Throughput Benchmark: FAILED or SKIPPED" >> "$REPORT_FILE"
fi

# Generate CSV comparison if both implementations completed
if [ "$FORMAT" = "csv" ] && { [ "$RUST_LATENCY_SUCCESS" = true ] || [ "$RUST_THROUGHPUT_SUCCESS" = true ]; }; then
    echo >> "$REPORT_FILE"
    echo >> "$REPORT_FILE"
    echo "CSV Comparison Files:" >> "$REPORT_FILE"
    echo "---------------------" >> "$REPORT_FILE"

    COMPARISON_CSV="${OUTPUT_PREFIX}_comparison.csv"

    echo "implementation,test_type,test_name,metric,value" > "$COMPARISON_CSV"

    # Parse and combine CSV results
    if [ "$RUST_LATENCY_SUCCESS" = true ] && [ -f "${OUTPUT_PREFIX}_rust_latency.csv" ]; then
        tail -n +2 "${OUTPUT_PREFIX}_rust_latency.csv" | while IFS=',' read -r test_name samples min max mean median p95 p99 stddev; do
            echo "Rust,latency,$test_name,mean_ms,$mean" >> "$COMPARISON_CSV"
            echo "Rust,latency,$test_name,p95_ms,$p95" >> "$COMPARISON_CSV"
            echo "Rust,latency,$test_name,p99_ms,$p99" >> "$COMPARISON_CSV"
        done
    fi

    if [ "$RUST_THROUGHPUT_SUCCESS" = true ] && [ -f "${OUTPUT_PREFIX}_rust_throughput.csv" ]; then
        tail -n +2 "${OUTPUT_PREFIX}_rust_throughput.csv" | while IFS=',' read -r test_name duration total success failed success_rate rps throughput avg_resp total_data; do
            echo "Rust,throughput,$test_name,requests_per_second,$rps" >> "$COMPARISON_CSV"
            echo "Rust,throughput,$test_name,throughput_mbps,$throughput" >> "$COMPARISON_CSV"
        done
    fi

    echo "Combined comparison saved to: $COMPARISON_CSV" >> "$REPORT_FILE"
    print_success "Combined CSV comparison: $COMPARISON_CSV"
fi

# Display summary
echo >> "$REPORT_FILE"
echo >> "$REPORT_FILE"
echo "Summary:" >> "$REPORT_FILE"
echo "--------" >> "$REPORT_FILE"
echo "All benchmark results have been saved to: $OUTPUT_DIR" >> "$REPORT_FILE"
echo "Timestamp: $TIMESTAMP" >> "$REPORT_FILE"

# Print report to console
cat "$REPORT_FILE"
print_success "Comparison report saved to: $REPORT_FILE"

# Generate visualization hints
echo
print_info "=== Visualization Suggestions ==="
echo "To visualize the results, you can use the following tools:"
echo
echo "1. For CSV files, use Python with pandas and matplotlib:"
echo "   python3 -c \"import pandas as pd; import matplotlib.pyplot as plt; df = pd.read_csv('$COMPARISON_CSV'); print(df.head())\""
echo
echo "2. For JSON files, use jq to process and analyze:"
echo "   cat ${OUTPUT_PREFIX}_rust_latency.json | jq '.[] | {test_name, mean_ms, p95_ms}'"
echo
echo "3. Import CSV into spreadsheet software (Excel, LibreOffice Calc, Google Sheets)"
echo "   for creating charts and graphs"
echo

print_success "Benchmark comparison completed successfully!"
exit 0
