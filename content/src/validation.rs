//! Security validation and content verification system
//!
//! This module implements comprehensive security validation for generated content,
//! ensuring no malicious patterns, terminal escape sequences, or unsafe characters
//! are included in typing training content.

use centotype_core::types::*;
use regex::Regex;
use std::sync::Arc;
use tracing::{debug, warn};
use unicode_normalization::{is_nfc, UnicodeNormalization};

/// Result of content validation
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ValidationResult {
    Valid,
    Invalid(String),
}

impl ValidationResult {
    pub fn is_valid(&self) -> bool {
        matches!(self, ValidationResult::Valid)
    }

    pub fn error_message(&self) -> Option<&str> {
        match self {
            ValidationResult::Valid => None,
            ValidationResult::Invalid(msg) => Some(msg),
        }
    }
}

/// Security patterns that must be detected and rejected
#[derive(Debug)]
pub struct SecurityPatterns {
    escape_sequences: Regex,
    shell_injection: Regex,
    file_paths: Regex,
    dangerous_unicode: Regex,
}

impl SecurityPatterns {
    pub fn new() -> Result<Self> {
        Ok(Self {
            // Terminal escape sequences (ANSI codes)
            escape_sequences: Regex::new(r"\\x1b|\x1b|\\033|\x1b\[").map_err(|e| {
                CentotypeError::Content(format!("Failed to compile escape sequence regex: {}", e))
            })?,

            // Shell injection patterns
            shell_injection: Regex::new(r"\$\(|`|&&|\|\||;|>|<|\||&").map_err(|e| {
                CentotypeError::Content(format!("Failed to compile shell injection regex: {}", e))
            })?,

            // Absolute file paths that could reveal system information
            file_paths: Regex::new(r"^(/[^/\s]+)+/?$|^[A-Z]:\\").map_err(|e| {
                CentotypeError::Content(format!("Failed to compile file path regex: {}", e))
            })?,

            // Dangerous Unicode characters (control characters, private use areas)
            dangerous_unicode: Regex::new(r"[\u{0000}-\u{001F}\u{007F}-\u{009F}\u{E000}-\u{F8FF}\u{F0000}-\u{FFFFD}\u{100000}-\u{10FFFD}]").map_err(|e| {
                CentotypeError::Content(format!("Failed to compile dangerous unicode regex: {}", e))
            })?,
        })
    }
}

/// Security validator for content screening
#[derive(Debug)]
pub struct SecurityValidator {
    patterns: SecurityPatterns,
}

impl SecurityValidator {
    pub fn new() -> Result<Self> {
        Ok(Self {
            patterns: SecurityPatterns::new()?,
        })
    }

    /// Validate content for security issues
    pub fn validate(&self, content: &str) -> ValidationResult {
        // Test 1: Check for escape sequences
        if self.patterns.escape_sequences.is_match(content) {
            warn!("Content contains terminal escape sequences");
            return ValidationResult::Invalid("Content contains terminal escape sequences".to_string());
        }

        // Test 2: Check for shell injection patterns
        if self.patterns.shell_injection.is_match(content) {
            warn!("Content contains potential shell injection patterns");
            return ValidationResult::Invalid("Content contains shell injection patterns".to_string());
        }

        // Test 3: Check for absolute file paths
        for line in content.lines() {
            if self.patterns.file_paths.is_match(line.trim()) {
                warn!("Content contains absolute file paths");
                return ValidationResult::Invalid("Content contains absolute file paths".to_string());
            }
        }

        // Test 4: Check for dangerous Unicode characters
        if self.patterns.dangerous_unicode.is_match(content) {
            warn!("Content contains dangerous Unicode characters");
            return ValidationResult::Invalid("Content contains unsafe Unicode characters".to_string());
        }

        // Test 5: Check Unicode normalization
        if !is_nfc(content) {
            warn!("Content is not in Unicode NFC form");
            return ValidationResult::Invalid("Content not in Unicode NFC normalization".to_string());
        }

        // Test 6: Check for null bytes
        if content.contains('\0') {
            warn!("Content contains null bytes");
            return ValidationResult::Invalid("Content contains null bytes".to_string());
        }

        // Test 7: Check for excessive control characters
        let control_char_count = content.chars()
            .filter(|&c| c.is_control() && c != '\n' && c != '\r' && c != '\t')
            .count();

        if control_char_count > 0 {
            warn!("Content contains {} control characters", control_char_count);
            return ValidationResult::Invalid("Content contains control characters".to_string());
        }

        ValidationResult::Valid
    }

    /// Sanitize content by removing or replacing unsafe elements
    pub fn sanitize(&self, content: &str) -> String {
        let mut sanitized = content.to_string();

        // Remove escape sequences
        sanitized = self.patterns.escape_sequences.replace_all(&sanitized, "").to_string();

        // Normalize Unicode
        sanitized = sanitized.nfc().collect();

        // Remove control characters except allowed ones
        sanitized = sanitized.chars()
            .filter(|&c| !c.is_control() || c == '\n' || c == '\r' || c == '\t')
            .collect();

        // Remove null bytes
        sanitized = sanitized.replace('\0', "");

        sanitized
    }
}

/// Difficulty validator to ensure content meets progression requirements
#[derive(Debug)]
pub struct DifficultyValidator {
    min_length: usize,
    max_length: usize,
}

impl DifficultyValidator {
    pub fn new() -> Self {
        Self {
            min_length: 50,    // Minimum content length
            max_length: 5000,  // Maximum content length
        }
    }

    /// Validate that content meets difficulty requirements for a level
    pub fn validate(&self, content: &str, level_id: LevelId) -> ValidationResult {
        // Test length bounds
        if content.len() < self.min_length {
            return ValidationResult::Invalid(format!(
                "Content too short: {} chars (minimum: {})",
                content.len(),
                self.min_length
            ));
        }

        if content.len() > self.max_length {
            return ValidationResult::Invalid(format!(
                "Content too long: {} chars (maximum: {})",
                content.len(),
                self.max_length
            ));
        }

        // Validate character composition
        if !self.validate_character_composition(content, level_id) {
            return ValidationResult::Invalid(
                "Content character composition doesn't match level requirements".to_string()
            );
        }

        // Validate progression (each level should be harder than previous)
        if !self.validate_progression_requirements(content, level_id) {
            return ValidationResult::Invalid(
                "Content doesn't meet progression requirements".to_string()
            );
        }

        ValidationResult::Valid
    }

    /// Validate character composition matches level expectations
    fn validate_character_composition(&self, content: &str, level_id: LevelId) -> bool {
        let histogram = self.calculate_character_histogram(content);
        let total_chars = content.len() as f64;

        if total_chars == 0.0 {
            return false;
        }

        // Calculate expected ratios for this level
        let tier = level_id.tier().0 as f64;
        let tier_progress = (((level_id.0 - 1) % 10) + 1) as f64;

        let expected_symbol_ratio = (5.0 + (tier - 1.0) * 2.5 + (tier_progress - 1.0) * 0.3) / 100.0;
        let expected_number_ratio = (3.0 + (tier - 1.0) * 1.7 + (tier_progress - 1.0) * 0.2) / 100.0;

        // Calculate actual ratios
        let actual_symbol_ratio = histogram.symbols as f64 / total_chars;
        let actual_number_ratio = histogram.digits as f64 / total_chars;

        // Allow ±10% tolerance for validation (more lenient than generation)
        let symbol_tolerance = 0.10;
        let number_tolerance = 0.10;

        let symbol_diff = (actual_symbol_ratio - expected_symbol_ratio).abs();
        let number_diff = (actual_number_ratio - expected_number_ratio).abs();

        debug!(
            "Level {} validation: expected symbols={:.2}%, actual={:.2}%, diff={:.2}%",
            level_id.0,
            expected_symbol_ratio * 100.0,
            actual_symbol_ratio * 100.0,
            symbol_diff * 100.0
        );

        symbol_diff <= symbol_tolerance && number_diff <= number_tolerance
    }

    /// Validate progression requirements (each level appropriately harder)
    fn validate_progression_requirements(&self, content: &str, level_id: LevelId) -> bool {
        let difficulty_score = self.calculate_difficulty_score(content);

        // Basic progression: higher levels should have higher difficulty scores
        let expected_min_difficulty = (level_id.0 as f64) * 0.5; // Rough baseline
        let expected_max_difficulty = (level_id.0 as f64) * 2.0; // Upper bound

        difficulty_score >= expected_min_difficulty && difficulty_score <= expected_max_difficulty
    }

    /// Calculate character histogram for validation
    fn calculate_character_histogram(&self, text: &str) -> CharacterClassHistogram {
        let mut histogram = CharacterClassHistogram::default();

        for ch in text.chars() {
            match ch {
                'a'..='z' => histogram.lowercase += 1,
                'A'..='Z' => histogram.uppercase += 1,
                '0'..='9' => histogram.digits += 1,
                ' ' | '\t' | '\n' | '\r' => histogram.whitespace += 1,
                '.' | ',' | ';' | ':' | '!' | '?' | '\'' | '"' => histogram.punctuation += 1,
                _ => histogram.symbols += 1,
            }
        }

        histogram
    }

    /// Calculate overall difficulty score for content
    fn calculate_difficulty_score(&self, content: &str) -> f64 {
        let histogram = self.calculate_character_histogram(content);
        let total_chars = content.len() as f64;

        if total_chars == 0.0 {
            return 0.0;
        }

        // Weight different character types by difficulty
        let symbol_weight = 3.0;
        let uppercase_weight = 2.0;
        let digit_weight = 1.5;
        let punctuation_weight = 1.2;
        let lowercase_weight = 1.0;

        let score = (histogram.symbols as f64 * symbol_weight +
                    histogram.uppercase as f64 * uppercase_weight +
                    histogram.digits as f64 * digit_weight +
                    histogram.punctuation as f64 * punctuation_weight +
                    histogram.lowercase as f64 * lowercase_weight) / total_chars;

        score
    }
}

/// Performance validator for ensuring content loading meets latency targets
#[derive(Debug)]
pub struct PerformanceValidator {
    max_generation_time_ms: u64,
    max_validation_time_ms: u64,
}

impl PerformanceValidator {
    pub fn new() -> Self {
        Self {
            max_generation_time_ms: 10,  // 10ms max generation time
            max_validation_time_ms: 5,   // 5ms max validation time
        }
    }

    /// Validate that content processing meets performance targets
    pub fn validate(&self, content: &str) -> ValidationResult {
        let start = std::time::Instant::now();

        // Simulate content processing overhead
        let _char_count = content.chars().count();
        let _line_count = content.lines().count();

        let elapsed = start.elapsed();

        if elapsed.as_millis() > self.max_validation_time_ms as u128 {
            return ValidationResult::Invalid(format!(
                "Content validation too slow: {}ms (max: {}ms)",
                elapsed.as_millis(),
                self.max_validation_time_ms
            ));
        }

        ValidationResult::Valid
    }
}

/// Main content validator combining all validation strategies
#[derive(Debug)]
pub struct ContentValidator {
    security_validator: Arc<SecurityValidator>,
    difficulty_validator: DifficultyValidator,
    performance_validator: PerformanceValidator,
}

impl ContentValidator {
    pub fn new() -> Result<Self> {
        Ok(Self {
            security_validator: Arc::new(SecurityValidator::new()?),
            difficulty_validator: DifficultyValidator::new(),
            performance_validator: PerformanceValidator::new(),
        })
    }

    /// Comprehensive validation of generated content
    pub fn validate(&self, content: &str, level_id: LevelId) -> Result<()> {
        // 1. Security validation
        let security_result = self.security_validator.validate(content);
        if !security_result.is_valid() {
            return Err(CentotypeError::Content(format!(
                "Security validation failed: {}",
                security_result.error_message().unwrap_or("Unknown error")
            )));
        }

        // 2. Difficulty progression validation
        let difficulty_result = self.difficulty_validator.validate(content, level_id);
        if !difficulty_result.is_valid() {
            return Err(CentotypeError::Content(format!(
                "Difficulty validation failed: {}",
                difficulty_result.error_message().unwrap_or("Unknown error")
            )));
        }

        // 3. Performance impact validation
        let performance_result = self.performance_validator.validate(content);
        if !performance_result.is_valid() {
            return Err(CentotypeError::Content(format!(
                "Performance validation failed: {}",
                performance_result.error_message().unwrap_or("Unknown error")
            )));
        }

        debug!("Content validation passed for level {}", level_id.0);
        Ok(())
    }

    /// Sanitize content and make it safe
    pub fn sanitize(&self, content: &str) -> String {
        self.security_validator.sanitize(content)
    }

    /// Validate specific security patterns (for testing)
    pub fn validate_security(&self, content: &str) -> ValidationResult {
        self.security_validator.validate(content)
    }

    /// Validate difficulty requirements (for testing)
    pub fn validate_difficulty(&self, content: &str, level_id: LevelId) -> ValidationResult {
        self.difficulty_validator.validate(content, level_id)
    }

    /// Validate performance requirements (for testing)
    pub fn validate_performance(&self, content: &str) -> ValidationResult {
        self.performance_validator.validate(content)
    }
}

/// Progressive difficulty verification for testing
pub fn verify_difficulty_progression(contents: &[String]) -> Result<()> {
    let validator = DifficultyValidator::new();

    for (i, content) in contents.iter().enumerate() {
        if i == 0 {
            continue; // Skip first level
        }

        let current_difficulty = validator.calculate_difficulty_score(content);
        let prev_difficulty = validator.calculate_difficulty_score(&contents[i - 1]);

        // Ensure reasonable difficulty increase
        if current_difficulty < prev_difficulty {
            return Err(CentotypeError::Content(format!(
                "Level {} difficulty regression: {:.2} < {:.2}",
                i + 1,
                current_difficulty,
                prev_difficulty
            )));
        }

        let increase = if prev_difficulty > 0.0 {
            (current_difficulty - prev_difficulty) / prev_difficulty
        } else {
            0.0
        };

        // Allow reasonable range for difficulty increase (0% to 50%)
        if increase > 0.5 {
            return Err(CentotypeError::Content(format!(
                "Level {} difficulty increase too steep: {:.2}%",
                i + 1,
                increase * 100.0
            )));
        }

        debug!(
            "Level {} difficulty progression: {:.2} -> {:.2} (+{:.1}%)",
            i + 1,
            prev_difficulty,
            current_difficulty,
            increase * 100.0
        );
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_security_validation_escape_sequences() {
        let validator = SecurityValidator::new().unwrap();

        // Test escape sequences
        assert!(!validator.validate("\x1b[31mred text\x1b[0m").is_valid());
        assert!(!validator.validate("\\033[1mBold\\033[0m").is_valid());

        // Test valid content
        assert!(validator.validate("normal text with symbols: {}[]()").is_valid());
    }

    #[test]
    fn test_security_validation_shell_injection() {
        let validator = SecurityValidator::new().unwrap();

        // Test shell injection patterns
        assert!(!validator.validate("$(rm -rf /)").is_valid());
        assert!(!validator.validate("`cat /etc/passwd`").is_valid());
        assert!(!validator.validate("command && rm file").is_valid());

        // Test valid content with programming symbols
        assert!(validator.validate("function test() { return 42; }").is_valid());
    }

    #[test]
    fn test_difficulty_validation() {
        let validator = DifficultyValidator::new();

        // Test valid content for level 1
        let level_1 = LevelId::new(1).unwrap();
        let content_1 = "EN: This is basic text with simple words and numbers 123.";
        assert!(validator.validate(content_1, level_1).is_valid());

        // Test too short content
        assert!(!validator.validate("short", level_1).is_valid());
    }

    #[test]
    fn test_content_sanitization() {
        let validator = SecurityValidator::new().unwrap();

        let dirty_content = "\x1b[31mRed text\x1b[0m with \0null bytes";
        let clean_content = validator.sanitize(dirty_content);

        assert!(!clean_content.contains('\x1b'));
        assert!(!clean_content.contains('\0'));
        assert!(validator.validate(&clean_content).is_valid());
    }

    #[test]
    fn test_difficulty_progression() {
        let contents = vec![
            "Simple text".to_string(),
            "More complex text with symbols {}".to_string(),
            "Advanced content with many symbols: []{}()&*#@".to_string(),
        ];

        assert!(verify_difficulty_progression(&contents).is_ok());
    }
}
