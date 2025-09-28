//! # Centotype Core
//!
//! The core crate provides the fundamental state management, scoring engine, and domain logic
//! for the Centotype typing trainer. This crate contains:
//!
//! - Session state management and transitions
//! - Real-time scoring calculations (WPM, accuracy, skill index)
//! - Error detection and classification algorithms
//! - Level progression and unlock logic
//! - Core domain types and interfaces
//!
//! ## Performance Requirements
//!
//! - State updates: P99 < 5ms
//! - Scoring calculations: P95 < 2ms
//! - Memory usage: < 10MB for state and scoring engine
//!
//! ## Key Components
//!
//! - `SessionManager`: Thread-safe session state management
//! - `ScoringEngine`: Real-time and final scoring calculations
//! - `LevelManager`: Level progression and unlock logic
//! - `ErrorClassifier`: Damerau-Levenshtein error detection

pub mod error;
pub mod events;
pub mod level;
pub mod scoring;
pub mod session;
pub mod traits;
pub mod types;

// Re-export main types for convenience
pub use error::Error as ErrorClassifier;
pub use events::*;
pub use level::Level as LevelManager;
pub use scoring::Scoring as ScoringEngine;
pub use session::SessionManager;
pub use traits::*;
pub use types::*;

use parking_lot::RwLock;
use std::sync::Arc;

/// Core engine that coordinates all components
pub struct CentotypeCore {
    session_manager: Arc<RwLock<SessionManager>>,
    scoring_engine: Arc<ScoringEngine>,
    level_manager: Arc<LevelManager>,
    error_classifier: Arc<ErrorClassifier>,
}

impl CentotypeCore {
    pub fn new() -> Self {
        Self {
            session_manager: Arc::new(RwLock::new(SessionManager::new())),
            scoring_engine: Arc::new(ScoringEngine::new()),
            level_manager: Arc::new(LevelManager::new()),
            error_classifier: Arc::new(ErrorClassifier::new()),
        }
    }

    pub fn session_manager(&self) -> Arc<RwLock<SessionManager>> {
        Arc::clone(&self.session_manager)
    }

    pub fn scoring_engine(&self) -> Arc<ScoringEngine> {
        Arc::clone(&self.scoring_engine)
    }

    pub fn level_manager(&self) -> Arc<LevelManager> {
        Arc::clone(&self.level_manager)
    }

    pub fn error_classifier(&self) -> Arc<ErrorClassifier> {
        Arc::clone(&self.error_classifier)
    }

    /// Start a new typing session - stub implementation
    pub fn start_session(&self, mode: TrainingMode, target_text: String) -> Result<uuid::Uuid> {
        let session_id = uuid::Uuid::new_v4();
        let session_state = SessionState {
            session_id,
            mode,
            target_text,
            typed_text: String::new(),
            cursor_position: 0,
            started_at: chrono::Utc::now(),
            paused_duration: std::time::Duration::default(),
            is_paused: false,
            is_completed: false,
            keystrokes: Vec::new(),
        };

        self.session_manager.write().start_session(session_state)?;
        Ok(session_id)
    }

    /// Process a keystroke - stub implementation
    pub fn process_keystroke(
        &self,
        _char_typed: Option<char>,
        _is_correction: bool,
    ) -> Result<LiveMetrics> {
        Ok(LiveMetrics::default())
    }

    /// Complete the current session - stub implementation
    pub fn complete_session(&self) -> Result<SessionResult> {
        let session_id = uuid::Uuid::new_v4();
        Ok(SessionResult {
            session_id,
            mode: TrainingMode::Arcade {
                level: LevelId::new(1).unwrap(),
            },
            completed_at: chrono::Utc::now(),
            duration_seconds: 60.0,
            metrics: FinalMetrics {
                raw_wpm: 50.0,
                effective_wpm: 45.0,
                accuracy: 90.0,
                consistency: 85.0,
                longest_streak: 100,
                errors: ErrorStats::default(),
                latency_p99: std::time::Duration::from_millis(20),
            },
            skill_index: 700.0,
            grade: Grade::B,
            stars: 2,
        })
    }
}

impl Default for CentotypeCore {
    fn default() -> Self {
        Self::new()
    }
}
