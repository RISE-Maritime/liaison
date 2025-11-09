#!/usr/bin/env python3
"""
Liaison FMI Performance Results Aggregator

This script collects benchmark results from various sources and generates
summary statistics, comparison tables, and exports to multiple formats.

Usage:
    python aggregate_results.py [options]

Options:
    --input-dir DIR         Directory containing benchmark results (default: ./benchmarks/results)
    --output-dir DIR        Directory for output files (default: ./benchmarks/reports)
    --format FORMAT         Output format: markdown, csv, json, html, all (default: all)
    --compare               Generate comparison charts (requires matplotlib)
    --verbose               Enable verbose output
"""

import argparse
import csv
import json
import os
import sys
from collections import defaultdict
from dataclasses import dataclass, asdict
from datetime import datetime
from pathlib import Path
from typing import Dict, List, Optional, Tuple
import statistics


@dataclass
class BenchmarkResult:
    """Represents a single benchmark measurement."""
    name: str
    implementation: str  # 'cpp' or 'rust'
    metric: str  # 'latency', 'throughput', 'memory', etc.
    value: float
    unit: str
    timestamp: str
    metadata: Dict = None

    def __post_init__(self):
        if self.metadata is None:
            self.metadata = {}


class ResultsAggregator:
    """Aggregates and analyzes benchmark results."""

    def __init__(self, input_dir: Path, output_dir: Path):
        self.input_dir = input_dir
        self.output_dir = output_dir
        self.results: List[BenchmarkResult] = []
        self.summary: Dict = {}

    def load_results(self) -> int:
        """Load all benchmark results from input directory."""
        count = 0

        # Try to load from JSON files
        json_files = list(self.input_dir.glob("**/*.json"))
        for json_file in json_files:
            try:
                with open(json_file, 'r') as f:
                    data = json.load(f)
                    if isinstance(data, list):
                        for item in data:
                            self.results.append(BenchmarkResult(**item))
                            count += 1
                    elif isinstance(data, dict) and 'results' in data:
                        for item in data['results']:
                            self.results.append(BenchmarkResult(**item))
                            count += 1
            except Exception as e:
                print(f"Warning: Failed to load {json_file}: {e}")

        # Try to load from CSV files
        csv_files = list(self.input_dir.glob("**/*.csv"))
        for csv_file in csv_files:
            try:
                with open(csv_file, 'r') as f:
                    reader = csv.DictReader(f)
                    for row in reader:
                        result = BenchmarkResult(
                            name=row.get('name', ''),
                            implementation=row.get('implementation', ''),
                            metric=row.get('metric', ''),
                            value=float(row.get('value', 0)),
                            unit=row.get('unit', ''),
                            timestamp=row.get('timestamp', datetime.now().isoformat()),
                            metadata={}
                        )
                        self.results.append(result)
                        count += 1
            except Exception as e:
                print(f"Warning: Failed to load {csv_file}: {e}")

        print(f"Loaded {count} benchmark results from {len(json_files) + len(csv_files)} files")
        return count

    def generate_sample_data(self) -> int:
        """Generate sample benchmark data for demonstration."""
        print("No benchmark data found. Generating sample data...")

        sample_benchmarks = [
            # FMI Function Latency (microseconds)
            ("fmi3GetVersion", "latency", 118, 121, "µs"),
            ("fmi3SetDebugLogging", "latency", 125, 128, "µs"),
            ("fmi3InstantiateCoSimulation", "latency", 1450, 1478, "µs"),
            ("fmi3FreeInstance", "latency", 342, 348, "µs"),
            ("fmi3EnterInitializationMode", "latency", 156, 159, "µs"),
            ("fmi3GetFloat64_1", "latency", 132, 135, "µs"),
            ("fmi3GetFloat64_10", "latency", 142, 145, "µs"),
            ("fmi3GetFloat64_100", "latency", 287, 293, "µs"),
            ("fmi3DoStep", "latency", 145, 148, "µs"),

            # Throughput (operations/second)
            ("DoStep_1ms", "throughput", 2847, 2819, "ops/sec"),
            ("DoStep_10ms", "throughput", 2921, 2905, "ops/sec"),
            ("DoStep_100ms", "throughput", 2953, 2948, "ops/sec"),

            # Memory Usage (MB)
            ("server_startup", "memory", 8.2, 8.5, "MB"),
            ("server_idle", "memory", 12.1, 12.4, "MB"),
            ("server_1_instance", "memory", 15.3, 15.6, "MB"),
            ("server_10_instances", "memory", 46.8, 47.2, "MB"),

            # Build Time (seconds)
            ("clean_build", "build_time", 35, 85, "seconds"),
            ("incremental_build", "build_time", 6, 5, "seconds"),

            # Binary Size (MB)
            ("server_binary", "binary_size", 8.2, 15.0, "MB"),
            ("client_library", "binary_size", 5.1, 13.0, "MB"),
        ]

        timestamp = datetime.now().isoformat()
        for name, metric, cpp_value, rust_value, unit in sample_benchmarks:
            # Add C++ result
            self.results.append(BenchmarkResult(
                name=name,
                implementation="cpp",
                metric=metric,
                value=cpp_value,
                unit=unit,
                timestamp=timestamp
            ))
            # Add Rust result
            self.results.append(BenchmarkResult(
                name=name,
                implementation="rust",
                metric=metric,
                value=rust_value,
                unit=unit,
                timestamp=timestamp
            ))

        print(f"Generated {len(self.results)} sample benchmark results")
        return len(self.results)

    def compute_statistics(self) -> Dict:
        """Compute summary statistics for all benchmarks."""
        stats = defaultdict(lambda: defaultdict(list))

        # Group results by name and implementation
        for result in self.results:
            key = (result.name, result.metric)
            stats[key][result.implementation].append(result.value)

        # Compute statistics
        summary = {}
        for (name, metric), implementations in stats.items():
            summary[name] = {
                'metric': metric,
                'implementations': {}
            }

            for impl, values in implementations.items():
                summary[name]['implementations'][impl] = {
                    'mean': statistics.mean(values) if values else 0,
                    'median': statistics.median(values) if values else 0,
                    'min': min(values) if values else 0,
                    'max': max(values) if values else 0,
                    'count': len(values),
                }

                # Add standard deviation if we have enough samples
                if len(values) >= 2:
                    summary[name]['implementations'][impl]['stddev'] = statistics.stdev(values)

            # Compute comparison if both implementations exist
            if 'cpp' in implementations and 'rust' in implementations:
                cpp_mean = summary[name]['implementations']['cpp']['mean']
                rust_mean = summary[name]['implementations']['rust']['mean']
                if cpp_mean > 0:
                    diff_pct = ((rust_mean - cpp_mean) / cpp_mean) * 100
                    summary[name]['comparison'] = {
                        'rust_vs_cpp_pct': diff_pct,
                        'winner': 'cpp' if diff_pct > 0 else 'rust' if diff_pct < 0 else 'tie'
                    }

        self.summary = summary
        return summary

    def export_markdown(self, output_path: Path) -> None:
        """Export results as Markdown tables."""
        with open(output_path, 'w') as f:
            f.write("# Benchmark Results Summary\n\n")
            f.write(f"**Generated:** {datetime.now().strftime('%Y-%m-%d %H:%M:%S')}\n\n")
            f.write(f"**Total Benchmarks:** {len(self.summary)}\n\n")

            # Group by metric type
            metrics = defaultdict(list)
            for name, data in self.summary.items():
                metrics[data['metric']].append((name, data))

            for metric_type, benchmarks in sorted(metrics.items()):
                f.write(f"\n## {metric_type.replace('_', ' ').title()} Benchmarks\n\n")
                f.write("| Benchmark | C++ | Rust | Difference | Winner |\n")
                f.write("|-----------|-----|------|------------|--------|\n")

                for name, data in sorted(benchmarks):
                    cpp_val = data['implementations'].get('cpp', {}).get('mean', 0)
                    rust_val = data['implementations'].get('rust', {}).get('mean', 0)
                    comp = data.get('comparison', {})
                    diff = comp.get('rust_vs_cpp_pct', 0)
                    winner = comp.get('winner', 'N/A')

                    # Get unit from first result with this name
                    unit = next((r.unit for r in self.results if r.name == name), '')

                    f.write(f"| {name} | {cpp_val:.2f} {unit} | {rust_val:.2f} {unit} | "
                           f"{diff:+.1f}% | {winner.upper()} |\n")

            # Add summary statistics
            f.write("\n## Summary Statistics\n\n")

            # Calculate overall averages
            latency_diffs = []
            throughput_diffs = []
            memory_diffs = []

            for name, data in self.summary.items():
                comp = data.get('comparison', {})
                if comp:
                    diff = comp['rust_vs_cpp_pct']
                    metric = data['metric']
                    if metric == 'latency':
                        latency_diffs.append(diff)
                    elif metric == 'throughput':
                        throughput_diffs.append(diff)
                    elif metric == 'memory':
                        memory_diffs.append(diff)

            f.write("| Metric Category | Average Difference (Rust vs C++) |\n")
            f.write("|----------------|-----------------------------------|\n")
            if latency_diffs:
                f.write(f"| Latency | {statistics.mean(latency_diffs):+.2f}% |\n")
            if throughput_diffs:
                f.write(f"| Throughput | {statistics.mean(throughput_diffs):+.2f}% |\n")
            if memory_diffs:
                f.write(f"| Memory | {statistics.mean(memory_diffs):+.2f}% |\n")

        print(f"Exported Markdown to {output_path}")

    def export_csv(self, output_path: Path) -> None:
        """Export results as CSV."""
        with open(output_path, 'w', newline='') as f:
            writer = csv.writer(f)
            writer.writerow(['Benchmark', 'Metric', 'Implementation', 'Mean', 'Median',
                           'Min', 'Max', 'StdDev', 'Unit'])

            for name, data in sorted(self.summary.items()):
                metric = data['metric']
                # Get unit from first result
                unit = next((r.unit for r in self.results if r.name == name), '')

                for impl, stats in data['implementations'].items():
                    writer.writerow([
                        name,
                        metric,
                        impl,
                        f"{stats['mean']:.4f}",
                        f"{stats['median']:.4f}",
                        f"{stats['min']:.4f}",
                        f"{stats['max']:.4f}",
                        f"{stats.get('stddev', 0):.4f}",
                        unit
                    ])

        print(f"Exported CSV to {output_path}")

    def export_json(self, output_path: Path) -> None:
        """Export results as JSON."""
        output = {
            'generated': datetime.now().isoformat(),
            'total_benchmarks': len(self.summary),
            'summary': self.summary,
            'raw_results': [asdict(r) for r in self.results]
        }

        with open(output_path, 'w') as f:
            json.dump(output, f, indent=2)

        print(f"Exported JSON to {output_path}")

    def export_html(self, output_path: Path) -> None:
        """Export results as HTML with embedded charts."""
        html_template = """<!DOCTYPE html>
<html lang="en">
<head>
    <meta charset="UTF-8">
    <meta name="viewport" content="width=device-width, initial-scale=1.0">
    <title>Liaison FMI Benchmark Results</title>
    <style>
        body {{
            font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
            max-width: 1200px;
            margin: 0 auto;
            padding: 20px;
            background: #f5f5f5;
        }}
        .header {{
            background: white;
            padding: 20px;
            border-radius: 8px;
            margin-bottom: 20px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }}
        h1 {{ margin: 0; color: #333; }}
        .meta {{ color: #666; margin-top: 10px; }}
        .section {{
            background: white;
            padding: 20px;
            border-radius: 8px;
            margin-bottom: 20px;
            box-shadow: 0 2px 4px rgba(0,0,0,0.1);
        }}
        table {{
            width: 100%;
            border-collapse: collapse;
            margin-top: 10px;
        }}
        th, td {{
            padding: 12px;
            text-align: left;
            border-bottom: 1px solid #ddd;
        }}
        th {{
            background: #f8f9fa;
            font-weight: 600;
        }}
        .winner-rust {{ color: #28a745; font-weight: 600; }}
        .winner-cpp {{ color: #dc3545; font-weight: 600; }}
        .winner-tie {{ color: #6c757d; }}
        .positive {{ color: #dc3545; }}
        .negative {{ color: #28a745; }}
        .chart {{
            margin: 20px 0;
            height: 300px;
        }}
    </style>
</head>
<body>
    <div class="header">
        <h1>Liaison FMI Benchmark Results</h1>
        <div class="meta">
            <strong>Generated:</strong> {generated}<br>
            <strong>Total Benchmarks:</strong> {total_benchmarks}
        </div>
    </div>

    {sections}

    <div class="section">
        <h2>Summary Statistics</h2>
        <table>
            <tr>
                <th>Metric Category</th>
                <th>Average Difference (Rust vs C++)</th>
            </tr>
            {summary_rows}
        </table>
    </div>
</body>
</html>"""

        # Generate sections for each metric type
        metrics = defaultdict(list)
        for name, data in self.summary.items():
            metrics[data['metric']].append((name, data))

        sections_html = ""
        for metric_type, benchmarks in sorted(metrics.items()):
            sections_html += f'<div class="section"><h2>{metric_type.replace("_", " ").title()} Benchmarks</h2>'
            sections_html += '<table><tr><th>Benchmark</th><th>C++</th><th>Rust</th><th>Difference</th><th>Winner</th></tr>'

            for name, data in sorted(benchmarks):
                cpp_val = data['implementations'].get('cpp', {}).get('mean', 0)
                rust_val = data['implementations'].get('rust', {}).get('mean', 0)
                comp = data.get('comparison', {})
                diff = comp.get('rust_vs_cpp_pct', 0)
                winner = comp.get('winner', 'N/A')
                unit = next((r.unit for r in self.results if r.name == name), '')

                diff_class = 'positive' if diff > 0 else 'negative' if diff < 0 else ''
                winner_class = f'winner-{winner}'

                sections_html += f'<tr><td>{name}</td><td>{cpp_val:.2f} {unit}</td>'
                sections_html += f'<td>{rust_val:.2f} {unit}</td>'
                sections_html += f'<td class="{diff_class}">{diff:+.1f}%</td>'
                sections_html += f'<td class="{winner_class}">{winner.upper()}</td></tr>'

            sections_html += '</table></div>'

        # Calculate summary statistics
        latency_diffs = []
        throughput_diffs = []
        memory_diffs = []

        for name, data in self.summary.items():
            comp = data.get('comparison', {})
            if comp:
                diff = comp['rust_vs_cpp_pct']
                metric = data['metric']
                if metric == 'latency':
                    latency_diffs.append(diff)
                elif metric == 'throughput':
                    throughput_diffs.append(diff)
                elif metric == 'memory':
                    memory_diffs.append(diff)

        summary_rows = ""
        if latency_diffs:
            avg = statistics.mean(latency_diffs)
            summary_rows += f'<tr><td>Latency</td><td>{avg:+.2f}%</td></tr>'
        if throughput_diffs:
            avg = statistics.mean(throughput_diffs)
            summary_rows += f'<tr><td>Throughput</td><td>{avg:+.2f}%</td></tr>'
        if memory_diffs:
            avg = statistics.mean(memory_diffs)
            summary_rows += f'<tr><td>Memory</td><td>{avg:+.2f}%</td></tr>'

        html = html_template.format(
            generated=datetime.now().strftime('%Y-%m-%d %H:%M:%S'),
            total_benchmarks=len(self.summary),
            sections=sections_html,
            summary_rows=summary_rows
        )

        with open(output_path, 'w') as f:
            f.write(html)

        print(f"Exported HTML to {output_path}")

    def generate_comparison_chart(self, output_path: Path) -> None:
        """Generate comparison charts using matplotlib."""
        try:
            import matplotlib.pyplot as plt
            import numpy as np
        except ImportError:
            print("Warning: matplotlib not available. Skipping chart generation.")
            print("Install with: pip install matplotlib")
            return

        # Group benchmarks by metric
        metrics = defaultdict(lambda: {'names': [], 'cpp': [], 'rust': []})

        for name, data in self.summary.items():
            if 'comparison' not in data:
                continue

            metric = data['metric']
            cpp_val = data['implementations'].get('cpp', {}).get('mean', 0)
            rust_val = data['implementations'].get('rust', {}).get('mean', 0)

            metrics[metric]['names'].append(name)
            metrics[metric]['cpp'].append(cpp_val)
            metrics[metric]['rust'].append(rust_val)

        # Create a figure with subplots for each metric
        num_metrics = len(metrics)
        fig, axes = plt.subplots(num_metrics, 1, figsize=(12, 6 * num_metrics))

        if num_metrics == 1:
            axes = [axes]

        for ax, (metric, data) in zip(axes, sorted(metrics.items())):
            x = np.arange(len(data['names']))
            width = 0.35

            ax.bar(x - width/2, data['cpp'], width, label='C++', color='#dc3545')
            ax.bar(x + width/2, data['rust'], width, label='Rust', color='#007bff')

            ax.set_xlabel('Benchmark')
            ax.set_ylabel('Value')
            ax.set_title(f'{metric.replace("_", " ").title()} Comparison')
            ax.set_xticks(x)
            ax.set_xticklabels(data['names'], rotation=45, ha='right')
            ax.legend()
            ax.grid(axis='y', alpha=0.3)

        plt.tight_layout()
        plt.savefig(output_path, dpi=150, bbox_inches='tight')
        print(f"Generated comparison chart: {output_path}")


def main():
    """Main entry point."""
    parser = argparse.ArgumentParser(
        description='Aggregate and analyze Liaison FMI benchmark results',
        formatter_class=argparse.RawDescriptionHelpFormatter,
        epilog=__doc__
    )
    parser.add_argument(
        '--input-dir',
        type=Path,
        default=Path('./benchmarks/results'),
        help='Directory containing benchmark results (default: ./benchmarks/results)'
    )
    parser.add_argument(
        '--output-dir',
        type=Path,
        default=Path('./benchmarks/reports'),
        help='Directory for output files (default: ./benchmarks/reports)'
    )
    parser.add_argument(
        '--format',
        choices=['markdown', 'csv', 'json', 'html', 'all'],
        default='all',
        help='Output format (default: all)'
    )
    parser.add_argument(
        '--compare',
        action='store_true',
        help='Generate comparison charts (requires matplotlib)'
    )
    parser.add_argument(
        '--verbose',
        action='store_true',
        help='Enable verbose output'
    )

    args = parser.parse_args()

    # Create output directory if it doesn't exist
    args.output_dir.mkdir(parents=True, exist_ok=True)

    # Initialize aggregator
    aggregator = ResultsAggregator(args.input_dir, args.output_dir)

    # Load results or generate sample data
    count = aggregator.load_results()
    if count == 0:
        aggregator.generate_sample_data()

    # Compute statistics
    print("Computing statistics...")
    aggregator.compute_statistics()

    # Export results
    timestamp = datetime.now().strftime('%Y%m%d_%H%M%S')

    formats = [args.format] if args.format != 'all' else ['markdown', 'csv', 'json', 'html']

    for fmt in formats:
        output_file = args.output_dir / f'benchmark_results_{timestamp}.{fmt if fmt != "markdown" else "md"}'

        if fmt == 'markdown':
            aggregator.export_markdown(output_file)
        elif fmt == 'csv':
            aggregator.export_csv(output_file)
        elif fmt == 'json':
            aggregator.export_json(output_file)
        elif fmt == 'html':
            aggregator.export_html(output_file)

    # Generate comparison charts if requested
    if args.compare:
        chart_file = args.output_dir / f'comparison_chart_{timestamp}.png'
        aggregator.generate_comparison_chart(chart_file)

    print(f"\n✓ Results aggregation complete!")
    print(f"  Total benchmarks: {len(aggregator.summary)}")
    print(f"  Output directory: {args.output_dir}")


if __name__ == '__main__':
    main()
