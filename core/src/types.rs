//! Shared types and traits for the Centotype typing trainer.
//!
//! This module defines the core data structures and interfaces used across all crates.
//! These types are designed to be:
//! - Zero-copy where possible for performance
//! - Serializable for persistence
//! - Thread-safe where needed for concurrent operations

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::time::Duration;

// ============================================================================
// Core Domain Types
// ============================================================================

/// Unique identifier for a level (1-100)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct LevelId(pub u8);

impl LevelId {
    pub const MIN: u8 = 1;
    pub const MAX: u8 = 100;

    pub fn new(id: u8) -> Result<Self> {
        if (Self::MIN..=Self::MAX).contains(&id) {
            Ok(LevelId(id))
        } else {
            Err(CentotypeError::State(format!(
                "Level ID must be between {} and {}",
                Self::MIN,
                Self::MAX
            )))
        }
    }

    pub fn tier(&self) -> Tier {
        Tier::from_level(self.0)
    }
}

/// Difficulty tier (1-10), each containing 10 levels
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
pub struct Tier(pub u8);

impl Tier {
    pub fn from_level(level: u8) -> Self {
        Tier((level - 1) / 10 + 1)
    }

    pub fn weight(&self) -> f64 {
        1.0 + (self.0 as f64 - 1.0) * 0.15
    }
}

/// Training mode selection
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum TrainingMode {
    Arcade {
        level: LevelId,
    },
    Drill {
        category: DrillCategory,
        duration_secs: u32,
    },
    Endurance {
        duration_secs: u32,
    },
}

/// Drill practice categories
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum DrillCategory {
    Numbers,
    Punctuation,
    Symbols,
    CamelCase,
    SnakeCase,
    Operators,
}

/// Language for text content
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Language {
    English,
    Indonesian,
}

/// Keyboard layout
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum KeyboardLayout {
    Qwerty,
    Qwertz,
    Azerty,
}

// ============================================================================
// Session State Types
// ============================================================================

/// Complete state of an active typing session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionState {
    pub session_id: uuid::Uuid,
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

/// Individual keystroke event with timing
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Keystroke {
    pub timestamp: DateTime<Utc>,
    pub char_typed: Option<char>,
    pub is_correction: bool,
    pub cursor_pos: usize,
}

/// Real-time performance metrics during a session
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct LiveMetrics {
    pub raw_wpm: f64,
    pub effective_wpm: f64,
    pub accuracy: f64,
    pub current_streak: u32,
    pub longest_streak: u32,
    pub errors: ErrorStats,
    pub elapsed_seconds: f64,
}

/// Error statistics and classification
#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
pub struct ErrorStats {
    pub substitution: u32,
    pub insertion: u32,
    pub deletion: u32,
    pub transposition: u32,
    pub backspace_count: u32,
    pub idle_events: u32,
}

impl ErrorStats {
    pub fn total_errors(&self) -> u32 {
        self.substitution + self.insertion + self.deletion + self.transposition
    }

    pub fn severity_score(&self) -> f64 {
        (self.transposition * 3 + self.substitution * 2 + self.insertion + self.deletion) as f64
    }
}

// ============================================================================
// Scoring Types
// ============================================================================

/// Complete session results with final scoring
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SessionResult {
    pub session_id: uuid::Uuid,
    pub mode: TrainingMode,
    pub completed_at: DateTime<Utc>,
    pub duration_seconds: f64,
    pub metrics: FinalMetrics,
    pub skill_index: f64,
    pub grade: Grade,
    pub stars: u8,
}

/// Final calculated metrics for a completed session
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct FinalMetrics {
    pub raw_wpm: f64,
    pub effective_wpm: f64,
    pub accuracy: f64,
    pub consistency: f64,
    pub longest_streak: u32,
    pub errors: ErrorStats,
    pub latency_p99: Duration,
}

/// Performance grade (S/A/B/C/D)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Grade {
    S,
    A,
    B,
    C,
    D,
}

impl Grade {
    pub fn from_skill_index(si: f64, tier: Tier) -> Self {
        let tier_factor = tier.weight();
        if si >= 900.0 * tier_factor {
            Grade::S
        } else if si >= 800.0 * tier_factor {
            Grade::A
        } else if si >= 700.0 * tier_factor {
            Grade::B
        } else if si >= 600.0 * tier_factor {
            Grade::C
        } else {
            Grade::D
        }
    }

    pub fn min_for_progression() -> Self {
        Grade::C
    }
}

// ============================================================================
// Content Types
// ============================================================================

/// Text content for a typing session
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextContent {
    pub text: String,
    pub metadata: ContentMetadata,
}

/// Metadata about text content
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentMetadata {
    pub level: Option<LevelId>,
    pub category: ContentCategory,
    pub language: Language,
    pub difficulty_score: f64,
    pub char_classes: CharacterClassHistogram,
    pub estimated_duration_secs: u32,
}

/// Category of text content
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ContentCategory {
    Code,
    Prose,
    Mixed,
    Technical,
}

/// Distribution of character classes in text
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct CharacterClassHistogram {
    pub lowercase: u32,
    pub uppercase: u32,
    pub digits: u32,
    pub punctuation: u32,
    pub symbols: u32,
    pub whitespace: u32,
}

// ============================================================================
// Performance Measurement Types
// ============================================================================

/// Performance metrics for monitoring system health
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMetrics {
    pub input_latency_p50: Duration,
    pub input_latency_p95: Duration,
    pub input_latency_p99: Duration,
    pub render_time_p50: Duration,
    pub render_time_p95: Duration,
    pub startup_time: Duration,
    pub memory_rss_bytes: u64,
    pub cpu_usage_percent: f64,
}

impl PerformanceMetrics {
    pub fn meets_targets(&self) -> PerformanceCheck {
        PerformanceCheck {
            input_latency_ok: self.input_latency_p99 <= Duration::from_millis(25),
            render_ok: self.render_time_p95 <= Duration::from_millis(33),
            startup_ok: self.startup_time <= Duration::from_millis(200),
            memory_ok: self.memory_rss_bytes <= 50 * 1024 * 1024, // 50MB
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct PerformanceCheck {
    pub input_latency_ok: bool,
    pub render_ok: bool,
    pub startup_ok: bool,
    pub memory_ok: bool,
}

impl PerformanceCheck {
    pub fn all_ok(&self) -> bool {
        self.input_latency_ok && self.render_ok && self.startup_ok && self.memory_ok
    }
}

// ============================================================================
// Configuration Types
// ============================================================================

/// Application configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub layout: KeyboardLayout,
    pub language: Language,
    pub theme: Theme,
    pub sound_enabled: bool,
    pub telemetry_enabled: bool,
    pub log_level: String,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            layout: KeyboardLayout::Qwerty,
            language: Language::English,
            theme: Theme::Default,
            sound_enabled: false,
            telemetry_enabled: false,
            log_level: "info".to_string(),
        }
    }
}

/// Visual theme configuration
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Theme {
    Default,
    HighContrast,
    Mono,
}

// ============================================================================
// Core Traits
// ============================================================================

/// Trait for components that measure performance
pub trait PerformanceMeasurable {
    fn record_latency(&mut self, operation: &str, duration: Duration);
    fn get_metrics(&self) -> PerformanceMetrics;
}

/// Trait for state management
pub trait StateManager {
    fn current_state(&self) -> &SessionState;
    fn update_state(&mut self, update: StateUpdate) -> Result<()>;
    fn reset(&mut self);
}

/// State update commands
#[derive(Debug, Clone)]
pub enum StateUpdate {
    AddKeystroke(Keystroke),
    SetPaused(bool),
    MoveCursor(usize),
    Complete,
}

/// Trait for scoring engines
pub trait ScoringEngine {
    fn calculate_wpm(&self, chars_typed: usize, duration: Duration) -> f64;
    fn calculate_accuracy(&self, target: &str, typed: &str) -> f64;
    fn calculate_skill_index(&self, metrics: &FinalMetrics, tier: Tier) -> f64;
    fn classify_errors(&self, target: &str, typed: &str) -> ErrorStats;
}

/// Trait for content generation
pub trait ContentGenerator {
    fn generate(&self, params: ContentParams) -> Result<TextContent>;
    fn validate(&self, content: &TextContent) -> Result<()>;
}

/// Parameters for content generation
#[derive(Debug, Clone)]
pub struct ContentParams {
    pub level: Option<LevelId>,
    pub category: ContentCategory,
    pub language: Language,
    pub length_chars: usize,
    pub symbol_ratio: f64,
    pub number_density: f64,
    pub seed: Option<u64>,
}

// ============================================================================
// User Progress Types
// ============================================================================

/// User progress tracking across all levels and sessions
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct UserProgress {
    pub best_results: std::collections::HashMap<LevelId, SessionResult>,
    pub total_sessions: u32,
    pub total_time_seconds: f64,
    pub overall_skill_index: f64,
}

impl UserProgress {
    pub fn update_with_result(&mut self, result: SessionResult) {
        self.total_sessions += 1;
        self.total_time_seconds += result.duration_seconds;

        // Update best result for the level if this is better
        if let TrainingMode::Arcade { level } = result.mode {
            let should_update = self
                .best_results
                .get(&level)
                .map(|best| result.skill_index > best.skill_index)
                .unwrap_or(true);

            if should_update {
                self.best_results.insert(level, result);
            }
        }

        // Recalculate overall skill index
        self.recalculate_overall_skill_index();
    }

    fn recalculate_overall_skill_index(&mut self) {
        if self.best_results.is_empty() {
            self.overall_skill_index = 0.0;
            return;
        }

        // Weight recent and higher-tier results more heavily
        let weighted_sum: f64 = self
            .best_results
            .values()
            .map(|result| {
                let tier_weight = if let TrainingMode::Arcade { level } = result.mode {
                    level.tier().weight()
                } else {
                    1.0
                };
                result.skill_index * tier_weight
            })
            .sum();

        let total_weight: f64 = self
            .best_results
            .values()
            .map(|result| {
                if let TrainingMode::Arcade { level } = result.mode {
                    level.tier().weight()
                } else {
                    1.0
                }
            })
            .sum();

        self.overall_skill_index = if total_weight > 0.0 {
            weighted_sum / total_weight
        } else {
            0.0
        };
    }
}

// ============================================================================
// Error Types
// ============================================================================

/// Top-level error type for the application
#[derive(Debug, thiserror::Error)]
pub enum CentotypeError {
    #[error("Configuration error: {0}")]
    Config(String),

    #[error("Content generation error: {0}")]
    Content(String),

    #[error("Persistence error: {0}")]
    Persistence(String),

    #[error("Platform error: {0}")]
    Platform(String),

    #[error("Input processing error: {0}")]
    Input(String),

    #[error("State error: {0}")]
    State(String),

    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),

    #[error("Serialization error: {0}")]
    Serialization(#[from] serde_json::Error),
}

pub type Result<T> = std::result::Result<T, CentotypeError>;
