#!/bin/bash
# Comprehensive security scanning script for Centotype
# Scans for embedded secrets, credentials, and security vulnerabilities

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
CYAN='\033[0;36m'
NC='\033[0m' # No Color

# Configuration
SCAN_DEPTH=3
MAX_FILE_SIZE=10485760  # 10MB
TEMP_DIR="/tmp/centotype_security_scan"
OUTPUT_DIR="$PROJECT_ROOT/security_reports"

# Security patterns to detect
declare -A SECURITY_PATTERNS=(
    ["password"]="password|passwd|pwd"
    ["api_key"]="api[_-]?key|apikey"
    ["secret"]="secret|private[_-]?key|secret[_-]?key"
    ["token"]="token|access[_-]?token|auth[_-]?token"
    ["credential"]="credential|creds"
    ["connection_string"]="connection[_-]?string|conn[_-]?str"
    ["database_url"]="database[_-]?url|db[_-]?url"
    ["private_key"]="-----BEGIN (RSA )?PRIVATE KEY-----"
    ["certificate"]="-----BEGIN CERTIFICATE-----"
    ["aws_access"]="AKIA[0-9A-Z]{16}"
    ["github_token"]="ghp_[0-9a-zA-Z]{36}"
    ["slack_token"]="xox[baprs]-[0-9a-zA-Z-]+"
    ["jwt_token"]="eyJ[A-Za-z0-9-_=]+\\.[A-Za-z0-9-_=]+\\.[A-Za-z0-9-_.+/=]*"
    ["hardcoded_ip"]="[0-9]{1,3}\\.[0-9]{1,3}\\.[0-9]{1,3}\\.[0-9]{1,3}"
    ["hardcoded_url"]="https?://[a-zA-Z0-9.-]+\\.[a-zA-Z]{2,}"
)

# Dangerous file patterns
declare -A DANGEROUS_PATTERNS=(
    ["system_paths"]="/home/|/Users/|C:\\\\|/etc/|/var/|/root/"
    ["shell_commands"]="\\$\\(|`|\\&\\&|\\|\\||;|>|<"
    ["unsafe_rust"]="unsafe|transmute|from_raw"
    ["sql_injection"]="SELECT|INSERT|UPDATE|DELETE|DROP|UNION"
    ["xss_patterns"]="<script|javascript:|onload=|onerror="
    ["command_injection"]="system\\(|exec\\(|shell_exec\\(|eval\\("
)

echo -e "${BLUE}=== Centotype Security Scanner ===${NC}"
echo "Project root: $PROJECT_ROOT"
echo "Scan depth: $SCAN_DEPTH levels"
echo "Max file size: $((MAX_FILE_SIZE / 1024 / 1024))MB"
echo

# Create output directory
mkdir -p "$OUTPUT_DIR"
mkdir -p "$TEMP_DIR"

# Initialize counters
TOTAL_FILES=0
SCANNED_FILES=0
SECURITY_ISSUES=0
HIGH_RISK_ISSUES=0
MEDIUM_RISK_ISSUES=0
LOW_RISK_ISSUES=0

# Report files
SECRETS_REPORT="$OUTPUT_DIR/secrets_scan_$(date +%Y%m%d_%H%M%S).txt"
VULNERABILITIES_REPORT="$OUTPUT_DIR/vulnerabilities_scan_$(date +%Y%m%d_%H%M%S).txt"
SUMMARY_REPORT="$OUTPUT_DIR/security_summary_$(date +%Y%m%d_%H%M%S).txt"

# Initialize reports
echo "Centotype Security Scan Report - $(date)" > "$SECRETS_REPORT"
echo "=========================================" >> "$SECRETS_REPORT"
echo "" >> "$SECRETS_REPORT"

echo "Centotype Vulnerability Scan Report - $(date)" > "$VULNERABILITIES_REPORT"
echo "===============================================" >> "$VULNERABILITIES_REPORT"
echo "" >> "$VULNERABILITIES_REPORT"

# Function to log findings
log_finding() {
    local severity="$1"
    local category="$2"
    local file="$3"
    local line="$4"
    local pattern="$5"
    local context="$6"

    echo "[$(date '+%H:%M:%S')] [$severity] $category in $file:$line" | tee -a "$SECRETS_REPORT"
    echo "  Pattern: $pattern" | tee -a "$SECRETS_REPORT"
    echo "  Context: $context" | tee -a "$SECRETS_REPORT"
    echo "" | tee -a "$SECRETS_REPORT"

    case "$severity" in
        "HIGH")    ((HIGH_RISK_ISSUES++)) ;;
        "MEDIUM")  ((MEDIUM_RISK_ISSUES++)) ;;
        "LOW")     ((LOW_RISK_ISSUES++)) ;;
    esac
    ((SECURITY_ISSUES++))
}

# Function to scan file for secrets
scan_file_for_secrets() {
    local file="$1"
    local basename_file
    basename_file=$(basename "$file")

    # Skip binary files and large files
    if ! file "$file" | grep -q "text" || [[ $(stat -f%z "$file" 2>/dev/null || stat -c%s "$file" 2>/dev/null || echo 0) -gt $MAX_FILE_SIZE ]]; then
        return 0
    fi

    ((SCANNED_FILES++))

    # Scan for each security pattern
    for pattern_name in "${!SECURITY_PATTERNS[@]}"; do
        local pattern="${SECURITY_PATTERNS[$pattern_name]}"

        # Use ripgrep for better performance if available, otherwise use grep
        if command -v rg &> /dev/null; then
            local matches
            matches=$(rg -n -i "$pattern" "$file" 2>/dev/null || true)
        else
            local matches
            matches=$(grep -n -i -E "$pattern" "$file" 2>/dev/null || true)
        fi

        if [[ -n "$matches" ]]; then
            while IFS= read -r match; do
                local line_num
                line_num=$(echo "$match" | cut -d: -f1)
                local content
                content=$(echo "$match" | cut -d: -f2-)

                # Determine severity based on pattern type
                local severity="MEDIUM"
                case "$pattern_name" in
                    "private_key"|"certificate"|"aws_access"|"github_token"|"slack_token")
                        severity="HIGH"
                        ;;
                    "password"|"secret"|"token"|"credential")
                        severity="MEDIUM"
                        ;;
                    *)
                        severity="LOW"
                        ;;
                esac

                log_finding "$severity" "Secret/Credential" "$file" "$line_num" "$pattern_name" "$content"
            done <<< "$matches"
        fi
    done

    # Scan for dangerous patterns
    for pattern_name in "${!DANGEROUS_PATTERNS[@]}"; do
        local pattern="${DANGEROUS_PATTERNS[$pattern_name]}"

        if command -v rg &> /dev/null; then
            local matches
            matches=$(rg -n -E "$pattern" "$file" 2>/dev/null || true)
        else
            local matches
            matches=$(grep -n -E "$pattern" "$file" 2>/dev/null || true)
        fi

        if [[ -n "$matches" ]]; then
            while IFS= read -r match; do
                local line_num
                line_num=$(echo "$match" | cut -d: -f1)
                local content
                content=$(echo "$match" | cut -d: -f2-)

                # All dangerous patterns are medium risk by default
                log_finding "MEDIUM" "Dangerous Pattern" "$file" "$line_num" "$pattern_name" "$content"
            done <<< "$matches"
        fi
    done
}

# Function to check dependencies for vulnerabilities
check_dependency_vulnerabilities() {
    echo -e "${CYAN}Checking dependency vulnerabilities...${NC}"

    cd "$PROJECT_ROOT"

    # Install cargo-audit if not present
    if ! command -v cargo-audit &> /dev/null; then
        echo "Installing cargo-audit..."
        cargo install cargo-audit
    fi

    # Run cargo audit
    echo "Running cargo audit..." | tee -a "$VULNERABILITIES_REPORT"
    if cargo audit --format json > "$TEMP_DIR/audit_results.json" 2>&1; then
        echo "✅ No vulnerabilities found in dependencies" | tee -a "$VULNERABILITIES_REPORT"
    else
        echo "⚠️ Vulnerabilities found in dependencies:" | tee -a "$VULNERABILITIES_REPORT"
        cat "$TEMP_DIR/audit_results.json" | tee -a "$VULNERABILITIES_REPORT"
        ((MEDIUM_RISK_ISSUES++))
        ((SECURITY_ISSUES++))
    fi

    # Install and run cargo-deny if available
    if command -v cargo-deny &> /dev/null || cargo install cargo-deny; then
        echo "Running cargo deny..." | tee -a "$VULNERABILITIES_REPORT"
        if cargo deny check 2>&1 | tee -a "$VULNERABILITIES_REPORT"; then
            echo "✅ Dependency policy check passed" | tee -a "$VULNERABILITIES_REPORT"
        else
            echo "⚠️ Dependency policy violations found" | tee -a "$VULNERABILITIES_REPORT"
            ((LOW_RISK_ISSUES++))
            ((SECURITY_ISSUES++))
        fi
    fi
}

# Function to check file permissions
check_file_permissions() {
    echo -e "${CYAN}Checking file permissions...${NC}"

    # Check for overly permissive files
    echo "Checking file permissions..." | tee -a "$VULNERABILITIES_REPORT"

    if command -v find &> /dev/null; then
        # Find world-writable files
        local world_writable
        world_writable=$(find "$PROJECT_ROOT" -type f -perm -002 2>/dev/null || true)
        if [[ -n "$world_writable" ]]; then
            echo "⚠️ World-writable files found:" | tee -a "$VULNERABILITIES_REPORT"
            echo "$world_writable" | tee -a "$VULNERABILITIES_REPORT"
            log_finding "HIGH" "File Permissions" "Multiple files" "N/A" "world_writable" "$world_writable"
        fi

        # Find files with excessive permissions
        local excessive_perms
        excessive_perms=$(find "$PROJECT_ROOT" -type f -perm -777 2>/dev/null || true)
        if [[ -n "$excessive_perms" ]]; then
            echo "⚠️ Files with excessive permissions found:" | tee -a "$VULNERABILITIES_REPORT"
            echo "$excessive_perms" | tee -a "$VULNERABILITIES_REPORT"
            log_finding "MEDIUM" "File Permissions" "Multiple files" "N/A" "excessive_permissions" "$excessive_perms"
        fi
    fi
}

# Function to scan code for security issues
scan_code_security() {
    echo -e "${CYAN}Scanning source code for security issues...${NC}"

    # Find all source files to scan
    local file_patterns=("*.rs" "*.toml" "*.yaml" "*.yml" "*.json" "*.sh" "*.py" "*.js" "*.ts")
    local files_to_scan=()

    for pattern in "${file_patterns[@]}"; do
        while IFS= read -r -d '' file; do
            files_to_scan+=("$file")
        done < <(find "$PROJECT_ROOT" -name "$pattern" -type f -print0 2>/dev/null || true)
    done

    TOTAL_FILES=${#files_to_scan[@]}
    echo "Found $TOTAL_FILES files to scan"

    # Scan each file
    local progress=0
    for file in "${files_to_scan[@]}"; do
        # Skip target directory and hidden files
        if [[ "$file" == *"/target/"* ]] || [[ "$file" == *"/.git/"* ]] || [[ "$(basename "$file")" == .* ]]; then
            continue
        fi

        scan_file_for_secrets "$file"

        # Show progress
        ((progress++))
        if ((progress % 10 == 0)); then
            local percent=$((progress * 100 / TOTAL_FILES))
            echo "Progress: $progress/$TOTAL_FILES files ($percent%)"
        fi
    done
}

# Function to check for hardcoded test data
check_test_data() {
    echo -e "${CYAN}Checking for hardcoded test data...${NC}"

    # Look for potential test credentials in test files
    local test_files
    test_files=$(find "$PROJECT_ROOT" -path "*/tests/*" -name "*.rs" -o -path "*/test/*" -name "*.rs" 2>/dev/null || true)

    if [[ -n "$test_files" ]]; then
        echo "Scanning test files for hardcoded data..." | tee -a "$SECRETS_REPORT"

        while IFS= read -r file; do
            if [[ -f "$file" ]]; then
                # Look for hardcoded credentials in tests
                local test_patterns=("password.*=.*\".*\"" "token.*=.*\".*\"" "key.*=.*\".*\"" "secret.*=.*\".*\"")

                for pattern in "${test_patterns[@]}"; do
                    if command -v rg &> /dev/null; then
                        local matches
                        matches=$(rg -n -i "$pattern" "$file" 2>/dev/null || true)
                    else
                        local matches
                        matches=$(grep -n -i -E "$pattern" "$file" 2>/dev/null || true)
                    fi

                    if [[ -n "$matches" ]]; then
                        while IFS= read -r match; do
                            local line_num
                            line_num=$(echo "$match" | cut -d: -f1)
                            local content
                            content=$(echo "$match" | cut -d: -f2-)
                            log_finding "LOW" "Test Data" "$file" "$line_num" "hardcoded_test_credential" "$content"
                        done <<< "$matches"
                    fi
                done
            fi
        done <<< "$test_files"
    fi
}

# Function to generate summary report
generate_summary() {
    echo -e "${CYAN}Generating security summary report...${NC}"

    cat > "$SUMMARY_REPORT" << EOF
Centotype Security Scan Summary
===============================
Scan Date: $(date)
Project: $PROJECT_ROOT

SCAN STATISTICS:
- Total files found: $TOTAL_FILES
- Files scanned: $SCANNED_FILES
- Security issues found: $SECURITY_ISSUES

RISK BREAKDOWN:
- High Risk Issues: $HIGH_RISK_ISSUES
- Medium Risk Issues: $MEDIUM_RISK_ISSUES
- Low Risk Issues: $LOW_RISK_ISSUES

SECURITY ASSESSMENT:
EOF

    # Determine overall security grade
    local grade="A"
    local recommendation="Excellent security posture"

    if ((HIGH_RISK_ISSUES > 0)); then
        grade="F"
        recommendation="CRITICAL: High-risk security issues require immediate attention"
    elif ((MEDIUM_RISK_ISSUES > 10)); then
        grade="D"
        recommendation="Poor security posture - multiple medium-risk issues need resolution"
    elif ((MEDIUM_RISK_ISSUES > 5)); then
        grade="C"
        recommendation="Moderate security concerns - address medium-risk issues"
    elif ((MEDIUM_RISK_ISSUES > 0)); then
        grade="B"
        recommendation="Good security posture with minor issues to address"
    elif ((LOW_RISK_ISSUES > 10)); then
        grade="B"
        recommendation="Good security posture but review low-risk issues"
    fi

    cat >> "$SUMMARY_REPORT" << EOF
- Overall Grade: $grade
- Recommendation: $recommendation

REPORT FILES:
- Secrets Report: $SECRETS_REPORT
- Vulnerabilities Report: $VULNERABILITIES_REPORT
- Summary Report: $SUMMARY_REPORT

NEXT STEPS:
1. Review all HIGH risk issues immediately
2. Plan remediation for MEDIUM risk issues
3. Consider addressing LOW risk issues in next iteration
4. Implement continuous security scanning in CI/CD pipeline

EOF

    echo "Summary report generated: $SUMMARY_REPORT"
}

# Main execution
main() {
    echo -e "${BLUE}Starting comprehensive security scan...${NC}"
    echo

    # 1. Scan source code for secrets and dangerous patterns
    scan_code_security

    # 2. Check dependency vulnerabilities
    check_dependency_vulnerabilities

    # 3. Check file permissions
    check_file_permissions

    # 4. Check test data
    check_test_data

    # 5. Generate summary report
    generate_summary

    # Display results
    echo
    echo -e "${BLUE}=== Security Scan Results ===${NC}"
    echo -e "Total files scanned: ${CYAN}$SCANNED_FILES${NC}"
    echo -e "Security issues found: ${CYAN}$SECURITY_ISSUES${NC}"
    echo -e "  - High Risk: ${RED}$HIGH_RISK_ISSUES${NC}"
    echo -e "  - Medium Risk: ${YELLOW}$MEDIUM_RISK_ISSUES${NC}"
    echo -e "  - Low Risk: ${GREEN}$LOW_RISK_ISSUES${NC}"
    echo

    if ((HIGH_RISK_ISSUES > 0)); then
        echo -e "${RED}🚨 CRITICAL: High-risk security issues found!${NC}"
        echo -e "${RED}Please review and fix immediately before proceeding.${NC}"
        exit 1
    elif ((MEDIUM_RISK_ISSUES > 10)); then
        echo -e "${YELLOW}⚠️ WARNING: Multiple medium-risk security issues found.${NC}"
        echo -e "${YELLOW}Recommend addressing before production deployment.${NC}"
        exit 1
    elif ((SECURITY_ISSUES > 0)); then
        echo -e "${YELLOW}ℹ️ INFO: Some security issues found for review.${NC}"
        echo -e "${YELLOW}See detailed reports for more information.${NC}"
    else
        echo -e "${GREEN}✅ No significant security issues found!${NC}"
        echo -e "${GREEN}Security posture looks good.${NC}"
    fi

    echo
    echo "Detailed reports available in: $OUTPUT_DIR"

    # Cleanup
    rm -rf "$TEMP_DIR"
}

# Run main function
main "$@"