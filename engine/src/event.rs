//! Event loop with high-performance input processing and <25ms P99 latency
use centotype_core::{types::*, CentotypeCore};
use centotype_platform::PlatformManager;
use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEvent, KeyModifiers};
use parking_lot::{Mutex, RwLock};
use std::collections::VecDeque;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::mpsc;
use tokio::time::timeout;
use tracing::{debug, error, info, warn};

/// High-performance event loop with latency tracking
pub struct Event {
    core: Arc<CentotypeCore>,
    platform: Arc<PlatformManager>,
    performance_tracker: Arc<Mutex<EventPerformanceTracker>>,
    is_running: Arc<RwLock<bool>>,
}

impl Event {
    pub fn new(core: Arc<CentotypeCore>, platform: Arc<PlatformManager>) -> Self {
        Self {
            core,
            platform,
            performance_tracker: Arc::new(Mutex::new(EventPerformanceTracker::new())),
            is_running: Arc::new(RwLock::new(false)),
        }
    }

    /// Start the main event loop with performance monitoring
    pub async fn run(&mut self, session_id: uuid::Uuid) -> Result<()> {
        let start_time = Instant::now();
        info!("Starting event loop for session {}", session_id);

        // Set running state
        *self.is_running.write() = true;

        // Create channels for event processing
        let (event_tx, mut event_rx) = mpsc::unbounded_channel::<EngineEvent>();
        let (render_tx, render_rx) = mpsc::unbounded_channel::<RenderCommand>();

        // Start input processing task
        let input_task = self.start_input_processing(event_tx.clone()).await?;

        // Start render loop task
        let render_task = self.start_render_loop(render_rx).await?;

        // Main event processing loop
        let mut last_live_update = Instant::now();
        let live_update_interval = Duration::from_millis(100); // Update live metrics every 100ms

        loop {
            // Check if we should stop
            if !*self.is_running.read() {
                break;
            }

            // Process events with timeout to ensure responsiveness
            match timeout(Duration::from_millis(50), event_rx.recv()).await {
                Ok(Some(engine_event)) => {
                    let process_start = Instant::now();

                    match self.process_event(engine_event).await {
                        Ok(should_continue) => {
                            if !should_continue {
                                info!("Event loop terminating due to quit signal");
                                break;
                            }
                        }
                        Err(e) => {
                            error!("Error processing event: {}", e);
                            // Continue processing other events
                        }
                    }

                    // Track processing latency
                    let processing_time = process_start.elapsed();
                    self.performance_tracker.lock().record_event_processing(processing_time);

                    // Request render update
                    if render_tx.send(RenderCommand::Update).is_err() {
                        warn!("Render channel closed");
                        break;
                    }
                }
                Ok(None) => {
                    // Channel closed
                    debug!("Event channel closed");
                    break;
                }
                Err(_) => {
                    // Timeout - continue loop for responsiveness
                }
            }

            // Periodic live metrics update
            if last_live_update.elapsed() >= live_update_interval {
                if let Err(e) = self.update_live_metrics().await {
                    warn!("Failed to update live metrics: {}", e);
                }
                last_live_update = Instant::now();
            }
        }

        // Cleanup
        *self.is_running.write() = false;

        // Wait for tasks to complete
        if let Err(e) = input_task.await {
            error!("Input task error: {:?}", e);
        }
        if let Err(e) = render_task.await {
            error!("Render task error: {:?}", e);
        }

        let total_time = start_time.elapsed();
        info!("Event loop completed in {:?}", total_time);

        Ok(())
    }

    /// Emergency stop the event loop
    pub fn stop(&self) {
        *self.is_running.write() = false;
        info!("Event loop stop requested");
    }

    /// Get performance metrics
    pub fn get_performance_metrics(&self) -> EventMetrics {
        self.performance_tracker.lock().get_metrics()
    }

    // Private methods

    async fn start_input_processing(
        &self,
        event_tx: mpsc::UnboundedSender<EngineEvent>,
    ) -> Result<tokio::task::JoinHandle<()>> {
        let is_running = Arc::clone(&self.is_running);
        let performance_tracker = Arc::clone(&self.performance_tracker);

        let task = tokio::spawn(async move {
            // Use crossterm event polling instead of EventStream for compatibility
            while *is_running.read() {
                let input_start = Instant::now();

                match timeout(Duration::from_millis(10), async move {
                    match crossterm::event::poll(Duration::from_millis(1)) {
                        Ok(true) => {
                            match crossterm::event::read() {
                                Ok(event) => Ok(Some(event)),
                                Err(e) => Err(CentotypeError::Input(format!("Read error: {}", e))),
                            }
                        }
                        Ok(false) => Ok(None),
                        Err(e) => Err(CentotypeError::Input(format!("Poll error: {}", e))),
                    }
                }).await {
                    Ok(Ok(Some(crossterm_event))) => {
                        let input_latency = input_start.elapsed();
                        performance_tracker.lock().record_input_latency(input_latency);

                        // Convert crossterm event to engine event
                        if let Some(engine_event) = Self::convert_crossterm_event(crossterm_event) {
                            if event_tx.send(engine_event).is_err() {
                                debug!("Event channel closed, stopping input processing");
                                break;
                            }
                        }
                    }
                    Ok(Ok(None)) => {
                        // No input available, continue
                    }
                    Ok(Err(e)) => {
                        error!("Input error: {}", e);
                        tokio::time::sleep(Duration::from_millis(1)).await;
                    }
                    Err(_) => {
                        // Timeout - continue loop
                    }
                }
            }

            debug!("Input processing task completed");
        });

        Ok(task)
    }

    async fn start_render_loop(
        &self,
        mut render_rx: mpsc::UnboundedReceiver<RenderCommand>,
    ) -> Result<tokio::task::JoinHandle<()>> {
        let is_running = Arc::clone(&self.is_running);
        let performance_tracker = Arc::clone(&self.performance_tracker);

        let task = tokio::spawn(async move {
            while *is_running.read() {
                match timeout(Duration::from_millis(33), render_rx.recv()).await {
                    Ok(Some(command)) => {
                        let render_start = Instant::now();

                        match command {
                            RenderCommand::Update => {
                                // Perform render update
                                // This would integrate with the actual renderer
                            }
                            RenderCommand::Clear => {
                                // Clear screen
                            }
                            RenderCommand::Stop => {
                                break;
                            }
                        }

                        let render_time = render_start.elapsed();
                        performance_tracker.lock().record_render_time(render_time);
                    }
                    Ok(None) => {
                        // Channel closed
                        break;
                    }
                    Err(_) => {
                        // Timeout - maintain 30 FPS by continuing
                    }
                }
            }

            debug!("Render loop task completed");
        });

        Ok(task)
    }

    async fn process_event(&self, event: EngineEvent) -> Result<bool> {
        match event {
            EngineEvent::Key(key_event) => {
                self.handle_key_event(key_event).await?;
                Ok(true)
            }
            EngineEvent::Resize { width, height } => {
                self.handle_resize(width, height).await?;
                Ok(true)
            }
            EngineEvent::Quit => {
                info!("Quit event received");
                Ok(false)
            }
            EngineEvent::Pause => {
                self.handle_pause().await?;
                Ok(true)
            }
            EngineEvent::Resume => {
                self.handle_resume().await?;
                Ok(true)
            }
        }
    }

    async fn handle_key_event(&self, key_event: KeyEvent) -> Result<()> {
        // Handle special key combinations first
        if key_event.modifiers.contains(KeyModifiers::CONTROL) {
            match key_event.code {
                KeyCode::Char('c') => {
                    // Ctrl+C - emergency stop
                    self.stop();
                    return Ok(());
                }
                KeyCode::Char('p') => {
                    // Ctrl+P - pause/resume
                    return self.handle_pause().await;
                }
                _ => {}
            }
        }

        // Handle escape key
        if key_event.code == KeyCode::Esc {
            self.stop();
            return Ok(());
        }

        // Process typing input
        match key_event.code {
            KeyCode::Char(c) => {
                let keystroke = Keystroke {
                    timestamp: chrono::Utc::now(),
                    char_typed: Some(c),
                    is_correction: false,
                    cursor_pos: 0, // This would be updated from session state
                };

                self.core.process_keystroke(Some(c), false)?;

                // Update session state
                let state_update = StateUpdate::AddKeystroke(keystroke);
                if let Err(e) = self.update_session_state(state_update).await {
                    warn!("Failed to update session state: {}", e);
                }
            }
            KeyCode::Backspace => {
                let keystroke = Keystroke {
                    timestamp: chrono::Utc::now(),
                    char_typed: None,
                    is_correction: true,
                    cursor_pos: 0,
                };

                self.core.process_keystroke(None, true)?;

                let state_update = StateUpdate::AddKeystroke(keystroke);
                if let Err(e) = self.update_session_state(state_update).await {
                    warn!("Failed to update session state: {}", e);
                }
            }
            KeyCode::Enter => {
                // Complete session or handle enter
                self.handle_enter().await?;
            }
            _ => {
                // Ignore other keys
            }
        }

        Ok(())
    }

    async fn handle_resize(&self, _width: u16, _height: u16) -> Result<()> {
        // Handle terminal resize
        debug!("Terminal resized to {}x{}", _width, _height);
        Ok(())
    }

    async fn handle_pause(&self) -> Result<()> {
        let state_update = StateUpdate::SetPaused(true);
        self.update_session_state(state_update).await?;
        debug!("Session paused");
        Ok(())
    }

    async fn handle_resume(&self) -> Result<()> {
        let state_update = StateUpdate::SetPaused(false);
        self.update_session_state(state_update).await?;
        debug!("Session resumed");
        Ok(())
    }

    async fn handle_enter(&self) -> Result<()> {
        // Check if session is complete
        // This is a simplified implementation
        let state_update = StateUpdate::Complete;
        self.update_session_state(state_update).await?;
        debug!("Session completed");
        Ok(())
    }

    async fn update_session_state(&self, update: StateUpdate) -> Result<()> {
        // This would need to be implemented in the core
        // For now, it's a placeholder
        debug!("Session state update: {:?}", update);
        Ok(())
    }

    async fn update_live_metrics(&self) -> Result<()> {
        // Update live performance metrics
        // This would calculate and broadcast current WPM, accuracy, etc.
        debug!("Updating live metrics");
        Ok(())
    }

    fn convert_crossterm_event(event: CrosstermEvent) -> Option<EngineEvent> {
        match event {
            CrosstermEvent::Key(key) => Some(EngineEvent::Key(key)),
            CrosstermEvent::Resize(width, height) => Some(EngineEvent::Resize { width, height }),
            _ => None,
        }
    }
}

/// Engine events processed by the event loop
#[derive(Debug, Clone)]
pub enum EngineEvent {
    Key(KeyEvent),
    Resize { width: u16, height: u16 },
    Quit,
    Pause,
    Resume,
}

/// Render commands for the render loop
#[derive(Debug, Clone)]
enum RenderCommand {
    Update,
    Clear,
    Stop,
}

/// Performance tracking for event processing
#[derive(Debug)]
struct EventPerformanceTracker {
    input_latencies: VecDeque<Duration>,
    event_processing_times: VecDeque<Duration>,
    render_times: VecDeque<Duration>,
    total_events: u64,
    total_renders: u64,
}

impl EventPerformanceTracker {
    fn new() -> Self {
        Self {
            input_latencies: VecDeque::new(),
            event_processing_times: VecDeque::new(),
            render_times: VecDeque::new(),
            total_events: 0,
            total_renders: 0,
        }
    }

    fn record_input_latency(&mut self, latency: Duration) {
        self.input_latencies.push_back(latency);
        if self.input_latencies.len() > 1000 {
            self.input_latencies.pop_front();
        }
    }

    fn record_event_processing(&mut self, duration: Duration) {
        self.event_processing_times.push_back(duration);
        self.total_events += 1;
        if self.event_processing_times.len() > 1000 {
            self.event_processing_times.pop_front();
        }
    }

    fn record_render_time(&mut self, duration: Duration) {
        self.render_times.push_back(duration);
        self.total_renders += 1;
        if self.render_times.len() > 1000 {
            self.render_times.pop_front();
        }
    }

    fn get_metrics(&self) -> EventMetrics {
        EventMetrics {
            input_latency_p50: self.calculate_percentile(&self.input_latencies, 0.5),
            input_latency_p95: self.calculate_percentile(&self.input_latencies, 0.95),
            input_latency_p99: self.calculate_percentile(&self.input_latencies, 0.99),
            event_processing_p50: self.calculate_percentile(&self.event_processing_times, 0.5),
            event_processing_p95: self.calculate_percentile(&self.event_processing_times, 0.95),
            render_time_p50: self.calculate_percentile(&self.render_times, 0.5),
            render_time_p95: self.calculate_percentile(&self.render_times, 0.95),
            total_events: self.total_events,
            total_renders: self.total_renders,
        }
    }

    fn calculate_percentile(&self, data: &VecDeque<Duration>, percentile: f64) -> Duration {
        if data.is_empty() {
            return Duration::default();
        }

        let mut sorted: Vec<Duration> = data.iter().cloned().collect();
        sorted.sort();

        let index = ((data.len() as f64 - 1.0) * percentile) as usize;
        sorted[index.min(sorted.len() - 1)]
    }
}

/// Event loop performance metrics
#[derive(Debug, Clone)]
pub struct EventMetrics {
    pub input_latency_p50: Duration,
    pub input_latency_p95: Duration,
    pub input_latency_p99: Duration,
    pub event_processing_p50: Duration,
    pub event_processing_p95: Duration,
    pub render_time_p50: Duration,
    pub render_time_p95: Duration,
    pub total_events: u64,
    pub total_renders: u64,
}

impl EventMetrics {
    /// Check if performance targets are met
    pub fn meets_targets(&self) -> bool {
        self.input_latency_p99 <= Duration::from_millis(25)
            && self.render_time_p95 <= Duration::from_millis(33)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_event_performance_tracker() {
        let mut tracker = EventPerformanceTracker::new();

        // Record some test data
        tracker.record_input_latency(Duration::from_millis(10));
        tracker.record_input_latency(Duration::from_millis(20));
        tracker.record_input_latency(Duration::from_millis(30));

        let metrics = tracker.get_metrics();
        assert!(metrics.input_latency_p50 > Duration::default());
        assert_eq!(metrics.total_events, 0); // No events processed yet
    }

    #[test]
    fn test_event_metrics_targets() {
        let good_metrics = EventMetrics {
            input_latency_p50: Duration::from_millis(5),
            input_latency_p95: Duration::from_millis(15),
            input_latency_p99: Duration::from_millis(20),
            event_processing_p50: Duration::from_millis(2),
            event_processing_p95: Duration::from_millis(5),
            render_time_p50: Duration::from_millis(16),
            render_time_p95: Duration::from_millis(30),
            total_events: 100,
            total_renders: 1000,
        };

        assert!(good_metrics.meets_targets());

        let bad_metrics = EventMetrics {
            input_latency_p50: Duration::from_millis(20),
            input_latency_p95: Duration::from_millis(40),
            input_latency_p99: Duration::from_millis(50), // Exceeds 25ms target
            event_processing_p50: Duration::from_millis(10),
            event_processing_p95: Duration::from_millis(20),
            render_time_p50: Duration::from_millis(20),
            render_time_p95: Duration::from_millis(40), // Exceeds 33ms target
            total_events: 100,
            total_renders: 1000,
        };

        assert!(!bad_metrics.meets_targets());
    }
}
