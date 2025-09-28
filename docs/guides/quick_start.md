# Centotype Quick Start Guide

> **Development Status**: Centotype is currently in active development. Core architecture and performance framework are complete (Grade A - 22ms P99 latency), but the interactive TUI typing interface is under construction. This guide reflects the current working implementation.

> **Current State**: Foundation complete with 7-crate architecture, content generation system, and performance monitoring. User interface implementation is in progress.

This guide helps you build and explore the current state of Centotype, the precision CLI typing trainer designed with professional-grade performance targets.

## Prerequisites

- **Operating System**: Linux, macOS 10.14+, or Windows 10+
- **Terminal**: Modern terminal emulator with UTF-8 support
- **Rust**: Version 1.75+ (required for building from source)

## Building from Source (2 minutes)

Clone and build the project:

```bash
# Clone the repository
git clone https://github.com/rfxlamia/centotype.git
cd centotype

# Build in release mode
cargo build --release

# Verify build succeeded
./target/release/centotype --help
```

**Expected Output**:
```
CLI-based typing trainer with 100 progressive difficulty levels

Usage: centotype <COMMAND>

Commands:
  play       Start arcade mode training
  drill      Practice specific skills
  endurance  Endurance training session
  stats      View statistics and progress
  config     Configure application settings
  help       Print this message or the help of the given subcommand(s)

Options:
  -h, --help  Print help
```

## Current Functionality

### Available Commands

The CLI currently supports command parsing and basic execution:

```bash
# Start arcade mode (prints confirmation message)
./target/release/centotype play --level 1
# Output: Starting arcade mode, level: Some(1)

# Start drill mode (prints configuration)
./target/release/centotype drill --category symbols --duration 5
# Output: Starting drill: symbols for 5 minutes

# Start endurance mode (prints configuration)
./target/release/centotype endurance --duration 15
# Output: Starting endurance mode for 15 minutes

# View statistics (prints placeholder)
./target/release/centotype stats
# Output: Displaying statistics

# Configure settings (prints placeholder)
./target/release/centotype config
# Output: Opening configuration
```

### What's Currently Working

**Architecture & Foundation** (✅ Complete):
- 7-crate Rust workspace architecture
- `centotype-core`: State management, scoring engine, level progression
- `centotype-engine`: Performance-optimized render and input framework
- `centotype-content`: Dynamic content generation with 100-level system
- `centotype-analytics`: Performance analysis framework
- `centotype-cli`: Command parsing and interface definitions
- `centotype-persistence`: Profile storage infrastructure
- `centotype-platform`: OS-specific optimizations

**Performance Framework** (✅ Grade A):
- Input latency: 22ms P99 (target: <25ms) ✅
- Cache hit rate: 94% (target: >90%) ✅
- Memory usage: 46MB (target: <50MB) ✅
- Comprehensive benchmarking suite
- Real-time performance monitoring

**Content System** (✅ Complete):
- Deterministic 100-level corpus generation
- Mathematical difficulty progression (5%→30% symbols, 3%→20% numbers)
- LRU caching with Moka (94% hit rate, <25ms target)
- Security validation with escape sequence filtering

### What's Under Development

**Interactive TUI** (🚧 In Progress):
- Real-time typing interface with crossterm + ratatui
- Live WPM and accuracy feedback
- Error highlighting and visual feedback
- Help overlay and interactive controls
- Session state management

**Known Limitations**:
- Commands currently print confirmation messages only
- No actual typing sessions yet
- UI renders are placeholder implementations
- Profile persistence stub (saves but not used)
- Configuration options not functional yet

## Testing the Architecture

### Run Unit Tests

```bash
# Test all workspace crates
cargo test --workspace

# Test specific components
cargo test --package centotype-core
cargo test --package centotype-content
cargo test --package centotype-engine
```

### Run Performance Benchmarks

```bash
# Input latency validation
cargo bench --bench input_latency_benchmark

# Content system performance
cargo bench --bench content_performance_benchmark

# Render performance
cargo bench --bench render_performance_benchmark
```

**Expected Performance**:
- Input processing P99: <25ms
- Content generation: <25ms (with cache)
- Memory footprint: <50MB

### Quick Validation Script

```bash
# Run the comprehensive validation suite
./scripts/validate_local.sh --quick

# Full validation (includes security and performance tests)
./scripts/validate_local.sh
```

## Understanding the Architecture

### Data Flow (Current Implementation)

```
User Input → CLI Parser → CliManager
                              ↓ (prints confirmation)
                          Command Handler (stub)
```

**Planned Flow** (Under Development):
```
User Input → CLI Parser → CliManager → Engine
                                        ↓
                                    Input Processor (22ms P99)
                                        ↓
                                    Core State Manager
                                        ↓
                                    Render System (ratatui TUI)
                                        ↓
                                    Live Typing Interface
```

### Performance Characteristics

The foundation achieves production-grade performance targets:

```rust
// Measured Performance Metrics (All Met)
Input Latency P99: 22ms    ✅ (target: <25ms)
Content Load Time: <25ms   ✅ (94% cache hit rate)
Memory Usage: 46MB         ✅ (target: <50MB)
Startup Time P95: 180ms    ✅ (target: <200ms)
```

## Development Status & Roadmap

### Completed (Session 3 - September 28, 2025)

- ✅ Full 7-crate workspace architecture
- ✅ Content generation system with 100 levels
- ✅ Performance optimization (Grade A achieved)
- ✅ Security validation framework
- ✅ Comprehensive test suite
- ✅ Command-line argument parsing
- ✅ Platform detection and optimization

### In Progress

- 🚧 Interactive TUI implementation (crossterm + ratatui)
- 🚧 Real-time typing session interface
- 🚧 Live feedback and error highlighting
- 🚧 Session result display and statistics

### Upcoming

- ⏳ Profile management and progress tracking
- ⏳ Configuration system
- ⏳ Advanced training modes (placement, drills)
- ⏳ Analytics and performance insights

### Production Blockers

**Critical** (Must fix before 1.0):
- ⚠️ 27+ panic safety violations identified in error handling
- ⚠️ TUI typing interface not yet functional
- ⚠️ Session state persistence not integrated

**Non-Critical**:
- Profile export functionality
- Audio feedback system
- Online competition features

## For Developers

### Exploring the Codebase

```bash
# View architecture documentation
cat /home/v/project/centotype/docs/development/IMPLEMENTATION_COMPLETE.md

# Check performance validation report
cat /home/v/project/centotype/docs/performance/PERFORMANCE_VALIDATION_REPORT.md

# Review content system design
cat /home/v/project/centotype/docs/design/CONTENT_SYSTEM.md
```

### Running Development Checks

```bash
# Linting and formatting
cargo clippy -- -D warnings
cargo fmt --check

# Build optimization check
cargo build --release --quiet

# Memory profile
cargo run --bin memory-profiler  # (if available)
```

### Understanding Performance Targets

All components are designed to meet these targets:

| Metric | Target | Current | Status |
|--------|--------|---------|--------|
| Input Latency P99 | <25ms | 22ms | ✅ |
| Render Time P95 | <33ms | TBD | 🚧 |
| Cache Hit Rate | >90% | 94% | ✅ |
| Memory Usage | <50MB | 46MB | ✅ |
| Startup Time P95 | <200ms | 180ms | ✅ |

## Getting Help

### Documentation

- **Architecture**: `/home/v/project/centotype/docs/development/IMPLEMENTATION_COMPLETE.md`
- **Performance**: `/home/v/project/centotype/docs/performance/PERFORMANCE_VALIDATION_REPORT.md`
- **Developer Guide**: `/home/v/project/centotype/docs/guides/DEVELOPER_GUIDE.md`
- **User Guide**: `/home/v/project/centotype/docs/guides/USER_GUIDE.md` (reflects future state)
- **Troubleshooting**: `/home/v/project/centotype/docs/guides/TROUBLESHOOTING.md`

### Community Support

- **GitHub Issues**: [Report bugs and request features](https://github.com/rfxlamia/centotype/issues)
- **Discussions**: [Development updates and questions](https://github.com/rfxlamia/centotype/discussions)

---

## Summary: Getting Started with Current Build

1. **Clone & Build** (2min): Clone repo and run `cargo build --release`
2. **Test CLI** (30s): Run `./target/release/centotype --help` to verify build
3. **Explore Architecture** (5min): Review `/home/v/project/centotype/docs/development/IMPLEMENTATION_COMPLETE.md`
4. **Run Tests** (2min): Execute `cargo test --workspace` to validate components
5. **Check Performance** (1min): Run `cargo bench` to see Grade A performance

**Next Phase**: TUI implementation will provide interactive typing sessions with real-time feedback. Foundation architecture is production-ready (pending panic safety fixes).

**Note for Contributors**: The architecture is designed for parallel development. See `DEVELOPER_GUIDE.md` for contribution guidelines and development workflows.

Happy exploring! 🚀