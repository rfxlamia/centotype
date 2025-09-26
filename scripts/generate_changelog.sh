#!/bin/bash
set -euo pipefail

# Changelog generation script
# Generates changelog from git commits and pull requests

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

# Get version from Cargo.toml
VERSION=$(grep '^version =' "$PROJECT_ROOT/Cargo.toml" | head -1 | sed 's/.*= "//' | sed 's/".*//')

# Get previous tag
PREVIOUS_TAG=$(git describe --tags --abbrev=0 HEAD^ 2>/dev/null || echo "")

echo "# Centotype v$VERSION"
echo
echo "**Release Date:** $(date +%Y-%m-%d)"
echo

if [[ -n "$PREVIOUS_TAG" ]]; then
    echo "**Changes since $PREVIOUS_TAG:**"
    echo

    # Get commits since last tag
    COMMITS=$(git log --pretty=format:"%h %s" "$PREVIOUS_TAG..HEAD" 2>/dev/null || git log --pretty=format:"%h %s" -10)

    # Categorize commits
    FEATURES=""
    FIXES=""
    PERFORMANCE=""
    DOCUMENTATION=""
    OTHER=""

    while IFS= read -r commit; do
        if [[ -z "$commit" ]]; then
            continue
        fi

        hash=$(echo "$commit" | cut -d' ' -f1)
        message=$(echo "$commit" | cut -d' ' -f2-)

        case "$message" in
            *"feat:"*|*"feature:"*|*"add:"*)
                FEATURES="$FEATURES\n- $message ($hash)"
                ;;
            *"fix:"*|*"bug:"*|*"hotfix:"*)
                FIXES="$FIXES\n- $message ($hash)"
                ;;
            *"perf:"*|*"performance:"*|*"optimize:"*)
                PERFORMANCE="$PERFORMANCE\n- $message ($hash)"
                ;;
            *"docs:"*|*"doc:"*|*"documentation:"*)
                DOCUMENTATION="$DOCUMENTATION\n- $message ($hash)"
                ;;
            *)
                OTHER="$OTHER\n- $message ($hash)"
                ;;
        esac
    done <<< "$COMMITS"

    # Output categorized changes
    if [[ -n "$FEATURES" ]]; then
        echo "## 🚀 New Features"
        echo -e "$FEATURES"
        echo
    fi

    if [[ -n "$FIXES" ]]; then
        echo "## 🐛 Bug Fixes"
        echo -e "$FIXES"
        echo
    fi

    if [[ -n "$PERFORMANCE" ]]; then
        echo "## ⚡ Performance Improvements"
        echo -e "$PERFORMANCE"
        echo
    fi

    if [[ -n "$DOCUMENTATION" ]]; then
        echo "## 📚 Documentation"
        echo -e "$DOCUMENTATION"
        echo
    fi

    if [[ -n "$OTHER" ]]; then
        echo "## 🔧 Other Changes"
        echo -e "$OTHER"
        echo
    fi

else
    echo "**Initial release**"
    echo
fi

# Performance metrics from CI
if [[ -f "benchmark-results/startup_benchmark.json" ]]; then
    STARTUP_P95=$(jq -r '.results.p95' benchmark-results/startup_benchmark.json 2>/dev/null || echo "N/A")
    echo "## 📊 Performance Metrics"
    echo
    echo "- Startup time (P95): ${STARTUP_P95}ms"
fi

if [[ -f "benchmark-results/input_latency_benchmark.json" ]]; then
    LATENCY_P99=$(jq -r '.results.p99' benchmark-results/input_latency_benchmark.json 2>/dev/null || echo "N/A")
    echo "- Input latency (P99): ${LATENCY_P99}ms"
fi

echo
echo "## 🔧 Installation"
echo
echo "### Binary Download"
echo "Download the appropriate binary for your platform from the release assets."
echo
echo "### Package Managers"
echo "\`\`\`bash"
echo "# Via Cargo"
echo "cargo install centotype"
echo
echo "# Via npm"
echo "npm install -g centotype"
echo "\`\`\`"
echo
echo "### Build from Source"
echo "\`\`\`bash"
echo "git clone https://github.com/centotype/centotype.git"
echo "cd centotype"
echo "cargo build --release"
echo "\`\`\`"
echo
echo "## 🔍 Verification"
echo
echo "All releases are:"
echo "- ✅ Automatically tested across platforms"
echo "- ✅ Performance validated against benchmarks"
echo "- ✅ Security scanned for vulnerabilities"
echo "- ✅ Cross-compiled for multiple architectures"
echo
echo "## 📋 Supported Platforms"
echo
echo "- **Linux:** x86_64, ARM64 (Ubuntu 20.04+, RHEL 8+)"
echo "- **macOS:** x86_64, ARM64 (macOS 11+)"
echo "- **Windows:** x86_64, ARM64 (Windows 10+)"
echo
echo "## 🐛 Known Issues"
echo
echo "- Check [GitHub Issues](https://github.com/centotype/centotype/issues) for current known issues"
echo
echo "## 🤝 Contributing"
echo
echo "See [CONTRIBUTING.md](https://github.com/centotype/centotype/blob/main/CONTRIBUTING.md) for guidelines."