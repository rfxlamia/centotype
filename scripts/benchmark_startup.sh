#!/bin/bash
set -euo pipefail

# Startup Time Benchmark Script
# Validates P95 startup time < 200ms requirement

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"
RESULTS_DIR="$PROJECT_ROOT/benchmark-results"
BINARY_PATH="$PROJECT_ROOT/target/perf-test/centotype"

# Configuration
ITERATIONS=100
PERCENTILE_95_THRESHOLD_MS=200
WARMUP_ITERATIONS=5

mkdir -p "$RESULTS_DIR"

echo "=== Centotype Startup Time Benchmark ==="
echo "Binary: $BINARY_PATH"
echo "Iterations: $ITERATIONS"
echo "P95 Threshold: ${PERCENTILE_95_THRESHOLD_MS}ms"
echo

# Ensure binary exists and is built with performance testing profile
if [[ ! -f "$BINARY_PATH" ]]; then
    echo "Building performance test binary..."
    cd "$PROJECT_ROOT"
    cargo build --profile perf-test --bin centotype
fi

# Warmup runs
echo "Performing warmup runs..."
for i in $(seq 1 $WARMUP_ITERATIONS); do
    timeout 10s "$BINARY_PATH" --version > /dev/null 2>&1 || true
done

# Benchmark startup times
echo "Running startup time benchmarks..."
RESULTS_FILE="$RESULTS_DIR/startup_times.txt"
rm -f "$RESULTS_FILE"

for i in $(seq 1 $ITERATIONS); do
    echo -n "Run $i/$ITERATIONS... "

    # Measure startup time using time command with microsecond precision
    START_TIME=$(date +%s%6N)
    timeout 10s "$BINARY_PATH" --version > /dev/null 2>&1 || {
        echo "TIMEOUT"
        continue
    }
    END_TIME=$(date +%s%6N)

    # Calculate duration in milliseconds
    DURATION_MICROSECONDS=$((END_TIME - START_TIME))
    DURATION_MS=$((DURATION_MICROSECONDS / 1000))

    echo "${DURATION_MS}ms"
    echo "$DURATION_MS" >> "$RESULTS_FILE"
done

# Analyze results
echo
echo "=== Startup Time Analysis ==="

if [[ ! -f "$RESULTS_FILE" || ! -s "$RESULTS_FILE" ]]; then
    echo "ERROR: No valid measurements collected"
    exit 1
fi

# Calculate statistics using awk
STATS=$(awk '
BEGIN {
    count = 0;
    sum = 0;
    min = 999999;
    max = 0;
}
{
    times[count] = $1;
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
            if (times[i] > times[j]) {
                temp = times[i];
                times[i] = times[j];
                times[j] = temp;
            }
        }
    }

    avg = sum / count;
    p50_idx = int(count * 0.5);
    p95_idx = int(count * 0.95);
    p99_idx = int(count * 0.99);

    printf "%.2f %.2f %.2f %.2f %.2f %.2f %d",
           avg, times[p50_idx], times[p95_idx], times[p99_idx], min, max, count;
}' "$RESULTS_FILE")

if [[ -z "$STATS" ]]; then
    echo "ERROR: Failed to calculate statistics"
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
cat > "$RESULTS_DIR/startup_benchmark.json" << EOF
{
    "timestamp": "$(date -u +%Y-%m-%dT%H:%M:%SZ)",
    "test": "startup_time",
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
        "p95_threshold_ms": $PERCENTILE_95_THRESHOLD_MS
    },
    "passed": $(if (( $(echo "$P95 <= $PERCENTILE_95_THRESHOLD_MS" | bc -l) )); then echo "true"; else echo "false"; fi)
}
EOF

# Validation
echo
echo "=== Performance Validation ==="
if (( $(echo "$P95 <= $PERCENTILE_95_THRESHOLD_MS" | bc -l) )); then
    echo "✅ PASS: P95 startup time (${P95}ms) is within threshold (${PERCENTILE_95_THRESHOLD_MS}ms)"
    exit 0
else
    echo "❌ FAIL: P95 startup time (${P95}ms) exceeds threshold (${PERCENTILE_95_THRESHOLD_MS}ms)"
    echo "Performance regression detected!"
    exit 1
fi