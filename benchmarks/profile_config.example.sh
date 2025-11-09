#!/bin/bash
# Memory Profiling Configuration
#
# This file contains advanced configuration options for memory profiling.
# Copy this file to profile_config.sh and customize as needed.
# The memory_profile.sh script will source this file if it exists.

# Valgrind Massif Options
# See: valgrind --tool=massif --help
export MASSIF_TIME_UNIT="ms"              # Time unit: i, ms, B
export MASSIF_DETAILED_FREQ=1              # Detailed snapshot frequency
export MASSIF_MAX_SNAPSHOTS=100            # Maximum number of snapshots
export MASSIF_PEAK_INACCURACY=1.0          # Peak inaccuracy percentage
export MASSIF_HEAP=yes                     # Profile heap
export MASSIF_STACKS=no                    # Profile stacks (adds overhead)
export MASSIF_DEPTH=30                     # Stack trace depth

# Test Configuration
export MEMORY_TEST_ITERATIONS=100          # Number of test iterations
export MEMORY_TEST_INSTANCES=10            # Number of FMU instances
export MEMORY_TEST_STEPS=100               # Simulation steps per instance
export MEMORY_TEST_DATA_SIZE=50000         # Data size per instance

# Profiling Duration
export SERVER_PROFILE_DURATION=10          # Server profiling duration (seconds)
export CLIENT_PROFILE_DURATION=30          # Client profiling duration (seconds)

# Output Configuration
export RESULTS_RETENTION_DAYS=30           # Keep results for N days
export AUTO_CLEANUP_OLD_RESULTS=false      # Automatically clean old results
export GENERATE_PLOTS_AUTO=false           # Auto-generate plots
export EXPORT_CSV_AUTO=false               # Auto-export to CSV

# Comparison Settings
export AUTO_COMPARE_BASELINE=true          # Always compare with baseline
export FAIL_ON_REGRESSION_PCT=20           # Fail if memory grows > N%
export WARN_ON_REGRESSION_PCT=10           # Warn if memory grows > N%

# Additional Tools
export USE_HEAPTRACK=false                 # Use heaptrack if available
export USE_PERF=false                      # Use perf for additional profiling
export USE_TIME_V=true                     # Use /usr/bin/time -v

# Notification Settings
export NOTIFY_ON_COMPLETION=false          # Send notification when done
export NOTIFICATION_EMAIL=""               # Email for notifications
export NOTIFICATION_WEBHOOK=""             # Webhook URL for notifications

# CI/CD Settings
export CI_MODE=false                       # Enable CI mode (less verbose)
export CI_FAIL_ON_ERROR=true              # Fail CI on profiling errors
export CI_UPLOAD_ARTIFACTS=true           # Upload results as artifacts

# Debug Settings
export PROFILING_DEBUG=false              # Enable debug output
export KEEP_TEMP_FILES=false              # Keep temporary files
export VERBOSE_OUTPUT=false               # Verbose profiling output

# Rust Build Settings
export RUSTFLAGS="-C force-frame-pointers=yes"  # Better profiling symbols
export CARGO_PROFILE="release"                   # Build profile to use

# Custom Test Selection
# Comma-separated list of tests to run, empty = all
export MEMORY_TESTS=""                     # e.g., "test_memory_leak_detection,memory_stress"

# Platform-Specific Settings
if [[ "$OSTYPE" == "linux-gnu"* ]]; then
    export ENABLE_PROCFS_MONITORING=true
    export ENABLE_CGROUP_MONITORING=false
elif [[ "$OSTYPE" == "darwin"* ]]; then
    export ENABLE_PROCFS_MONITORING=false
    export USE_INSTRUMENTS=true
fi

# Baseline Management
export BASELINE_DIR="results/baseline"
export AUTO_UPDATE_BASELINE=false          # Automatically update baseline
export BASELINE_UPDATE_THRESHOLD=5         # Update if improvement > N%

# Report Generation
export GENERATE_HTML_REPORT=false
export REPORT_TEMPLATE=""
export REPORT_INCLUDE_GRAPHS=true
export REPORT_INCLUDE_RAW_DATA=false

# Example: Custom pre-profiling hook
# This function is called before profiling starts
pre_profiling_hook() {
    echo "Running pre-profiling setup..."
    # Your custom setup here
    # e.g., prepare test data, check system state, etc.
}

# Example: Custom post-profiling hook
# This function is called after profiling completes
post_profiling_hook() {
    echo "Running post-profiling cleanup..."
    # Your custom cleanup here
    # e.g., archive results, send notifications, etc.

    # Example: Send notification
    if [[ "$NOTIFY_ON_COMPLETION" == true ]] && [[ -n "$NOTIFICATION_WEBHOOK" ]]; then
        curl -X POST "$NOTIFICATION_WEBHOOK" \
            -H "Content-Type: application/json" \
            -d "{\"text\": \"Memory profiling completed\"}"
    fi
}

# Example: Result filtering
# Return 0 to keep result, 1 to discard
should_keep_result() {
    local result_dir="$1"

    # Example: Keep only results with significant findings
    if [[ -f "$result_dir/SUMMARY.md" ]]; then
        # Check if peak memory is above threshold
        return 0
    fi

    return 1
}

# Profiling constraints (for CI environments)
export MAX_PROFILING_TIME=600              # Max total profiling time (seconds)
export MAX_MEMORY_USAGE_MB=4096           # Max memory to use during profiling
export MAX_DISK_USAGE_MB=1024             # Max disk space for results

# Feature flags
export ENABLE_MEMORY_LEAK_CHECK=true
export ENABLE_PERFORMANCE_METRICS=true
export ENABLE_CONCURRENCY_TEST=true
export ENABLE_STRESS_TEST=true

# Log levels
export LOG_LEVEL="info"                   # debug, info, warning, error
