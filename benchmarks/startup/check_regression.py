#!/usr/bin/env python3
"""
Performance regression checker for Liaison FMI benchmarks.
Compares current benchmark results against baseline to detect regressions.
"""

import json
import sys
from pathlib import Path
from typing import Dict, List, Tuple

# Regression thresholds (percentage increase that triggers failure)
THRESHOLDS = {
    "total_startup_ms": 20.0,      # 20% slower startup is a regression
    "fmu_loading_ms": 30.0,        # 30% slower FMU loading is a regression
    "zenoh_init_ms": 25.0,         # 25% slower Zenoh init is a regression
    "first_request_ms": 40.0,      # 40% slower cold start is acceptable
    "warm_request_ms": 30.0,       # 30% slower warm request is a regression
    "binary_size_mb": 50.0,        # 50% larger binary is a concern
    "fmu_instantiation_ms": 30.0,  # 30% slower instantiation
    "config_loading_ms": 25.0,     # 25% slower config loading
    "total_init_ms": 25.0,         # 25% slower total init
}

# Warning thresholds (percentage increase that triggers warning)
WARNING_THRESHOLDS = {
    "total_startup_ms": 10.0,
    "fmu_loading_ms": 15.0,
    "zenoh_init_ms": 12.0,
    "first_request_ms": 20.0,
    "warm_request_ms": 15.0,
    "binary_size_mb": 25.0,
    "fmu_instantiation_ms": 15.0,
    "config_loading_ms": 12.0,
    "total_init_ms": 12.0,
}


class Colors:
    """ANSI color codes for terminal output."""
    RED = '\033[0;31m'
    GREEN = '\033[0;32m'
    YELLOW = '\033[1;33m'
    BLUE = '\033[0;34m'
    NC = '\033[0m'  # No Color


def load_json_file(filepath: Path) -> Dict:
    """Load and parse a JSON file."""
    try:
        with open(filepath, 'r') as f:
            return json.load(f)
    except Exception as e:
        print(f"{Colors.RED}Error loading {filepath}: {e}{Colors.NC}")
        return {}


def compare_metrics(baseline: Dict, current: Dict, metric_name: str) -> Tuple[float, str]:
    """
    Compare a metric between baseline and current.
    Returns (percentage_change, status) where status is 'pass', 'warn', or 'fail'.
    """
    if metric_name not in baseline or metric_name not in current:
        return 0.0, 'unknown'

    baseline_val = baseline[metric_name]
    current_val = current[metric_name]

    if baseline_val == 0:
        return 0.0, 'unknown'

    percent_change = ((current_val - baseline_val) / baseline_val) * 100.0

    if metric_name in THRESHOLDS:
        if percent_change > THRESHOLDS[metric_name]:
            return percent_change, 'fail'
        elif percent_change > WARNING_THRESHOLDS.get(metric_name, float('inf')):
            return percent_change, 'warn'

    return percent_change, 'pass'


def check_startup_regression(baseline_file: Path, current_file: Path) -> bool:
    """Check for startup performance regressions."""
    print(f"\n{Colors.BLUE}=== Checking Startup Performance ==={Colors.NC}\n")

    baseline = load_json_file(baseline_file)
    current = load_json_file(current_file)

    if not baseline or not current:
        print(f"{Colors.RED}Failed to load startup results{Colors.NC}")
        return False

    metrics = [
        "total_startup_ms",
        "fmu_loading_ms",
        "zenoh_init_ms",
        "first_request_ms",
        "warm_request_ms",
        "binary_size_mb",
    ]

    has_regression = False
    has_warning = False

    for metric in metrics:
        percent_change, status = compare_metrics(baseline, current, metric)

        if metric in baseline and metric in current:
            baseline_val = baseline[metric]
            current_val = current[metric]

            # Format the metric name
            display_name = metric.replace('_', ' ').title()

            if status == 'fail':
                print(f"{Colors.RED}FAIL{Colors.NC} {display_name:30s} "
                      f"{baseline_val:>10.2f} -> {current_val:>10.2f} "
                      f"({percent_change:+.1f}%)")
                has_regression = True
            elif status == 'warn':
                print(f"{Colors.YELLOW}WARN{Colors.NC} {display_name:30s} "
                      f"{baseline_val:>10.2f} -> {current_val:>10.2f} "
                      f"({percent_change:+.1f}%)")
                has_warning = True
            else:
                print(f"{Colors.GREEN}PASS{Colors.NC} {display_name:30s} "
                      f"{baseline_val:>10.2f} -> {current_val:>10.2f} "
                      f"({percent_change:+.1f}%)")

    return not has_regression


def check_init_regression(baseline_file: Path, current_file: Path) -> bool:
    """Check for initialization performance regressions."""
    print(f"\n{Colors.BLUE}=== Checking Initialization Performance ==={Colors.NC}\n")

    baseline = load_json_file(baseline_file)
    current = load_json_file(current_file)

    if not baseline or not current:
        print(f"{Colors.RED}Failed to load init results{Colors.NC}")
        return False

    # Check average metrics
    baseline_avg = baseline.get("average_metrics", {})
    current_avg = current.get("average_metrics", {})

    metrics = [
        "fmu_instantiation_ms",
        "config_loading_ms",
        "variable_init_ms",
        "setup_experiment_ms",
        "enter_init_mode_ms",
        "exit_init_mode_ms",
        "cleanup_ms",
        "total_init_ms",
    ]

    has_regression = False
    has_warning = False

    for metric in metrics:
        percent_change, status = compare_metrics(baseline_avg, current_avg, metric)

        if metric in baseline_avg and metric in current_avg:
            baseline_val = baseline_avg[metric]
            current_val = current_avg[metric]

            display_name = metric.replace('_', ' ').title()

            if status == 'fail':
                print(f"{Colors.RED}FAIL{Colors.NC} {display_name:30s} "
                      f"{baseline_val:>10.2f} -> {current_val:>10.2f} "
                      f"({percent_change:+.1f}%)")
                has_regression = True
            elif status == 'warn':
                print(f"{Colors.YELLOW}WARN{Colors.NC} {display_name:30s} "
                      f"{baseline_val:>10.2f} -> {current_val:>10.2f} "
                      f"({percent_change:+.1f}%)")
                has_warning = True
            else:
                print(f"{Colors.GREEN}PASS{Colors.NC} {display_name:30s} "
                      f"{baseline_val:>10.2f} -> {current_val:>10.2f} "
                      f"({percent_change:+.1f}%)")

    return not has_regression


def main():
    """Main entry point."""
    if len(sys.argv) < 2:
        print("Usage: check_regression.py <baseline_dir> [<current_dir>]")
        print("\nExample:")
        print("  check_regression.py benchmark_results_baseline/")
        print("  check_regression.py baseline/ current/")
        sys.exit(1)

    baseline_dir = Path(sys.argv[1])
    current_dir = Path(sys.argv[2]) if len(sys.argv) > 2 else Path(".")

    if not baseline_dir.exists():
        print(f"{Colors.RED}Baseline directory not found: {baseline_dir}{Colors.NC}")
        sys.exit(1)

    print(f"{Colors.BLUE}========================================{Colors.NC}")
    print(f"{Colors.BLUE}Performance Regression Check{Colors.NC}")
    print(f"{Colors.BLUE}========================================{Colors.NC}")
    print(f"\nBaseline: {baseline_dir}")
    print(f"Current:  {current_dir}")

    # Check startup performance
    startup_baseline = baseline_dir / "startup_results.json"
    startup_current = current_dir / "startup_results.json"

    startup_ok = True
    if startup_baseline.exists() and startup_current.exists():
        startup_ok = check_startup_regression(startup_baseline, startup_current)
    else:
        print(f"\n{Colors.YELLOW}Skipping startup check (files not found){Colors.NC}")

    # Check initialization performance
    init_baseline = baseline_dir / "init_results.json"
    init_current = current_dir / "init_results.json"

    init_ok = True
    if init_baseline.exists() and init_current.exists():
        init_ok = check_init_regression(init_baseline, init_current)
    else:
        print(f"\n{Colors.YELLOW}Skipping init check (files not found){Colors.NC}")

    # Final verdict
    print(f"\n{Colors.BLUE}========================================{Colors.NC}")
    if startup_ok and init_ok:
        print(f"{Colors.GREEN}✓ No performance regressions detected{Colors.NC}")
        sys.exit(0)
    else:
        print(f"{Colors.RED}✗ Performance regressions detected{Colors.NC}")
        sys.exit(1)


if __name__ == "__main__":
    main()
