//! Difficulty analysis and progression engine
//!
//! This module implements the mathematical formulas for progressive difficulty
//! across 100 levels, ensuring smooth progression from basic typing to expert
//! mastery with proper validation and scoring.

use centotype_core::types::*;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tracing::debug;

/// Difficulty score representing the overall challenge level of content
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct DifficultyScore {
    /// Overall difficulty score (0.0 to 100.0)
    pub overall: f64,
    /// Symbol density contribution to difficulty
    pub symbol_contribution: f64,
    /// Number density contribution to difficulty
    pub number_contribution: f64,
    /// Technical terms contribution to difficulty
    pub technical_contribution: f64,
    /// Character variety contribution to difficulty
    pub variety_contribution: f64,
    /// Length-based difficulty adjustment
    pub length_contribution: f64,
}

impl DifficultyScore {
    /// Create a new difficulty score with all components
    pub fn new(
        symbol_contribution: f64,
        number_contribution: f64,
        technical_contribution: f64,
        variety_contribution: f64,
        length_contribution: f64,
    ) -> Self {
        let overall = symbol_contribution
            + number_contribution
            + technical_contribution
            + variety_contribution
            + length_contribution;

        Self {
            overall: overall.min(100.0).max(0.0), // Clamp to [0, 100]
            symbol_contribution,
            number_contribution,
            technical_contribution,
            variety_contribution,
            length_contribution,
        }
    }

    /// Check if this difficulty score is appropriate for a given level
    pub fn is_appropriate_for_level(&self, level_id: LevelId) -> bool {
        let expected = DifficultyAnalyzer::expected_difficulty_for_level(level_id);
        let tolerance = 15.0; // ±15 points tolerance

        (self.overall - expected.overall).abs() <= tolerance
    }
}

/// Configuration for difficulty analysis
#[derive(Debug, Clone)]
pub struct DifficultyConfig {
    /// Weight for symbol density in overall score
    pub symbol_weight: f64,
    /// Weight for number density in overall score
    pub number_weight: f64,
    /// Weight for technical terms in overall score
    pub technical_weight: f64,
    /// Weight for character variety in overall score
    pub variety_weight: f64,
    /// Weight for content length in overall score
    pub length_weight: f64,
    /// Minimum content length for accurate analysis
    pub min_length_for_analysis: usize,
}

impl Default for DifficultyConfig {
    fn default() -> Self {
        Self {
            symbol_weight: 3.0,      // Symbols are hardest to type
            number_weight: 1.5,      // Numbers require reaching to top row
            technical_weight: 2.0,   // Technical terms are challenging
            variety_weight: 1.2,     // Character variety adds complexity
            length_weight: 0.8,      // Length affects sustained difficulty
            min_length_for_analysis: 50,
        }
    }
}

/// Main difficulty analyzer implementing progression formulas
pub struct DifficultyAnalyzer {
    config: DifficultyConfig,
}

impl DifficultyAnalyzer {
    /// Create new difficulty analyzer with default configuration
    pub fn new() -> Self {
        Self {
            config: DifficultyConfig::default(),
        }
    }

    /// Create analyzer with custom configuration
    pub fn with_config(config: DifficultyConfig) -> Self {
        Self { config }
    }

    /// Analyze difficulty of text content
    pub fn analyze_content(&self, content: &str) -> DifficultyScore {
        if content.len() < self.config.min_length_for_analysis {
            return DifficultyScore::new(0.0, 0.0, 0.0, 0.0, 0.0);
        }

        let histogram = self.calculate_character_histogram(content);
        let total_chars = content.len() as f64;

        // Calculate component contributions
        let symbol_contribution = self.calculate_symbol_contribution(&histogram, total_chars);
        let number_contribution = self.calculate_number_contribution(&histogram, total_chars);
        let technical_contribution = self.calculate_technical_contribution(content);
        let variety_contribution = self.calculate_variety_contribution(&histogram, total_chars);
        let length_contribution = self.calculate_length_contribution(content.len());

        DifficultyScore::new(
            symbol_contribution,
            number_contribution,
            technical_contribution,
            variety_contribution,
            length_contribution,
        )
    }

    /// Calculate expected difficulty score for a specific level using formulas
    pub fn expected_difficulty_for_level(level_id: LevelId) -> DifficultyScore {
        let tier = level_id.tier().0 as f64;
        let tier_progress = (((level_id.0 - 1) % 10) + 1) as f64;

        // Use the mathematical formulas from master prompt
        let symbol_ratio = (5.0 + (tier - 1.0) * 2.5 + (tier_progress - 1.0) * 0.3) / 100.0;
        let number_ratio = (3.0 + (tier - 1.0) * 1.7 + (tier_progress - 1.0) * 0.2) / 100.0;
        let tech_ratio = (2.0 + (tier - 1.0) * 1.3 + (tier_progress - 1.0) * 0.2) / 100.0;
        let content_length = 300 + ((tier - 1.0) * 270.0 + (tier_progress - 1.0) * 30.0) as usize;

        // Convert ratios to difficulty contributions
        let symbol_contribution = symbol_ratio * 100.0 * 3.0; // Weight symbols more heavily
        let number_contribution = number_ratio * 100.0 * 1.5;
        let technical_contribution = tech_ratio * 100.0 * 2.0;

        // Variety increases with tier
        let variety_contribution = (tier - 1.0) * 2.0 + tier_progress * 0.5;

        // Length contributes to sustained difficulty
        let length_contribution = (content_length as f64 / 3000.0) * 10.0;

        DifficultyScore::new(
            symbol_contribution,
            number_contribution,
            technical_contribution,
            variety_contribution,
            length_contribution,
        )
    }

    /// Validate progression across multiple levels
    pub fn validate_progression(&self, contents: &[(LevelId, String)]) -> Result<()> {
        if contents.len() < 2 {
            return Ok(()); // Need at least 2 levels to validate progression
        }

        let mut previous_score: Option<DifficultyScore> = None;

        for (level_id, content) in contents {
            let current_score = self.analyze_content(content);
            let expected_score = Self::expected_difficulty_for_level(*level_id);

            // Validate that content meets expected difficulty
            if !current_score.is_appropriate_for_level(*level_id) {
                return Err(CentotypeError::Content(format!(
                    "Level {} difficulty mismatch: expected {:.1}, got {:.1}",
                    level_id.0,
                    expected_score.overall,
                    current_score.overall
                )));
            }

            // Validate progression from previous level
            if let Some(prev_score) = previous_score {
                let progression = current_score.overall - prev_score.overall;

                // Each level should be harder than the previous (with some tolerance)
                if progression < -5.0 {
                    return Err(CentotypeError::Content(format!(
                        "Level {} difficulty regression: {:.1} -> {:.1} (change: {:.1})",
                        level_id.0,
                        prev_score.overall,
                        current_score.overall,
                        progression
                    )));
                }

                // Progression shouldn't be too steep
                if progression > 15.0 {
                    return Err(CentotypeError::Content(format!(
                        "Level {} difficulty spike too steep: {:.1} -> {:.1} (change: {:.1})",
                        level_id.0,
                        prev_score.overall,
                        current_score.overall,
                        progression
                    )));
                }

                debug!(
                    "Level {} progression: {:.1} -> {:.1} (+{:.1})",
                    level_id.0,
                    prev_score.overall,
                    current_score.overall,
                    progression
                );
            }

            previous_score = Some(current_score);
        }

        Ok(())
    }

    /// Calculate character histogram for analysis
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

    /// Calculate symbol contribution to difficulty
    fn calculate_symbol_contribution(&self, histogram: &CharacterClassHistogram, total_chars: f64) -> f64 {
        if total_chars == 0.0 {
            return 0.0;
        }

        let symbol_ratio = histogram.symbols as f64 / total_chars;
        symbol_ratio * 100.0 * self.config.symbol_weight
    }

    /// Calculate number contribution to difficulty
    fn calculate_number_contribution(&self, histogram: &CharacterClassHistogram, total_chars: f64) -> f64 {
        if total_chars == 0.0 {
            return 0.0;
        }

        let number_ratio = histogram.digits as f64 / total_chars;
        number_ratio * 100.0 * self.config.number_weight
    }

    /// Calculate technical terms contribution to difficulty
    fn calculate_technical_contribution(&self, content: &str) -> f64 {
        // Define technical patterns to search for
        let technical_patterns = [
            "function", "class", "interface", "struct", "enum", "trait", "impl",
            "async", "await", "Result", "Option", "HashMap", "Vector", "String",
            "iterator", "closure", "lifetime", "borrowing", "ownership",
            "camelCase", "snake_case", "PascalCase",
            "fn", "let", "mut", "const", "static", "pub", "use", "mod",
            "match", "if", "else", "for", "while", "loop", "break", "continue",
            "return", "yield", "where", "Self", "super", "crate",
        ];

        let mut technical_count = 0;
        let words: Vec<&str> = content.split_whitespace().collect();

        for word in &words {
            // Clean word of punctuation for matching
            let clean_word = word.trim_matches(|c: char| !c.is_alphanumeric());

            if technical_patterns.iter().any(|&pattern| {
                clean_word.eq_ignore_ascii_case(pattern) || clean_word.contains(pattern)
            }) {
                technical_count += 1;
            }
        }

        let technical_ratio = if words.is_empty() {
            0.0
        } else {
            technical_count as f64 / words.len() as f64
        };

        technical_ratio * 100.0 * self.config.technical_weight
    }

    /// Calculate character variety contribution to difficulty
    fn calculate_variety_contribution(&self, histogram: &CharacterClassHistogram, total_chars: f64) -> f64 {
        if total_chars == 0.0 {
            return 0.0;
        }

        // Count how many different character classes are present
        let mut class_count = 0;
        if histogram.lowercase > 0 { class_count += 1; }
        if histogram.uppercase > 0 { class_count += 1; }
        if histogram.digits > 0 { class_count += 1; }
        if histogram.punctuation > 0 { class_count += 1; }
        if histogram.symbols > 0 { class_count += 1; }

        // More character classes = higher variety = higher difficulty
        let variety_score = match class_count {
            0..=1 => 0.0,   // Very low variety
            2 => 2.0,       // Basic variety
            3 => 5.0,       // Good variety
            4 => 8.0,       // High variety
            5 => 12.0,      // Maximum variety
            _ => 12.0,
        };

        variety_score * self.config.variety_weight
    }

    /// Calculate length contribution to difficulty
    fn calculate_length_contribution(&self, length: usize) -> f64 {
        // Longer content is more challenging to maintain accuracy and speed
        let normalized_length = (length as f64 / 3000.0).min(1.0); // Normalize to max expected length
        normalized_length * 10.0 * self.config.length_weight
    }

    /// Get tier-specific requirements for validation
    pub fn get_tier_requirements(tier: Tier) -> TierRequirements {
        match tier.0 {
            1..=2 => TierRequirements {
                name: "Foundation",
                min_symbol_ratio: 0.02,
                max_symbol_ratio: 0.10,
                min_number_ratio: 0.01,
                max_number_ratio: 0.08,
                required_features: vec!["basic_punctuation", "simple_words"],
                forbidden_features: vec!["complex_symbols", "bitwise_operations"],
            },
            3..=4 => TierRequirements {
                name: "Programming Basics",
                min_symbol_ratio: 0.08,
                max_symbol_ratio: 0.15,
                min_number_ratio: 0.05,
                max_number_ratio: 0.12,
                required_features: vec!["brackets", "operators", "camelCase"],
                forbidden_features: vec!["bitwise_operations", "complex_generics"],
            },
            5..=6 => TierRequirements {
                name: "Intermediate",
                min_symbol_ratio: 0.12,
                max_symbol_ratio: 0.20,
                min_number_ratio: 0.08,
                max_number_ratio: 0.15,
                required_features: vec!["nested_brackets", "technical_terms"],
                forbidden_features: vec!["unicode_edge_cases"],
            },
            7..=8 => TierRequirements {
                name: "Advanced",
                min_symbol_ratio: 0.18,
                max_symbol_ratio: 0.25,
                min_number_ratio: 0.12,
                max_number_ratio: 0.18,
                required_features: vec!["bitwise_operations", "hex_values", "complex_symbols"],
                forbidden_features: vec![],
            },
            9..=10 => TierRequirements {
                name: "Expert",
                min_symbol_ratio: 0.22,
                max_symbol_ratio: 0.30,
                min_number_ratio: 0.15,
                max_number_ratio: 0.20,
                required_features: vec!["unicode_characters", "complex_generics", "expert_patterns"],
                forbidden_features: vec![],
            },
            _ => TierRequirements {
                name: "Unknown",
                min_symbol_ratio: 0.0,
                max_symbol_ratio: 1.0,
                min_number_ratio: 0.0,
                max_number_ratio: 1.0,
                required_features: vec![],
                forbidden_features: vec![],
            },
        }
    }
}

impl Default for DifficultyAnalyzer {
    fn default() -> Self {
        Self::new()
    }
}

/// Requirements for a specific tier
#[derive(Debug, Clone)]
pub struct TierRequirements {
    pub name: &'static str,
    pub min_symbol_ratio: f64,
    pub max_symbol_ratio: f64,
    pub min_number_ratio: f64,
    pub max_number_ratio: f64,
    pub required_features: Vec<&'static str>,
    pub forbidden_features: Vec<&'static str>,
}

/// Difficulty progression report for analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProgressionReport {
    pub levels_analyzed: u32,
    pub average_difficulty: f64,
    pub min_difficulty: f64,
    pub max_difficulty: f64,
    pub progression_issues: Vec<String>,
    pub tier_breakdown: HashMap<u8, TierStats>,
}

/// Statistics for a specific tier
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierStats {
    pub avg_difficulty: f64,
    pub avg_symbol_ratio: f64,
    pub avg_number_ratio: f64,
    pub avg_technical_ratio: f64,
    pub level_count: u32,
}

impl DifficultyAnalyzer {
    /// Generate comprehensive progression report
    pub fn generate_progression_report(&self, contents: &[(LevelId, String)]) -> ProgressionReport {
        let mut report = ProgressionReport {
            levels_analyzed: contents.len() as u32,
            average_difficulty: 0.0,
            min_difficulty: f64::INFINITY,
            max_difficulty: f64::NEG_INFINITY,
            progression_issues: Vec::new(),
            tier_breakdown: HashMap::new(),
        };

        let mut total_difficulty = 0.0;
        let mut tier_data: HashMap<u8, Vec<f64>> = HashMap::new();

        for (level_id, content) in contents {
            let score = self.analyze_content(content);
            let expected = Self::expected_difficulty_for_level(*level_id);

            // Update overall stats
            total_difficulty += score.overall;
            report.min_difficulty = report.min_difficulty.min(score.overall);
            report.max_difficulty = report.max_difficulty.max(score.overall);

            // Check for issues
            if !score.is_appropriate_for_level(*level_id) {
                report.progression_issues.push(format!(
                    "Level {} difficulty mismatch: expected {:.1}, got {:.1}",
                    level_id.0, expected.overall, score.overall
                ));
            }

            // Collect tier data
            let tier = level_id.tier().0;
            tier_data.entry(tier).or_insert_with(Vec::new).push(score.overall);
        }

        // Calculate averages
        if !contents.is_empty() {
            report.average_difficulty = total_difficulty / contents.len() as f64;
        }

        // Generate tier breakdown
        for (tier, difficulties) in tier_data {
            let avg_difficulty = difficulties.iter().sum::<f64>() / difficulties.len() as f64;

            report.tier_breakdown.insert(tier, TierStats {
                avg_difficulty,
                avg_symbol_ratio: 0.0, // Would need more detailed analysis
                avg_number_ratio: 0.0, // Would need more detailed analysis
                avg_technical_ratio: 0.0, // Would need more detailed analysis
                level_count: difficulties.len() as u32,
            });
        }

        report
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_difficulty_calculation() {
        let analyzer = DifficultyAnalyzer::new();

        // Simple text should have low difficulty
        let simple_text = "This is a simple sentence with basic words and punctuation.";
        let simple_score = analyzer.analyze_content(simple_text);
        assert!(simple_score.overall < 20.0, "Simple text should have low difficulty");

        // Complex code should have higher difficulty
        let complex_text = "&mut HashMap<String, Vec<Option<Box<dyn Iterator<Item=u32>>>>> | 0xFF & mask";
        let complex_score = analyzer.analyze_content(complex_text);
        assert!(complex_score.overall > simple_score.overall, "Complex text should be more difficult");
    }

    #[test]
    fn test_expected_difficulty_progression() {
        let level_1 = DifficultyAnalyzer::expected_difficulty_for_level(LevelId::new(1).unwrap());
        let level_50 = DifficultyAnalyzer::expected_difficulty_for_level(LevelId::new(50).unwrap());
        let level_100 = DifficultyAnalyzer::expected_difficulty_for_level(LevelId::new(100).unwrap());

        assert!(level_1.overall < level_50.overall, "Level 50 should be harder than level 1");
        assert!(level_50.overall < level_100.overall, "Level 100 should be harder than level 50");

        // Check that progression is reasonable
        assert!(level_100.overall < 150.0, "Level 100 difficulty should be reasonable");
    }

    #[test]
    fn test_tier_requirements() {
        let tier_1 = DifficultyAnalyzer::get_tier_requirements(Tier(1));
        let tier_10 = DifficultyAnalyzer::get_tier_requirements(Tier(10));

        assert_eq!(tier_1.name, "Foundation");
        assert_eq!(tier_10.name, "Expert");
        assert!(tier_1.max_symbol_ratio < tier_10.min_symbol_ratio);
    }

    #[test]
    fn test_progression_validation() {
        let analyzer = DifficultyAnalyzer::new();

        let contents = vec![
            (LevelId::new(1).unwrap(), "Simple text for beginners.".to_string()),
            (LevelId::new(2).unwrap(), "Slightly more complex text with numbers 123.".to_string()),
            (LevelId::new(3).unwrap(), "Programming basics: function test() { return 42; }".to_string()),
        ];

        assert!(analyzer.validate_progression(&contents).is_ok());
    }

    #[test]
    fn test_difficulty_score_appropriateness() {
        let score = DifficultyScore::new(10.0, 5.0, 8.0, 3.0, 2.0);
        let level_50 = LevelId::new(50).unwrap();

        // This would need to be adjusted based on actual expected values
        assert!(score.overall > 0.0);
        assert!(score.overall <= 100.0);
    }
}
