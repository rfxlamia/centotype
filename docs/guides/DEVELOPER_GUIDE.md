# Centotype Developer Guide

> **Current Implementation Status**: This developer guide reflects the actual state of Centotype as of September 28, 2025. The foundation architecture is production-ready with Grade A performance (22ms P99 latency), but the interactive TUI interface is under active development.

> **For Contributors**: This guide documents what's implemented, what works, and what needs to be completed. Perfect for developers wanting to contribute to the TUI implementation or performance optimizations.

This comprehensive technical guide covers development setup, architecture understanding, testing strategies, and contribution guidelines for the Centotype typing trainer project.

## Table of Contents

1. [Current Implementation Status](#current-implementation-status)
2. [Development Environment Setup](#development-environment-setup)
3. [Project Architecture](#project-architecture)
4. [Development Workflow](#development-workflow)
5. [Testing and Performance](#testing-and-performance)
6. [Code Quality Standards](#code-quality-standards)
7. [Inter-Crate Development](#inter-crate-development)
8. [TUI Implementation Guide](#tui-implementation-guide)
9. [Performance Development](#performance-development)
10. [Contributing Guidelines](#contributing-guidelines)

---

## Current Implementation Status

### What's Production-Ready ✅

**Core Architecture (100% Complete)**:
- 7-crate workspace with modular design
- Type system with shared interfaces
- Error handling with CentotypeError
- Performance monitoring infrastructure
- Cross-platform compatibility layer
- Security validation framework

**Content System (100% Complete)**:
- Dynamic 100-level corpus generation
- Mathematical difficulty progression
- LRU caching with 94% hit rate
- Content validation and sanitization
- Deterministic generation with ChaCha8Rng

**Performance Framework (Grade A)**:
- Input processing: 22ms P99 (target: <25ms) ✅
- Memory usage: 46MB (target: <50MB) ✅
- Cache performance: 94% hit rate ✅
- Startup time: 180ms P95 ✅

**CLI Interface (Parsing Complete)**:
- Command argument parsing with clap
- Proper validation and error messages
- Help system and command structure
- Platform initialization sequence

### What's Under Development 🚧

**TUI Implementation (In Progress)**:
- Interactive typing interface (crossterm + ratatui)
- Real-time input processing and feedback
- Session state management
- Live metrics display and error highlighting

**Integration Layer (Needs Work)**:
- Engine ↔ Core communication
- Session persistence integration
- Configuration system implementation
- Profile management functionality

### Critical Issues ⚠️

**Safety Violations** (Production Blocker):
- 27+ panic safety violations identified
- Error propagation needs improvement
- Resource cleanup patterns need hardening

**Missing Functionality**:
- TUI typing sessions (placeholder implementations)
- Real-time render pipeline incomplete
- Configuration system non-functional

---

## Development Environment Setup

### Prerequisites

```bash
# Required Rust toolchain
rustc 1.75.0+          # Latest stable Rust
cargo 1.75.0+          # Cargo package manager
git 2.30+              # Version control

# Development tools (recommended)
cargo-watch            # File watching during development
cargo-audit            # Security auditing
criterion              # Benchmarking framework
```

### Quick Setup

```bash
# Clone the repository
git clone https://github.com/rfxlamia/centotype.git
cd centotype

# Install development dependencies
cargo install cargo-watch cargo-audit

# Verify build works
cargo build --release

# Run test suite
cargo test --workspace

# Check performance benchmarks
cargo bench --bench input_latency_benchmark
```

### Development Dependencies

The workspace uses these key dependencies:

```toml
# Core dependencies (shared across crates)
tokio = { version = "1.47", features = ["full"] }
tracing = "0.1"
anyhow = "1.0"
clap = { version = "4.5", features = ["derive"] }

# TUI and terminal handling
crossterm = "0.27"
ratatui = "0.24"

# Performance and caching
moka = { version = "0.12", features = ["future"] }
criterion = "0.5"

# Testing
proptest = "1.4"
```

### IDE Configuration

**VS Code (Recommended)**:
```json
{
  "rust-analyzer.cargo.allFeatures": true,
  "rust-analyzer.check.command": "clippy",
  "rust-analyzer.linkedProjects": ["./Cargo.toml"],
  "rust-analyzer.runnables.cargoExtraArgs": ["--workspace"]
}
```

**Vim/Neovim with rust-analyzer**:
```lua
-- LSP configuration for rust-analyzer
require('lspconfig').rust_analyzer.setup({
  settings = {
    ["rust-analyzer"] = {
      cargo = { allFeatures = true },
      checkOnSave = { command = "clippy" }
    }
  }
})
```

---

## Project Architecture

### 7-Crate Workspace Structure

```
centotype/                         # Root workspace
├── Cargo.toml                     # Workspace configuration
├── core/                          # ✅ Complete
│   ├── src/
│   │   ├── lib.rs                 # Main coordinator
│   │   ├── types.rs               # Shared types (critical file)
│   │   ├── session.rs             # State management
│   │   ├── scoring.rs             # Performance calculation
│   │   ├── level.rs               # Progression logic
│   │   └── error.rs               # Error classification
├── engine/                        # 🚧 TUI implementation needed
│   ├── src/
│   │   ├── lib.rs                 # Event loop coordinator
│   │   ├── input.rs               # Input processing (complete)
│   │   ├── render.rs              # Render system (placeholders)
│   │   ├── tty.rs                 # Terminal control
│   │   └── performance.rs         # Latency monitoring
├── content/                       # ✅ Complete
│   ├── src/
│   │   ├── lib.rs                 # Content manager
│   │   ├── generator.rs           # Text generation
│   │   ├── corpus.rs              # 100-level corpus
│   │   ├── difficulty.rs          # Difficulty analysis
│   │   └── cache.rs               # LRU caching
├── platform/                     # ✅ Complete
│   ├── src/
│   │   ├── lib.rs                 # Platform manager
│   │   ├── detection.rs           # OS/terminal detection
│   │   ├── terminal.rs            # Terminal capabilities
│   │   └── performance.rs         # System metrics
├── analytics/                     # ✅ Complete
│   ├── src/
│   │   ├── lib.rs                 # Analytics coordinator
│   │   ├── metrics.rs             # Performance tracking
│   │   └── insights.rs            # User analytics
├── cli/                          # ✅ Parsing complete, handlers stub
│   ├── src/
│   │   ├── lib.rs                 # CLI interface
│   │   ├── commands.rs            # Command definitions
│   │   └── navigation.rs          # Menu systems
├── persistence/                  # ✅ Infrastructure complete
│   ├── src/
│   │   ├── lib.rs                 # Persistence manager
│   │   ├── profile.rs             # User profiles
│   │   └── config.rs              # Configuration
└── centotype-bin/                # ✅ Complete integration
    └── src/main.rs                # Application entry point
```

### Data Flow Architecture

**Current Working Flow**:
```
main.rs → CLI Parser → CliManager → Print Confirmation
    ↓
Platform Detection → Core Initialization → Engine Setup (unused)
```

**Target Implementation** (TUI Integration Needed):
```
User Input → Input Processor (22ms P99) → Core State → Render System
    ↓
Terminal Events → Event Loop → Session Manager → Live Feedback
    ↓
Content Generator → Cache (94% hit) → Difficulty Analysis → Display
```

### Key Integration Points

**Critical Files for TUI Implementation**:
1. `engine/src/lib.rs` - Main event loop integration
2. `engine/src/render.rs` - TUI rendering (needs real implementation)
3. `cli/src/lib.rs` - Command handlers (currently stubs)
4. `core/src/session.rs` - Session state management

**Performance-Critical Paths**:
1. Input processing → Core state updates
2. Content generation → Cache lookups
3. Render pipeline → Terminal output
4. Session persistence → Profile updates

---

## Development Workflow

### Daily Development

```bash
# Start development session
cargo watch -x "build --workspace" -x "test --workspace"

# Run specific crate tests
cargo test --package centotype-engine
cargo test --package centotype-content --all-features

# Performance validation
cargo bench --bench input_latency_benchmark

# Check for regressions
./scripts/validate_local.sh --quick
```

### Feature Development Process

1. **Understand Current State**: Review what's implemented
2. **Identify Dependencies**: Check inter-crate requirements
3. **Write Tests First**: Add tests for new functionality
4. **Implement Incrementally**: Small, focused changes
5. **Validate Performance**: Ensure targets are maintained
6. **Update Documentation**: Keep guides current

### Testing Strategy

```bash
# Unit tests (fast feedback)
cargo test --lib --workspace

# Integration tests (cross-crate functionality)
cargo test --test integration_tests

# Performance tests (ensure no regressions)
cargo bench

# Security validation
cargo audit
cargo test security_validation
```

### Common Development Tasks

**Adding New Content Level**:
```rust
// In content/src/generator.rs
impl ContentGenerator {
    fn generate_level_x(&self, seed: u64) -> Result<TextContent> {
        // Implementation following existing pattern
    }
}
```

**Adding TUI Component**:
```rust
// In engine/src/render.rs
impl Render {
    fn render_new_component(&mut self, frame: &mut Frame) -> Result<()> {
        // Follow existing ratatui patterns
    }
}
```

**Performance Optimization**:
```rust
// Add benchmark first
#[bench]
fn bench_new_feature(b: &mut Bencher) {
    // Benchmark implementation
}

// Then optimize implementation
```

---

## Testing and Performance

### Testing Hierarchy

**Unit Tests** (Fast, Isolated):
```bash
# Test individual crate functionality
cargo test --package centotype-core
cargo test --package centotype-content
cargo test --package centotype-engine
```

**Integration Tests** (Cross-Crate):
```bash
# Test crate interactions
cargo test --test integration_tests
cargo test --workspace --test "*"
```

**Performance Tests** (Regression Prevention):
```bash
# Input latency validation
cargo bench --bench input_latency_benchmark

# Content system performance
cargo bench --bench content_performance_benchmark

# Memory usage validation
cargo test --package centotype-engine --test memory_validation
```

### Performance Targets

All development must maintain these targets:

| Component | Metric | Target | Current | Status |
|-----------|--------|--------|---------|--------|
| Input Processing | P99 Latency | <25ms | 22ms | ✅ |
| Content Loading | P95 Time | <25ms | <25ms | ✅ |
| Memory Usage | Peak RSS | <50MB | 46MB | ✅ |
| Cache Performance | Hit Rate | >90% | 94% | ✅ |
| Startup Time | P95 Duration | <200ms | 180ms | ✅ |

### Performance Development Workflow

```bash
# Before implementing new feature
cargo bench --bench input_latency_benchmark > before.txt

# Implement feature

# After implementation
cargo bench --bench input_latency_benchmark > after.txt

# Compare results (should not regress)
diff before.txt after.txt
```

---

## Code Quality Standards

### Rust Code Guidelines

**Safety and Reliability**:
```rust
// Prefer Result<T, E> over panic!() or unwrap()
fn safe_operation() -> Result<String, CentotypeError> {
    // Implementation that can fail gracefully
}

// Use RAII for resource management
struct ResourceManager {
    resource: Resource,
}

impl Drop for ResourceManager {
    fn drop(&mut self) {
        // Cleanup resources
    }
}
```

**Performance Patterns**:
```rust
// Use Arc<T> for shared ownership across crates
let shared_core = Arc::new(CentotypeCore::new());

// Clone Arc, not the underlying data
let core_ref = Arc::clone(&shared_core);

// Prefer borrowing over cloning when possible
fn process_data(data: &TextContent) -> Result<Metrics> {
    // Work with borrowed data
}
```

**Error Handling**:
```rust
// Use consistent error types
#[derive(Debug, thiserror::Error)]
pub enum CentotypeError {
    #[error("Performance target exceeded: {metric}")]
    PerformanceViolation { metric: String },

    #[error("Content generation failed: {reason}")]
    ContentGeneration { reason: String },
}
```

### Documentation Standards

**Public APIs**:
```rust
/// Processes user input with performance monitoring
///
/// # Arguments
/// * `input` - Raw keyboard input event
/// * `mode` - Current training mode for validation
///
/// # Returns
/// * `Ok(ProcessedInput)` - Validated and sanitized input
/// * `Err(CentotypeError)` - Input validation or security failure
///
/// # Performance
/// This function targets <25ms P99 latency and includes performance monitoring.
pub fn process_input(
    &mut self,
    input: KeyEvent,
    mode: TrainingMode
) -> Result<ProcessedInput> {
    // Implementation
}
```

### Code Review Checklist

- [ ] No panics or unwraps in production code
- [ ] Error handling follows CentotypeError patterns
- [ ] Performance targets maintained (run benchmarks)
- [ ] Tests added for new functionality
- [ ] Documentation updated for public APIs
- [ ] Cross-platform compatibility verified
- [ ] Memory usage patterns reviewed

---

## Inter-Crate Development

### Dependency Management

**Current Dependencies** (Working):
```
centotype-bin → all crates
core → (no dependencies, shared types)
engine → core, platform
content → core
cli → core
analytics → core
persistence → core
platform → (no dependencies)
```

**Communication Patterns**:
```rust
// Shared state via Arc
let core = Arc::new(CentotypeCore::new());
let engine = CentotypeEngine::new(Arc::clone(&core), platform).await?;

// Event passing via channels
let (tx, rx) = tokio::sync::mpsc::channel(100);

// Error propagation via Result
fn cross_crate_operation() -> Result<Output, CentotypeError> {
    let content = content_manager.generate(level)?;
    let metrics = analytics.process(content)?;
    Ok(metrics)
}
```

### Adding New Inter-Crate Features

1. **Define in Core Types**: Add shared types to `core/src/types.rs`
2. **Implement in Specific Crate**: Add functionality to relevant crate
3. **Test Integration**: Add tests in `tests/` directory
4. **Update Documentation**: Document new interfaces

### Performance Optimization Across Crates

```rust
// Use efficient data sharing
pub struct SharedContext {
    content: Arc<ContentManager>,
    analytics: Arc<AnalyticsManager>,
    // Avoid cloning large data structures
}

// Minimize allocations in hot paths
impl InputProcessor {
    fn process_hot_path(&self, input: &KeyEvent) -> ProcessedInput {
        // Reuse buffers, avoid allocations
        self.buffer.clear();
        // Process without allocating
    }
}
```

---

## TUI Implementation Guide

### Current TUI State

**What's Working**:
- Terminal initialization and cleanup
- Basic ratatui setup and frame management
- Input event capturing with crossterm
- Performance monitoring infrastructure

**What Needs Implementation**:
- Real typing session interface
- Live feedback and error highlighting
- Progress display and metrics
- Interactive help overlay

### TUI Architecture Pattern

```rust
// Main TUI structure
pub struct TypingSession {
    terminal: Terminal<CrosstermBackend<Stdout>>,
    state: SessionState,
    render: Render,
    input_processor: InputProcessor,
}

impl TypingSession {
    pub async fn run(
        &mut self,
        content: TextContent
    ) -> Result<SessionResult> {
        loop {
            // Render current state
            self.render.draw(&mut self.terminal, &self.state)?;

            // Process input events
            if let Some(event) = self.input_processor.poll().await? {
                match event {
                    InputEvent::Character(c) => {
                        self.state.process_character(c)?;
                    }
                    InputEvent::Control(ctrl) => {
                        if matches!(ctrl, ControlEvent::Quit) {
                            break;
                        }
                    }
                }
            }

            // Check completion
            if self.state.is_complete() {
                break;
            }
        }

        Ok(self.state.into_result())
    }
}
```

### Implementing Real Typing Sessions

**Step 1: Replace Placeholder Render Methods**:
```rust
// In engine/src/render.rs
impl Render {
    fn render_typing_pane(&mut self, frame: &mut Frame, area: Rect) -> Result<()> {
        let state = &self.render_state;

        // Show target text
        let target_paragraph = Paragraph::new(state.target_text.as_str())
            .style(Style::default().fg(self.colors.normal_text));

        // Show user input with highlighting
        let input_paragraph = Paragraph::new(state.user_input.as_str())
            .style(Style::default().fg(self.colors.input_text));

        // Render with proper layout
        frame.render_widget(target_paragraph, target_area);
        frame.render_widget(input_paragraph, input_area);

        Ok(())
    }
}
```

**Step 2: Implement Real Session Logic**:
```rust
// In cli/src/lib.rs
impl CliManager {
    pub fn run(&self, cli: Cli) -> Result<()> {
        match cli.command {
            Commands::Play { level } => {
                // Instead of printing, start real session
                let session = TypingSession::new(level)?;
                let result = session.run().await?;
                self.display_results(result)?;
            }
            // Other commands...
        }
        Ok(())
    }
}
```

### TUI Component Guidelines

**Layout Management**:
```rust
use ratatui::{
    layout::{Constraint, Direction, Layout},
    widgets::{Block, Borders, Paragraph},
};

fn create_layout(area: Rect) -> (Rect, Rect, Rect) {
    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(3),      // Header
            Constraint::Min(10),        // Typing area
            Constraint::Length(3),      // Status bar
        ])
        .split(area);

    (chunks[0], chunks[1], chunks[2])
}
```

**Color Scheme (WCAG AA Compliant)**:
```rust
pub struct UiColors {
    pub correct_text: Color,    // Green: RGB(144, 238, 144)
    pub error_text: Color,      // Red: RGB(255, 182, 193)
    pub cursor: Color,          // Yellow: High contrast
    pub normal_text: Color,     // Gray: Readable
}
```

---

## Performance Development

### Performance Monitoring Integration

```rust
// Add performance monitoring to new features
use std::time::Instant;
use centotype_analytics::PerformanceMonitor;

impl NewFeature {
    pub fn performance_critical_operation(&mut self) -> Result<Output> {
        let start = Instant::now();

        // Your implementation
        let result = self.do_work()?;

        // Record performance metrics
        let duration = start.elapsed();
        self.performance_monitor.record_latency("operation_name", duration);

        // Validate performance target
        if duration > Duration::from_millis(25) {
            warn!("Performance target exceeded: {}ms", duration.as_millis());
        }

        Ok(result)
    }
}
```

### Benchmark Development

```rust
// In benches/feature_benchmark.rs
use criterion::{black_box, criterion_group, criterion_main, Criterion};

fn bench_new_feature(c: &mut Criterion) {
    let mut feature = NewFeature::new();

    c.bench_function("new_feature_operation", |b| {
        b.iter(|| {
            black_box(feature.operation(black_box(input_data)))
        })
    });
}

criterion_group!(benches, bench_new_feature);
criterion_main!(benches);
```

### Memory Usage Optimization

```rust
// Use memory-efficient patterns
pub struct MemoryEfficientProcessor {
    // Reuse buffers instead of allocating
    buffer: Vec<char>,
    // Use Box<str> for immutable strings
    cached_content: Box<str>,
}

impl MemoryEfficientProcessor {
    pub fn process(&mut self, input: &str) -> Result<ProcessedOutput> {
        // Clear and reuse buffer
        self.buffer.clear();
        self.buffer.extend(input.chars());

        // Process without additional allocations
        self.process_buffer()
    }
}
```

---

## Contributing Guidelines

### Getting Started with Contributions

1. **Choose Your Area**:
   - **TUI Implementation**: High impact, needs crossterm/ratatui experience
   - **Performance Optimization**: Requires benchmarking and profiling skills
   - **Safety Improvements**: Needs experience with error handling patterns
   - **Feature Implementation**: Good for learning the codebase

2. **Review Current State**:
   ```bash
   # Understand what's working
   cargo test --workspace
   cargo bench

   # Check for current issues
   cargo clippy -- -D warnings
   ```

3. **Start Small**:
   - Fix compilation warnings
   - Add missing tests
   - Improve documentation
   - Optimize existing code

### Contribution Workflow

```bash
# 1. Setup development environment
git clone https://github.com/rfxlamia/centotype.git
cd centotype
cargo build --workspace

# 2. Create feature branch
git checkout -b feature/tui-typing-session

# 3. Implement with tests
cargo test --package centotype-engine

# 4. Validate performance
cargo bench --bench input_latency_benchmark

# 5. Check code quality
cargo clippy -- -D warnings
cargo fmt --check

# 6. Submit PR with description
```

### High-Priority Contribution Areas

**Critical (Production Blockers)**:
1. **TUI Typing Sessions**: Replace placeholder render methods with real TUI
2. **Panic Safety**: Fix 27+ identified panic safety violations
3. **Session Integration**: Connect engine to CLI command handlers

**High Impact**:
1. **Performance Optimization**: Further reduce input latency
2. **Error Recovery**: Improve graceful failure handling
3. **Memory Optimization**: Reduce memory footprint
4. **Cross-Platform Testing**: Ensure compatibility across platforms

**Good for Learning**:
1. **Documentation**: Improve code comments and guides
2. **Test Coverage**: Add tests for edge cases
3. **Configuration System**: Implement config file handling
4. **Profile Management**: Complete persistence integration

### Code Review Process

**What Reviewers Look For**:
- Performance impact (run benchmarks)
- Error handling patterns (no panics)
- Test coverage for new code
- Documentation for public APIs
- Cross-platform compatibility
- Memory usage patterns

**PR Requirements**:
- All tests pass: `cargo test --workspace`
- No clippy warnings: `cargo clippy -- -D warnings`
- Formatted code: `cargo fmt`
- Performance validation: `cargo bench` (if applicable)
- Updated documentation

---

## Support and Resources

### Architecture Documentation

- **Implementation Status**: `/home/v/project/centotype/docs/development/IMPLEMENTATION_COMPLETE.md`
- **Performance Analysis**: `/home/v/project/centotype/docs/performance/PERFORMANCE_VALIDATION_REPORT.md`
- **Content System**: `/home/v/project/centotype/docs/design/CONTENT_SYSTEM.md`

### Development References

- **Rust Performance Book**: https://nnethercote.github.io/perf-book/
- **Ratatui Documentation**: https://ratatui.rs/
- **Crossterm Documentation**: https://docs.rs/crossterm/
- **Tokio Async Programming**: https://tokio.rs/tokio/tutorial

### Community Support

- **GitHub Issues**: Bug reports and feature requests
- **GitHub Discussions**: Development questions and design discussions
- **Code Reviews**: Submit PRs for guidance and feedback

---

## Summary

Centotype has a solid foundation architecture achieving production-grade performance targets. The next major milestone is completing the TUI implementation to provide interactive typing sessions.

**For New Contributors**:
- Start by understanding the current architecture
- Focus on the TUI implementation in `engine/src/render.rs`
- Maintain performance targets and safety standards
- Write tests for new functionality

**Architecture Strengths**:
- Modular 7-crate design enables parallel development
- Performance monitoring ensures quality targets
- Comprehensive test suite provides confidence
- Clear separation of concerns simplifies maintenance

The foundation is ready for building the interactive experience - contributions welcome! 🚀