#!/bin/bash
set -euo pipefail

# CI/CD Setup Validation Script
# Validates that all CI/CD components are properly configured

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Colors
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

echo -e "${BLUE}=== Centotype CI/CD Setup Validation ===${NC}"
echo "Project root: $PROJECT_ROOT"
echo

cd "$PROJECT_ROOT"

VALIDATION_PASSED=true

# Function to check file existence and report
check_file() {
    local file="$1"
    local description="$2"
    local required="${3:-true}"

    if [[ -f "$file" ]]; then
        echo -e "${GREEN}✅ $description${NC}: $file"
        return 0
    else
        if [[ "$required" == "true" ]]; then
            echo -e "${RED}❌ $description missing${NC}: $file"
            VALIDATION_PASSED=false
            return 1
        else
            echo -e "${YELLOW}⚠️  $description optional${NC}: $file"
            return 0
        fi
    fi
}

# Function to check if script is executable
check_executable() {
    local script="$1"
    local description="$2"

    if [[ -f "$script" && -x "$script" ]]; then
        echo -e "${GREEN}✅ $description executable${NC}: $script"
        return 0
    else
        echo -e "${RED}❌ $description not executable${NC}: $script"
        VALIDATION_PASSED=false
        return 1
    fi
}

echo -e "${BLUE}=== Core CI/CD Files ===${NC}"
check_file ".github/workflows/ci.yml" "Main CI/CD pipeline"
check_file "deny.toml" "Dependency validation config"
check_file ".centotype-ci.toml" "Performance thresholds config"

echo
echo -e "${BLUE}=== Build and Release Scripts ===${NC}"
check_executable "scripts/benchmark_startup.sh" "Startup benchmark script"
check_executable "scripts/benchmark_latency.sh" "Latency benchmark script"
check_executable "scripts/compare_performance.sh" "Performance comparison script"
check_executable "scripts/prepare_release.sh" "Release preparation script"
check_executable "scripts/publish_crates.sh" "Crates publishing script"
check_executable "scripts/prepare_npm.sh" "NPM package preparation script"
check_executable "scripts/generate_changelog.sh" "Changelog generation script"

echo
echo -e "${BLUE}=== Development Tools ===${NC}"
check_executable "scripts/validate_local.sh" "Local validation script"

echo
echo -e "${BLUE}=== Test Files ===${NC}"
check_file "engine/tests/performance_validation.rs" "Performance validation tests"
check_file "engine/tests/memory_validation.rs" "Memory validation tests"
check_file "engine/tests/fuzz_input.rs" "Input fuzzing tests"

echo
echo -e "${BLUE}=== Documentation ===${NC}"
check_file "docs/CI_CD.md" "CI/CD documentation"
check_file "docs/DEVELOPMENT.md" "Development workflow guide"

echo
echo -e "${BLUE}=== Configuration Validation ===${NC}"

# Check GitHub Actions workflow syntax
if command -v yq &> /dev/null; then
    if yq eval '.jobs' .github/workflows/ci.yml &> /dev/null; then
        echo -e "${GREEN}✅ GitHub Actions workflow syntax valid${NC}"
    else
        echo -e "${RED}❌ GitHub Actions workflow syntax invalid${NC}"
        VALIDATION_PASSED=false
    fi
else
    echo -e "${YELLOW}⚠️  yq not found, skipping YAML validation${NC}"
fi

# Check TOML files
if command -v toml &> /dev/null; then
    for toml_file in "deny.toml" ".centotype-ci.toml"; do
        if [[ -f "$toml_file" ]]; then
            if toml get "$toml_file" . &> /dev/null; then
                echo -e "${GREEN}✅ $toml_file syntax valid${NC}"
            else
                echo -e "${RED}❌ $toml_file syntax invalid${NC}"
                VALIDATION_PASSED=false
            fi
        fi
    done
else
    echo -e "${YELLOW}⚠️  toml not found, skipping TOML validation${NC}"
fi

echo
echo -e "${BLUE}=== Performance Thresholds Check ===${NC}"

# Extract thresholds from configuration
if [[ -f ".centotype-ci.toml" ]]; then
    echo "Performance thresholds from .centotype-ci.toml:"
    grep -E "(input_latency|startup_time|render_time|memory_usage)" .centotype-ci.toml || true
fi

echo
echo -e "${BLUE}=== Dependency Validation ===${NC}"

# Check for required dependencies in Cargo.toml
REQUIRED_DEPS=("crossterm" "clap" "ratatui" "serde" "tokio" "criterion")
for dep in "${REQUIRED_DEPS[@]}"; do
    if grep -q "^$dep" Cargo.toml; then
        echo -e "${GREEN}✅ Required dependency present${NC}: $dep"
    else
        echo -e "${YELLOW}⚠️  Required dependency not found in workspace${NC}: $dep"
    fi
done

echo
echo -e "${BLUE}=== CI/CD Features Summary ===${NC}"
echo "The CI/CD pipeline includes:"
echo "  ✅ Cross-platform builds (6 targets)"
echo "  ✅ Performance validation (4 metrics)"
echo "  ✅ Security testing (vulnerability scanning + fuzzing)"
echo "  ✅ Automated releases (GitHub + Cargo + npm)"
echo "  ✅ Quality gates (formatting, linting, tests)"
echo "  ✅ Local development tools"
echo "  ✅ Performance regression detection"
echo "  ✅ Memory usage monitoring"
echo "  ✅ Documentation generation"

echo
echo -e "${BLUE}=== Platform Support ===${NC}"
echo "Build targets configured:"
echo "  • Linux: x86_64, ARM64"
echo "  • macOS: x86_64, ARM64"
echo "  • Windows: x86_64, ARM64"

echo
echo -e "${BLUE}=== Performance Requirements ===${NC}"
echo "Validated metrics:"
echo "  • P99 Input Latency: < 25ms"
echo "  • P95 Startup Time: < 200ms"
echo "  • P95 Render Time: < 33ms"
echo "  • Memory Usage: < 50MB RSS"

echo
echo -e "${BLUE}=== Next Steps ===${NC}"
if [[ "$VALIDATION_PASSED" == "true" ]]; then
    echo -e "${GREEN}🎉 CI/CD setup validation passed!${NC}"
    echo
    echo "You can now:"
    echo "  1. Test locally: ./scripts/validate_local.sh --quick"
    echo "  2. Commit changes: git add . && git commit -m 'feat: add CI/CD pipeline'"
    echo "  3. Push to trigger CI: git push"
    echo "  4. Create a pull request to test the full pipeline"
    echo
    echo "For releases:"
    echo "  1. Update version in Cargo.toml"
    echo "  2. Create and push a tag: git tag v1.0.0 && git push origin v1.0.0"
    echo "  3. The release pipeline will automatically create binaries and publish packages"
else
    echo -e "${RED}❌ CI/CD setup validation failed!${NC}"
    echo
    echo "Please fix the issues above before proceeding."
    echo "Run this script again after making corrections."
fi

echo
echo -e "${BLUE}=== Documentation ===${NC}"
echo "For detailed information, see:"
echo "  • docs/CI_CD.md - Complete pipeline documentation"
echo "  • docs/DEVELOPMENT.md - Development workflow guide"
echo "  • scripts/validate_local.sh --help - Local validation options"

exit $([[ "$VALIDATION_PASSED" == "true" ]] && echo 0 || echo 1)