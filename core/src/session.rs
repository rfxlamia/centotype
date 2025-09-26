//! Session management with thread-safe state tracking and performance monitoring
use crate::types::*;
use chrono::{DateTime, Utc};
use parking_lot::{Mutex, RwLock};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{debug, error, info, warn};

/// Thread-safe session manager with performance tracking
pub struct SessionManager {
    current_session: Arc<RwLock<Option<SessionState>>>,
    sessions: Arc<Mutex<HashMap<uuid::Uuid, SessionState>>>,
    performance_tracker: Arc<Mutex<SessionPerformanceTracker>>,
}

impl SessionManager {
    pub fn new() -> Self {
        Self {
            current_session: Arc::new(RwLock::new(None)),
            sessions: Arc::new(Mutex::new(HashMap::new())),
            performance_tracker: Arc::new(Mutex::new(SessionPerformanceTracker::new())),
        }
    }

    /// Start a new typing session with comprehensive state initialization
    pub fn start_session(&mut self, mut session_state: SessionState) -> Result<()> {
        let start_time = Instant::now();

        // Validate session state
        if session_state.target_text.is_empty() {
            return Err(CentotypeError::State("Target text cannot be empty".to_string()));
        }

        // Ensure consistent initial state
        session_state.typed_text.clear();
        session_state.cursor_position = 0;
        session_state.keystrokes.clear();
        session_state.is_paused = false;
        session_state.is_completed = false;
        session_state.started_at = Utc::now();
        session_state.paused_duration = Duration::default();

        let session_id = session_state.session_id;

        // Store session and set as current
        {
            let mut sessions = self.sessions.lock();
            sessions.insert(session_id, session_state.clone());
        }

        {
            let mut current = self.current_session.write();
            *current = Some(session_state);
        }

        // Track performance
        let elapsed = start_time.elapsed();
        self.performance_tracker.lock().record_session_start(elapsed);

        info!(session_id = %session_id, "Started new typing session");
        Ok(())
    }

    /// Get current session state (read-only)
    pub fn current_state(&self) -> Result<SessionState> {
        let current = self.current_session.read();
        current
            .as_ref()
            .map(|s| s.clone())
            .ok_or_else(|| CentotypeError::State("No active session".to_string()))
    }

    /// Check if there's an active session
    pub fn has_active_session(&self) -> bool {
        self.current_session.read().is_some()
    }

    /// Get session by ID
    pub fn get_session(&self, session_id: uuid::Uuid) -> Result<SessionState> {
        let sessions = self.sessions.lock();
        sessions
            .get(&session_id)
            .cloned()
            .ok_or_else(|| CentotypeError::State(format!("Session {} not found", session_id)))
    }

    /// Update session state with atomic operations
    pub fn update_state(&mut self, update: StateUpdate) -> Result<()> {
        let start_time = Instant::now();

        let mut current = self.current_session.write();
        let session = current
            .as_mut()
            .ok_or_else(|| CentotypeError::State("No active session".to_string()))?;

        // Apply state update
        match update {
            StateUpdate::AddKeystroke(keystroke) => {
                self.apply_keystroke(session, keystroke)?;
            }
            StateUpdate::SetPaused(paused) => {
                self.set_paused_state(session, paused)?;
            }
            StateUpdate::MoveCursor(position) => {
                self.move_cursor(session, position)?;
            }
            StateUpdate::Complete => {
                self.complete_session_internal(session)?;
            }
        }

        // Update stored session
        let session_id = session.session_id;
        {
            let mut sessions = self.sessions.lock();
            sessions.insert(session_id, session.clone());
        }

        // Track performance
        let elapsed = start_time.elapsed();
        self.performance_tracker.lock().record_state_update(elapsed);

        debug!(session_id = %session_id, "Updated session state");
        Ok(())
    }

    /// Reset session manager state
    pub fn reset(&mut self) {
        let mut current = self.current_session.write();
        *current = None;

        // Clear performance tracking
        *self.performance_tracker.lock() = SessionPerformanceTracker::new();

        info!("Reset session manager");
    }

    /// Complete the current session
    pub fn complete_current_session(&mut self) -> Result<SessionState> {
        let mut current = self.current_session.write();
        let session = current
            .as_mut()
            .ok_or_else(|| CentotypeError::State("No active session to complete".to_string()))?;

        self.complete_session_internal(session)?;
        let completed_session = session.clone();

        // Update stored session
        let session_id = session.session_id;
        {
            let mut sessions = self.sessions.lock();
            sessions.insert(session_id, session.clone());
        }

        info!(session_id = %session_id, "Completed typing session");
        Ok(completed_session)
    }

    /// Get performance metrics for monitoring
    pub fn get_performance_metrics(&self) -> SessionManagerMetrics {
        self.performance_tracker.lock().get_metrics()
    }

    // Private helper methods

    fn apply_keystroke(&self, session: &mut SessionState, keystroke: Keystroke) -> Result<()> {
        // Validate keystroke timing
        if keystroke.timestamp < session.started_at {
            return Err(CentotypeError::State("Invalid keystroke timestamp".to_string()));
        }

        // Handle different keystroke types
        match keystroke.char_typed {
            Some(ch) => {
                if keystroke.is_correction {
                    self.handle_correction(session, ch)?;
                } else {
                    self.handle_regular_input(session, ch)?;
                }
            }
            None => {
                // Handle special keys (backspace, etc.)
                self.handle_special_key(session)?;
            }
        }

        // Record keystroke
        session.keystrokes.push(keystroke);
        Ok(())
    }

    fn handle_regular_input(&self, session: &mut SessionState, ch: char) -> Result<()> {
        // Validate cursor position
        if session.cursor_position > session.target_text.len() {
            return Err(CentotypeError::State("Cursor position out of bounds".to_string()));
        }

        // Add character to typed text
        if session.cursor_position == session.typed_text.len() {
            session.typed_text.push(ch);
        } else {
            // Insert character at cursor position
            let mut chars: Vec<char> = session.typed_text.chars().collect();
            if session.cursor_position <= chars.len() {
                chars.insert(session.cursor_position, ch);
                session.typed_text = chars.into_iter().collect();
            }
        }

        session.cursor_position += 1;
        Ok(())
    }

    fn handle_correction(&self, session: &mut SessionState, _ch: char) -> Result<()> {
        // Handle correction logic - this would be more complex in practice
        if session.cursor_position > 0 {
            session.cursor_position -= 1;
            if session.cursor_position < session.typed_text.len() {
                let mut chars: Vec<char> = session.typed_text.chars().collect();
                chars.remove(session.cursor_position);
                session.typed_text = chars.into_iter().collect();
            }
        }
        Ok(())
    }

    fn handle_special_key(&self, session: &mut SessionState) -> Result<()> {
        // Handle backspace and other special keys
        if session.cursor_position > 0 && !session.typed_text.is_empty() {
            session.cursor_position -= 1;
            if session.cursor_position < session.typed_text.len() {
                let mut chars: Vec<char> = session.typed_text.chars().collect();
                chars.remove(session.cursor_position);
                session.typed_text = chars.into_iter().collect();
            }
        }
        Ok(())
    }

    fn set_paused_state(&self, session: &mut SessionState, paused: bool) -> Result<()> {
        if session.is_completed {
            return Err(CentotypeError::State("Cannot pause completed session".to_string()));
        }

        if session.is_paused == paused {
            return Ok(()); // No change needed
        }

        if paused {
            session.is_paused = true;
            debug!("Session paused");
        } else {
            session.is_paused = false;
            debug!("Session resumed");
        }

        Ok(())
    }

    fn move_cursor(&self, session: &mut SessionState, position: usize) -> Result<()> {
        if position > session.target_text.len() {
            return Err(CentotypeError::State("Cursor position out of bounds".to_string()));
        }

        session.cursor_position = position;
        Ok(())
    }

    fn complete_session_internal(&self, session: &mut SessionState) -> Result<()> {
        if session.is_completed {
            return Ok(()); // Already completed
        }

        session.is_completed = true;
        session.is_paused = false;

        info!(session_id = %session.session_id, "Session marked as completed");
        Ok(())
    }
}

impl Default for SessionManager {
    fn default() -> Self {
        Self::new()
    }
}

impl StateManager for SessionManager {
    fn current_state(&self) -> &SessionState {
        // This trait method can't return a Result, so we'll need to handle this differently
        // For now, we'll panic on error - in production this trait might need revision
        unimplemented!("Use current_state() method instead which returns Result")
    }

    fn update_state(&mut self, update: StateUpdate) -> Result<()> {
        self.update_state(update)
    }

    fn reset(&mut self) {
        self.reset()
    }
}

/// Performance tracking for session operations
#[derive(Debug, Clone)]
struct SessionPerformanceTracker {
    session_starts: Vec<Duration>,
    state_updates: Vec<Duration>,
    total_operations: u64,
}

impl SessionPerformanceTracker {
    fn new() -> Self {
        Self {
            session_starts: Vec::new(),
            state_updates: Vec::new(),
            total_operations: 0,
        }
    }

    fn record_session_start(&mut self, duration: Duration) {
        self.session_starts.push(duration);
        self.total_operations += 1;

        // Keep only recent measurements for memory efficiency
        if self.session_starts.len() > 1000 {
            self.session_starts.remove(0);
        }
    }

    fn record_state_update(&mut self, duration: Duration) {
        self.state_updates.push(duration);
        self.total_operations += 1;

        // Keep only recent measurements
        if self.state_updates.len() > 10000 {
            self.state_updates.remove(0);
        }
    }

    fn get_metrics(&self) -> SessionManagerMetrics {
        SessionManagerMetrics {
            avg_session_start_time: self.calculate_average(&self.session_starts),
            avg_state_update_time: self.calculate_average(&self.state_updates),
            p95_state_update_time: self.calculate_percentile(&self.state_updates, 0.95),
            p99_state_update_time: self.calculate_percentile(&self.state_updates, 0.99),
            total_operations: self.total_operations,
        }
    }

    fn calculate_average(&self, durations: &[Duration]) -> Duration {
        if durations.is_empty() {
            return Duration::default();
        }

        let sum: Duration = durations.iter().sum();
        sum / durations.len() as u32
    }

    fn calculate_percentile(&self, durations: &[Duration], percentile: f64) -> Duration {
        if durations.is_empty() {
            return Duration::default();
        }

        let mut sorted: Vec<Duration> = durations.to_vec();
        sorted.sort();

        let index = ((durations.len() as f64 - 1.0) * percentile) as usize;
        sorted[index.min(sorted.len() - 1)]
    }
}

/// Metrics for session manager performance monitoring
#[derive(Debug, Clone)]
pub struct SessionManagerMetrics {
    pub avg_session_start_time: Duration,
    pub avg_state_update_time: Duration,
    pub p95_state_update_time: Duration,
    pub p99_state_update_time: Duration,
    pub total_operations: u64,
}

impl SessionManagerMetrics {
    /// Check if performance targets are met (P99 state updates < 5ms)
    pub fn meets_targets(&self) -> bool {
        self.p99_state_update_time <= Duration::from_millis(5)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_session_manager_creation() {
        let manager = SessionManager::new();
        assert!(!manager.has_active_session());
    }

    #[test]
    fn test_start_session() {
        let mut manager = SessionManager::new();
        let session_state = SessionState {
            session_id: uuid::Uuid::new_v4(),
            mode: TrainingMode::Arcade { level: LevelId::new(1).unwrap() },
            target_text: "hello world".to_string(),
            typed_text: String::new(),
            cursor_position: 0,
            started_at: Utc::now(),
            paused_duration: Duration::default(),
            is_paused: false,
            is_completed: false,
            keystrokes: Vec::new(),
        };

        assert!(manager.start_session(session_state).is_ok());
        assert!(manager.has_active_session());
    }

    #[test]
    fn test_empty_target_text_error() {
        let mut manager = SessionManager::new();
        let session_state = SessionState {
            session_id: uuid::Uuid::new_v4(),
            mode: TrainingMode::Arcade { level: LevelId::new(1).unwrap() },
            target_text: String::new(), // Empty target text
            typed_text: String::new(),
            cursor_position: 0,
            started_at: Utc::now(),
            paused_duration: Duration::default(),
            is_paused: false,
            is_completed: false,
            keystrokes: Vec::new(),
        };

        assert!(manager.start_session(session_state).is_err());
    }

    #[test]
    fn test_keystroke_handling() {
        let mut manager = SessionManager::new();
        let session_state = SessionState {
            session_id: uuid::Uuid::new_v4(),
            mode: TrainingMode::Arcade { level: LevelId::new(1).unwrap() },
            target_text: "test".to_string(),
            typed_text: String::new(),
            cursor_position: 0,
            started_at: Utc::now(),
            paused_duration: Duration::default(),
            is_paused: false,
            is_completed: false,
            keystrokes: Vec::new(),
        };

        manager.start_session(session_state).unwrap();

        let keystroke = Keystroke {
            timestamp: Utc::now(),
            char_typed: Some('t'),
            is_correction: false,
            cursor_pos: 0,
        };

        assert!(manager.update_state(StateUpdate::AddKeystroke(keystroke)).is_ok());

        let current = manager.current_state().unwrap();
        assert_eq!(current.typed_text, "t");
        assert_eq!(current.cursor_position, 1);
    }
}
