#!/bin/bash

# Run initialization benchmark for Liaison FMI
# Automates building and running initialization performance tests

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Default configuration
FMU_PATH="BouncingBallLiaison.fmu"
ITERATIONS=100

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
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
            echo "  --fmu PATH           Path to FMU file (default: BouncingBallLiaison.fmu)"
            echo "  --iterations N       Number of iterations (default: 100)"
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
echo -e "${BLUE}Liaison FMI Initialization Benchmark${NC}"
echo -e "${BLUE}======================================${NC}"
echo ""

# Check if FMU exists
if [ ! -f "$FMU_PATH" ]; then
    echo -e "${RED}Error: FMU not found at $FMU_PATH${NC}"
    exit 1
fi

echo -e "${GREEN}Running initialization benchmark...${NC}"
echo "FMU: $FMU_PATH"
echo "Iterations: $ITERATIONS"
echo ""

# Compile the benchmark binary
echo -e "${BLUE}Compiling benchmark...${NC}"
cd "$(dirname "$0")"
rustc init_bench.rs -o init_bench -O
echo ""

# Run the benchmark
./init_bench --fmu "../../$FMU_PATH" --iterations "$ITERATIONS"

echo ""
echo -e "${GREEN}Benchmark complete!${NC}"
echo ""
echo "Results saved to init_results.json"
