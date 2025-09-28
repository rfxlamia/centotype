# Centotype Master Prompt System v2.0 - Week 3-4 Integration Phase

## Enhanced Coordination for Phase 1 Close-out

This document provides the master coordination framework for closing Week 3-4 gaps and achieving Phase 1 completion. The content generation system is **production-ready**; focus shifts to engine integration, performance optimization, and comprehensive testing.

**Current Status**: Sophisticated CLI skeleton with placeholder implementation → Full typing trainer with <25ms P99 latency

## Content Generation Master Prompt

You are generating typing training content for Centotype CLI, a progressive 100-level typing trainer. Create content that follows these specifications:

### Level Parameters
- **Level ID**: {level_id} (1-100)
- **Tier**: {tier} (1-10, calculated as ceil(level_id/10))
- **Tier Progress**: {tier_progress} (1-10, level_id % 10 or 10 if divisible)
- **Random Seed**: {seed} (for deterministic generation)

### Difficulty Progression Formulas

#### Symbol Density (%)
```
symbol_ratio = 5 + (tier - 1) * 2.5 + (tier_progress - 1) * 0.3
// Level 1: 5.0%, Level 100: 30.0%
```

#### Number Density (%)
```
number_ratio = 3 + (tier - 1) * 1.7 + (tier_progress - 1) * 0.2
// Level 1: 3.0%, Level 100: 20.0%
```

#### Technical Terms Density (%)
```
tech_ratio = 2 + (tier - 1) * 1.3 + (tier_progress - 1) * 0.2
// Level 1: 2.0%, Level 100: 15.0%
```

#### Content Length (characters)
```
content_length = 300 + (tier - 1) * 270 + (tier_progress - 1) * 30
// Level 1: 300 chars, Level 100: 3000 chars
```

#### Language Switching Frequency
```
switch_freq = max(200 - (tier - 1) * 15, 50)
// Level 1: 200 chars, Level 100: 50 chars
```

### Content Composition Requirements

#### Tier 1-2 (Levels 1-20): Foundation
- Basic alphabetic text with common words
- Simple punctuation (., , ! ?)
- Numbers in contexts (dates, simple math)
- Indonesian/English common vocabulary

#### Tier 3-4 (Levels 21-40): Programming Basics
- Introduction of programming symbols: {} [] ()
- Basic operators: + - * / =
- camelCase and snake_case patterns
- Simple code snippets and technical terms

#### Tier 5-6 (Levels 41-60): Intermediate Complexity
- Advanced symbols: | & ^ ~ % $ #
- Nested brackets and complex expressions
- Mixed-case scenarios with technical jargon
- Code documentation patterns

#### Tier 7-8 (Levels 61-80): Advanced Programming
- Complex symbol combinations: <> |&| ^^^ ~~~
- Bitwise operations and hex values
- Multi-language code switching
- Advanced technical terminology

#### Tier 9-10 (Levels 81-100): Expert Mastery
- Maximum symbol density with edge cases
- Unicode characters and special encodings
- Rapid language switching with no warnings
- Professional technical writing complexity

### Security Validation Requirements

#### Input Sanitization Rules
1. **No Escape Sequences**: Content must not contain terminal escape sequences (\x1b, \033)
2. **No Shell Commands**: No executable commands or shell metacharacters in inappropriate contexts
3. **No File Paths**: Avoid absolute file paths that could reveal system information
4. **Safe Unicode**: Only use printable Unicode characters in Basic Latin + common extensions
5. **Length Bounds**: Respect minimum/maximum content length parameters

#### Validation Tests (Must Pass)
```rust
// Test 1: No escape sequences
assert!(!content.contains('\x1b'));
assert!(!content.contains("\033"));

// Test 2: No shell injection patterns
assert!(!content.contains("$("));
assert!(!content.contains("`"));
assert!(!content.contains("&&"));

// Test 3: Length validation
assert!(content.len() >= min_length);
assert!(content.len() <= max_length);

// Test 4: Character composition
assert!(validate_symbol_ratio(content, target_symbol_ratio));
```

### Performance Constraints

#### Generation Requirements
- **Deterministic**: Same seed + level must produce identical content
- **Fast Generation**: <10ms generation time per level
- **Memory Efficient**: <1MB memory footprint during generation
- **Cacheable**: Content structure supports efficient LRU caching

#### Caching Integration
- Generate cache key: `content_v1_{level_id}_{seed}`
- Support cache invalidation for content updates
- Optimize for <25ms content loading (including cache lookup)

### Content Templates by Tier

#### Template Structure
```
{language_indicator}:{content_type}:{difficulty_markers}
{generated_content}
{validation_checksum}
```

#### Tier-Specific Patterns

**Tier 1-2**: Simple prose with basic punctuation
```
ID: Saya suka menulis kode yang bersih dan mudah dibaca.
EN: I love writing clean and readable code every day.
```

**Tier 5-6**: Mixed programming content
```
ID: function calculateSum(arr) { return arr.reduce((a,b) => a+b, 0); }
EN: const config = { debug: true, timeout: 5000, retries: 3 };
```

**Tier 9-10**: Expert-level complexity
```
ID: &mut HashMap<String, Vec<Option<Box<dyn Iterator<Item=u32>>>>>
EN: 0x1F3F4E40 | (mask << 8) ^ ~(flags & 0xFF) >> 2
```

### Quality Metrics

#### Measurable Success Criteria
1. **Progression Smoothness**: Each level 3-7% harder than previous
2. **Completion Rate**: >85% of generated levels meet difficulty targets
3. **Performance Compliance**: 100% of content loads within 25ms
4. **Security Pass Rate**: 100% pass security validation
5. **Deterministic Success**: Identical output for same seed+level across 100 runs

#### Content Balance Validation
```python
def validate_content_balance(content, level_id):
    actual_symbols = count_symbols(content) / len(content)
    expected_symbols = calculate_symbol_ratio(level_id)
    return abs(actual_symbols - expected_symbols) < 0.02  # ±2% tolerance
```

## Agent Coordination Specifications v2.0

### Critical Path Dependencies (Week 3-4 Close-out)

**Tier 1 (Blockers)**: rust-pro, backend-architect, test-automator
**Tier 2 (Core)**: ui-ux-designer, performance-engineer
**Tier 3 (Quality)**: debugger, code-reviewer, security-auditor
**Tier 4 (Support)**: devops-troubleshooter, dx-optimizer, docs-architect, tutorial-engineer

### For rust-pro Agent [TIER 1 - CRITICAL BLOCKER]
**Context**: Transform stub engine::run() into fully functional typing loop
**Critical Gap**: Engine loop integration prevents Phase 1 close-out

**Primary Objective**: Fill stub engine::run() method with complete typing loop implementation
- Replace `self.core.complete_session()` placeholder with actual typing loop
- Implement input capture → diff calculation → real-time scoring → render cycle
- Wire core::scoring + content + analytics + persistence integration
- Target: Build succeeds, P99 input latency ≤25ms, SessionResult fully populated

**Key Implementation Requirements**:
```rust
// BEFORE (stub):
pub async fn run(&mut self, _mode: TrainingMode, _target_text: String) -> Result<SessionResult> {
    self.core.complete_session()  // Returns empty result
}

// AFTER (functional):
pub async fn run(&mut self, mode: TrainingMode, target_text: String) -> Result<SessionResult> {
    // 1. Initialize session with content from content/ crate
    let content = self.content_loader.load_level_content(mode.level_id()).await?;
    self.core.start_session(mode, content.clone())?;

    // 2. Setup TTY raw mode and alt-screen
    self.terminal.enter_raw_mode()?;
    self.terminal.enter_alternate_screen()?;

    // 3. Main typing loop
    let result = loop {
        // Render current state (ratatui integration)
        self.render_frame(&self.core.current_state())?;

        // Capture input with <25ms latency target
        if let Some(event) = self.input_handler.poll_event(Duration::from_millis(10))? {
            match event {
                InputEvent::Key(key) => {
                    // Process keystroke through core scoring
                    let scoring_result = self.core.process_keystroke(key)?;

                    // Update analytics
                    self.analytics.record_keystroke(key, scoring_result)?;

                    // Check completion
                    if self.core.is_session_complete() {
                        break self.core.complete_session()?;
                    }
                }
                InputEvent::Quit => {
                    break self.core.abort_session()?;
                }
                _ => {}
            }
        }
    };

    // 4. Cleanup TTY state (critical for all paths)
    self.terminal.leave_alternate_screen()?;
    self.terminal.leave_raw_mode()?;

    // 5. Persist session results
    self.persistence.save_session(&result).await?;

    Ok(result)
}
```

**Integration Points**:
- `content/`: Load level content with cache (94% hit rate maintained)
- `core/`: Scoring engine, session state, error classification
- `analytics/`: Keystroke timing, error patterns, WPM calculation
- `persistence/`: Session history, profile updates
- `platform/`: TTY management, input handling, terminal detection

**Performance Constraints**:
- Input polling: <10ms intervals
- Keystroke processing: <5ms per key
- Frame render: <33ms P95
- Total input-to-visual: <25ms P99
- Memory: No allocations in hot path (use arena or pre-allocated buffers)

**Error Handling**:
- Use `Result<T, anyhow::Error>` consistently
- NO `unwrap()` or `expect()` in production paths
- Ensure TTY cleanup in Drop impl and panic handlers
- Graceful degradation for cache misses

**Success Criteria**:
- ✅ `cargo build --release` succeeds without warnings
- ✅ `cargo test -p centotype-engine` passes all tests
- ✅ Manual test: `./target/release/centotype play --level 1` enters typing loop
- ✅ Performance: Input latency P99 ≤25ms (measured with benchmark suite)
- ✅ SessionResult contains complete data: keystrokes, errors, timing, accuracy

**Coordination Notes**:
- Depends on: backend-architect event contracts, ui-ux-designer ratatui layout
- Blocks: performance-engineer optimization work, test-automator E2E tests
- Review with: code-reviewer for panic safety, security-auditor for input sanitization

### For backend-architect Agent [TIER 1 - CRITICAL BLOCKER]
**Context**: Freeze stable event contracts and trait boundaries for coordinated development
**Critical Gap**: Unstable interfaces prevent parallel crate development

**Primary Objective**: Define and freeze stable event system and trait boundaries
- Lock down event types: KeyIn, Hit/Miss variants (Subst/Ins/Del/Transp), Tick, Render
- Establish trait boundaries between all crates with performance constraints
- Create ADR (Architecture Decision Record) documenting data flow
- Enable parallel development across rust-pro, ui-ux-designer, performance-engineer

**Event System Definition**:
```rust
// events.rs - Frozen contract for Week 3-4
#[derive(Debug, Clone, PartialEq)]
pub enum GameEvent {
    KeyIn {
        key: KeyCode,
        timestamp: Instant,
        modifiers: KeyModifiers
    },
    Hit {
        position: usize,
        expected: char,
        actual: char,
        timestamp: Instant
    },
    Miss {
        error_type: ErrorType,
        position: usize,
        expected: char,
        actual: char,
        timestamp: Instant
    },
    Tick {
        elapsed: Duration,
        session_progress: f64
    },
    Render {
        frame_time: Duration,
        components: Vec<ComponentUpdate>
    },
    SessionComplete {
        result: SessionResult
    },
    Quit,
}

#[derive(Debug, Clone, PartialEq)]
pub enum ErrorType {
    Substitution,  // Wrong character
    Insertion,     // Extra character
    Deletion,      // Missing character
    Transposition, // Adjacent swap (teh → the)
}
```

**Core Trait Boundaries**:
```rust
// Trait: Core ↔ Engine
pub trait ScoringEngine {
    fn process_keystroke(&mut self, key: KeyCode) -> Result<ScoringResult>;
    fn current_state(&self) -> &SessionState;
    fn is_session_complete(&self) -> bool;
    fn complete_session(&mut self) -> Result<SessionResult>;
}

// Trait: Engine ↔ Content
pub trait ContentLoader {
    async fn load_level_content(&self, level_id: LevelId) -> Result<String>;
    fn get_cached_content(&self, level_id: LevelId) -> Option<String>;
    fn preload_next_levels(&self, current: LevelId, count: usize) -> impl Future<Output = Result<()>>;
}

// Trait: Engine ↔ Analytics
pub trait AnalyticsCollector {
    fn record_keystroke(&mut self, key: KeyCode, result: ScoringResult) -> Result<()>;
    fn calculate_wpm(&self) -> f64;
    fn calculate_accuracy(&self) -> f64;
    fn get_error_distribution(&self) -> ErrorDistribution;
}

// Trait: Engine ↔ Persistence
pub trait SessionPersistence {
    async fn save_session(&self, result: &SessionResult) -> Result<()>;
    async fn load_profile(&self, profile_id: &str) -> Result<UserProfile>;
    async fn update_progress(&self, level_id: LevelId, result: &SessionResult) -> Result<()>;
}

// Trait: Engine ↔ Platform
pub trait TerminalManager {
    fn enter_raw_mode(&mut self) -> Result<()>;
    fn leave_raw_mode(&mut self) -> Result<()>;
    fn enter_alternate_screen(&mut self) -> Result<()>;
    fn leave_alternate_screen(&mut self) -> Result<()>;
    fn poll_event(&self, timeout: Duration) -> Result<Option<InputEvent>>;
}
```

**Data Flow ADR** (create as separate file: `docs/architecture/ADR-001-data-flow.md`):
```
Title: Inter-Crate Data Flow and Performance Boundaries
Status: Accepted
Context: Engine integration requires stable data flow with <25ms P99 latency

Flow Pattern:
content/ → core/ → engine/ → cli/ (main path)
analytics/ ← engine/ (side effects)
persistence/ ← engine/ (async, non-blocking)
platform/ ↔ engine/ (bidirectional, low-level)

Performance Requirements:
- content/ → core/: <5ms content lookup (cache hit)
- core/ → engine/: <5ms scoring calculation
- engine/ → cli/: <15ms render update
- Total input-to-visual: <25ms P99

Constraints:
- No heap allocations in engine/ hot path
- Use Arc<> for shared data, minimize clone()
- Async boundaries only at persistence/ and content/ loading
- Error propagation via Result<T, anyhow::Error>
```

**Success Criteria**:
- ✅ All trait definitions compile and are consistent across crates
- ✅ Event system is complete and covers all user interactions
- ✅ ADR document is created and reviewed
- ✅ Mock implementations allow parallel development
- ✅ Performance constraints are measurable and testable

**Coordination Notes**:
- Enables: rust-pro engine implementation, ui-ux-designer event handling
- Blocks: None (enables parallel development)
- Review with: All agents for interface compatibility

### For test-automator Agent [TIER 1 - CRITICAL BLOCKER]
**Context**: Fix compilation errors and establish comprehensive test coverage
**Critical Gap**: Test failures block CI/CD and prevent regression detection

**Primary Objective**: Resolve all compilation errors and establish test coverage
- Fix missing `Duration` imports and other compilation failures
- Add unit tests for scoring algorithms with deterministic seeds
- Create golden tests for content generation consistency
- Implement E2E tests for play→score→save workflow across platforms

**Immediate Fixes Required**:
```rust
// Fix missing imports in core/src/lib.rs
use std::time::{Duration, Instant};

// Fix test compilation in core/tests/
#[cfg(test)]
mod tests {
    use super::*;
    use std::time::Duration; // Add this import

    #[test]
    fn test_scoring_deterministic() {
        // Test with fixed seed for reproducible results
        let seed = 12345u64;
        let mut rng = ChaCha8Rng::seed_from_u64(seed);
        // ... test implementation
    }
}
```

**Test Coverage Requirements**:
1. **Unit Tests** (Target: ≥80% coverage for scoring):
   ```bash
   # Core scoring algorithms
   cargo test -p centotype-core scoring::tests::test_wpm_calculation
   cargo test -p centotype-core scoring::tests::test_accuracy_calculation
   cargo test -p centotype-core scoring::tests::test_error_classification
   cargo test -p centotype-core scoring::tests::test_transposition_detection

   # Deterministic content generation
   cargo test -p centotype-content generator::tests::test_deterministic_generation
   cargo test -p centotype-content validation::tests::test_security_validation
   ```

2. **Golden Tests** (Snapshot testing with insta crate):
   ```rust
   #[test]
   fn test_level_content_golden() {
       for level_id in 1..=10 {
           let content = generator.generate_level_content(level_id, FIXED_SEED)?;
           insta::assert_snapshot!(format!("level_{}", level_id), content);
       }
   }
   ```

3. **E2E Tests** (Cross-platform validation):
   ```bash
   # Test complete workflow
   ./target/release/centotype play --level 1 --test-mode < test_input.txt
   # Verify session was saved and stats updated
   ./target/release/centotype stats | grep "Level 1"
   ```

4. **Performance Tests** (Regression prevention):
   ```rust
   #[bench]
   fn bench_input_latency(b: &mut Bencher) {
       // Measure keystroke processing time
       // Assert P99 ≤ 25ms
   }
   ```

**Test Infrastructure**:
- Use `cargo-nextest` for faster test execution
- Setup test fixtures for deterministic content
- Mock implementations for external dependencies
- CI gates: All tests must pass before merge

**Success Criteria**:
- ✅ `cargo test --workspace` passes without compilation errors
- ✅ Coverage: scoring ≥80%, overall ≥70%
- ✅ E2E test validates complete play→score→save flow
- ✅ Performance tests establish regression baselines
- ✅ Golden tests prevent content generation drift

**Coordination Notes**:
- Depends on: backend-architect trait definitions
- Blocks: CI/CD pipeline stability, regression detection
- Review with: code-reviewer for test quality, debugger for failure analysis

### For ui-ux-designer Agent [TIER 2 - CORE]
**Context**: Minimal ratatui layout with accessibility compliance
**Critical Gap**: No visual interface for typing loop interaction

**Primary Objective**: Create functional TUI layout supporting real-time typing
- Design typing pane with cursor positioning and text highlighting
- Status bar with WPM/Accuracy/Combo counters
- Progress indicator and help keymap overlay
- Alt-screen guard with WCAG AA contrast and mono font fallback

**Layout Specification**:
```
┌─────────────────────────────────────────────────────────────┐
│ Centotype CLI - Level 5 (Tier 1) - Programming Basics      │ Header
├─────────────────────────────────────────────────────────────┤
│                                                             │
│ Target: function calculateSum(arr) {                        │
│                return arr.reduce((a,b) => a+b, 0);         │ Typing
│         }                                                   │ Pane
│                                                             │
│ Input:  function calculateSum(arr) {█                       │ (60% height)
│                                                             │
├─────────────────────────────────────────────────────────────┤
│ WPM: 45 │ ACC: 94.2% │ COMBO: x12 │ ⏱️  1:23 │ ERR: 3     │ Status Bar
├─────────────────────────────────────────────────────────────┤
│ Progress: ████████████████░░░░░░░░░░ 65% (195/300 chars)   │ Progress
├─────────────────────────────────────────────────────────────┤
│ ESC:quit │ F1:help │ Ctrl+R:restart │ Tab:stats             │ Help Bar
└─────────────────────────────────────────────────────────────┘
```

**Component Implementation**:
```rust
pub struct TypingUI {
    typing_pane: TypingPane,
    status_bar: StatusBar,
    progress_bar: ProgressBar,
    help_overlay: HelpOverlay,
    terminal: Terminal<CrosstermBackend<io::Stdout>>,
}

impl TypingUI {
    pub fn render_frame(&mut self, state: &SessionState) -> Result<()> {
        self.terminal.draw(|f| {
            let chunks = Layout::default()
                .direction(Direction::Vertical)
                .constraints([
                    Constraint::Length(1),      // Header
                    Constraint::Percentage(60), // Typing pane
                    Constraint::Length(1),      // Status bar
                    Constraint::Length(1),      // Progress
                    Constraint::Length(1),      // Help
                ])
                .split(f.size());

            // Header with level info
            let header = Paragraph::new(format!(
                "Centotype CLI - Level {} (Tier {}) - {}",
                state.level_id,
                state.tier,
                state.description
            ));
            f.render_widget(header, chunks[0]);

            // Typing pane with cursor and highlighting
            self.typing_pane.render(f, chunks[1], state);

            // Real-time status updates
            self.status_bar.render(f, chunks[2], state);

            // Visual progress indicator
            self.progress_bar.render(f, chunks[3], state);

            // Help keymap
            self.help_overlay.render(f, chunks[4]);
        })?;

        Ok(())
    }
}

// Critical: Cursor positioning for real-time feedback
impl TypingPane {
    fn render(&self, frame: &mut Frame, area: Rect, state: &SessionState) {
        let target_text = &state.target_text;
        let current_input = &state.current_input;

        // Highlight correct (green), incorrect (red), cursor position
        let styled_text = self.style_text_with_cursor(target_text, current_input);

        let paragraph = Paragraph::new(styled_text)
            .wrap(Wrap { trim: true })
            .block(Block::default().borders(Borders::ALL));

        frame.render_widget(paragraph, area);

        // Position cursor at current input location
        if let Some(cursor_pos) = self.calculate_cursor_position(current_input.len(), area) {
            frame.set_cursor(cursor_pos.x, cursor_pos.y);
        }
    }
}
```

**Accessibility Requirements (WCAG AA)**:
- **Contrast**: 4.5:1 minimum for text/background
- **Color Independence**: Errors shown with symbols, not just color
- **Keyboard Navigation**: All functions accessible via keyboard
- **Screen Reader**: Descriptive text for status updates
- **Mono Font Fallback**: Handle missing font gracefully

**Color Scheme** (Terminal-safe):
```rust
pub const UI_COLORS: UiColors = UiColors {
    correct_text: Color::Green,
    incorrect_text: Color::Red,
    cursor: Color::Yellow,
    normal_text: Color::White,
    status_bg: Color::DarkGray,
    progress_complete: Color::Blue,
    progress_remaining: Color::Gray,
};
```

**Success Criteria**:
- ✅ Typing interface shows target text and real-time input
- ✅ Cursor positioning is accurate during typing
- ✅ Status bar updates show WPM/accuracy in real-time
- ✅ Color contrast meets WCAG AA standards
- ✅ Layout works on minimum 80x24 terminal size
- ✅ Alt-screen is properly managed (no terminal corruption)

**Coordination Notes**:
- Depends on: backend-architect event system, rust-pro engine integration
- Provides: Visual interface for engine typing loop
- Review with: dx-optimizer for usability, test-automator for UI testing

### For performance-engineer Agent [TIER 2 - CORE]
**Context**: Critical 3ms optimization needed (28ms → <25ms P99 latency)
**Critical Gap**: Performance grade B+ → A requires hot-path optimization

**Primary Objective**: Achieve <25ms P99 input latency through systematic optimization
- Profile keystroke→frame hot path to identify bottlenecks
- Apply specific optimizations: ANSI batching, line precomposition, arena allocation
- Target breakdown: input capture <5ms, scoring <5ms, render <15ms
- Maintain other targets: P95 render ≤33ms, P95 startup ≤200ms, RSS ≤50MB

**Hot Path Profiling Setup**:
```rust
// Add to engine/src/profiler.rs
pub struct LatencyProfiler {
    input_capture_times: CircularBuffer<Duration>,
    scoring_times: CircularBuffer<Duration>,
    render_times: CircularBuffer<Duration>,
    total_times: CircularBuffer<Duration>,
}

impl LatencyProfiler {
    pub fn measure_input_cycle<F, R>(&mut self, f: F) -> R
    where F: FnOnce() -> R {
        let start = Instant::now();
        let result = f();
        let elapsed = start.elapsed();

        self.total_times.push(elapsed);

        // Detailed breakdown measurement
        result
    }

    pub fn report_p99_latency(&self) -> LatencyReport {
        LatencyReport {
            total_p99: self.calculate_percentile(&self.total_times, 0.99),
            input_p99: self.calculate_percentile(&self.input_capture_times, 0.99),
            scoring_p99: self.calculate_percentile(&self.scoring_times, 0.99),
            render_p99: self.calculate_percentile(&self.render_times, 0.99),
        }
    }
}
```

**Specific Optimizations to Apply**:

1. **ANSI Sequence Batching** (Target: 40% render improvement):
```rust
// Instead of: individual cursor movements
write!(stdout, "\x1b[{};{}H{}", y, x, char)?;
write!(stdout, "\x1b[{};{}H{}", y+1, x, char)?;

// Use: batched sequences
let batch = format!(
    "\x1b[{};{}H{}\x1b[{};{}H{}",
    y, x, char1, y+1, x, char2
);
stdout.write_all(batch.as_bytes())?;
stdout.flush()?; // Single flush per frame
```

2. **Line Precomposition** (Target: 30% memory improvement):
```rust
// Pre-compose styled lines instead of character-by-character
pub struct PrecomposedLine {
    content: String,        // Full line with ANSI codes
    cursor_positions: Vec<usize>, // Byte offsets for cursor placement
}

impl TypingPane {
    fn precompose_lines(&self, target: &str, input: &str) -> Vec<PrecomposedLine> {
        // Generate complete lines with styling upfront
        // Reuse for multiple frames until input changes
    }
}
```

3. **Arena Allocation** (Target: Zero hot-path allocations):
```rust
pub struct RenderArena {
    string_buffer: Vec<u8>,     // Reused string building
    line_buffer: Vec<PrecomposedLine>, // Reused line storage
    style_buffer: Vec<Style>,   // Reused style calculations
}

impl RenderArena {
    pub fn prepare_frame(&mut self, state: &SessionState) -> FrameData {
        // Clear buffers (no deallocation)
        self.string_buffer.clear();
        self.line_buffer.clear();

        // Build frame using pre-allocated capacity
        // Return structured data, no heap allocation
    }
}
```

4. **Input Event Batching** (Target: Reduce system calls):
```rust
pub fn poll_events_batch(&self, timeout: Duration) -> Result<Vec<InputEvent>> {
    let mut events = Vec::with_capacity(8);
    let deadline = Instant::now() + timeout;

    // Collect multiple events in single system call cycle
    while Instant::now() < deadline && events.len() < 8 {
        if let Some(event) = self.poll_single_event(Duration::from_millis(1))? {
            events.push(event);
        }
    }

    Ok(events)
}
```

**Benchmarking Setup**:
```rust
// Add to benches/input_latency.rs
fn bench_input_to_visual_latency(b: &mut Bencher) {
    let mut engine = setup_test_engine();
    let test_keys = generate_test_keystroke_sequence();

    b.iter(|| {
        let start = Instant::now();

        // Simulate keystroke processing
        for key in &test_keys {
            engine.process_keystroke(*key)?;
            engine.render_frame()?;
        }

        let elapsed = start.elapsed();
        assert!(elapsed.as_millis() < 25, "P99 latency exceeded: {}ms", elapsed.as_millis());
    });
}
```

**Memory Usage Optimization**:
- Target: RSS ≤50MB during active typing
- Use `jemalloc` for better allocation patterns
- Monitor with: `cargo run --bin memory-profiler`
- Profile with: `heaptrack` or `valgrind massif`

**Success Criteria**:
- ✅ P99 input latency ≤25ms (measured with realistic typing patterns)
- ✅ P95 render time ≤33ms
- ✅ P95 startup time ≤200ms
- ✅ Memory usage ≤50MB RSS
- ✅ Benchmark suite shows consistent improvements
- ✅ No performance regressions in other components

**Coordination Notes**:
- Depends on: rust-pro engine implementation, ui-ux-designer render components
- Provides: Performance validation for Phase 1 close-out
- Review with: code-reviewer for optimization safety, test-automator for benchmark integration

### For debugger Agent [TIER 3 - QUALITY]
**Context**: Triage critical issues blocking smooth typing experience
**Critical Gap**: Latency spikes, input handling edge cases, TTY state management

**Primary Objective**: Diagnose and resolve system-level issues
- Investigate latency spikes and input processing delays
- Fix ghost input and key repeat issues
- Ensure robust TTY state restoration on all exit paths
- Provide reproducible test cases and quick patches

**Issue Categories to Investigate**:

1. **Latency Spikes** (Target: Eliminate outliers >50ms):
```rust
// Debug latency distribution
pub fn debug_latency_spikes() {
    let mut profiler = LatencyProfiler::new();

    // Simulate 1000 keystrokes
    for i in 0..1000 {
        let latency = profiler.measure_keystroke_cycle(|| {
            // Full input→render cycle
        });

        // Log outliers for analysis
        if latency > Duration::from_millis(50) {
            eprintln!("SPIKE at keystroke {}: {}ms", i, latency.as_millis());
            // Capture stack trace, system state
        }
    }
}
```

2. **Ghost Input Detection**:
```rust
pub fn debug_ghost_input() -> Result<()> {
    let mut input_handler = InputHandler::new()?;

    println!("Press keys to test input handling (Ctrl+C to exit):");

    loop {
        match input_handler.poll_event(Duration::from_millis(100))? {
            Some(event) => {
                println!("Event: {:?} at {:?}", event, Instant::now());

                // Check for duplicate events
                if let Some(last_event) = last_event {
                    let time_diff = event.timestamp - last_event.timestamp;
                    if time_diff < Duration::from_millis(5) && event.key == last_event.key {
                        println!("GHOST INPUT DETECTED: Duplicate key within 5ms");
                    }
                }
            },
            None => {
                // Check for missed inputs or system lag
            }
        }
    }
}
```

3. **TTY State Recovery**:
```rust
// Test TTY cleanup on various exit scenarios
pub fn test_tty_recovery() -> Result<()> {
    let scenarios = vec![
        "normal_exit",
        "ctrl_c_interrupt",
        "panic_during_render",
        "external_sigterm",
        "terminal_resize",
    ];

    for scenario in scenarios {
        println!("Testing TTY recovery for: {}", scenario);

        match scenario {
            "panic_during_render" => {
                // Simulate panic and ensure TTY is restored
                let _guard = TTYGuard::new()?;
                panic!("Simulated panic");
            },
            // ... other scenarios
        }

        // Verify terminal is in normal state
        assert_terminal_state_normal()?;
    }

    Ok(())
}

// Safety guard for TTY state
pub struct TTYGuard {
    terminal: Terminal,
}

impl Drop for TTYGuard {
    fn drop(&mut self) {
        // Always restore terminal state
        let _ = self.terminal.leave_alternate_screen();
        let _ = self.terminal.leave_raw_mode();

        // Force cursor visibility
        let _ = write!(io::stdout(), "\x1b[?25h");
        let _ = io::stdout().flush();
    }
}
```

**Diagnostic Tools**:

1. **Latency Analyzer**:
```bash
# Create debug binary
cargo build --bin latency-debugger

# Run with detailed logging
RUST_LOG=debug ./target/debug/latency-debugger --profile-keystrokes=1000

# Analyze results
./analyze_latency_spikes.py latency_log.json
```

2. **Input Event Logger**:
```rust
// Log all input events with microsecond precision
pub struct InputEventLogger {
    log_file: File,
    start_time: Instant,
}

impl InputEventLogger {
    pub fn log_event(&mut self, event: &InputEvent) -> Result<()> {
        let timestamp = self.start_time.elapsed();
        let log_entry = serde_json::json!({
            "timestamp_us": timestamp.as_micros(),
            "event_type": format!("{:?}", event),
            "key_code": event.key_code(),
            "modifiers": event.modifiers(),
        });

        writeln!(self.log_file, "{}", log_entry)?;
        Ok(())
    }
}
```

3. **System Resource Monitor**:
```rust
// Monitor system resources during typing
pub fn monitor_system_resources() -> Result<()> {
    let mut monitor = SystemMonitor::new();

    loop {
        let stats = SystemStats {
            cpu_usage: monitor.cpu_usage()?,
            memory_usage: monitor.memory_usage()?,
            file_descriptors: monitor.fd_count()?,
            context_switches: monitor.context_switches()?,
        };

        // Alert on resource spikes
        if stats.cpu_usage > 50.0 {
            eprintln!("HIGH CPU: {}%", stats.cpu_usage);
        }

        thread::sleep(Duration::from_millis(100));
    }
}
```

**Reproducer Scripts**:
```bash
#!/bin/bash
# reproduce_latency_spike.sh

echo "Reproducing latency spike scenario..."

# Setup test environment
export RUST_LOG=debug
export CENTOTYPE_DEBUG=1

# Run typing test with known problematic input
echo "abcdefghijklmnopqrstuvwxyz" | \
    timeout 30s ./target/debug/centotype play --level 1 --debug-latency

# Analyze logs
if [ -f latency_debug.log ]; then
    echo "=== LATENCY SPIKES DETECTED ==="
    grep "SPIKE" latency_debug.log
fi
```

**Success Criteria**:
- ✅ P99 latency spikes <50ms (no outliers beyond 2x target)
- ✅ Zero ghost input events in 10-minute continuous typing
- ✅ TTY state properly restored on all exit paths (normal, panic, signal)
- ✅ Reproducible test cases for all identified issues
- ✅ Performance patches validated by performance-engineer

**Coordination Notes**:
- Depends on: rust-pro engine implementation for reproduction
- Provides: Issue diagnosis and quick fixes
- Review with: code-reviewer for safety, performance-engineer for impact validation

### For code-reviewer Agent [TIER 3 - QUALITY]
**Context**: Maintain code quality during rapid integration phase
**Critical Gap**: Need systematic review process preventing technical debt

**Primary Objective**: Establish code review standards and maintain quality gates
- Create CR Playbook v1 with prioritized review categories
- Ensure consistent Result<T, anyhow::Error> patterns, no unwrap()/expect()
- Review all engine integration PRs for panic safety and performance
- Maintain ≤24hr review SLA for critical path blockers

**CR Playbook v1 Structure**:
```markdown
# Code Review Playbook v1.0

## Review Priority Matrix
🚨 **P0 - Critical**: Panic safety, TTY cleanup, performance regressions
⚠️  **P1 - High**: Memory leaks, error handling, API consistency
📋 **P2 - Medium**: Style, documentation, test coverage
💡 **P3 - Low**: Optimizations, refactoring suggestions
✅ **P4 - Info**: Acknowledgment, design questions

## Review Categories

### 🚨 Critical Review Points

#### Panic Safety (Zero Tolerance)
❌ NO unwrap() or expect() in production code paths
❌ NO panic!() in user-reachable code
❌ NO unchecked array/vector indexing
✅ Use Result<T, anyhow::Error> consistently
✅ Graceful error handling with context

Example Comments:
```rust
// ❌ BLOCKING: Panic risk
let content = self.cache.get(&key).unwrap();

// ✅ APPROVED: Safe error handling
let content = self.cache.get(&key)
    .ok_or_else(|| anyhow!("Content not found for key: {}", key))?;
```

#### TTY State Management
❌ NO raw TTY operations without cleanup guards
❌ NO alt-screen without proper restoration
✅ Use RAII guards for TTY state
✅ Test all exit paths (normal, panic, signal)

### ⚠️ High Priority Review Points

#### Performance Impact
- Measure: Any change affecting hot path must include benchmark results
- Memory: No allocations in input processing loop
- Latency: Document impact on <25ms P99 target

#### Error Propagation
```rust
// ✅ PREFERRED: Consistent error handling
pub fn process_keystroke(&mut self, key: KeyCode) -> Result<ScoringResult> {
    let analysis = self.analyzer.analyze_key(key)
        .context("Failed to analyze keystroke")?;

    let score = self.scorer.calculate_score(&analysis)
        .context("Failed to calculate score")?;

    Ok(ScoringResult { analysis, score })
}
```

## Review Templates

### Engine Integration PR Template
- [ ] ✅ No panic/unwrap in hot path
- [ ] ✅ TTY cleanup tested on all exit paths
- [ ] ✅ Performance benchmark results included
- [ ] ✅ Error handling follows Result<T, anyhow::Error>
- [ ] ✅ Memory allocations documented and justified
- [ ] ✅ Integration tests pass
```

**Automated Quality Gates (CI Integration)**:
```yaml
# .github/workflows/quality-gates.yml
name: Quality Gates

on: [push, pull_request]

jobs:
  code-quality:
    runs-on: ubuntu-latest
    steps:
      - name: Format Check
        run: cargo fmt --check

      - name: Clippy (Deny Warnings)
        run: cargo clippy -- -D warnings

      - name: Test All Features
        run: cargo test --all-features

      - name: Security Audit
        run: cargo audit

      - name: Dependency Check
        run: cargo deny check

      - name: Coverage Gate
        run: |
          cargo tarpaulin --out Xml
          # Require: scoring ≥80%, overall ≥70%
          python check_coverage.py --min-scoring=80 --min-overall=70

      - name: Performance Gate
        run: |
          cargo bench --bench input_latency_benchmark
          # Fail if P99 >25ms or P95 render >33ms vs baseline
          python check_performance_regression.py
```

**PR Review Templates**:

1. **Engine Integration Review**:
```markdown
## 🚨 Critical Review - Engine Integration

### Panic Safety ✅❌
- [ ] No unwrap()/expect() in production paths
- [ ] All array/vector access bounds-checked
- [ ] Error handling uses Result<T, anyhow::Error>

### TTY Management ✅❌
- [ ] Raw mode properly guarded with RAII
- [ ] Alt-screen cleanup tested
- [ ] Panic handlers restore terminal state

### Performance Impact ✅❌
- [ ] Benchmark results show <25ms P99 maintained
- [ ] No heap allocations in hot path
- [ ] Memory usage ≤50MB validated

### Integration Points ✅❌
- [ ] Content loading maintains 94% cache hit rate
- [ ] Cross-crate communication follows trait boundaries
- [ ] SessionResult properly populated

**Review Decision**: ✅ APPROVE | ⚠️ APPROVE WITH CHANGES | ❌ REQUEST CHANGES

**Comments**:
[Specific feedback with code snippets and suggestions]
```

2. **Test Coverage Review**:
```markdown
## 📋 Test Coverage Review

### Coverage Metrics ✅❌
- [ ] Scoring algorithms: ≥80%
- [ ] Overall project: ≥70%
- [ ] New code: 100% covered

### Test Quality ✅❌
- [ ] Unit tests with deterministic seeds
- [ ] Integration tests for cross-crate communication
- [ ] E2E tests for complete workflows
- [ ] Performance regression tests

**Missing Coverage**:
[List specific uncovered code paths]

**Test Recommendations**:
[Suggest additional test scenarios]
```

**Review SLA Management**:
```rust
// Internal tracking for review team
pub struct ReviewMetrics {
    pr_submitted: Instant,
    first_review: Option<Instant>,
    approval_time: Option<Instant>,
    review_iterations: u32,
}

impl ReviewMetrics {
    pub fn sla_status(&self) -> SLAStatus {
        let elapsed = self.pr_submitted.elapsed();

        match elapsed {
            d if d < Duration::from_hours(4) => SLAStatus::OnTime,
            d if d < Duration::from_hours(24) => SLAStatus::Warning,
            _ => SLAStatus::Breached,
        }
    }
}
```

**Daily CR Digest Format**:
```markdown
# Code Review Digest - 2025-09-28

## 🚨 Critical Issues Found
- engine/src/lib.rs:245 - Panic risk with unwrap() in hot path
- core/src/scoring.rs:89 - Unchecked vector indexing

## 🔒 Locked Architecture Decisions
- ADR-001: Event system contracts frozen
- ADR-002: Error handling standardized on anyhow::Error

## 📋 Action Items
- [ ] rust-pro: Fix panic safety issues by EOD
- [ ] test-automator: Add missing Duration imports
- [ ] performance-engineer: Validate benchmark baselines

## ✅ Approved & Merged
- content: Security validation improvements
- docs: Updated MASTER_PROMPT.md with v2.0 specs

**Next 24hr Focus**: Engine integration PR reviews
```

**Success Criteria**:
- ✅ CR Playbook v1 documented and team-adopted
- ✅ Zero unwrap()/expect() in merged production code
- ✅ 100% of critical PRs reviewed within 24hr SLA
- ✅ Performance gates prevent regressions
- ✅ All quality gates pass before merge

**Coordination Notes**:
- Reviews: All agent deliverables before merge
- Blocks: PR merges until quality standards met
- Escalates: Architecture violations to backend-architect

### For security-auditor Agent [TIER 3 - QUALITY]
**Context**: Comprehensive security validation for terminal application
**Critical Gap**: Input sanitization, escape sequence filtering, permission auditing

**Primary Objective**: Audit all security vectors and establish continuous validation
- Audit terminal escape sequence handling and input sanitization
- Validate file system permissions and path traversal protection
- Run fuzzing tests ≥4 hours without crashes (zero high-risk findings)
- Create security test suite for continuous validation

**Security Audit Categories**:

1. **Terminal Escape Sequence Injection**:
```rust
// Test malicious escape sequences in content
#[test]
fn test_escape_sequence_filtering() {
    let malicious_inputs = vec![
        "\x1b]0;rm -rf /\x07",           // Terminal title injection
        "\x1b[2J\x1b[H\x1b[3J",         // Screen clearing
        "\x1b[?1049h",                   // Alt screen manipulation
        "\x1b[?25l",                     // Cursor hiding
        "\x1b[6n",                       // Cursor position request
        "\x1b[>c",                       // Device attributes request
        "hello\x1b[A\x1b[2Kworld",      // Cursor manipulation
    ];

    for input in malicious_inputs {
        let filtered = SecurityValidator::sanitize_content(input)?;

        // Must not contain any escape sequences
        assert!(!filtered.contains('\x1b'), "Escape sequence not filtered: {}", input);
        assert!(!filtered.contains('\x07'), "BEL character not filtered: {}", input);
        assert!(!filtered.contains('\x0c'), "Form feed not filtered: {}", input);
    }
}

// Validate content generation security
pub fn audit_content_security() -> Result<SecurityReport> {
    let mut report = SecurityReport::new();

    // Test all 100 levels for security violations
    for level_id in 1..=100 {
        let content = generator.generate_level_content(level_id, TEST_SEED)?;

        // Check for escape sequences
        if content.contains('\x1b') || content.contains('\x07') {
            report.add_violation(SecurityViolation::EscapeSequence {
                level_id,
                content: content.clone(),
            });
        }

        // Check for shell injection patterns
        let injection_patterns = [
            "$(",  "$(", "`", "&&", "||", ";", "|",
            "rm ", "cat ", "curl ", "wget ", "chmod "
        ];

        for pattern in &injection_patterns {
            if content.contains(pattern) {
                report.add_violation(SecurityViolation::ShellInjection {
                    level_id,
                    pattern: pattern.to_string(),
                    content: content.clone(),
                });
            }
        }
    }

    Ok(report)
}
```

2. **File System Security**:
```rust
// Audit file access patterns and permissions
pub fn audit_file_permissions() -> Result<PermissionReport> {
    let mut report = PermissionReport::new();

    // Check config file permissions
    let config_path = get_config_path()?;
    let metadata = fs::metadata(&config_path)?;
    let permissions = metadata.permissions();

    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mode = permissions.mode();

        // Config files should not be world-readable (no 004 bit)
        if mode & 0o004 != 0 {
            report.add_issue(PermissionIssue::WorldReadable {
                path: config_path.clone(),
                mode,
            });
        }

        // Should not be group-writable (no 020 bit)
        if mode & 0o020 != 0 {
            report.add_issue(PermissionIssue::GroupWritable {
                path: config_path.clone(),
                mode,
            });
        }
    }

    // Test path traversal protection
    let malicious_paths = vec![
        "../../../etc/passwd",
        "..\\..\\..\\windows\\system32\\config\\sam",
        "/etc/shadow",
        "~/.ssh/id_rsa",
        "/dev/null",
        "/proc/self/mem",
    ];

    for path in malicious_paths {
        match persistence.load_profile(path).await {
            Ok(_) => {
                report.add_issue(PermissionIssue::PathTraversal {
                    attempted_path: path.to_string(),
                });
            },
            Err(_) => {
                // Expected - should reject malicious paths
            }
        }
    }

    Ok(report)
}
```

3. **Input Fuzzing**:
```rust
// Fuzz test input handling for crashes and memory safety
pub fn fuzz_input_handling(duration: Duration) -> Result<FuzzReport> {
    let mut report = FuzzReport::new();
    let mut rng = ChaCha8Rng::from_entropy();
    let end_time = Instant::now() + duration;

    let mut test_count = 0;

    while Instant::now() < end_time {
        // Generate random input sequences
        let input_length = rng.gen_range(1..=1000);
        let mut input_sequence = Vec::with_capacity(input_length);

        for _ in 0..input_length {
            // Mix of printable, control characters, and edge cases
            let char_class = rng.gen_range(0..10);
            let ch = match char_class {
                0..=6 => rng.gen_range(b' '..=b'~') as char, // Printable ASCII
                7 => rng.gen_range(0..=31) as char,          // Control characters
                8 => char::from_u32(rng.gen_range(128..=255)).unwrap_or('\0'), // Extended ASCII
                9 => char::from_u32(rng.gen_range(0x1000..=0x1FFFF)).unwrap_or('\0'), // Unicode
                _ => unreachable!(),
            };
            input_sequence.push(ch);
        }

        let input_string: String = input_sequence.into_iter().collect();

        // Test input processing without crashing
        match test_input_processing(&input_string) {
            Ok(_) => {},
            Err(e) if e.is_recoverable() => {},
            Err(e) => {
                report.add_crash(FuzzCrash {
                    input: input_string,
                    error: e,
                    test_number: test_count,
                });
            }
        }

        test_count += 1;

        if test_count % 1000 == 0 {
            println!("Fuzz progress: {} tests completed", test_count);
        }
    }

    report.total_tests = test_count;
    report.duration = duration;

    Ok(report)
}

fn test_input_processing(input: &str) -> Result<(), RecoverableError> {
    // Initialize engine in safe test mode
    let mut engine = TypingEngine::new_test_mode()?;

    // Process input character by character
    for ch in input.chars() {
        let key_code = KeyCode::Char(ch);
        match engine.process_keystroke(key_code) {
            Ok(_) => {},
            Err(e) if e.is_user_error() => {
                // Expected errors (invalid input, etc.) are OK
            },
            Err(e) => {
                return Err(RecoverableError::ProcessingError(e));
            }
        }
    }

    Ok(())
}
```

4. **Secrets and Credentials Scanning**:
```bash
#!/bin/bash
# Security scan for embedded secrets

echo "=== Scanning for embedded secrets ==="

# Scan source code for potential secrets
rg -i "password|secret|token|api_key|private_key" \
   --type rust \
   --context 2 \
   src/

# Check for hardcoded paths that might expose system info
rg "/home/|/Users/|C:\\|/etc/|/var/" \
   --type rust \
   src/

# Scan for unsafe operations
rg "unsafe|transmute|from_raw" \
   --type rust \
   src/

# Check test files for credentials
find . -name "*.rs" -path "*/tests/*" -exec \
  rg -l "password|secret|key" {} \;
```

**Security Test Suite**:
```rust
// Comprehensive security test harness
#[cfg(test)]
mod security_tests {
    use super::*;

    #[test]
    fn test_comprehensive_security_suite() {
        // 1. Escape sequence filtering
        test_escape_sequence_filtering().expect("Escape sequence test failed");

        // 2. File permission audit
        let perm_report = audit_file_permissions().expect("Permission audit failed");
        assert_eq!(perm_report.critical_issues().len(), 0,
                  "Critical permission issues found: {:?}", perm_report.critical_issues());

        // 3. Input sanitization
        test_input_sanitization().expect("Input sanitization test failed");

        // 4. Content generation security
        let content_report = audit_content_security().expect("Content security audit failed");
        assert_eq!(content_report.high_risk_violations().len(), 0,
                  "High-risk content violations: {:?}", content_report.high_risk_violations());
    }

    #[test]
    #[ignore] // Long-running test
    fn test_extended_fuzz_suite() {
        let duration = Duration::from_secs(4 * 3600); // 4 hours
        let fuzz_report = fuzz_input_handling(duration).expect("Fuzz test failed");

        // Zero high-risk crashes allowed
        assert_eq!(fuzz_report.high_risk_crashes().len(), 0,
                  "High-risk crashes found: {:?}", fuzz_report.high_risk_crashes());

        println!("Fuzz test completed: {} tests, {} crashes",
                fuzz_report.total_tests, fuzz_report.total_crashes());
    }
}
```

**Continuous Security Monitoring**:
```yaml
# .github/workflows/security.yml
name: Security Audit

on:
  push:
    branches: [ main ]
  pull_request:
    branches: [ main ]
  schedule:
    - cron: '0 2 * * *'  # Daily at 2 AM

jobs:
  security-audit:
    runs-on: ubuntu-latest
    steps:
      - name: Cargo Audit
        run: cargo audit --deny warnings

      - name: Cargo Deny
        run: cargo deny check

      - name: Security Test Suite
        run: cargo test security_tests::test_comprehensive_security_suite

      - name: Extended Fuzz (Weekly)
        if: github.event.schedule
        run: cargo test security_tests::test_extended_fuzz_suite -- --ignored

      - name: CodeQL Analysis
        uses: github/codeql-action/analyze@v2
        with:
          languages: rust

      - name: Semgrep Security Scan
        run: |
          pip install semgrep
          semgrep --config=auto --error --verbose ./src/
```

**Success Criteria**:
- ✅ Zero high-risk security findings in comprehensive audit
- ✅ 4-hour fuzz test completes without crashes
- ✅ All escape sequences properly filtered from content generation
- ✅ File permissions follow least-privilege principle
- ✅ No embedded secrets or credentials in codebase
- ✅ Security test suite integrated into CI/CD pipeline

**Coordination Notes**:
- Audits: All agent deliverables for security compliance
- Blocks: Merges with high-risk security findings
- Escalates: Critical security issues to code-reviewer and project lead

### For devops-troubleshooter Agent [TIER 4 - SUPPORT]
**Context**: Maintain CI/CD stability during integration phase
**Critical Gap**: Cross-platform build matrix, performance monitoring, reproducible artifacts

**Primary Objective**: Ensure robust CI/CD infrastructure supporting Phase 1 close-out
- Maintain green build matrix (Linux gnu/musl, macOS Intel/ARM, Windows)
- Add nightly performance report generation
- Create reproducible build artifacts for distribution readiness
- Monitor system stability during rapid development phase

**Build Matrix Stability**:
```yaml
# .github/workflows/ci.yml
name: CI Matrix

on: [push, pull_request]

strategy:
  fail-fast: false
  matrix:
    include:
      # Linux builds
      - target: x86_64-unknown-linux-gnu
        os: ubuntu-latest
        rust: stable

      - target: x86_64-unknown-linux-musl
        os: ubuntu-latest
        rust: stable

      - target: aarch64-unknown-linux-gnu
        os: ubuntu-latest
        rust: stable

      # macOS builds
      - target: x86_64-apple-darwin
        os: macos-latest
        rust: stable

      - target: aarch64-apple-darwin
        os: macos-latest
        rust: stable

      # Windows builds
      - target: x86_64-pc-windows-msvc
        os: windows-latest
        rust: stable

      - target: x86_64-pc-windows-gnu
        os: windows-latest
        rust: stable

jobs:
  test:
    name: Test ${{ matrix.target }}
    runs-on: ${{ matrix.os }}

    steps:
      - name: Checkout
        uses: actions/checkout@v3

      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          profile: minimal
          toolchain: ${{ matrix.rust }}
          target: ${{ matrix.target }}
          override: true

      - name: Setup Linux Dependencies
        if: matrix.os == 'ubuntu-latest'
        run: |
          sudo apt-get update
          sudo apt-get install -y musl-tools gcc-aarch64-linux-gnu

      - name: Build
        run: cargo build --target ${{ matrix.target }} --release

      - name: Test
        run: cargo test --target ${{ matrix.target }} --all-features

      - name: Upload Artifacts
        uses: actions/upload-artifact@v3
        with:
          name: centotype-${{ matrix.target }}
          path: target/${{ matrix.target }}/release/centotype*
```

**Nightly Performance Monitoring**:
```yaml
# .github/workflows/nightly-perf.yml
name: Nightly Performance Report

on:
  schedule:
    - cron: '0 3 * * *'  # 3 AM daily
  workflow_dispatch:     # Manual trigger

jobs:
  performance-benchmarks:
    runs-on: ubuntu-latest

    steps:
      - name: Checkout
        uses: actions/checkout@v3

      - name: Setup Rust
        uses: actions-rs/toolchain@v1
        with:
          profile: minimal
          toolchain: stable
          override: true

      - name: Run Benchmark Suite
        run: |
          # Build optimized binary
          cargo build --release

          # Run all performance benchmarks
          cargo bench --bench input_latency_benchmark > latency_results.txt
          cargo bench --bench content_performance_benchmark > content_results.txt
          cargo bench --bench render_performance_benchmark > render_results.txt

          # Generate memory usage report
          cargo run --bin memory-profiler > memory_report.txt

          # Startup time analysis
          for i in {1..10}; do
            /usr/bin/time -f "%e" ./target/release/centotype --help 2>> startup_times.txt
          done

      - name: Analyze Results
        run: |
          python scripts/analyze_performance.py \
            --latency latency_results.txt \
            --content content_results.txt \
            --render render_results.txt \
            --memory memory_report.txt \
            --startup startup_times.txt \
            --output performance_report.json

      - name: Check Performance Regression
        run: |
          python scripts/check_regression.py \
            --current performance_report.json \
            --baseline performance_baseline.json \
            --fail-on-regression

      - name: Update Performance Dashboard
        run: |
          # Upload to performance tracking dashboard
          curl -X POST \
            -H "Content-Type: application/json" \
            -d @performance_report.json \
            "$PERFORMANCE_DASHBOARD_URL"

      - name: Generate Performance Badge
        run: |
          # Create performance grade badge
          python scripts/generate_badge.py \
            --input performance_report.json \
            --output performance_badge.svg

      - name: Commit Updated Baseline
        if: github.event_name == 'schedule'
        run: |
          cp performance_report.json performance_baseline.json
          git config --local user.email "action@github.com"
          git config --local user.name "GitHub Action"
          git add performance_baseline.json performance_badge.svg
          git commit -m "Update performance baseline - $(date)"
          git push
```

**Reproducible Build Configuration**:
```toml
# .cargo/config.toml
[build]
rustflags = [
    # Reproducible builds
    "--remap-path-prefix", "/home/runner/work/=",
    "--remap-path-prefix", "/Users/runner/work/=",
    "--remap-path-prefix", "C:\\Users\\runneradmin\\work\\=",

    # Security hardening
    "-C", "relro-level=full",
    "-C", "stack-protector=strong",
]

[target.x86_64-unknown-linux-musl]
rustflags = [
    "-C", "link-arg=-static",
    "-C", "target-feature=+crt-static",
]

[target.x86_64-pc-windows-gnu]
rustflags = [
    "-C", "link-args=-static-libgcc -static-libstdc++",
]
```

**Build Health Monitoring**:
```python
# scripts/monitor_build_health.py
import requests
import json
from datetime import datetime, timedelta

class BuildHealthMonitor:
    def __init__(self, github_token):
        self.token = github_token
        self.headers = {
            'Authorization': f'token {github_token}',
            'Accept': 'application/vnd.github.v3+json'
        }

    def check_build_matrix_health(self):
        """Monitor build success rates across platforms"""

        # Get recent workflow runs
        url = "https://api.github.com/repos/rfxlamia/centotype/actions/runs"
        params = {
            'per_page': 100,
            'created': f'>{(datetime.now() - timedelta(days=7)).isoformat()}'
        }

        response = requests.get(url, headers=self.headers, params=params)
        runs = response.json()['workflow_runs']

        # Analyze success rates by platform
        platform_stats = {}
        for run in runs:
            if run['name'] == 'CI Matrix':
                platform = self.extract_platform(run)
                if platform not in platform_stats:
                    platform_stats[platform] = {'total': 0, 'success': 0}

                platform_stats[platform]['total'] += 1
                if run['conclusion'] == 'success':
                    platform_stats[platform]['success'] += 1

        # Generate alert for platforms with <90% success rate
        alerts = []
        for platform, stats in platform_stats.items():
            success_rate = stats['success'] / stats['total']
            if success_rate < 0.9:
                alerts.append(f"Platform {platform}: {success_rate:.1%} success rate")

        return alerts

    def check_performance_trends(self):
        """Monitor performance regression trends"""

        # Load recent performance reports
        reports = self.load_recent_performance_reports()

        # Calculate trends
        latency_trend = self.calculate_trend([r['p99_latency'] for r in reports])
        memory_trend = self.calculate_trend([r['memory_usage'] for r in reports])

        alerts = []
        if latency_trend > 0.05:  # >5% increase
            alerts.append(f"P99 latency trending up: +{latency_trend:.1%}")

        if memory_trend > 0.1:   # >10% increase
            alerts.append(f"Memory usage trending up: +{memory_trend:.1%}")

        return alerts

    def generate_weekly_report(self):
        """Generate comprehensive weekly build health report"""

        report = {
            'timestamp': datetime.now().isoformat(),
            'build_health': self.check_build_matrix_health(),
            'performance_trends': self.check_performance_trends(),
            'dependency_updates': self.check_dependency_freshness(),
            'security_alerts': self.check_security_advisories(),
        }

        return report

if __name__ == "__main__":
    monitor = BuildHealthMonitor(os.environ['GITHUB_TOKEN'])
    report = monitor.generate_weekly_report()

    # Send to monitoring dashboard
    print(json.dumps(report, indent=2))
```

**Artifact Management**:
```bash
#!/bin/bash
# scripts/prepare_release_artifacts.sh

set -euo pipefail

echo "=== Preparing Release Artifacts ==="

# Build matrix for all targets
TARGETS=(
    "x86_64-unknown-linux-gnu"
    "x86_64-unknown-linux-musl"
    "x86_64-apple-darwin"
    "aarch64-apple-darwin"
    "x86_64-pc-windows-msvc"
)

mkdir -p artifacts/

for target in "${TARGETS[@]}"; do
    echo "Building for target: $target"

    cargo build --target "$target" --release

    # Create platform-specific archive
    if [[ "$target" == *"windows"* ]]; then
        zip -j "artifacts/centotype-$target.zip" \
            "target/$target/release/centotype.exe"
    else
        tar -czf "artifacts/centotype-$target.tar.gz" \
            -C "target/$target/release" \
            centotype
    fi

    # Generate checksum
    if [[ "$OSTYPE" == "darwin"* ]]; then
        shasum -a 256 "artifacts/centotype-$target"* > "artifacts/centotype-$target.sha256"
    else
        sha256sum "artifacts/centotype-$target"* > "artifacts/centotype-$target.sha256"
    fi
done

echo "=== Artifacts prepared ==="
ls -la artifacts/
```

**Success Criteria**:
- ✅ Build matrix maintains >95% success rate across all platforms
- ✅ Nightly performance reports generated and stored
- ✅ Performance regression detection prevents degradation
- ✅ Reproducible builds generate consistent artifacts
- ✅ Build health monitoring alerts for issues

**Coordination Notes**:
- Monitors: All CI/CD pipeline health and performance trends
- Alerts: Platform-specific build failures and performance regressions
- Provides: Stable infrastructure for rapid development phase

### For dx-optimizer Agent [TIER 4 - SUPPORT]
**Context**: Polish CLI ergonomics and developer experience
**Critical Gap**: Command-line usability, error messages, help documentation

**Primary Objective**: Enhance command-line interface usability and error handling
- Improve CLI argument parsing with intuitive flags (--level, --from, --no-splash)
- Create clear, actionable error messages with suggestions
- Ensure consistent exit codes and deterministic help output
- Polish first-run experience and common workflow friction

**CLI Ergonomics Enhancement**:
```rust
// Enhanced CLI argument structure
use clap::{Parser, Subcommand, Args};

#[derive(Parser)]
#[command(name = "centotype")]
#[command(about = "CLI typing trainer with 100 progressive difficulty levels")]
#[command(version, long_about = None)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Commands,

    /// Disable splash screen and animations
    #[arg(long, global = true)]
    pub no_splash: bool,

    /// Enable verbose logging
    #[arg(short, long, global = true)]
    pub verbose: bool,

    /// Set custom config directory
    #[arg(long, global = true, value_name = "DIR")]
    pub config_dir: Option<PathBuf>,
}

#[derive(Subcommand)]
pub enum Commands {
    /// Start a typing session
    Play {
        /// Level to play (1-100)
        #[arg(short, long, value_parser = level_validator)]
        level: Option<u8>,

        /// Start from specific tier (1-10)
        #[arg(short = 'f', long, value_parser = tier_validator)]
        from_tier: Option<u8>,

        /// Practice specific category
        #[arg(short, long, value_enum)]
        category: Option<Category>,

        /// Time limit in seconds
        #[arg(short, long, value_name = "SECONDS")]
        time_limit: Option<u64>,

        /// Enable practice mode (no scoring)
        #[arg(long)]
        practice: bool,
    },

    /// View statistics and progress
    Stats {
        /// Show detailed breakdown
        #[arg(short, long)]
        detailed: bool,

        /// Filter by level range
        #[arg(long, value_name = "START-END")]
        level_range: Option<String>,

        /// Export to file
        #[arg(short, long, value_name = "FILE")]
        export: Option<PathBuf>,
    },

    /// Practice specific skills
    Drill {
        /// Category to practice
        #[arg(value_enum)]
        category: Category,

        /// Duration in minutes
        #[arg(short, long, default_value = "5")]
        duration: u64,

        /// Difficulty multiplier
        #[arg(long, value_name = "FACTOR", default_value = "1.0")]
        difficulty: f32,
    },

    /// Configure settings
    Config {
        #[command(subcommand)]
        action: ConfigAction,
    },

    /// Reset progress and settings
    Reset {
        /// Confirm reset without prompt
        #[arg(short, long)]
        yes: bool,

        /// Reset only statistics
        #[arg(long)]
        stats_only: bool,
    },
}

#[derive(clap::ValueEnum, Clone)]
pub enum Category {
    Symbols,
    Numbers,
    Programming,
    Punctuation,
    Mixed,
}

fn level_validator(s: &str) -> Result<u8, String> {
    let level: u8 = s.parse().map_err(|_| "Level must be a number")?;
    if level < 1 || level > 100 {
        Err("Level must be between 1 and 100".to_string())
    } else {
        Ok(level)
    }
}

fn tier_validator(s: &str) -> Result<u8, String> {
    let tier: u8 = s.parse().map_err(|_| "Tier must be a number")?;
    if tier < 1 || tier > 10 {
        Err("Tier must be between 1 and 10".to_string())
    } else {
        Ok(tier)
    }
}
```

**Enhanced Error Messages**:
```rust
// User-friendly error handling
pub fn handle_user_error(error: &CentotypeError) -> ! {
    match error {
        CentotypeError::LevelNotUnlocked(level) => {
            eprintln!("❌ Level {} is not yet unlocked", level);
            eprintln!();
            eprintln!("💡 Complete level {} first to unlock level {}", level - 1, level);
            eprintln!("   Run: centotype play --level {}", level - 1);
            process::exit(exitcode::DATAERR);
        },

        CentotypeError::ConfigNotFound(path) => {
            eprintln!("❌ Configuration file not found");
            eprintln!("   Expected: {}", path.display());
            eprintln!();
            eprintln!("💡 Create default config with: centotype config init");
            process::exit(exitcode::CONFIG);
        },

        CentotypeError::PermissionDenied(path) => {
            eprintln!("❌ Permission denied accessing: {}", path.display());
            eprintln!();
            eprintln!("💡 Try running with appropriate permissions:");
            eprintln!("   sudo chown $USER:$USER {}", path.display());
            process::exit(exitcode::NOPERM);
        },

        CentotypeError::NetworkError(url) => {
            eprintln!("❌ Network error connecting to: {}", url);
            eprintln!();
            eprintln!("💡 Check your internet connection and try again");
            eprintln!("   Run: centotype play --offline");
            process::exit(exitcode::UNAVAILABLE);
        },

        CentotypeError::PerformanceError { metric, actual, target } => {
            eprintln!("⚠️  Performance warning: {} is {}ms (target: {}ms)",
                     metric, actual, target);
            eprintln!();
            eprintln!("💡 This may affect typing experience. Consider:");
            eprintln!("   • Close other applications to free resources");
            eprintln!("   • Run: centotype play --low-performance-mode");
            process::exit(exitcode::SOFTWARE);
        },

        _ => {
            eprintln!("❌ An unexpected error occurred:");
            eprintln!("   {}", error);
            eprintln!();
            eprintln!("💡 Please report this issue at:");
            eprintln!("   https://github.com/rfxlamia/centotype/issues");
            process::exit(exitcode::SOFTWARE);
        }
    }
}

// Progress indication for first-time users
pub fn show_first_run_guidance() {
    println!("🎉 Welcome to Centotype!");
    println!();
    println!("This is your first time running Centotype. Here's how to get started:");
    println!();
    println!("📚 Begin with the basics:");
    println!("   centotype play --level 1");
    println!();
    println!("📊 Check your progress:");
    println!("   centotype stats");
    println!();
    println!("🎯 Practice specific skills:");
    println!("   centotype drill symbols --duration 5");
    println!();
    println!("⚙️  Configure settings:");
    println!("   centotype config edit");
    println!();
    println!("❓ Get help anytime:");
    println!("   centotype --help");
    println!("   centotype play --help");
    println!();
    println!("Press Enter to continue...");
    let _ = io::stdin().read_line(&mut String::new());
}
```

**Consistent Exit Codes**:
```rust
// Standard exit codes for scripting compatibility
pub mod exitcode {
    pub const OK: i32 = 0;           // Success
    pub const GENERAL: i32 = 1;      // General error
    pub const USAGE: i32 = 2;        // Misuse of shell command
    pub const DATAERR: i32 = 65;     // Data format error
    pub const NOINPUT: i32 = 66;     // Cannot open input
    pub const NOUSER: i32 = 67;      // Addressee unknown
    pub const NOHOST: i32 = 68;      // Host name unknown
    pub const UNAVAILABLE: i32 = 69; // Service unavailable
    pub const SOFTWARE: i32 = 70;    // Internal software error
    pub const OSERR: i32 = 71;       // System error
    pub const OSFILE: i32 = 72;      // Critical OS file missing
    pub const CANTCREAT: i32 = 73;   // Can't create output file
    pub const IOERR: i32 = 74;       // Input/output error
    pub const TEMPFAIL: i32 = 75;    // Temporary failure
    pub const PROTOCOL: i32 = 76;    // Protocol error
    pub const NOPERM: i32 = 77;      // Permission denied
    pub const CONFIG: i32 = 78;      // Configuration error
}

// Usage in main function
pub fn main() -> Result<()> {
    let cli = Cli::parse();

    let result = match cli.command {
        Commands::Play { level, from_tier, category, time_limit, practice } => {
            run_play_session(PlayOptions {
                level,
                from_tier,
                category,
                time_limit,
                practice,
                no_splash: cli.no_splash,
            }).await
        },
        Commands::Stats { detailed, level_range, export } => {
            show_statistics(StatsOptions {
                detailed,
                level_range,
                export,
            }).await
        },
        // ... other commands
    };

    match result {
        Ok(_) => process::exit(exitcode::OK),
        Err(e) => handle_user_error(&e),
    }
}
```

**Help Documentation Enhancement**:
```rust
// Context-sensitive help
pub fn show_contextual_help(context: HelpContext) {
    match context {
        HelpContext::FirstTime => {
            println!("🎯 Centotype Quick Start Guide");
            println!();
            println!("Centotype is a progressive typing trainer with 100 levels.");
            println!("Each level increases difficulty gradually to build your skills.");
            println!();
            println!("Basic Commands:");
            println!("  centotype play           Start from your current level");
            println!("  centotype play -l 5      Play specific level");
            println!("  centotype stats          View your progress");
            println!("  centotype drill symbols  Practice symbols");
            println!();
            println!("During typing:");
            println!("  ESC     Quit session");
            println!("  F1      Help overlay");
            println!("  Ctrl+R  Restart level");
            println!("  Tab     View stats");
        },

        HelpContext::InGame => {
            println!("⌨️  In-Game Commands");
            println!();
            println!("Navigation:");
            println!("  ESC        Quit and save progress");
            println!("  Ctrl+C     Force quit (progress lost)");
            println!("  Ctrl+R     Restart current level");
            println!("  F1         Toggle this help overlay");
            println!();
            println!("Information:");
            println!("  Tab        Show detailed statistics");
            println!("  F2         Show target text preview");
            println!("  F3         Toggle difficulty hints");
            println!();
            println!("Performance:");
            println!("  F10        Performance debug overlay");
            println!("  F11        Toggle low-performance mode");
        },

        HelpContext::Configuration => {
            println!("⚙️  Configuration Options");
            println!();
            println!("centotype config init     Create default configuration");
            println!("centotype config edit     Edit configuration file");
            println!("centotype config show     Display current settings");
            println!("centotype config reset    Reset to defaults");
            println!();
            println!("Configuration file location:");
            let config_path = get_config_path();
            println!("  {}", config_path.display());
        },
    }
}
```

**Performance and Accessibility Options**:
```rust
// Low-performance mode for resource-constrained environments
pub struct PerformanceOptions {
    pub reduced_animations: bool,
    pub simple_rendering: bool,
    pub lower_refresh_rate: bool,
    pub minimal_colors: bool,
}

impl PerformanceOptions {
    pub fn low_performance() -> Self {
        Self {
            reduced_animations: true,
            simple_rendering: true,
            lower_refresh_rate: true,
            minimal_colors: false, // Maintain accessibility
        }
    }

    pub fn accessibility_mode() -> Self {
        Self {
            reduced_animations: true,
            simple_rendering: false,
            lower_refresh_rate: false,
            minimal_colors: true, // High contrast only
        }
    }
}

// Command line flags for accessibility
#[derive(Args)]
pub struct AccessibilityArgs {
    /// Enable high contrast mode
    #[arg(long)]
    pub high_contrast: bool,

    /// Use screen reader friendly output
    #[arg(long)]
    pub screen_reader: bool,

    /// Disable animations
    #[arg(long)]
    pub no_animations: bool,

    /// Use large text mode
    #[arg(long)]
    pub large_text: bool,
}
```

**Success Criteria**:
- ✅ Intuitive CLI with discoverable commands and flags
- ✅ Clear error messages with actionable suggestions
- ✅ Consistent exit codes for script integration
- ✅ Context-sensitive help documentation
- ✅ First-run experience guides new users
- ✅ Accessibility options for diverse users

**Coordination Notes**:
- Enhances: User experience for all agent deliverables
- Integrates: With ui-ux-designer for consistent experience
- Reviews: With code-reviewer for error handling patterns

### For docs-architect Agent [TIER 4 - SUPPORT]
**Context**: Update documentation to reflect integrated system
**Critical Gap**: Documentation must match actual implementation with engine integration

**Primary Objective**: Synchronize all documentation with engine-integrated system
- Update README.md with accurate feature descriptions and performance targets
- Sync ARCHITECTURE.md with current inter-crate communication patterns
- Revise CONTENT_GUIDE.md for production-ready content system
- Ensure troubleshooting guides match actual error conditions

**Documentation Synchronization Plan**:
```markdown
# Documentation Update Checklist

## README.md Updates
- [ ] Update project status: "CLI skeleton" → "Full typing trainer"
- [ ] Correct performance metrics: Current P99 latency, memory usage, startup time
- [ ] Add actual command examples with real output
- [ ] Update installation instructions for integrated system
- [ ] Add performance grade badge (current: B+, target: A)

## ARCHITECTURE.md Updates
- [ ] Document actual trait boundaries between crates
- [ ] Update data flow diagrams with engine integration
- [ ] Add performance bottleneck analysis and optimization plans
- [ ] Include ADR references for frozen contracts

## CONTENT_GUIDE.md Updates
- [ ] Mark content generation system as production-ready
- [ ] Update cache performance metrics (94% hit rate)
- [ ] Add security validation documentation
- [ ] Include content testing and validation procedures

## New Documentation Needs
- [ ] ENGINE_INTEGRATION.md - Technical guide for typing loop
- [ ] TROUBLESHOOTING.md - Common issues and solutions
- [ ] PERFORMANCE_TUNING.md - Optimization guide
- [ ] TESTING_GUIDE.md - Test suite documentation
```

**Updated README.md Structure**:
```markdown
# Centotype - Progressive CLI Typing Trainer

[![Build Status](https://github.com/rfxlamia/centotype/workflows/CI/badge.svg)](https://github.com/rfxlamia/centotype/actions)
[![Performance Grade](https://img.shields.io/badge/Performance-B%2B-yellow)](docs/performance/PERFORMANCE_VALIDATION_REPORT.md)
[![Coverage](https://img.shields.io/badge/Coverage-75%25-orange)](docs/development/TESTING_GUIDE.md)

**Status**: ✅ **Phase 1 Complete** - Full typing trainer with engine integration

Centotype is a CLI-based typing trainer featuring 100 progressive difficulty levels, deterministic scoring, and sub-25ms input latency. Built in Rust with a modular 7-crate architecture for performance and maintainability.

## Features

### ✅ Completed
- **100-Level Progression System**: Mathematical difficulty scaling from 5% to 30% symbols
- **Real-time Typing Interface**: Input capture, scoring, and visual feedback
- **Performance Optimized**: P99 input latency 28ms (target: <25ms), 94% cache hit rate
- **Content Generation**: Deterministic corpus with security validation
- **Cross-platform Support**: Linux, macOS, Windows with native terminal integration
- **Analytics & Progress**: Keystroke analysis, WPM calculation, error classification

### 🚧 In Progress
- **Performance Grade A**: Optimizing from B+ (28ms) to <25ms P99 latency
- **Extended Testing**: Cross-platform validation and edge case handling

## Installation

### Quick Start
```bash
# Build from source
git clone https://github.com/rfxlamia/centotype.git
cd centotype
cargo build --release

# Run first session
./target/release/centotype play --level 1
```

### Package Managers (Coming Soon)
```bash
# Cargo
cargo install centotype

# NPM
npm install -g centotype

# Homebrew
brew install centotype
```

## Usage

### Basic Commands
```bash
# Start typing session
centotype play                    # Continue from current level
centotype play --level 5          # Play specific level
centotype play --from-tier 3      # Start from tier 3

# View progress
centotype stats                   # Overview
centotype stats --detailed        # Detailed breakdown

# Practice skills
centotype drill symbols           # Symbol practice
centotype drill programming       # Code-focused practice

# Configuration
centotype config edit             # Edit settings
centotype reset --stats-only      # Reset progress
```

### Example Session
```
$ centotype play --level 5

┌─────────────────────────────────────────────────────────────┐
│ Centotype CLI - Level 5 (Tier 1) - Programming Basics      │
├─────────────────────────────────────────────────────────────┤
│ function calculateSum(arr) {                                │
│     return arr.reduce((a,b) => a+b, 0);                    │
│ }                                                           │
│                                                             │
│ Input: function calculateSum(█                              │
├─────────────────────────────────────────────────────────────┤
│ WPM: 45 │ ACC: 94.2% │ COMBO: x12 │ ⏱️  1:23 │ ERR: 3     │
├─────────────────────────────────────────────────────────────┤
│ Progress: ████████████░░░░░░░░ 65% (195/300 chars)         │
└─────────────────────────────────────────────────────────────┘
```

## Performance

**Current Metrics** (Target vs Actual):
- **Input Latency**: <25ms P99 (28ms current) - 🟡 Optimization needed
- **Startup Time**: <200ms P95 (180ms current) - ✅ Exceeds target
- **Memory Usage**: <50MB (46MB current) - ✅ Within target
- **Cache Hit Rate**: >90% (94% current) - ✅ Exceeds target

**Performance Grade**: B+ → A (optimization in progress)

See [Performance Guide](docs/performance/PERFORMANCE_GUIDE.md) for detailed analysis.

## Architecture

7-crate modular workspace optimized for performance:

```
centotype/
├── core/           # Scoring engine, session state, level progression
├── engine/         # ✅ Input handling, TTY, render loop [INTEGRATED]
├── content/        # ✅ Text corpus, LRU cache, content generation
├── analytics/      # Performance metrics, error classification
├── cli/            # Command parsing, interactive navigation
├── persistence/    # Profile storage, configuration management
├── platform/       # OS-specific terminal detection and handling
└── centotype-bin/  # Main application binary
```

**Inter-Crate Communication**:
- `content/` → `core/` → `engine/` → `cli/` (main data flow)
- `analytics/` ← `engine/` (metrics collection)
- `persistence/` ← `engine/` (async session storage)

See [Architecture Guide](docs/architecture/ARCHITECTURE.md) for complete details.

## Development

### Quick Validation
```bash
# Fast development cycle
./scripts/validate_local.sh --quick

# Full validation suite
./scripts/validate_local.sh

# Performance benchmarking
cargo bench --bench input_latency_benchmark
```

### Testing
```bash
# All tests
cargo test --workspace --all-features

# Specific crates
cargo test -p centotype-core
cargo test -p centotype-engine
cargo test -p centotype-content

# Performance validation
cargo test --release performance_tests
```

See [Development Guide](docs/development/README.md) for complete workflow.
```

**New Technical Documentation**:

1. **ENGINE_INTEGRATION.md**:
```markdown
# Engine Integration Technical Guide

## Overview
The engine crate serves as the central coordination point for real-time typing sessions, managing the complete input→processing→rendering cycle.

## Core Integration Points

### Typing Loop Architecture
```rust
pub async fn run(&mut self, mode: TrainingMode, target_text: String) -> Result<SessionResult> {
    // 1. Session initialization
    let content = self.content_loader.load_level_content(mode.level_id()).await?;
    self.core.start_session(mode, content.clone())?;

    // 2. TTY setup with cleanup guards
    let _tty_guard = TTYGuard::new(&mut self.terminal)?;

    // 3. Main event loop
    loop {
        // Render current state (target: <33ms P95)
        self.render_frame(&self.core.current_state())?;

        // Input capture (target: <5ms)
        if let Some(event) = self.input_handler.poll_event(Duration::from_millis(10))? {
            // Processing & scoring (target: <5ms)
            let result = self.process_input_event(event)?;

            // Session completion check
            if result.is_complete() {
                return Ok(result);
            }
        }
    }
}
```

### Performance Constraints
- **Total Input Latency**: <25ms P99 (keystroke → visual feedback)
- **Render Frame Time**: <33ms P95
- **Memory Allocation**: Zero in hot path (use pre-allocated buffers)
- **Content Loading**: <5ms via LRU cache (94% hit rate)

### Error Handling Patterns
All engine operations use `Result<T, anyhow::Error>` with context:
```rust
self.core.process_keystroke(key)
    .context("Failed to process keystroke in scoring engine")?;

self.analytics.record_keystroke(key, result)
    .context("Failed to record keystroke analytics")?;
```

### TTY Safety
Critical: Terminal state must be restored on ALL exit paths:
```rust
pub struct TTYGuard<'a> {
    terminal: &'a mut Terminal,
}

impl<'a> Drop for TTYGuard<'a> {
    fn drop(&mut self) {
        // Always restore, even on panic
        let _ = self.terminal.leave_alternate_screen();
        let _ = self.terminal.leave_raw_mode();
        let _ = write!(io::stdout(), "\x1b[?25h"); // Show cursor
    }
}
```
```

2. **TROUBLESHOOTING.md**:
```markdown
# Troubleshooting Guide

## Common Issues

### Performance Issues

#### High Input Latency (>25ms)
**Symptoms**: Delayed character appearance, sluggish response
**Diagnosis**:
```bash
# Run latency benchmark
cargo bench --bench input_latency_benchmark

# Check system resources
cargo run --bin performance-monitor
```
**Solutions**:
1. Close resource-intensive applications
2. Enable low-performance mode: `centotype play --low-performance-mode`
3. Check terminal compatibility: some terminals have higher latency

#### Memory Usage High (>50MB)
**Symptoms**: System slowdown, swap usage
**Diagnosis**:
```bash
# Memory profiler
cargo run --bin memory-profiler
```
**Solutions**:
1. Clear cache: `centotype config reset-cache`
2. Reduce content preloading: `centotype config set preload-levels 1`

### Terminal Issues

#### Corrupted Terminal State
**Symptoms**: Terminal doesn't respond after quit, cursor missing
**Solutions**:
```bash
# Reset terminal manually
reset
# Or
tput sgr0; tput cnorm
```
**Prevention**: Use `centotype play` instead of Ctrl+C to quit

#### Display Artifacts
**Symptoms**: Overlapping text, incorrect colors
**Causes**: Terminal compatibility issues
**Solutions**:
1. Try different terminal: iTerm2, Alacritty, Windows Terminal
2. Force simple rendering: `centotype play --simple-render`
3. Check terminal size: minimum 80x24 required

### Content Loading Issues

#### Cache Misses / Slow Loading
**Symptoms**: Pauses before level start, loading indicators
**Diagnosis**:
```bash
# Check cache statistics
centotype stats --cache-info
```
**Solutions**:
1. Warm up cache: `centotype preload --levels 1-10`
2. Check disk space in cache directory
3. Verify content generation: `cargo test -p centotype-content`

### Configuration Issues

#### Settings Not Persisting
**Symptoms**: Configuration resets between sessions
**Diagnosis**:
```bash
# Check config file permissions
ls -la $(centotype config path)
```
**Solutions**:
1. Fix permissions: `chmod 644 $(centotype config path)`
2. Verify config directory writable
3. Reset config: `centotype config init --force`

## Diagnostic Commands

### Performance Analysis
```bash
# Complete performance report
centotype diagnose --performance

# Latency distribution
centotype diagnose --latency --samples 1000

# Memory usage over time
centotype diagnose --memory --duration 60s
```

### System Information
```bash
# Environment details
centotype diagnose --system

# Terminal compatibility
centotype diagnose --terminal

# Build information
centotype --version --verbose
```

## Getting Help

### Debug Logs
Enable detailed logging:
```bash
RUST_LOG=debug centotype play --level 1 2>debug.log
```

### Issue Reporting
Include this information when reporting issues:
```bash
# System info
centotype diagnose --system > system-info.txt

# Performance report
centotype diagnose --performance > performance-report.txt

# Configuration
centotype config show > config-dump.txt
```

Submit to: https://github.com/rfxlamia/centotype/issues
```

**Success Criteria**:
- ✅ README.md accurately reflects integrated system functionality
- ✅ ARCHITECTURE.md matches current inter-crate communication
- ✅ All documentation references correct file paths and performance metrics
- ✅ Troubleshooting guide covers actual reported issues
- ✅ API documentation matches implemented interfaces

**Coordination Notes**:
- Syncs: With all agents to reflect actual implementation
- Validates: Documentation accuracy against running system
- Updates: Performance metrics and feature status in real-time

### For tutorial-engineer Agent [TIER 4 - SUPPORT]
**Context**: Update learning materials for integrated typing trainer
**Critical Gap**: Tutorials must reflect actual system functionality, not placeholder implementation

**Primary Objective**: Create accurate quickstart and learning materials for production system
- Update "3-minute first experience" quickstart guide
- Revise T1-T3 15-minute practice guide for actual difficulty progression
- Create troubleshooting scenarios based on real user interactions
- Ensure all tutorial examples work with integrated engine

**Updated Quickstart Guide**:
```markdown
# Centotype Quick Start - 3 Minutes to First Session

## What You'll Learn
- How to start your first typing session
- Understanding the interface and controls
- Basic navigation and progress tracking
- Performance optimization tips

**Time Required**: 3 minutes
**Prerequisites**: Centotype installed and built

## Step 1: First Launch (30 seconds)

Open your terminal and run:
```bash
./target/release/centotype play --level 1
```

You'll see the typing interface:
```
┌─────────────────────────────────────────────────────────────┐
│ Centotype CLI - Level 1 (Tier 1) - Foundation              │
├─────────────────────────────────────────────────────────────┤
│ Target: The quick brown fox jumps over the lazy dog.       │
│         Practice makes perfect typing skills develop.       │
│                                                             │
│ Input:  █                                                   │
├─────────────────────────────────────────────────────────────┤
│ WPM: 0  │ ACC: 100% │ COMBO: x0  │ ⏱️  0:00 │ ERR: 0      │
├─────────────────────────────────────────────────────────────┤
│ Progress: ░░░░░░░░░░░░░░░░░░░░░░░░ 0% (0/120 chars)         │
└─────────────────────────────────────────────────────────────┘
```

## Step 2: Start Typing (1 minute)

1. **Type the target text**: Start typing the text shown in the Target section
2. **Watch real-time feedback**:
   - ✅ Green = correct characters
   - ❌ Red = incorrect characters
   - 📍 Yellow cursor shows current position
3. **Monitor your stats**:
   - **WPM**: Words per minute (updates in real-time)
   - **ACC**: Accuracy percentage
   - **COMBO**: Consecutive correct keystrokes
   - **ERR**: Error count

**Try this**: Type "The quick brown" and watch your stats update!

## Step 3: Complete Your First Level (1 minute)

1. **Finish the text**: Complete typing all target text
2. **Session results**: You'll see a completion summary:
```
🎉 Level 1 Complete!

📊 Session Results:
   • Final WPM: 32
   • Accuracy: 94.2%
   • Time: 1:23
   • Errors: 3 (2 substitutions, 1 insertion)

📈 Progress:
   • Level 2 unlocked!
   • Tier 1: 10% complete

🎯 Next Steps:
   • Continue to Level 2: centotype play --level 2
   • View detailed stats: centotype stats
   • Practice symbols: centotype drill symbols
```

## Step 4: Navigation Essentials (30 seconds)

**During typing**:
- `ESC` - Save progress and quit
- `F1` - Help overlay
- `Ctrl+R` - Restart current level
- `Tab` - Show detailed statistics

**Between sessions**:
```bash
centotype stats              # View your progress
centotype play --level 2     # Continue to next level
centotype drill symbols      # Practice specific skills
centotype --help            # Complete command reference
```

## Next Steps

✅ **You're ready to continue!**

- **Continue progression**: `centotype play` (continues from your highest level)
- **Follow the 15-minute guide**: See [T1-T3 Practice Guide](T1_T3_PRACTICE_GUIDE.md)
- **Customize settings**: `centotype config edit`

**Need help?** See [Troubleshooting Guide](../TROUBLESHOOTING.md)
```

**T1-T3 Practice Guide (Updated)**:
```markdown
# T1-T3 Mastery Guide - 15 Minutes to Tier 3

## Overview
Master the first 3 tiers (Levels 1-30) with focused practice sessions. Each tier introduces new challenges while building on previous skills.

**Time Investment**: 15 minutes focused practice
**Skill Level**: Beginner to Intermediate
**Expected Outcome**: Consistent 40+ WPM with 95%+ accuracy in Tier 3

## Tier 1 (Levels 1-10): Foundation - 5 minutes

### Skills Developed
- Basic alphabetic typing
- Simple punctuation (. , ! ?)
- Common English words
- Muscle memory for letter positions

### Practice Session
```bash
# Start Tier 1 progression
centotype play --from-tier 1

# Focus areas for improvement
centotype drill punctuation --duration 2
```

### Target Performance
- **WPM**: 25-35
- **Accuracy**: >92%
- **Common errors**: Letter substitutions, rhythm breaks

### Sample Text (Level 5)
```
The quick brown fox jumps over the lazy dog. Practice
makes perfect when you focus on accuracy first. Speed
will follow naturally as you build muscle memory.
```

**💡 Tips**:
- Focus on accuracy over speed initially
- Use all fingers, avoid "hunt and peck"
- Take breaks if accuracy drops below 90%

## Tier 2 (Levels 11-20): Complexity - 5 minutes

### New Skills Introduced
- Numbers in context (dates, simple math)
- Programming punctuation: () {} []
- Mixed case patterns
- Basic technical terms

### Practice Session
```bash
# Continue Tier 2 progression
centotype play --from-tier 2

# Strengthen number typing
centotype drill numbers --duration 2

# Mixed punctuation practice
centotype drill programming --duration 1
```

### Target Performance
- **WPM**: 30-40
- **Accuracy**: >93%
- **Common errors**: Number placement, bracket matching

### Sample Text (Level 15)
```
function calculateAge(birthYear) {
    const currentYear = 2025;
    return currentYear - birthYear;
}
// Expected result: 25 for birth year 2000
```

**💡 Tips**:
- Learn number row without looking
- Practice bracket pairs: () {} []
- Maintain accuracy when switching between letters and numbers

## Tier 3 (Levels 21-30): Programming - 5 minutes

### Advanced Skills
- Programming symbols: + - * / = < >
- camelCase and snake_case patterns
- Code documentation style
- Multi-language switching (English/Indonesian)

### Practice Session
```bash
# Complete Tier 3 mastery
centotype play --from-tier 3

# Symbol-heavy practice
centotype drill symbols --duration 3
```

### Target Performance
- **WPM**: 35-45
- **Accuracy**: >94%
- **Common errors**: Symbol placement, case switching

### Sample Text (Level 25)
```
const userConfig = {
    maxRetries: 3,
    timeout: 5000,
    enableLogging: true,
    apiUrl: "https://api.example.com/v1"
};

// Konfigurasi pengguna untuk sistem logging
if (userConfig.enableLogging) {
    console.log("Logging enabled");
}
```

**💡 Tips**:
- Master symbol placement without looking
- Practice switching between camelCase and snake_case
- Build familiarity with code patterns

## Assessment & Next Steps

### Self-Assessment Checklist
After completing all 3 tiers:

**Tier 1 Mastery** ✅❌
- [ ] Can type common English words at 35+ WPM
- [ ] Punctuation (. , ! ?) feels natural
- [ ] Accuracy consistently >92%

**Tier 2 Mastery** ✅❌
- [ ] Numbers integrated smoothly with text
- [ ] Basic programming punctuation () {} [] comfortable
- [ ] Mixed case doesn't slow down typing

**Tier 3 Mastery** ✅❌
- [ ] Programming symbols + - * / = accessible
- [ ] Code patterns (camelCase, snake_case) familiar
- [ ] Can switch between English/Indonesian smoothly

### Performance Validation
```bash
# Check your tier mastery
centotype stats --detailed --level-range 1-30

# Expected results:
# Tier 1 Average: 32+ WPM, 92%+ accuracy
# Tier 2 Average: 37+ WPM, 93%+ accuracy
# Tier 3 Average: 42+ WPM, 94%+ accuracy
```

### Progression Paths

**🚀 Ready for Tier 4-6 (Levels 31-60)**:
- Advanced symbol combinations
- Nested programming structures
- Technical documentation patterns

**📚 Need More Practice**:
```bash
# Focused improvement sessions
centotype drill [weakness] --duration 10

# Repeat challenging levels
centotype play --level [difficult_level] --practice
```

**🎯 Optimization Focus**:
- **Speed Building**: Target 50+ WPM in comfortable content
- **Accuracy Refinement**: Aim for 96%+ consistency
- **Symbol Mastery**: Practice advanced programming patterns

## Performance Troubleshooting

### Common Issues & Solutions

**Accuracy Dropping in Tier 2+**:
- Slow down and focus on precision
- Practice isolated difficult patterns
- Use `centotype play --practice` mode for low-pressure practice

**WPM Plateau**:
- Ensure proper finger positioning
- Practice rhythm with metronome apps
- Focus on common word patterns

**Symbol Confusion**:
- Create muscle memory with dedicated drills
- Practice symbol sequences: += -= *= /=
- Use `centotype drill symbols --difficulty 1.5`

**Need Help?** See [Troubleshooting Guide](../TROUBLESHOOTING.md) or run `centotype diagnose --performance`
```

**Interactive Learning Scenarios**:
```markdown
# Interactive Learning Scenarios

## Scenario 1: "My First Mistake"
**Learning Goal**: Understand error handling and correction strategies

**Setup**: Start Level 3, intentionally make errors
```bash
centotype play --level 3
```

**What to Do**:
1. Type the first sentence correctly
2. Make an intentional substitution error (type 'x' instead of 's')
3. Observe the red highlighting and error counter
4. Continue typing without correcting (Centotype handles this automatically)
5. Complete the level and review error analysis

**Learning Points**:
- Errors are highlighted in real-time
- No backspace needed - continue typing for best flow
- Error analysis helps identify patterns
- Accuracy matters more than speed initially

## Scenario 2: "Speed vs Accuracy Balance"
**Learning Goal**: Find optimal typing rhythm

**Setup**: Practice same level with different approaches
```bash
centotype play --level 10 --practice  # No scoring pressure
```

**Exercise**:
1. **Speed Focus**: Type as fast as possible, ignore accuracy
2. **Accuracy Focus**: Type slowly, aim for 100% accuracy
3. **Balanced**: Find middle ground - steady rhythm with high accuracy

**Metrics to Watch**:
- Speed-focused: High WPM, low accuracy, high error count
- Accuracy-focused: Low WPM, high accuracy, few errors
- Balanced: Moderate WPM (30-40), high accuracy (94%+), controlled errors

**Target**: 35+ WPM with 94%+ accuracy

## Scenario 3: "Symbol Integration Challenge"
**Learning Goal**: Smooth integration of programming symbols

**Setup**: Progressive symbol introduction
```bash
centotype drill symbols --duration 5
```

**Practice Progression**:
1. **Level A**: Basic math symbols (+, -, *, /)
2. **Level B**: Comparison operators (<, >, =, !)
3. **Level C**: Grouping symbols (, ), [, ], {, }
4. **Level D**: Combined patterns: +=, -=, ==, !=

**Success Metrics**:
- Symbols don't cause hesitation or slowdown
- Accuracy maintained when symbols appear
- Smooth transitions: letter → symbol → letter

## Scenario 4: "Multi-Language Context"
**Learning Goal**: Handle English/Indonesian switching

**Setup**: Higher tier levels with mixed languages
```bash
centotype play --level 25  # Includes mixed language content
```

**Practice Focus**:
- Smooth mental switching between languages
- Maintain typing rhythm during language transitions
- Handle mixed punctuation and capitalization

**Common Challenges**:
- Indonesian words may feel unfamiliar
- Different sentence structures
- Mixed technical terms

**Tips**:
- Treat unfamiliar words as typing patterns
- Focus on character-by-character accuracy
- Build familiarity through repetition
```

**Success Criteria**:
- ✅ Quickstart guide works with actual integrated system
- ✅ All tutorial examples run successfully on real implementation
- ✅ Performance targets match current system capabilities
- ✅ Troubleshooting scenarios address real user issues
- ✅ Learning progression aligns with mathematical difficulty formulas

**Coordination Notes**:
- Validates: All tutorial examples against working system
- Updates: Based on dx-optimizer ergonomics improvements
- References: Actual performance metrics from performance-engineer

## Summary & Next Steps

The Enhanced Prompt Kit v2 provides comprehensive coordination for Week 3-4 close-out:

1. **Tier 1 Blockers**: rust-pro engine integration, backend-architect contracts, test-automator fixes
2. **Tier 2 Core**: ui-ux-designer interface, performance-engineer optimization
3. **Tier 3 Quality**: debugger diagnosis, code-reviewer standards, security-auditor validation
4. **Tier 4 Support**: DevOps stability, DX polish, documentation sync, tutorial updates

**Master coordination completed**. Ready to deploy specialized agents with enhanced prompts.
- Ensure content/ → core/ → engine/ data flow <25ms
- Design content preloading strategy for next 3 levels
- Implement graceful degradation for cache misses
- Optimize cross-crate communication patterns

### For security-auditor Agent
**Context**: Comprehensive security validation of corpus content
**Requirements**:
- Implement terminal escape sequence detection
- Add malicious pattern recognition (injection attempts)
- Validate Unicode character safety
- Create security test suite for content validation

### For performance-engineer Agent
**Context**: Benchmark system against P99 latency targets
**Requirements**:
- Measure content generation latency distribution
- Benchmark cache hit/miss performance
- Profile memory usage during content loading
- Validate <25ms P99 input latency with content system

### For test-automator Agent
**Context**: Comprehensive testing for content system
**Requirements**:
- Unit tests for deterministic generation
- Integration tests for cache performance
- Snapshot testing for content consistency
- Property-based testing for security validation

### For docs-architect Agent
**Context**: User and developer documentation
**Requirements**:
- Document content generation API
- Create troubleshooting guide for content issues
- Write quickstart for custom content development
- Explain difficulty progression system

## Validation System Implementation

### Content Validation Pipeline
```rust
pub struct ContentValidator {
    security_validator: SecurityValidator,
    difficulty_validator: DifficultyValidator,
    performance_validator: PerformanceValidator,
}

impl ContentValidator {
    pub fn validate_generated_content(
        &self,
        content: &str,
        level_id: LevelId
    ) -> ValidationResult {
        // 1. Security validation
        self.security_validator.validate(content)?;

        // 2. Difficulty progression validation
        self.difficulty_validator.validate(content, level_id)?;

        // 3. Performance impact validation
        self.performance_validator.validate(content)?;

        ValidationResult::Valid
    }
}
```

### Progressive Difficulty Verification
```rust
pub fn verify_difficulty_progression(contents: &[String]) -> Result<()> {
    for (i, content) in contents.iter().enumerate() {
        let current_difficulty = calculate_difficulty_score(content);
        if i > 0 {
            let prev_difficulty = calculate_difficulty_score(&contents[i-1]);
            let increase = (current_difficulty - prev_difficulty) / prev_difficulty;

            // Ensure 3-7% difficulty increase per level
            assert!(increase >= 0.03 && increase <= 0.07,
                "Level {} difficulty increase {:.2}% outside target range",
                i + 1, increase * 100.0);
        }
    }
    Ok(())
}
```

### Cache Performance Monitoring
```rust
pub struct CacheMetrics {
    hit_rate: f64,
    avg_lookup_time: Duration,
    memory_usage: usize,
    eviction_count: u64,
}

impl CacheMetrics {
    pub fn validate_performance_targets(&self) -> Result<()> {
        assert!(self.hit_rate >= 0.90, "Cache hit rate too low: {:.2}%", self.hit_rate * 100.0);
        assert!(self.avg_lookup_time.as_millis() < 5, "Cache lookup too slow: {}ms", self.avg_lookup_time.as_millis());
        Ok(())
    }
}
```

## Usage Instructions

### For Content Generation
1. Use the master prompt template with specific level parameters
2. Apply security validation to all generated content
3. Verify difficulty progression meets mathematical formulas
4. Test cache integration and performance targets
5. Document any edge cases or special handling required

### For Agent Coordination
1. Each agent should use their specific context section
2. Implement required traits and interfaces as specified
3. Coordinate with other agents through well-defined APIs
4. Validate integration points through comprehensive testing
5. Maintain performance targets throughout development

### Success Validation
- All 100 levels generate successfully with unique, appropriate content
- Content loading consistently meets <25ms P99 target
- Security validation passes 100% of test cases
- Difficulty progression shows measurable, consistent improvement
- Cross-platform compatibility verified on Linux/macOS/Windows

This master prompt system ensures consistent, secure, and performant content generation for the Centotype typing trainer while enabling effective coordination between specialized development agents.