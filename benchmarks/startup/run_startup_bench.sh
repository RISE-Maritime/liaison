#!/bin/bash

# Run startup benchmark for Liaison FMI
# Automates building and running startup performance tests

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Default configuration
BINARY_PATH="target/release/liaison"
FMU_PATH="BouncingBallLiaison.fmu"
ITERATIONS=10
BUILD_FIRST=true
RUN_ANALYSIS=true

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --no-build)
            BUILD_FIRST=false
            shift
            ;;
        --no-analysis)
            RUN_ANALYSIS=false
            shift
            ;;
        --binary)
            BINARY_PATH="$2"
            shift 2
            ;;
        --fmu)
            FMU_PATH="$2"
            shift 2
            ;;
        --iterations)
            ITERATIONS="$2"
            shift 2
            ;;
        --help)
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  --no-build           Skip building the binary"
            echo "  --no-analysis        Skip binary size analysis"
            echo "  --binary PATH        Path to liaison binary (default: target/release/liaison)"
            echo "  --fmu PATH           Path to FMU file (default: BouncingBallLiaison.fmu)"
            echo "  --iterations N       Number of iterations (default: 10)"
            echo "  --help               Show this help message"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            echo "Use --help for usage information"
            exit 1
            ;;
    esac
done

echo -e "${BLUE}======================================${NC}"
echo -e "${BLUE}Liaison FMI Startup Benchmark${NC}"
echo -e "${BLUE}======================================${NC}"
echo ""

# Build the binary if requested
if [ "$BUILD_FIRST" = true ]; then
    echo -e "${GREEN}Building Rust binary in release mode...${NC}"
    cargo build --release --bin liaison
    echo -e "${GREEN}Build complete!${NC}"
    echo ""
fi

# Check if binary exists
if [ ! -f "$BINARY_PATH" ]; then
    echo -e "${RED}Error: Binary not found at $BINARY_PATH${NC}"
    echo "Run with --no-build if you want to use a pre-built binary"
    exit 1
fi

# Check if FMU exists
if [ ! -f "$FMU_PATH" ]; then
    echo -e "${RED}Error: FMU not found at $FMU_PATH${NC}"
    exit 1
fi

echo -e "${GREEN}Running startup benchmark...${NC}"
echo "Binary: $BINARY_PATH"
echo "FMU: $FMU_PATH"
echo "Iterations: $ITERATIONS"
echo ""

# Compile the benchmark binary
echo -e "${BLUE}Compiling benchmark...${NC}"
cd "$(dirname "$0")"
rustc startup_bench.rs -o startup_bench -O
echo ""

# Run the benchmark
./startup_bench --binary "../../$BINARY_PATH" --fmu "../../$FMU_PATH" --iterations "$ITERATIONS"

echo ""

# Run binary analysis if requested
if [ "$RUN_ANALYSIS" = true ]; then
    echo -e "${GREEN}Running binary size analysis...${NC}"
    echo ""
    ./binary_analysis.sh
fi

echo ""
echo -e "${GREEN}All benchmarks complete!${NC}"
echo ""
echo "Results saved to:"
echo "  - startup_results.json"
echo "  - binary_analysis_report.txt"
