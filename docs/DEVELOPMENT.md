# Development Workflow Guide

This guide describes the recommended development workflow for contributing to Centotype.

## Quick Start

### Prerequisites
```bash
# Install Rust (1.75.0+)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install required tools
cargo install cargo-audit cargo-deny

# Clone repository
git clone https://github.com/centotype/centotype.git
cd centotype
```

### Development Workflow

#### 1. Pre-Development Validation
```bash
# Quick check before starting work
./scripts/validate_local.sh --quick
```

#### 2. Development Cycle
```bash
# Build and test changes
cargo build
cargo test

# Run specific tests
cargo test --package centotype-core
```

#### 3. Pre-Commit Validation
```bash
# Full validation before committing
./scripts/validate_local.sh

# Or run individual checks
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

#### 4. Performance Testing (Optional)
```bash
# Run performance benchmarks
./scripts/benchmark_startup.sh
./scripts/benchmark_latency.sh

# Memory validation
cargo test --package centotype-engine --test memory_validation -- --ignored
```

## Commit Guidelines

### Commit Message Format
```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

### Types
- `feat`: New feature
- `fix`: Bug fix
- `perf`: Performance improvement
- `docs`: Documentation changes
- `style`: Code style changes
- `refactor`: Code refactoring
- `test`: Test additions/modifications
- `build`: Build system changes
- `ci`: CI/CD changes

### Examples
```bash
git commit -m "feat(engine): add input latency tracking"
git commit -m "fix(cli): resolve startup crash on Windows"
git commit -m "perf(core): optimize text rendering pipeline"
```

## Pull Request Process

### 1. Create Feature Branch
```bash
git checkout -b feature/your-feature-name
```

### 2. Development and Testing
- Make your changes
- Run local validation
- Add tests for new functionality
- Update documentation if needed

### 3. Performance Validation
```bash
# Ensure no performance regressions
./scripts/compare_performance.sh
```

### 4. Submit Pull Request
- Push your branch
- Create PR with descriptive title and description
- Wait for CI/CD pipeline to complete
- Address any feedback

### 5. Merge Process
- All CI checks must pass
- Code review approval required
- Automatic performance comparison
- Squash and merge to main

## Testing Strategy

### Unit Tests
```bash
# Run all unit tests
cargo test --workspace --lib

# Run specific crate tests
cargo test --package centotype-engine --lib
```

### Integration Tests
```bash
# Run integration tests
cargo test --workspace --test '*'

# Run specific integration test
cargo test --test terminal_integration
```

### Performance Tests
```bash
# Run performance validation
cargo test --package centotype-engine --test performance_validation -- --ignored

# Run memory tests
cargo test --package centotype-engine --test memory_validation -- --ignored
```

### Security Tests
```bash
# Run input fuzzing tests
cargo test --package centotype-engine --test fuzz_input -- --ignored

# Security audit
cargo audit
```

## Performance Guidelines

### Requirements
- P99 input latency < 25ms
- P95 startup time < 200ms
- P95 render time < 33ms
- Memory usage < 50MB RSS

### Optimization Tips
1. **Minimize allocations** in hot paths
2. **Use lazy initialization** for heavy objects
3. **Profile before optimizing** with `cargo bench`
4. **Test on all platforms** via CI

### Benchmarking
```bash
# Run all benchmarks
cargo bench --workspace

# Run specific benchmark
cargo bench --package centotype-engine input_latency
```

## Debugging

### Debug Builds
```bash
# Build with debug info
cargo build

# Run with debug logging
RUST_LOG=debug target/debug/centotype
```

### Performance Debugging
```bash
# Build performance test binary
cargo build --profile perf-test

# Profile with perf (Linux)
perf record target/perf-test/centotype
perf report
```

### Memory Debugging
```bash
# Valgrind (Linux)
valgrind --tool=memcheck target/debug/centotype

# Instruments (macOS)
instruments -t "Leaks" target/debug/centotype
```

## Release Process

### Version Management
1. Update version in `Cargo.toml`
2. Update `CHANGELOG.md`
3. Create git tag: `git tag v1.0.0`
4. Push tag: `git push origin v1.0.0`

### Release Validation
- All tests pass
- Performance benchmarks within thresholds
- Security scans clean
- Cross-platform builds successful

## IDE Setup

### VS Code
```json
// .vscode/settings.json
{
    "rust-analyzer.cargo.features": "all",
    "rust-analyzer.checkOnSave.command": "clippy",
    "rust-analyzer.checkOnSave.allTargets": true
}
```

### Recommended Extensions
- rust-analyzer
- CodeLLDB (debugging)
- Error Lens
- GitLens

## Troubleshooting

### Common Issues

#### Compilation Errors
```bash
# Clean build
cargo clean

# Update toolchain
rustup update

# Check Rust version
rustc --version
```

#### Test Failures
```bash
# Run with nocapture for full output
cargo test -- --nocapture

# Run single test
cargo test test_name -- --exact
```

#### Performance Issues
```bash
# Profile debug builds
cargo build --profile perf-test
./scripts/benchmark_startup.sh
```

### Getting Help

1. Check existing GitHub issues
2. Run `./scripts/validate_local.sh --help`
3. Review CI/CD logs for similar failures
4. Contact team via discussions

## Contributing Guidelines

### Code Style
- Follow Rust conventions
- Use `cargo fmt` for formatting
- Address all `clippy` warnings
- Write comprehensive tests

### Documentation
- Update relevant documentation
- Add docstrings for public APIs
- Include examples in doc comments
- Update README if needed

### Testing
- Add unit tests for new functionality
- Include integration tests for features
- Verify performance impact
- Test across platforms

---

For more detailed information, see:
- [CI/CD Pipeline Documentation](./CI_CD.md)
- [Architecture Documentation](../ARCHITECTURE.md)
- [Performance Requirements](../prd_vnext.md)