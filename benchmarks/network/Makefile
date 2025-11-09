# Makefile for Liaison FMI Network Benchmarks
#
# Common targets:
#   make build        - Build benchmarks in release mode
#   make test         - Run quick smoke test
#   make bench        - Run full benchmarks
#   make compare      - Run comparison benchmarks
#   make clean        - Clean build artifacts
#   make help         - Show this help

# Configuration
FMU_PATH ?= ../../BouncingBallLiaison.fmu
RESPONDER_ID ?= test_responder
ITERATIONS ?= 100
DURATION ?= 10
CLIENTS ?= 16
FORMAT ?= csv
OUTPUT_DIR ?= ./results

# Build targets
.PHONY: build
build:
	@echo "Building network benchmarks..."
	cargo build --release

.PHONY: build-debug
build-debug:
	@echo "Building network benchmarks (debug)..."
	cargo build

# Test targets
.PHONY: test
test: build
	@echo "Running quick smoke test..."
	./target/release/latency_test $(FMU_PATH) $(RESPONDER_ID) --iterations 10
	./target/release/throughput_test $(FMU_PATH) $(RESPONDER_ID) --duration 5

# Benchmark targets
.PHONY: bench-latency
bench-latency: build
	@echo "Running latency benchmarks..."
	@mkdir -p $(OUTPUT_DIR)
	./target/release/latency_test $(FMU_PATH) $(RESPONDER_ID) \
		--iterations $(ITERATIONS) \
		--format $(FORMAT) \
		| tee $(OUTPUT_DIR)/latency_results_$(shell date +%Y%m%d_%H%M%S).$(FORMAT)

.PHONY: bench-throughput
bench-throughput: build
	@echo "Running throughput benchmarks..."
	@mkdir -p $(OUTPUT_DIR)
	./target/release/throughput_test $(FMU_PATH) $(RESPONDER_ID) \
		--duration $(DURATION) \
		--clients $(CLIENTS) \
		--format $(FORMAT) \
		| tee $(OUTPUT_DIR)/throughput_results_$(shell date +%Y%m%d_%H%M%S).$(FORMAT)

.PHONY: bench
bench: bench-latency bench-throughput

# Comparison target
.PHONY: compare
compare: build
	@echo "Running comparison benchmarks..."
	./compare_network.sh $(FMU_PATH) $(RESPONDER_ID) \
		--iterations $(ITERATIONS) \
		--duration $(DURATION) \
		--clients $(CLIENTS) \
		--format $(FORMAT) \
		--output-dir $(OUTPUT_DIR)

# Specific test categories
.PHONY: bench-lifecycle
bench-lifecycle: build
	@echo "Running lifecycle benchmarks..."
	./target/release/latency_test $(FMU_PATH) $(RESPONDER_ID) \
		--test lifecycle \
		--iterations $(ITERATIONS)

.PHONY: bench-data
bench-data: build
	@echo "Running data operation benchmarks..."
	./target/release/latency_test $(FMU_PATH) $(RESPONDER_ID) \
		--test data \
		--iterations $(ITERATIONS)

.PHONY: bench-simulation
bench-simulation: build
	@echo "Running simulation benchmarks..."
	./target/release/latency_test $(FMU_PATH) $(RESPONDER_ID) \
		--test simulation \
		--iterations $(ITERATIONS)

.PHONY: bench-concurrent
bench-concurrent: build
	@echo "Running concurrent client benchmarks..."
	./target/release/throughput_test $(FMU_PATH) $(RESPONDER_ID) \
		--test concurrent \
		--clients $(CLIENTS) \
		--duration $(DURATION)

.PHONY: bench-transfer
bench-transfer: build
	@echo "Running data transfer benchmarks..."
	./target/release/throughput_test $(FMU_PATH) $(RESPONDER_ID) \
		--test transfer \
		--duration $(DURATION)

# Output format targets
.PHONY: bench-csv
bench-csv:
	@$(MAKE) FORMAT=csv bench

.PHONY: bench-json
bench-json:
	@$(MAKE) FORMAT=json bench

.PHONY: bench-human
bench-human:
	@$(MAKE) FORMAT=human bench

# Utility targets
.PHONY: clean
clean:
	@echo "Cleaning build artifacts..."
	cargo clean

.PHONY: clean-results
clean-results:
	@echo "Cleaning benchmark results..."
	rm -rf $(OUTPUT_DIR)/*.csv $(OUTPUT_DIR)/*.json $(OUTPUT_DIR)/*.txt

.PHONY: clean-all
clean-all: clean clean-results

# Check targets
.PHONY: check
check:
	@echo "Checking code..."
	cargo check

.PHONY: clippy
clippy:
	@echo "Running clippy..."
	cargo clippy --all-targets -- -D warnings

.PHONY: fmt
fmt:
	@echo "Formatting code..."
	cargo fmt

.PHONY: fmt-check
fmt-check:
	@echo "Checking code formatting..."
	cargo fmt -- --check

# Verification
.PHONY: verify-fmu
verify-fmu:
	@echo "Verifying FMU file..."
	@if [ ! -f "$(FMU_PATH)" ]; then \
		echo "Error: FMU file not found at $(FMU_PATH)"; \
		exit 1; \
	else \
		echo "FMU file found: $(FMU_PATH)"; \
	fi

.PHONY: verify-server
verify-server:
	@echo "Verifying liaison server..."
	@if [ ! -f "../../target/release/liaison" ] && [ ! -f "../../target/debug/liaison" ]; then \
		echo "Warning: Liaison server binary not found. Building..."; \
		cd ../.. && cargo build --release; \
	else \
		echo "Liaison server binary found"; \
	fi

.PHONY: verify
verify: verify-fmu verify-server

# Installation
.PHONY: install
install: build
	@echo "Installing benchmarks..."
	@mkdir -p ~/.cargo/bin
	cp target/release/latency_test ~/.cargo/bin/
	cp target/release/throughput_test ~/.cargo/bin/
	@echo "Benchmarks installed to ~/.cargo/bin/"

# Documentation
.PHONY: docs
docs:
	@echo "Opening documentation..."
	@if command -v xdg-open > /dev/null; then \
		xdg-open README.md; \
	elif command -v open > /dev/null; then \
		open README.md; \
	else \
		cat README.md; \
	fi

.PHONY: quickstart
quickstart:
	@cat QUICKSTART.md

# Help
.PHONY: help
help:
	@echo "Liaison FMI Network Benchmarks - Make targets"
	@echo ""
	@echo "Build targets:"
	@echo "  make build              Build benchmarks in release mode"
	@echo "  make build-debug        Build benchmarks in debug mode"
	@echo ""
	@echo "Test targets:"
	@echo "  make test               Run quick smoke test"
	@echo ""
	@echo "Benchmark targets:"
	@echo "  make bench              Run all benchmarks"
	@echo "  make bench-latency      Run latency benchmarks only"
	@echo "  make bench-throughput   Run throughput benchmarks only"
	@echo "  make compare            Run comparison benchmarks"
	@echo ""
	@echo "Specific test categories:"
	@echo "  make bench-lifecycle    Test lifecycle operations"
	@echo "  make bench-data         Test data operations"
	@echo "  make bench-simulation   Test simulation operations"
	@echo "  make bench-concurrent   Test concurrent clients"
	@echo "  make bench-transfer     Test data transfer rates"
	@echo ""
	@echo "Output format targets:"
	@echo "  make bench-csv          Run benchmarks with CSV output"
	@echo "  make bench-json         Run benchmarks with JSON output"
	@echo "  make bench-human        Run benchmarks with human-readable output"
	@echo ""
	@echo "Utility targets:"
	@echo "  make clean              Clean build artifacts"
	@echo "  make clean-results      Clean benchmark results"
	@echo "  make clean-all          Clean everything"
	@echo "  make check              Check code"
	@echo "  make clippy             Run clippy linter"
	@echo "  make fmt                Format code"
	@echo "  make fmt-check          Check code formatting"
	@echo ""
	@echo "Verification targets:"
	@echo "  make verify             Verify FMU and server"
	@echo "  make verify-fmu         Verify FMU file exists"
	@echo "  make verify-server      Verify server binary exists"
	@echo ""
	@echo "Documentation:"
	@echo "  make docs               Open documentation"
	@echo "  make quickstart         Show quick start guide"
	@echo "  make help               Show this help"
	@echo ""
	@echo "Configuration (can be overridden):"
	@echo "  FMU_PATH=$(FMU_PATH)"
	@echo "  RESPONDER_ID=$(RESPONDER_ID)"
	@echo "  ITERATIONS=$(ITERATIONS)"
	@echo "  DURATION=$(DURATION)"
	@echo "  CLIENTS=$(CLIENTS)"
	@echo "  FORMAT=$(FORMAT)"
	@echo "  OUTPUT_DIR=$(OUTPUT_DIR)"
	@echo ""
	@echo "Examples:"
	@echo "  make bench FMU_PATH=./my.fmu ITERATIONS=1000"
	@echo "  make bench-csv OUTPUT_DIR=./my_results"
	@echo "  make compare DURATION=60 CLIENTS=32"

.DEFAULT_GOAL := help
