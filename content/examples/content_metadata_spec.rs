use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Content metadata specification and validation rules for Centotype
///
/// This module defines the complete metadata schema for typing content,
/// validation rules ensuring quality and progression, and content management
/// utilities for the Centotype typing trainer system.

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentMetadata {
    pub version: String,
    pub generated_at: String,
    pub total_levels: u32,
    pub tier_definitions: HashMap<String, TierDefinition>,
    pub validation_rules: ValidationRules,
    pub content_schema: ContentSchema,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierDefinition {
    pub name: String,
    pub level_range: (u32, u32),
    pub description: String,
    pub character_sets: Vec<String>,
    pub progression_type: ProgressionType,
    pub learning_objectives: Vec<String>,
    pub difficulty_curve: DifficultyParameters,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ProgressionType {
    Linear,
    Exponential,
    Logarithmic,
    Custom(Vec<f64>),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DifficultyParameters {
    pub base_score: f64,
    pub tier_multiplier: f64,
    pub max_level_increment: f64,
    pub complexity_factors: Vec<ComplexityWeight>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplexityWeight {
    pub factor_type: String,
    pub weight: f64,
    pub tier_scaling: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationRules {
    pub content_quality: ContentQualityRules,
    pub difficulty_progression: DifficultyProgressionRules,
    pub performance_constraints: PerformanceConstraints,
    pub accessibility_requirements: AccessibilityRequirements,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentQualityRules {
    pub min_unique_characters: u32,
    pub max_repetition_ratio: f64,
    pub min_readability_score: f64,
    pub max_obscurity_ratio: f64,
    pub profanity_filtering: bool,
    pub cultural_neutrality: bool,
    pub professional_appropriateness: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DifficultyProgressionRules {
    pub max_difficulty_jump: f64,
    pub min_difficulty_increment: f64,
    pub smoothness_tolerance: f64,
    pub tier_transition_validation: bool,
    pub regression_prevention: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceConstraints {
    pub max_content_size_bytes: usize,
    pub max_loading_time_ms: u32,
    pub max_validation_time_ms: u32,
    pub memory_limit_mb: u32,
    pub cache_efficiency_target: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AccessibilityRequirements {
    pub contrast_ratio_minimum: f64,
    pub font_size_scalability: bool,
    pub screen_reader_compatibility: bool,
    pub keyboard_navigation_only: bool,
    pub color_blind_safe_indicators: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentSchema {
    pub text_entry: TextEntrySchema,
    pub metadata_fields: MetadataFields,
    pub language_support: LanguageSupport,
    pub content_categories: Vec<ContentCategory>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextEntrySchema {
    pub required_fields: Vec<String>,
    pub optional_fields: Vec<String>,
    pub content_constraints: ContentConstraints,
    pub encoding_requirements: EncodingRequirements,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentConstraints {
    pub min_length_chars: u32,
    pub max_length_chars: u32,
    pub allowed_unicode_ranges: Vec<UnicodeRange>,
    pub forbidden_patterns: Vec<String>,
    pub required_character_distribution: CharacterDistribution,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnicodeRange {
    pub name: String,
    pub start: u32,
    pub end: u32,
    pub description: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CharacterDistribution {
    pub letters_min_ratio: f64,
    pub numbers_max_ratio: f64,
    pub punctuation_max_ratio: f64,
    pub symbols_max_ratio: f64,
    pub whitespace_target_ratio: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EncodingRequirements {
    pub encoding: String,
    pub normalization_form: String,
    pub byte_order_mark: bool,
    pub line_ending_style: LineEndingStyle,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum LineEndingStyle {
    Unix,    // \n
    Windows, // \r\n
    Mac,     // \r
    Auto,    // Platform-specific
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetadataFields {
    pub core_metrics: Vec<MetricDefinition>,
    pub difficulty_indicators: Vec<DifficultyIndicator>,
    pub learning_analytics: Vec<AnalyticsField>,
    pub content_classification: ClassificationSchema,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricDefinition {
    pub name: String,
    pub data_type: MetricDataType,
    pub calculation_method: String,
    pub validation_range: (f64, f64),
    pub precision: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MetricDataType {
    Float,
    Integer,
    Percentage,
    Duration,
    Count,
    Ratio,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DifficultyIndicator {
    pub indicator_name: String,
    pub weight_in_total_score: f64,
    pub calculation_formula: String,
    pub tier_specific_adjustments: HashMap<String, f64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AnalyticsField {
    pub field_name: String,
    pub purpose: AnalyticsPurpose,
    pub data_retention_days: u32,
    pub privacy_level: PrivacyLevel,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AnalyticsPurpose {
    PerformanceTracking,
    DifficultyAdjustment,
    UserProgress,
    ContentOptimization,
    ErrorAnalysis,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PrivacyLevel {
    Anonymous,
    Aggregated,
    PersonalIdentifiable,
    Sensitive,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ClassificationSchema {
    pub primary_categories: Vec<String>,
    pub secondary_tags: Vec<String>,
    pub skill_focus_areas: Vec<SkillArea>,
    pub content_themes: Vec<ContentTheme>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkillArea {
    pub name: String,
    pub description: String,
    pub associated_tiers: Vec<u32>,
    pub progression_indicators: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentTheme {
    pub theme_name: String,
    pub description: String,
    pub target_audience: TargetAudience,
    pub vocabulary_complexity: VocabularyComplexity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TargetAudience {
    General,
    Technical,
    Academic,
    Professional,
    Creative,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VocabularyComplexity {
    Basic,
    Intermediate,
    Advanced,
    Expert,
    Specialized,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageSupport {
    pub primary_languages: Vec<LanguageConfig>,
    pub secondary_languages: Vec<LanguageConfig>,
    pub mixed_content_rules: MixedContentRules,
    pub localization_requirements: LocalizationRequirements,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LanguageConfig {
    pub language_code: String,
    pub language_name: String,
    pub character_set: String,
    pub typing_difficulty_modifier: f64,
    pub frequency_patterns: FrequencyPatterns,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FrequencyPatterns {
    pub letter_frequencies: HashMap<char, f64>,
    pub bigram_frequencies: HashMap<String, f64>,
    pub common_words: Vec<String>,
    pub difficult_combinations: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MixedContentRules {
    pub max_language_switches_per_100_chars: u32,
    pub min_segment_length_chars: u32,
    pub transition_smoothness_requirement: f64,
    pub context_preservation_rules: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LocalizationRequirements {
    pub number_format_localization: bool,
    pub date_format_localization: bool,
    pub currency_symbol_support: bool,
    pub cultural_context_adaptation: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentCategory {
    pub category_name: String,
    pub description: String,
    pub tier_availability: Vec<u32>,
    pub content_characteristics: ContentCharacteristics,
    pub generation_parameters: GenerationParameters,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentCharacteristics {
    pub average_word_length: f64,
    pub sentence_complexity: SentenceComplexity,
    pub technical_terminology_ratio: f64,
    pub punctuation_density: f64,
    pub symbol_usage_patterns: Vec<SymbolPattern>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SentenceComplexity {
    Simple,          // Single clause
    Compound,        // Multiple independent clauses
    Complex,         // Dependent clauses
    CompoundComplex, // Both compound and complex elements
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SymbolPattern {
    pub symbol_class: String,
    pub usage_frequency: f64,
    pub context_rules: Vec<String>,
    pub difficulty_contribution: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationParameters {
    pub seed_algorithms: Vec<SeedAlgorithm>,
    pub content_templates: Vec<ContentTemplate>,
    pub variability_factors: VariabilityFactors,
    pub quality_assurance: QualityAssurance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeedAlgorithm {
    pub algorithm_name: String,
    pub deterministic: bool,
    pub parameter_ranges: HashMap<String, (f64, f64)>,
    pub output_consistency_target: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentTemplate {
    pub template_id: String,
    pub pattern_description: String,
    pub variable_slots: Vec<VariableSlot>,
    pub difficulty_scaling: DifficultyScaling,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableSlot {
    pub slot_name: String,
    pub content_type: SlotContentType,
    pub generation_rules: Vec<GenerationRule>,
    pub validation_constraints: Vec<ValidationConstraint>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SlotContentType {
    Word,
    Number,
    Symbol,
    Phrase,
    TechnicalTerm,
    Punctuation,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationRule {
    pub rule_type: RuleType,
    pub parameters: HashMap<String, String>,
    pub priority: u32,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RuleType {
    LengthConstraint,
    CharacterClass,
    FrequencyBased,
    ContextDependent,
    DifficultyScaled,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidationConstraint {
    pub constraint_type: ConstraintType,
    pub threshold_value: f64,
    pub error_severity: ErrorSeverity,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ConstraintType {
    MinLength,
    MaxLength,
    CharacterDistribution,
    DifficultyRange,
    ReadabilityScore,
    UniquenessRatio,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorSeverity {
    Warning,
    Error,
    Critical,
    Blocking,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DifficultyScaling {
    pub base_difficulty: f64,
    pub scaling_function: ScalingFunction,
    pub tier_adjustments: HashMap<u32, f64>,
    pub level_progressions: Vec<LevelProgression>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ScalingFunction {
    Linear(f64),          // slope
    Exponential(f64),     // base
    Logarithmic(f64),     // coefficient
    Polynomial(Vec<f64>), // coefficients
    Custom(String),       // formula
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LevelProgression {
    pub level: u32,
    pub target_difficulty: f64,
    pub expected_wpm: u32,
    pub mastery_criteria: MasteryCriteria,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MasteryCriteria {
    pub min_accuracy: f64,
    pub max_error_severity: f64,
    pub consistency_requirement: f64,
    pub time_constraints: TimeConstraints,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TimeConstraints {
    pub max_completion_time_seconds: u32,
    pub min_sustained_wpm: u32,
    pub fatigue_resistance_factor: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariabilityFactors {
    pub content_randomization: f64,
    pub difficulty_variance: f64,
    pub pattern_prevention: PatternPrevention,
    pub freshness_maintenance: FreshnessMaintenance,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PatternPrevention {
    pub max_repetition_frequency: f64,
    pub pattern_detection_window: u32,
    pub variation_enforcement: bool,
    pub anti_memorization_measures: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FreshnessMaintenance {
    pub content_rotation_period_days: u32,
    pub new_content_introduction_rate: f64,
    pub obsolete_content_retirement: bool,
    pub seasonal_content_updates: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualityAssurance {
    pub automated_validation: AutomatedValidation,
    pub human_review_requirements: HumanReviewRequirements,
    pub continuous_monitoring: ContinuousMonitoring,
    pub feedback_integration: FeedbackIntegration,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AutomatedValidation {
    pub difficulty_scoring: bool,
    pub readability_analysis: bool,
    pub character_distribution_check: bool,
    pub progression_validation: bool,
    pub performance_impact_assessment: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HumanReviewRequirements {
    pub new_content_review: bool,
    pub tier_transition_approval: bool,
    pub cultural_sensitivity_check: bool,
    pub technical_accuracy_verification: bool,
    pub accessibility_compliance_audit: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContinuousMonitoring {
    pub user_performance_tracking: bool,
    pub content_effectiveness_measurement: bool,
    pub difficulty_curve_optimization: bool,
    pub error_pattern_analysis: bool,
    pub real_time_adjustments: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FeedbackIntegration {
    pub user_feedback_collection: bool,
    pub automated_improvement_suggestions: bool,
    pub a_b_testing_framework: bool,
    pub community_content_contributions: bool,
    pub expert_review_incorporation: bool,
}

/// Default content metadata configuration for Centotype
impl Default for ContentMetadata {
    fn default() -> Self {
        let mut tier_definitions = HashMap::new();

        // Tier 1: Letters
        tier_definitions.insert(
            "tier_1".to_string(),
            TierDefinition {
                name: "Letters".to_string(),
                level_range: (1, 40),
                description: "Basic letter combinations and common words".to_string(),
                character_sets: vec!["a-z".to_string(), "A-Z".to_string()],
                progression_type: ProgressionType::Linear,
                learning_objectives: vec![
                    "Master home row keys".to_string(),
                    "Develop finger independence".to_string(),
                    "Build muscle memory for all letters".to_string(),
                    "Achieve consistent rhythm".to_string(),
                ],
                difficulty_curve: DifficultyParameters {
                    base_score: 1.0,
                    tier_multiplier: 1.0,
                    max_level_increment: 0.2,
                    complexity_factors: vec![
                        ComplexityWeight {
                            factor_type: "finger_distance".to_string(),
                            weight: 0.4,
                            tier_scaling: 1.0,
                        },
                        ComplexityWeight {
                            factor_type: "character_frequency".to_string(),
                            weight: 0.6,
                            tier_scaling: 1.0,
                        },
                    ],
                },
            },
        );

        // Tier 2: Punctuation
        tier_definitions.insert(
            "tier_2".to_string(),
            TierDefinition {
                name: "Punctuation".to_string(),
                level_range: (41, 60),
                description: "Progressive punctuation introduction".to_string(),
                character_sets: vec![
                    "a-z".to_string(),
                    "A-Z".to_string(),
                    ".,!?;:\"'()".to_string(),
                ],
                progression_type: ProgressionType::Exponential,
                learning_objectives: vec![
                    "Master basic punctuation marks".to_string(),
                    "Develop sentence structure awareness".to_string(),
                    "Handle complex punctuation combinations".to_string(),
                    "Maintain accuracy with increased cognitive load".to_string(),
                ],
                difficulty_curve: DifficultyParameters {
                    base_score: 5.0,
                    tier_multiplier: 1.2,
                    max_level_increment: 0.3,
                    complexity_factors: vec![
                        ComplexityWeight {
                            factor_type: "punctuation_density".to_string(),
                            weight: 0.5,
                            tier_scaling: 1.2,
                        },
                        ComplexityWeight {
                            factor_type: "sentence_complexity".to_string(),
                            weight: 0.3,
                            tier_scaling: 1.1,
                        },
                        ComplexityWeight {
                            factor_type: "cognitive_load".to_string(),
                            weight: 0.2,
                            tier_scaling: 1.3,
                        },
                    ],
                },
            },
        );

        // Continue with other tiers...

        Self {
            version: "1.0.0".to_string(),
            generated_at: "2025-09-27T00:00:00Z".to_string(),
            total_levels: 100,
            tier_definitions,
            validation_rules: ValidationRules {
                content_quality: ContentQualityRules {
                    min_unique_characters: 4,
                    max_repetition_ratio: 0.3,
                    min_readability_score: 6.0,
                    max_obscurity_ratio: 0.1,
                    profanity_filtering: true,
                    cultural_neutrality: true,
                    professional_appropriateness: true,
                },
                difficulty_progression: DifficultyProgressionRules {
                    max_difficulty_jump: 0.5,
                    min_difficulty_increment: 0.1,
                    smoothness_tolerance: 0.15,
                    tier_transition_validation: true,
                    regression_prevention: true,
                },
                performance_constraints: PerformanceConstraints {
                    max_content_size_bytes: 1048576, // 1MB
                    max_loading_time_ms: 50,
                    max_validation_time_ms: 10,
                    memory_limit_mb: 50,
                    cache_efficiency_target: 0.85,
                },
                accessibility_requirements: AccessibilityRequirements {
                    contrast_ratio_minimum: 4.5,
                    font_size_scalability: true,
                    screen_reader_compatibility: true,
                    keyboard_navigation_only: true,
                    color_blind_safe_indicators: true,
                },
            },
            content_schema: ContentSchema {
                text_entry: TextEntrySchema {
                    required_fields: vec![
                        "id".to_string(),
                        "content".to_string(),
                        "language".to_string(),
                        "difficulty_score".to_string(),
                    ],
                    optional_fields: vec![
                        "metadata".to_string(),
                        "tags".to_string(),
                        "source".to_string(),
                    ],
                    content_constraints: ContentConstraints {
                        min_length_chars: 40,
                        max_length_chars: 3500,
                        allowed_unicode_ranges: vec![
                            UnicodeRange {
                                name: "Basic Latin".to_string(),
                                start: 0x0020,
                                end: 0x007F,
                                description: "Standard ASCII characters".to_string(),
                            },
                            UnicodeRange {
                                name: "Latin-1 Supplement".to_string(),
                                start: 0x00A0,
                                end: 0x00FF,
                                description: "Extended Latin characters".to_string(),
                            },
                        ],
                        forbidden_patterns: vec![
                            "password".to_string(),
                            "secret".to_string(),
                            "private".to_string(),
                        ],
                        required_character_distribution: CharacterDistribution {
                            letters_min_ratio: 0.6,
                            numbers_max_ratio: 0.2,
                            punctuation_max_ratio: 0.15,
                            symbols_max_ratio: 0.25,
                            whitespace_target_ratio: 0.15,
                        },
                    },
                    encoding_requirements: EncodingRequirements {
                        encoding: "UTF-8".to_string(),
                        normalization_form: "NFC".to_string(),
                        byte_order_mark: false,
                        line_ending_style: LineEndingStyle::Unix,
                    },
                },
                metadata_fields: MetadataFields {
                    core_metrics: vec![
                        MetricDefinition {
                            name: "difficulty_score".to_string(),
                            data_type: MetricDataType::Float,
                            calculation_method: "weighted_character_analysis".to_string(),
                            validation_range: (1.0, 20.0),
                            precision: 2,
                        },
                        MetricDefinition {
                            name: "estimated_wpm".to_string(),
                            data_type: MetricDataType::Integer,
                            calculation_method: "inverse_difficulty_mapping".to_string(),
                            validation_range: (20.0, 150.0),
                            precision: 0,
                        },
                    ],
                    difficulty_indicators: vec![DifficultyIndicator {
                        indicator_name: "character_complexity".to_string(),
                        weight_in_total_score: 0.4,
                        calculation_formula: "weighted_sum(char_frequencies)".to_string(),
                        tier_specific_adjustments: HashMap::new(),
                    }],
                    learning_analytics: vec![AnalyticsField {
                        field_name: "completion_time".to_string(),
                        purpose: AnalyticsPurpose::PerformanceTracking,
                        data_retention_days: 90,
                        privacy_level: PrivacyLevel::Anonymous,
                    }],
                    content_classification: ClassificationSchema {
                        primary_categories: vec![
                            "letters".to_string(),
                            "punctuation".to_string(),
                            "numbers".to_string(),
                            "symbols".to_string(),
                        ],
                        secondary_tags: vec![
                            "programming".to_string(),
                            "technical".to_string(),
                            "general".to_string(),
                        ],
                        skill_focus_areas: vec![SkillArea {
                            name: "finger_independence".to_string(),
                            description: "Ability to move fingers independently".to_string(),
                            associated_tiers: vec![1, 2],
                            progression_indicators: vec![
                                "reduced_adjacent_finger_errors".to_string()
                            ],
                        }],
                        content_themes: vec![ContentTheme {
                            theme_name: "software_development".to_string(),
                            description: "Programming and technical content".to_string(),
                            target_audience: TargetAudience::Technical,
                            vocabulary_complexity: VocabularyComplexity::Advanced,
                        }],
                    },
                },
                language_support: LanguageSupport {
                    primary_languages: vec![LanguageConfig {
                        language_code: "en".to_string(),
                        language_name: "English".to_string(),
                        character_set: "a-zA-Z".to_string(),
                        typing_difficulty_modifier: 1.0,
                        frequency_patterns: FrequencyPatterns {
                            letter_frequencies: HashMap::new(),
                            bigram_frequencies: HashMap::new(),
                            common_words: vec![
                                "the".to_string(),
                                "and".to_string(),
                                "for".to_string(),
                            ],
                            difficult_combinations: vec!["ght".to_string(), "tch".to_string()],
                        },
                    }],
                    secondary_languages: vec![LanguageConfig {
                        language_code: "id".to_string(),
                        language_name: "Indonesian".to_string(),
                        character_set: "a-zA-Z".to_string(),
                        typing_difficulty_modifier: 1.1,
                        frequency_patterns: FrequencyPatterns {
                            letter_frequencies: HashMap::new(),
                            bigram_frequencies: HashMap::new(),
                            common_words: vec![
                                "yang".to_string(),
                                "dan".to_string(),
                                "untuk".to_string(),
                            ],
                            difficult_combinations: vec!["ng".to_string(), "ny".to_string()],
                        },
                    }],
                    mixed_content_rules: MixedContentRules {
                        max_language_switches_per_100_chars: 2,
                        min_segment_length_chars: 20,
                        transition_smoothness_requirement: 0.8,
                        context_preservation_rules: vec!["maintain_sentence_coherence".to_string()],
                    },
                    localization_requirements: LocalizationRequirements {
                        number_format_localization: false,
                        date_format_localization: false,
                        currency_symbol_support: true,
                        cultural_context_adaptation: true,
                    },
                },
                content_categories: vec![ContentCategory {
                    category_name: "programming".to_string(),
                    description: "Code snippets and technical content".to_string(),
                    tier_availability: vec![3, 4],
                    content_characteristics: ContentCharacteristics {
                        average_word_length: 8.5,
                        sentence_complexity: SentenceComplexity::Complex,
                        technical_terminology_ratio: 0.4,
                        punctuation_density: 0.12,
                        symbol_usage_patterns: vec![SymbolPattern {
                            symbol_class: "brackets".to_string(),
                            usage_frequency: 0.08,
                            context_rules: vec!["balanced_pairs".to_string()],
                            difficulty_contribution: 1.5,
                        }],
                    },
                    generation_parameters: GenerationParameters {
                        seed_algorithms: vec![SeedAlgorithm {
                            algorithm_name: "deterministic_pattern".to_string(),
                            deterministic: true,
                            parameter_ranges: HashMap::new(),
                            output_consistency_target: 0.95,
                        }],
                        content_templates: vec![ContentTemplate {
                            template_id: "code_function".to_string(),
                            pattern_description: "Function definition pattern".to_string(),
                            variable_slots: vec![VariableSlot {
                                slot_name: "function_name".to_string(),
                                content_type: SlotContentType::TechnicalTerm,
                                generation_rules: vec![GenerationRule {
                                    rule_type: RuleType::LengthConstraint,
                                    parameters: HashMap::new(),
                                    priority: 1,
                                }],
                                validation_constraints: vec![ValidationConstraint {
                                    constraint_type: ConstraintType::MinLength,
                                    threshold_value: 3.0,
                                    error_severity: ErrorSeverity::Error,
                                }],
                            }],
                            difficulty_scaling: DifficultyScaling {
                                base_difficulty: 8.0,
                                scaling_function: ScalingFunction::Linear(0.2),
                                tier_adjustments: HashMap::new(),
                                level_progressions: vec![LevelProgression {
                                    level: 90,
                                    target_difficulty: 14.0,
                                    expected_wpm: 35,
                                    mastery_criteria: MasteryCriteria {
                                        min_accuracy: 0.985,
                                        max_error_severity: 4.0,
                                        consistency_requirement: 0.9,
                                        time_constraints: TimeConstraints {
                                            max_completion_time_seconds: 180,
                                            min_sustained_wpm: 30,
                                            fatigue_resistance_factor: 0.85,
                                        },
                                    },
                                }],
                            },
                        }],
                        variability_factors: VariabilityFactors {
                            content_randomization: 0.3,
                            difficulty_variance: 0.1,
                            pattern_prevention: PatternPrevention {
                                max_repetition_frequency: 0.2,
                                pattern_detection_window: 50,
                                variation_enforcement: true,
                                anti_memorization_measures: vec![
                                    "rotate_vocabulary".to_string(),
                                    "shuffle_sentence_structures".to_string(),
                                ],
                            },
                            freshness_maintenance: FreshnessMaintenance {
                                content_rotation_period_days: 30,
                                new_content_introduction_rate: 0.1,
                                obsolete_content_retirement: true,
                                seasonal_content_updates: false,
                            },
                        },
                        quality_assurance: QualityAssurance {
                            automated_validation: AutomatedValidation {
                                difficulty_scoring: true,
                                readability_analysis: true,
                                character_distribution_check: true,
                                progression_validation: true,
                                performance_impact_assessment: true,
                            },
                            human_review_requirements: HumanReviewRequirements {
                                new_content_review: true,
                                tier_transition_approval: true,
                                cultural_sensitivity_check: true,
                                technical_accuracy_verification: true,
                                accessibility_compliance_audit: true,
                            },
                            continuous_monitoring: ContinuousMonitoring {
                                user_performance_tracking: true,
                                content_effectiveness_measurement: true,
                                difficulty_curve_optimization: true,
                                error_pattern_analysis: true,
                                real_time_adjustments: false,
                            },
                            feedback_integration: FeedbackIntegration {
                                user_feedback_collection: true,
                                automated_improvement_suggestions: true,
                                a_b_testing_framework: false,
                                community_content_contributions: false,
                                expert_review_incorporation: true,
                            },
                        },
                    },
                }],
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_content_metadata_creation() {
        let metadata = ContentMetadata::default();
        assert_eq!(metadata.total_levels, 100);
        assert_eq!(metadata.tier_definitions.len(), 2); // Only tier_1 and tier_2 in default
    }

    #[test]
    fn test_validation_rules() {
        let metadata = ContentMetadata::default();
        assert!(
            metadata
                .validation_rules
                .content_quality
                .min_unique_characters
                >= 4
        );
        assert!(
            metadata
                .validation_rules
                .content_quality
                .max_repetition_ratio
                <= 0.5
        );
    }
}

fn main() {
    println!("Content metadata specification - see module documentation");
}
