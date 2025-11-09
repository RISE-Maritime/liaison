#!/bin/bash
# Script to check clippy warnings after fixes

echo "Checking clippy warnings for liaison-fmi crate..."
cd liaison-fmi
cargo clippy --all-features 2>&1 | tee clippy_output.txt

echo ""
echo "Checking if code compiles..."
cargo build --all-features 2>&1 | tee build_output.txt

echo ""
echo "Results saved to clippy_output.txt and build_output.txt"
