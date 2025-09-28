//! Terminal state management with reliable cleanup and RAII pattern
use centotype_core::types::*;
use crossterm::{
    cursor,
    event::{DisableMouseCapture, EnableMouseCapture},
    execute,
    terminal::{disable_raw_mode, enable_raw_mode, EnterAlternateScreen, LeaveAlternateScreen},
};
use std::io::{self, Write};
use tracing::{debug, error, info, warn};

/// Terminal manager with automatic cleanup on drop
pub struct TtyManager {
    /// Original terminal state before modifications
    original_state: TerminalState,
    /// Current terminal state
    current_state: TerminalState,
    /// Whether we're in an active session requiring cleanup
    needs_cleanup: bool,
}

/// Terminal state tracking
#[derive(Debug, Clone, Copy, PartialEq)]
struct TerminalState {
    raw_mode: bool,
    alternate_screen: bool,
    mouse_capture: bool,
}

impl Default for TerminalState {
    fn default() -> Self {
        Self {
            raw_mode: false,
            alternate_screen: false,
            mouse_capture: false,
        }
    }
}

impl TtyManager {
    /// Create new TTY manager and capture initial state
    pub fn new() -> Result<Self> {
        let original_state = Self::detect_current_state()?;

        debug!("TTY manager initialized with state: {:?}", original_state);

        Ok(Self {
            original_state,
            current_state: original_state,
            needs_cleanup: false,
        })
    }

    /// Enter typing mode: raw mode + alternate screen + mouse capture
    pub fn enter_typing_mode(&mut self) -> Result<()> {
        info!("Entering typing mode");

        // Enable raw mode for immediate input
        if !self.current_state.raw_mode {
            enable_raw_mode().map_err(|e| {
                CentotypeError::Platform(format!("Failed to enable raw mode: {}", e))
            })?;
            self.current_state.raw_mode = true;
            debug!("Raw mode enabled");
        }

        // Enter alternate screen to preserve user's terminal content
        if !self.current_state.alternate_screen {
            execute!(io::stdout(), EnterAlternateScreen).map_err(|e| {
                CentotypeError::Platform(format!("Failed to enter alternate screen: {}", e))
            })?;
            self.current_state.alternate_screen = true;
            debug!("Alternate screen enabled");
        }

        // Enable mouse capture for potential future features
        if !self.current_state.mouse_capture {
            execute!(io::stdout(), EnableMouseCapture).map_err(|e| {
                CentotypeError::Platform(format!("Failed to enable mouse capture: {}", e))
            })?;
            self.current_state.mouse_capture = true;
            debug!("Mouse capture enabled");
        }

        // Hide cursor for cleaner typing interface
        execute!(io::stdout(), cursor::Hide).map_err(|e| {
            CentotypeError::Platform(format!("Failed to hide cursor: {}", e))
        })?;

        self.needs_cleanup = true;
        info!("Typing mode active");
        Ok(())
    }

    /// Exit typing mode: restore original terminal state
    pub fn exit_typing_mode(&mut self) -> Result<()> {
        info!("Exiting typing mode");

        // Show cursor
        if let Err(e) = execute!(io::stdout(), cursor::Show) {
            warn!("Failed to show cursor during cleanup: {}", e);
        }

        // Disable mouse capture
        if self.current_state.mouse_capture {
            if let Err(e) = execute!(io::stdout(), DisableMouseCapture) {
                warn!("Failed to disable mouse capture: {}", e);
            } else {
                self.current_state.mouse_capture = false;
                debug!("Mouse capture disabled");
            }
        }

        // Leave alternate screen
        if self.current_state.alternate_screen {
            if let Err(e) = execute!(io::stdout(), LeaveAlternateScreen) {
                error!("Failed to leave alternate screen: {}", e);
                // This is critical - user's terminal content could be lost
            } else {
                self.current_state.alternate_screen = false;
                debug!("Alternate screen disabled");
            }
        }

        // Disable raw mode
        if self.current_state.raw_mode {
            if let Err(e) = disable_raw_mode() {
                error!("Failed to disable raw mode: {}", e);
                // This is critical - terminal could be left in broken state
            } else {
                self.current_state.raw_mode = false;
                debug!("Raw mode disabled");
            }
        }

        self.needs_cleanup = false;
        info!("Terminal state restored");
        Ok(())
    }

    /// Emergency cleanup - called on panic or critical error
    pub fn emergency_cleanup(&mut self) {
        warn!("Emergency TTY cleanup initiated");

        // Best effort cleanup - don't propagate errors
        let _ = execute!(io::stdout(), cursor::Show);

        if self.current_state.mouse_capture {
            let _ = execute!(io::stdout(), DisableMouseCapture);
        }

        if self.current_state.alternate_screen {
            let _ = execute!(io::stdout(), LeaveAlternateScreen);
        }

        if self.current_state.raw_mode {
            let _ = disable_raw_mode();
        }

        // Flush output to ensure commands are executed
        let _ = io::stdout().flush();

        self.needs_cleanup = false;
        warn!("Emergency cleanup completed");
    }

    /// Check if terminal is in typing mode
    pub fn is_in_typing_mode(&self) -> bool {
        self.current_state.raw_mode && self.current_state.alternate_screen
    }

    /// Get current terminal size
    pub fn get_size(&self) -> Result<(u16, u16)> {
        crossterm::terminal::size().map_err(|e| {
            CentotypeError::Platform(format!("Failed to get terminal size: {}", e))
        })
    }

    /// Clear the screen
    pub fn clear_screen(&self) -> Result<()> {
        execute!(
            io::stdout(),
            crossterm::terminal::Clear(crossterm::terminal::ClearType::All)
        )
        .map_err(|e| CentotypeError::Platform(format!("Failed to clear screen: {}", e)))
    }

    /// Move cursor to position
    pub fn move_cursor(&self, x: u16, y: u16) -> Result<()> {
        execute!(io::stdout(), cursor::MoveTo(x, y)).map_err(|e| {
            CentotypeError::Platform(format!("Failed to move cursor: {}", e))
        })
    }

    /// Flush output buffer
    pub fn flush(&self) -> Result<()> {
        io::stdout().flush().map_err(|e| {
            CentotypeError::Platform(format!("Failed to flush output: {}", e))
        })
    }

    // Private helper methods

    fn detect_current_state() -> Result<TerminalState> {
        // For now, assume terminal starts in normal state
        // In a full implementation, we might try to detect actual state
        Ok(TerminalState::default())
    }
}

impl Drop for TtyManager {
    /// Ensure terminal state is restored when TTY manager is dropped
    fn drop(&mut self) {
        if self.needs_cleanup {
            warn!("TTY manager dropped with active session - performing emergency cleanup");
            self.emergency_cleanup();
        }
    }
}

/// Terminal guard that ensures cleanup on scope exit
pub struct TypingModeGuard<'a> {
    tty: &'a mut TtyManager,
}

impl<'a> TypingModeGuard<'a> {
    pub fn new(tty: &'a mut TtyManager) -> Result<Self> {
        tty.enter_typing_mode()?;
        Ok(Self { tty })
    }
}

impl<'a> Drop for TypingModeGuard<'a> {
    fn drop(&mut self) {
        if let Err(e) = self.tty.exit_typing_mode() {
            error!("Failed to exit typing mode in guard: {}", e);
        }
    }
}

/// Re-export for convenience
pub use TtyManager as Tty;

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_tty_manager_creation() {
        let tty = TtyManager::new().unwrap();
        assert!(!tty.is_in_typing_mode());
        assert!(!tty.needs_cleanup);
    }

    #[test]
    fn test_state_tracking() {
        let tty = TtyManager::new().unwrap();
        let initial_state = tty.current_state;
        assert!(!initial_state.raw_mode);
        assert!(!initial_state.alternate_screen);
        assert!(!initial_state.mouse_capture);
    }

    #[test]
    fn test_emergency_cleanup_safe() {
        let mut tty = TtyManager::new().unwrap();
        tty.needs_cleanup = true;

        // Should not panic
        tty.emergency_cleanup();
        assert!(!tty.needs_cleanup);
    }
}
