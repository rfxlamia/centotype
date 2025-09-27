//! Level progression system with 100 levels across 10 tiers
use crate::types::*;
use indexmap::IndexMap;
use once_cell::sync::Lazy;
use std::collections::HashMap;

/// Level manager for 100 progressive difficulty levels
pub struct Level {
    level_definitions: &'static LevelDefinitions,
    unlock_cache: HashMap<LevelId, bool>,
}

impl Level {
    pub fn new() -> Self {
        Self {
            level_definitions: &LEVEL_DEFINITIONS,
            unlock_cache: HashMap::new(),
        }
    }

    /// Get level definition by ID
    pub fn get_level(&self, level_id: LevelId) -> Result<&LevelDefinition> {
        self.level_definitions
            .levels
            .get(&level_id)
            .ok_or_else(|| CentotypeError::State(format!("Level {} not found", level_id.0)))
    }

    /// Check if level is unlocked for a user
    pub fn is_unlocked(&mut self, level_id: LevelId, progress: &UserProgress) -> bool {
        // Check cache first
        if let Some(&cached) = self.unlock_cache.get(&level_id) {
            return cached;
        }

        let is_unlocked = self.calculate_unlock_status(level_id, progress);
        self.unlock_cache.insert(level_id, is_unlocked);
        is_unlocked
    }

    /// Get next recommended level based on user progress
    pub fn get_next_level(&mut self, progress: &UserProgress) -> Result<LevelId> {
        // Start with Level 1 if no progress
        if progress.best_results.is_empty() {
            return LevelId::new(1);
        }

        // Find the highest completed level with good performance
        let mut next_level = 1u8;

        for level_id in 1..=100 {
            let level_id = LevelId::new(level_id)?;

            if let Some(result) = progress.best_results.get(&level_id) {
                // Check if level was completed with sufficient grade
                if result.grade >= Grade::min_for_progression() {
                    next_level = level_id.0 + 1;
                } else {
                    // Current level needs improvement
                    return Ok(level_id);
                }
            } else {
                // First unplayed level
                break;
            }
        }

        // Ensure we don't exceed level 100
        LevelId::new(next_level.min(100))
    }

    /// Check if Level 100 mastery criteria is met
    pub fn is_level_100_mastered(&self, result: &SessionResult) -> bool {
        if let TrainingMode::Arcade { level } = result.mode {
            if level.0 == 100 {
                return self.meets_mastery_criteria(&result.metrics);
            }
        }
        false
    }

    /// Get tier progression information
    pub fn get_tier_progress(&self, progress: &UserProgress) -> TierProgress {
        let mut tier_stats = HashMap::new();

        // Initialize all tiers
        for tier_num in 1..=10 {
            tier_stats.insert(
                Tier(tier_num),
                TierStats {
                    completed_levels: 0,
                    total_levels: 10,
                    average_grade: None,
                    is_unlocked: tier_num == 1, // Only tier 1 is unlocked initially
                },
            );
        }

        // Calculate stats based on user progress
        for (level_id, result) in &progress.best_results {
            let tier = level_id.tier();
            let tier_stat = tier_stats.get_mut(&tier).unwrap();

            if result.grade >= Grade::min_for_progression() {
                tier_stat.completed_levels += 1;
            }
        }

        // Calculate average grades and unlock status
        for tier_num in 1..=10 {
            let tier = Tier(tier_num);

            // Calculate average grade for completed levels in tier
            let tier_results: Vec<&SessionResult> = progress
                .best_results
                .iter()
                .filter(|(level_id, result)| {
                    level_id.tier() == tier && result.grade >= Grade::min_for_progression()
                })
                .map(|(_, result)| result)
                .collect();

            if !tier_results.is_empty() {
                let grade_sum: u8 = tier_results
                    .iter()
                    .map(|r| match r.grade {
                        Grade::S => 5,
                        Grade::A => 4,
                        Grade::B => 3,
                        Grade::C => 2,
                        Grade::D => 1,
                    })
                    .sum();

                let avg_grade_val = grade_sum as f64 / tier_results.len() as f64;
                let average_grade = Some(match avg_grade_val {
                    x if x >= 4.5 => Grade::S,
                    x if x >= 3.5 => Grade::A,
                    x if x >= 2.5 => Grade::B,
                    x if x >= 1.5 => Grade::C,
                    _ => Grade::D,
                });

                // Update the tier stat
                if let Some(tier_stat) = tier_stats.get_mut(&tier) {
                    tier_stat.average_grade = average_grade;
                }
            }

            // Unlock next tier if current tier is mostly completed
            if tier_num > 1 {
                let prev_tier = Tier(tier_num - 1);
                let prev_completed = tier_stats
                    .get(&prev_tier)
                    .map(|stat| stat.completed_levels)
                    .unwrap_or(0);

                // Unlock if previous tier has at least 7/10 levels completed with C+ grade
                if prev_completed >= 7 {
                    if let Some(tier_stat) = tier_stats.get_mut(&tier) {
                        tier_stat.is_unlocked = true;
                    }
                }
            }
        }

        TierProgress { tier_stats }
    }

    /// Get level suggestions based on weak areas
    pub fn get_practice_suggestions(&self, progress: &UserProgress) -> Vec<PracticeSuggestion> {
        let mut suggestions = Vec::new();

        // Analyze performance patterns
        for (level_id, result) in &progress.best_results {
            if result.grade < Grade::B {
                // Suggest improvement for poor performance
                let suggestion = PracticeSuggestion {
                    level: *level_id,
                    reason: SuggestionReason::LowGrade(result.grade),
                    target_metrics: self.calculate_target_metrics(*level_id),
                };
                suggestions.push(suggestion);
            } else if result.metrics.accuracy < 95.0 {
                // Suggest accuracy improvement
                let suggestion = PracticeSuggestion {
                    level: *level_id,
                    reason: SuggestionReason::LowAccuracy(result.metrics.accuracy),
                    target_metrics: self.calculate_target_metrics(*level_id),
                };
                suggestions.push(suggestion);
            } else if result.metrics.effective_wpm < self.get_tier_target_wpm(level_id.tier()) {
                // Suggest speed improvement
                let suggestion = PracticeSuggestion {
                    level: *level_id,
                    reason: SuggestionReason::LowSpeed(result.metrics.effective_wpm),
                    target_metrics: self.calculate_target_metrics(*level_id),
                };
                suggestions.push(suggestion);
            }
        }

        // Limit to top 5 suggestions
        suggestions.truncate(5);
        suggestions
    }

    // Private helper methods

    fn calculate_unlock_status(&self, level_id: LevelId, progress: &UserProgress) -> bool {
        // Level 1 is always unlocked
        if level_id.0 == 1 {
            return true;
        }

        // Check if previous level is completed with sufficient grade
        if let Ok(prev_level) = LevelId::new(level_id.0 - 1) {
            if let Some(prev_result) = progress.best_results.get(&prev_level) {
                return prev_result.grade >= Grade::min_for_progression();
            }
        }

        false
    }

    fn meets_mastery_criteria(&self, metrics: &FinalMetrics) -> bool {
        // Level 100 mastery criteria: 130 WPM effective, 99.5% accuracy, ≤3 error severity
        metrics.effective_wpm >= 130.0
            && metrics.accuracy >= 99.5
            && metrics.errors.severity_score() <= 3.0
    }

    fn get_tier_target_wpm(&self, tier: Tier) -> f64 {
        // Progressive WPM targets by tier
        match tier.0 {
            1 => 20.0,   // Tier 1: 20 WPM
            2 => 30.0,   // Tier 2: 30 WPM
            3 => 40.0,   // Tier 3: 40 WPM
            4 => 50.0,   // Tier 4: 50 WPM
            5 => 60.0,   // Tier 5: 60 WPM
            6 => 70.0,   // Tier 6: 70 WPM
            7 => 80.0,   // Tier 7: 80 WPM
            8 => 90.0,   // Tier 8: 90 WPM
            9 => 110.0,  // Tier 9: 110 WPM
            10 => 130.0, // Tier 10: 130 WPM (Level 100 mastery)
            _ => 20.0,   // Default
        }
    }

    fn calculate_target_metrics(&self, level_id: LevelId) -> TargetMetrics {
        let tier = level_id.tier();
        let base_wpm = self.get_tier_target_wpm(tier);

        // Adjust for specific level within tier
        let level_in_tier = ((level_id.0 - 1) % 10) + 1;
        let wpm_progression = (level_in_tier as f64 - 1.0) * 1.0; // +1 WPM per level

        TargetMetrics {
            target_wpm: base_wpm + wpm_progression,
            target_accuracy: if level_id.0 <= 90 { 95.0 } else { 99.0 }, // Higher accuracy for final levels
            target_consistency: 70.0 + (tier.0 as f64 * 3.0), // Higher consistency for higher tiers
        }
    }
}

impl Default for Level {
    fn default() -> Self {
        Self::new()
    }
}

/// Complete level definitions for all 100 levels
#[derive(Debug)]
pub struct LevelDefinitions {
    pub levels: IndexMap<LevelId, LevelDefinition>,
}

/// Definition for a single level
#[derive(Debug, Clone)]
pub struct LevelDefinition {
    pub id: LevelId,
    pub tier: Tier,
    pub name: String,
    pub description: String,
    pub character_focus: CharacterFocus,
    pub difficulty_modifiers: DifficultyModifiers,
    pub min_grade_to_progress: Grade,
    pub estimated_duration_minutes: u8,
}

/// Character focus for level content
#[derive(Debug, Clone)]
pub enum CharacterFocus {
    Letters { lowercase: bool, uppercase: bool },
    Numbers,
    Punctuation(Vec<char>),
    Symbols(Vec<char>),
    Mixed { ratios: CharacterRatios },
}

/// Ratios for mixed character content
#[derive(Debug, Clone)]
pub struct CharacterRatios {
    pub letters: f64,
    pub numbers: f64,
    pub punctuation: f64,
    pub symbols: f64,
}

/// Difficulty modifiers for level content generation
#[derive(Debug, Clone)]
pub struct DifficultyModifiers {
    pub word_length_avg: f64,
    pub rare_word_ratio: f64,
    pub capitalization_ratio: f64,
    pub number_density: f64,
    pub symbol_complexity: f64,
}

/// Progress tracking across tiers
#[derive(Debug, Clone)]
pub struct TierProgress {
    pub tier_stats: HashMap<Tier, TierStats>,
}

/// Statistics for a single tier
#[derive(Debug, Clone)]
pub struct TierStats {
    pub completed_levels: u8,
    pub total_levels: u8,
    pub average_grade: Option<Grade>,
    pub is_unlocked: bool,
}

/// Practice suggestion for improvement
#[derive(Debug, Clone)]
pub struct PracticeSuggestion {
    pub level: LevelId,
    pub reason: SuggestionReason,
    pub target_metrics: TargetMetrics,
}

/// Reason for practice suggestion
#[derive(Debug, Clone)]
pub enum SuggestionReason {
    LowGrade(Grade),
    LowAccuracy(f64),
    LowSpeed(f64),
    InconsistentTiming,
}

/// Target metrics for level completion
#[derive(Debug, Clone)]
pub struct TargetMetrics {
    pub target_wpm: f64,
    pub target_accuracy: f64,
    pub target_consistency: f64,
}

/// Static level definitions - loaded once
static LEVEL_DEFINITIONS: Lazy<LevelDefinitions> = Lazy::new(|| {
    let mut levels = IndexMap::new();

    // Generate all 100 levels across 10 tiers
    for level_num in 1..=100 {
        let level_id = LevelId::new(level_num).unwrap();
        let tier = level_id.tier();
        let level_in_tier = ((level_num - 1) % 10) + 1;

        let definition = match tier.0 {
            1 => create_tier_1_level(level_id, level_in_tier),
            2 => create_tier_2_level(level_id, level_in_tier),
            3 => create_tier_3_level(level_id, level_in_tier),
            4 => create_tier_4_level(level_id, level_in_tier),
            5 => create_tier_5_level(level_id, level_in_tier),
            6 => create_tier_6_level(level_id, level_in_tier),
            7 => create_tier_7_level(level_id, level_in_tier),
            8 => create_tier_8_level(level_id, level_in_tier),
            9 => create_tier_9_level(level_id, level_in_tier),
            10 => create_tier_10_level(level_id, level_in_tier),
            _ => unreachable!(),
        };

        levels.insert(level_id, definition);
    }

    LevelDefinitions { levels }
});

// Level generation functions for each tier
fn create_tier_1_level(id: LevelId, level_in_tier: u8) -> LevelDefinition {
    LevelDefinition {
        id,
        tier: Tier(1),
        name: format!("Basic Letters {}", level_in_tier),
        description: format!(
            "Learn home row and basic letter combinations - Level {}",
            level_in_tier
        ),
        character_focus: CharacterFocus::Letters {
            lowercase: true,
            uppercase: level_in_tier > 5,
        },
        difficulty_modifiers: DifficultyModifiers {
            word_length_avg: 3.0 + (level_in_tier as f64 * 0.3),
            rare_word_ratio: 0.1,
            capitalization_ratio: if level_in_tier > 5 { 0.1 } else { 0.0 },
            number_density: 0.0,
            symbol_complexity: 0.0,
        },
        min_grade_to_progress: Grade::C,
        estimated_duration_minutes: 3,
    }
}

fn create_tier_2_level(id: LevelId, level_in_tier: u8) -> LevelDefinition {
    LevelDefinition {
        id,
        tier: Tier(2),
        name: format!("Letter Mastery {}", level_in_tier),
        description: format!(
            "Master all letters with mixed case - Level {}",
            level_in_tier
        ),
        character_focus: CharacterFocus::Letters {
            lowercase: true,
            uppercase: true,
        },
        difficulty_modifiers: DifficultyModifiers {
            word_length_avg: 4.0 + (level_in_tier as f64 * 0.4),
            rare_word_ratio: 0.15 + (level_in_tier as f64 * 0.02),
            capitalization_ratio: 0.2 + (level_in_tier as f64 * 0.03),
            number_density: 0.0,
            symbol_complexity: 0.0,
        },
        min_grade_to_progress: Grade::C,
        estimated_duration_minutes: 4,
    }
}

fn create_tier_3_level(id: LevelId, level_in_tier: u8) -> LevelDefinition {
    LevelDefinition {
        id,
        tier: Tier(3),
        name: format!("Numbers Introduction {}", level_in_tier),
        description: format!(
            "Learn number typing and basic combinations - Level {}",
            level_in_tier
        ),
        character_focus: CharacterFocus::Mixed {
            ratios: CharacterRatios {
                letters: 0.7,
                numbers: 0.2 + (level_in_tier as f64 * 0.01),
                punctuation: 0.1,
                symbols: 0.0,
            },
        },
        difficulty_modifiers: DifficultyModifiers {
            word_length_avg: 4.5 + (level_in_tier as f64 * 0.3),
            rare_word_ratio: 0.2,
            capitalization_ratio: 0.25,
            number_density: 0.15 + (level_in_tier as f64 * 0.02),
            symbol_complexity: 0.0,
        },
        min_grade_to_progress: Grade::C,
        estimated_duration_minutes: 5,
    }
}

fn create_tier_4_level(id: LevelId, level_in_tier: u8) -> LevelDefinition {
    LevelDefinition {
        id,
        tier: Tier(4),
        name: format!("Basic Punctuation {}", level_in_tier),
        description: format!("Master common punctuation marks - Level {}", level_in_tier),
        character_focus: CharacterFocus::Mixed {
            ratios: CharacterRatios {
                letters: 0.6,
                numbers: 0.2,
                punctuation: 0.15 + (level_in_tier as f64 * 0.01),
                symbols: 0.05,
            },
        },
        difficulty_modifiers: DifficultyModifiers {
            word_length_avg: 5.0 + (level_in_tier as f64 * 0.3),
            rare_word_ratio: 0.25,
            capitalization_ratio: 0.3,
            number_density: 0.2,
            symbol_complexity: 0.1 + (level_in_tier as f64 * 0.02),
        },
        min_grade_to_progress: Grade::C,
        estimated_duration_minutes: 6,
    }
}

fn create_tier_5_level(id: LevelId, level_in_tier: u8) -> LevelDefinition {
    LevelDefinition {
        id,
        tier: Tier(5),
        name: format!("Advanced Punctuation {}", level_in_tier),
        description: format!(
            "Complex punctuation and formatting - Level {}",
            level_in_tier
        ),
        character_focus: CharacterFocus::Punctuation(vec![
            '.', ',', ';', ':', '!', '?', '"', '\'', '(', ')', '[', ']',
        ]),
        difficulty_modifiers: DifficultyModifiers {
            word_length_avg: 5.5 + (level_in_tier as f64 * 0.3),
            rare_word_ratio: 0.3,
            capitalization_ratio: 0.35,
            number_density: 0.25,
            symbol_complexity: 0.2 + (level_in_tier as f64 * 0.03),
        },
        min_grade_to_progress: Grade::C,
        estimated_duration_minutes: 7,
    }
}

fn create_tier_6_level(id: LevelId, level_in_tier: u8) -> LevelDefinition {
    LevelDefinition {
        id,
        tier: Tier(6),
        name: format!("Symbol Introduction {}", level_in_tier),
        description: format!("Learn basic programming symbols - Level {}", level_in_tier),
        character_focus: CharacterFocus::Symbols(vec![
            '@', '#', '$', '%', '^', '&', '*', '+', '=', '-', '_',
        ]),
        difficulty_modifiers: DifficultyModifiers {
            word_length_avg: 6.0 + (level_in_tier as f64 * 0.3),
            rare_word_ratio: 0.35,
            capitalization_ratio: 0.4,
            number_density: 0.3,
            symbol_complexity: 0.3 + (level_in_tier as f64 * 0.04),
        },
        min_grade_to_progress: Grade::B,
        estimated_duration_minutes: 8,
    }
}

fn create_tier_7_level(id: LevelId, level_in_tier: u8) -> LevelDefinition {
    LevelDefinition {
        id,
        tier: Tier(7),
        name: format!("Advanced Symbols {}", level_in_tier),
        description: format!(
            "Master complex symbol combinations - Level {}",
            level_in_tier
        ),
        character_focus: CharacterFocus::Symbols(vec![
            '<', '>', '/', '\\', '|', '`', '~', '{', '}', '[', ']',
        ]),
        difficulty_modifiers: DifficultyModifiers {
            word_length_avg: 6.5 + (level_in_tier as f64 * 0.3),
            rare_word_ratio: 0.4,
            capitalization_ratio: 0.45,
            number_density: 0.35,
            symbol_complexity: 0.4 + (level_in_tier as f64 * 0.05),
        },
        min_grade_to_progress: Grade::B,
        estimated_duration_minutes: 9,
    }
}

fn create_tier_8_level(id: LevelId, level_in_tier: u8) -> LevelDefinition {
    LevelDefinition {
        id,
        tier: Tier(8),
        name: format!("Code Patterns {}", level_in_tier),
        description: format!(
            "Programming language patterns and syntax - Level {}",
            level_in_tier
        ),
        character_focus: CharacterFocus::Mixed {
            ratios: CharacterRatios {
                letters: 0.4,
                numbers: 0.25,
                punctuation: 0.15,
                symbols: 0.2,
            },
        },
        difficulty_modifiers: DifficultyModifiers {
            word_length_avg: 7.0 + (level_in_tier as f64 * 0.3),
            rare_word_ratio: 0.45,
            capitalization_ratio: 0.5,
            number_density: 0.4,
            symbol_complexity: 0.5 + (level_in_tier as f64 * 0.05),
        },
        min_grade_to_progress: Grade::B,
        estimated_duration_minutes: 10,
    }
}

fn create_tier_9_level(id: LevelId, level_in_tier: u8) -> LevelDefinition {
    LevelDefinition {
        id,
        tier: Tier(9),
        name: format!("Expert Combinations {}", level_in_tier),
        description: format!(
            "Complex multi-character sequences - Level {}",
            level_in_tier
        ),
        character_focus: CharacterFocus::Mixed {
            ratios: CharacterRatios {
                letters: 0.35,
                numbers: 0.3,
                punctuation: 0.15,
                symbols: 0.2,
            },
        },
        difficulty_modifiers: DifficultyModifiers {
            word_length_avg: 8.0 + (level_in_tier as f64 * 0.3),
            rare_word_ratio: 0.5,
            capitalization_ratio: 0.55,
            number_density: 0.45,
            symbol_complexity: 0.6 + (level_in_tier as f64 * 0.04),
        },
        min_grade_to_progress: Grade::A,
        estimated_duration_minutes: 12,
    }
}

fn create_tier_10_level(id: LevelId, level_in_tier: u8) -> LevelDefinition {
    let is_final_level = level_in_tier == 10; // Level 100

    LevelDefinition {
        id,
        tier: Tier(10),
        name: if is_final_level {
            "Ultimate Mastery".to_string()
        } else {
            format!("Master Level {}", level_in_tier)
        },
        description: if is_final_level {
            "The ultimate typing challenge - Level 100 Mastery".to_string()
        } else {
            format!("Elite-level typing precision and speed - Level {}", id.0)
        },
        character_focus: CharacterFocus::Mixed {
            ratios: CharacterRatios {
                letters: 0.3,
                numbers: 0.3,
                punctuation: 0.2,
                symbols: 0.2,
            },
        },
        difficulty_modifiers: DifficultyModifiers {
            word_length_avg: 9.0 + (level_in_tier as f64 * 0.4),
            rare_word_ratio: 0.6 + (level_in_tier as f64 * 0.04),
            capitalization_ratio: 0.6 + (level_in_tier as f64 * 0.04),
            number_density: 0.5 + (level_in_tier as f64 * 0.02),
            symbol_complexity: 0.7 + (level_in_tier as f64 * 0.03),
        },
        min_grade_to_progress: if is_final_level { Grade::S } else { Grade::A },
        estimated_duration_minutes: if is_final_level { 20 } else { 15 },
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_level_creation() {
        let level_manager = Level::new();

        // Test level 1
        let level_1 = level_manager.get_level(LevelId::new(1).unwrap()).unwrap();
        assert_eq!(level_1.tier, Tier(1));
        assert_eq!(level_1.min_grade_to_progress, Grade::C);

        // Test level 100
        let level_100 = level_manager.get_level(LevelId::new(100).unwrap()).unwrap();
        assert_eq!(level_100.tier, Tier(10));
        assert_eq!(level_100.min_grade_to_progress, Grade::S);
    }

    #[test]
    fn test_unlock_logic() {
        let mut level_manager = Level::new();
        let empty_progress = UserProgress::default();

        // Level 1 should always be unlocked
        assert!(level_manager.is_unlocked(LevelId::new(1).unwrap(), &empty_progress));

        // Level 2 should not be unlocked without progress
        assert!(!level_manager.is_unlocked(LevelId::new(2).unwrap(), &empty_progress));
    }

    #[test]
    fn test_next_level_recommendation() {
        let mut level_manager = Level::new();
        let empty_progress = UserProgress::default();

        // Should recommend Level 1 for new users
        let next = level_manager.get_next_level(&empty_progress).unwrap();
        assert_eq!(next, LevelId::new(1).unwrap());
    }

    #[test]
    fn test_level_100_mastery() {
        let level_manager = Level::new();

        let mastery_metrics = FinalMetrics {
            raw_wpm: 135.0,
            effective_wpm: 130.0,
            accuracy: 99.5,
            consistency: 95.0,
            longest_streak: 500,
            errors: ErrorStats {
                substitution: 1,
                insertion: 0,
                deletion: 0,
                transposition: 0,
                backspace_count: 2,
                idle_events: 0,
            },
            latency_p99: Duration::from_millis(15),
        };

        assert!(level_manager.meets_mastery_criteria(&mastery_metrics));

        let insufficient_metrics = FinalMetrics {
            raw_wpm: 100.0,
            effective_wpm: 90.0,
            accuracy: 95.0,
            consistency: 80.0,
            longest_streak: 200,
            errors: ErrorStats::default(),
            latency_p99: Duration::from_millis(30),
        };

        assert!(!level_manager.meets_mastery_criteria(&insufficient_metrics));
    }

    #[test]
    fn test_tier_progression() {
        let level_manager = Level::new();

        // Test that all tiers have correct WPM targets
        assert_eq!(level_manager.get_tier_target_wpm(Tier(1)), 20.0);
        assert_eq!(level_manager.get_tier_target_wpm(Tier(5)), 60.0);
        assert_eq!(level_manager.get_tier_target_wpm(Tier(10)), 130.0);
    }
}
