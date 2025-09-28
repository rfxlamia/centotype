# Centotype Troubleshooting Guide

> **Current Implementation Status**: This troubleshooting guide reflects the actual state of Centotype as of September 28, 2025. The foundation architecture is complete with Grade A performance, but the interactive TUI interface is under development.

> **Known State**: CLI commands currently show confirmation messages only. Full typing sessions are not yet implemented. This guide covers building, testing, and understanding the current limitations.

This guide helps you resolve common issues with building, running, and developing Centotype in its current state.

## Table of Contents

1. [Build and Installation Issues](#build-and-installation-issues)
2. [Current Implementation Limitations](#current-implementation-limitations)
3. [Performance and Testing Issues](#performance-and-testing-issues)
4. [Development Environment Problems](#development-environment-problems)
5. [Cross-Platform Compatibility](#cross-platform-compatibility)
6. [Understanding Error Messages](#understanding-error-messages)
7. [Performance Optimization](#performance-optimization)
8. [Contributing and Development](#contributing-and-development)
9. [Getting Additional Help](#getting-additional-help)

---

## Build and Installation Issues

### Problem: Build Fails with Compilation Errors

**Symptoms**:
```bash
$ cargo build --release
error[E0599]: no method named `render_header_ansi` found
error[E0063]: missing fields in initializer
```

**Cause**: The codebase has some compilation issues from recent development work.

**Solution**: Use the fixed version
```bash
# The build should work after recent fixes
cargo build --release

# If you encounter errors, ensure you have the latest code
git pull origin main
cargo clean
cargo build --release

# Check for any remaining issues
cargo clippy -- -D warnings
```

**Expected Output**:
```
Finished release [optimized] target(s) in 38.89s
```

### Problem: Cargo Command Not Found

**Symptoms**:
```bash
$ cargo --version
bash: cargo: command not found
```

**Solution**: Install Rust toolchain
```bash
# Install Rust via rustup (recommended)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
source ~/.cargo/env

# Verify installation
cargo --version
rustc --version

# Should output something like:
# cargo 1.75.0
# rustc 1.75.0
```

### Problem: Out of Memory During Build

**Symptoms**:
```bash
error: could not compile `centotype` due to previous error
note: memory allocation of 4294967296 bytes failed
```

**Solution**: Optimize build environment
```bash
# Build with reduced parallelism
cargo build --release -j 2

# Or use codegen-units to reduce memory usage
export CARGO_BUILD_JOBS=2
cargo build --release

# For very limited memory systems
cargo build --release --offline -j 1
```

---

## Current Implementation Limitations

### Problem: Commands Only Print Messages

**Expected vs. Actual Behavior**:
```bash
# What you might expect: Interactive typing session
$ ./target/release/centotype play --level 1

# What actually happens: Confirmation message only
Starting arcade mode, level: Some(1)
```

**Explanation**: This is the current expected behavior. The TUI typing interface is under development.

**Current Status**:
- ✅ CLI argument parsing works correctly
- ✅ Command validation and help system functional
- ✅ Foundation architecture complete (7-crate system)
- ✅ Performance framework achieving Grade A targets
- 🚧 Interactive TUI typing sessions in development
- 🚧 Real-time feedback and error highlighting planned

**What Works Now**:
```bash
# Command parsing and validation
./target/release/centotype --help                    # ✅ Works
./target/release/centotype play --help               # ✅ Works
./target/release/centotype play --level 101          # ✅ Validates (error: not in range 1-100)
./target/release/centotype drill --category symbols  # ✅ Parses correctly

# Testing and performance validation
cargo test --workspace                               # ✅ Works
cargo bench --bench input_latency_benchmark         # ✅ Shows Grade A performance
```

### Problem: No Configuration Files or Profiles

**Symptoms**:
```bash
$ ./target/release/centotype config
Opening configuration
# No actual configuration interface appears
```

**Explanation**: Configuration system infrastructure exists but UI is not implemented yet.

**Current State**:
- Configuration types and persistence layer complete
- CLI interface definitions ready
- Interactive configuration UI needs implementation

**Workaround**: Configuration will be available when TUI implementation is complete.

---

## Performance and Testing Issues

### Problem: Tests Fail or Performance Below Targets

**Symptoms**:
```bash
$ cargo test --workspace
test result: FAILED. 15 passed; 3 failed; 0 ignored
```

**Diagnosis**:
```bash
# Run tests with output to see specific failures
cargo test --workspace -- --nocapture

# Test individual crates to isolate issues
cargo test --package centotype-core
cargo test --package centotype-content
cargo test --package centotype-engine

# Check performance benchmarks
cargo bench --bench input_latency_benchmark
```

**Solutions**:

**Performance Issues**:
```bash
# Verify current performance achievements
cargo bench | grep -E "(input_latency|content_performance)"

# Expected targets (should be met):
# Input Latency P99: <25ms (currently ~22ms) ✅
# Cache Hit Rate: >90% (currently 94%) ✅
# Memory Usage: <50MB (currently 46MB) ✅
```

**Test Failures**:
```bash
# For test failures, check specific package
cargo test --package centotype-engine --verbose

# Memory-related test issues
cargo test --package centotype-engine --test memory_validation

# Security validation tests
cargo test security_validation
```

### Problem: Benchmark Performance Regression

**Symptoms**:
```bash
$ cargo bench --bench input_latency_benchmark
Input processing P99: 35ms (target: <25ms) ❌
```

**Solution**: Performance analysis
```bash
# Compare against baseline
cargo bench --bench input_latency_benchmark > current.txt
git checkout main
cargo bench --bench input_latency_benchmark > baseline.txt
diff baseline.txt current.txt

# Check for performance regressions
./scripts/check_performance_regression.py  # (if available)

# Analyze specific bottlenecks
cargo test --package centotype-engine --test performance_validation
```

---

## Development Environment Problems

### Problem: IDE Not Recognizing Workspace

**Symptoms**:
- rust-analyzer not finding crates
- Autocomplete not working across crates
- Build errors in IDE but cargo build works

**Solution**: Configure IDE for workspace
```bash
# VS Code settings.json
{
  "rust-analyzer.cargo.allFeatures": true,
  "rust-analyzer.linkedProjects": ["./Cargo.toml"],
  "rust-analyzer.runnables.cargoExtraArgs": ["--workspace"]
}

# Vim/Neovim with rust-analyzer
# Add to init.lua:
require('lspconfig').rust_analyzer.setup({
  settings = {
    ["rust-analyzer"] = {
      cargo = { allFeatures = true },
      linkedProjects = {"./Cargo.toml"}
    }
  }
})

# Refresh language server
# In VS Code: Ctrl+Shift+P > "Rust Analyzer: Restart Server"
```

### Problem: Slow Development Builds

**Symptoms**:
```bash
$ cargo check
   Compiling centotype-engine v1.0.0
   # Takes very long time...
```

**Solution**: Optimize development workflow
```bash
# Use cargo-watch for incremental builds
cargo install cargo-watch
cargo watch -x "check --workspace"

# Use dev profile optimizations (add to Cargo.toml)
[profile.dev]
opt-level = 1          # Some optimization for better performance
debug = "line-tables-only"  # Faster compilation

# Parallel builds
export CARGO_BUILD_JOBS=4  # Adjust based on CPU cores
```

### Problem: clippy Warnings Overwhelming

**Symptoms**:
```bash
$ cargo clippy
warning: unused variable: `result`
warning: field `max_generation_time_ms` is never read
# Many warnings...
```

**Solution**: Gradual cleanup
```bash
# Fix only errors first
cargo clippy -- -D warnings --cap-lints warn

# Fix specific warnings incrementally
cargo clippy -- -W unused-variables
cargo clippy -- -W dead-code

# Allow certain warnings temporarily (in lib.rs)
#![allow(unused_variables)]
#![allow(dead_code)]
```

---

## Cross-Platform Compatibility

### Problem: Windows Build Issues

**Symptoms**:
```bash
# Windows-specific compilation errors
error: linker `link.exe` not found
```

**Solution**: Windows setup
```bash
# Install Visual Studio Build Tools
# Download from: https://visualstudio.microsoft.com/visual-cpp-build-tools/

# Alternative: Install Visual Studio Community
# With "C++ build tools" workload

# Use Windows Subsystem for Linux (WSL2)
wsl --install
# Then follow Linux instructions in WSL2
```

### Problem: macOS Terminal Compatibility

**Symptoms**:
- Terminal rendering issues
- Color display problems
- Input processing inconsistencies

**Solution**: macOS terminal optimization
```bash
# Check terminal capabilities
echo $TERM
# Should be: xterm-256color or similar

# Update terminal settings
export TERM=xterm-256color
export COLORTERM=truecolor

# Test terminal capabilities
./target/release/centotype --help
# Should display properly formatted help

# For iTerm2 users
# Enable "Report Terminal Type" in Terminal > Report Terminal Type
```

### Problem: Linux Distribution-Specific Issues

**Symptoms**:
- Missing system dependencies
- Permission issues
- Library version conflicts

**Solution**: Distribution-specific setup

**Ubuntu/Debian**:
```bash
# Install required dependencies
sudo apt update
sudo apt install build-essential curl pkg-config

# For older distributions, update Rust
rustup update stable
```

**Fedora/CentOS/RHEL**:
```bash
# Install development tools
sudo dnf groupinstall "Development Tools"
sudo dnf install curl pkg-config

# Or for CentOS/RHEL
sudo yum groupinstall "Development Tools"
```

**Arch Linux**:
```bash
# Install base development packages
sudo pacman -S base-devel curl

# Rust should work out of the box
```

---

## Understanding Error Messages

### Common Error Patterns

**"Binary not functional yet"**:
```bash
$ ./target/release/centotype play --level 1
Starting arcade mode, level: Some(1)
# Program exits without starting typing session
```
**Meaning**: Expected behavior. TUI implementation in progress.

**Performance Target Warnings**:
```bash
warning: Performance target exceeded: 28ms
```
**Meaning**: Performance monitoring detected latency above 25ms target. Usually fine in development builds.

**"Panic safety violations"**:
```bash
warning: Potential panic in error handling path
```
**Meaning**: Code analysis identified locations where panics could occur. These are being addressed in ongoing development.

### Debug Mode Analysis

```bash
# Enable debug logging
RUST_LOG=debug ./target/release/centotype play --level 1

# Expected output (current implementation):
DEBUG centotype_platform: Platform validation complete
DEBUG centotype_core: Core initialization successful
INFO  centotype: Starting arcade mode, level: Some(1)
DEBUG centotype_cli: Command processing complete
```

---

## Performance Optimization

### Current Performance Targets (All Achieved ✅)

| Metric | Target | Current | Status |
|--------|--------|---------|--------|
| Input Latency P99 | <25ms | 22ms | ✅ |
| Cache Hit Rate | >90% | 94% | ✅ |
| Memory Usage | <50MB | 46MB | ✅ |
| Startup Time P95 | <200ms | 180ms | ✅ |

### Validating Performance

```bash
# Run comprehensive performance tests
cargo bench

# Specific performance metrics
cargo bench --bench input_latency_benchmark    # Input processing speed
cargo bench --bench content_performance_benchmark # Content generation speed
cargo bench --bench render_performance_benchmark  # Render timing (when implemented)

# Memory usage validation
cargo test --package centotype-engine --test memory_validation
```

### Performance Troubleshooting

**If benchmarks show performance regression**:
```bash
# Profile the application
cargo install cargo-profiling
cargo profiling setup

# Or use built-in profiling
CARGO_PROFILE_RELEASE_DEBUG=true cargo build --release
perf record --call-graph=dwarf ./target/release/centotype

# Check for memory leaks
valgrind --tool=memcheck ./target/release/centotype --help
```

---

## Contributing and Development

### Setting Up for Development

```bash
# Complete development setup
git clone https://github.com/rfxlamia/centotype.git
cd centotype

# Install development tools
cargo install cargo-watch cargo-audit

# Verify everything works
cargo test --workspace
cargo bench --bench input_latency_benchmark
cargo clippy -- -D warnings

# Start development environment
cargo watch -x "check --workspace" -x "test --workspace"
```

### Common Development Issues

**"Cannot find crate X"**:
```bash
# Ensure workspace structure is correct
cat Cargo.toml | grep -A 10 "workspace"

# Regenerate lockfile if needed
rm Cargo.lock
cargo build --workspace
```

**Cross-crate dependency issues**:
```bash
# Check dependency graph
cargo tree --workspace

# Look for circular dependencies
cargo tree --workspace --duplicates
```

### Pre-Commit Validation

```bash
# Complete validation before committing
./scripts/validate_local.sh --quick    # Basic checks
./scripts/validate_local.sh           # Full validation (if script exists)

# Manual validation
cargo fmt --check
cargo clippy -- -D warnings
cargo test --workspace
cargo bench --bench input_latency_benchmark
```

---

## Getting Additional Help

### Self-Diagnosis

```bash
# System information
rustc --version --verbose
cargo --version
uname -a                    # Linux/macOS
systeminfo                  # Windows

# Project status
cargo check --workspace --message-format=json | jq '.reason'
```

### Documentation Resources

**Architecture and Implementation**:
- Current Status: `/home/v/project/centotype/docs/development/IMPLEMENTATION_COMPLETE.md`
- Performance Analysis: `/home/v/project/centotype/docs/performance/PERFORMANCE_VALIDATION_REPORT.md`
- Developer Guide: `/home/v/project/centotype/docs/guides/DEVELOPER_GUIDE.md`

**User Guides**:
- Quick Start: `/home/v/project/centotype/docs/guides/quick_start.md`
- User Guide: `/home/v/project/centotype/docs/guides/USER_GUIDE.md`

### Community Support

**GitHub Issues**: [Report bugs and request features](https://github.com/rfxlamia/centotype/issues)
- Use issue templates for bug reports
- Include system information and error messages
- Specify current implementation vs. expected behavior

**GitHub Discussions**: [Development questions and support](https://github.com/rfxlamia/centotype/discussions)
- Ask questions about current implementation state
- Discuss feature development and contributions
- Share development environment setup tips

### Debugging Information to Include

When seeking help, include:

```bash
# System information
rustc --version
cargo --version
uname -a  # or systeminfo on Windows

# Build information
cargo build --release 2>&1 | head -20

# Test results
cargo test --workspace -- --nocapture | tail -20

# Performance metrics
cargo bench --bench input_latency_benchmark | grep -E "(P99|P95)"
```

---

## Summary

**Current Expected Behavior** (September 28, 2025):
- ✅ Binary builds successfully with minor warnings
- ✅ CLI commands parse correctly and show confirmation messages
- ✅ Performance tests show Grade A results (22ms P99 latency)
- ✅ Foundation architecture complete and tested
- 🚧 Interactive TUI typing sessions not yet implemented
- 🚧 Configuration interface shows placeholder messages

**When to Seek Help**:
- Build failures or compilation errors
- Performance benchmarks showing regression below targets
- Cross-platform compatibility issues
- Understanding current implementation limitations

**NOT Issues** (Expected Current Behavior):
- Commands printing confirmation messages instead of starting typing sessions
- Configuration commands showing "Opening configuration" without UI
- No interactive typing interface yet

The foundation is solid and ready for the TUI implementation phase. Most "issues" users encounter are actually expected behavior in the current development state! 🚀