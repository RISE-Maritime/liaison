# Makefile for Liaison FMI Memory Profiling
#
# Common targets:
#   make profile        - Run full memory profiling
#   make quick-profile  - Run quick memory profiling
#   make test           - Run memory tests without profiling
#   make analyze        - Analyze latest results
#   make compare        - Compare with baseline
#   make clean          - Clean results
#   make help           - Show this help

.PHONY: help profile quick-profile test analyze compare plot clean install-deps

# Variables
WORKSPACE_ROOT := $(shell cd .. && pwd)
LATEST_RESULT := $(shell ls -td results/*/ 2>/dev/null | head -1)
BASELINE := results/baseline

# Colors
BLUE := \033[0;34m
GREEN := \033[0;32m
YELLOW := \033[1;33m
NC := \033[0m # No Color

help:
	@echo "$(BLUE)Liaison FMI Memory Profiling Tools$(NC)"
	@echo ""
	@echo "Available targets:"
	@echo "  $(GREEN)make profile$(NC)        - Run full memory profiling (server + client)"
	@echo "  $(GREEN)make quick-profile$(NC)  - Run quick memory profiling"
	@echo "  $(GREEN)make server-profile$(NC) - Profile server only"
	@echo "  $(GREEN)make client-profile$(NC) - Profile client library only"
	@echo "  $(GREEN)make test$(NC)           - Run memory tests (no Valgrind)"
	@echo "  $(GREEN)make test-massif$(NC)    - Run tests with Massif profiling"
	@echo "  $(GREEN)make analyze$(NC)        - Analyze latest profiling results"
	@echo "  $(GREEN)make compare$(NC)        - Compare latest with baseline"
	@echo "  $(GREEN)make plot$(NC)           - Generate plots from latest results"
	@echo "  $(GREEN)make baseline$(NC)       - Set current results as baseline"
	@echo "  $(GREEN)make clean$(NC)          - Remove all results"
	@echo "  $(GREEN)make install-deps$(NC)   - Install required dependencies"
	@echo ""
	@echo "Environment:"
	@echo "  Workspace: $(WORKSPACE_ROOT)"
	@echo "  Latest:    $(LATEST_RESULT)"
	@echo "  Baseline:  $(BASELINE)"

profile:
	@echo "$(BLUE)Running full memory profiling...$(NC)"
	./memory_profile.sh
	@echo "$(GREEN)Profiling complete!$(NC)"
	@$(MAKE) --no-print-directory analyze

quick-profile:
	@echo "$(BLUE)Running quick memory profiling...$(NC)"
	./memory_profile.sh --quick
	@echo "$(GREEN)Quick profiling complete!$(NC)"
	@$(MAKE) --no-print-directory analyze

server-profile:
	@echo "$(BLUE)Profiling server only...$(NC)"
	./memory_profile.sh --server-only

client-profile:
	@echo "$(BLUE)Profiling client library only...$(NC)"
	./memory_profile.sh --client-only

test:
	@echo "$(BLUE)Running memory tests...$(NC)"
	cd $(WORKSPACE_ROOT) && cargo test --test memory_test --release -- --nocapture

test-massif:
	@echo "$(BLUE)Running memory tests with Massif...$(NC)"
	@mkdir -p results/manual
	cd $(WORKSPACE_ROOT) && valgrind --tool=massif \
		--massif-out-file=$(WORKSPACE_ROOT)/benchmarks/results/manual/massif.out \
		cargo test --test memory_test --release -- --nocapture
	@echo "$(GREEN)Results saved to results/manual/massif.out$(NC)"
	@ms_print results/manual/massif.out | head -50

test-leak:
	@echo "$(BLUE)Running memory leak detection...$(NC)"
	cd $(WORKSPACE_ROOT) && valgrind --leak-check=full --show-leak-kinds=all \
		--track-origins=yes --verbose \
		cargo test --test memory_test --release test_memory_leak_detection -- --nocapture

analyze:
	@if [ -z "$(LATEST_RESULT)" ]; then \
		echo "$(YELLOW)No results found. Run 'make profile' first.$(NC)"; \
		exit 1; \
	fi
	@echo "$(BLUE)Analyzing: $(LATEST_RESULT)$(NC)"
	python3 analyze_memory.py $(LATEST_RESULT) --verbose
	@echo ""
	@echo "View detailed report: cat $(LATEST_RESULT)ANALYSIS.md"

compare:
	@if [ -z "$(LATEST_RESULT)" ]; then \
		echo "$(YELLOW)No results found. Run 'make profile' first.$(NC)"; \
		exit 1; \
	fi
	@if [ ! -d "$(BASELINE)" ]; then \
		echo "$(YELLOW)No baseline found. Run 'make baseline' first.$(NC)"; \
		exit 1; \
	fi
	@echo "$(BLUE)Comparing with baseline...$(NC)"
	python3 analyze_memory.py $(LATEST_RESULT) --compare $(BASELINE) --verbose
	@echo ""
	@echo "View comparison: cat $(LATEST_RESULT)COMPARISON.md"

plot:
	@if [ -z "$(LATEST_RESULT)" ]; then \
		echo "$(YELLOW)No results found. Run 'make profile' first.$(NC)"; \
		exit 1; \
	fi
	@echo "$(BLUE)Generating plots...$(NC)"
	python3 analyze_memory.py $(LATEST_RESULT) --plot
	@echo "$(GREEN)Plot saved to: $(LATEST_RESULT)memory_usage.png$(NC)"

export-csv:
	@if [ -z "$(LATEST_RESULT)" ]; then \
		echo "$(YELLOW)No results found. Run 'make profile' first.$(NC)"; \
		exit 1; \
	fi
	@echo "$(BLUE)Exporting to CSV...$(NC)"
	python3 analyze_memory.py $(LATEST_RESULT) --export-csv
	@echo "$(GREEN)CSV files saved to: $(LATEST_RESULT)csv/$(NC)"

baseline:
	@if [ -z "$(LATEST_RESULT)" ]; then \
		echo "$(YELLOW)No results found. Run 'make profile' first.$(NC)"; \
		exit 1; \
	fi
	@echo "$(BLUE)Setting baseline...$(NC)"
	@rm -rf $(BASELINE)
	@cp -r $(LATEST_RESULT) $(BASELINE)
	@echo "$(GREEN)Baseline set to: $(LATEST_RESULT)$(NC)"

list-results:
	@echo "$(BLUE)Available results:$(NC)"
	@ls -lhd results/*/ 2>/dev/null || echo "No results found"

show-latest:
	@if [ -z "$(LATEST_RESULT)" ]; then \
		echo "$(YELLOW)No results found.$(NC)"; \
		exit 1; \
	fi
	@echo "$(BLUE)Latest results:$(NC)"
	@cat $(LATEST_RESULT)SUMMARY.md

clean:
	@echo "$(YELLOW)Cleaning all results (except baseline)...$(NC)"
	@find results -mindepth 1 -maxdepth 1 -type d ! -name baseline -exec rm -rf {} +
	@echo "$(GREEN)Cleaned!$(NC)"

clean-all:
	@echo "$(YELLOW)Cleaning ALL results (including baseline)...$(NC)"
	@rm -rf results
	@echo "$(GREEN)All results removed!$(NC)"

install-deps:
	@echo "$(BLUE)Installing dependencies...$(NC)"
	@echo "Checking for apt-get..."
	@if command -v apt-get >/dev/null 2>&1; then \
		echo "Installing Valgrind..."; \
		sudo apt-get update && sudo apt-get install -y valgrind; \
		echo "Installing Python matplotlib (optional)..."; \
		pip3 install matplotlib || echo "$(YELLOW)Failed to install matplotlib (optional)$(NC)"; \
	else \
		echo "$(YELLOW)apt-get not found. Please install valgrind manually.$(NC)"; \
	fi
	@echo "$(GREEN)Dependencies check complete!$(NC)"

check-deps:
	@echo "$(BLUE)Checking dependencies...$(NC)"
	@command -v valgrind >/dev/null 2>&1 && echo "  ✓ valgrind" || echo "  ✗ valgrind (required)"
	@command -v ms_print >/dev/null 2>&1 && echo "  ✓ ms_print" || echo "  ✗ ms_print (required)"
	@command -v cargo >/dev/null 2>&1 && echo "  ✓ cargo" || echo "  ✗ cargo (required)"
	@command -v python3 >/dev/null 2>&1 && echo "  ✓ python3" || echo "  ✗ python3 (required)"
	@python3 -c "import matplotlib" 2>/dev/null && echo "  ✓ matplotlib" || echo "  ℹ matplotlib (optional)"
	@command -v heaptrack >/dev/null 2>&1 && echo "  ✓ heaptrack" || echo "  ℹ heaptrack (optional)"

# Continuous profiling (for CI/CD)
ci-profile:
	@echo "$(BLUE)Running CI memory profiling...$(NC)"
	./memory_profile.sh --quick
	@if [ -d "$(BASELINE)" ]; then \
		$(MAKE) --no-print-directory compare; \
	else \
		echo "$(YELLOW)No baseline for comparison$(NC)"; \
	fi

# Development workflow
dev-check: quick-profile analyze
	@echo "$(GREEN)Quick development check complete!$(NC)"

# Full analysis with all outputs
full-analysis:
	@if [ -z "$(LATEST_RESULT)" ]; then \
		echo "$(YELLOW)No results found. Run 'make profile' first.$(NC)"; \
		exit 1; \
	fi
	@echo "$(BLUE)Running full analysis...$(NC)"
	python3 analyze_memory.py $(LATEST_RESULT) --export-csv --plot --verbose
	@if [ -d "$(BASELINE)" ]; then \
		python3 analyze_memory.py $(LATEST_RESULT) --compare $(BASELINE) --verbose; \
	fi
	@echo "$(GREEN)Full analysis complete!$(NC)"
	@echo ""
	@echo "Generated files:"
	@ls -lh $(LATEST_RESULT)
