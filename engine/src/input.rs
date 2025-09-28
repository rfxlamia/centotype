//! High-performance input handling with security validation and event batching
use centotype_core::types::*;
use crossterm::event::{Event as CrosstermEvent, KeyCode, KeyEvent, KeyModifiers};
use regex::Regex;
use std::collections::VecDeque;
use std::time::{Duration, Instant};
use tracing::{debug, warn};
use unicode_segmentation::UnicodeSegmentation;

/// Enhanced input statistics with performance metrics
#[derive(Debug, Clone)]
pub struct EnhancedInputStatistics {
    // Legacy statistics
    pub total_processed: u64,
    pub filtered_sequences: u64,
    pub rate_limited_inputs: u64,
    pub rate_limiter_stats: RateLimiterStats,

    // Performance metrics
    pub performance_stats: InputPerformanceStats,
    pub batch_stats: BatchStats,

    // Overall assessment
    pub performance_grade: char,
}

/// Input performance statistics
#[derive(Debug, Clone)]
pub struct InputPerformanceStats {
    pub avg_processing_time: Duration,
    pub p95_processing_time: Duration,
    pub p99_processing_time: Duration,
    pub total_events: u64,
}

/// High-performance input processor with batching and security validation
pub struct Input {
    /// Character allowlist configuration
    allowed_characters: AllowedCharacters,
    /// Escape sequence filter
    escape_filter: EscapeSequenceFilter,
    /// Rate limiter
    rate_limiter: RateLimiter,
    /// Security policy
    security_policy: SecurityPolicy,
    /// Event batch processor for reducing system calls
    event_batcher: EventBatcher,
    /// Performance monitoring
    performance_monitor: InputPerformanceMonitor,
}

/// Event batching system for reducing system call overhead
#[derive(Debug)]
pub struct EventBatcher {
    /// Pending events to be processed
    event_queue: VecDeque<CrosstermEvent>,
    /// Maximum events to batch per cycle
    max_batch_size: usize,
    /// Timeout for batching (to ensure responsiveness)
    batch_timeout: Duration,
    /// Last batch processing time
    last_batch_time: Instant,
    /// Batch statistics
    batch_stats: BatchStats,
}

/// Batch processing statistics
#[derive(Debug, Default, Clone)]
pub struct BatchStats {
    pub total_batches: u64,
    pub total_events: u64,
    pub avg_batch_size: f64,
    pub max_batch_size: usize,
    pub system_calls_saved: u64,
    pub batch_efficiency: f64,
}

/// Input performance monitoring for hot-path optimization
#[derive(Debug)]
pub struct InputPerformanceMonitor {
    /// Event processing times
    processing_times: VecDeque<Duration>,
    /// Security validation times
    validation_times: VecDeque<Duration>,
    /// Batch processing times
    batch_times: VecDeque<Duration>,
    /// Total processed events
    total_events: u64,
    /// Performance targets
    targets: InputPerformanceTargets,
}

/// Performance targets for input processing
#[derive(Debug, Clone)]
pub struct InputPerformanceTargets {
    pub max_processing_time: Duration,
    pub max_validation_time: Duration,
    pub max_batch_time: Duration,
    pub target_batch_efficiency: f64,
}

impl Default for InputPerformanceTargets {
    fn default() -> Self {
        Self {
            max_processing_time: Duration::from_millis(5),
            max_validation_time: Duration::from_millis(1),
            max_batch_time: Duration::from_millis(2),
            target_batch_efficiency: 0.8,
        }
    }
}

impl EventBatcher {
    pub fn new() -> Self {
        Self {
            event_queue: VecDeque::with_capacity(32),
            max_batch_size: 8, // Process up to 8 events per batch
            batch_timeout: Duration::from_millis(5), // 5ms timeout for responsiveness
            last_batch_time: Instant::now(),
            batch_stats: BatchStats::default(),
        }
    }

    /// Poll events in batches for optimal performance
    pub fn poll_events_batch(&mut self, timeout: Duration) -> Result<Vec<CrosstermEvent>> {
        let batch_start = Instant::now();
        let mut events = Vec::with_capacity(self.max_batch_size);
        let deadline = Instant::now() + timeout;

        // Collect events from the queue first
        while !self.event_queue.is_empty() && events.len() < self.max_batch_size {
            if let Some(event) = self.event_queue.pop_front() {
                events.push(event);
            }
        }

        // Poll for new events if we haven't reached batch size
        while events.len() < self.max_batch_size && Instant::now() < deadline {
            match crossterm::event::poll(Duration::from_millis(1)) {
                Ok(true) => {
                    match crossterm::event::read() {
                        Ok(event) => {
                            events.push(event);
                        }
                        Err(e) => {
                            warn!("Failed to read event: {}", e);
                            break;
                        }
                    }
                }
                Ok(false) => {
                    // No events available, check timeout
                    if events.is_empty() && self.last_batch_time.elapsed() < self.batch_timeout {
                        continue;
                    } else {
                        break;
                    }
                }
                Err(e) => {
                    warn!("Event polling error: {}", e);
                    break;
                }
            }
        }

        // Update statistics
        if !events.is_empty() {
            self.update_batch_stats(&events, batch_start.elapsed());
            self.last_batch_time = Instant::now();
        }

        Ok(events)
    }

    /// Update batch processing statistics
    fn update_batch_stats(&mut self, events: &[CrosstermEvent], processing_time: Duration) {
        self.batch_stats.total_batches += 1;
        self.batch_stats.total_events += events.len() as u64;
        self.batch_stats.max_batch_size = self.batch_stats.max_batch_size.max(events.len());

        // Calculate average batch size
        self.batch_stats.avg_batch_size = self.batch_stats.total_events as f64 / self.batch_stats.total_batches as f64;

        // Estimate system calls saved (assuming we would have made one call per event)
        if events.len() > 1 {
            self.batch_stats.system_calls_saved += (events.len() - 1) as u64;
        }

        // Calculate batch efficiency (events per millisecond)
        let processing_ms = processing_time.as_secs_f64() * 1000.0;
        if processing_ms > 0.0 {
            self.batch_stats.batch_efficiency = events.len() as f64 / processing_ms;
        }

        debug!("Processed batch of {} events in {:?}", events.len(), processing_time);
    }

    /// Get batch statistics
    pub fn get_stats(&self) -> &BatchStats {
        &self.batch_stats
    }
}

impl InputPerformanceMonitor {
    pub fn new() -> Self {
        Self {
            processing_times: VecDeque::with_capacity(1000),
            validation_times: VecDeque::with_capacity(1000),
            batch_times: VecDeque::with_capacity(1000),
            total_events: 0,
            targets: InputPerformanceTargets::default(),
        }
    }

    /// Record event processing time
    pub fn record_processing_time(&mut self, duration: Duration) {
        self.processing_times.push_back(duration);
        if self.processing_times.len() > 1000 {
            self.processing_times.pop_front();
        }

        if duration > self.targets.max_processing_time {
            warn!("Input processing time {:?} exceeds target {:?}",
                  duration, self.targets.max_processing_time);
        }

        self.total_events += 1;
    }

    /// Get performance statistics
    pub fn get_performance_stats(&self) -> InputPerformanceStats {
        InputPerformanceStats {
            avg_processing_time: self.calculate_average(&self.processing_times),
            p95_processing_time: self.calculate_percentile(&self.processing_times, 0.95),
            p99_processing_time: self.calculate_percentile(&self.processing_times, 0.99),
            total_events: self.total_events,
        }
    }

    /// Calculate average duration
    fn calculate_average(&self, durations: &VecDeque<Duration>) -> Duration {
        if durations.is_empty() {
            return Duration::ZERO;
        }
        durations.iter().sum::<Duration>() / durations.len() as u32
    }

    /// Calculate percentile
    fn calculate_percentile(&self, durations: &VecDeque<Duration>, percentile: f64) -> Duration {
        if durations.is_empty() {
            return Duration::ZERO;
        }

        let mut sorted: Vec<Duration> = durations.iter().cloned().collect();
        sorted.sort();

        let index = ((sorted.len() as f64 - 1.0) * percentile) as usize;
        sorted[index.min(sorted.len() - 1)]
    }
}

impl Input {
    /// Create new input processor with batching and performance monitoring
    pub fn new() -> Self {
        Self {
            allowed_characters: AllowedCharacters::new(),
            escape_filter: EscapeSequenceFilter::new(),
            rate_limiter: RateLimiter::new(),
            security_policy: SecurityPolicy::default(),
            event_batcher: EventBatcher::new(),
            performance_monitor: InputPerformanceMonitor::new(),
        }
    }

    /// Process key event with enhanced performance monitoring
    pub fn process_key_event(&mut self, key_event: KeyEvent) -> Result<ProcessedInput> {
        let processing_start = Instant::now();

        // Rate limiting check
        if !self.rate_limiter.allow_input() {
            warn!("Input rate limit exceeded");
            return Err(CentotypeError::Input("Rate limit exceeded".to_string()));
        }

        // Process the key event
        let processed = self.sanitize_and_validate(key_event)?;

        // Record processing time for monitoring
        let processing_time = processing_start.elapsed();
        self.performance_monitor.record_processing_time(processing_time);

        debug!(
            duration_ms = %processing_time.as_millis(),
            input_type = ?processed.input_type,
            "Processed input"
        );

        Ok(processed)
    }

    /// Process batch of events for optimal performance
    pub fn process_event_batch(&mut self, timeout: Duration) -> Result<Vec<ProcessedInput>> {
        let batch_start = Instant::now();

        // Get batched events
        let events = self.event_batcher.poll_events_batch(timeout)?;
        let mut processed_events = Vec::with_capacity(events.len());

        // Process each event in the batch
        for event in events {
            match event {
                CrosstermEvent::Key(key_event) => {
                    let processed = self.process_key_event(key_event)?;
                    processed_events.push(processed);
                }
                _ => {
                    // Handle other event types if needed
                    debug!("Ignoring non-key event: {:?}", event);
                }
            }
        }

        let _batch_time = batch_start.elapsed();

        Ok(processed_events)
    }

    /// Get comprehensive input statistics including performance metrics
    pub fn get_statistics(&self) -> EnhancedInputStatistics {
        let performance_stats = self.performance_monitor.get_performance_stats();
        let batch_stats = self.event_batcher.get_stats();

        EnhancedInputStatistics {
            // Legacy statistics
            total_processed: self.rate_limiter.get_total_processed(),
            filtered_sequences: self.escape_filter.get_filtered_count(),
            rate_limited_inputs: 0, // Would need to track this
            rate_limiter_stats: self.rate_limiter.get_stats(),

            // Performance metrics
            performance_stats: performance_stats.clone(),
            batch_stats: batch_stats.clone(),

            // Overall performance assessment
            performance_grade: self.calculate_performance_grade(&performance_stats, batch_stats),
        }
    }

    /// Calculate overall performance grade
    fn calculate_performance_grade(&self, perf_stats: &InputPerformanceStats, batch_stats: &BatchStats) -> char {
        let mut score = 100.0;

        // Deduct points for slow processing
        if perf_stats.p99_processing_time > Duration::from_millis(5) {
            score -= 20.0;
        }

        // Deduct points for poor batch efficiency
        if batch_stats.batch_efficiency < 5.0 { // Less than 5 events per ms
            score -= 15.0;
        }

        // Award points for good batching
        if batch_stats.avg_batch_size > 2.0 {
            score += 10.0;
        }

        match score as i32 {
            90..=100 => 'A',
            80..=89 => 'B',
            70..=79 => 'C',
            60..=69 => 'D',
            _ => 'F',
        }
    }

    /// Sanitize text input against injection attacks
    pub fn sanitize_text(&mut self, text: &str) -> Result<String> {
        // Filter out control characters and escape sequences
        let filtered = self.escape_filter.filter_string(text)?;

        // Validate character allowlist
        let validated = self.validate_characters(&filtered)?;

        // Check for suspicious patterns
        self.check_security_patterns(&validated)?;

        Ok(validated)
    }

    /// Validate that input contains only allowed characters
    pub fn validate_characters(&self, text: &str) -> Result<String> {
        let mut sanitized = String::new();

        for grapheme in text.graphemes(true) {
            if self.allowed_characters.is_allowed(grapheme) {
                sanitized.push_str(grapheme);
            } else {
                // Log suspicious input but don't fail - just filter it
                debug!("Filtered disallowed character: {:?}", grapheme);
            }
        }

        Ok(sanitized)
    }

    /// Check input length limits to prevent buffer overflow attacks
    pub fn check_length_limits(&self, text: &str) -> Result<()> {
        if text.len() > self.security_policy.max_input_length {
            return Err(CentotypeError::Input(format!(
                "Input too long: {} > {}",
                text.len(),
                self.security_policy.max_input_length
            )));
        }

        let grapheme_count = text.graphemes(true).count();
        if grapheme_count > self.security_policy.max_grapheme_count {
            return Err(CentotypeError::Input(format!(
                "Too many characters: {} > {}",
                grapheme_count, self.security_policy.max_grapheme_count
            )));
        }

        Ok(())
    }

    /// Update allowed character set based on training mode
    pub fn set_training_mode(&mut self, mode: TrainingMode) {
        self.allowed_characters.set_mode(mode);
        debug!("Updated allowed characters for mode: {:?}", mode);
    }
}

impl Default for EventBatcher {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for InputPerformanceMonitor {
    fn default() -> Self {
        Self::new()
    }
}

impl Input {
    // Private methods

    fn sanitize_and_validate(&mut self, key_event: KeyEvent) -> Result<ProcessedInput> {
        // Handle special key combinations first
        if key_event.modifiers.contains(KeyModifiers::CONTROL) {
            return Ok(ProcessedInput {
                input_type: InputType::Control(key_event),
                sanitized_char: None,
                is_valid: true,
                security_flags: SecurityFlags::default(),
            });
        }

        // Handle special keys
        match key_event.code {
            KeyCode::Char(c) => {
                let char_str = c.to_string();

                // Check character allowlist
                if !self.allowed_characters.is_allowed(&char_str) {
                    return Ok(ProcessedInput {
                        input_type: InputType::Filtered,
                        sanitized_char: None,
                        is_valid: false,
                        security_flags: SecurityFlags {
                            disallowed_character: true,
                            ..Default::default()
                        },
                    });
                }

                // Check for control characters in disguise
                if c.is_control() && c != '\t' && c != '\n' && c != '\r' {
                    warn!("Control character filtered: {:?}", c);
                    return Ok(ProcessedInput {
                        input_type: InputType::Filtered,
                        sanitized_char: None,
                        is_valid: false,
                        security_flags: SecurityFlags {
                            control_character: true,
                            ..Default::default()
                        },
                    });
                }

                Ok(ProcessedInput {
                    input_type: InputType::Character(c),
                    sanitized_char: Some(c),
                    is_valid: true,
                    security_flags: SecurityFlags::default(),
                })
            }
            KeyCode::Backspace => Ok(ProcessedInput {
                input_type: InputType::Backspace,
                sanitized_char: None,
                is_valid: true,
                security_flags: SecurityFlags::default(),
            }),
            KeyCode::Enter => Ok(ProcessedInput {
                input_type: InputType::Enter,
                sanitized_char: None,
                is_valid: true,
                security_flags: SecurityFlags::default(),
            }),
            KeyCode::Tab => {
                if self.allowed_characters.allows_tab() {
                    Ok(ProcessedInput {
                        input_type: InputType::Character('\t'),
                        sanitized_char: Some('\t'),
                        is_valid: true,
                        security_flags: SecurityFlags::default(),
                    })
                } else {
                    Ok(ProcessedInput {
                        input_type: InputType::Filtered,
                        sanitized_char: None,
                        is_valid: false,
                        security_flags: SecurityFlags::default(),
                    })
                }
            }
            KeyCode::Esc => Ok(ProcessedInput {
                input_type: InputType::Escape,
                sanitized_char: None,
                is_valid: true,
                security_flags: SecurityFlags::default(),
            }),
            _ => Ok(ProcessedInput {
                input_type: InputType::Other(key_event),
                sanitized_char: None,
                is_valid: false,
                security_flags: SecurityFlags::default(),
            }),
        }
    }

    fn check_security_patterns(&self, text: &str) -> Result<()> {
        // Check for common injection patterns
        for pattern in &self.security_policy.forbidden_patterns {
            if pattern.is_match(text) {
                warn!("Suspicious pattern detected in input: {}", text);
                return Err(CentotypeError::Input(
                    "Suspicious input pattern".to_string(),
                ));
            }
        }

        // Check for excessive repetition (potential DoS)
        if self.has_excessive_repetition(text) {
            return Err(CentotypeError::Input(
                "Excessive character repetition".to_string(),
            ));
        }

        Ok(())
    }

    fn has_excessive_repetition(&self, text: &str) -> bool {
        if text.len() < 10 {
            return false;
        }

        let chars: Vec<char> = text.chars().collect();
        let mut consecutive_count = 1;
        let mut max_consecutive = 1;

        for i in 1..chars.len() {
            if chars[i] == chars[i - 1] {
                consecutive_count += 1;
                max_consecutive = max_consecutive.max(consecutive_count);
            } else {
                consecutive_count = 1;
            }
        }

        max_consecutive > self.security_policy.max_consecutive_chars
    }
}

impl Default for Input {
    fn default() -> Self {
        Self::new()
    }
}

/// Allowed character configuration based on training mode
#[derive(Debug)]
struct AllowedCharacters {
    mode: Option<TrainingMode>,
    allow_letters: bool,
    allow_numbers: bool,
    allow_punctuation: bool,
    allow_symbols: bool,
    allow_whitespace: bool,
    allow_tab: bool,
    custom_allowed: Vec<char>,
}

impl AllowedCharacters {
    fn new() -> Self {
        Self {
            mode: None,
            allow_letters: true,
            allow_numbers: true,
            allow_punctuation: true,
            allow_symbols: true,
            allow_whitespace: true,
            allow_tab: false,
            custom_allowed: Vec::new(),
        }
    }

    fn set_mode(&mut self, mode: TrainingMode) {
        self.mode = Some(mode);

        match mode {
            TrainingMode::Arcade { level } => {
                let tier = level.tier();
                match tier.0 {
                    1..=2 => {
                        // Letters only for basic tiers
                        self.allow_letters = true;
                        self.allow_numbers = false;
                        self.allow_punctuation = false;
                        self.allow_symbols = false;
                        self.allow_whitespace = true;
                    }
                    3 => {
                        // Add numbers
                        self.allow_letters = true;
                        self.allow_numbers = true;
                        self.allow_punctuation = false;
                        self.allow_symbols = false;
                        self.allow_whitespace = true;
                    }
                    4..=5 => {
                        // Add punctuation
                        self.allow_letters = true;
                        self.allow_numbers = true;
                        self.allow_punctuation = true;
                        self.allow_symbols = false;
                        self.allow_whitespace = true;
                    }
                    6..=10 => {
                        // Everything allowed for advanced tiers
                        self.allow_letters = true;
                        self.allow_numbers = true;
                        self.allow_punctuation = true;
                        self.allow_symbols = true;
                        self.allow_whitespace = true;
                        self.allow_tab = true;
                    }
                    _ => {
                        // Default to basic letters
                        self.allow_letters = true;
                        self.allow_numbers = false;
                        self.allow_punctuation = false;
                        self.allow_symbols = false;
                        self.allow_whitespace = true;
                    }
                }
            }
            TrainingMode::Drill { category, .. } => {
                // Reset all to false, then enable based on category
                self.allow_letters = false;
                self.allow_numbers = false;
                self.allow_punctuation = false;
                self.allow_symbols = false;
                self.allow_whitespace = true;

                match category {
                    DrillCategory::Numbers => self.allow_numbers = true,
                    DrillCategory::Punctuation => self.allow_punctuation = true,
                    DrillCategory::Symbols => self.allow_symbols = true,
                    DrillCategory::CamelCase | DrillCategory::SnakeCase => {
                        self.allow_letters = true;
                    }
                    DrillCategory::Operators => {
                        self.allow_symbols = true;
                        self.allow_punctuation = true;
                    }
                }
            }
            TrainingMode::Endurance { .. } => {
                // Everything allowed for endurance mode
                self.allow_letters = true;
                self.allow_numbers = true;
                self.allow_punctuation = true;
                self.allow_symbols = true;
                self.allow_whitespace = true;
                self.allow_tab = true;
            }
        }
    }

    fn is_allowed(&self, grapheme: &str) -> bool {
        if grapheme.len() != 1 {
            // Multi-byte graphemes need special handling
            return self.is_complex_grapheme_allowed(grapheme);
        }

        let ch = grapheme.chars().next().unwrap();

        // Check custom allowed characters first
        if self.custom_allowed.contains(&ch) {
            return true;
        }

        // Check standard categories
        if self.allow_letters && ch.is_alphabetic() {
            return true;
        }

        if self.allow_numbers && ch.is_numeric() {
            return true;
        }

        if self.allow_whitespace && ch.is_whitespace() && ch != '\t' {
            return true;
        }

        if self.allow_tab && ch == '\t' {
            return true;
        }

        if self.allow_punctuation && self.is_punctuation(ch) {
            return true;
        }

        if self.allow_symbols && self.is_symbol(ch) {
            return true;
        }

        false
    }

    fn allows_tab(&self) -> bool {
        self.allow_tab
    }

    fn is_punctuation(&self, ch: char) -> bool {
        matches!(
            ch,
            '.' | ','
                | ';'
                | ':'
                | '!'
                | '?'
                | '"'
                | '\''
                | '('
                | ')'
                | '['
                | ']'
                | '{'
                | '}'
                | '-'
                | '_'
        )
    }

    fn is_symbol(&self, ch: char) -> bool {
        matches!(
            ch,
            '@' | '#'
                | '$'
                | '%'
                | '^'
                | '&'
                | '*'
                | '+'
                | '='
                | '<'
                | '>'
                | '/'
                | '\\'
                | '|'
                | '`'
                | '~'
        )
    }

    fn is_complex_grapheme_allowed(&self, _grapheme: &str) -> bool {
        // For now, reject complex graphemes for security
        // In a full implementation, you might have a whitelist of allowed Unicode ranges
        false
    }
}

/// Escape sequence filter to prevent terminal manipulation
#[derive(Debug)]
struct EscapeSequenceFilter {
    filtered_count: u64,
}

impl EscapeSequenceFilter {
    fn new() -> Self {
        Self { filtered_count: 0 }
    }

    fn filter_string(&mut self, input: &str) -> Result<String> {
        let mut filtered = String::new();
        let mut chars = input.chars();

        while let Some(ch) = chars.next() {
            match ch {
                '\x1b' => {
                    // ESC character - potential escape sequence
                    self.filtered_count += 1;
                    warn!("Filtered escape sequence starting with ESC");
                    // Skip the escape sequence
                    self.skip_escape_sequence(&mut chars);
                }
                '\x00'..='\x08' | '\x0b'..='\x1f' | '\x7f' => {
                    // Other control characters (except \t, \n, \r)
                    self.filtered_count += 1;
                    debug!("Filtered control character: {:02x}", ch as u8);
                }
                ch if ch.is_control() && ch != '\t' && ch != '\n' && ch != '\r' => {
                    // Unicode control characters
                    self.filtered_count += 1;
                    debug!("Filtered Unicode control character: U+{:04X}", ch as u32);
                }
                _ => {
                    filtered.push(ch);
                }
            }
        }

        Ok(filtered)
    }

    fn skip_escape_sequence(&self, chars: &mut std::str::Chars) {
        // Skip common escape sequence patterns
        // This is a simplified implementation - production would need more comprehensive handling
        let mut bracket_seen = false;
        let mut sequence_length = 0;

        for ch in chars.by_ref() {
            sequence_length += 1;
            if sequence_length > 20 {
                // Prevent infinite loops from malformed sequences
                break;
            }

            match ch {
                '[' if !bracket_seen => {
                    bracket_seen = true;
                }
                'A'..='Z' | 'a'..='z' if bracket_seen => {
                    // End of CSI sequence
                    break;
                }
                _ if !bracket_seen => {
                    // Simple escape sequence
                    break;
                }
                _ => {
                    // Continue reading sequence
                }
            }
        }
    }

    fn get_filtered_count(&self) -> u64 {
        self.filtered_count
    }
}

/// Rate limiter to prevent input flooding attacks
#[derive(Debug)]
struct RateLimiter {
    window_start: Instant,
    window_count: u32,
    total_processed: u64,
    max_per_window: u32,
    window_duration: Duration,
}

impl RateLimiter {
    fn new() -> Self {
        Self {
            window_start: Instant::now(),
            window_count: 0,
            total_processed: 0,
            max_per_window: 1000, // Max 1000 inputs per second
            window_duration: Duration::from_secs(1),
        }
    }

    fn allow_input(&mut self) -> bool {
        let now = Instant::now();

        // Reset window if expired
        if now.duration_since(self.window_start) >= self.window_duration {
            self.window_start = now;
            self.window_count = 0;
        }

        // Check if under limit
        if self.window_count >= self.max_per_window {
            return false;
        }

        self.window_count += 1;
        self.total_processed += 1;
        true
    }

    fn get_stats(&self) -> RateLimiterStats {
        RateLimiterStats {
            current_window_count: self.window_count,
            max_per_window: self.max_per_window,
            total_processed: self.total_processed,
        }
    }

    fn get_total_processed(&self) -> u64 {
        self.total_processed
    }
}

/// Security policy configuration
#[derive(Debug)]
struct SecurityPolicy {
    max_input_length: usize,
    max_grapheme_count: usize,
    max_consecutive_chars: usize,
    forbidden_patterns: Vec<Regex>,
}

impl Default for SecurityPolicy {
    fn default() -> Self {
        let mut forbidden_patterns = Vec::new();

        // Add common injection patterns
        if let Ok(regex) = Regex::new(r"\\x[0-9a-fA-F]{2}") {
            forbidden_patterns.push(regex);
        }
        if let Ok(regex) = Regex::new(r"\\u[0-9a-fA-F]{4}") {
            forbidden_patterns.push(regex);
        }
        if let Ok(regex) = Regex::new(r"\x1b\[[0-9;]*[a-zA-Z]") {
            forbidden_patterns.push(regex);
        }

        Self {
            max_input_length: 10000,   // 10KB max
            max_grapheme_count: 5000,  // Max 5000 characters
            max_consecutive_chars: 50, // Max 50 consecutive identical chars
            forbidden_patterns,
        }
    }
}

/// Processed input result
#[derive(Debug, Clone)]
pub struct ProcessedInput {
    pub input_type: InputType,
    pub sanitized_char: Option<char>,
    pub is_valid: bool,
    pub security_flags: SecurityFlags,
}

/// Type of processed input
#[derive(Debug, Clone)]
pub enum InputType {
    Character(char),
    Backspace,
    Enter,
    Escape,
    Control(KeyEvent),
    Filtered,
    Other(KeyEvent),
}

/// Security flags for input validation
#[derive(Debug, Clone, Default)]
pub struct SecurityFlags {
    pub disallowed_character: bool,
    pub control_character: bool,
    pub escape_sequence: bool,
    pub rate_limited: bool,
    pub pattern_match: bool,
}

/// Input processing statistics
#[derive(Debug, Clone)]
pub struct InputStatistics {
    pub rate_limiter_stats: RateLimiterStats,
    pub filtered_sequences: u64,
    pub total_processed: u64,
}

/// Rate limiter statistics
#[derive(Debug, Clone)]
pub struct RateLimiterStats {
    pub current_window_count: u32,
    pub max_per_window: u32,
    pub total_processed: u64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_character_processing() {
        let mut input_handler = Input::new();

        let key_event = KeyEvent {
            code: KeyCode::Char('a'),
            modifiers: KeyModifiers::NONE,
            kind: crossterm::event::KeyEventKind::Press,
            state: crossterm::event::KeyEventState::NONE,
        };

        let result = input_handler.process_key_event(key_event).unwrap();
        assert!(result.is_valid);
        assert_eq!(result.sanitized_char, Some('a'));
    }

    #[test]
    fn test_control_character_filtering() {
        let filter = EscapeSequenceFilter::new();
        let mut filter = filter;

        let result = filter.filter_string("hello\x1b[31mworld\x1b[0m").unwrap();
        assert_eq!(result, "helloworld");
        assert!(filter.get_filtered_count() > 0);
    }

    #[test]
    fn test_rate_limiting() {
        let mut rate_limiter = RateLimiter::new();

        // Should allow first 1000 inputs
        for _ in 0..1000 {
            assert!(rate_limiter.allow_input());
        }

        // Should reject further inputs in same window
        assert!(!rate_limiter.allow_input());
    }

    #[test]
    fn test_allowed_characters_by_mode() {
        let mut allowed = AllowedCharacters::new();

        // Test basic tier (letters only)
        allowed.set_mode(TrainingMode::Arcade {
            level: LevelId::new(1).unwrap(),
        });
        assert!(allowed.is_allowed("a"));
        assert!(!allowed.is_allowed("1"));
        assert!(!allowed.is_allowed("@"));

        // Test advanced tier (everything)
        allowed.set_mode(TrainingMode::Arcade {
            level: LevelId::new(91).unwrap(),
        });
        assert!(allowed.is_allowed("a"));
        assert!(allowed.is_allowed("1"));
        assert!(allowed.is_allowed("@"));
    }

    #[test]
    fn test_security_pattern_detection() {
        let input_handler = Input::new();

        // Test excessive repetition
        assert!(input_handler.has_excessive_repetition(&"a".repeat(100)));
        assert!(!input_handler.has_excessive_repetition("hello world"));
    }

    #[test]
    fn test_text_sanitization() {
        let mut input_handler = Input::new();

        let result = input_handler
            .sanitize_text("hello\x00world\x1b[31m")
            .unwrap();
        assert!(!result.contains('\x00'));
        assert!(!result.contains('\x1b'));
    }
}
