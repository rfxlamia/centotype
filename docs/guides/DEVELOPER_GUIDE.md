# Centotype Developer Guide

> **Complete technical implementation guide for developers working on Centotype**

This guide provides comprehensive technical guidance for developers contributing to Centotype, covering development setup, coding standards, testing strategies, and advanced implementation details.

## Table of Contents

1. [Development Environment Setup](#development-environment-setup)
2. [Project Structure](#project-structure)
3. [Development Workflow](#development-workflow)
4. [Testing Strategies](#testing-strategies)
5. [Performance Development](#performance-development)
6. [Inter-Crate Development](#inter-crate-development)
7. [API Design Guidelines](#api-design-guidelines)
8. [Security Implementation](#security-implementation)
9. [Error Handling Patterns](#error-handling-patterns)
10. [Contributing Guidelines](#contributing-guidelines)

---

## Development Environment Setup

### Prerequisites

```bash
# Required tools and versions
rustc 1.75.0+          # Rust compiler
cargo 1.75.0+          # Rust package manager
git 2.30+              # Version control
docker 20.10+          # For containerized testing (optional)
```

### Initial Setup

```bash
# Clone and setup the repository
git clone https://github.com/rfxlamia/centotype.git
cd centotype

# Install development dependencies
cargo install cargo-watch cargo-audit cargo-tarpaulin
cargo install criterion

# Setup pre-commit hooks
./scripts/setup-dev-environment.sh

# Verify installation
cargo check --all-targets
cargo test --workspace
cargo clippy --all-targets -- -D warnings
```

### Development Tools Configuration

#### VS Code Setup

```json
// .vscode/settings.json
{
    "rust-analyzer.cargo.allFeatures": true,
    "rust-analyzer.checkOnSave.command": "clippy",
    "rust-analyzer.checkOnSave.allTargets": false,
    "rust-analyzer.cargo.loadOutDirsFromCheck": true,
    "files.watcherExclude": {
        "**/target/**": true
    },
    "editor.formatOnSave": true,
    "rust-analyzer.inlayHints.enable": true
}
```

#### IntelliJ IDEA Setup

```xml
<!-- IntelliJ Rust plugin configuration -->
<application>
  <component name="RustProjectSettings">
    <option name="autoUpdateEnabled" value="true" />
    <option name="useOffline" value="false" />
    <option name="version" value="1.75.0" />
  </component>
</application>
```

### Environment Variables

```bash
# Development environment setup
export RUST_LOG=debug
export RUST_BACKTRACE=1
export CENTOTYPE_ENV=development
export CENTOTYPE_CONTENT_CACHE_SIZE=50  # Reduced for development
export CENTOTYPE_ENABLE_METRICS=true
```

---

## Project Structure

### Workspace Organization

```
centotype/
├── Cargo.toml                 # Workspace configuration
├── Cargo.lock                 # Dependency lock file
├── README.md                  # Project documentation
├── LICENSE.md                 # License information
├── docs/                      # Comprehensive documentation
├── scripts/                   # Development and build scripts
├── tests/                     # Integration tests
├── benches/                   # Performance benchmarks
├── examples/                  # Usage examples
├── .github/                   # GitHub workflows and templates
│
├── core/                      # Core business logic crate
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── error.rs           # Error types and handling
│   │   ├── level.rs           # Level management
│   │   ├── scoring.rs         # Scoring algorithms
│   │   ├── session.rs         # Session state management
│   │   └── types.rs           # Core type definitions
│   └── tests/
│
├── engine/                    # Real-time processing engine
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── event_loop.rs      # Main event processing
│   │   ├── input.rs           # Input handling
│   │   ├── render.rs          # Display rendering
│   │   └── performance.rs     # Performance monitoring
│   └── tests/
│
├── content/                   # Content generation and caching
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── cache.rs           # LRU caching implementation
│   │   ├── corpus.rs          # Text corpus management
│   │   ├── difficulty.rs      # Difficulty analysis
│   │   ├── generator.rs       # Content generation engine
│   │   └── validation.rs      # Security validation
│   └── tests/
│
├── analytics/                 # Performance analysis
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── metrics.rs         # Metrics collection
│   │   ├── analysis.rs        # Statistical analysis
│   │   └── reporting.rs       # Report generation
│   └── tests/
│
├── cli/                       # Command-line interface
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── commands.rs        # Command definitions
│   │   ├── navigation.rs      # Interactive navigation
│   │   └── ui.rs              # User interface components
│   └── tests/
│
├── persistence/               # Data storage and configuration
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── config.rs          # Configuration management
│   │   ├── profile.rs         # User profile storage
│   │   └── storage.rs         # File system operations
│   └── tests/
│
├── platform/                  # OS and terminal abstraction
│   ├── Cargo.toml
│   ├── src/
│   │   ├── lib.rs
│   │   ├── detection.rs       # Platform capability detection
│   │   ├── input.rs           # Platform-specific input handling
│   │   ├── performance.rs     # Performance monitoring
│   │   └── terminal.rs        # Terminal management
│   └── tests/
│
└── centotype-bin/             # Main binary crate
    ├── Cargo.toml
    ├── src/
    │   └── main.rs
    └── tests/
```

### Code Organization Principles

1. **Separation of Concerns**: Each crate has a single, well-defined responsibility
2. **Dependency Direction**: Dependencies flow upward (platform → core → engine → cli)
3. **Interface Segregation**: Clean, minimal interfaces between crates
4. **Performance Isolation**: Performance-critical code is clearly separated
5. **Testing Strategy**: Comprehensive unit, integration, and performance tests

---

## Development Workflow

### Git Workflow

```bash
# Feature development workflow
git checkout main
git pull origin main
git checkout -b feature/your-feature-name

# Make changes and commit
git add .
git commit -m "feat: add new feature description"

# Push and create pull request
git push origin feature/your-feature-name
# Create PR via GitHub UI
```

### Commit Message Convention

```
<type>(<scope>): <description>

[optional body]

[optional footer]
```

**Types**: `feat`, `fix`, `docs`, `style`, `refactor`, `perf`, `test`, `chore`

**Examples**:
```
feat(content): add mathematical difficulty progression
fix(engine): resolve input latency in event loop
perf(cache): optimize LRU eviction strategy
docs(api): add comprehensive API documentation
```

### Development Commands

```bash
# Development workflow
cargo check --all-targets              # Fast compilation check
cargo test --workspace                 # Run all tests
cargo clippy --all-targets -- -D warnings  # Lint code
cargo fmt                              # Format code
cargo audit                            # Security audit

# Performance testing
cargo bench                            # Run benchmarks
cargo test --release performance_tests # Performance regression tests

# Documentation
cargo doc --open                       # Generate and open docs
mdbook serve docs/                     # Serve documentation locally

# Continuous development
cargo watch -x "check --all-targets"   # Auto-recompile on changes
cargo watch -x "test"                  # Auto-test on changes
```

### Code Quality Gates

Before submitting a pull request, ensure all quality gates pass:

```bash
#!/bin/bash
# scripts/quality-gate.sh

set -e

echo "Running quality gate checks..."

# 1. Compilation check
echo "Checking compilation..."
cargo check --all-targets

# 2. Test suite
echo "Running test suite..."
cargo test --workspace

# 3. Code formatting
echo "Checking code formatting..."
cargo fmt -- --check

# 4. Linting
echo "Running clippy lints..."
cargo clippy --all-targets -- -D warnings

# 5. Security audit
echo "Running security audit..."
cargo audit

# 6. Performance regression tests
echo "Running performance tests..."
cargo test --release performance_tests

# 7. Documentation tests
echo "Testing documentation..."
cargo test --doc

echo "All quality gates passed! ✅"
```

---

## Testing Strategies

### Test Organization

```rust
// Test module organization pattern
#[cfg(test)]
mod tests {
    use super::*;

    // Unit tests for individual functions
    mod unit_tests {
        use super::*;

        #[test]
        fn test_specific_function() {
            // Test implementation
        }
    }

    // Integration tests for module interactions
    mod integration_tests {
        use super::*;

        #[tokio::test]
        async fn test_module_integration() {
            // Integration test implementation
        }
    }

    // Performance tests
    mod performance_tests {
        use super::*;
        use std::time::Instant;

        #[test]
        fn test_performance_target() {
            // Performance test implementation
        }
    }
}
```

### Unit Testing Patterns

#### Content System Testing

```rust
#[cfg(test)]
mod content_tests {
    use super::*;
    use tokio_test;

    #[tokio::test]
    async fn test_deterministic_content_generation() {
        let generator = CentotypeContentGenerator::new(
            Arc::new(ContentValidator::new().unwrap())
        );

        let level = LevelId::new(5).unwrap();
        let seed = 12345;

        // Generate content multiple times with same seed
        let content1 = generator.generate_level_content(level, seed).unwrap();
        let content2 = generator.generate_level_content(level, seed).unwrap();

        assert_eq!(content1, content2, "Content generation must be deterministic");
        assert!(!content1.is_empty(), "Generated content must not be empty");

        // Validate content meets level requirements
        let difficulty_params = DifficultyParams::calculate(level);
        let actual_length = content1.len();
        let expected_length = difficulty_params.content_length;

        assert!(
            (actual_length as f64 - expected_length as f64).abs() / expected_length as f64 < 0.1,
            "Content length {} should be within 10% of expected {}",
            actual_length, expected_length
        );
    }

    #[tokio::test]
    async fn test_cache_performance() {
        let cache = ContentCache::new(CacheConfig::default()).unwrap();
        let level = LevelId::new(1).unwrap();
        let seed = 54321;

        // First access (cache miss)
        let start = Instant::now();
        let content1 = cache.get_content(level, seed).await.unwrap();
        let miss_duration = start.elapsed();

        // Second access (cache hit)
        let start = Instant::now();
        let content2 = cache.get_content(level, seed).await.unwrap();
        let hit_duration = start.elapsed();

        assert_eq!(content1, content2, "Cache should return same content");
        assert!(hit_duration < miss_duration, "Cache hit should be faster than miss");
        assert!(hit_duration.as_millis() < 5, "Cache hit should be <5ms");

        let metrics = cache.get_metrics();
        assert!(metrics.hit_count > 0, "Should have recorded cache hit");
    }
}
```

#### Engine Testing

```rust
#[cfg(test)]
mod engine_tests {
    use super::*;

    #[tokio::test]
    async fn test_input_processing_latency() {
        let mut engine = CentotypeEngine::new().await.unwrap();
        let mut latencies = Vec::new();

        // Process 100 keystrokes and measure latency
        for i in 0..100 {
            let test_char = ((i % 26) as u8 + b'a') as char;
            let start = Instant::now();

            engine.process_keystroke(test_char).await.unwrap();

            let latency = start.elapsed();
            latencies.push(latency);
        }

        // Validate latency targets
        latencies.sort();
        let p50 = latencies[50];
        let p95 = latencies[95];
        let p99 = latencies[99];

        assert!(p99.as_millis() < 25, "P99 latency {} exceeds 25ms target", p99.as_millis());
        assert!(p95.as_millis() < 15, "P95 latency {} exceeds 15ms target", p95.as_millis());
        assert!(p50.as_millis() < 10, "P50 latency {} exceeds 10ms target", p50.as_millis());
    }

    #[tokio::test]
    async fn test_engine_state_consistency() {
        let engine = CentotypeEngine::new().await.unwrap();

        // Start a session
        let session_id = engine.start_session(
            TrainingMode::Arcade { level: LevelId::new(1).unwrap() },
            "test content".to_string()
        ).await.unwrap();

        // Process some keystrokes
        let keystrokes = "test";
        for ch in keystrokes.chars() {
            engine.process_keystroke(ch).await.unwrap();
        }

        // Verify state consistency
        let session_state = engine.get_session_state(session_id).await.unwrap();
        assert_eq!(session_state.typed_text, "test");
        assert_eq!(session_state.cursor_position, 4);
        assert!(!session_state.is_completed);

        // Complete session
        let result = engine.complete_session().await.unwrap();
        assert_eq!(result.session_id, session_id);
    }
}
```

### Integration Testing

```rust
// tests/integration_tests.rs
use centotype_content::ContentManager;
use centotype_core::CentotypeCore;
use centotype_engine::CentotypeEngine;

#[tokio::test]
async fn test_end_to_end_typing_session() {
    // Initialize all components
    let content_manager = ContentManager::new().await.unwrap();
    let core = CentotypeCore::new();
    let engine = CentotypeEngine::new().await.unwrap();

    // Get content for level 1
    let level = LevelId::new(1).unwrap();
    let content = content_manager.get_level_content(level, None).await.unwrap();

    // Start session
    let session_id = core.start_session(
        TrainingMode::Arcade { level },
        content.clone()
    ).unwrap();

    // Simulate typing the content
    let mut typed_chars = 0;
    for ch in content.chars().take(50) { // Type first 50 characters
        let metrics = core.process_keystroke(Some(ch), false).unwrap();
        typed_chars += 1;

        // Validate metrics are reasonable
        assert!(metrics.raw_wpm >= 0.0);
        assert!(metrics.accuracy >= 0.0 && metrics.accuracy <= 100.0);

        // Simulate realistic typing speed (50ms between keystrokes)
        tokio::time::sleep(Duration::from_millis(50)).await;
    }

    // Complete session
    let result = core.complete_session().unwrap();
    assert_eq!(result.session_id, session_id);
    assert!(result.metrics.effective_wpm > 0.0);
    assert!(result.metrics.accuracy > 95.0); // Should be high with perfect typing
}

#[tokio::test]
async fn test_cross_crate_performance_targets() {
    let content_manager = ContentManager::new().await.unwrap();
    let engine = CentotypeEngine::new().await.unwrap();

    // Test content loading performance
    let content_start = Instant::now();
    let content = content_manager.get_level_content(LevelId::new(1).unwrap(), None).await.unwrap();
    let content_duration = content_start.elapsed();

    assert!(content_duration.as_millis() < 50, "Content loading too slow: {}ms", content_duration.as_millis());

    // Test engine processing performance
    let engine_start = Instant::now();
    engine.process_keystroke('a').await.unwrap();
    let engine_duration = engine_start.elapsed();

    assert!(engine_duration.as_millis() < 25, "Engine processing too slow: {}ms", engine_duration.as_millis());

    // Test memory usage
    let memory_usage = get_current_memory_usage();
    assert!(memory_usage < 50 * 1024 * 1024, "Memory usage {} exceeds 50MB", memory_usage);
}
```

### Property-Based Testing

```rust
use proptest::prelude::*;

proptest! {
    #[test]
    fn test_difficulty_progression_properties(
        level1 in 1u8..100,
        level2 in 1u8..100
    ) {
        prop_assume!(level1 < level2);

        let params1 = DifficultyParams::calculate(LevelId::new(level1).unwrap());
        let params2 = DifficultyParams::calculate(LevelId::new(level2).unwrap());

        // Difficulty should increase with level
        prop_assert!(params2.symbol_ratio >= params1.symbol_ratio);
        prop_assert!(params2.number_ratio >= params1.number_ratio);
        prop_assert!(params2.tech_ratio >= params1.tech_ratio);
        prop_assert!(params2.content_length >= params1.content_length);
    }

    #[test]
    fn test_content_generation_bounds(
        level in 1u8..=100,
        seed in any::<u64>()
    ) {
        let generator = CentotypeContentGenerator::new(
            Arc::new(ContentValidator::new().unwrap())
        );

        let level_id = LevelId::new(level).unwrap();
        let content = generator.generate_level_content(level_id, seed).unwrap();

        let params = DifficultyParams::calculate(level_id);

        // Content length should be within reasonable bounds
        prop_assert!(content.len() >= params.content_length / 2);
        prop_assert!(content.len() <= params.content_length * 2);

        // Content should not be empty
        prop_assert!(!content.is_empty());

        // Content should contain printable characters
        prop_assert!(content.chars().all(|c| c.is_ascii_graphic() || c.is_whitespace()));
    }
}
```

---

## Performance Development

### Performance-First Development

When developing performance-critical code:

1. **Measure First**: Always establish baseline performance before optimizing
2. **Profile**: Use profiling tools to identify actual bottlenecks
3. **Test**: Include performance tests with every change
4. **Document**: Document performance characteristics and assumptions

### Performance Testing Framework

```rust
// Performance test helper macros
macro_rules! benchmark_function {
    ($name:ident, $func:expr, $iterations:expr, $target_ms:expr) => {
        #[test]
        fn $name() {
            let mut durations = Vec::new();

            for _ in 0..$iterations {
                let start = Instant::now();
                $func();
                durations.push(start.elapsed());
            }

            durations.sort();
            let p50 = durations[$iterations / 2];
            let p95 = durations[($iterations * 95) / 100];
            let p99 = durations[($iterations * 99) / 100];

            println!("Performance results for {}: P50: {}ms, P95: {}ms, P99: {}ms",
                    stringify!($name), p50.as_millis(), p95.as_millis(), p99.as_millis());

            assert!(p99.as_millis() <= $target_ms,
                   "P99 latency {}ms exceeds target {}ms",
                   p99.as_millis(), $target_ms);
        }
    };
}

// Usage example
benchmark_function!(
    test_content_cache_performance,
    || {
        let cache = ContentCache::new(CacheConfig::default()).unwrap();
        let _content = cache.get_cached_content(LevelId::new(1).unwrap(), 0);
    },
    1000,
    5
);
```

### Memory Profiling

```rust
#[cfg(test)]
mod memory_tests {
    use super::*;

    #[test]
    fn test_memory_usage_bounds() {
        let initial_memory = get_process_memory_usage();

        // Create content manager and perform operations
        let rt = tokio::runtime::Runtime::new().unwrap();
        rt.block_on(async {
            let content_manager = ContentManager::new().await.unwrap();

            // Generate content for all levels to test maximum memory usage
            for level_num in 1..=100 {
                let level = LevelId::new(level_num).unwrap();
                let _content = content_manager.get_level_content(level, None).await.unwrap();
            }

            let peak_memory = get_process_memory_usage();
            let memory_delta = peak_memory - initial_memory;

            assert!(memory_delta < 50 * 1024 * 1024,
                   "Memory usage {} exceeds 50MB limit", memory_delta);
        });
    }

    fn get_process_memory_usage() -> u64 {
        #[cfg(target_os = "linux")]
        {
            let status = std::fs::read_to_string("/proc/self/status").unwrap();
            for line in status.lines() {
                if line.starts_with("VmRSS:") {
                    let kb: u64 = line.split_whitespace().nth(1).unwrap().parse().unwrap();
                    return kb * 1024; // Convert to bytes
                }
            }
        }

        // Fallback for other platforms
        0
    }
}
```

### Continuous Performance Monitoring

```rust
// Include in CI/CD pipeline
pub struct PerformanceCI {
    baseline_metrics: BaselineMetrics,
    current_metrics: CurrentMetrics,
}

impl PerformanceCI {
    pub fn validate_performance_regression(&self) -> Result<(), PerformanceRegressionError> {
        // Input latency regression check
        if self.current_metrics.input_latency_p99 >
           self.baseline_metrics.input_latency_p99 * 1.1 {
            return Err(PerformanceRegressionError::InputLatencyRegression {
                current: self.current_metrics.input_latency_p99,
                baseline: self.baseline_metrics.input_latency_p99,
                regression_percent: (
                    (self.current_metrics.input_latency_p99.as_millis() as f64 -
                     self.baseline_metrics.input_latency_p99.as_millis() as f64) /
                    self.baseline_metrics.input_latency_p99.as_millis() as f64
                ) * 100.0,
            });
        }

        // Memory usage regression check
        if self.current_metrics.memory_usage_mb >
           self.baseline_metrics.memory_usage_mb * 1.15 {
            return Err(PerformanceRegressionError::MemoryUsageRegression {
                current_mb: self.current_metrics.memory_usage_mb,
                baseline_mb: self.baseline_metrics.memory_usage_mb,
            });
        }

        // Cache performance regression check
        if self.current_metrics.cache_hit_rate <
           self.baseline_metrics.cache_hit_rate * 0.95 {
            return Err(PerformanceRegressionError::CachePerformanceRegression {
                current_rate: self.current_metrics.cache_hit_rate,
                baseline_rate: self.baseline_metrics.cache_hit_rate,
            });
        }

        Ok(())
    }
}
```

---

## Inter-Crate Development

### Crate Dependencies and Interfaces

```rust
// Example: Adding a new feature that spans multiple crates

// 1. Core crate: Define the domain types
// core/src/types.rs
#[derive(Debug, Clone)]
pub struct TypingStatistics {
    pub session_id: uuid::Uuid,
    pub keystroke_intervals: Vec<Duration>,
    pub error_patterns: Vec<ErrorPattern>,
    pub skill_progression: SkillProgression,
}

// 2. Analytics crate: Implement analysis logic
// analytics/src/statistics.rs
impl StatisticsAnalyzer {
    pub fn analyze_session(&self, session_result: &SessionResult) -> TypingStatistics {
        // Implementation that uses core types
        TypingStatistics {
            session_id: session_result.session_id,
            keystroke_intervals: self.calculate_intervals(&session_result.keystrokes),
            error_patterns: self.analyze_errors(&session_result.errors),
            skill_progression: self.assess_skill_progression(session_result),
        }
    }
}

// 3. Engine crate: Integrate with real-time processing
// engine/src/session.rs
impl SessionEngine {
    pub async fn complete_session_with_analysis(&mut self) -> Result<SessionAnalysis> {
        // Get session result from core
        let session_result = self.core.complete_session()?;

        // Perform analysis
        let statistics = self.analytics.analyze_session(&session_result)?;

        // Store results
        self.persistence.save_session_analysis(&statistics).await?;

        Ok(SessionAnalysis {
            result: session_result,
            statistics,
        })
    }
}
```

### Cross-Crate Communication Patterns

#### Synchronous Communication (Performance Critical)

```rust
// For performance-critical paths, use direct function calls
pub trait FastContentProvider {
    fn get_cached_content(&self, level_id: LevelId) -> Option<&str>;
}

impl FastContentProvider for ContentManager {
    #[inline]
    fn get_cached_content(&self, level_id: LevelId) -> Option<&str> {
        // Direct cache access with no async overhead
        self.hot_cache.get(&level_id).map(|s| s.as_str())
    }
}
```

#### Asynchronous Communication (Non-Critical Path)

```rust
// For non-critical operations, use message passing
pub struct BackgroundTaskCoordinator {
    content_tasks: mpsc::Sender<ContentTask>,
    analytics_tasks: mpsc::Sender<AnalyticsTask>,
}

impl BackgroundTaskCoordinator {
    pub async fn schedule_content_preload(&self, level_id: LevelId) -> Result<()> {
        self.content_tasks.send(ContentTask::Preload(level_id)).await?;
        Ok(())
    }

    pub async fn schedule_analytics_update(&self, session_data: SessionData) -> Result<()> {
        self.analytics_tasks.send(AnalyticsTask::UpdateMetrics(session_data)).await?;
        Ok(())
    }
}
```

### Dependency Injection Patterns

```rust
// Use dependency injection for testability and modularity
pub struct CentotypeApp {
    content_manager: Arc<dyn ContentProvider>,
    scoring_engine: Arc<dyn ScoringProvider>,
    analytics_engine: Arc<dyn AnalyticsProvider>,
    persistence_manager: Arc<dyn PersistenceProvider>,
}

impl CentotypeApp {
    pub fn new(
        content_manager: Arc<dyn ContentProvider>,
        scoring_engine: Arc<dyn ScoringProvider>,
        analytics_engine: Arc<dyn AnalyticsProvider>,
        persistence_manager: Arc<dyn PersistenceProvider>,
    ) -> Self {
        Self {
            content_manager,
            scoring_engine,
            analytics_engine,
            persistence_manager,
        }
    }

    // Production constructor
    pub async fn production() -> Result<Self> {
        let content_manager = Arc::new(ContentManager::new().await?);
        let scoring_engine = Arc::new(ScoringEngine::new());
        let analytics_engine = Arc::new(AnalyticsEngine::new());
        let persistence_manager = Arc::new(PersistenceManager::new().await?);

        Ok(Self::new(
            content_manager,
            scoring_engine,
            analytics_engine,
            persistence_manager,
        ))
    }

    // Test constructor with mocks
    #[cfg(test)]
    pub fn test() -> Self {
        let content_manager = Arc::new(MockContentProvider::new());
        let scoring_engine = Arc::new(MockScoringProvider::new());
        let analytics_engine = Arc::new(MockAnalyticsProvider::new());
        let persistence_manager = Arc::new(MockPersistenceProvider::new());

        Self::new(
            content_manager,
            scoring_engine,
            analytics_engine,
            persistence_manager,
        )
    }
}
```

---

## API Design Guidelines

### Interface Design Principles

1. **Consistency**: Similar operations should have similar signatures
2. **Clarity**: Function names should clearly indicate their purpose
3. **Safety**: Use Rust's type system to prevent misuse
4. **Performance**: Design APIs to enable optimization
5. **Testability**: APIs should be easy to test and mock

### Error Handling API Design

```rust
// Consistent error handling across crates
#[derive(Debug, thiserror::Error)]
pub enum CentotypeError {
    #[error("Content generation failed: {message}")]
    ContentGeneration { message: String },

    #[error("Cache operation failed: {operation}")]
    CacheOperation { operation: String },

    #[error("Performance target violated: {metric} = {actual}, target = {target}")]
    PerformanceTarget {
        metric: String,
        actual: String,
        target: String,
    },

    #[error("IO operation failed")]
    Io(#[from] std::io::Error),

    #[error("Serialization failed")]
    Serialization(#[from] serde_json::Error),
}

// Result type alias for consistency
pub type Result<T> = std::result::Result<T, CentotypeError>;

// Context extension for better error messages
pub trait ResultExt<T> {
    fn with_context<F>(self, f: F) -> Result<T>
    where
        F: FnOnce() -> String;
}

impl<T, E> ResultExt<T> for std::result::Result<T, E>
where
    E: Into<CentotypeError>,
{
    fn with_context<F>(self, f: F) -> Result<T>
    where
        F: FnOnce() -> String,
    {
        self.map_err(|e| {
            let base_error = e.into();
            match base_error {
                CentotypeError::ContentGeneration { message } => {
                    CentotypeError::ContentGeneration {
                        message: format!("{}: {}", f(), message),
                    }
                },
                other => other,
            }
        })
    }
}
```

### Async API Design

```rust
// Consistent async patterns
pub trait AsyncContentProvider {
    async fn get_content(&self, level_id: LevelId) -> Result<String>;

    // Provide both async and sync variants where appropriate
    fn get_cached_content(&self, level_id: LevelId) -> Option<String>;

    // Use streams for ongoing data
    fn content_updates(&self) -> impl Stream<Item = ContentUpdate>;

    // Provide cancellation support for long operations
    async fn generate_content_with_cancellation(
        &self,
        level_id: LevelId,
        cancellation_token: CancellationToken,
    ) -> Result<String>;
}

// Configuration API pattern
pub trait Configurable {
    type Config: Clone + Send + Sync;

    async fn get_config(&self) -> Self::Config;
    async fn update_config(&self, config: Self::Config) -> Result<()>;

    // Validate configuration before applying
    fn validate_config(config: &Self::Config) -> Result<()>;
}
```

### Builder Pattern for Complex Configuration

```rust
// Builder pattern for complex initialization
pub struct ContentManagerBuilder {
    cache_config: Option<CacheConfig>,
    difficulty_config: Option<DifficultyConfig>,
    validator: Option<Arc<ContentValidator>>,
    corpus_data: Option<CorpusData>,
}

impl ContentManagerBuilder {
    pub fn new() -> Self {
        Self {
            cache_config: None,
            difficulty_config: None,
            validator: None,
            corpus_data: None,
        }
    }

    pub fn with_cache_config(mut self, config: CacheConfig) -> Self {
        self.cache_config = Some(config);
        self
    }

    pub fn with_difficulty_config(mut self, config: DifficultyConfig) -> Self {
        self.difficulty_config = Some(config);
        self
    }

    pub fn with_validator(mut self, validator: Arc<ContentValidator>) -> Self {
        self.validator = Some(validator);
        self
    }

    pub async fn build(self) -> Result<ContentManager> {
        let cache_config = self.cache_config.unwrap_or_default();
        let difficulty_config = self.difficulty_config.unwrap_or_default();
        let validator = self.validator.unwrap_or_else(|| {
            Arc::new(ContentValidator::new().expect("Failed to create default validator"))
        });

        ContentManager::with_components(cache_config, difficulty_config, validator).await
    }
}
```

---

## Security Implementation

### Input Validation

```rust
// Comprehensive input validation framework
pub struct InputValidator {
    character_whitelist: HashSet<char>,
    max_input_rate: RateLimiter,
    sequence_detector: MaliciousSequenceDetector,
}

impl InputValidator {
    pub fn validate_keystroke(&mut self, input: char, timestamp: Instant) -> Result<ValidatedInput> {
        // Rate limiting
        if !self.max_input_rate.check_rate(timestamp) {
            return Err(CentotypeError::Security("Input rate limit exceeded".to_string()));
        }

        // Character validation
        if !self.character_whitelist.contains(&input) {
            return Err(CentotypeError::Security(
                format!("Unauthorized character: {:?}", input)
            ));
        }

        // Sequence detection
        self.sequence_detector.add_character(input);
        if let Some(malicious_sequence) = self.sequence_detector.detect_malicious_sequence() {
            return Err(CentotypeError::Security(
                format!("Malicious sequence detected: {}", malicious_sequence)
            ));
        }

        Ok(ValidatedInput {
            character: input,
            timestamp,
            validation_level: ValidationLevel::Passed,
        })
    }
}

// Content security validation
pub struct ContentSecurityValidator {
    escape_detector: EscapeSequenceDetector,
    content_scanner: ContentScanner,
    length_validator: LengthValidator,
}

impl ContentSecurityValidator {
    pub fn validate_content(&self, content: &str) -> SecurityValidationResult {
        let mut issues = Vec::new();

        // Check for escape sequences
        if let Some(escape_issues) = self.escape_detector.scan(content) {
            issues.extend(escape_issues.into_iter().map(SecurityIssue::EscapeSequence));
        }

        // Check content length
        if content.len() > self.length_validator.max_length {
            issues.push(SecurityIssue::ContentTooLong {
                actual: content.len(),
                maximum: self.length_validator.max_length,
            });
        }

        // Deep content scan
        if let Some(content_issues) = self.content_scanner.scan(content) {
            issues.extend(content_issues.into_iter().map(SecurityIssue::SuspiciousContent));
        }

        SecurityValidationResult {
            is_safe: issues.is_empty(),
            issues,
            confidence_score: self.calculate_confidence_score(&issues),
        }
    }
}
```

### Secure Data Handling

```rust
// Secure configuration management
pub struct SecureConfig {
    inner: RwLock<ConfigData>,
    encryption_key: SecretKey,
}

impl SecureConfig {
    pub fn new(encryption_key: SecretKey) -> Self {
        Self {
            inner: RwLock::new(ConfigData::default()),
            encryption_key,
        }
    }

    pub async fn load_from_file<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> Result<()> {
        let encrypted_data = tokio::fs::read(path).await?;
        let decrypted_data = self.decrypt_data(&encrypted_data)?;
        let config_data: ConfigData = serde_json::from_slice(&decrypted_data)?;

        let mut inner = self.inner.write();
        *inner = config_data;

        Ok(())
    }

    pub async fn save_to_file<P: AsRef<Path>>(
        &self,
        path: P,
    ) -> Result<()> {
        let inner = self.inner.read();
        let serialized = serde_json::to_vec(&*inner)?;
        let encrypted_data = self.encrypt_data(&serialized)?;

        // Atomic write operation
        let temp_path = path.as_ref().with_extension("tmp");
        tokio::fs::write(&temp_path, encrypted_data).await?;
        tokio::fs::rename(temp_path, path).await?;

        Ok(())
    }

    fn encrypt_data(&self, data: &[u8]) -> Result<Vec<u8>> {
        // Implementation using secure encryption
        // (placeholder - use actual encryption library)
        Ok(data.to_vec())
    }

    fn decrypt_data(&self, data: &[u8]) -> Result<Vec<u8>> {
        // Implementation using secure decryption
        // (placeholder - use actual encryption library)
        Ok(data.to_vec())
    }
}
```

---

## Error Handling Patterns

### Structured Error Handling

```rust
// Hierarchical error types
#[derive(Debug, thiserror::Error)]
pub enum CentotypeError {
    #[error("System error: {0}")]
    System(#[from] SystemError),

    #[error("Content error: {0}")]
    Content(#[from] ContentError),

    #[error("Engine error: {0}")]
    Engine(#[from] EngineError),

    #[error("Analytics error: {0}")]
    Analytics(#[from] AnalyticsError),
}

#[derive(Debug, thiserror::Error)]
pub enum ContentError {
    #[error("Generation failed: {message}")]
    GenerationFailed { message: String },

    #[error("Cache miss for level {level}")]
    CacheMiss { level: u8 },

    #[error("Validation failed: {reason}")]
    ValidationFailed { reason: String },

    #[error("Security issue: {issue}")]
    SecurityIssue { issue: String },
}

// Error context and recovery
pub struct ErrorContext {
    pub operation: String,
    pub component: String,
    pub timestamp: DateTime<Utc>,
    pub session_id: Option<uuid::Uuid>,
    pub user_id: Option<String>,
    pub system_state: SystemState,
}

pub trait ErrorRecovery {
    fn can_recover(&self, error: &CentotypeError, context: &ErrorContext) -> bool;
    async fn attempt_recovery(&self, error: CentotypeError, context: ErrorContext) -> Result<()>;
}
```

### Error Recovery Strategies

```rust
pub struct ContentErrorRecovery;

impl ErrorRecovery for ContentErrorRecovery {
    fn can_recover(&self, error: &CentotypeError, context: &ErrorContext) -> bool {
        match error {
            CentotypeError::Content(ContentError::CacheMiss { .. }) => true,
            CentotypeError::Content(ContentError::GenerationFailed { .. }) => true,
            CentotypeError::Content(ContentError::SecurityIssue { .. }) => false,
            _ => false,
        }
    }

    async fn attempt_recovery(&self, error: CentotypeError, context: ErrorContext) -> Result<()> {
        match error {
            CentotypeError::Content(ContentError::CacheMiss { level }) => {
                // Attempt to generate content for the missing level
                warn!("Cache miss for level {}, attempting generation", level);

                let level_id = LevelId::new(level)?;
                let generator = ContentGenerator::new();
                let content = generator.generate_level_content(level_id, 0).await?;

                // Cache the generated content
                let cache = ContentCache::global();
                cache.insert(level_id, content).await;

                info!("Successfully recovered from cache miss for level {}", level);
                Ok(())
            },

            CentotypeError::Content(ContentError::GenerationFailed { message }) => {
                // Attempt with fallback content
                warn!("Content generation failed: {}, using fallback", message);

                let fallback_content = get_fallback_content(&context)?;

                // Use the fallback temporarily
                if let Some(session_id) = context.session_id {
                    let session_manager = SessionManager::global();
                    session_manager.update_content(session_id, fallback_content).await?;
                }

                info!("Successfully recovered using fallback content");
                Ok(())
            },

            _ => Err(CentotypeError::System(SystemError::RecoveryFailed {
                original_error: Box::new(error),
                recovery_context: context,
            })),
        }
    }
}
```

---

## Contributing Guidelines

### Code Review Process

1. **Self Review**: Review your own changes thoroughly before submitting
2. **Automated Checks**: Ensure all CI checks pass
3. **Peer Review**: Get at least one approving review from a team member
4. **Performance Review**: For performance-critical changes, get review from performance team
5. **Security Review**: For security-related changes, get review from security team

### Pull Request Template

```markdown
## Description
Brief description of changes and motivation.

## Type of Change
- [ ] Bug fix (non-breaking change that fixes an issue)
- [ ] New feature (non-breaking change that adds functionality)
- [ ] Breaking change (fix or feature that would cause existing functionality to not work as expected)
- [ ] Performance improvement
- [ ] Documentation update

## Testing
- [ ] Unit tests added/updated
- [ ] Integration tests added/updated
- [ ] Performance tests added/updated
- [ ] Manual testing completed

## Performance Impact
- [ ] No performance impact
- [ ] Performance improvement (include benchmark results)
- [ ] Potential performance impact (include analysis)

## Security Considerations
- [ ] No security implications
- [ ] Security review completed
- [ ] Input validation added/updated
- [ ] Error handling reviewed

## Checklist
- [ ] Code follows project style guidelines
- [ ] Self-review completed
- [ ] Comments added for complex logic
- [ ] Documentation updated
- [ ] All tests pass
- [ ] No compiler warnings
```

### Release Process

```bash
# Release checklist
1. Version bump in Cargo.toml files
2. Update CHANGELOG.md
3. Run full test suite
4. Performance regression testing
5. Security audit
6. Create release branch
7. Generate release notes
8. Tag release
9. Publish to crates.io
10. Update documentation
```

This Developer Guide provides comprehensive technical guidance for contributing to Centotype. It covers all aspects of development from environment setup to release processes, ensuring consistent, high-quality contributions to the project.