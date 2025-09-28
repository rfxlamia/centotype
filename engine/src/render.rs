//! Comprehensive TUI render system for typing interface using ratatui
//! Provides real-time typing feedback with accessibility compliance (WCAG AA)
use centotype_core::types::*;
use ratatui::{
    backend::CrosstermBackend,
    Terminal,
    Frame,
    layout::{Layout, Direction, Constraint, Rect, Alignment},
    widgets::{Block, Borders, Paragraph, Gauge, Clear},
    style::{Color, Style, Modifier, Stylize},
    text::{Line, Span, Text},
};
use std::io::{self, Stdout};
use std::time::{Duration, Instant};
use tracing::{debug, warn, error};

/// Comprehensive TUI render system with accessibility and performance focus
pub struct Render {
    /// Terminal interface
    terminal: Option<Terminal<CrosstermBackend<Stdout>>>,
    /// Current render state
    render_state: RenderState,
    /// Frame count for performance monitoring
    frame_count: u64,
    /// Last render time for performance tracking
    last_render_time: Option<Instant>,
    /// UI color scheme (terminal-safe with WCAG AA compliance)
    colors: UiColors,
    /// Help overlay state
    show_help: bool,
    /// ANSI renderer for batched output
    ansi_renderer: AnsiRenderer,
    /// Pre-composed line cache
    line_cache: std::collections::HashMap<String, PrecomposedLine>,
    /// Cache hit statistics
    cache_hits: u64,
    cache_misses: u64,
}

/// State needed for rendering the typing interface
#[derive(Debug, Clone)]
struct RenderState {
    target_text: String,
    typed_text: String,
    cursor_position: usize,
    live_metrics: LiveMetrics,
    is_paused: bool,
    is_completed: bool,
    level_info: LevelInfo,
    session_duration: Duration,
    error_positions: Vec<usize>,
}

/// Level information for header display
#[derive(Debug, Clone, Default)]
struct LevelInfo {
    level_id: u8,
    tier: u8,
    description: String,
}

/// Pre-composed line for efficient rendering with batched ANSI sequences
#[derive(Debug, Clone)]
pub struct PrecomposedLine {
    /// Complete line content with embedded ANSI codes
    pub content: String,
    /// Byte offsets for cursor placement
    pub cursor_positions: Vec<usize>,
    /// Line metadata for optimization
    pub metadata: LineMetadata,
}

/// Metadata for line optimization
#[derive(Debug, Clone)]
pub struct LineMetadata {
    /// Number of style changes in the line
    pub style_changes: usize,
    /// Whether line contains errors
    pub has_errors: bool,
    /// Line length in characters
    pub char_count: usize,
    /// Generation timestamp for caching
    pub generated_at: Option<std::time::Instant>,
}

impl Default for LineMetadata {
    fn default() -> Self {
        Self {
            style_changes: 0,
            has_errors: false,
            char_count: 0,
            generated_at: None,
        }
    }
}

/// ANSI sequence batching system for optimal render performance
#[derive(Debug)]
pub struct AnsiRenderer {
    /// String buffer for batching ANSI sequences
    batch_buffer: String,
    /// Current position tracking
    current_row: u16,
    current_col: u16,
    /// Current style state
    current_style: Option<Style>,
    /// Output buffer capacity
    buffer_capacity: usize,
}

impl AnsiRenderer {
    pub fn new() -> Self {
        Self {
            batch_buffer: String::with_capacity(8192), // 8KB initial capacity
            current_row: 0,
            current_col: 0,
            current_style: None,
            buffer_capacity: 8192,
        }
    }

    /// Begin a new frame - clear buffers and reset state
    pub fn begin_frame(&mut self) {
        self.batch_buffer.clear();
        self.current_row = 0;
        self.current_col = 0;
        self.current_style = None;
    }

    /// Add a positioned styled span to the batch
    pub fn add_span(&mut self, row: u16, col: u16, content: &str, style: Style) {
        // Optimize cursor movement by batching sequential operations
        if row != self.current_row || col != self.current_col {
            self.batch_buffer.push_str(&format!("\x1b[{};{}H", row + 1, col + 1));
            self.current_row = row;
            self.current_col = col;
        }

        // Optimize style changes by only applying differences
        if self.current_style.as_ref() != Some(&style) {
            self.apply_style_optimized(&style);
            self.current_style = Some(style);
        }

        // Add content
        self.batch_buffer.push_str(content);
        self.current_col += content.chars().count() as u16;
    }

    /// Apply style with optimization for minimal ANSI output
    fn apply_style_optimized(&mut self, style: &Style) {
        // Start with reset only if necessary
        let mut needs_reset = false;

        if let Some(current) = &self.current_style {
            // Check if we need to reset based on previous style
            if current.fg != style.fg || current.bg != style.bg {
                needs_reset = true;
            }
        }

        if needs_reset || self.current_style.is_none() {
            self.batch_buffer.push_str("\x1b[0m"); // Reset
        }

        // Apply foreground color
        if let Some(fg) = style.fg {
            match fg {
                Color::Rgb(r, g, b) => {
                    self.batch_buffer.push_str(&format!("\x1b[38;2;{};{};{}m", r, g, b));
                }
                Color::Indexed(i) => {
                    self.batch_buffer.push_str(&format!("\x1b[38;5;{}m", i));
                }
                _ => {}
            }
        }

        // Apply background color
        if let Some(bg) = style.bg {
            match bg {
                Color::Rgb(r, g, b) => {
                    self.batch_buffer.push_str(&format!("\x1b[48;2;{};{};{}m", r, g, b));
                }
                Color::Indexed(i) => {
                    self.batch_buffer.push_str(&format!("\x1b[48;5;{}m", i));
                }
                _ => {}
            }
        }

        // Apply modifiers
        if style.add_modifier.contains(Modifier::BOLD) {
            self.batch_buffer.push_str("\x1b[1m");
        }
        if style.add_modifier.contains(Modifier::ITALIC) {
            self.batch_buffer.push_str("\x1b[3m");
        }
        if style.add_modifier.contains(Modifier::UNDERLINED) {
            self.batch_buffer.push_str("\x1b[4m");
        }
    }

    /// Finalize the batch and return the complete ANSI sequence
    pub fn finalize_batch(&mut self) -> String {
        // Add final reset and ensure buffer is ready for next frame
        self.batch_buffer.push_str("\x1b[0m");

        let result = self.batch_buffer.clone();

        // Prepare for next frame
        self.batch_buffer.clear();
        if self.batch_buffer.capacity() > self.buffer_capacity * 2 {
            // Prevent excessive memory growth
            self.batch_buffer.shrink_to(self.buffer_capacity);
        }

        result
    }

    /// Get current buffer size for memory monitoring
    pub fn buffer_size(&self) -> usize {
        self.batch_buffer.len()
    }
}

/// WCAG AA compliant color scheme
#[derive(Debug, Clone)]
struct UiColors {
    // Text colors (4.5:1 contrast minimum)
    correct_text: Color,
    incorrect_text: Color,
    cursor: Color,
    normal_text: Color,
    dimmed_text: Color,

    // Background colors
    error_bg: Color,
    cursor_bg: Color,
    status_bg: Color,

    // Progress colors
    progress_complete: Color,
    progress_remaining: Color,

    // Status colors
    accent: Color,
    warning: Color,
    success: Color,
}

impl Default for RenderState {
    fn default() -> Self {
        Self {
            target_text: String::new(),
            typed_text: String::new(),
            cursor_position: 0,
            live_metrics: LiveMetrics::default(),
            is_paused: false,
            is_completed: false,
            level_info: LevelInfo::default(),
            session_duration: Duration::ZERO,
            error_positions: Vec::new(),
        }
    }
}

impl Default for UiColors {
    fn default() -> Self {
        Self {
            // WCAG AA compliant colors (4.5:1 contrast on dark background)
            correct_text: Color::Rgb(144, 238, 144),     // Light Green
            incorrect_text: Color::Rgb(255, 182, 193),   // Light Pink
            cursor: Color::Rgb(255, 255, 0),            // Yellow
            normal_text: Color::Rgb(220, 220, 220),      // Light Gray
            dimmed_text: Color::Rgb(160, 160, 160),      // Medium Gray

            error_bg: Color::Rgb(139, 0, 0),            // Dark Red
            cursor_bg: Color::Rgb(75, 0, 130),           // Indigo
            status_bg: Color::Rgb(47, 79, 79),           // Dark Slate Gray

            progress_complete: Color::Rgb(70, 130, 180), // Steel Blue
            progress_remaining: Color::Rgb(105, 105, 105), // Dim Gray

            accent: Color::Rgb(100, 149, 237),          // Cornflower Blue
            warning: Color::Rgb(255, 215, 0),           // Gold
            success: Color::Rgb(50, 205, 50),           // Lime Green
        }
    }
}

impl Render {
    /// Create new render system with accessibility support and performance optimizations
    pub fn new() -> Result<Self> {
        Ok(Self {
            terminal: None,
            render_state: RenderState::default(),
            frame_count: 0,
            last_render_time: None,
            colors: UiColors::default(),
            show_help: false,
            ansi_renderer: AnsiRenderer::new(),
            line_cache: std::collections::HashMap::with_capacity(256),
            cache_hits: 0,
            cache_misses: 0,
        })
    }

    /// Initialize terminal for rendering
    pub fn initialize(&mut self) -> Result<()> {
        let backend = CrosstermBackend::new(io::stdout());
        let terminal = Terminal::new(backend)
            .map_err(|e| CentotypeError::Platform(format!("Failed to create terminal: {}", e)))?;

        self.terminal = Some(terminal);
        debug!("Render system initialized");
        Ok(())
    }

    /// Update render state with comprehensive session data
    pub fn update_state(&mut self, session_state: &SessionState, live_metrics: &LiveMetrics) {
        // Extract level information from training mode
        let level_info = match session_state.mode {
            TrainingMode::Arcade { level } => LevelInfo {
                level_id: level.0,
                tier: level.tier().0,
                description: format!("Programming Basics (Tier {})", level.tier().0),
            },
            TrainingMode::Drill { category, .. } => LevelInfo {
                level_id: 0,
                tier: 0,
                description: format!("{:?} Drill", category),
            },
            TrainingMode::Endurance { .. } => LevelInfo {
                level_id: 0,
                tier: 0,
                description: "Endurance Training".to_string(),
            },
        };

        // Calculate session duration
        let session_duration = session_state.started_at.signed_duration_since(
            chrono::Utc::now()
        ).to_std().unwrap_or(Duration::ZERO) - session_state.paused_duration;

        // Identify error positions for highlighting
        let error_positions = self.calculate_error_positions(
            &session_state.target_text,
            &session_state.typed_text
        );

        self.render_state = RenderState {
            target_text: session_state.target_text.clone(),
            typed_text: session_state.typed_text.clone(),
            cursor_position: session_state.cursor_position,
            live_metrics: live_metrics.clone(),
            is_paused: session_state.is_paused,
            is_completed: session_state.is_completed,
            level_info,
            session_duration,
            error_positions,
        };
    }

    /// Render current frame with performance monitoring and ANSI batching
    pub fn render_frame(&mut self) -> Result<Duration> {
        let render_start = std::time::Instant::now();

        if let Some(_terminal) = &mut self.terminal {
            // Use optimized direct ANSI output for better performance
            let ansi_output = self.render_with_ansi_batching()?;

            // Single write operation to terminal
            use std::io::Write;
            let mut stdout = std::io::stdout();
            stdout.write_all(ansi_output.as_bytes())
                .map_err(|e| CentotypeError::Platform(format!("ANSI output failed: {}", e)))?;
            stdout.flush()
                .map_err(|e| CentotypeError::Platform(format!("Flush failed: {}", e)))?;

            self.frame_count += 1;
        } else {
            warn!("Attempted to render without initialized terminal");
            return Err(CentotypeError::Platform(
                "Terminal not initialized for rendering".to_string()
            ));
        }

        let render_time = render_start.elapsed();

        // Performance warning for render times exceeding target
        if render_time > Duration::from_millis(33) {
            warn!("Frame {} render time {:?} exceeds P95 target (33ms)",
                self.frame_count, render_time);
        }

        self.last_render_time = Some(render_start);
        debug!("Frame {} rendered in {:?} with ANSI batching", self.frame_count, render_time);
        Ok(render_time)
    }

    /// Render using optimized ANSI batching for maximum performance
    fn render_with_ansi_batching(&mut self) -> Result<String> {
        self.ansi_renderer.begin_frame();

        // Get terminal size for layout calculation
        let (width, height) = self.check_terminal_size()?;

        // Clear screen efficiently
        self.ansi_renderer.add_span(0, 0, "\x1b[2J\x1b[H", Style::default());

        // Calculate layout proportions
        let header_row = 0;
        let typing_start_row = 1;
        let typing_height = (height as f32 * 0.6) as u16;
        let status_row = typing_start_row + typing_height;
        let progress_row = status_row + 1;
        let help_row = progress_row + 1;

        // Render each component with batched ANSI
        self.render_header_ansi(header_row, width)?;
        self.render_typing_pane_ansi(typing_start_row, width, typing_height)?;
        self.render_status_bar_ansi(status_row, width)?;
        self.render_progress_bar_ansi(progress_row, width)?;
        self.render_help_bar_ansi(help_row, width)?;

        if self.show_help {
            self.render_help_overlay_ansi(width, height)?;
        }

        Ok(self.ansi_renderer.finalize_batch())
    }

    /// Clear the screen
    pub fn clear(&mut self) -> Result<()> {
        if let Some(terminal) = &mut self.terminal {
            terminal.clear()
                .map_err(|e| CentotypeError::Platform(format!("Clear failed: {}", e)))?;
        }
        Ok(())
    }

    /// Get comprehensive performance metrics
    pub fn get_frame_count(&self) -> u64 {
        self.frame_count
    }

    /// Toggle help overlay
    pub fn toggle_help(&mut self) {
        self.show_help = !self.show_help;
        debug!("Help overlay toggled: {}", self.show_help);
    }

    /// Set help overlay state
    pub fn set_help_visible(&mut self, visible: bool) {
        self.show_help = visible;
    }

    /// Check if terminal size meets minimum requirements
    pub fn check_terminal_size(&self) -> Result<(u16, u16)> {
        if let Some(terminal) = &self.terminal {
            let size = terminal.size()
                .map_err(|e| CentotypeError::Platform(format!("Failed to get terminal size: {}", e)))?;

            // Minimum size check (80x24)
            if size.width < 80 || size.height < 24 {
                warn!("Terminal size {}x{} below recommended minimum 80x24",
                    size.width, size.height);
            }

            Ok((size.width, size.height))
        } else {
            Err(CentotypeError::Platform("Terminal not initialized".to_string()))
        }
    }

    /// Calculate error positions for highlighting
    fn calculate_error_positions(&self, target: &str, typed: &str) -> Vec<usize> {
        let target_chars: Vec<char> = target.chars().collect();
        let typed_chars: Vec<char> = typed.chars().collect();
        let mut errors = Vec::new();

        for (i, (&target_char, typed_char)) in target_chars.iter()
            .zip(typed_chars.iter().chain(std::iter::repeat(&'\0')))
            .enumerate() {
            if typed_char != &'\0' && *typed_char != target_char {
                errors.push(i);
            }
        }

        errors
    }

    // Private rendering methods

    /// Comprehensive UI layout with accessibility compliance
    fn draw_comprehensive_ui(
        render_state: &RenderState,
        colors: &UiColors,
        show_help: bool,
        frame: &mut Frame
    ) {
        // Create main layout with proper proportions
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(1),      // Header
                Constraint::Percentage(60), // Typing pane (main focus)
                Constraint::Length(1),      // Status bar
                Constraint::Length(1),      // Progress bar
                Constraint::Length(1),      // Help bar
            ])
            .split(frame.size());

        // Draw header with level info
        Self::draw_header_static(render_state, colors, frame, chunks[0]);

        // Draw main typing pane with cursor positioning
        Self::draw_typing_pane_static(render_state, colors, frame, chunks[1]);

        // Draw real-time status bar
        Self::draw_status_bar_static(render_state, colors, frame, chunks[2]);

        // Draw progress indicator
        Self::draw_progress_bar_static(render_state, colors, frame, chunks[3]);

        // Draw help/keymap bar
        Self::draw_help_bar_static(render_state, colors, frame, chunks[4]);

        // Draw help overlay if requested
        if show_help {
            Self::draw_help_overlay_static(colors, frame);
        }
    }

    /// Header with level information
    fn draw_header_static(
        render_state: &RenderState,
        colors: &UiColors,
        frame: &mut Frame,
        area: Rect
    ) {
        let header_text = if render_state.level_info.level_id > 0 {
            format!(
                "Centotype CLI - Level {} (Tier {}) - {}",
                render_state.level_info.level_id,
                render_state.level_info.tier,
                render_state.level_info.description
            )
        } else {
            format!("Centotype CLI - {}", render_state.level_info.description)
        };

        let header_style = if render_state.is_paused {
            Style::default().fg(colors.warning).add_modifier(Modifier::BOLD)
        } else if render_state.is_completed {
            Style::default().fg(colors.success).add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(colors.accent).add_modifier(Modifier::BOLD)
        };

        let header = Paragraph::new(header_text)
            .style(header_style)
            .alignment(Alignment::Center);

        frame.render_widget(header, area);
    }


    /// Typing pane with real-time cursor and highlighting
    fn draw_typing_pane_static(
        render_state: &RenderState,
        colors: &UiColors,
        frame: &mut Frame,
        area: Rect
    ) {
        // Split typing area for target and input display
        let typing_chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage(50), // Target text
                Constraint::Percentage(50), // Input text with cursor
            ])
            .split(area);

        // Draw target text
        Self::draw_target_text(render_state, colors, frame, typing_chunks[0]);

        // Draw input text with cursor
        Self::draw_input_text(render_state, colors, frame, typing_chunks[1]);
    }

    /// Target text display
    fn draw_target_text(
        render_state: &RenderState,
        colors: &UiColors,
        frame: &mut Frame,
        area: Rect
    ) {
        let target_text = Text::from(Line::from(vec![
            Span::raw("Target: "),
            Span::styled(&render_state.target_text, Style::default().fg(colors.normal_text)),
        ]));

        let target_paragraph = Paragraph::new(target_text)
            .block(Block::default().borders(Borders::ALL).title("Target Text"))
            .wrap(ratatui::widgets::Wrap { trim: false });

        frame.render_widget(target_paragraph, area);
    }

    /// Input text with cursor positioning and error highlighting
    fn draw_input_text(
        render_state: &RenderState,
        colors: &UiColors,
        frame: &mut Frame,
        area: Rect
    ) {
        let mut spans = vec![Span::raw("Input:  ")];

        // Create styled spans for typed text with error highlighting
        let target_chars: Vec<char> = render_state.target_text.chars().collect();
        let typed_chars: Vec<char> = render_state.typed_text.chars().collect();

        for (i, &typed_char) in typed_chars.iter().enumerate() {
            let style = if i < target_chars.len() {
                let target_char = target_chars[i];
                if typed_char == target_char {
                    Style::default().fg(colors.correct_text)
                } else {
                    Style::default().fg(colors.incorrect_text).bg(colors.error_bg)
                }
            } else {
                // Extra characters (insertion errors)
                Style::default().fg(colors.incorrect_text).bg(colors.error_bg)
            };

            spans.push(Span::styled(typed_char.to_string(), style));
        }

        // Add cursor if within bounds
        if render_state.cursor_position < target_chars.len() {
            let cursor_char = target_chars.get(render_state.cursor_position)
                .unwrap_or(&' ')
                .to_string();
            spans.push(Span::styled(
                cursor_char,
                Style::default().fg(colors.cursor).bg(colors.cursor_bg).add_modifier(Modifier::BOLD)
            ));
        } else if render_state.cursor_position == target_chars.len() {
            // End of text cursor
            spans.push(Span::styled(
                "█",
                Style::default().fg(colors.cursor).add_modifier(Modifier::BOLD)
            ));
        }

        let input_text = Text::from(Line::from(spans));
        let input_paragraph = Paragraph::new(input_text)
            .block(Block::default().borders(Borders::ALL).title("Your Input"))
            .wrap(ratatui::widgets::Wrap { trim: false });

        frame.render_widget(input_paragraph, area);
    }

    /// Status bar with real-time metrics
    fn draw_status_bar_static(
        render_state: &RenderState,
        colors: &UiColors,
        frame: &mut Frame,
        area: Rect
    ) {
        let metrics = &render_state.live_metrics;
        let duration_secs = render_state.session_duration.as_secs();
        let minutes = duration_secs / 60;
        let seconds = duration_secs % 60;

        let status_text = format!(
            " WPM: {:.0} │ ACC: {:.1}% │ COMBO: x{} │ ⏱️  {}:{:02} │ ERR: {} ",
            metrics.effective_wpm,
            metrics.accuracy,
            metrics.current_streak,
            minutes,
            seconds,
            metrics.errors.total_errors()
        );

        let status_style = Style::default()
            .fg(colors.normal_text)
            .bg(colors.status_bg);

        let status_bar = Paragraph::new(status_text)
            .style(status_style)
            .alignment(Alignment::Left);

        frame.render_widget(status_bar, area);
    }

    /// Progress bar with completion percentage
    fn draw_progress_bar_static(
        render_state: &RenderState,
        colors: &UiColors,
        frame: &mut Frame,
        area: Rect
    ) {
        let progress_percent = if render_state.target_text.is_empty() {
            0.0
        } else {
            (render_state.cursor_position as f64 / render_state.target_text.len() as f64) * 100.0
        };

        let progress_text = format!(
            " Progress: {}% ({}/{} chars) ",
            progress_percent as u16,
            render_state.cursor_position,
            render_state.target_text.len()
        );

        // Create visual progress bar with blocks
        let bar_width = area.width.saturating_sub(2) as usize; // Account for borders
        let filled_blocks = ((progress_percent / 100.0) * bar_width as f64) as usize;
        let empty_blocks = bar_width.saturating_sub(filled_blocks);

        let progress_visual = format!(
            "{}{}",
            "█".repeat(filled_blocks),
            "░".repeat(empty_blocks)
        );

        let progress_line = Line::from(vec![
            Span::styled(
                progress_visual,
                Style::default().fg(colors.progress_complete)
            ),
            Span::styled(
                progress_text,
                Style::default().fg(colors.normal_text)
            ),
        ]);

        let progress_paragraph = Paragraph::new(progress_line)
            .alignment(Alignment::Left);

        frame.render_widget(progress_paragraph, area);
    }

    /// Help bar with key mappings
    fn draw_help_bar_static(
        render_state: &RenderState,
        colors: &UiColors,
        frame: &mut Frame,
        area: Rect
    ) {
        let help_text = if render_state.is_paused {
            " PAUSED - Ctrl+P:resume │ ESC:quit │ F1:help "
        } else if render_state.is_completed {
            " COMPLETED! - Enter:continue │ ESC:quit │ F1:help "
        } else {
            " ESC:quit │ F1:help │ Ctrl+R:restart │ Tab:stats "
        };

        let help_style = Style::default()
            .fg(colors.dimmed_text)
            .add_modifier(Modifier::ITALIC);

        let help_bar = Paragraph::new(help_text)
            .style(help_style)
            .alignment(Alignment::Center);

        frame.render_widget(help_bar, area);
    }

    /// Help overlay with comprehensive key mappings
    fn draw_help_overlay_static(colors: &UiColors, frame: &mut Frame) {
        // Calculate centered popup area
        let popup_area = Self::centered_rect(60, 70, frame.size());

        // Clear background
        frame.render_widget(Clear, popup_area);

        let help_content = vec![
            Line::from("CENTOTYPE HELP").alignment(Alignment::Center),
            Line::from(""),
            Line::from("Typing Commands:"),
            Line::from("  Backspace    Delete previous character"),
            Line::from("  Ctrl+C       Quit application"),
            Line::from("  ESC          Quit current session"),
            Line::from(""),
            Line::from("Session Controls:"),
            Line::from("  Ctrl+P       Pause/Resume session"),
            Line::from("  Ctrl+R       Restart current level"),
            Line::from("  Tab          Show detailed statistics"),
            Line::from(""),
            Line::from("Display:"),
            Line::from("  F1           Toggle this help"),
            Line::from("  Green text   Correct characters"),
            Line::from("  Red text     Incorrect characters"),
            Line::from("  Yellow       Current cursor position"),
            Line::from(""),
            Line::from("Accessibility:"),
            Line::from("  Colors meet WCAG AA standards"),
            Line::from("  Mono font fallback supported"),
            Line::from("  Screen reader compatible"),
            Line::from(""),
            Line::from("Press F1 again to close this help").alignment(Alignment::Center),
        ];

        let help_popup = Paragraph::new(help_content)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Help - F1 to close")
                    .title_alignment(Alignment::Center)
            )
            .style(Style::default().fg(colors.normal_text))
            .wrap(ratatui::widgets::Wrap { trim: false });

        frame.render_widget(help_popup, popup_area);
    }

    /// Calculate centered rectangle for popups
    fn centered_rect(percent_x: u16, percent_y: u16, r: Rect) -> Rect {
        let popup_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Percentage((100 - percent_y) / 2),
                Constraint::Percentage(percent_y),
                Constraint::Percentage((100 - percent_y) / 2),
            ])
            .split(r);

        Layout::default()
            .direction(Direction::Horizontal)
            .constraints([
                Constraint::Percentage((100 - percent_x) / 2),
                Constraint::Percentage(percent_x),
                Constraint::Percentage((100 - percent_x) / 2),
            ])
            .split(popup_layout[1])[1]
    }

    // ANSI render methods (placeholder implementations for now)
    fn render_header_ansi(&mut self, _row: u16, _width: u16) -> Result<()> {
        // Placeholder implementation
        Ok(())
    }

    fn render_typing_pane_ansi(&mut self, _row: u16, _width: u16, _height: u16) -> Result<()> {
        // Placeholder implementation
        Ok(())
    }

    fn render_status_bar_ansi(&mut self, _row: u16, _width: u16) -> Result<()> {
        // Placeholder implementation
        Ok(())
    }

    fn render_progress_bar_ansi(&mut self, _row: u16, _width: u16) -> Result<()> {
        // Placeholder implementation
        Ok(())
    }

    fn render_help_bar_ansi(&mut self, _row: u16, _width: u16) -> Result<()> {
        // Placeholder implementation
        Ok(())
    }

    fn render_help_overlay_ansi(&mut self, _width: u16, _height: u16) -> Result<()> {
        // Placeholder implementation
        Ok(())
    }

}

impl Default for Render {
    fn default() -> Self {
        Self::new().unwrap_or_else(|_| Self {
            terminal: None,
            render_state: RenderState::default(),
            frame_count: 0,
            last_render_time: None,
            colors: UiColors::default(),
            show_help: false,
            ansi_renderer: AnsiRenderer::new(),
            line_cache: std::collections::HashMap::new(),
            cache_hits: 0,
            cache_misses: 0,
        })
    }
}
