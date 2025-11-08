# Makefile for Liaison Integration Tests

.PHONY: all build test clean mock-fmu help

# Default target
all: build test

# Build the liaison server
build:
	@echo "Building liaison server..."
	@cd .. && cargo build --bin liaison

# Build just the mock FMU
mock-fmu:
	@echo "Building mock FMU..."
	@cd fixtures && ./build_mock_fmu.sh

# Run all integration tests
test: build
	@echo "Running integration tests..."
	@cd .. && cargo test --test integration_test

# Run tests with output
test-verbose: build
	@echo "Running integration tests (verbose)..."
	@cd .. && cargo test --test integration_test -- --nocapture

# Run tests with debug logging
test-debug: build
	@echo "Running integration tests (debug)..."
	@cd .. && RUST_LOG=debug cargo test --test integration_test -- --nocapture

# Run a specific test
test-one: build
	@echo "Running specific test: $(TEST)"
	@cd .. && cargo test --test integration_test $(TEST) -- --nocapture

# Clean build artifacts
clean:
	@echo "Cleaning build artifacts..."
	@rm -rf fixtures/build
	@cd .. && cargo clean

# Show help
help:
	@echo "Liaison Integration Tests - Makefile"
	@echo ""
	@echo "Available targets:"
	@echo "  make build         - Build the liaison server"
	@echo "  make mock-fmu      - Build the mock FMU only"
	@echo "  make test          - Run all integration tests"
	@echo "  make test-verbose  - Run tests with output"
	@echo "  make test-debug    - Run tests with debug logging"
	@echo "  make test-one      - Run specific test (use TEST=test_name)"
	@echo "  make clean         - Clean build artifacts"
	@echo "  make help          - Show this help message"
	@echo ""
	@echo "Examples:"
	@echo "  make test"
	@echo "  make test-debug"
	@echo "  make test-one TEST=test_full_simulation_cycle"
