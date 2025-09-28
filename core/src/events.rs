//! Stable event contracts for inter-crate communication
//!
//! This module defines the frozen event system contracts that enable coordinated
//! development across all crates. These interfaces MUST remain stable to prevent
//! breaking changes during parallel development.
//!
//! ## Event System Contract
//!
//! Events flow through the system in a predictable pattern:
//! 1. Input events (KeyIn) are processed by the engine
//! 2. Game events (Hit/Miss) are generated from input processing
//! 3. Tick events provide regular timing updates
//! 4. Render events trigger UI updates
//! 5. SessionComplete events finalize scoring
//!
//! ## Performance Requirements
//!
//! - Event processing: P99 < 5ms per event
//! - Event queue depth: < 100 events under normal load
//! - Memory overhead: < 1KB per event batch
//!
//! ## Error Classification
//!
//! Typing errors are classified using the Damerau-Levenshtein distance algorithm:
//! - Substitution: Wrong character (a → b)
//! - Insertion: Extra character (abc → abdc)
//! - Deletion: Missing character (abc → ac)
//! - Transposition: Adjacent swap (abc → acb)

use crossterm::event::{KeyCode, KeyModifiers};
use serde::{Deserialize, Serialize};
use std::time::Duration;

use crate::types::{SessionResult, LevelId};

/// Core game events that flow through the system
///
/// This enum defines the complete set of events that can occur during a typing session.
/// All events include timing information for performance analysis and latency tracking.
///
/// **FROZEN CONTRACT**: This enum must remain stable for coordinated development.
/// Changes require coordination across all development teams.
///
/// Note: Instant is not serializable, so we use timestamps as Duration from epoch
/// for events that need persistence.
#[derive(Debug, Clone, PartialEq)]
pub enum GameEvent {
    /// User input event with timing and modifiers
    KeyIn {
        /// The key that was pressed (stored as string for serialization)
        key: String,
        /// Milliseconds since session start
        timestamp_ms: u64,
        /// Modifier keys held during press (stored as bitflags)
        modifiers: u8,
        /// Raw input latency measurement in microseconds
        input_latency_us: u64,
    },

    /// Successful character match
    Hit {
        /// Position in target text
        position: usize,
        /// Expected character
        expected: char,
        /// Actually typed character
        actual: char,
        /// Milliseconds since session start
        timestamp_ms: u64,
        /// Time since last keystroke in microseconds
        keystroke_interval_us: u64,
    },

    /// Typing error with classification
    Miss {
        /// Type of error detected
        error_type: ErrorType,
        /// Position in target text where error occurred
        position: usize,
        /// Expected character at this position
        expected: char,
        /// Actually typed character
        actual: char,
        /// Milliseconds since session start
        timestamp_ms: u64,
        /// Time since last keystroke in microseconds
        keystroke_interval_us: u64,
    },

    /// Regular timing tick for live updates
    Tick {
        /// Total elapsed milliseconds since session start
        elapsed_ms: u64,
        /// Progress through current session (0.0-1.0)
        session_progress: f64,
        /// Milliseconds since session start
        timestamp_ms: u64,
    },

    /// Render frame update event
    Render {
        /// Time taken to render this frame in microseconds
        frame_time_us: u64,
        /// Components that were updated
        components: Vec<ComponentUpdate>,
        /// Milliseconds since session start
        timestamp_ms: u64,
    },

    /// Session completion event
    SessionComplete {
        /// Final session results
        result: SessionResult,
        /// Milliseconds since session start
        timestamp_ms: u64,
    },

    /// Emergency quit signal
    Quit {
        /// Reason for quitting
        reason: QuitReason,
        /// Milliseconds since session start
        timestamp_ms: u64,
    },

    /// Session pause/resume
    Pause {
        /// Whether pausing (true) or resuming (false)
        paused: bool,
        /// Milliseconds since session start
        timestamp_ms: u64,
    },

    /// Level change event
    LevelChange {
        /// Previous level (None if starting)
        from_level: Option<LevelId>,
        /// New level
        to_level: LevelId,
        /// Milliseconds since session start
        timestamp_ms: u64,
    },
}

impl GameEvent {
    /// Convert KeyCode to string for serialization
    pub fn key_code_to_string(key: KeyCode) -> String {
        match key {
            KeyCode::Backspace => "Backspace".to_string(),
            KeyCode::Enter => "Enter".to_string(),
            KeyCode::Left => "Left".to_string(),
            KeyCode::Right => "Right".to_string(),
            KeyCode::Up => "Up".to_string(),
            KeyCode::Down => "Down".to_string(),
            KeyCode::Home => "Home".to_string(),
            KeyCode::End => "End".to_string(),
            KeyCode::PageUp => "PageUp".to_string(),
            KeyCode::PageDown => "PageDown".to_string(),
            KeyCode::Tab => "Tab".to_string(),
            KeyCode::BackTab => "BackTab".to_string(),
            KeyCode::Delete => "Delete".to_string(),
            KeyCode::Insert => "Insert".to_string(),
            KeyCode::F(n) => format!("F{}", n),
            KeyCode::Char(c) => c.to_string(),
            KeyCode::Null => "Null".to_string(),
            KeyCode::Esc => "Esc".to_string(),
            KeyCode::CapsLock => "CapsLock".to_string(),
            KeyCode::ScrollLock => "ScrollLock".to_string(),
            KeyCode::NumLock => "NumLock".to_string(),
            KeyCode::PrintScreen => "PrintScreen".to_string(),
            KeyCode::Pause => "Pause".to_string(),
            KeyCode::Menu => "Menu".to_string(),
            KeyCode::KeypadBegin => "KeypadBegin".to_string(),
            KeyCode::Media(media) => format!("Media({:?})", media),
            KeyCode::Modifier(modifier) => format!("Modifier({:?})", modifier),
        }
    }

    /// Convert KeyModifiers to u8 for serialization
    pub fn modifiers_to_u8(modifiers: KeyModifiers) -> u8 {
        modifiers.bits()
    }

    /// Create a KeyIn event from crossterm types
    pub fn key_in(key: KeyCode, modifiers: KeyModifiers, timestamp_ms: u64, input_latency: Duration) -> Self {
        Self::KeyIn {
            key: Self::key_code_to_string(key),
            timestamp_ms,
            modifiers: Self::modifiers_to_u8(modifiers),
            input_latency_us: input_latency.as_micros() as u64,
        }
    }

    /// Get timestamp as Duration for processing
    pub fn timestamp(&self) -> Duration {
        let ms = match self {
            GameEvent::KeyIn { timestamp_ms, .. } => *timestamp_ms,
            GameEvent::Hit { timestamp_ms, .. } => *timestamp_ms,
            GameEvent::Miss { timestamp_ms, .. } => *timestamp_ms,
            GameEvent::Tick { timestamp_ms, .. } => *timestamp_ms,
            GameEvent::Render { timestamp_ms, .. } => *timestamp_ms,
            GameEvent::SessionComplete { timestamp_ms, .. } => *timestamp_ms,
            GameEvent::Quit { timestamp_ms, .. } => *timestamp_ms,
            GameEvent::Pause { timestamp_ms, .. } => *timestamp_ms,
            GameEvent::LevelChange { timestamp_ms, .. } => *timestamp_ms,
        };
        Duration::from_millis(ms)
    }
}

/// Classification of typing errors using Damerau-Levenshtein algorithm
///
/// These error types correspond to the four basic edit operations in the
/// Damerau-Levenshtein distance algorithm. They enable detailed error
/// analysis and targeted practice recommendations.
///
/// **FROZEN CONTRACT**: This enum must remain stable for coordinated development.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ErrorType {
    /// Wrong character typed (substitution)
    ///
    /// Example: "cat" → "cbt" (a → b substitution)
    /// Weight: 2.0 (moderate severity)
    Substitution,

    /// Extra character typed (insertion)
    ///
    /// Example: "cat" → "caxt" (x inserted)
    /// Weight: 1.0 (least severe, often corrected quickly)
    Insertion,

    /// Missing character (deletion)
    ///
    /// Example: "cat" → "ct" (a deleted)
    /// Weight: 1.0 (least severe)
    Deletion,

    /// Adjacent characters swapped (transposition)
    ///
    /// Example: "cat" → "act" (c and a swapped)
    /// Weight: 3.0 (most severe, indicates finger coordination issues)
    Transposition,
}

impl ErrorType {
    /// Get severity weight for scoring calculations
    pub fn weight(&self) -> f64 {
        match self {
            ErrorType::Insertion | ErrorType::Deletion => 1.0,
            ErrorType::Substitution => 2.0,
            ErrorType::Transposition => 3.0,
        }
    }

    /// Get human-readable description
    pub fn description(&self) -> &'static str {
        match self {
            ErrorType::Substitution => "Wrong character",
            ErrorType::Insertion => "Extra character",
            ErrorType::Deletion => "Missing character",
            ErrorType::Transposition => "Characters swapped",
        }
    }

    /// Get practice recommendation for this error type
    pub fn practice_recommendation(&self) -> &'static str {
        match self {
            ErrorType::Substitution => "Focus on accuracy over speed",
            ErrorType::Insertion => "Practice deliberate keystrokes",
            ErrorType::Deletion => "Ensure complete key presses",
            ErrorType::Transposition => "Work on finger independence",
        }
    }
}

/// Reasons for session termination
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum QuitReason {
    /// User pressed Ctrl+C or Escape
    UserRequest,
    /// System error or crash
    SystemError,
    /// Performance target exceeded (emergency stop)
    PerformanceTimeout,
    /// Session completed normally
    Completion,
}

/// UI component update information
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ComponentUpdate {
    /// Text area update
    TextArea {
        /// New cursor position
        cursor_pos: usize,
        /// Highlighted error positions
        error_positions: Vec<usize>,
    },
    /// Live metrics display update
    MetricsDisplay {
        /// Current WPM
        wpm: f64,
        /// Current accuracy percentage
        accuracy: f64,
        /// Current error count
        errors: u32,
    },
    /// Progress bar update
    ProgressBar {
        /// Progress percentage (0.0-1.0)
        progress: f64,
    },
    /// Status message update
    StatusMessage {
        /// New status message
        message: String,
        /// Message type (info, warning, error)
        message_type: MessageType,
    },
}

/// Message types for status updates
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MessageType {
    Info,
    Warning,
    Error,
    Success,
}

/// Event batch for efficient processing
///
/// Events can be batched together for performance optimization,
/// especially during high-frequency input periods.
#[derive(Debug, Clone)]
pub struct EventBatch {
    /// Events in this batch
    pub events: Vec<GameEvent>,
    /// Batch creation timestamp (milliseconds since session start)
    pub batch_timestamp_ms: u64,
    /// Batch sequence number for ordering
    pub sequence: u64,
}

impl EventBatch {
    /// Create new event batch
    pub fn new(events: Vec<GameEvent>, sequence: u64, timestamp_ms: u64) -> Self {
        Self {
            events,
            batch_timestamp_ms: timestamp_ms,
            sequence,
        }
    }

    /// Check if batch should be processed immediately
    pub fn is_urgent(&self) -> bool {
        // Quit events and errors are urgent
        self.events.iter().any(|event| {
            matches!(
                event,
                GameEvent::Quit { .. } | GameEvent::Miss { .. }
            )
        })
    }

    /// Get batch processing priority (higher = more urgent)
    pub fn priority(&self) -> u8 {
        if self.is_urgent() {
            255
        } else if self.events.iter().any(|e| matches!(e, GameEvent::KeyIn { .. })) {
            128
        } else {
            64
        }
    }
}

/// Event processing result
#[derive(Debug, Clone)]
pub struct EventProcessingResult {
    /// Whether processing was successful
    pub success: bool,
    /// Processing latency in microseconds
    pub processing_time_us: u64,
    /// Number of events processed
    pub events_processed: usize,
    /// Any errors that occurred
    pub errors: Vec<String>,
}

/// Event system performance metrics
#[derive(Debug, Clone)]
pub struct EventSystemMetrics {
    /// Total events processed
    pub total_events: u64,
    /// Events processed per second
    pub events_per_second: f64,
    /// Average processing latency in microseconds
    pub avg_processing_time_us: u64,
    /// P99 processing latency in microseconds
    pub p99_processing_time_us: u64,
    /// Current queue depth
    pub queue_depth: usize,
    /// Number of dropped events (performance failures)
    pub dropped_events: u64,
}

impl EventSystemMetrics {
    /// Check if performance targets are met
    pub fn meets_targets(&self) -> bool {
        let p99_duration = Duration::from_micros(self.p99_processing_time_us);
        p99_duration <= Duration::from_millis(5)
            && self.queue_depth <= 100
            && self.dropped_events == 0
    }
}

/// Trait for components that can process game events
pub trait EventProcessor {
    /// Process a single event
    fn process_event(&mut self, event: GameEvent) -> Result<EventProcessingResult, crate::types::CentotypeError>;

    /// Process a batch of events
    fn process_batch(&mut self, batch: EventBatch) -> Result<EventProcessingResult, crate::types::CentotypeError> {
        let start = std::time::Instant::now();
        let mut total_events = 0;
        let mut errors = Vec::new();

        for event in batch.events {
            match self.process_event(event) {
                Ok(result) => {
                    total_events += result.events_processed;
                    errors.extend(result.errors);
                }
                Err(e) => {
                    errors.push(e.to_string());
                }
            }
        }

        Ok(EventProcessingResult {
            success: errors.is_empty(),
            processing_time_us: start.elapsed().as_micros() as u64,
            events_processed: total_events,
            errors,
        })
    }

    /// Get current performance metrics
    fn get_metrics(&self) -> EventSystemMetrics;
}

/// Trait for components that can emit events
pub trait EventEmitter {
    /// Emit a single event
    fn emit_event(&self, event: GameEvent) -> Result<(), crate::types::CentotypeError>;

    /// Emit a batch of events
    fn emit_batch(&self, batch: EventBatch) -> Result<(), crate::types::CentotypeError>;
}

/// Event router for distributing events to multiple processors
pub struct EventRouter {
    processors: Vec<Box<dyn EventProcessor + Send + Sync>>,
    metrics: EventSystemMetrics,
}

impl EventRouter {
    /// Create new event router
    pub fn new() -> Self {
        Self {
            processors: Vec::new(),
            metrics: EventSystemMetrics {
                total_events: 0,
                events_per_second: 0.0,
                avg_processing_time_us: 0,
                p99_processing_time_us: 0,
                queue_depth: 0,
                dropped_events: 0,
            },
        }
    }

    /// Add event processor
    pub fn add_processor(&mut self, processor: Box<dyn EventProcessor + Send + Sync>) {
        self.processors.push(processor);
    }

    /// Route event to all processors
    pub fn route_event(&mut self, event: GameEvent) -> Result<Vec<EventProcessingResult>, crate::types::CentotypeError> {
        let mut results = Vec::new();

        for processor in &mut self.processors {
            match processor.process_event(event.clone()) {
                Ok(result) => results.push(result),
                Err(e) => return Err(e),
            }
        }

        self.metrics.total_events += 1;
        Ok(results)
    }
}

impl Default for EventRouter {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crossterm::event::KeyCode;

    #[test]
    fn test_error_type_weights() {
        assert_eq!(ErrorType::Insertion.weight(), 1.0);
        assert_eq!(ErrorType::Deletion.weight(), 1.0);
        assert_eq!(ErrorType::Substitution.weight(), 2.0);
        assert_eq!(ErrorType::Transposition.weight(), 3.0);
    }

    #[test]
    fn test_event_batch_priority() {
        let normal_events = vec![GameEvent::Tick {
            elapsed_ms: 1000,
            session_progress: 0.5,
            timestamp_ms: 1000,
        }];
        let batch = EventBatch::new(normal_events, 1, 1000);
        assert_eq!(batch.priority(), 64);

        let urgent_events = vec![GameEvent::Quit {
            reason: QuitReason::UserRequest,
            timestamp_ms: 1000,
        }];
        let urgent_batch = EventBatch::new(urgent_events, 2, 1000);
        assert_eq!(urgent_batch.priority(), 255);
    }

    #[test]
    fn test_keycode_conversion() {
        let key_str = GameEvent::key_code_to_string(KeyCode::Char('a'));
        assert_eq!(key_str, "a");

        let key_str = GameEvent::key_code_to_string(KeyCode::Enter);
        assert_eq!(key_str, "Enter");

        let key_str = GameEvent::key_code_to_string(KeyCode::F(1));
        assert_eq!(key_str, "F1");
    }

    #[test]
    fn test_event_timestamp() {
        let event = GameEvent::Hit {
            position: 10,
            expected: 'a',
            actual: 'a',
            timestamp_ms: 1500,
            keystroke_interval_us: 100_000,
        };

        assert_eq!(event.timestamp(), Duration::from_millis(1500));
    }

    #[test]
    fn test_metrics_targets() {
        let good_metrics = EventSystemMetrics {
            total_events: 1000,
            events_per_second: 200.0,
            avg_processing_time_us: 2000,
            p99_processing_time_us: 4000, // 4ms
            queue_depth: 50,
            dropped_events: 0,
        };
        assert!(good_metrics.meets_targets());

        let bad_metrics = EventSystemMetrics {
            total_events: 1000,
            events_per_second: 200.0,
            avg_processing_time_us: 2000,
            p99_processing_time_us: 10_000, // 10ms - exceeds 5ms target
            queue_depth: 50,
            dropped_events: 0,
        };
        assert!(!bad_metrics.meets_targets());
    }
}