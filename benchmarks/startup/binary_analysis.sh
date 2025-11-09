#!/bin/bash

# Binary size comparison script for Liaison FMI
# Compares Rust vs C++ binary sizes and analyzes sections

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Default paths
RUST_BINARY="${RUST_BINARY:-target/release/liaison}"
RUST_LIB="${RUST_LIB:-target/release/libliaisonfmu.so}"
CPP_LIB="${CPP_LIB:-binaries/x86_64-linux/libliaisonfmu.so}"
RUST_BINARY_DEBUG="${RUST_BINARY_DEBUG:-target/debug/liaison}"

echo -e "${BLUE}======================================${NC}"
echo -e "${BLUE}Liaison FMI Binary Size Analysis${NC}"
echo -e "${BLUE}======================================${NC}"
echo ""

# Function to format bytes
format_bytes() {
    local bytes=$1
    if [ $bytes -ge 1048576 ]; then
        echo "$(awk "BEGIN {printf \"%.2f\", $bytes/1048576}") MB"
    elif [ $bytes -ge 1024 ]; then
        echo "$(awk "BEGIN {printf \"%.2f\", $bytes/1024}") KB"
    else
        echo "$bytes B"
    fi
}

# Function to get file size
get_size() {
    local file=$1
    if [ -f "$file" ]; then
        stat -c%s "$file" 2>/dev/null || stat -f%z "$file" 2>/dev/null || echo "0"
    else
        echo "0"
    fi
}

# Function to analyze binary sections
analyze_sections() {
    local binary=$1
    local title=$2

    echo -e "\n${GREEN}=== $title ===${NC}"
    echo ""

    if [ ! -f "$binary" ]; then
        echo -e "${RED}File not found: $binary${NC}"
        return 1
    fi

    # Get total size
    local total_size=$(get_size "$binary")
    echo "Total Size: $(format_bytes $total_size)"

    # Check if stripped
    if file "$binary" | grep -q "not stripped"; then
        echo -e "Debug Info: ${YELLOW}Present (not stripped)${NC}"
    else
        echo -e "Debug Info: ${GREEN}Stripped${NC}"
    fi

    # Analyze sections using size command
    echo ""
    echo "Section Breakdown:"
    if command -v size &> /dev/null; then
        size "$binary" | tail -n 1 | while read text data bss dec hex filename; do
            echo "  .text (code):    $(format_bytes $text)"
            echo "  .data:           $(format_bytes $data)"
            echo "  .bss:            $(format_bytes $bss)"
            echo "  Total sections:  $(format_bytes $dec)"
        done
    fi

    # Detailed section analysis with readelf
    if command -v readelf &> /dev/null; then
        echo ""
        echo "Detailed Sections (top 10 by size):"
        readelf -S "$binary" | grep '\[' | awk '{print $7, $2}' | \
            grep -E '^[0-9a-f]+' | \
            while read size name; do
                printf "  %-20s %10s\n" "$name" "$(format_bytes $((16#$size)))"
            done | sort -k2 -rh | head -10
    fi

    # Symbol table size
    if command -v nm &> /dev/null; then
        echo ""
        local symbol_count=$(nm "$binary" 2>/dev/null | wc -l)
        echo "Symbol Count: $symbol_count"
    fi

    return 0
}

# Function to compare two binaries
compare_binaries() {
    local file1=$1
    local file2=$2
    local name1=$3
    local name2=$4

    if [ ! -f "$file1" ] || [ ! -f "$file2" ]; then
        echo -e "${RED}Cannot compare: one or both files not found${NC}"
        return 1
    fi

    local size1=$(get_size "$file1")
    local size2=$(get_size "$file2")

    echo -e "\n${GREEN}=== Comparison: $name1 vs $name2 ===${NC}"
    echo ""
    printf "%-20s %15s\n" "$name1:" "$(format_bytes $size1)"
    printf "%-20s %15s\n" "$name2:" "$(format_bytes $size2)"
    echo ""

    # Calculate difference
    local diff=$((size1 - size2))
    local percent=$(awk "BEGIN {printf \"%.1f\", ($diff / $size2) * 100}")

    if [ $diff -gt 0 ]; then
        echo -e "$name1 is ${RED}$(format_bytes ${diff#-}) larger${NC} ($percent%)"
    elif [ $diff -lt 0 ]; then
        echo -e "$name1 is ${GREEN}$(format_bytes ${diff#-}) smaller${NC} ($percent%)"
    else
        echo -e "Binaries are ${GREEN}the same size${NC}"
    fi
}

# Function to strip binary and compare
strip_and_compare() {
    local binary=$1
    local name=$2

    if [ ! -f "$binary" ]; then
        echo -e "${RED}File not found: $binary${NC}"
        return 1
    fi

    local original_size=$(get_size "$binary")
    local stripped_binary="${binary}.stripped"

    # Create stripped copy
    cp "$binary" "$stripped_binary"
    strip "$stripped_binary"

    local stripped_size=$(get_size "$stripped_binary")
    local debug_size=$((original_size - stripped_size))
    local percent=$(awk "BEGIN {printf \"%.1f\", ($debug_size / $original_size) * 100}")

    echo -e "\n${GREEN}=== Strip Analysis: $name ===${NC}"
    echo ""
    printf "%-20s %15s\n" "Original:" "$(format_bytes $original_size)"
    printf "%-20s %15s\n" "Stripped:" "$(format_bytes $stripped_size)"
    printf "%-20s %15s (${YELLOW}%s%%%${NC})\n" "Debug Info:" "$(format_bytes $debug_size)" "$percent"

    # Clean up
    rm "$stripped_binary"
}

# Main analysis
echo "Analyzing Rust Binaries..."

# Analyze Rust release binary
analyze_sections "$RUST_BINARY" "Rust Release Binary (liaison)"

# Analyze Rust release library
if [ -f "$RUST_LIB" ]; then
    analyze_sections "$RUST_LIB" "Rust Release Library (libliaisonfmu.so)"
fi

# Analyze Rust debug binary
if [ -f "$RUST_BINARY_DEBUG" ]; then
    analyze_sections "$RUST_BINARY_DEBUG" "Rust Debug Binary (liaison)"
fi

# Analyze C++ library
if [ -f "$CPP_LIB" ]; then
    analyze_sections "$CPP_LIB" "C++ Library (libliaisonfmu.so)"
fi

echo ""
echo -e "${BLUE}======================================${NC}"
echo -e "${BLUE}Comparisons${NC}"
echo -e "${BLUE}======================================${NC}"

# Compare Rust release vs debug
if [ -f "$RUST_BINARY" ] && [ -f "$RUST_BINARY_DEBUG" ]; then
    compare_binaries "$RUST_BINARY" "$RUST_BINARY_DEBUG" "Rust Release" "Rust Debug"
fi

# Compare Rust lib vs C++ lib
if [ -f "$RUST_LIB" ] && [ -f "$CPP_LIB" ]; then
    compare_binaries "$RUST_LIB" "$CPP_LIB" "Rust Lib" "C++ Lib"
fi

# Strip analysis
if [ -f "$RUST_BINARY" ]; then
    strip_and_compare "$RUST_BINARY" "Rust Release Binary"
fi

if [ -f "$RUST_LIB" ]; then
    strip_and_compare "$RUST_LIB" "Rust Release Library"
fi

echo ""
echo -e "${BLUE}======================================${NC}"
echo -e "${BLUE}Trade-offs Analysis${NC}"
echo -e "${BLUE}======================================${NC}"
echo ""

cat << 'EOF'
Rust vs C++ Binary Size Trade-offs:

PROS (Rust):
  + Memory safety without runtime overhead
  + Zero-cost abstractions
  + Modern dependency management
  + Better error handling patterns
  + Excellent tooling (cargo, clippy, rustfmt)

CONS (Rust):
  - Larger binary sizes (typically 20-50% larger)
  - Includes more standard library code
  - Generic code monomorphization increases size
  - Debug symbols are larger

OPTIMIZATION STRATEGIES:
  1. Use LTO (Link Time Optimization) - already enabled
  2. Strip debug symbols in production
  3. Set opt-level = 'z' for size optimization
  4. Use codegen-units = 1 - already enabled
  5. Consider using cargo-bloat to identify large dependencies

CURRENT CONFIGURATION:
  - LTO: enabled
  - codegen-units: 1
  - strip: true (in release profile)
  - opt-level: 3 (performance over size)

SIZE RECOMMENDATIONS:
  - For development: Use debug builds
  - For production: Use release builds with strip=true
  - For embedded: Consider opt-level='z' and panic='abort'
EOF

echo ""
echo -e "${GREEN}Analysis complete!${NC}"
echo ""

# Save results to file
{
    echo "Liaison FMI Binary Analysis Report"
    echo "Generated: $(date)"
    echo ""
    echo "Rust Release Binary: $(format_bytes $(get_size "$RUST_BINARY"))"
    [ -f "$RUST_LIB" ] && echo "Rust Release Library: $(format_bytes $(get_size "$RUST_LIB"))"
    [ -f "$CPP_LIB" ] && echo "C++ Library: $(format_bytes $(get_size "$CPP_LIB"))"
    [ -f "$RUST_BINARY_DEBUG" ] && echo "Rust Debug Binary: $(format_bytes $(get_size "$RUST_BINARY_DEBUG"))"
} > binary_analysis_report.txt

echo "Report saved to binary_analysis_report.txt"
