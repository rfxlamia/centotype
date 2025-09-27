# Centotype Architecture - Complete Implementation Foundation

This repository contains the complete architectural foundation for Centotype, a CLI-based typing trainer with 100 progressive difficulty levels built in Rust.

## 🏗️ Architecture Overview

The architecture is designed for **parallel development** with clear module boundaries and strict performance targets:

- **P99 input latency**: < 25ms
- **P95 startup time**: < 200ms
- **P95 render time**: < 33ms (30fps)
- **Memory usage**: < 50MB RSS
- **Cross-platform**: Linux, macOS, Windows (x86_64, ARM64)

## 📦 Crate Structure

```
centotype/
├── core/           # State management, scoring engine, business logic
├── engine/         # Event loop, input handling, render loop
├── content/        # Text corpus loading, dynamic generation
├── analytics/      # Performance analysis, error classification
├── cli/            # Command parsing, interactive navigation
├── persistence/    # Profile storage, configuration management
├── platform/       # OS-specific integrations, terminal detection
└── centotype-bin/  # Main binary application
```

## 🔗 Key Interface Definitions

### Core Types (core/src/types.rs)

All shared types and traits are defined in the core crate:

```rust
// Session management
pub struct SessionState { /* ... */ }
pub struct SessionResult { /* ... */ }
pub struct LiveMetrics { /* ... */ }

// Performance monitoring
pub struct PerformanceMetrics { /* ... */ }
pub trait PerformanceMeasurable { /* ... */ }

// Content generation
pub struct TextContent { /* ... */ }
pub trait ContentGenerator { /* ... */ }

// Error handling
pub enum CentotypeError { /* ... */ }
pub type Result<T> = std::result::Result<T, CentotypeError>;
```

### Critical Performance Interfaces

**Input Processing (engine → core):**
```rust
impl CentotypeCore {
    pub fn process_keystroke(
        &self,
        char_typed: Option<char>,
        is_correction: bool
    ) -> Result<LiveMetrics>;
}
```

**Content Generation (core → content):**
```rust
impl ContentManager {
    pub fn get_level_content(&self, level: LevelId) -> Result<TextContent>;
    pub fn get_drill_content(&self, category: DrillCategory, duration: u32) -> Result<TextContent>;
}
```

**Performance Monitoring (engine → platform):**
```rust
impl PerformanceMonitor {
    pub fn record_input_latency(&mut self, latency: Duration);
    pub fn get_metrics(&self) -> PerformanceMetrics;
    pub fn meets_targets(&self) -> PerformanceCheck;
}
```

## 🚀 Implementation Roadmap

### Phase 1: Foundation (Weeks 1-2)

**Core Crate Implementation:**
1. Complete session state management (`SessionManager`)
2. Implement scoring engine with real-time calculations
3. Build error classification system (Damerau-Levenshtein)
4. Create level progression logic

**Engine Crate Implementation:**
1. Set up event loop with performance monitoring
2. Implement raw mode input processing
3. Build render system with double buffering
4. Create TTY management with graceful cleanup

**Platform Crate Implementation:**
1. Complete platform detection and validation
2. Implement terminal capability analysis
3. Build performance optimization system
4. Create error recovery strategies

### Phase 2: Content & Integration (Weeks 3-4)

**Content Crate Implementation:**
1. Build static corpus management system
2. Implement dynamic content generator with seeding
3. Create difficulty analysis and validation
4. Set up content caching with LRU eviction

**Integration Testing:**
1. Core ↔ Engine integration with latency validation
2. Content ↔ Core integration with caching tests
3. Platform ↔ Engine integration across OS targets
4. End-to-end performance validation

### Phase 3: User Interface (Weeks 5-6)

**CLI Crate Implementation:**
1. Complete command parsing and validation
2. Build interactive menu systems
3. Implement session navigation and controls
4. Create statistics and progress displays

**Persistence & Analytics:**
1. Implement atomic profile storage
2. Build configuration management
3. Create performance analysis engine
4. Set up telemetry collection (opt-in)

## 🔧 Development Guidelines

### Performance Requirements

**Input Processing Hot Path:**
```rust
// Target: P99 < 25ms
#[inline(always)]
pub fn process_keystroke(&mut self, input: RawInput) -> Result<ProcessedInput> {
    let start = Instant::now();

    // Minimal processing only
    let processed = ProcessedInput::from_raw(input);

    // Defer expensive operations
    self.background_queue.send(input).ok();

    Ok(processed)
}
```

**Memory Management:**
```rust
// Target: < 50MB total RSS
pub struct MemoryBudget {
    core_state: 5,      // MB - session state, scoring
    engine_buffers: 10, // MB - input/render buffers
    content_cache: 20,  // MB - text content cache
    analytics_data: 5,  // MB - performance tracking
    platform_data: 5,  // MB - platform abstractions
    overhead: 5,        // MB - allocator overhead
}
```

**Render Loop Optimization:**
```rust
// Target: P95 < 33ms (30fps)
pub fn render_frame(&mut self, state: &RenderState) -> Result<()> {
    // Skip unnecessary renders
    if !self.needs_render(state) {
        return Ok(());
    }

    // Double buffering for smooth updates
    self.render_to_back_buffer(state)?;
    self.swap_buffers();

    Ok(())
}
```

### Error Handling Patterns

**Graceful Degradation:**
```rust
// Continue operation with reduced functionality
if let Err(e) = self.optimal_operation() {
    tracing::warn!("Optimal operation failed: {}, using fallback", e);
    self.fallback_operation()?;
}
```

**Emergency Cleanup:**
```rust
impl Drop for CentotypeEngine {
    fn drop(&mut self) {
        // Critical: Always restore terminal state
        let _ = self.tty_manager.exit_raw_mode();
    }
}
```

### Testing Infrastructure

**Performance Validation:**
```bash
# Benchmark critical paths
cargo bench --bench input_latency
cargo bench --bench render_performance
cargo bench --bench scoring_engine

# Memory leak detection
cargo test --test memory_usage -- --ignored
```

**Cross-Platform Testing:**
```bash
# CI/CD pipeline tests
cargo test --target x86_64-unknown-linux-gnu
cargo test --target x86_64-apple-darwin
cargo test --target x86_64-pc-windows-msvc
```

## 📊 Monitoring and Validation

### Performance Metrics Collection

The engine continuously monitors performance targets:

```rust
pub struct PerformanceTargets {
    pub input_latency_p99_ms: 25,
    pub render_time_p95_ms: 33,
    pub startup_time_p95_ms: 200,
    pub memory_limit_mb: 50,
}

// Real-time validation
if !performance.meets_targets() {
    engine.apply_fallback_mode();
    tracing::warn!("Performance targets not met, reducing effects");
}
```

### Error Recovery

Each crate implements specific recovery strategies:

- **Engine**: Terminal state restoration, input processor restart
- **Content**: Cache invalidation, fallback to static corpus
- **Platform**: Graceful feature degradation, compatibility mode
- **Persistence**: Atomic writes, backup file recovery

## 🚦 Ready for Implementation

### Parallel Development Streams

The architecture enables **4 parallel development streams**:

1. **Core + Engine**: Senior Rust developer
2. **Content + Analytics**: Content specialist + Rust developer
3. **CLI + Persistence**: Frontend/CLI developer
4. **Platform**: Systems/DevOps engineer

### Integration Points

Clear interfaces ensure smooth integration:

- **Shared types** in `core/src/types.rs`
- **Error handling** patterns in `ERROR_HANDLING.md`
- **Data flow** specifications in `ARCHITECTURE.md`
- **Performance contracts** enforced at compile time

### Validation Strategy

**Week 2 Go/No-Go Decision:**
- [ ] Input latency prototype meets P99 < 25ms target
- [ ] Cross-platform compatibility validated
- [ ] Memory usage under 50MB in stress tests
- [ ] Error recovery tested on all platforms

## 📈 Success Metrics

The architecture is designed to achieve:

- **Level 100 mastery criteria**: 130+ WPM, 99.5%+ accuracy, ≤3 error severity
- **User progression**: Median users advance ≥1 tier within 2 weeks
- **Stability**: <0.01% crash rate, 100% profile data consistency
- **Performance**: All targets met on reference hardware

## 🔄 Next Steps

1. **Set up development environment** with all crate dependencies
2. **Implement core session management** as the foundation
3. **Build input processing pipeline** with performance validation
4. **Create content generation system** with deterministic seeding
5. **Integrate components** with continuous performance monitoring

This architecture provides a solid foundation for building a high-performance, cross-platform typing trainer that meets all requirements and enables efficient parallel development.

---

**Files Overview:**
- `/home/v/project/centotype/Cargo.toml` - Workspace configuration
- `/home/v/project/centotype/core/src/types.rs` - Shared types and traits
- `/home/v/project/centotype/*/src/lib.rs` - Crate entry points
- `/home/v/project/centotype/ERROR_HANDLING.md` - Error patterns
- `/home/v/project/centotype/ARCHITECTURE.md` - Detailed data flow