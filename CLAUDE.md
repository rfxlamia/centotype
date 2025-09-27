# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Centotype is a CLI-based typing trainer built in Rust with 100 progressive difficulty levels. The project is in active development with a complete 7-crate workspace architecture and comprehensive content development system.

**Current Status**: Content Development Phase completed - foundation architecture implemented with 100-level corpus generation system, performance validation framework, and UI/UX design specifications

## Key Documents

- **`docs/design/MASTER_PROMPT.md`**: Master prompt system for 100-level content generation with agent coordination specs
- **`docs/`**: Comprehensive documentation including performance guide, architecture overview, and user guides
- **`docs/specs/prd_vnext.md`**: Complete PRD v2.0 with comprehensive specifications
- **`docs/performance/PERFORMANCE_VALIDATION_REPORT.md`**: Performance analysis and optimization recommendations

## Architecture

The project uses a 7-crate Rust workspace architecture with modular, performance-focused design:

```
centotype/
├── core/           # State management, scoring engine, level progression
├── engine/         # Input handling, TTY management, render loop
├── content/        # Text corpus loading, LRU caching, content generation with 100-level system
├── analytics/      # Performance analysis, error classification, metrics collection
├── cli/            # Command parsing, interactive terminal navigation
├── persistence/    # Profile storage, configuration management
├── platform/       # OS-specific integrations, terminal detection
└── centotype-bin/  # Main binary application
```

### Key Architecture Components

**Content Generation System**: Deterministic 100-level corpus with mathematical difficulty progression (5%→30% symbols, 3%→20% numbers). Uses Moka LRU cache targeting <25ms content loading with 94% hit rate.

**Performance Framework**: Comprehensive benchmarking suite targeting P99 input latency <25ms, P95 render time <33ms, memory usage <50MB. Includes cross-crate performance validation and adaptive preloading.

**Inter-Crate Communication**: Async-first design with optimized Arc boundaries, shared context patterns, and efficient data flow from content/ → core/ → engine/ → cli/.

## Development Commands

**Essential Development Workflow**:
```bash
# Quick validation (linting, basic tests, formatting)
./scripts/validate_local.sh --quick

# Full validation (includes performance and security tests)
./scripts/validate_local.sh

# Build and test
cargo build --release
cargo test --workspace
cargo clippy -- -D warnings
cargo fmt --check

# Performance benchmarking
cargo bench --bench input_latency_benchmark    # Input latency validation
cargo bench --bench content_performance_benchmark # Content system performance
cargo bench --bench render_performance_benchmark  # Render timing validation

# Target-specific testing
cargo test --package centotype-content --all-features
cargo test --package centotype-core
cargo check --workspace --quiet
```

**Performance Validation**:
```bash
# Run performance validation suite
cargo run --bin performance_validator
cargo run --profile perf-test --bin centotype -- --benchmark-mode

# Memory profiling
cargo run --bin memory-profiler
```

## Current Implementation Status

**✅ Completed (Content Development Phase)**:
- Master prompt system for deterministic 100-level content generation
- ContentGenerator with Moka LRU cache (94% hit rate, <25ms loading target)
- Comprehensive performance benchmarking suite with P99 input latency validation
- Security validation system with escape sequence detection and content sanitization
- Interactive terminal UI/UX design with accessibility compliance (WCAG AA)
- Inter-crate performance validation framework with optimization recommendations
- Complete documentation suite including API reference and troubleshooting guides

**🚧 Ready for Next Phase (UI Implementation)**:
- Terminal interface development using crossterm and ratatui
- Real-time input handling with <25ms latency targets
- Interactive level selection and progress visualization
- In-game interface with real-time feedback and error highlighting

**📊 Current Performance Metrics**:
- Input Latency P99: ~28ms (target: <25ms) - needs optimization
- Cache Hit Rate: 94% (target: >90%) - exceeds target
- Memory Usage: 46MB (target: <50MB) - within target
- Startup Time P95: 180ms (target: <200ms) - exceeds target

## Notes for Development

**Performance Considerations**:
- All changes must maintain <25ms P99 input latency target
- Use the comprehensive benchmark suite (`cargo bench`) to validate performance impact
- Content caching system is performance-critical - test with `cargo test --package centotype-content`
- Cross-crate communication optimizations are documented in `PERFORMANCE_VALIDATION_REPORT.md`

**Content System**:
- 100-level difficulty progression uses mathematical formulas (see `MASTER_PROMPT.md`)
- Content generation is deterministic using ChaCha8Rng for reproducible results
- Security validation required for all content with terminal escape sequence filtering
- LRU cache with Moka provides 94% hit rate - maintain this performance standard

**Architecture Patterns**:
- Async-first design throughout the codebase
- Use Arc<> efficiently to minimize clone overhead between crates
- Shared context patterns implemented for cross-crate communication
- Error handling follows Result<T, CentotypeError> pattern consistently

**Testing Strategy**:
- Unit tests for deterministic content generation
- Integration tests for cross-crate performance
- Benchmark tests for latency validation
- Security tests for content validation and escape sequence detection