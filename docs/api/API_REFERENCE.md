# Centotype API Reference

> **Complete API documentation for all public interfaces across the 7-crate system**

This reference provides comprehensive documentation for all public APIs in Centotype, organized by crate and functionality. Each API includes usage examples, error handling patterns, and performance considerations.

## Table of Contents

1. [Content Management APIs](#content-management-apis)
2. [Core Engine APIs](#core-engine-apis)
3. [Platform APIs](#platform-apis)
4. [CLI APIs](#cli-apis)
5. [Analytics APIs](#analytics-apis)
6. [Persistence APIs](#persistence-apis)
7. [Types and Data Structures](#types-and-data-structures)
8. [Error Handling](#error-handling)
9. [Performance Monitoring](#performance-monitoring)
10. [Configuration APIs](#configuration-apis)

---

## Content Management APIs

### ContentManager

The primary interface for content generation, caching, and management.

```rust
use centotype_content::ContentManager;

pub struct ContentManager {
    // Internal implementation
}
```

#### Core Methods

##### `new() -> Result<Self>`

Creates a new ContentManager with default configuration.

```rust
// Example: Basic initialization
let manager = ContentManager::new().await?;
```

**Returns**: `Result<ContentManager>`
**Performance**: ~50ms initialization time
**Errors**: `ContentError::InitializationFailed`

---

##### `with_config(config: ContentConfig) -> Result<Self>`

Creates ContentManager with custom configuration.

```rust
// Example: Custom configuration
let config = ContentConfig {
    enable_preloading: true,
    preload_strategy: PreloadStrategy::Adaptive,
    cache_config: CacheConfig {
        max_items: 75,
        soft_limit_bytes: 15 * 1024 * 1024,
        hard_limit_bytes: 20 * 1024 * 1024,
        preload_count: 5,
        enable_background_eviction: true,
    },
    difficulty_config: DifficultyConfig::default(),
    enable_validation: true,
    default_seed: Some(12345),
};

let manager = ContentManager::with_config(config).await?;
```

**Parameters**:
- `config`: `ContentConfig` - Configuration settings
**Returns**: `Result<ContentManager>`
**Performance**: ~60ms with custom config
**Errors**: `ContentError::ConfigurationInvalid`

---

##### `get_level_content(level_id: LevelId, seed: Option<u64>) -> Result<String>`

Primary method for retrieving content for a specific level.

```rust
// Example: Get content for level 5
let level = LevelId::new(5)?;
let content = manager.get_level_content(level, None).await?;
println!("Level 5 content: {}", content);

// Example: Deterministic content with seed
let content = manager.get_level_content(level, Some(12345)).await?;
```

**Parameters**:
- `level_id`: `LevelId` - Target level (1-100)
- `seed`: `Option<u64>` - Optional seed for deterministic generation
**Returns**: `Result<String>` - Generated content
**Performance**:
  - Cache hit: <2ms P99
  - Cache miss: <25ms P99
**Errors**:
  - `ContentError::InvalidLevel`
  - `ContentError::GenerationFailed`
  - `ContentError::CacheError`

---

##### `get_cached_content(level_id: LevelId, seed: Option<u64>) -> Option<String>`

Retrieves content from cache only, no generation on miss.

```rust
// Example: Check cache without generation
let level = LevelId::new(10)?;
if let Some(content) = manager.get_cached_content(level, None).await {
    println!("Found cached content");
} else {
    println!("Cache miss - would need generation");
}
```

**Parameters**:
- `level_id`: `LevelId` - Target level
- `seed`: `Option<u64>` - Optional seed
**Returns**: `Option<String>` - Cached content if available
**Performance**: <1ms response time
**Thread Safety**: Safe for concurrent access

---

##### `preload_upcoming_levels(current_level: LevelId) -> Result<()>`

Preloads content for upcoming levels based on strategy.

```rust
// Example: Preload content around current level
let current_level = LevelId::new(15)?;
manager.preload_upcoming_levels(current_level).await?;
```

**Parameters**:
- `current_level`: `LevelId` - Current user level
**Returns**: `Result<()>`
**Performance**: Background operation, <5ms overhead
**Behavior**: Strategy-dependent (Sequential, Adaptive, UserHistory)

---

##### `invalidate_level(level_id: LevelId, seed: Option<u64>)`

Removes specific content from cache.

```rust
// Example: Force regeneration by invalidating cache
let level = LevelId::new(7)?;
manager.invalidate_level(level, None).await;
let fresh_content = manager.get_level_content(level, None).await?;
```

**Parameters**:
- `level_id`: `LevelId` - Level to invalidate
- `seed`: `Option<u64>` - Optional seed
**Returns**: `()` (always succeeds)
**Performance**: <1ms operation

---

##### `clear_cache()`

Clears all cached content.

```rust
// Example: Clear cache for memory management
manager.clear_cache().await;
```

**Returns**: `()`
**Performance**: <10ms operation
**Memory Impact**: Frees all cache memory

---

##### `get_cache_metrics() -> CacheMetrics`

Returns current cache performance metrics.

```rust
// Example: Monitor cache performance
let metrics = manager.get_cache_metrics();
println!("Cache hit rate: {:.1}%", metrics.hit_rate() * 100.0);
println!("Memory usage: {:.1}MB", metrics.memory_usage_bytes as f64 / 1024.0 / 1024.0);
```

**Returns**: `CacheMetrics`
**Performance**: <1ms operation

```rust
pub struct CacheMetrics {
    pub hit_count: u64,
    pub miss_count: u64,
    pub memory_usage_bytes: u64,
    pub avg_generation_time: Duration,
    pub p99_access_time: Duration,
    pub preload_efficiency: f64,
}

impl CacheMetrics {
    pub fn hit_rate(&self) -> f64 {
        if self.hit_count + self.miss_count == 0 {
            0.0
        } else {
            self.hit_count as f64 / (self.hit_count + self.miss_count) as f64
        }
    }
}
```

---

##### `analyze_difficulty(content: &str) -> DifficultyScore`

Analyzes the difficulty characteristics of content.

```rust
// Example: Analyze content difficulty
let content = "function fibonacci(n) { return n < 2 ? n : fibonacci(n-1) + fibonacci(n-2); }";
let difficulty = manager.analyze_difficulty(content);

println!("Overall difficulty: {:.3}", difficulty.overall);
println!("Symbol density: {:.3}", difficulty.symbol_density);
println!("Pattern complexity: {:.3}", difficulty.pattern_complexity);
```

**Parameters**:
- `content`: `&str` - Content to analyze
**Returns**: `DifficultyScore`
**Performance**: <5ms for typical content

```rust
pub struct DifficultyScore {
    pub overall: f64,               // 0.0-1.0 overall difficulty
    pub character_complexity: f64,  // Character-level difficulty
    pub pattern_complexity: f64,    // Pattern recognition difficulty
    pub symbol_density: f64,        // Ratio of symbols to total characters
    pub number_density: f64,        // Ratio of numbers to total characters
    pub tech_term_density: f64,     // Ratio of technical terms
    pub estimated_wpm: f64,         // Estimated typing speed for average user
}
```

---

##### `validate_progression(level_range: Range<u8>) -> Result<()>`

Validates difficulty progression across multiple levels.

```rust
// Example: Validate Bronze tier progression
manager.validate_progression(1..11).await?;

// Example: Validate custom range
manager.validate_progression(25..35).await?;
```

**Parameters**:
- `level_range`: `Range<u8>` - Range of levels to validate
**Returns**: `Result<()>`
**Performance**: ~100ms for 10-level range
**Errors**: `ContentError::ProgressionViolation`

---

### ContentValidator

Security and content validation APIs.

```rust
pub struct ContentValidator {
    // Internal implementation
}
```

##### `validate_security(content: &str) -> ValidationResult`

Validates content for security issues.

```rust
// Example: Security validation
let validator = ContentValidator::new()?;
let safe_content = "function test() { return 42; }";
let unsafe_content = "\x1b[31mRed text\x1b[0m"; // ANSI escape sequence

let safe_result = validator.validate_security(safe_content);
assert!(safe_result.is_valid());

let unsafe_result = validator.validate_security(unsafe_content);
assert!(!unsafe_result.is_valid());
```

**Parameters**:
- `content`: `&str` - Content to validate
**Returns**: `ValidationResult`
**Performance**: <2ms validation time

```rust
pub struct ValidationResult {
    pub is_valid: bool,
    pub issues: Vec<SecurityIssue>,
    pub severity: SecuritySeverity,
}

pub enum SecurityIssue {
    EscapeSequence { sequence: String, position: usize },
    UnauthorizedCharacter { character: char, position: usize },
    ContentTooLong { length: usize, max_allowed: usize },
    SuspiciousPattern { pattern: String, position: usize },
}

pub enum SecuritySeverity {
    None,
    Low,
    Medium,
    High,
    Critical,
}
```

---

## Core Engine APIs

### CentotypeCore

Central coordination and business logic.

```rust
use centotype_core::CentotypeCore;

pub struct CentotypeCore {
    // Internal implementation
}
```

##### `new() -> Self`

Creates a new core engine instance.

```rust
// Example: Initialize core engine
let core = CentotypeCore::new();
```

**Returns**: `CentotypeCore`
**Performance**: <10ms initialization

---

##### `start_session(mode: TrainingMode, target_text: String) -> Result<Uuid>`

Starts a new typing session.

```rust
// Example: Start arcade mode session
let session_id = core.start_session(
    TrainingMode::Arcade { level: LevelId::new(5)? },
    "The quick brown fox jumps over the lazy dog".to_string()
)?;

// Example: Start drill mode session
let session_id = core.start_session(
    TrainingMode::Drill { category: DrillCategory::Symbols },
    "!@#$%^&*(){}[]".to_string()
)?;
```

**Parameters**:
- `mode`: `TrainingMode` - Type of training session
- `target_text`: `String` - Content to type
**Returns**: `Result<Uuid>` - Session identifier
**Performance**: <5ms session creation
**Errors**: `CoreError::SessionCreationFailed`

```rust
pub enum TrainingMode {
    Arcade { level: LevelId },
    Drill { category: DrillCategory },
    Endurance { duration: Duration },
    Placement,
    Custom { config: CustomConfig },
}

pub enum DrillCategory {
    Symbols,
    Numbers,
    Code,
    Brackets,
    WeakKeys,
}
```

---

##### `process_keystroke(char_typed: Option<char>, is_correction: bool) -> Result<LiveMetrics>`

Processes a single keystroke and returns live metrics.

```rust
// Example: Process regular keystroke
let metrics = core.process_keystroke(Some('h'), false)?;
println!("Current WPM: {:.1}", metrics.raw_wpm);

// Example: Process correction (backspace)
let metrics = core.process_keystroke(None, true)?;
println!("Accuracy after correction: {:.1}%", metrics.accuracy);
```

**Parameters**:
- `char_typed`: `Option<char>` - Character typed (None for backspace)
- `is_correction`: `bool` - Whether this is a correction
**Returns**: `Result<LiveMetrics>` - Real-time performance metrics
**Performance**: <5ms P99 processing time

```rust
pub struct LiveMetrics {
    pub raw_wpm: f64,              // Raw words per minute
    pub effective_wpm: f64,        // Accuracy-adjusted WPM
    pub accuracy: f64,             // Character accuracy percentage
    pub consistency: f64,          // Timing consistency score
    pub current_streak: u32,       // Current correct streak
    pub characters_typed: u32,     // Total characters typed
    pub errors_made: u32,          // Total errors made
    pub elapsed_time: Duration,    // Session duration
}
```

---

##### `complete_session() -> Result<SessionResult>`

Completes the current session and returns final results.

```rust
// Example: Complete session and get results
let result = core.complete_session()?;
println!("Final WPM: {:.1}", result.metrics.effective_wpm);
println!("Accuracy: {:.1}%", result.metrics.accuracy);
println!("Grade: {:?}", result.grade);
```

**Returns**: `Result<SessionResult>`
**Performance**: <20ms completion processing

```rust
pub struct SessionResult {
    pub session_id: Uuid,
    pub mode: TrainingMode,
    pub completed_at: DateTime<Utc>,
    pub duration_seconds: f64,
    pub metrics: FinalMetrics,
    pub skill_index: f64,           // 0-1000 skill rating
    pub grade: Grade,               // A-F letter grade
    pub stars: u8,                  // 1-5 star rating
}

pub struct FinalMetrics {
    pub raw_wpm: f64,
    pub effective_wpm: f64,
    pub accuracy: f64,
    pub consistency: f64,
    pub longest_streak: u32,
    pub errors: ErrorStats,
    pub latency_p99: Duration,      // Input processing latency
}

pub enum Grade {
    A, B, C, D, F
}
```

---

### SessionManager

Session state management APIs.

```rust
pub struct SessionManager {
    // Internal implementation
}
```

##### `get_current_session() -> Option<&SessionState>`

Returns reference to current session state.

```rust
// Example: Check current session
if let Some(session) = session_manager.get_current_session() {
    println!("Typed: {}", session.typed_text);
    println!("Progress: {}/{}", session.cursor_position, session.target_text.len());
}
```

**Returns**: `Option<&SessionState>`
**Performance**: <1ms access time

```rust
pub struct SessionState {
    pub session_id: Uuid,
    pub mode: TrainingMode,
    pub target_text: String,
    pub typed_text: String,
    pub cursor_position: usize,
    pub started_at: DateTime<Utc>,
    pub paused_duration: Duration,
    pub is_paused: bool,
    pub is_completed: bool,
    pub keystrokes: Vec<Keystroke>,
}

pub struct Keystroke {
    pub character: Option<char>,
    pub timestamp: DateTime<Utc>,
    pub is_correction: bool,
    pub processing_latency: Duration,
}
```

---

### ScoringEngine

Scoring and metrics calculation APIs.

```rust
pub struct ScoringEngine {
    // Internal implementation
}
```

##### `calculate_live_metrics(session_state: &SessionState) -> LiveMetrics`

Calculates real-time performance metrics.

```rust
// Example: Calculate current metrics
let metrics = scoring_engine.calculate_live_metrics(&session_state);
```

**Parameters**:
- `session_state`: `&SessionState` - Current session state
**Returns**: `LiveMetrics`
**Performance**: <2ms calculation time

---

##### `calculate_final_metrics(session_state: &SessionState) -> FinalMetrics`

Calculates final session metrics.

```rust
// Example: Calculate final results
let final_metrics = scoring_engine.calculate_final_metrics(&session_state);
```

**Parameters**:
- `session_state`: `&SessionState` - Completed session state
**Returns**: `FinalMetrics`
**Performance**: <10ms calculation time

---

## Platform APIs

### PlatformManager

Platform detection and optimization APIs.

```rust
use centotype_platform::PlatformManager;

pub struct PlatformManager {
    // Internal implementation
}
```

##### `new() -> Result<Self>`

Creates platform manager with automatic detection.

```rust
// Example: Initialize platform manager
let platform_manager = PlatformManager::new()?;
```

**Returns**: `Result<PlatformManager>`
**Performance**: <20ms initialization
**Errors**: `PlatformError::DetectionFailed`

---

##### `detect_capabilities() -> PlatformCapabilities`

Detects current platform capabilities.

```rust
// Example: Check platform capabilities
let capabilities = platform_manager.detect_capabilities();
println!("OS: {:?}", capabilities.operating_system);
println!("Terminal: {:?}", capabilities.terminal_type);
println!("Supports colors: {}", capabilities.supports_colors);
```

**Returns**: `PlatformCapabilities`
**Performance**: <5ms detection time

```rust
pub struct PlatformCapabilities {
    pub operating_system: OperatingSystem,
    pub terminal_type: TerminalType,
    pub supports_colors: bool,
    pub supports_unicode: bool,
    pub supports_mouse: bool,
    pub max_colors: u16,
    pub terminal_size: (u16, u16),      // (width, height)
    pub input_latency_baseline: Duration,
}

pub enum OperatingSystem {
    Linux,
    MacOS,
    Windows,
    Other(String),
}

pub enum TerminalType {
    Xterm,
    GnomeTerminal,
    ITerm2,
    WindowsTerminal,
    Cmd,
    PowerShell,
    Other(String),
}
```

---

##### `optimize_for_performance() -> Result<()>`

Applies platform-specific performance optimizations.

```rust
// Example: Apply performance optimizations
platform_manager.optimize_for_performance()?;
```

**Returns**: `Result<()>`
**Performance**: <10ms optimization time
**Errors**: `PlatformError::OptimizationFailed`

---

##### `validate_performance_targets() -> PerformanceValidation`

Validates that platform can meet performance targets.

```rust
// Example: Validate performance capabilities
let validation = platform_manager.validate_performance_targets();
if !validation.meets_input_latency_target {
    println!("Warning: Platform may not meet input latency targets");
}
```

**Returns**: `PerformanceValidation`
**Performance**: <50ms validation time

```rust
pub struct PerformanceValidation {
    pub meets_input_latency_target: bool,
    pub meets_render_target: bool,
    pub meets_memory_target: bool,
    pub estimated_input_latency: Duration,
    pub estimated_render_time: Duration,
    pub available_memory_mb: u64,
    pub recommendations: Vec<String>,
}
```

---

### InputHandler

Low-level input processing APIs.

```rust
pub struct InputHandler {
    // Internal implementation
}
```

##### `start_input_processing() -> Result<Receiver<InputEvent>>`

Starts input processing and returns event receiver.

```rust
// Example: Start input processing
let input_receiver = input_handler.start_input_processing()?;

// Process input events
while let Ok(input_event) = input_receiver.recv().await {
    match input_event.event_type {
        InputEventType::Character(ch) => {
            println!("Character typed: {}", ch);
        },
        InputEventType::Backspace => {
            println!("Backspace pressed");
        },
        InputEventType::Control(key) => {
            println!("Control key: {:?}", key);
        },
    }
}
```

**Returns**: `Result<Receiver<InputEvent>>`
**Performance**: <5ms startup time

```rust
pub struct InputEvent {
    pub event_type: InputEventType,
    pub timestamp: Instant,
    pub modifiers: KeyModifiers,
    pub processing_latency: Duration,
}

pub enum InputEventType {
    Character(char),
    Backspace,
    Control(ControlKey),
    Navigation(NavigationKey),
}

pub enum ControlKey {
    Escape,
    Tab,
    Enter,
    CtrlC,
}

bitflags! {
    pub struct KeyModifiers: u8 {
        const SHIFT = 0b00000001;
        const CTRL = 0b00000010;
        const ALT = 0b00000100;
        const SUPER = 0b00001000;
    }
}
```

---

## CLI APIs

### CommandProcessor

Command parsing and execution APIs.

```rust
use centotype_cli::CommandProcessor;

pub struct CommandProcessor {
    // Internal implementation
}
```

##### `parse_command(args: Vec<String>) -> Result<Command>`

Parses command line arguments into commands.

```rust
// Example: Parse command
let args = vec!["play".to_string(), "--level".to_string(), "5".to_string()];
let command = command_processor.parse_command(args)?;

match command {
    Command::Play { level, options } => {
        println!("Starting level {}", level.0);
    },
    _ => {},
}
```

**Parameters**:
- `args`: `Vec<String>` - Command line arguments
**Returns**: `Result<Command>`
**Performance**: <1ms parsing time

```rust
pub enum Command {
    Play {
        level: Option<LevelId>,
        options: PlayOptions,
    },
    Drill {
        category: DrillCategory,
        options: DrillOptions,
    },
    Endurance {
        duration: Option<Duration>,
        options: EnduranceOptions,
    },
    Stats {
        detailed: bool,
        level: Option<LevelId>,
    },
    Config {
        action: ConfigAction,
    },
    Placement,
}

pub struct PlayOptions {
    pub continue_session: bool,
    pub level_range: Option<Range<u8>>,
    pub duration_limit: Option<Duration>,
}
```

---

##### `execute_command(command: Command) -> Result<CommandResult>`

Executes a parsed command.

```rust
// Example: Execute command
let result = command_processor.execute_command(command).await?;
match result {
    CommandResult::SessionCompleted(session_result) => {
        println!("Session completed with {} WPM", session_result.metrics.effective_wpm);
    },
    CommandResult::ConfigUpdated => {
        println!("Configuration updated successfully");
    },
    _ => {},
}
```

**Parameters**:
- `command`: `Command` - Parsed command to execute
**Returns**: `Result<CommandResult>`
**Performance**: Variable based on command

```rust
pub enum CommandResult {
    SessionCompleted(SessionResult),
    StatsDisplayed(UserStats),
    ConfigUpdated,
    Help(String),
    Error(String),
}
```

---

### NavigationManager

Interactive navigation APIs.

```rust
pub struct NavigationManager {
    // Internal implementation
}
```

##### `start_interactive_mode() -> Result<()>`

Starts interactive navigation interface.

```rust
// Example: Start interactive mode
navigation_manager.start_interactive_mode().await?;
```

**Returns**: `Result<()>`
**Performance**: Real-time interactive interface

---

## Analytics APIs

### AnalyticsEngine

Performance analysis and reporting APIs.

```rust
use centotype_analytics::AnalyticsEngine;

pub struct AnalyticsEngine {
    // Internal implementation
}
```

##### `analyze_session(session_result: &SessionResult) -> AnalysisReport`

Analyzes a completed session.

```rust
// Example: Analyze session performance
let report = analytics_engine.analyze_session(&session_result);
println!("Strengths: {:?}", report.strengths);
println!("Improvement areas: {:?}", report.improvement_areas);
```

**Parameters**:
- `session_result`: `&SessionResult` - Completed session data
**Returns**: `AnalysisReport`
**Performance**: <10ms analysis time

```rust
pub struct AnalysisReport {
    pub overall_score: f64,
    pub strengths: Vec<SkillArea>,
    pub improvement_areas: Vec<SkillArea>,
    pub consistency_rating: f64,
    pub error_patterns: Vec<ErrorPattern>,
    pub recommended_practice: Vec<DrillCategory>,
    pub skill_progression: SkillProgression,
}

pub enum SkillArea {
    Speed,
    Accuracy,
    Consistency,
    Symbols,
    Numbers,
    CodePatterns,
}
```

---

##### `generate_progress_report(user_id: &str, timeframe: Timeframe) -> ProgressReport`

Generates progress report over time.

```rust
// Example: Generate weekly progress report
let report = analytics_engine.generate_progress_report("user123", Timeframe::Week)?;
```

**Parameters**:
- `user_id`: `&str` - User identifier
- `timeframe`: `Timeframe` - Time period for report
**Returns**: `ProgressReport`
**Performance**: <50ms for typical user

```rust
pub enum Timeframe {
    Day,
    Week,
    Month,
    Year,
    Custom { start: DateTime<Utc>, end: DateTime<Utc> },
}

pub struct ProgressReport {
    pub timeframe: Timeframe,
    pub sessions_completed: u32,
    pub total_practice_time: Duration,
    pub average_wpm: f64,
    pub wpm_improvement: f64,
    pub accuracy_trend: AccuracyTrend,
    pub levels_completed: Vec<LevelId>,
    pub achievements: Vec<Achievement>,
}
```

---

## Persistence APIs

### ConfigManager

Configuration management APIs.

```rust
use centotype_persistence::ConfigManager;

pub struct ConfigManager {
    // Internal implementation
}
```

##### `load_config() -> Result<AppConfig>`

Loads application configuration.

```rust
// Example: Load configuration
let config = config_manager.load_config()?;
println!("Theme: {:?}", config.ui.theme);
```

**Returns**: `Result<AppConfig>`
**Performance**: <10ms load time

```rust
pub struct AppConfig {
    pub ui: UiConfig,
    pub performance: PerformanceConfig,
    pub content: ContentConfig,
    pub analytics: AnalyticsConfig,
}

pub struct UiConfig {
    pub theme: Theme,
    pub layout: KeyboardLayout,
    pub sound_enabled: bool,
    pub animations_enabled: bool,
}

pub enum Theme {
    Dark,
    Light,
    Auto,
}

pub enum KeyboardLayout {
    Qwerty,
    Qwertz,
    Azerty,
    Dvorak,
}
```

---

##### `save_config(config: &AppConfig) -> Result<()>`

Saves application configuration.

```rust
// Example: Save configuration
let mut config = config_manager.load_config()?;
config.ui.theme = Theme::Dark;
config_manager.save_config(&config)?;
```

**Parameters**:
- `config`: `&AppConfig` - Configuration to save
**Returns**: `Result<()>`
**Performance**: <5ms save time (atomic operation)

---

### ProfileManager

User profile management APIs.

```rust
pub struct ProfileManager {
    // Internal implementation
}
```

##### `load_profile(user_id: &str) -> Result<UserProfile>`

Loads user profile data.

```rust
// Example: Load user profile
let profile = profile_manager.load_profile("user123")?;
println!("Best WPM: {:.1}", profile.best_wpm);
```

**Parameters**:
- `user_id`: `&str` - User identifier
**Returns**: `Result<UserProfile>`
**Performance**: <5ms load time

```rust
pub struct UserProfile {
    pub user_id: String,
    pub created_at: DateTime<Utc>,
    pub last_active: DateTime<Utc>,
    pub total_sessions: u32,
    pub total_practice_time: Duration,
    pub best_wpm: f64,
    pub best_accuracy: f64,
    pub current_level: LevelId,
    pub completed_levels: HashSet<LevelId>,
    pub skill_index: f64,
    pub achievements: Vec<Achievement>,
    pub preferences: UserPreferences,
}
```

---

##### `save_session_result(user_id: &str, session_result: &SessionResult) -> Result<()>`

Saves session result to user profile.

```rust
// Example: Save session result
profile_manager.save_session_result("user123", &session_result)?;
```

**Parameters**:
- `user_id`: `&str` - User identifier
- `session_result`: `&SessionResult` - Session data to save
**Returns**: `Result<()>`
**Performance**: <10ms save time

---

## Types and Data Structures

### Core Types

#### LevelId

Represents a typing level (1-100).

```rust
pub struct LevelId(pub u8);

impl LevelId {
    pub const MIN: u8 = 1;
    pub const MAX: u8 = 100;

    pub fn new(level: u8) -> Result<Self> {
        if level >= Self::MIN && level <= Self::MAX {
            Ok(LevelId(level))
        } else {
            Err(CentotypeError::InvalidLevel(level))
        }
    }

    pub fn tier(&self) -> Tier {
        match self.0 {
            1..=20 => Tier::Bronze,
            21..=40 => Tier::Silver,
            41..=60 => Tier::Gold,
            61..=80 => Tier::Platinum,
            81..=100 => Tier::Diamond,
            _ => unreachable!(),
        }
    }

    pub fn next(&self) -> Option<LevelId> {
        if self.0 < Self::MAX {
            Some(LevelId(self.0 + 1))
        } else {
            None
        }
    }

    pub fn previous(&self) -> Option<LevelId> {
        if self.0 > Self::MIN {
            Some(LevelId(self.0 - 1))
        } else {
            None
        }
    }
}
```

#### Tier

Represents skill tiers (Bronze through Diamond).

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Tier {
    Bronze = 1,
    Silver = 2,
    Gold = 3,
    Platinum = 4,
    Diamond = 5,
}

impl Tier {
    pub fn requirements(&self) -> TierRequirements {
        match self {
            Tier::Bronze => TierRequirements {
                min_wpm: 40.0,
                min_accuracy: 95.0,
                max_error_severity: 8.0,
            },
            Tier::Silver => TierRequirements {
                min_wpm: 60.0,
                min_accuracy: 97.0,
                max_error_severity: 6.0,
            },
            Tier::Gold => TierRequirements {
                min_wpm: 80.0,
                min_accuracy: 98.0,
                max_error_severity: 4.0,
            },
            Tier::Platinum => TierRequirements {
                min_wpm: 100.0,
                min_accuracy: 99.0,
                max_error_severity: 3.0,
            },
            Tier::Diamond => TierRequirements {
                min_wpm: 130.0,
                min_accuracy: 99.5,
                max_error_severity: 2.0,
            },
        }
    }
}
```

---

## Error Handling

### Error Types Hierarchy

```rust
#[derive(Debug, thiserror::Error)]
pub enum CentotypeError {
    #[error("Content error: {0}")]
    Content(#[from] ContentError),

    #[error("Core engine error: {0}")]
    Core(#[from] CoreError),

    #[error("Platform error: {0}")]
    Platform(#[from] PlatformError),

    #[error("Analytics error: {0}")]
    Analytics(#[from] AnalyticsError),

    #[error("Persistence error: {0}")]
    Persistence(#[from] PersistenceError),

    #[error("Configuration error: {0}")]
    Configuration(#[from] ConfigurationError),
}

#[derive(Debug, thiserror::Error)]
pub enum ContentError {
    #[error("Invalid level: {0}")]
    InvalidLevel(u8),

    #[error("Content generation failed: {message}")]
    GenerationFailed { message: String },

    #[error("Cache operation failed: {operation}")]
    CacheError { operation: String },

    #[error("Validation failed: {reason}")]
    ValidationFailed { reason: String },

    #[error("Security issue detected: {issue}")]
    SecurityIssue { issue: String },
}
```

### Error Recovery Patterns

```rust
// Example: Robust error handling with recovery
async fn get_content_with_fallback(
    manager: &ContentManager,
    level_id: LevelId,
) -> Result<String> {
    match manager.get_level_content(level_id, None).await {
        Ok(content) => Ok(content),
        Err(CentotypeError::Content(ContentError::CacheError { .. })) => {
            // Cache error - try direct generation
            warn!("Cache error, attempting direct generation");
            manager.generate_deterministic_content(level_id, 0).await
        },
        Err(CentotypeError::Content(ContentError::GenerationFailed { .. })) => {
            // Generation failed - use fallback content
            warn!("Generation failed, using fallback content");
            Ok(get_fallback_content_for_level(level_id))
        },
        Err(other) => Err(other),
    }
}
```

---

## Performance Monitoring

### Performance Metrics APIs

```rust
use centotype_core::performance::{PerformanceMonitor, PerformanceMetrics};

pub struct PerformanceMonitor {
    // Internal implementation
}

impl PerformanceMonitor {
    pub fn record_input_latency(&mut self, latency: Duration) {
        // Records input processing latency
    }

    pub fn record_render_time(&mut self, render_time: Duration) {
        // Records frame render time
    }

    pub fn get_current_metrics(&self) -> PerformanceMetrics {
        // Returns current performance metrics
    }

    pub fn validate_targets(&self) -> TargetValidation {
        // Validates against performance targets
    }
}

pub struct PerformanceMetrics {
    pub input_latency_p50: Duration,
    pub input_latency_p95: Duration,
    pub input_latency_p99: Duration,
    pub render_time_p50: Duration,
    pub render_time_p95: Duration,
    pub memory_usage_bytes: u64,
    pub cpu_usage_percent: f64,
    pub cache_hit_rate: f64,
}

pub struct TargetValidation {
    pub meets_input_latency_target: bool,
    pub meets_render_target: bool,
    pub meets_memory_target: bool,
    pub violations: Vec<PerformanceViolation>,
}
```

### Usage Examples

```rust
// Example: Monitor performance during session
let mut monitor = PerformanceMonitor::new();

// Record metrics during operation
let start = Instant::now();
process_keystroke('a').await?;
monitor.record_input_latency(start.elapsed());

// Check performance targets
let validation = monitor.validate_targets();
if !validation.meets_input_latency_target {
    warn!("Input latency target violated");
}

// Get detailed metrics
let metrics = monitor.get_current_metrics();
println!("P99 input latency: {}ms", metrics.input_latency_p99.as_millis());
```

---

## Configuration APIs

### Configuration Management

```rust
use centotype_persistence::{ConfigManager, AppConfig};

// Example: Complete configuration management
async fn configure_application() -> Result<()> {
    let config_manager = ConfigManager::new()?;

    // Load existing configuration
    let mut config = config_manager.load_config()?;

    // Modify configuration
    config.ui.theme = Theme::Dark;
    config.performance.enable_high_performance_mode = true;
    config.content.cache_config.max_items = 75;

    // Validate configuration
    config_manager.validate_config(&config)?;

    // Save configuration atomically
    config_manager.save_config(&config)?;

    // Apply configuration to running systems
    apply_configuration_changes(&config).await?;

    Ok(())
}

async fn apply_configuration_changes(config: &AppConfig) -> Result<()> {
    // Apply UI changes
    if let Some(ui_manager) = get_ui_manager() {
        ui_manager.set_theme(config.ui.theme).await?;
    }

    // Apply performance changes
    if let Some(perf_manager) = get_performance_manager() {
        perf_manager.update_config(&config.performance).await?;
    }

    // Apply content changes
    if let Some(content_manager) = get_content_manager() {
        content_manager.update_config(config.content.clone()).await?;
    }

    Ok(())
}
```

---

## Summary

This API Reference provides comprehensive documentation for all public interfaces in the Centotype system. Key highlights:

### Performance Guarantees

- **Content Loading**: P99 <25ms (cache hit <2ms)
- **Input Processing**: P99 <5ms per keystroke
- **Session Management**: <10ms for state operations
- **Configuration**: <10ms load/save operations

### Thread Safety

- All public APIs are thread-safe unless noted
- Concurrent access patterns are optimized
- Lock-free operations where possible

### Error Handling

- Comprehensive error types with context
- Recovery strategies for common failures
- Graceful degradation under load

### Memory Management

- Bounded memory usage with configurable limits
- Automatic cleanup and garbage collection
- Memory pressure monitoring and response

This API documentation serves as the authoritative reference for integrating with and extending the Centotype system.