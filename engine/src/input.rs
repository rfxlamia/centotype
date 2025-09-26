//! Input handler with security sanitization and escape sequence filtering
use crate::*;
use centotype_core::types::*;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use regex::Regex;
use std::collections::HashMap;
use std::time::{Duration, Instant};
use tracing::{debug, warn};
use unicode_segmentation::UnicodeSegmentation;

/// Secure input processor with sanitization and validation
pub struct Input {
    allowed_characters: AllowedCharacters,
    escape_filter: EscapeSequenceFilter,
    rate_limiter: RateLimiter,
    security_policy: SecurityPolicy,
}

impl Input {
    pub fn new() -> Self {
        Self {
            allowed_characters: AllowedCharacters::new(),
            escape_filter: EscapeSequenceFilter::new(),
            rate_limiter: RateLimiter::new(),
            security_policy: SecurityPolicy::default(),
        }
    }

    /// Process and sanitize keyboard input with security validation
    pub fn process_key_event(&mut self, key_event: KeyEvent) -> Result<ProcessedInput> {
        let start_time = Instant::now();

        // Rate limiting check
        if !self.rate_limiter.allow_input() {
            warn!("Input rate limit exceeded");
            return Err(CentotypeError::Input("Rate limit exceeded".to_string()));
        }

        // Process the key event
        let processed = self.sanitize_and_validate(key_event)?;

        // Record processing time for monitoring
        let processing_time = start_time.elapsed();
        debug!(
            duration_ms = %processing_time.as_millis(),
            input_type = ?processed.input_type,
            "Processed input"
        );

        Ok(processed)
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
                grapheme_count,
                self.security_policy.max_grapheme_count
            )));
        }

        Ok(())
    }

    /// Update allowed character set based on training mode
    pub fn set_training_mode(&mut self, mode: TrainingMode) {
        self.allowed_characters.set_mode(mode);
        debug!("Updated allowed characters for mode: {:?}", mode);
    }

    /// Get input processing statistics
    pub fn get_statistics(&self) -> InputStatistics {
        InputStatistics {
            rate_limiter_stats: self.rate_limiter.get_stats(),
            filtered_sequences: self.escape_filter.get_filtered_count(),
            total_processed: self.rate_limiter.get_total_processed(),
        }
    }

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
                return Err(CentotypeError::Input("Suspicious input pattern".to_string()));
            }
        }

        // Check for excessive repetition (potential DoS)
        if self.has_excessive_repetition(text) {
            return Err(CentotypeError::Input("Excessive character repetition".to_string()));
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
        matches!(ch, '.' | ',' | ';' | ':' | '!' | '?' | '"' | '\'' | '(' | ')' | '[' | ']' | '{' | '}' | '-' | '_')
    }

    fn is_symbol(&self, ch: char) -> bool {
        matches!(ch, '@' | '#' | '$' | '%' | '^' | '&' | '*' | '+' | '=' | '<' | '>' | '/' | '\\' | '|' | '`' | '~')
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
        Self {
            filtered_count: 0,
        }
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

        while let Some(ch) = chars.next() {
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
            max_input_length: 10000,    // 10KB max
            max_grapheme_count: 5000,   // Max 5000 characters
            max_consecutive_chars: 50,  // Max 50 consecutive identical chars
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
            level: LevelId::new(1).unwrap()
        });
        assert!(allowed.is_allowed("a"));
        assert!(!allowed.is_allowed("1"));
        assert!(!allowed.is_allowed("@"));

        // Test advanced tier (everything)
        allowed.set_mode(TrainingMode::Arcade {
            level: LevelId::new(91).unwrap()
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
        let input_handler = Input::new();

        let result = input_handler.sanitize_text("hello\x00world\x1b[31m").unwrap();
        assert!(!result.contains('\x00'));
        assert!(!result.contains('\x1b'));
    }
}
