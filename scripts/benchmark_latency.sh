#!/bin/bash
set -euo pipefail

# Input Latency Benchmark Script
# Validates P99 input latency < 25ms requirement

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
RESULTS_DIR="$PROJECT_ROOT/benchmark-results"
BINARY_PATH="$PROJECT_ROOT/target/perf-test/centotype"

# Configuration
ITERATIONS=1000
PERCENTILE_99_THRESHOLD_MS=25
TEST_DURATION_SECONDS=30

mkdir -p "$RESULTS_DIR"

echo "=== Centotype Input Latency Benchmark ==="
echo "Binary: $BINARY_PATH"
echo "Iterations: $ITERATIONS"
echo "P99 Threshold: ${PERCENTILE_99_THRESHOLD_MS}ms"
echo "Test Duration: ${TEST_DURATION_SECONDS}s"
echo

# Ensure binary exists
if [[ ! -f "$BINARY_PATH" ]]; then
    echo "Building performance test binary..."
    cd "$PROJECT_ROOT"
    cargo build --profile perf-test --bin centotype
fi

# Run input latency benchmark
echo "Running input latency benchmark..."
RESULTS_FILE="$RESULTS_DIR/input_latency.txt"

# Use the built-in latency benchmark mode
timeout $((TEST_DURATION_SECONDS + 10))s "$BINARY_PATH" \
    --benchmark-mode \
    --benchmark-type input-latency \
    --benchmark-iterations "$ITERATIONS" \
    --benchmark-output "$RESULTS_FILE" || {
    echo "ERROR: Benchmark execution failed"
    exit 1
}

# Analyze results
echo
echo "=== Input Latency Analysis ==="

if [[ ! -f "$RESULTS_FILE" || ! -s "$RESULTS_FILE" ]]; then
    echo "ERROR: No latency measurements collected"
    exit 1
fi

# Calculate statistics
STATS=$(awk '
BEGIN {
    count = 0;
    sum = 0;
    min = 999999;
    max = 0;
}
{
    latencies[count] = $1;
    sum += $1;
    if ($1 < min) min = $1;
    if ($1 > max) max = $1;
    count++;
}
END {
    if (count == 0) exit 1;

    # Sort array for percentiles
    for (i = 0; i < count; i++) {
        for (j = i + 1; j < count; j++) {
            if (latencies[i] > latencies[j]) {
                temp = latencies[i];
                latencies[i] = latencies[j];
                latencies[j] = temp;
            }
        }
    }

    avg = sum / count;
    p50_idx = int(count * 0.5);
    p95_idx = int(count * 0.95);
    p99_idx = int(count * 0.99);

    printf "%.3f %.3f %.3f %.3f %.3f %.3f %d",
           avg, latencies[p50_idx], latencies[p95_idx], latencies[p99_idx], min, max, count;
}' "$RESULTS_FILE")

if [[ -z "$STATS" ]]; then
    echo "ERROR: Failed to calculate latency statistics"
    exit 1
fi

read -r AVG P50 P95 P99 MIN MAX COUNT <<< "$STATS"

echo "Measurements: $COUNT"
echo "Average: ${AVG}ms"
echo "P50 (median): ${P50}ms"
echo "P95: ${P95}ms"
echo "P99: ${P99}ms"
echo "Min: ${MIN}ms"
echo "Max: ${MAX}ms"

# Generate JSON report
cat > "$RESULTS_DIR/input_latency_benchmark.json" << EOF
{
    "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
    "test": "input_latency",
    "iterations": $COUNT,
    "unit": "milliseconds",
    "results": {
        "average": $AVG,
        "p50": $P50,
        "p95": $P95,
        "p99": $P99,
        "min": $MIN,
        "max": $MAX
    },
    "thresholds": {
        "p99_threshold_ms": $PERCENTILE_99_THRESHOLD_MS
    },
    "passed": $(if (( $(echo "$P99 <= $PERCENTILE_99_THRESHOLD_MS" | bc -l) )); then echo "true"; else echo "false"; fi)
}
EOF

# Validation
echo
echo "=== Performance Validation ==="
if (( $(echo "$P99 <= $PERCENTILE_99_THRESHOLD_MS" | bc -l) )); then
    echo "✅ PASS: P99 input latency (${P99}ms) is within threshold (${PERCENTILE_99_THRESHOLD_MS}ms)"
    exit 0
else
    echo "❌ FAIL: P99 input latency (${P99}ms) exceeds threshold (${PERCENTILE_99_THRESHOLD_MS}ms)"
    echo "Performance regression detected!"
    exit 1
fi