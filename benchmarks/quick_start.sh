#!/bin/bash
# Quick Start Guide for Liaison FMI Memory Profiling
#
# This script walks you through the memory profiling tools

set -euo pipefail

# Colors
BLUE='\033[0;34m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${BLUE}"
cat << "EOF"
╔══════════════════════════════════════════════════════════════╗
║                                                              ║
║     Liaison FMI Memory Profiling - Quick Start Guide        ║
║                                                              ║
╚══════════════════════════════════════════════════════════════╝
EOF
echo -e "${NC}"

# Check if we're in the right directory
if [[ ! -f "memory_profile.sh" ]]; then
    echo -e "${RED}Error: Please run this script from the benchmarks directory${NC}"
    exit 1
fi

echo -e "${GREEN}Welcome to the Liaison FMI Memory Profiling Tools!${NC}"
echo ""
echo "This guide will help you get started with memory profiling."
echo ""

# Step 1: Check dependencies
echo -e "${BLUE}Step 1: Checking dependencies...${NC}"
echo ""

missing_deps=()

if ! command -v valgrind &> /dev/null; then
    echo -e "${RED}✗${NC} valgrind not found"
    missing_deps+=("valgrind")
else
    echo -e "${GREEN}✓${NC} valgrind found"
fi

if ! command -v cargo &> /dev/null; then
    echo -e "${RED}✗${NC} cargo not found"
    missing_deps+=("cargo")
else
    echo -e "${GREEN}✓${NC} cargo found"
fi

if ! command -v python3 &> /dev/null; then
    echo -e "${RED}✗${NC} python3 not found"
    missing_deps+=("python3")
else
    echo -e "${GREEN}✓${NC} python3 found"
fi

if [[ ${#missing_deps[@]} -gt 0 ]]; then
    echo ""
    echo -e "${YELLOW}Missing dependencies: ${missing_deps[*]}${NC}"
    echo ""
    read -p "Would you like to install them now? (y/n) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        make install-deps
    else
        echo -e "${YELLOW}Please install missing dependencies and try again.${NC}"
        exit 1
    fi
fi

echo ""
echo -e "${GREEN}All dependencies are installed!${NC}"
echo ""

# Step 2: Choose profiling mode
echo -e "${BLUE}Step 2: Choose profiling mode${NC}"
echo ""
echo "1) Quick profile (5-10 seconds, good for development)"
echo "2) Full profile (30-60 seconds, comprehensive)"
echo "3) Server only"
echo "4) Client only"
echo "5) Skip to test examples"
echo ""

read -p "Select an option (1-5): " -n 1 -r
echo

case $REPLY in
    1)
        echo ""
        echo -e "${BLUE}Running quick profile...${NC}"
        ./memory_profile.sh --quick
        ;;
    2)
        echo ""
        echo -e "${BLUE}Running full profile...${NC}"
        ./memory_profile.sh
        ;;
    3)
        echo ""
        echo -e "${BLUE}Profiling server only...${NC}"
        ./memory_profile.sh --server-only
        ;;
    4)
        echo ""
        echo -e "${BLUE}Profiling client only...${NC}"
        ./memory_profile.sh --client-only
        ;;
    5)
        echo ""
        echo -e "${YELLOW}Skipping to examples...${NC}"
        ;;
    *)
        echo -e "${RED}Invalid option${NC}"
        exit 1
        ;;
esac

# Step 3: Analyze results
if [[ $REPLY != "5" ]]; then
    echo ""
    echo -e "${BLUE}Step 3: Analyzing results...${NC}"
    echo ""

    LATEST=$(ls -td results/*/ 2>/dev/null | head -1)

    if [[ -n "$LATEST" ]]; then
        echo "Results saved to: $LATEST"
        echo ""
        echo "Summary:"
        echo "----------------------------------------"
        cat "${LATEST}SUMMARY.md"
        echo ""

        read -p "Would you like to see detailed analysis? (y/n) " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            python3 analyze_memory.py "$LATEST" --verbose
        fi

        echo ""
        read -p "Generate plots? (requires matplotlib) (y/n) " -n 1 -r
        echo
        if [[ $REPLY =~ ^[Yy]$ ]]; then
            python3 analyze_memory.py "$LATEST" --plot && \
                echo -e "${GREEN}Plot saved to: ${LATEST}memory_usage.png${NC}" || \
                echo -e "${YELLOW}Failed to generate plot. Install matplotlib: pip3 install matplotlib${NC}"
        fi
    fi
fi

# Step 4: Examples and next steps
echo ""
echo -e "${BLUE}Step 4: Examples and next steps${NC}"
echo ""
echo "Here are some useful commands:"
echo ""
echo -e "${GREEN}# Run memory tests manually${NC}"
echo "  cargo test --test memory_test --release -- --nocapture"
echo ""
echo -e "${GREEN}# Profile specific test${NC}"
echo "  valgrind --tool=massif cargo test --test memory_test --release test_memory_leak_detection"
echo ""
echo -e "${GREEN}# Use make targets${NC}"
echo "  make help              # Show all available targets"
echo "  make quick-profile     # Quick profiling"
echo "  make baseline          # Set current as baseline"
echo "  make compare           # Compare with baseline"
echo ""
echo -e "${GREEN}# Continuous profiling${NC}"
echo "  make ci-profile        # For CI/CD pipelines"
echo ""
echo -e "${YELLOW}TIP: Read README.md for detailed documentation${NC}"
echo ""

# Offer to create baseline
if [[ -n "${LATEST:-}" ]] && [[ ! -d "results/baseline" ]]; then
    echo ""
    read -p "Set this run as baseline for future comparisons? (y/n) " -n 1 -r
    echo
    if [[ $REPLY =~ ^[Yy]$ ]]; then
        make baseline
        echo -e "${GREEN}Baseline set!${NC}"
    fi
fi

echo ""
echo -e "${GREEN}Quick start complete!${NC}"
echo ""
echo "For more information, see:"
echo "  - README.md for full documentation"
echo "  - make help for available commands"
echo "  - memory_profile.sh --help for profiling options"
echo ""
echo -e "${BLUE}Happy profiling!${NC}"
