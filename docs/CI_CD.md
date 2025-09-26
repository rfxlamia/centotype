# Centotype CI/CD Pipeline Documentation

This document describes the comprehensive CI/CD pipeline for the Centotype CLI typing trainer.

## Overview

The CI/CD pipeline provides:
- ✅ Cross-platform builds (Linux, macOS, Windows on x64/ARM64)
- ✅ Performance validation against strict requirements
- ✅ Security testing and vulnerability scanning
- ✅ Automated releases with multi-platform binaries
- ✅ Package distribution (Cargo, npm)

## Pipeline Components

### 1. Quality Gates (`check` job)
- **Code formatting** with `cargo fmt`
- **Linting** with `clippy` (deny warnings)
- **Documentation** generation and validation
- **Fast feedback** for pull requests (< 10 minutes)

### 2. Security Scanning (`security` job)
- **Vulnerability audit** with `cargo-audit`
- **Dependency validation** with `cargo-deny`
- **Input fuzzing tests** for terminal security
- **License compliance** checking

### 3. Cross-Platform Builds (`build` job)
- **6 target platforms**: Linux/macOS/Windows on x64/ARM64
- **Cross-compilation** with optimized caching
- **Release artifacts** for distribution
- **Build verification** across all targets

### 4. Comprehensive Testing (`test` job)
- **Unit tests** across all workspace crates
- **Integration tests** for end-to-end functionality
- **Documentation tests** for code examples
- **Coverage reporting** (80% minimum)

### 5. Performance Validation (`performance` job)
- **Startup time**: P95 < 200ms
- **Input latency**: P99 < 25ms
- **Render performance**: P95 < 33ms
- **Memory usage**: < 50MB RSS
- **Regression detection** for pull requests

### 6. Release Automation (`release` job)
- **GitHub releases** with cross-platform binaries
- **Crates.io publishing** in dependency order
- **npm package** with binary wrapper
- **Changelog generation** from git history

## Performance Requirements

| Metric | Threshold | Validation Method |
|--------|-----------|-------------------|
| P99 Input Latency | < 25ms | Automated benchmark |
| P95 Startup Time | < 200ms | Script measurement |
| P95 Render Time | < 33ms | Performance tests |
| Memory Usage (RSS) | < 50MB | Memory profiling |

## Security Testing

### Input Validation
- ✅ ANSI escape sequence filtering
- ✅ Control character sanitization
- ✅ Unicode edge case handling
- ✅ Buffer overflow protection

### Vulnerability Scanning
- ✅ Dependency audit (cargo-audit)
- ✅ License compliance (cargo-deny)
- ✅ Security advisory monitoring
- ✅ Supply chain validation

## Local Development

### Quick Validation
```bash
# Run essential checks (5-10 minutes)
./scripts/validate_local.sh --quick
```

### Full Validation
```bash
# Run complete validation including performance tests (20-30 minutes)
./scripts/validate_local.sh
```

### Performance Testing
```bash
# Startup time benchmark
./scripts/benchmark_startup.sh

# Input latency benchmark
./scripts/benchmark_latency.sh

# Performance comparison (for PRs)
./scripts/compare_performance.sh
```

## Workflow Triggers

### Continuous Integration
- **Push to main/develop**: Full pipeline
- **Pull requests**: All jobs except release
- **Nightly schedule**: Extended performance monitoring

### Release Process
- **Git tag (v*)**: Complete release pipeline
- **Manual trigger**: Emergency releases

## Supported Platforms

### Primary Targets
| Platform | Architecture | OS Version | Status |
|----------|--------------|------------|--------|
| Linux | x86_64 | Ubuntu 20.04+ | ✅ |
| Linux | ARM64 | Ubuntu 20.04+ | ✅ |
| macOS | x86_64 | macOS 11+ | ✅ |
| macOS | ARM64 | macOS 11+ | ✅ |
| Windows | x86_64 | Windows 10+ | ✅ |
| Windows | ARM64 | Windows 10+ | ✅ |

### Terminal Compatibility
- ✅ xterm, gnome-terminal
- ✅ iTerm2, Terminal.app
- ✅ Windows Terminal, cmd.exe
- ✅ VS Code integrated terminal

## Distribution Channels

### Package Managers
```bash
# Cargo (Rust)
cargo install centotype

# npm (Node.js wrapper)
npm install -g centotype
```

### Direct Download
- GitHub Releases with platform-specific binaries
- Checksums for verification
- Installation scripts

## Monitoring & Alerts

### Performance Monitoring
- **Nightly benchmarks** track performance trends
- **Regression alerts** for threshold violations
- **Historical data** for performance analysis

### Security Monitoring
- **Daily vulnerability scans** of dependencies
- **Advisory notifications** for security issues
- **License compliance** tracking

## Configuration Files

| File | Purpose |
|------|---------|
| `.github/workflows/ci.yml` | Main CI/CD pipeline |
| `deny.toml` | Dependency validation rules |
| `.centotype-ci.toml` | Performance thresholds |
| `scripts/validate_local.sh` | Local development validation |

## Troubleshooting

### Common Issues

#### Build Failures
```bash
# Clean build cache
cargo clean

# Update dependencies
cargo update

# Check Rust version
rustc --version  # Should be 1.75.0+
```

#### Performance Test Failures
```bash
# Check system load
top

# Run isolated test
cargo test --package centotype-engine --test performance_validation -- --ignored --nocapture
```

#### Security Scan Issues
```bash
# Update advisory database
cargo audit update

# Check specific advisory
cargo audit --file Cargo.lock --db ~/.cargo/advisory-db
```

### Getting Help

1. **Check pipeline logs** in GitHub Actions
2. **Run local validation** to reproduce issues
3. **Review performance reports** in artifacts
4. **Consult documentation** for configuration details

## Performance Optimization

### Build Optimization
- **LTO (Link Time Optimization)** enabled for release builds
- **Single codegen unit** for maximum optimization
- **Symbol stripping** to reduce binary size
- **Profile-guided optimization** for critical paths

### Runtime Optimization
- **Lazy initialization** of heavy components
- **Memory pooling** for frequent allocations
- **SIMD optimizations** where applicable
- **Lock-free data structures** for concurrent access

## Future Enhancements

### Planned Improvements
- [ ] ARM64 Windows native builds
- [ ] WebAssembly target support
- [ ] Container image distribution
- [ ] Homebrew formula
- [ ] Debian/RPM packages

### Monitoring Enhancements
- [ ] Real-time performance dashboards
- [ ] User telemetry (opt-in)
- [ ] Crash reporting
- [ ] Usage analytics

---

For questions or issues with the CI/CD pipeline, please create an issue in the GitHub repository or contact the development team.