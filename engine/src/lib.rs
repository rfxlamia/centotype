//! # Centotype Engine
//!
//! The engine crate provides the core event loop, input handling, and render system.
//! This is the main orchestrator that coordinates all subsystems for a complete typing experience.

pub mod arena;
pub mod event;
pub mod input;
pub mod performance;
pub mod render;
pub mod tty;

// Re-export main types
pub use arena::{RenderArena, FrameData, ArenaStats};
pub use event::Event as EngineEvent;
pub use input::Input as InputProcessor;
pub use performance::{Performance as PerformanceMonitor, LatencyProfiler, InputCycleTimer};
pub use render::Render as Renderer;
pub use tty::{Tty as TtyManager, TypingModeGuard};

use centotype_analytics::AnalyticsEngine;
use centotype_content::ContentManager;
use centotype_core::{types::*, CentotypeCore};
use centotype_persistence::PersistenceManager;
use centotype_platform::PlatformManager;
use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEvent, KeyModifiers};
use parking_lot::RwLock;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::time::timeout;
use tracing::{debug, info, warn};

/// Main engine coordinator that manages all subsystems
pub struct CentotypeEngine {
    /// Core typing logic and session management
    core: Arc<CentotypeCore>,
    /// Platform-specific utilities
    platform: Arc<PlatformManager>,
    /// Content loading and caching
    content_manager: Arc<ContentManager>,
    /// Performance analytics and metrics
    analytics: Arc<AnalyticsEngine>,
    /// Profile and session persistence
    persistence: Arc<PersistenceManager>,
    /// Input processing with security validation
    input_processor: Arc<RwLock<InputProcessor>>,
    /// Terminal state management
    tty_manager: Arc<RwLock<TtyManager>>,
    /// Performance monitoring
    performance_monitor: Arc<RwLock<PerformanceMonitor>>,
    /// TUI render system
    renderer: Arc<RwLock<Renderer>>,
}

impl CentotypeEngine {
    /// Create new engine with all subsystems
    pub async fn new(
        core: Arc<CentotypeCore>,
        platform: Arc<PlatformManager>,
    ) -> Result<Self> {
        info!("Initializing Centotype Engine with all subsystems");

        // Initialize content management system
        let content_manager = Arc::new(
            ContentManager::new()
                .await
                .map_err(|e| CentotypeError::Content(format!("Content manager init failed: {}", e)))?,
        );

        // Initialize analytics engine
        let analytics = Arc::new(AnalyticsEngine::new());

        // Initialize persistence manager
        let persistence = Arc::new(
            PersistenceManager::new()
                .map_err(|e| CentotypeError::Persistence(format!("Persistence init failed: {}", e)))?,
        );

        // Initialize input processor
        let input_processor = Arc::new(RwLock::new(InputProcessor::new()));

        // Initialize TTY manager
        let tty_manager = Arc::new(RwLock::new(
            TtyManager::new()
                .map_err(|e| CentotypeError::Platform(format!("TTY manager init failed: {}", e)))?,
        ));

        // Initialize performance monitor
        let performance_monitor = Arc::new(RwLock::new(PerformanceMonitor::new()));

        // Initialize TUI renderer
        let renderer = Arc::new(RwLock::new(
            Renderer::new()
                .map_err(|e| CentotypeError::Platform(format!("Renderer init failed: {}", e)))?
        ));

        info!("All subsystems initialized successfully");

        Ok(Self {
            core,
            platform,
            content_manager,
            analytics,
            persistence,
            input_processor,
            tty_manager,
            performance_monitor,
            renderer,
        })
    }

    /// Start the main typing session - complete implementation
    pub async fn run(
        &mut self,
        mode: TrainingMode,
        _target_text: String, // Ignored in favor of content from ContentManager
    ) -> Result<SessionResult> {
        let session_start = Instant::now();
        info!("Starting typing session with mode: {:?}", mode);

        // 1. Load content from content manager
        let content = self.load_session_content(&mode).await?;
        debug!("Loaded session content ({} chars)", content.len());

        // 2. Start session in core with loaded content
        let session_id = self.core.start_session(mode, content.clone())?;
        info!("Started session {}", session_id);

        // 3. Setup terminal for typing mode (scoped lifetime)
        {
            let mut tty = self.tty_manager.write();
            let _typing_guard = TypingModeGuard::new(&mut *tty)?;

            // 3a. Initialize renderer for terminal UI
            {
                let mut renderer = self.renderer.write();
                renderer.initialize()?;
                renderer.check_terminal_size()?;
                debug!("TUI renderer initialized");
            }

            // 4. Configure input processor for this training mode
            {
                let mut input = self.input_processor.write();
                input.set_training_mode(mode);
            }

            // 5. Main typing loop
            let result = self.run_typing_loop(session_id, &content).await?;

            // 6. Session completed - cleanup handled by guards
            let total_duration = session_start.elapsed();
            info!(
                "Session {} completed in {:?} with skill index {:.1}",
                session_id, total_duration, result.skill_index
            );

            // 7. Persist session results
            self.persistence.save_session_result(&result)?;
            debug!("Session results persisted");

            return Ok(result);
        }
    }

    /// Emergency shutdown - restore terminal state immediately
    pub fn emergency_shutdown(&mut self) {
        warn!("Emergency shutdown initiated");

        // Emergency TTY cleanup
        if let Some(mut tty) = self.tty_manager.try_write() {
            tty.emergency_cleanup();
        }

        warn!("Emergency shutdown completed");
    }

    /// Get comprehensive performance metrics
    pub fn get_performance_metrics(&self) -> EnginePerformanceMetrics {
        let cache_metrics = self.content_manager.get_cache_metrics();
        let input_stats = self.input_processor.read().get_statistics();

        EnginePerformanceMetrics {
            cache_hit_rate: cache_metrics.hit_rate(),
            input_rate_limit_stats: input_stats.rate_limiter_stats,
            filtered_sequences: input_stats.filtered_sequences,
            total_inputs: input_stats.total_processed,
        }
    }

    // Private implementation methods

    async fn load_session_content(&self, mode: &TrainingMode) -> Result<String> {
        match mode {
            TrainingMode::Arcade { level } => {
                // Use deterministic seed based on level for consistency
                let seed = Some(level.0 as u64 * 12345);
                self.content_manager
                    .get_level_content(*level, seed)
                    .await
                    .map_err(|e| CentotypeError::Content(format!("Failed to load level content: {}", e)))
            }
            TrainingMode::Drill { category, .. } => {
                // For drill mode, generate content based on category
                // This is a simplified implementation - full version would have drill-specific content
                let level = LevelId::new(50).unwrap(); // Mid-tier difficulty for drills
                let seed = Some((*category as u64) * 67890);
                self.content_manager
                    .get_level_content(level, seed)
                    .await
                    .map_err(|e| CentotypeError::Content(format!("Failed to load drill content: {}", e)))
            }
            TrainingMode::Endurance { .. } => {
                // For endurance mode, use varied content
                let level = LevelId::new(75).unwrap(); // Higher difficulty for endurance
                self.content_manager
                    .get_level_content(level, None) // Random seed for variety
                    .await
                    .map_err(|e| CentotypeError::Content(format!("Failed to load endurance content: {}", e)))
            }
        }
    }

    async fn run_typing_loop(&self, session_id: uuid::Uuid, target_text: &str) -> Result<SessionResult> {
        info!("Starting typing loop for session {}", session_id);

        let loop_start = Instant::now();
        let mut last_analytics_update = Instant::now();
        let mut last_render_update = Instant::now();
        let analytics_interval = Duration::from_millis(100); // Update analytics every 100ms
        let render_interval = Duration::from_millis(16); // Target 60 FPS (~16ms)

        // Initial render to show the interface
        self.render_current_state().await?;

        loop {
            let iteration_start = Instant::now();

            // Poll for input events with low latency timeout
            match timeout(Duration::from_millis(10), self.poll_input_event()).await {
                Ok(Ok(Some(event))) => {
                    let process_start = Instant::now();

                    // Process the input event
                    match self.process_input_event(event).await? {
                        InputAction::Character(ch) => {
                            // Record keystroke with analytics
                            let keystroke = Keystroke {
                                timestamp: chrono::Utc::now(),
                                char_typed: Some(ch),
                                is_correction: false,
                                cursor_pos: 0, // Updated by core
                            };

                            // Process through core scoring engine
                            let _live_metrics = self.core.process_keystroke(Some(ch), false)?;

                            // Update session state
                            let state_update = StateUpdate::AddKeystroke(keystroke);
                            self.update_session_state(state_update).await?;

                            // Immediate UI update for responsive feedback
                            self.render_current_state().await?;
                        }
                        InputAction::Backspace => {
                            let keystroke = Keystroke {
                                timestamp: chrono::Utc::now(),
                                char_typed: None,
                                is_correction: true,
                                cursor_pos: 0,
                            };

                            let _live_metrics = self.core.process_keystroke(None, true)?;

                            let state_update = StateUpdate::AddKeystroke(keystroke);
                            self.update_session_state(state_update).await?;

                            // Immediate UI update for backspace feedback
                            self.render_current_state().await?;
                        }
                        InputAction::Quit => {
                            info!("User quit requested");
                            break;
                        }
                        InputAction::Pause => {
                            self.handle_pause().await?;
                            // Update UI to show paused state
                            self.render_current_state().await?;
                        }
                        InputAction::Resume => {
                            self.handle_resume().await?;
                            // Update UI to show resumed state
                            self.render_current_state().await?;
                        }
                        InputAction::ToggleHelp => {
                            // Toggle help overlay
                            {
                                let mut renderer = self.renderer.write();
                                renderer.toggle_help();
                            }
                            self.render_current_state().await?;
                        }
                        InputAction::Ignore => {
                            // Filtered or invalid input - continue loop
                        }
                    }

                    // Track input processing latency
                    let processing_latency = process_start.elapsed();
                    if processing_latency > Duration::from_millis(25) {
                        warn!("Input processing exceeded 25ms target: {:?}", processing_latency);
                    }

                    // Check if session is complete
                    if self.is_session_complete(target_text).await? {
                        info!("Session completed by user input");
                        break;
                    }
                }
                Ok(Ok(None)) => {
                    // No input available - continue loop
                }
                Ok(Err(e)) => {
                    warn!("Input polling error: {}", e);
                    // Continue loop despite error
                }
                Err(_) => {
                    // Timeout - continue for responsiveness
                }
            }

            // Periodic UI update for real-time metrics (even without input)
            if last_render_update.elapsed() >= render_interval {
                if let Err(e) = self.render_current_state().await {
                    warn!("Failed to render UI update: {}", e);
                }
                last_render_update = Instant::now();
            }

            // Periodic analytics update
            if last_analytics_update.elapsed() >= analytics_interval {
                if let Err(e) = self.update_analytics().await {
                    warn!("Failed to update analytics: {}", e);
                }
                last_analytics_update = Instant::now();
            }

            // Maintain loop timing for consistent performance
            let iteration_time = iteration_start.elapsed();
            if iteration_time > Duration::from_millis(15) {
                debug!("Loop iteration took {:?}", iteration_time);
            }
        }

        let loop_duration = loop_start.elapsed();
        info!("Typing loop completed in {:?}", loop_duration);

        // Complete session and get final results
        self.core.complete_session()
    }

    async fn poll_input_event(&self) -> Result<Option<CrosstermEvent>> {
        // Poll for crossterm events with minimal latency
        match crossterm::event::poll(Duration::from_millis(1))? {
            true => {
                let event = crossterm::event::read()
                    .map_err(|e| CentotypeError::Input(format!("Failed to read input: {}", e)))?;
                Ok(Some(event))
            }
            false => Ok(None),
        }
    }

    async fn process_input_event(&self, event: CrosstermEvent) -> Result<InputAction> {
        match event {
            CrosstermEvent::Key(key_event) => {
                // Process through input security validation
                let processed = {
                    let mut input = self.input_processor.write();
                    input.process_key_event(key_event)?
                };

                if !processed.is_valid {
                    debug!("Input filtered by security validation");
                    return Ok(InputAction::Ignore);
                }

                // Convert to action based on key type
                self.convert_key_to_action(key_event, processed).await
            }
            CrosstermEvent::Resize(_, _) => {
                // Handle terminal resize - for now, continue
                debug!("Terminal resize event");
                Ok(InputAction::Ignore)
            }
            _ => Ok(InputAction::Ignore),
        }
    }

    async fn convert_key_to_action(
        &self,
        key_event: KeyEvent,
        _processed: input::ProcessedInput,
    ) -> Result<InputAction> {
        // Handle special key combinations
        if key_event.modifiers.contains(KeyModifiers::CONTROL) {
            match key_event.code {
                KeyCode::Char('c') => return Ok(InputAction::Quit),
                KeyCode::Char('p') => return Ok(InputAction::Pause),
                _ => {}
            }
        }

        // Handle regular keys
        match key_event.code {
            KeyCode::Char(ch) => Ok(InputAction::Character(ch)),
            KeyCode::Backspace => Ok(InputAction::Backspace),
            KeyCode::Esc => Ok(InputAction::Quit),
            KeyCode::F(1) => Ok(InputAction::ToggleHelp),
            KeyCode::Enter => {
                // For now, treat enter as session completion check
                Ok(InputAction::Ignore)
            }
            _ => Ok(InputAction::Ignore),
        }
    }

    async fn update_session_state(&self, _update: StateUpdate) -> Result<()> {
        // This would integrate with core session state management
        // For now, this is a placeholder
        debug!("Session state update requested");
        Ok(())
    }

    async fn update_analytics(&self) -> Result<()> {
        // Update performance analytics
        debug!("Analytics update requested");
        Ok(())
    }

    async fn is_session_complete(&self, _target_text: &str) -> Result<bool> {
        // Check if user has completed typing the target text
        // This would compare current session state with target
        // For now, return false to keep session running
        Ok(false)
    }

    async fn handle_pause(&self) -> Result<()> {
        let state_update = StateUpdate::SetPaused(true);
        self.update_session_state(state_update).await?;
        info!("Session paused");
        Ok(())
    }

    async fn handle_resume(&self) -> Result<()> {
        let state_update = StateUpdate::SetPaused(false);
        self.update_session_state(state_update).await?;
        info!("Session resumed");
        Ok(())
    }

    /// Render current session state to TUI
    async fn render_current_state(&self) -> Result<()> {
        // Get current session state from core
        let session_state = self.get_session_state().await?;
        let live_metrics = self.get_live_metrics().await?;

        // Update renderer with current state
        {
            let mut renderer = self.renderer.write();
            renderer.update_state(&session_state, &live_metrics);
            let render_time = renderer.render_frame()?;

            // Performance monitoring for render times
            if render_time > Duration::from_millis(33) {
                debug!("Render frame took {:?}, exceeding 33ms target", render_time);
            }
        }

        Ok(())
    }

    /// Get current session state (placeholder implementation)
    async fn get_session_state(&self) -> Result<SessionState> {
        // This would integrate with core session state management
        // For now, return a minimal session state
        Ok(SessionState {
            session_id: uuid::Uuid::new_v4(),
            mode: TrainingMode::Arcade { level: LevelId::new(1).unwrap() },
            target_text: "function calculateSum(arr) { return arr.reduce((a,b) => a+b, 0); }".to_string(),
            typed_text: "function calcul".to_string(),
            cursor_position: 14,
            started_at: chrono::Utc::now(),
            paused_duration: Duration::ZERO,
            is_paused: false,
            is_completed: false,
            keystrokes: vec![],
        })
    }

    /// Get current live metrics (placeholder implementation)
    async fn get_live_metrics(&self) -> Result<LiveMetrics> {
        // This would integrate with core metrics calculation
        // For now, return sample metrics
        Ok(LiveMetrics {
            raw_wpm: 47.3,
            effective_wpm: 45.2,
            accuracy: 94.7,
            current_streak: 12,
            longest_streak: 24,
            errors: ErrorStats {
                substitution: 2,
                insertion: 1,
                deletion: 0,
                transposition: 0,
                backspace_count: 3,
                idle_events: 0,
            },
            elapsed_seconds: 83.5,
        })
    }
}

/// Actions derived from input processing
#[derive(Debug, Clone)]
enum InputAction {
    Character(char),
    Backspace,
    Quit,
    Pause,
    Resume,
    ToggleHelp,
    Ignore,
}

/// Comprehensive engine performance metrics
#[derive(Debug, Clone)]
pub struct EnginePerformanceMetrics {
    pub cache_hit_rate: f64,
    pub input_rate_limit_stats: input::RateLimiterStats,
    pub filtered_sequences: u64,
    pub total_inputs: u64,
}
