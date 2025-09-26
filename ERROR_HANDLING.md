# Error Handling Patterns and Shared Error Types

## Overview

Centotype uses a layered error handling approach with domain-specific error types that can be mapped to a common `CentotypeError` enum. This design ensures:

- **Consistent error reporting** across all crates
- **Contextual error information** for debugging and user feedback
- **Graceful degradation** when non-critical failures occur
- **Recovery strategies** for different error categories

## Error Hierarchy

```rust
// Already defined in core/src/types.rs
pub enum CentotypeError {
    Config(String),
    Content(String),
    Persistence(String),
    Platform(String),
    Input(String),
    State(String),
    Io(#[from] std::io::Error),
    Serialization(#[from] serde_json::Error),
}
```

## Per-Crate Error Patterns

### Core Crate Errors

**State Management Errors:**
```rust
// Session state validation
if self.current_session.is_none() {
    return Err(CentotypeError::State("No active session".to_string()));
}

// Invalid state transitions
if state.is_completed {
    return Err(CentotypeError::State("Cannot modify completed session".to_string()));
}
```

**Scoring Engine Errors:**
```rust
// Performance validation
if duration.as_secs_f64() == 0.0 {
    return Err(CentotypeError::State("Invalid session duration".to_string()));
}

// Data consistency checks
if target.is_empty() || typed.len() > target.len() * 2 {
    return Err(CentotypeError::State("Invalid text data for scoring".to_string()));
}
```

### Engine Crate Errors

**Input Processing Errors:**
```rust
// Terminal input failures
if let Err(e) = crossterm::event::read() {
    return Err(CentotypeError::Input(format!("Failed to read input: {}", e)));
}

// Performance target violations
if latency > Duration::from_millis(25) {
    tracing::warn!("Input latency exceeded target: {:?}", latency);
    // Continue execution but log performance issue
}
```

**Render Engine Errors:**
```rust
// Non-critical render failures (graceful degradation)
if let Err(e) = terminal.draw(|f| self.draw_ui(f, state)) {
    tracing::warn!("Render frame failed: {}", e);
    // Continue with previous frame, don't crash
    return Ok(());
}

// Critical render failures
if terminal.is_none() {
    return Err(CentotypeError::Platform("Terminal not initialized".to_string()));
}
```

### Platform Crate Errors

**Platform Detection Errors:**
```rust
// Unsupported platform
if !validation.is_supported {
    return Err(CentotypeError::Platform(format!(
        "Platform not supported: {}",
        validation.issues.join(", ")
    )));
}

// Feature availability warnings (non-fatal)
if !self.terminal_caps.supports_color {
    tracing::warn!("Color not supported, using monochrome mode");
    // Continue execution with fallback
}
```

### Content Crate Errors

**Content Generation Errors:**
```rust
// Generation failures
if params.length_chars == 0 {
    return Err(CentotypeError::Content("Invalid content length".to_string()));
}

// Validation failures
if difficulty.overall_score < expected_range.0 {
    return Err(CentotypeError::Content(format!(
        "Generated content too easy for tier {}: {:.2} < {:.2}",
        tier.0, difficulty.overall_score, expected_range.0
    )));
}
```

### Persistence Crate Errors

**File System Errors:**
```rust
// Configuration loading with fallback
pub fn load_config(&self) -> Result<Config> {
    let config_path = self.config_dir.join("config.toml");
    if config_path.exists() {
        match std::fs::read_to_string(&config_path) {
            Ok(content) => toml::from_str(&content).map_err(|e| {
                CentotypeError::Config(format!("Invalid config format: {}", e))
            }),
            Err(e) => {
                tracing::warn!("Failed to read config file: {}", e);
                Ok(Config::default()) // Use default config as fallback
            }
        }
    } else {
        Ok(Config::default())
    }
}

// Atomic write operations
pub fn save_profile(&self, profile: &UserProgress) -> Result<()> {
    let profile_path = self.data_dir.join("profile.json");
    let temp_path = profile_path.with_extension("json.tmp");

    // Write to temporary file first
    let content = serde_json::to_string_pretty(profile)
        .map_err(|e| CentotypeError::Serialization(e))?;

    std::fs::write(&temp_path, content)
        .map_err(|e| CentotypeError::Persistence(format!("Failed to write temp file: {}", e)))?;

    // Atomic rename
    std::fs::rename(temp_path, profile_path)
        .map_err(|e| CentotypeError::Persistence(format!("Failed to save profile: {}", e)))?;

    Ok(())
}
```

## Error Recovery Strategies

### Engine Recovery

```rust
impl CentotypeEngine {
    pub fn emergency_shutdown(&mut self) {
        // Critical: Always restore terminal state
        if let Err(e) = self.tty_manager.exit_raw_mode() {
            eprintln!("CRITICAL: Failed to restore terminal: {}", e);
        }

        // Stop input processing
        let _ = self.input_processor.stop();

        // Mark as not running
        *self.is_running.lock() = false;

        tracing::error!("Emergency shutdown completed");
    }

    fn handle_recoverable_error(&mut self, error: &CentotypeError) -> bool {
        match error {
            CentotypeError::Input(_) => {
                // Reset input processor
                if let Ok(()) = self.input_processor.restart() {
                    tracing::info!("Input processor restarted successfully");
                    true
                } else {
                    false
                }
            }
            CentotypeError::Platform(_) => {
                // Apply fallback mode
                self.renderer.enable_fallback_mode();
                tracing::info!("Switched to fallback rendering mode");
                true
            }
            _ => false, // Not recoverable
        }
    }
}
```

### Session Recovery

```rust
impl SessionManager {
    pub fn recover_from_interruption(&mut self) -> Result<()> {
        if let Some(session) = &mut self.current_session {
            // Validate session state
            if session.keystrokes.is_empty() {
                return Err(CentotypeError::State("No keystrokes to recover".to_string()));
            }

            // Recalculate state from keystrokes
            let mut typed_text = String::new();
            let mut cursor_pos = 0;

            for keystroke in &session.keystrokes {
                match keystroke.char_typed {
                    Some(ch) if !keystroke.is_correction => {
                        typed_text.push(ch);
                        cursor_pos += 1;
                    }
                    None if keystroke.is_correction => {
                        if !typed_text.is_empty() {
                            typed_text.pop();
                            cursor_pos = cursor_pos.saturating_sub(1);
                        }
                    }
                    _ => {}
                }
            }

            session.typed_text = typed_text;
            session.cursor_position = cursor_pos;

            tracing::info!("Session state recovered successfully");
        }
        Ok(())
    }
}
```

## Error Context Enhancement

### Adding Context to Errors

```rust
use std::fmt;

// Enhanced error with context
#[derive(Debug)]
pub struct ErrorContext {
    pub error: CentotypeError,
    pub operation: String,
    pub session_id: Option<uuid::Uuid>,
    pub timestamp: chrono::DateTime<chrono::Utc>,
    pub platform_info: Option<String>,
}

impl ErrorContext {
    pub fn new(error: CentotypeError, operation: &str) -> Self {
        Self {
            error,
            operation: operation.to_string(),
            session_id: None,
            timestamp: chrono::Utc::now(),
            platform_info: None,
        }
    }

    pub fn with_session_id(mut self, session_id: uuid::Uuid) -> Self {
        self.session_id = Some(session_id);
        self
    }

    pub fn with_platform_info(mut self, info: String) -> Self {
        self.platform_info = Some(info);
        self
    }
}

impl fmt::Display for ErrorContext {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "Error in {}: {}", self.operation, self.error)?;
        if let Some(session_id) = self.session_id {
            write!(f, " (session: {})", session_id)?;
        }
        if let Some(platform) = &self.platform_info {
            write!(f, " (platform: {})", platform)?;
        }
        Ok(())
    }
}
```

## Error Reporting and Telemetry

### User-Friendly Error Messages

```rust
impl CentotypeError {
    pub fn user_message(&self) -> String {
        match self {
            CentotypeError::Config(msg) => {
                format!("Configuration problem: {}. Please check your config file.", msg)
            }
            CentotypeError::Platform(msg) => {
                format!("Platform compatibility issue: {}. Try using --fallback mode.", msg)
            }
            CentotypeError::Input(msg) => {
                format!("Input processing error: {}. Press Ctrl+C to exit safely.", msg)
            }
            CentotypeError::Persistence(msg) => {
                format!("Unable to save progress: {}. Your session data may be lost.", msg)
            }
            _ => "An unexpected error occurred. Please check the logs for details.".to_string(),
        }
    }

    pub fn severity(&self) -> ErrorSeverity {
        match self {
            CentotypeError::Config(_) => ErrorSeverity::Warning,
            CentotypeError::Content(_) => ErrorSeverity::Warning,
            CentotypeError::Platform(_) => ErrorSeverity::Error,
            CentotypeError::Input(_) => ErrorSeverity::Critical,
            CentotypeError::State(_) => ErrorSeverity::Error,
            CentotypeError::Persistence(_) => ErrorSeverity::Warning,
            _ => ErrorSeverity::Error,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorSeverity {
    Info,
    Warning,
    Error,
    Critical,
}
```

## Testing Error Conditions

### Error Injection for Testing

```rust
#[cfg(test)]
mod error_tests {
    use super::*;

    #[test]
    fn test_session_recovery() {
        let mut manager = SessionManager::new();

        // Simulate interrupted session
        let session = SessionState {
            session_id: uuid::Uuid::new_v4(),
            mode: TrainingMode::Arcade { level: LevelId::new(1).unwrap() },
            target_text: "hello world".to_string(),
            typed_text: "hello".to_string(),
            cursor_position: 5,
            started_at: chrono::Utc::now(),
            paused_duration: Duration::default(),
            is_paused: false,
            is_completed: false,
            keystrokes: vec![
                Keystroke { timestamp: chrono::Utc::now(), char_typed: Some('h'), is_correction: false, cursor_pos: 0 },
                Keystroke { timestamp: chrono::Utc::now(), char_typed: Some('e'), is_correction: false, cursor_pos: 1 },
                // ... more keystrokes
            ],
        };

        manager.current_session = Some(session);

        // Test recovery
        assert!(manager.recover_from_interruption().is_ok());
        assert_eq!(manager.current_state().typed_text, "hello");
    }

    #[test]
    fn test_error_context() {
        let error = CentotypeError::Input("Mock input error".to_string());
        let context = ErrorContext::new(error, "process_keystroke")
            .with_session_id(uuid::Uuid::new_v4());

        assert!(context.to_string().contains("process_keystroke"));
        assert!(context.session_id.is_some());
    }
}
```

This error handling pattern ensures robust operation while providing clear diagnostics and recovery paths for different failure modes.