#!/bin/bash

# Performance benchmark execution script for CI/CD
# This script coordinates the execution of all benchmark suites and validates results

set -euo pipefail

# Configuration
BENCHMARK_TIMEOUT=1800  # 30 minutes
RESULTS_DIR="./performance_results"
TIMESTAMP=$(date +%Y%m%d_%H%M%S)
COMMIT_HASH=${GITHUB_SHA:-$(git rev-parse HEAD)}
BRANCH=${GITHUB_REF_NAME:-$(git branch --show-current)}

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

log_info() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

log_warn() {
    echo -e "${YELLOW}[WARN]${NC} $1"
}

log_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

log_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

# Cleanup function
cleanup() {
    log_info "Cleaning up temporary files..."
    # Kill any background processes
    jobs -p | xargs -r kill 2>/dev/null || true
}

trap cleanup EXIT

# Ensure results directory exists
mkdir -p "$RESULTS_DIR"

log_info "Starting comprehensive performance validation"
log_info "Commit: $COMMIT_HASH"
log_info "Branch: $BRANCH"
log_info "Timestamp: $TIMESTAMP"

# Check system resources
log_info "System resource check:"
echo "CPU cores: $(nproc)"
echo "Memory: $(free -h | awk '/^Mem:/ {print $2}')"
echo "Disk space: $(df -h . | awk 'NR==2 {print $4}')"

# Build optimized binary for performance testing
log_info "Building optimized performance test binary..."
cargo build --release --profile perf-test
if [ $? -ne 0 ]; then
    log_error "Failed to build performance test binary"
    exit 1
fi

# Function to run benchmark with timeout and error handling
run_benchmark() {
    local benchmark_name="$1"
    local benchmark_command="$2"
    local output_file="$3"

    log_info "Running $benchmark_name benchmark..."

    if timeout "$BENCHMARK_TIMEOUT" bash -c "$benchmark_command" > "$output_file" 2>&1; then
        log_success "$benchmark_name benchmark completed successfully"
        return 0
    else
        local exit_code=$?
        if [ $exit_code -eq 124 ]; then
            log_error "$benchmark_name benchmark timed out after ${BENCHMARK_TIMEOUT}s"
        else
            log_error "$benchmark_name benchmark failed with exit code $exit_code"
        fi
        return $exit_code
    fi
}

# Run individual benchmark suites
BENCHMARK_FAILURES=0

# Input Latency Benchmarks
if ! run_benchmark "Input Latency" \
    "cargo bench --bench input_latency_benchmark -- --output-format json" \
    "$RESULTS_DIR/input_latency_${TIMESTAMP}.json"; then
    ((BENCHMARK_FAILURES++))
fi

# Render Performance Benchmarks
if ! run_benchmark "Render Performance" \
    "cargo bench --bench render_performance_benchmark -- --output-format json" \
    "$RESULTS_DIR/render_performance_${TIMESTAMP}.json"; then
    ((BENCHMARK_FAILURES++))
fi

# Content System Benchmarks
if ! run_benchmark "Content System" \
    "cargo bench --bench content_performance_benchmark -- --output-format json" \
    "$RESULTS_DIR/content_performance_${TIMESTAMP}.json"; then
    ((BENCHMARK_FAILURES++))
fi

# Memory and Concurrency Benchmarks
if ! run_benchmark "Memory and Concurrency" \
    "cargo bench --bench memory_concurrency_benchmark -- --output-format json" \
    "$RESULTS_DIR/memory_concurrency_${TIMESTAMP}.json"; then
    ((BENCHMARK_FAILURES++))
fi

# Cross-crate Integration Benchmarks
if ! run_benchmark "Cross-crate Integration" \
    "cargo test --release performance_validation::test_ -- --nocapture" \
    "$RESULTS_DIR/integration_${TIMESTAMP}.json"; then
    ((BENCHMARK_FAILURES++))
fi

# System-level performance tests
log_info "Running system-level performance tests..."

# Memory leak detection
if command -v valgrind >/dev/null 2>&1; then
    log_info "Running memory leak detection with Valgrind..."
    valgrind --tool=memcheck --leak-check=full --show-leak-kinds=all \
        --track-origins=yes --log-file="$RESULTS_DIR/valgrind_${TIMESTAMP}.log" \
        target/perf-test/centotype --test-mode --duration 30 2>/dev/null || {
        log_warn "Valgrind memory check completed with warnings (check log)"
    }
else
    log_warn "Valgrind not available, skipping memory leak detection"
fi

# Startup time measurement
log_info "Measuring startup time..."
startup_times=()
for i in {1..10}; do
    start_time=$(date +%s%N)
    target/perf-test/centotype --test-mode --quick-exit >/dev/null 2>&1
    end_time=$(date +%s%N)
    startup_time=$(( (end_time - start_time) / 1000000 )) # Convert to milliseconds
    startup_times+=($startup_time)
done

# Calculate startup time statistics
startup_times_sorted=($(printf "%s\n" "${startup_times[@]}" | sort -n))
startup_p95_index=$(( ${#startup_times_sorted[@]} * 95 / 100 ))
startup_p95=${startup_times_sorted[$startup_p95_index]}

echo "{\"p95_startup_time_ms\": $startup_p95}" > "$RESULTS_DIR/startup_time_${TIMESTAMP}.json"
log_info "Startup time P95: ${startup_p95}ms"

# Generate comprehensive performance report
log_info "Generating comprehensive performance report..."
cargo run --release --bin performance_reporter -- \
    --results-dir "$RESULTS_DIR" \
    --commit-hash "$COMMIT_HASH" \
    --branch "$BRANCH" \
    --timestamp "$TIMESTAMP" \
    --output-file "$RESULTS_DIR/comprehensive_report_${TIMESTAMP}.json"

# Validate performance targets
log_info "Validating performance targets..."
if cargo run --release --bin performance_validator -- \
    --input-file "$RESULTS_DIR/comprehensive_report_${TIMESTAMP}.json" \
    --targets-file .github/performance_targets.json \
    --output-file "$RESULTS_DIR/validation_result_${TIMESTAMP}.json"; then
    log_success "All performance targets met!"
    VALIDATION_PASSED=true
else
    log_error "Performance validation failed!"
    VALIDATION_PASSED=false
fi

# Performance regression analysis
if [ -d ".performance_data" ] && [ "$(ls -A .performance_data)" ]; then
    log_info "Running performance regression analysis..."
    cargo run --release --bin regression_analyzer -- \
        --current-file "$RESULTS_DIR/comprehensive_report_${TIMESTAMP}.json" \
        --historical-dir ".performance_data" \
        --output-file "$RESULTS_DIR/regression_analysis_${TIMESTAMP}.json"
else
    log_warn "No historical performance data found, skipping regression analysis"
fi

# Generate human-readable summary
log_info "Generating performance summary..."
cat > "$RESULTS_DIR/summary_${TIMESTAMP}.md" << EOF
# Performance Validation Summary

**Commit:** \`$COMMIT_HASH\`
**Branch:** \`$BRANCH\`
**Timestamp:** $TIMESTAMP
**Validation Status:** $([ "$VALIDATION_PASSED" = true ] && echo "✅ PASSED" || echo "❌ FAILED")

## Benchmark Execution Results

- Input Latency Benchmarks: $([ -f "$RESULTS_DIR/input_latency_${TIMESTAMP}.json" ] && echo "✅ Completed" || echo "❌ Failed")
- Render Performance Benchmarks: $([ -f "$RESULTS_DIR/render_performance_${TIMESTAMP}.json" ] && echo "✅ Completed" || echo "❌ Failed")
- Content System Benchmarks: $([ -f "$RESULTS_DIR/content_performance_${TIMESTAMP}.json" ] && echo "✅ Completed" || echo "❌ Failed")
- Memory and Concurrency Benchmarks: $([ -f "$RESULTS_DIR/memory_concurrency_${TIMESTAMP}.json" ] && echo "✅ Completed" || echo "❌ Failed")

## System Information

- **OS:** $(uname -s)
- **Architecture:** $(uname -m)
- **CPU Cores:** $(nproc)
- **Memory:** $(free -h | awk '/^Mem:/ {print $2}')
- **Rust Version:** $(rustc --version)

## Files Generated

EOF

# List all generated files
find "$RESULTS_DIR" -name "*${TIMESTAMP}*" -type f | while read -r file; do
    echo "- \`$(basename "$file")\`" >> "$RESULTS_DIR/summary_${TIMESTAMP}.md"
done

# Final status report
echo
log_info "Performance validation completed!"
echo "Results directory: $RESULTS_DIR"
echo "Summary file: $RESULTS_DIR/summary_${TIMESTAMP}.md"
echo "Benchmark failures: $BENCHMARK_FAILURES"
echo "Validation passed: $VALIDATION_PASSED"

# Exit with appropriate code
if [ "$VALIDATION_PASSED" = true ] && [ $BENCHMARK_FAILURES -eq 0 ]; then
    log_success "All performance tests passed successfully!"
    exit 0
elif [ "$VALIDATION_PASSED" = false ]; then
    log_error "Performance validation failed - targets not met"
    exit 1
else
    log_error "Some benchmarks failed to execute"
    exit 2
fi