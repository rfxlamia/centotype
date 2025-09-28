#!/usr/bin/env python3
"""
Performance Regression Detection Script
Validates that performance targets are maintained across code changes.
"""

import sys
import re
import argparse
from pathlib import Path
from typing import Optional, Dict, Any

def parse_benchmark_results(results_file: Path) -> Dict[str, Any]:
    """Parse benchmark results from Criterion output."""
    if not results_file.exists():
        return {}

    content = results_file.read_text()
    metrics = {}

    # Parse P99 latency from input benchmark
    p99_match = re.search(r"input_latency.*?99th percentile:\s*(\d+(?:\.\d+)?)\s*(ns|μs|ms)", content, re.IGNORECASE)
    if p99_match:
        value = float(p99_match.group(1))
        unit = p99_match.group(2)

        # Convert to milliseconds
        if unit == "ns":
            value /= 1_000_000
        elif unit == "μs":
            value /= 1_000

        metrics["p99_latency_ms"] = value

    # Parse cache hit rate from content benchmark
    hit_rate_match = re.search(r"cache.*?hit rate:\s*(\d+(?:\.\d+)?)%", content, re.IGNORECASE)
    if hit_rate_match:
        metrics["cache_hit_rate"] = float(hit_rate_match.group(1)) / 100

    # Parse memory usage
    memory_match = re.search(r"memory usage:\s*(\d+(?:\.\d+)?)\s*(MB|GB)", content, re.IGNORECASE)
    if memory_match:
        value = float(memory_match.group(1))
        unit = memory_match.group(2)

        if unit == "GB":
            value *= 1024

        metrics["memory_usage_mb"] = value

    return metrics

def validate_performance_targets(metrics: Dict[str, Any], args: argparse.Namespace) -> bool:
    """Validate metrics against performance targets."""
    all_passed = True

    # Validate P99 latency target
    if "p99_latency_ms" in metrics:
        p99_latency = metrics["p99_latency_ms"]
        if p99_latency > args.max_p99_ms:
            print(f"❌ FAIL: P99 latency {p99_latency:.2f}ms exceeds target {args.max_p99_ms}ms")
            all_passed = False
        else:
            print(f"✅ PASS: P99 latency {p99_latency:.2f}ms within target {args.max_p99_ms}ms")

    # Validate cache hit rate
    if "cache_hit_rate" in metrics:
        hit_rate = metrics["cache_hit_rate"]
        if hit_rate < args.min_hit_rate:
            print(f"❌ FAIL: Cache hit rate {hit_rate:.1%} below target {args.min_hit_rate:.1%}")
            all_passed = False
        else:
            print(f"✅ PASS: Cache hit rate {hit_rate:.1%} meets target {args.min_hit_rate:.1%}")

    # Validate memory usage
    if "memory_usage_mb" in metrics:
        memory_usage = metrics["memory_usage_mb"]
        if memory_usage > args.max_memory_mb:
            print(f"❌ FAIL: Memory usage {memory_usage:.1f}MB exceeds target {args.max_memory_mb}MB")
            all_passed = False
        else:
            print(f"✅ PASS: Memory usage {memory_usage:.1f}MB within target {args.max_memory_mb}MB")

    return all_passed

def main():
    parser = argparse.ArgumentParser(description="Validate performance regression")
    parser.add_argument("--results-file", type=Path, default="bench_results.txt",
                        help="Benchmark results file")
    parser.add_argument("--max-p99-ms", type=float, default=25.0,
                        help="Maximum P99 latency in milliseconds")
    parser.add_argument("--min-hit-rate", type=float, default=0.90,
                        help="Minimum cache hit rate (0.0-1.0)")
    parser.add_argument("--max-memory-mb", type=float, default=50.0,
                        help="Maximum memory usage in MB")

    args = parser.parse_args()

    print("🚀 Performance Regression Validation")
    print(f"📊 Targets: P99 <{args.max_p99_ms}ms, Hit Rate >{args.min_hit_rate:.0%}, Memory <{args.max_memory_mb}MB")
    print()

    # Parse benchmark results
    metrics = parse_benchmark_results(args.results_file)

    if not metrics:
        print("⚠️  No performance metrics found in benchmark results")
        print("📋 Validation skipped - ensure benchmarks ran successfully")
        return 0

    # Validate against targets
    all_passed = validate_performance_targets(metrics, args)

    print()
    if all_passed:
        print("🎉 All performance targets met!")
        return 0
    else:
        print("💥 Performance regression detected!")
        print("📋 Review changes for performance impact and optimize before merge")
        return 1

if __name__ == "__main__":
    sys.exit(main())