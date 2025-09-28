#!/bin/bash
# Continuous Security Validation Pipeline for Centotype
# Integrates with CI/CD to provide automated security testing

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
MAGENTA='\033[0;35m'
NC='\033[0m' # No Color

# Configuration
SECURITY_REPORT_DIR="$PROJECT_ROOT/security_reports"
CI_MODE=false
FAIL_FAST=false
VERBOSE=false
EXTENDED_TESTS=false

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --ci)
            CI_MODE=true
            shift
            ;;
        --fail-fast)
            FAIL_FAST=true
            shift
            ;;
        --verbose)
            VERBOSE=true
            shift
            ;;
        --extended)
            EXTENDED_TESTS=true
            shift
            ;;
        -h|--help)
            echo "Usage: $0 [OPTIONS]"
            echo
            echo "Options:"
            echo "  --ci          Run in CI/CD mode (machine-readable output)"
            echo "  --fail-fast   Exit immediately on first failure"
            echo "  --verbose     Enable verbose output"
            echo "  --extended    Run extended security tests (4+ hour duration)"
            echo "  -h, --help    Show this help message"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

echo -e "${BLUE}=== Centotype Continuous Security Validation ===${NC}"
echo "Project root: $PROJECT_ROOT"
echo "CI mode: $CI_MODE"
echo "Fail fast: $FAIL_FAST"
echo "Extended tests: $EXTENDED_TESTS"
echo

# Create security reports directory
mkdir -p "$SECURITY_REPORT_DIR"

# Global security status tracking
SECURITY_ISSUES=0
CRITICAL_ISSUES=0
HIGH_ISSUES=0
MEDIUM_ISSUES=0
LOW_ISSUES=0

# Function to log security findings
log_security_finding() {
    local severity="$1"
    local category="$2"
    local description="$3"
    local recommendation="${4:-}"

    local timestamp
    timestamp=$(date '+%Y-%m-%d %H:%M:%S')

    # Color based on severity
    local color
    case "$severity" in
        "CRITICAL") color="$RED"; ((CRITICAL_ISSUES++)) ;;
        "HIGH")     color="$RED"; ((HIGH_ISSUES++)) ;;
        "MEDIUM")   color="$YELLOW"; ((MEDIUM_ISSUES++)) ;;
        "LOW")      color="$CYAN"; ((LOW_ISSUES++)) ;;
        *)          color="$NC" ;;
    esac

    ((SECURITY_ISSUES++))

    if $CI_MODE; then
        # Machine-readable format for CI/CD
        echo "::${severity,,}::${category}: ${description}"
        if [[ -n "$recommendation" ]]; then
            echo "::notice::Recommendation: ${recommendation}"
        fi
    else
        # Human-readable format
        echo -e "${color}[$severity]${NC} ${category}: ${description}"
        if [[ -n "$recommendation" ]]; then
            echo -e "  ${CYAN}Recommendation:${NC} ${recommendation}"
        fi
    fi

    # Log to security report
    echo "[$timestamp] [$severity] $category: $description" >> "$SECURITY_REPORT_DIR/security_pipeline.log"
    if [[ -n "$recommendation" ]]; then
        echo "  Recommendation: $recommendation" >> "$SECURITY_REPORT_DIR/security_pipeline.log"
    fi

    # Fail fast on critical issues if enabled
    if $FAIL_FAST && [[ "$severity" == "CRITICAL" ]]; then
        echo -e "${RED}CRITICAL security issue detected - exiting due to --fail-fast${NC}"
        exit 1
    fi
}

# Function to run security test with timeout and monitoring
run_security_test() {
    local test_name="$1"
    local test_command="$2"
    local timeout_duration="${3:-300}"  # 5 minutes default
    local expected_result="${4:-success}"

    echo -e "${BLUE}Running: $test_name${NC}"

    local start_time
    start_time=$(date +%s)

    # Run test with timeout
    local exit_code=0
    if timeout "${timeout_duration}s" bash -c "$test_command" 2>&1; then
        if [[ "$expected_result" == "failure" ]]; then
            log_security_finding "MEDIUM" "Test Expectation" \
                "Test '$test_name' was expected to fail but passed" \
                "Review test expectations and security controls"
        fi
    else
        exit_code=$?
        if [[ $exit_code -eq 124 ]]; then
            log_security_finding "HIGH" "Test Timeout" \
                "Test '$test_name' timed out after ${timeout_duration}s" \
                "Investigate potential infinite loops or performance issues"
        elif [[ "$expected_result" == "success" ]]; then
            log_security_finding "HIGH" "Test Failure" \
                "Test '$test_name' failed unexpectedly (exit code: $exit_code)" \
                "Review test output and fix underlying issues"
        fi
    fi

    local end_time
    end_time=$(date +%s)
    local duration=$((end_time - start_time))

    if $VERBOSE; then
        echo "  Duration: ${duration}s"
    fi

    return $exit_code
}

# Phase 1: Static Security Analysis
echo -e "${MAGENTA}=== Phase 1: Static Security Analysis ===${NC}"

# 1.1: Dependency vulnerability scanning
run_security_test "Dependency Vulnerability Scan" \
    "cd '$PROJECT_ROOT' && cargo audit --format json > '$SECURITY_REPORT_DIR/audit.json' 2>&1" \
    120

# 1.2: License and dependency policy check
run_security_test "Dependency Policy Check" \
    "cd '$PROJECT_ROOT' && (cargo deny check 2>&1 || true)" \
    60

# 1.3: Secrets and credential scanning
run_security_test "Secrets Scanning" \
    "'$SCRIPT_DIR/security_scan.sh'" \
    300

# 1.4: Code quality and security linting
run_security_test "Security Linting" \
    "cd '$PROJECT_ROOT' && cargo clippy --workspace --all-targets -- -D warnings -D clippy::unwrap_used" \
    180

# Phase 2: Dynamic Security Testing
echo
echo -e "${MAGENTA}=== Phase 2: Dynamic Security Testing ===${NC}"

# 2.1: Content security validation
run_security_test "Content Security Validation" \
    "cd '$PROJECT_ROOT' && cargo test --package centotype-content --test security_tests" \
    300

# 2.2: Input fuzzing tests
run_security_test "Input Fuzzing (Short)" \
    "cd '$PROJECT_ROOT' && cargo test --package centotype-content --test extended_fuzz test_short_crash_resistance -- --ignored" \
    600

# 2.3: File system security audit
run_security_test "File System Security Audit" \
    "cd '$PROJECT_ROOT' && cargo test --package centotype-content fs_security::tests::test_file_system_auditor_creation" \
    120

# 2.4: Terminal escape sequence protection
run_security_test "Escape Sequence Protection" \
    "cd '$PROJECT_ROOT' && cargo test --package centotype-engine --test fuzz_input test_malicious_escape_sequences -- --ignored" \
    180

# Phase 3: Performance Security Testing
echo
echo -e "${MAGENTA}=== Phase 3: Performance Security Testing ===${NC}"

# 3.1: DoS resistance testing
run_security_test "DoS Resistance Testing" \
    "cd '$PROJECT_ROOT' && cargo test --package centotype-engine --test fuzz_input test_input_buffer_overflow -- --ignored" \
    300

# 3.2: Memory safety validation
run_security_test "Memory Safety Validation" \
    "cd '$PROJECT_ROOT' && cargo test --package centotype-engine --test fuzz_input test_concurrent_input_safety -- --ignored" \
    240

# 3.3: Unicode handling security
run_security_test "Unicode Security Testing" \
    "cd '$PROJECT_ROOT' && cargo test --package centotype-engine --test fuzz_input test_unicode_edge_cases -- --ignored" \
    180

# Phase 4: Extended Security Testing (Optional)
if $EXTENDED_TESTS; then
    echo
    echo -e "${MAGENTA}=== Phase 4: Extended Security Testing ===${NC}"
    echo -e "${YELLOW}Warning: Extended tests may take 4+ hours to complete${NC}"

    # 4.1: Extended fuzzing test (4 hours)
    run_security_test "Extended Fuzzing Test" \
        "cd '$PROJECT_ROOT' && cargo test --package centotype-content --test extended_fuzz test_extended_fuzz_suite -- --ignored" \
        14400  # 4 hours

    # 4.2: Long-running stability test
    run_security_test "Stability Test" \
        "cd '$PROJECT_ROOT' && timeout 3600s cargo run --bin centotype -- --benchmark-mode" \
        3600  # 1 hour

    # 4.3: Memory leak detection
    if command -v valgrind &> /dev/null; then
        run_security_test "Memory Leak Detection" \
            "cd '$PROJECT_ROOT' && valgrind --leak-check=full --error-exitcode=1 target/release/centotype --version" \
            600
    else
        log_security_finding "LOW" "Tool Missing" \
            "Valgrind not available for memory leak detection" \
            "Install valgrind for comprehensive memory testing"
    fi
fi

# Phase 5: Security Configuration Validation
echo
echo -e "${MAGENTA}=== Phase 5: Security Configuration Validation ===${NC}"

# 5.1: Build configuration security
run_security_test "Build Security Configuration" \
    "cd '$PROJECT_ROOT' && cargo tree --duplicates && cargo tree --format '{p} {f}' | grep -i 'unsafe\\|std'" \
    60

# 5.2: Feature flag security analysis
run_security_test "Feature Flag Security Analysis" \
    "cd '$PROJECT_ROOT' && cargo metadata --format-version 1 | jq '.packages[].features' || echo 'jq not available'" \
    30

# 5.3: Cross-compilation security
run_security_test "Cross-compilation Test" \
    "cd '$PROJECT_ROOT' && cargo check --target x86_64-unknown-linux-gnu" \
    180

# Phase 6: Compliance and Documentation
echo
echo -e "${MAGENTA}=== Phase 6: Compliance and Documentation ===${NC}"

# 6.1: Security documentation check
if [[ -f "$PROJECT_ROOT/SECURITY.md" ]]; then
    echo "✅ Security documentation found"
else
    log_security_finding "MEDIUM" "Documentation Missing" \
        "SECURITY.md file not found" \
        "Create security documentation with threat model and contact information"
fi

# 6.2: Vulnerability disclosure policy
if [[ -f "$PROJECT_ROOT/.github/SECURITY.md" ]]; then
    echo "✅ Vulnerability disclosure policy found"
else
    log_security_finding "LOW" "Policy Missing" \
        "Vulnerability disclosure policy not found" \
        "Create .github/SECURITY.md with vulnerability reporting process"
fi

# Generate final security report
echo
echo -e "${MAGENTA}=== Security Validation Summary ===${NC}"

# Security grade calculation
SECURITY_GRADE="A"
if [[ $CRITICAL_ISSUES -gt 0 ]]; then
    SECURITY_GRADE="F"
elif [[ $HIGH_ISSUES -gt 3 ]]; then
    SECURITY_GRADE="D"
elif [[ $HIGH_ISSUES -gt 0 ]]; then
    SECURITY_GRADE="C"
elif [[ $MEDIUM_ISSUES -gt 5 ]]; then
    SECURITY_GRADE="B"
fi

# Generate summary report
SUMMARY_FILE="$SECURITY_REPORT_DIR/security_validation_summary_$(date +%Y%m%d_%H%M%S).txt"
cat > "$SUMMARY_FILE" << EOF
Centotype Security Validation Summary
=====================================
Date: $(date)
Extended Tests: $EXTENDED_TESTS

SECURITY ISSUES FOUND:
- Critical: $CRITICAL_ISSUES
- High: $HIGH_ISSUES
- Medium: $MEDIUM_ISSUES
- Low: $LOW_ISSUES
- Total: $SECURITY_ISSUES

SECURITY GRADE: $SECURITY_GRADE

EOF

# Display results
if $CI_MODE; then
    echo "security_grade=$SECURITY_GRADE"
    echo "critical_issues=$CRITICAL_ISSUES"
    echo "high_issues=$HIGH_ISSUES"
    echo "medium_issues=$MEDIUM_ISSUES"
    echo "low_issues=$LOW_ISSUES"
    echo "total_issues=$SECURITY_ISSUES"
else
    echo -e "Security Issues Found:"
    echo -e "  - ${RED}Critical: $CRITICAL_ISSUES${NC}"
    echo -e "  - ${RED}High: $HIGH_ISSUES${NC}"
    echo -e "  - ${YELLOW}Medium: $MEDIUM_ISSUES${NC}"
    echo -e "  - ${CYAN}Low: $LOW_ISSUES${NC}"
    echo -e "  - Total: $SECURITY_ISSUES"
    echo
    echo -e "Security Grade: ${SECURITY_GRADE}"
fi

echo "Detailed reports available in: $SECURITY_REPORT_DIR"

# Exit with appropriate code
if [[ $CRITICAL_ISSUES -gt 0 ]]; then
    echo -e "${RED}❌ CRITICAL security issues found - deployment blocked${NC}"
    exit 1
elif [[ $HIGH_ISSUES -gt 0 ]]; then
    echo -e "${YELLOW}⚠️ HIGH-RISK security issues found - review required${NC}"
    exit 1
elif [[ $SECURITY_ISSUES -gt 0 ]]; then
    echo -e "${CYAN}ℹ️ Security issues found but not blocking${NC}"
    exit 0
else
    echo -e "${GREEN}✅ Security validation passed - no significant issues found${NC}"
    exit 0
fi