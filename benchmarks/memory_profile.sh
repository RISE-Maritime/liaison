#!/bin/bash
# Memory Profiling Script for Liaison FMI
#
# This script profiles memory usage of both liaison-server and the FMI client library
# using Valgrind's Massif tool for heap profiling and custom measurements.
#
# Usage:
#   ./memory_profile.sh [OPTIONS]
#
# Options:
#   --server-only     Profile only the liaison server
#   --client-only     Profile only the client library tests
#   --quick           Run quick profile (fewer iterations)
#   --detailed        Generate detailed massif visualization
#   --compare PATH    Compare with previous results at PATH
#   --help            Show this help message
#
# Requirements:
#   - valgrind (with massif)
#   - ms_print (comes with valgrind)
#   - heaptrack (optional, for additional profiling)
#   - cargo (Rust toolchain)
#
# Output:
#   Results are saved to ./results/TIMESTAMP/

set -euo pipefail

# Configuration
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
RESULTS_DIR="$SCRIPT_DIR/results/$TIMESTAMP"
PROFILE_SERVER=true
PROFILE_CLIENT=true
QUICK_MODE=false
DETAILED_MODE=false
COMPARE_PATH=""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --server-only)
            PROFILE_CLIENT=false
            shift
            ;;
        --client-only)
            PROFILE_SERVER=false
            shift
            ;;
        --quick)
            QUICK_MODE=true
            shift
            ;;
        --detailed)
            DETAILED_MODE=true
            shift
            ;;
        --compare)
            COMPARE_PATH="$2"
            shift 2
            ;;
        --help)
            head -n 25 "$0" | tail -n 22
            exit 0
            ;;
        *)
            echo -e "${RED}Unknown option: $1${NC}"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

# Helper functions
log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

log_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

check_dependencies() {
    log_info "Checking dependencies..."

    local missing_deps=()

    if ! command -v valgrind &> /dev/null; then
        missing_deps+=("valgrind")
    fi

    if ! command -v cargo &> /dev/null; then
        missing_deps+=("cargo")
    fi

    if [[ ${#missing_deps[@]} -gt 0 ]]; then
        log_error "Missing required dependencies: ${missing_deps[*]}"
        log_info "Install with: sudo apt-get install ${missing_deps[*]}"
        exit 1
    fi

    if ! command -v ms_print &> /dev/null; then
        log_warning "ms_print not found. Massif visualization will be limited."
    fi

    if ! command -v heaptrack &> /dev/null; then
        log_warning "heaptrack not found. Additional profiling will be skipped."
    fi

    log_success "All required dependencies found"
}

build_project() {
    log_info "Building project in release mode..."
    cd "$WORKSPACE_ROOT"

    # Build with debug symbols for better profiling
    RUSTFLAGS="-C force-frame-pointers=yes" cargo build --release

    if [[ $? -eq 0 ]]; then
        log_success "Build completed successfully"
    else
        log_error "Build failed"
        exit 1
    fi
}

profile_server() {
    log_info "Profiling liaison-server..."

    local server_results="$RESULTS_DIR/server"
    mkdir -p "$server_results"

    # Find the server binary
    local server_bin="$WORKSPACE_ROOT/target/release/liaison"

    if [[ ! -f "$server_bin" ]]; then
        log_error "Server binary not found at $server_bin"
        return 1
    fi

    # Check if FMU exists
    local fmu_path="$WORKSPACE_ROOT/BouncingBallLiaison.fmu"
    if [[ ! -f "$fmu_path" ]]; then
        log_warning "BouncingBallLiaison.fmu not found. Server profiling may fail."
        log_info "Run this first: cargo test --test reference_fmu_tests"
    fi

    log_info "Running Massif profiling on server..."

    # Massif profiling
    local massif_out="$server_results/massif.out"
    timeout 30s valgrind \
        --tool=massif \
        --massif-out-file="$massif_out" \
        --time-unit=ms \
        --detailed-freq=1 \
        --max-snapshots=100 \
        --heap=yes \
        --stacks=no \
        "$server_bin" serve-fmu "$fmu_path" 2>&1 | tee "$server_results/server.log" &

    local server_pid=$!

    # Let it run for a bit
    if [[ "$QUICK_MODE" == true ]]; then
        sleep 5
    else
        sleep 10
    fi

    # Kill the server gracefully
    kill -TERM $server_pid 2>/dev/null || true
    wait $server_pid 2>/dev/null || true

    # Generate massif report
    if [[ -f "$massif_out" ]]; then
        ms_print "$massif_out" > "$server_results/massif_report.txt" 2>/dev/null || true

        # Extract peak memory usage
        local peak_mem=$(grep "peak" "$server_results/massif_report.txt" | head -1 || echo "N/A")
        echo "$peak_mem" > "$server_results/peak_memory.txt"

        log_success "Server profiling complete. Peak memory: $peak_mem"
    else
        log_warning "Massif output not generated for server"
    fi

    # Memory footprint at startup
    log_info "Measuring server startup memory..."
    /usr/bin/time -v "$server_bin" --help 2>&1 | grep -E "(Maximum resident|elapsed)" > "$server_results/startup_memory.txt" || true
}

profile_client() {
    log_info "Profiling client library via memory tests..."

    local client_results="$RESULTS_DIR/client"
    mkdir -p "$client_results"

    cd "$WORKSPACE_ROOT"

    # Run memory test with Massif
    local massif_out="$client_results/massif.out"

    log_info "Running Massif profiling on memory tests..."

    if [[ "$QUICK_MODE" == true ]]; then
        # Quick mode: single iteration
        valgrind \
            --tool=massif \
            --massif-out-file="$massif_out" \
            --time-unit=ms \
            --detailed-freq=1 \
            --max-snapshots=100 \
            cargo test --release memory_stress -- --nocapture 2>&1 | tee "$client_results/test.log"
    else
        # Full mode: comprehensive tests
        valgrind \
            --tool=massif \
            --massif-out-file="$massif_out" \
            --time-unit=ms \
            --detailed-freq=1 \
            --max-snapshots=100 \
            cargo test --release --test memory_test -- --nocapture 2>&1 | tee "$client_results/test.log"
    fi

    # Generate massif report
    if [[ -f "$massif_out" ]]; then
        ms_print "$massif_out" > "$client_results/massif_report.txt" 2>/dev/null || true

        # Extract peak memory
        local peak_mem=$(grep "peak" "$client_results/massif_report.txt" | head -1 || echo "N/A")
        echo "$peak_mem" > "$client_results/peak_memory.txt"

        log_success "Client profiling complete. Peak memory: $peak_mem"
    else
        log_warning "Massif output not generated for client"
    fi
}

generate_summary() {
    log_info "Generating summary report..."

    local summary_file="$RESULTS_DIR/SUMMARY.md"

    cat > "$summary_file" << EOF
# Memory Profiling Summary

**Date:** $(date)
**Mode:** $([ "$QUICK_MODE" == true ] && echo "Quick" || echo "Full")

## Configuration
- Profile Server: $PROFILE_SERVER
- Profile Client: $PROFILE_CLIENT
- Detailed Mode: $DETAILED_MODE

## Results

EOF

    # Server results
    if [[ "$PROFILE_SERVER" == true ]] && [[ -f "$RESULTS_DIR/server/peak_memory.txt" ]]; then
        cat >> "$summary_file" << EOF
### Server Memory Usage

\`\`\`
$(cat "$RESULTS_DIR/server/peak_memory.txt")
\`\`\`

EOF
    fi

    # Client results
    if [[ "$PROFILE_CLIENT" == true ]] && [[ -f "$RESULTS_DIR/client/peak_memory.txt" ]]; then
        cat >> "$summary_file" << EOF
### Client Memory Usage

\`\`\`
$(cat "$RESULTS_DIR/client/peak_memory.txt")
\`\`\`

EOF
    fi

    cat >> "$summary_file" << EOF
## Files Generated

- Server results: \`$RESULTS_DIR/server/\`
  - \`massif.out\` - Raw Massif data
  - \`massif_report.txt\` - Human-readable report
  - \`peak_memory.txt\` - Peak memory usage
  - \`server.log\` - Server output

- Client results: \`$RESULTS_DIR/client/\`
  - \`massif.out\` - Raw Massif data
  - \`massif_report.txt\` - Human-readable report
  - \`peak_memory.txt\` - Peak memory usage
  - \`test.log\` - Test output

## Viewing Results

To view detailed Massif results:
\`\`\`bash
ms_print $RESULTS_DIR/server/massif.out
ms_print $RESULTS_DIR/client/massif.out
\`\`\`

To analyze with the Python script:
\`\`\`bash
python3 $SCRIPT_DIR/analyze_memory.py $RESULTS_DIR
\`\`\`

EOF

    log_success "Summary report generated: $summary_file"
    cat "$summary_file"
}

compare_results() {
    if [[ -z "$COMPARE_PATH" ]]; then
        return
    fi

    log_info "Comparing with previous results at $COMPARE_PATH..."

    local compare_file="$RESULTS_DIR/COMPARISON.md"

    cat > "$compare_file" << EOF
# Memory Profile Comparison

**Current:** $RESULTS_DIR
**Previous:** $COMPARE_PATH

EOF

    # Compare server peaks
    if [[ -f "$RESULTS_DIR/server/peak_memory.txt" ]] && [[ -f "$COMPARE_PATH/server/peak_memory.txt" ]]; then
        cat >> "$compare_file" << EOF
## Server Memory

**Current:**
\`\`\`
$(cat "$RESULTS_DIR/server/peak_memory.txt")
\`\`\`

**Previous:**
\`\`\`
$(cat "$COMPARE_PATH/server/peak_memory.txt")
\`\`\`

EOF
    fi

    # Compare client peaks
    if [[ -f "$RESULTS_DIR/client/peak_memory.txt" ]] && [[ -f "$COMPARE_PATH/client/peak_memory.txt" ]]; then
        cat >> "$compare_file" << EOF
## Client Memory

**Current:**
\`\`\`
$(cat "$RESULTS_DIR/client/peak_memory.txt")
\`\`\`

**Previous:**
\`\`\`
$(cat "$COMPARE_PATH/client/peak_memory.txt")
\`\`\`

EOF
    fi

    log_success "Comparison report generated: $compare_file"
    cat "$compare_file"
}

# Main execution
main() {
    log_info "Starting memory profiling for Liaison FMI"
    log_info "Results will be saved to: $RESULTS_DIR"

    mkdir -p "$RESULTS_DIR"

    check_dependencies
    build_project

    if [[ "$PROFILE_SERVER" == true ]]; then
        profile_server
    fi

    if [[ "$PROFILE_CLIENT" == true ]]; then
        profile_client
    fi

    generate_summary
    compare_results

    log_success "Memory profiling complete!"
    log_info "Results saved to: $RESULTS_DIR"
    log_info "View summary: cat $RESULTS_DIR/SUMMARY.md"
}

main
