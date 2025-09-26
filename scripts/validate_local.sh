#!/bin/bash
set -euo pipefail

# Local development validation script
# Runs the same checks as CI/CD pipeline locally

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
QUICK_MODE=false
PERFORMANCE_TESTS=true
SECURITY_TESTS=true

# Parse command line arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --quick)
            QUICK_MODE=true
            PERFORMANCE_TESTS=false
            SECURITY_TESTS=false
            shift
            ;;
        --no-perf)
            PERFORMANCE_TESTS=false
            shift
            ;;
        --no-security)
            SECURITY_TESTS=false
            shift
            ;;
        -h|--help)
            echo "Usage: $0 [OPTIONS]"
            echo
            echo "Options:"
            echo "  --quick       Quick validation (skip performance and security tests)"
            echo "  --no-perf     Skip performance tests"
            echo "  --no-security Skip security tests"
            echo "  -h, --help    Show this help message"
            exit 0
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

echo -e "${BLUE}=== Centotype Local Development Validation ===${NC}"
echo "Project root: $PROJECT_ROOT"
echo "Quick mode: $QUICK_MODE"
echo "Performance tests: $PERFORMANCE_TESTS"
echo "Security tests: $SECURITY_TESTS"
echo

cd "$PROJECT_ROOT"

# Function to run a test step
run_step() {
    local step_name="$1"
    local command="$2"
    local required="${3:-true}"

    echo -e "${BLUE}🔍 $step_name${NC}"

    if eval "$command"; then
        echo -e "${GREEN}✅ $step_name passed${NC}"
        return 0
    else
        if [[ "$required" == "true" ]]; then
            echo -e "${RED}❌ $step_name failed${NC}"
            return 1
        else
            echo -e "${YELLOW}⚠️  $step_name failed (non-critical)${NC}"
            return 0
        fi
    fi
}

# Track overall status
OVERALL_STATUS=0

# 1. Environment checks
echo -e "${BLUE}=== Environment Validation ===${NC}"

run_step "Rust toolchain" "cargo --version && rustc --version" || OVERALL_STATUS=1

if ! $QUICK_MODE; then
    run_step "Required tools" "command -v git && command -v bc" "false"
fi

# 2. Code quality checks
echo
echo -e "${BLUE}=== Code Quality Checks ===${NC}"

run_step "Code formatting" "cargo fmt --all -- --check" || OVERALL_STATUS=1
run_step "Linting (clippy)" "cargo clippy --workspace --all-targets --all-features -- -D warnings" || OVERALL_STATUS=1
run_step "Documentation" "cargo doc --workspace --no-deps --document-private-items" "false"

# 3. Build validation
echo
echo -e "${BLUE}=== Build Validation ===${NC}"

run_step "Debug build" "cargo build --workspace" || OVERALL_STATUS=1
run_step "Release build" "cargo build --release --workspace" || OVERALL_STATUS=1

if ! $QUICK_MODE; then
    run_step "Performance test build" "cargo build --profile perf-test --workspace" "false"
fi

# 4. Test execution
echo
echo -e "${BLUE}=== Test Execution ===${NC}"

run_step "Unit tests" "cargo test --workspace --lib" || OVERALL_STATUS=1
run_step "Integration tests" "cargo test --workspace --test '*'" || OVERALL_STATUS=1
run_step "Documentation tests" "cargo test --workspace --doc" "false"

# 5. Performance validation
if $PERFORMANCE_TESTS; then
    echo
    echo -e "${BLUE}=== Performance Validation ===${NC}"

    run_step "Benchmark compilation" "cargo bench --workspace --no-run" "false"

    if [[ -f "scripts/benchmark_startup.sh" ]]; then
        run_step "Startup time benchmark" "./scripts/benchmark_startup.sh" "false"
    fi

    if [[ -f "scripts/benchmark_latency.sh" ]]; then
        run_step "Input latency benchmark" "./scripts/benchmark_latency.sh" "false"
    fi

    run_step "Performance validation tests" "cargo test --package centotype-engine --test performance_validation -- --ignored" "false"
    run_step "Memory validation tests" "cargo test --package centotype-engine --test memory_validation -- --ignored" "false"
fi

# 6. Security validation
if $SECURITY_TESTS; then
    echo
    echo -e "${BLUE}=== Security Validation ===${NC}"

    # Install cargo-audit if not present
    if ! command -v cargo-audit &> /dev/null; then
        echo "Installing cargo-audit..."
        cargo install cargo-audit
    fi

    run_step "Security audit" "cargo audit" "false"

    # Install cargo-deny if not present
    if ! command -v cargo-deny &> /dev/null; then
        echo "Installing cargo-deny..."
        cargo install cargo-deny
    fi

    run_step "Dependency validation" "cargo deny check" "false"
    run_step "Input fuzzing tests" "cargo test --package centotype-engine --test fuzz_input -- --ignored" "false"
fi

# 7. Distribution validation
echo
echo -e "${BLUE}=== Distribution Validation ===${NC}"

run_step "Binary execution test" "target/release/centotype --version" || OVERALL_STATUS=1

if ! $QUICK_MODE; then
    run_step "Help output validation" "target/release/centotype --help | grep -q 'Centotype'" || OVERALL_STATUS=1
fi

# 8. Example and documentation validation
echo
echo -e "${BLUE}=== Documentation Validation ===${NC}"

if [[ -f "examples/demo.rs" ]]; then
    run_step "Example compilation" "cargo build --examples" "false"
fi

if [[ -f "README.md" ]]; then
    run_step "README validation" "test -s README.md" "false"
fi

# Final summary
echo
echo -e "${BLUE}=== Validation Summary ===${NC}"

if [[ $OVERALL_STATUS -eq 0 ]]; then
    echo -e "${GREEN}🎉 All critical validations passed!${NC}"
    echo "Your code is ready for:"
    echo "  - Pull request submission"
    echo "  - CI/CD pipeline"
    echo "  - Release preparation"
else
    echo -e "${RED}❌ Some critical validations failed${NC}"
    echo "Please fix the issues above before submitting."
fi

echo
echo "Next steps:"
echo "  - Run 'git add . && git commit' to commit changes"
echo "  - Run 'git push' to trigger CI/CD pipeline"
if $QUICK_MODE; then
    echo "  - Run '$0' (without --quick) for full validation"
fi

exit $OVERALL_STATUS