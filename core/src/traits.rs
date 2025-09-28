//! Stable trait boundaries for inter-crate communication
//!
//! This module defines the frozen trait contracts that enable coordinated development
//! across all crates. These interfaces MUST remain stable throughout the development
//! cycle to prevent breaking changes during parallel development.
//!
//! ## Data Flow Architecture
//!
//! ```text
//! content/ → core/ → engine/ → cli/ (main data path)
//! analytics/ ← engine/ (side effects, non-blocking)
//! persistence/ ← engine/ (async, non-blocking)
//! platform/ ↔ engine/ (bidirectional, low-level)
//! ```
//!
//! ## Performance Constraints
//!
//! - content/ → core/: <5ms content lookup (cache hit)
//! - core/ → engine/: <5ms scoring calculation
//! - engine/ → cli/: <15ms render update
//! - Total input-to-visual: <25ms P99
//!
//! ## Arc Usage Pattern
//!
//! All shared data uses Arc<> to minimize clone overhead. Implementations
//! should store Arc references and clone the Arc, not the contained data.

use async_trait::async_trait;
use crossterm::event::{Event as CrosstermEvent, KeyCode};
use std::sync::Arc;
use std::time::{Duration, Instant};

use crate::events::{GameEvent, ErrorType, EventSystemMetrics};
use crate::types::*;

// ============================================================================
// Core ↔ Engine Trait Boundary
// ============================================================================

/// Scoring engine interface for real-time performance calculations
///
/// **FROZEN CONTRACT**: This trait must remain stable for coordinated development.
/// Implementation: Core crate → Used by: Engine crate
///
/// Performance requirements:
/// - process_keystroke: P99 < 5ms
/// - current_state: P99 < 1ms (read-only)
/// - Memory usage: < 10MB total
#[async_trait]
pub trait ScoringEngine: Send + Sync {
    /// Process a keystroke and return immediate scoring result
    ///
    /// This is the hot path for real-time feedback. Must be extremely fast.
    /// No heap allocations allowed in this method.
    async fn process_keystroke(&mut self, key: KeyCode, timestamp: Instant) -> Result<ScoringResult>;

    /// Get current session state (read-only)
    ///
    /// Returns current state without modifications. Used for UI updates.
    fn current_state(&self) -> &SessionState;

    /// Check if current session is complete
    ///
    /// Fast boolean check for session completion logic.
    fn is_session_complete(&self) -> bool;

    /// Complete the current session and calculate final results
    ///
    /// This can be slower as it's called only once per session.
    /// Performs final calculations and validation.
    async fn complete_session(&mut self) -> Result<SessionResult>;

    /// Get live metrics for real-time display
    ///
    /// Called frequently during typing. Must be fast.
    fn get_live_metrics(&self) -> LiveMetrics;

    /// Classify an error between expected and actual characters
    ///
    /// Used for educational feedback and analytics.
    fn classify_error(&self, expected: char, actual: char, context: &str) -> ErrorType;

    /// Update session configuration mid-session
    ///
    /// Rarely used, can be slower than hot path methods.
    async fn update_session_config(&mut self, config: SessionConfig) -> Result<()>;
}

/// Result of processing a single keystroke
#[derive(Debug, Clone)]
pub struct ScoringResult {
    /// Whether the keystroke was correct
    pub is_correct: bool,
    /// Current position in text
    pub position: usize,
    /// Generated game event
    pub game_event: GameEvent,
    /// Updated live metrics
    pub live_metrics: LiveMetrics,
    /// Whether session should continue
    pub should_continue: bool,
}

/// Session configuration that can be updated during typing
#[derive(Debug, Clone)]
pub struct SessionConfig {
    /// Enable/disable live metrics calculation
    pub live_metrics_enabled: bool,
    /// Target accuracy threshold
    pub target_accuracy: f64,
    /// Maximum allowed errors before auto-completion
    pub max_errors: Option<u32>,
}

// ============================================================================
// Engine ↔ Content Trait Boundary
// ============================================================================

/// Content loading interface for dynamic text generation
///
/// **FROZEN CONTRACT**: This trait must remain stable for coordinated development.
/// Implementation: Content crate → Used by: Engine crate
///
/// Performance requirements:
/// - load_level_content: P99 < 25ms (including generation)
/// - get_cached_content: P99 < 5ms (cache hit only)
/// - Cache hit rate: > 90%
#[async_trait]
pub trait ContentLoader: Send + Sync {
    /// Load content for a specific level with caching
    ///
    /// This method handles cache misses by generating new content.
    /// Should automatically trigger preloading of upcoming levels.
    async fn load_level_content(&self, level_id: LevelId, seed: Option<u64>) -> Result<String>;

    /// Get cached content only (no generation on miss)
    ///
    /// Fast cache lookup that returns None if content not cached.
    /// Used for UI prefetching and availability checks.
    async fn get_cached_content(&self, level_id: LevelId, seed: Option<u64>) -> Option<String>;

    /// Preload content for upcoming levels in background
    ///
    /// Non-blocking preloading to warm cache for future levels.
    /// Implementation should use background tasks.
    async fn preload_next_levels(&self, current: LevelId, count: usize) -> Result<()>;

    /// Invalidate cached content for a level
    ///
    /// Used when content needs to be regenerated (e.g., seed change).
    async fn invalidate_content(&self, level_id: LevelId, seed: Option<u64>);

    /// Get content system performance metrics
    ///
    /// Used for monitoring cache performance and system health.
    fn get_content_metrics(&self) -> ContentMetrics;

    /// Validate that content meets difficulty requirements
    ///
    /// Used for quality assurance of generated content.
    fn validate_content_difficulty(&self, content: &str, level_id: LevelId) -> bool;
}

/// Content system performance metrics
#[derive(Debug, Clone)]
pub struct ContentMetrics {
    /// Cache hit rate (0.0-1.0)
    pub cache_hit_rate: f64,
    /// Average content loading time
    pub avg_load_time: Duration,
    /// P99 content loading time
    pub p99_load_time: Duration,
    /// Number of cached items
    pub cached_items: usize,
    /// Memory usage in bytes
    pub memory_usage: u64,
}

// ============================================================================
// Engine ↔ Analytics Trait Boundary
// ============================================================================

/// Analytics collection interface for performance tracking
///
/// **FROZEN CONTRACT**: This trait must remain stable for coordinated development.
/// Implementation: Analytics crate → Used by: Engine crate
///
/// Performance requirements:
/// - record_keystroke: P99 < 1ms (non-blocking)
/// - All operations must be async and non-blocking
/// - No impact on critical path performance
#[async_trait]
pub trait AnalyticsCollector: Send + Sync {
    /// Record a keystroke event for analysis
    ///
    /// Non-blocking operation that queues the event for processing.
    /// Should never fail or block the main game loop.
    async fn record_keystroke(&mut self, key: KeyCode, result: ScoringResult) -> Result<()>;

    /// Calculate current words per minute
    ///
    /// Fast calculation based on current session data.
    fn calculate_wpm(&self) -> f64;

    /// Calculate current accuracy percentage
    ///
    /// Fast calculation based on current session data.
    fn calculate_accuracy(&self) -> f64;

    /// Get current error distribution
    ///
    /// Returns breakdown of error types for educational feedback.
    fn get_error_distribution(&self) -> ErrorDistribution;

    /// Get typing rhythm analysis
    ///
    /// Advanced analytics for timing patterns and consistency.
    fn get_rhythm_analysis(&self) -> RhythmAnalysis;

    /// Record session completion for long-term analysis
    ///
    /// Called once per session to update user progress tracking.
    async fn record_session_completion(&mut self, result: &SessionResult) -> Result<()>;

    /// Get analytics system performance metrics
    ///
    /// Used for monitoring analytics system health.
    fn get_analytics_metrics(&self) -> AnalyticsMetrics;
}

/// Distribution of error types
#[derive(Debug, Clone, Default)]
pub struct ErrorDistribution {
    pub substitution_count: u32,
    pub insertion_count: u32,
    pub deletion_count: u32,
    pub transposition_count: u32,
    pub total_errors: u32,
}

/// Typing rhythm analysis
#[derive(Debug, Clone)]
pub struct RhythmAnalysis {
    /// Average time between keystrokes
    pub avg_keystroke_interval: Duration,
    /// Consistency score (0.0-1.0, higher = more consistent)
    pub consistency_score: f64,
    /// Typing burst patterns (speeds up/slows down)
    pub burst_patterns: Vec<Duration>,
    /// Pause detection (longer gaps in typing)
    pub pause_events: Vec<Duration>,
}

/// Analytics system performance metrics
#[derive(Debug, Clone)]
pub struct AnalyticsMetrics {
    /// Number of events processed
    pub events_processed: u64,
    /// Average processing latency
    pub avg_processing_latency: Duration,
    /// Current queue depth
    pub queue_depth: usize,
    /// Number of dropped events
    pub dropped_events: u64,
}

// ============================================================================
// Engine ↔ Persistence Trait Boundary
// ============================================================================

/// Session persistence interface for data storage
///
/// **FROZEN CONTRACT**: This trait must remain stable for coordinated development.
/// Implementation: Persistence crate → Used by: Engine crate
///
/// Performance requirements:
/// - All operations must be async and non-blocking
/// - save_session: Can be slower (1-2 seconds acceptable)
/// - load_profile: P95 < 500ms
/// - update_progress: P99 < 100ms
#[async_trait]
pub trait SessionPersistence: Send + Sync {
    /// Save complete session results
    ///
    /// Called at session completion. Can be slower as it's not on critical path.
    /// Should handle atomic writes and error recovery.
    async fn save_session(&self, result: &SessionResult) -> Result<()>;

    /// Load user profile data
    ///
    /// Called at application startup and level transitions.
    /// Should include progress, settings, and statistics.
    async fn load_profile(&self, profile_id: &str) -> Result<UserProfile>;

    /// Update user progress atomically
    ///
    /// Fast update for level progression and statistics.
    /// Must be atomic to prevent data corruption.
    async fn update_progress(&self, level_id: LevelId, result: &SessionResult) -> Result<()>;

    /// Save application configuration
    ///
    /// Called when settings change. Can be slower.
    async fn save_config(&self, config: &Config) -> Result<()>;

    /// Load application configuration
    ///
    /// Called at startup. Should return default config if none exists.
    async fn load_config(&self) -> Result<Config>;

    /// Backup user data
    ///
    /// Background operation for data safety.
    async fn backup_user_data(&self) -> Result<()>;

    /// Get persistence system metrics
    ///
    /// Used for monitoring storage system health.
    fn get_persistence_metrics(&self) -> PersistenceMetrics;
}

/// User profile data
#[derive(Debug, Clone)]
pub struct UserProfile {
    /// Unique identifier
    pub profile_id: String,
    /// Display name
    pub display_name: String,
    /// User progress tracking
    pub progress: UserProgress,
    /// Personal settings
    pub settings: UserSettings,
    /// Creation timestamp
    pub created_at: chrono::DateTime<chrono::Utc>,
    /// Last activity timestamp
    pub last_active: chrono::DateTime<chrono::Utc>,
}

/// User-specific settings
#[derive(Debug, Clone)]
pub struct UserSettings {
    /// Preferred keyboard layout
    pub keyboard_layout: KeyboardLayout,
    /// UI theme preference
    pub theme: Theme,
    /// Sound settings
    pub sound_enabled: bool,
    /// Target WPM goal
    pub target_wpm: Option<f64>,
    /// Practice schedule preferences
    pub practice_schedule: PracticeSchedule,
}

/// Practice schedule configuration
#[derive(Debug, Clone)]
pub struct PracticeSchedule {
    /// Daily practice goal (minutes)
    pub daily_goal_minutes: u32,
    /// Preferred practice times
    pub preferred_times: Vec<chrono::NaiveTime>,
    /// Reminder settings
    pub reminders_enabled: bool,
}

/// Persistence system metrics
#[derive(Debug, Clone)]
pub struct PersistenceMetrics {
    /// Number of successful operations
    pub successful_operations: u64,
    /// Number of failed operations
    pub failed_operations: u64,
    /// Average operation latency
    pub avg_operation_latency: Duration,
    /// Available storage space
    pub available_storage_bytes: u64,
    /// Data corruption incidents
    pub corruption_incidents: u64,
}

// ============================================================================
// Engine ↔ Platform Trait Boundary
// ============================================================================

/// Terminal management interface for low-level platform integration
///
/// **FROZEN CONTRACT**: This trait must remain stable for coordinated development.
/// Implementation: Platform crate → Used by: Engine crate
///
/// Performance requirements:
/// - poll_event: P99 < 10ms
/// - All mode changes: P95 < 50ms
/// - Emergency operations must be synchronous and fast
#[async_trait]
pub trait TerminalManager: Send + Sync {
    /// Enter raw mode for direct key capture
    ///
    /// Must be synchronous for immediate effect.
    /// Should store previous state for restoration.
    fn enter_raw_mode(&mut self) -> Result<()>;

    /// Leave raw mode and restore normal terminal
    ///
    /// Must be synchronous for immediate effect.
    /// Critical for emergency cleanup.
    fn leave_raw_mode(&mut self) -> Result<()>;

    /// Enter alternate screen buffer
    ///
    /// Used for full-screen application mode.
    fn enter_alternate_screen(&mut self) -> Result<()>;

    /// Leave alternate screen buffer
    ///
    /// Must restore original screen content.
    fn leave_alternate_screen(&mut self) -> Result<()>;

    /// Poll for input events with timeout
    ///
    /// Core input handling. Must be very fast and reliable.
    /// Timeout prevents blocking the event loop.
    async fn poll_event(&self, timeout: Duration) -> Result<Option<InputEvent>>;

    /// Get terminal dimensions
    ///
    /// Fast query for layout calculations.
    fn get_terminal_size(&self) -> Result<(u16, u16)>;

    /// Check terminal capabilities
    ///
    /// One-time check for feature availability.
    fn get_terminal_capabilities(&self) -> TerminalCapabilities;

    /// Apply platform-specific optimizations
    ///
    /// Called once during initialization.
    async fn apply_optimizations(&mut self) -> Result<()>;

    /// Emergency cleanup (panic handler)
    ///
    /// Must be synchronous and never fail.
    /// Used in panic handlers and emergency shutdown.
    fn emergency_cleanup(&mut self);

    /// Get platform performance metrics
    ///
    /// Used for monitoring platform integration health.
    fn get_platform_metrics(&self) -> PlatformMetrics;
}

/// Input event wrapper
#[derive(Debug, Clone)]
pub struct InputEvent {
    /// The crossterm event
    pub event: CrosstermEvent,
    /// When the event was received
    pub timestamp: Instant,
    /// Input processing latency
    pub processing_latency: Duration,
}

/// Terminal capability detection
#[derive(Debug, Clone)]
pub struct TerminalCapabilities {
    /// Supports color output
    pub supports_color: bool,
    /// Supports raw mode
    pub supports_raw_mode: bool,
    /// Supports alternate screen
    pub supports_alternate_screen: bool,
    /// Supports high precision timing
    pub supports_high_precision_timing: bool,
    /// Terminal type identifier
    pub terminal_type: String,
    /// Platform-specific optimizations available
    pub optimizations_available: Vec<String>,
}

/// Platform integration metrics
#[derive(Debug, Clone)]
pub struct PlatformMetrics {
    /// Input event latency P99
    pub input_latency_p99: Duration,
    /// Terminal operation latency
    pub terminal_op_latency: Duration,
    /// Number of platform errors
    pub platform_errors: u64,
    /// Available optimizations
    pub active_optimizations: Vec<String>,
    /// System resource usage
    pub resource_usage: SystemResourceUsage,
}

/// System resource usage information
#[derive(Debug, Clone)]
pub struct SystemResourceUsage {
    /// CPU usage percentage
    pub cpu_usage_percent: f64,
    /// Memory usage in bytes
    pub memory_usage_bytes: u64,
    /// Available memory in bytes
    pub available_memory_bytes: u64,
    /// Terminal buffer usage
    pub terminal_buffer_usage: usize,
}

// ============================================================================
// Composite Interfaces
// ============================================================================

/// Game engine coordinator that orchestrates all subsystems
///
/// This trait represents the main engine that coordinates all other components.
/// It serves as the central hub for the game loop and event processing.
#[async_trait]
pub trait GameEngine: Send + Sync {
    /// Initialize the game engine with all subsystems
    async fn initialize(
        &mut self,
        scoring_engine: Arc<dyn ScoringEngine>,
        content_loader: Arc<dyn ContentLoader>,
        analytics_collector: Arc<dyn AnalyticsCollector>,
        persistence: Arc<dyn SessionPersistence>,
        terminal_manager: Arc<dyn TerminalManager>,
    ) -> Result<()>;

    /// Start a new typing session
    async fn start_session(&mut self, mode: TrainingMode, level_id: LevelId) -> Result<uuid::Uuid>;

    /// Run the main game loop
    async fn run_game_loop(&mut self, session_id: uuid::Uuid) -> Result<SessionResult>;

    /// Pause the current session
    async fn pause_session(&mut self) -> Result<()>;

    /// Resume the current session
    async fn resume_session(&mut self) -> Result<()>;

    /// Stop the current session
    async fn stop_session(&mut self) -> Result<SessionResult>;

    /// Get current engine performance metrics
    fn get_engine_metrics(&self) -> EngineMetrics;

    /// Emergency shutdown (for panic handlers)
    fn emergency_shutdown(&mut self);
}

/// Engine performance metrics
#[derive(Debug, Clone)]
pub struct EngineMetrics {
    /// Overall system performance
    pub event_system_metrics: EventSystemMetrics,
    /// Content loading performance
    pub content_metrics: ContentMetrics,
    /// Analytics system performance
    pub analytics_metrics: AnalyticsMetrics,
    /// Persistence system performance
    pub persistence_metrics: PersistenceMetrics,
    /// Platform integration performance
    pub platform_metrics: PlatformMetrics,
    /// Current session state
    pub session_active: bool,
    /// Total sessions completed
    pub total_sessions: u64,
}

impl EngineMetrics {
    /// Check if all subsystems meet performance targets
    pub fn meets_all_targets(&self) -> bool {
        self.event_system_metrics.meets_targets()
            && self.content_metrics.p99_load_time <= Duration::from_millis(25)
            && self.platform_metrics.input_latency_p99 <= Duration::from_millis(10)
    }

    /// Get overall health score (0.0-1.0)
    pub fn health_score(&self) -> f64 {
        let event_score = if self.event_system_metrics.meets_targets() { 1.0 } else { 0.5 };
        let content_score = if self.content_metrics.cache_hit_rate > 0.9 { 1.0 } else { self.content_metrics.cache_hit_rate };
        let platform_score = if self.platform_metrics.platform_errors == 0 { 1.0 } else { 0.5 };

        (event_score + content_score + platform_score) / 3.0
    }
}