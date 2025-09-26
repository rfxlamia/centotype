use std::collections::HashMap;
use serde::{Deserialize, Serialize};

/// Difficulty progression algorithm and validation metrics for Centotype content
///
/// This module implements scientifically-grounded difficulty scoring based on:
/// - Character frequency analysis (ETAOIN SHRDLU principle)
/// - Finger movement distance and complexity
/// - Bigram and trigram rarity patterns
/// - Symbol density and cognitive load factors

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DifficultyMetrics {
    pub character_frequency_weights: HashMap<CharacterClass, f64>,
    pub complexity_factors: HashMap<ComplexityType, f64>,
    pub progression_constraints: ProgressionConstraints,
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub enum CharacterClass {
    CommonLetters,     // E, T, A, O, I, N, S, H, R
    UncommonLetters,   // D, L, U, C, M, W, F, G, Y, P, B
    RareLetters,       // V, K, J, X, Q, Z
    BasicPunctuation,  // . , ! ?
    ComplexPunctuation,// ; : " ' ( )
    Digits,            // 0-9
    MathOperators,     // + - * / =
    ProgrammingSymbols,// [ ] { } < > | & ^ ~
    RareSymbols,       // % $ # @ \ _ `
}

#[derive(Debug, Clone, Serialize, Deserialize, Hash, Eq, PartialEq)]
pub enum ComplexityType {
    BigramRarity,      // Uncommon letter combinations
    CaseSwitching,     // Mixed case requirements
    FingerDistance,    // Physical key distance
    SymbolDensity,     // Ratio of symbols to letters
    LanguageSwitching, // Mixed language content
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressionConstraints {
    pub max_difficulty_jump: f64,    // Maximum increase between adjacent levels
    pub min_difficulty_increment: f64, // Minimum increase to ensure progression
    pub smoothness_factor: f64,      // Exponential smoothing for curve fitting
    pub tier_transition_buffer: f64, // Extra validation at tier boundaries
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentValidation {
    pub difficulty_score: f64,
    pub estimated_wpm: u32,
    pub character_distribution: HashMap<CharacterClass, f64>,
    pub complexity_breakdown: HashMap<ComplexityType, f64>,
    pub validation_passed: bool,
    pub validation_errors: Vec<ValidationError>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ValidationError {
    DifficultyTooHigh { expected: f64, actual: f64 },
    DifficultyTooLow { expected: f64, actual: f64 },
    InsufficientProgression { previous: f64, current: f64 },
    ExcessiveJump { previous: f64, current: f64, max_allowed: f64 },
    CharacterSetViolation { expected: String, found: String },
    ContentTooShort { min_length: usize, actual: usize },
    ContentTooLong { max_length: usize, actual: usize },
    RepetitionTooHigh { ratio: f64, max_allowed: f64 },
}

impl Default for DifficultyMetrics {
    fn default() -> Self {
        let mut character_weights = HashMap::new();
        character_weights.insert(CharacterClass::CommonLetters, 1.0);
        character_weights.insert(CharacterClass::UncommonLetters, 1.5);
        character_weights.insert(CharacterClass::RareLetters, 2.0);
        character_weights.insert(CharacterClass::BasicPunctuation, 1.8);
        character_weights.insert(CharacterClass::ComplexPunctuation, 2.2);
        character_weights.insert(CharacterClass::Digits, 1.4);
        character_weights.insert(CharacterClass::MathOperators, 1.9);
        character_weights.insert(CharacterClass::ProgrammingSymbols, 2.8);
        character_weights.insert(CharacterClass::RareSymbols, 3.5);

        let mut complexity_factors = HashMap::new();
        complexity_factors.insert(ComplexityType::BigramRarity, 0.3);
        complexity_factors.insert(ComplexityType::CaseSwitching, 0.4);
        complexity_factors.insert(ComplexityType::FingerDistance, 0.5);
        complexity_factors.insert(ComplexityType::SymbolDensity, 0.8);
        complexity_factors.insert(ComplexityType::LanguageSwitching, 1.0);

        let progression_constraints = ProgressionConstraints {
            max_difficulty_jump: 0.5,
            min_difficulty_increment: 0.1,
            smoothness_factor: 0.85,
            tier_transition_buffer: 0.2,
        };

        Self {
            character_frequency_weights: character_weights,
            complexity_factors,
            progression_constraints,
        }
    }
}

/// Calculate difficulty score for a given text content
pub fn calculate_difficulty_score(content: &str, metrics: &DifficultyMetrics) -> f64 {
    let char_distribution = analyze_character_distribution(content);
    let complexity_factors = analyze_complexity_factors(content);

    let mut base_score = 0.0;

    // Character frequency contribution (60% of total score)
    for (char_class, frequency) in char_distribution.iter() {
        if let Some(weight) = metrics.character_frequency_weights.get(char_class) {
            base_score += frequency * weight * 0.6;
        }
    }

    // Complexity factors contribution (40% of total score)
    for (complexity_type, factor_value) in complexity_factors.iter() {
        if let Some(weight) = metrics.complexity_factors.get(complexity_type) {
            base_score += factor_value * weight * 0.4;
        }
    }

    // Apply length normalization
    let length_factor = (content.len() as f64 / 100.0).ln().max(0.5);
    base_score * length_factor
}

/// Analyze the distribution of character classes in text
fn analyze_character_distribution(content: &str) -> HashMap<CharacterClass, f64> {
    let mut distribution = HashMap::new();
    let total_chars = content.len() as f64;

    if total_chars == 0.0 {
        return distribution;
    }

    let mut class_counts = HashMap::new();

    for ch in content.chars() {
        let char_class = classify_character(ch);
        *class_counts.entry(char_class).or_insert(0) += 1;
    }

    for (char_class, count) in class_counts {
        distribution.insert(char_class, count as f64 / total_chars);
    }

    distribution
}

/// Classify a character into its difficulty class
fn classify_character(ch: char) -> CharacterClass {
    match ch {
        'e' | 't' | 'a' | 'o' | 'i' | 'n' | 's' | 'h' | 'r' |
        'E' | 'T' | 'A' | 'O' | 'I' | 'N' | 'S' | 'H' | 'R' => CharacterClass::CommonLetters,

        'd' | 'l' | 'u' | 'c' | 'm' | 'w' | 'f' | 'g' | 'y' | 'p' | 'b' |
        'D' | 'L' | 'U' | 'C' | 'M' | 'W' | 'F' | 'G' | 'Y' | 'P' | 'B' => CharacterClass::UncommonLetters,

        'v' | 'k' | 'j' | 'x' | 'q' | 'z' |
        'V' | 'K' | 'J' | 'X' | 'Q' | 'Z' => CharacterClass::RareLetters,

        '.' | ',' | '!' | '?' => CharacterClass::BasicPunctuation,
        ';' | ':' | '"' | '\'' | '(' | ')' => CharacterClass::ComplexPunctuation,

        '0'..='9' => CharacterClass::Digits,
        '+' | '-' | '*' | '/' | '=' => CharacterClass::MathOperators,

        '[' | ']' | '{' | '}' | '<' | '>' | '|' | '&' | '^' | '~' => CharacterClass::ProgrammingSymbols,
        '%' | '$' | '#' | '@' | '\\' | '_' | '`' => CharacterClass::RareSymbols,

        _ => CharacterClass::CommonLetters, // Default for spaces and other chars
    }
}

/// Analyze complexity factors in the text
fn analyze_complexity_factors(content: &str) -> HashMap<ComplexityType, f64> {
    let mut factors = HashMap::new();

    factors.insert(ComplexityType::BigramRarity, calculate_bigram_rarity(content));
    factors.insert(ComplexityType::CaseSwitching, calculate_case_switching(content));
    factors.insert(ComplexityType::FingerDistance, calculate_finger_distance(content));
    factors.insert(ComplexityType::SymbolDensity, calculate_symbol_density(content));
    factors.insert(ComplexityType::LanguageSwitching, detect_language_switching(content));

    factors
}

/// Calculate bigram rarity score based on English frequency patterns
fn calculate_bigram_rarity(content: &str) -> f64 {
    let common_bigrams = [
        "th", "er", "on", "an", "re", "he", "in", "ed", "nd", "ha",
        "at", "en", "es", "of", "or", "nt", "ea", "ti", "to", "it"
    ];

    let chars: Vec<char> = content.chars().collect();
    if chars.len() < 2 {
        return 0.0;
    }

    let mut rare_bigrams = 0;
    let mut total_bigrams = 0;

    for window in chars.windows(2) {
        if window[0].is_alphabetic() && window[1].is_alphabetic() {
            let bigram: String = window.iter().collect::<String>().to_lowercase();
            total_bigrams += 1;

            if !common_bigrams.contains(&bigram.as_str()) {
                rare_bigrams += 1;
            }
        }
    }

    if total_bigrams > 0 {
        rare_bigrams as f64 / total_bigrams as f64
    } else {
        0.0
    }
}

/// Calculate case switching complexity
fn calculate_case_switching(content: &str) -> f64 {
    let chars: Vec<char> = content.chars().filter(|c| c.is_alphabetic()).collect();
    if chars.len() < 2 {
        return 0.0;
    }

    let mut switches = 0;
    for window in chars.windows(2) {
        if window[0].is_lowercase() != window[1].is_lowercase() {
            switches += 1;
        }
    }

    switches as f64 / (chars.len() - 1) as f64
}

/// Calculate finger movement distance complexity
fn calculate_finger_distance(content: &str) -> f64 {
    // Simplified finger distance calculation based on QWERTY layout
    let key_positions = create_qwerty_position_map();
    let chars: Vec<char> = content.chars().collect();

    if chars.len() < 2 {
        return 0.0;
    }

    let mut total_distance = 0.0;
    let mut valid_transitions = 0;

    for window in chars.windows(2) {
        if let (Some(pos1), Some(pos2)) = (key_positions.get(&window[0]), key_positions.get(&window[1])) {
            let distance = ((pos1.0 - pos2.0).pow(2) + (pos1.1 - pos2.1).pow(2)).sqrt();
            total_distance += distance;
            valid_transitions += 1;
        }
    }

    if valid_transitions > 0 {
        total_distance / valid_transitions as f64 / 5.0 // Normalize to 0-1 range
    } else {
        0.0
    }
}

/// Create a simplified QWERTY keyboard position map
fn create_qwerty_position_map() -> HashMap<char, (f64, f64)> {
    let mut positions = HashMap::new();

    // Row 1 (numbers)
    let row1 = "1234567890";
    for (i, ch) in row1.chars().enumerate() {
        positions.insert(ch, (i as f64, 0.0));
    }

    // Row 2 (QWERTY)
    let row2 = "qwertyuiop";
    for (i, ch) in row2.chars().enumerate() {
        positions.insert(ch, (i as f64 + 0.5, 1.0));
        positions.insert(ch.to_uppercase().next().unwrap(), (i as f64 + 0.5, 1.0));
    }

    // Row 3 (ASDF)
    let row3 = "asdfghjkl";
    for (i, ch) in row3.chars().enumerate() {
        positions.insert(ch, (i as f64 + 0.75, 2.0));
        positions.insert(ch.to_uppercase().next().unwrap(), (i as f64 + 0.75, 2.0));
    }

    // Row 4 (ZXCV)
    let row4 = "zxcvbnm";
    for (i, ch) in row4.chars().enumerate() {
        positions.insert(ch, (i as f64 + 1.25, 3.0));
        positions.insert(ch.to_uppercase().next().unwrap(), (i as f64 + 1.25, 3.0));
    }

    positions
}

/// Calculate symbol density in the text
fn calculate_symbol_density(content: &str) -> f64 {
    let total_chars = content.len();
    if total_chars == 0 {
        return 0.0;
    }

    let symbol_count = content.chars().filter(|c| {
        !c.is_alphanumeric() && !c.is_whitespace() && *c != '.' && *c != ','
    }).count();

    symbol_count as f64 / total_chars as f64
}

/// Detect language switching patterns (simplified heuristic)
fn detect_language_switching(content: &str) -> f64 {
    // This is a simplified implementation
    // A full implementation would use language detection libraries
    let words: Vec<&str> = content.split_whitespace().collect();
    if words.len() < 2 {
        return 0.0;
    }

    // Heuristic: look for patterns indicating Indonesian vs English
    let indonesian_indicators = ["yang", "dan", "ini", "itu", "dengan", "untuk", "dari", "pada"];
    let english_indicators = ["the", "and", "this", "that", "with", "for", "from", "on"];

    let mut indonesian_words = 0;
    let mut english_words = 0;

    for word in &words {
        let lower_word = word.to_lowercase();
        if indonesian_indicators.contains(&lower_word.as_str()) {
            indonesian_words += 1;
        } else if english_indicators.contains(&lower_word.as_str()) {
            english_words += 1;
        }
    }

    let total_indicator_words = indonesian_words + english_words;
    if total_indicator_words > 0 && indonesian_words > 0 && english_words > 0 {
        1.0 // Language switching detected
    } else {
        0.0
    }
}

/// Validate content against expected difficulty and constraints
pub fn validate_content(
    content: &str,
    level: u32,
    expected_difficulty: f64,
    metrics: &DifficultyMetrics,
) -> ContentValidation {
    let mut validation = ContentValidation {
        difficulty_score: calculate_difficulty_score(content, metrics),
        estimated_wpm: estimate_wpm_from_difficulty(calculate_difficulty_score(content, metrics)),
        character_distribution: analyze_character_distribution(content),
        complexity_breakdown: analyze_complexity_factors(content),
        validation_passed: true,
        validation_errors: Vec::new(),
    };

    // Check difficulty score against expected value
    let difficulty_tolerance = 0.3;
    if validation.difficulty_score > expected_difficulty + difficulty_tolerance {
        validation.validation_errors.push(ValidationError::DifficultyTooHigh {
            expected: expected_difficulty,
            actual: validation.difficulty_score,
        });
        validation.validation_passed = false;
    } else if validation.difficulty_score < expected_difficulty - difficulty_tolerance {
        validation.validation_errors.push(ValidationError::DifficultyTooLow {
            expected: expected_difficulty,
            actual: validation.difficulty_score,
        });
        validation.validation_passed = false;
    }

    // Check content length constraints
    let (min_length, max_length) = get_length_constraints_for_level(level);
    if content.len() < min_length {
        validation.validation_errors.push(ValidationError::ContentTooShort {
            min_length,
            actual: content.len(),
        });
        validation.validation_passed = false;
    } else if content.len() > max_length {
        validation.validation_errors.push(ValidationError::ContentTooLong {
            max_length,
            actual: content.len(),
        });
        validation.validation_passed = false;
    }

    // Check for excessive repetition
    let repetition_ratio = calculate_repetition_ratio(content);
    if repetition_ratio > 0.3 {
        validation.validation_errors.push(ValidationError::RepetitionTooHigh {
            ratio: repetition_ratio,
            max_allowed: 0.3,
        });
        validation.validation_passed = false;
    }

    validation
}

/// Get appropriate length constraints for a given level
fn get_length_constraints_for_level(level: u32) -> (usize, usize) {
    match level {
        1..=10 => (40, 120),
        11..=20 => (80, 180),
        21..=40 => (120, 280),
        41..=60 => (160, 320),
        61..=80 => (240, 420),
        81..=100 => (350, 550),
        _ => (40, 120),
    }
}

/// Calculate repetition ratio in text
fn calculate_repetition_ratio(content: &str) -> f64 {
    let words: Vec<&str> = content.split_whitespace().collect();
    if words.is_empty() {
        return 0.0;
    }

    let mut word_counts = HashMap::new();
    for word in &words {
        *word_counts.entry(word.to_lowercase()).or_insert(0) += 1;
    }

    let total_words = words.len();
    let unique_words = word_counts.len();

    1.0 - (unique_words as f64 / total_words as f64)
}

/// Estimate WPM from difficulty score
fn estimate_wpm_from_difficulty(difficulty_score: f64) -> u32 {
    // Inverse relationship: higher difficulty = lower expected WPM
    let base_wpm = 80.0;
    let difficulty_penalty = difficulty_score * 5.0;
    (base_wpm - difficulty_penalty).max(20.0) as u32
}

/// Generate expected difficulty curve for all 100 levels
pub fn generate_difficulty_curve() -> Vec<f64> {
    let mut curve = Vec::with_capacity(100);

    for level in 1..=100 {
        let normalized_level = (level - 1) as f64 / 99.0; // 0.0 to 1.0

        // Exponential curve with tier-based acceleration
        let tier = ((level - 1) / 20) + 1;
        let tier_factor = match tier {
            1 => 1.0,  // Letters: gentle start
            2 => 1.2,  // Punctuation: moderate increase
            3 => 1.5,  // Numbers: steeper curve
            4 => 2.0,  // Symbols: rapid acceleration
            _ => 2.0,
        };

        // Base exponential curve: starts at 1.0, ends at 16.0
        let base_difficulty = 1.0 + (normalized_level.powf(1.8) * 15.0);
        let adjusted_difficulty = base_difficulty * tier_factor;

        curve.push(adjusted_difficulty);
    }

    curve
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_difficulty_calculation() {
        let metrics = DifficultyMetrics::default();

        // Simple text should have low difficulty
        let simple = "the quick brown fox";
        let simple_score = calculate_difficulty_score(simple, &metrics);

        // Complex text should have higher difficulty
        let complex = "Complex algorithms: O(n²) = n*log₂(n) + Σ(i=1→∞)[f(x)]";
        let complex_score = calculate_difficulty_score(complex, &metrics);

        assert!(complex_score > simple_score);
    }

    #[test]
    fn test_difficulty_curve_progression() {
        let curve = generate_difficulty_curve();

        // Ensure curve has correct length
        assert_eq!(curve.len(), 100);

        // Ensure curve is strictly increasing
        for i in 1..curve.len() {
            assert!(curve[i] > curve[i-1]);
        }

        // Ensure reasonable range
        assert!(curve[0] >= 1.0);
        assert!(curve[99] <= 20.0);
    }

    #[test]
    fn test_character_classification() {
        assert_eq!(classify_character('e'), CharacterClass::CommonLetters);
        assert_eq!(classify_character('z'), CharacterClass::RareLetters);
        assert_eq!(classify_character('!'), CharacterClass::BasicPunctuation);
        assert_eq!(classify_character('{'), CharacterClass::ProgrammingSymbols);
    }
}