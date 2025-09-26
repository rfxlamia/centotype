#!/bin/bash
set -euo pipefail

# Performance comparison script for pull requests
# Compares performance metrics between base and current branch

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
RESULTS_DIR="$PROJECT_ROOT/benchmark-results"

echo "=== Centotype Performance Comparison ==="

mkdir -p "$RESULTS_DIR"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

# Check if we're in a pull request context
if [[ -z "${GITHUB_BASE_REF:-}" ]]; then
    echo "⚠️  Not in a pull request context, running standalone benchmark"
    BASE_BRANCH="main"
else
    BASE_BRANCH="origin/$GITHUB_BASE_REF"
    echo "📊 Comparing against base branch: $BASE_BRANCH"
fi

# Function to run benchmarks and save results
run_benchmarks() {
    local label="$1"
    local output_prefix="$2"

    echo "Running benchmarks for: $label"

    # Build performance test binary
    cargo build --profile perf-test --bin centotype

    # Run startup time benchmark
    if [[ -f "scripts/benchmark_startup.sh" ]]; then
        echo "Running startup benchmark..."
        ./scripts/benchmark_startup.sh > /dev/null 2>&1 || true
        if [[ -f "$RESULTS_DIR/startup_benchmark.json" ]]; then
            cp "$RESULTS_DIR/startup_benchmark.json" "$RESULTS_DIR/${output_prefix}_startup.json"
        fi
    fi

    # Run input latency benchmark
    if [[ -f "scripts/benchmark_latency.sh" ]]; then
        echo "Running latency benchmark..."
        ./scripts/benchmark_latency.sh > /dev/null 2>&1 || true
        if [[ -f "$RESULTS_DIR/input_latency_benchmark.json" ]]; then
            cp "$RESULTS_DIR/input_latency_benchmark.json" "$RESULTS_DIR/${output_prefix}_latency.json"
        fi
    fi

    # Run Criterion benchmarks
    echo "Running criterion benchmarks..."
    cargo bench --workspace > "$RESULTS_DIR/${output_prefix}_criterion.log" 2>&1 || true
}

# Store current state
CURRENT_BRANCH=$(git rev-parse --abbrev-ref HEAD)
CURRENT_COMMIT=$(git rev-parse HEAD)

echo "Current branch: $CURRENT_BRANCH"
echo "Current commit: $CURRENT_COMMIT"

# Run benchmarks for current branch
echo
echo -e "${BLUE}=== Running Current Branch Benchmarks ===${NC}"
run_benchmarks "current branch" "current"

# If we have a base branch, compare against it
if git rev-parse --verify "$BASE_BRANCH" >/dev/null 2>&1; then
    echo
    echo -e "${BLUE}=== Running Base Branch Benchmarks ===${NC}"

    # Stash current changes if any
    if ! git diff --quiet || ! git diff --cached --quiet; then
        echo "Stashing current changes..."
        git stash push -m "Temporary stash for performance comparison"
        STASHED=true
    else
        STASHED=false
    fi

    # Checkout base branch
    git checkout "$BASE_BRANCH"
    run_benchmarks "base branch" "base"

    # Return to current branch
    git checkout "$CURRENT_BRANCH"

    # Restore stashed changes
    if [[ "$STASHED" == "true" ]]; then
        git stash pop
    fi

    # Compare results
    echo
    echo -e "${BLUE}=== Performance Comparison Results ===${NC}"

    # Function to compare JSON results
    compare_json_results() {
        local metric_name="$1"
        local base_file="$2"
        local current_file="$3"
        local threshold_field="$4"
        local result_field="$5"

        if [[ -f "$base_file" && -f "$current_file" ]]; then
            local base_value=$(jq -r ".results.$result_field" "$base_file" 2>/dev/null || echo "0")
            local current_value=$(jq -r ".results.$result_field" "$current_file" 2>/dev/null || echo "0")
            local threshold=$(jq -r ".thresholds.$threshold_field" "$current_file" 2>/dev/null || echo "0")

            if [[ "$base_value" != "null" && "$current_value" != "null" && "$base_value" != "0" ]]; then
                local change_percent=$(echo "scale=2; (($current_value - $base_value) / $base_value) * 100" | bc -l)

                echo -n "$metric_name: "
                printf "%.2fms → %.2fms " "$base_value" "$current_value"

                if (( $(echo "$change_percent > 5" | bc -l) )); then
                    echo -e "(${RED}+${change_percent}%${NC}) - REGRESSION"
                elif (( $(echo "$change_percent < -5" | bc -l) )); then
                    echo -e "(${GREEN}${change_percent}%${NC}) - IMPROVEMENT"
                else
                    echo -e "(${change_percent}%) - no significant change"
                fi

                # Check threshold
                if (( $(echo "$current_value > $threshold" | bc -l) )); then
                    echo -e "  ${RED}⚠️  Exceeds threshold: ${threshold}ms${NC}"
                fi
            else
                echo "$metric_name: Unable to compare (missing data)"
            fi
        else
            echo "$metric_name: No comparison data available"
        fi
    }

    # Compare startup times
    compare_json_results "Startup P95" \
        "$RESULTS_DIR/base_startup.json" \
        "$RESULTS_DIR/current_startup.json" \
        "p95_threshold_ms" \
        "p95"

    # Compare input latency
    compare_json_results "Input Latency P99" \
        "$RESULTS_DIR/base_latency.json" \
        "$RESULTS_DIR/current_latency.json" \
        "p99_threshold_ms" \
        "p99"

    # Analyze Criterion results if available
    if [[ -f "$RESULTS_DIR/base_criterion.log" && -f "$RESULTS_DIR/current_criterion.log" ]]; then
        echo
        echo "Criterion benchmark comparison:"
        echo "(Detailed analysis requires criterion report parsing)"
    fi

    # Generate summary report
    cat > "$RESULTS_DIR/comparison_summary.md" << EOF
# Performance Comparison Report

**Branch:** $CURRENT_BRANCH
**Base:** $BASE_BRANCH
**Timestamp:** $(date -u +%Y-%m-%dT%H:%M:%SZ)

## Results Summary

$(if [[ -f "$RESULTS_DIR/base_startup.json" && -f "$RESULTS_DIR/current_startup.json" ]]; then
    base_startup=$(jq -r '.results.p95' "$RESULTS_DIR/base_startup.json")
    current_startup=$(jq -r '.results.p95' "$RESULTS_DIR/current_startup.json")
    startup_change=$(echo "scale=2; (($current_startup - $base_startup) / $base_startup) * 100" | bc -l)
    echo "- **Startup Time (P95):** ${base_startup}ms → ${current_startup}ms (${startup_change}%)"
fi)

$(if [[ -f "$RESULTS_DIR/base_latency.json" && -f "$RESULTS_DIR/current_latency.json" ]]; then
    base_latency=$(jq -r '.results.p99' "$RESULTS_DIR/base_latency.json")
    current_latency=$(jq -r '.results.p99' "$RESULTS_DIR/current_latency.json")
    latency_change=$(echo "scale=2; (($current_latency - $base_latency) / $base_latency) * 100" | bc -l)
    echo "- **Input Latency (P99):** ${base_latency}ms → ${current_latency}ms (${latency_change}%)"
fi)

## Recommendations

- Review any regressions > 5%
- Ensure all metrics stay within defined thresholds
- Consider optimizations for significant slowdowns

---
*Generated by Centotype CI/CD performance comparison*
EOF

    echo
    echo "📄 Detailed report saved to: benchmark-results/comparison_summary.md"

else
    echo -e "${YELLOW}⚠️  Base branch not found, skipping comparison${NC}"
fi

echo
echo -e "${GREEN}✅ Performance comparison complete${NC}"