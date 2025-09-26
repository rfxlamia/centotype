use std::collections::HashMap;
use std::sync::{Arc, RwLock};
use std::time::{Duration, Instant};
use serde::{Deserialize, Serialize};
use lru::LruCache;

/// Content loading and caching strategy for Centotype
///
/// This module implements an efficient content management system designed to:
/// - Meet performance constraints (<50MB RSS, <50ms load time)
/// - Provide responsive content access for all 100 levels
/// - Support deterministic content generation and validation
/// - Enable efficient memory usage and cache management
/// - Support both static corpus and dynamic generation

#[derive(Debug, Clone)]
pub struct ContentManager {
    cache: Arc<RwLock<LevelCache>>,
    config: CacheConfig,
    metrics: Arc<RwLock<CacheMetrics>>,
    corpus_loader: CorpusLoader,
    generator: ContentGenerator,
}

#[derive(Debug)]
struct LevelCache {
    content_cache: LruCache<u32, CachedContent>,
    metadata_cache: LruCache<u32, LevelMetadata>,
    validation_cache: LruCache<String, ValidationResult>,
    preload_queue: Vec<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheConfig {
    pub max_cache_size_mb: usize,
    pub max_cached_levels: usize,
    pub preload_strategy: PreloadStrategy,
    pub cache_invalidation_ttl: Duration,
    pub compression_enabled: bool,
    pub memory_pressure_threshold: f64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum PreloadStrategy {
    None,
    Adjacent(u32),           // Preload N levels before/after current
    Progressive(Vec<u32>),   // Preload specific levels
    Tier(u32),              // Preload entire tier
    Adaptive,               // Based on user progress patterns
}

#[derive(Debug, Clone)]
pub struct CachedContent {
    pub level: u32,
    pub texts: Vec<TextContent>,
    pub metadata: LevelMetadata,
    pub cached_at: Instant,
    pub access_count: u32,
    pub compressed_size: usize,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextContent {
    pub id: String,
    pub content: String,
    pub language: String,
    pub difficulty_score: f64,
    pub estimated_wpm: u32,
    pub character_set: String,
    pub metadata: TextMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TextMetadata {
    pub unique_chars: u32,
    pub length: usize,
    pub bigram_complexity: f64,
    pub symbol_density: f64,
    pub finger_usage: Vec<String>,
    pub validation_passed: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LevelMetadata {
    pub level: u32,
    pub tier: u32,
    pub difficulty_score: f64,
    pub estimated_wpm: u32,
    pub character_set: String,
    pub focus_description: String,
    pub learning_objectives: Vec<String>,
    pub unlock_criteria: UnlockCriteria,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnlockCriteria {
    pub previous_level_grade: String,
    pub min_accuracy: f64,
    pub min_wpm: u32,
    pub max_error_severity: f64,
}

#[derive(Debug, Clone)]
pub struct ValidationResult {
    pub content_id: String,
    pub valid: bool,
    pub difficulty_score: f64,
    pub warnings: Vec<String>,
    pub errors: Vec<String>,
    pub validated_at: Instant,
}

#[derive(Debug, Clone)]
pub struct CacheMetrics {
    pub hit_rate: f64,
    pub miss_rate: f64,
    pub total_requests: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub memory_usage_mb: f64,
    pub average_load_time_ms: f64,
    pub eviction_count: u64,
    pub compression_ratio: f64,
}

#[derive(Debug)]
pub struct CorpusLoader {
    corpus_paths: HashMap<u32, String>,
    tier_configs: HashMap<u32, TierConfig>,
    file_cache: HashMap<String, (Vec<u8>, Instant)>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TierConfig {
    pub tier_id: u32,
    pub name: String,
    pub level_range: (u32, u32),
    pub base_path: String,
    pub content_format: ContentFormat,
    pub compression_type: CompressionType,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ContentFormat {
    JSON,
    MessagePack,
    Custom(String),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CompressionType {
    None,
    Gzip,
    Lz4,
    Zstd,
}

#[derive(Debug)]
pub struct ContentGenerator {
    seed_algorithms: HashMap<String, Box<dyn SeedAlgorithm>>,
    templates: HashMap<String, ContentTemplate>,
    difficulty_calculator: DifficultyCalculator,
}

pub trait SeedAlgorithm: Send + Sync {
    fn generate(&self, level: u32, seed: u64) -> GeneratedContent;
    fn validate_output(&self, content: &GeneratedContent) -> ValidationResult;
}

#[derive(Debug, Clone)]
pub struct GeneratedContent {
    pub text: String,
    pub metadata: GenerationMetadata,
    pub seed_used: u64,
    pub generation_time_ms: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationMetadata {
    pub algorithm_used: String,
    pub template_id: Option<String>,
    pub difficulty_score: f64,
    pub character_distribution: HashMap<String, f64>,
    pub validation_status: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentTemplate {
    pub template_id: String,
    pub pattern: String,
    pub variables: HashMap<String, VariableConfig>,
    pub difficulty_range: (f64, f64),
    pub tier_compatibility: Vec<u32>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableConfig {
    pub variable_type: VariableType,
    pub constraints: VariableConstraints,
    pub generation_rules: Vec<GenerationRule>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum VariableType {
    Word,
    Number,
    Symbol,
    TechnicalTerm,
    Phrase,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VariableConstraints {
    pub min_length: usize,
    pub max_length: usize,
    pub character_classes: Vec<String>,
    pub frequency_range: (f64, f64),
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GenerationRule {
    pub rule_type: String,
    pub parameters: HashMap<String, String>,
    pub priority: u32,
}

#[derive(Debug)]
pub struct DifficultyCalculator {
    base_weights: HashMap<String, f64>,
    tier_multipliers: HashMap<u32, f64>,
    complexity_functions: HashMap<String, Box<dyn ComplexityFunction>>,
}

pub trait ComplexityFunction: Send + Sync {
    fn calculate(&self, content: &str) -> f64;
}

impl ContentManager {
    pub fn new(config: CacheConfig) -> Self {
        let cache_capacity = config.max_cached_levels;

        Self {
            cache: Arc::new(RwLock::new(LevelCache {
                content_cache: LruCache::new(cache_capacity),
                metadata_cache: LruCache::new(cache_capacity * 2), // Metadata is smaller
                validation_cache: LruCache::new(1000), // Validation results
                preload_queue: Vec::new(),
            })),
            config,
            metrics: Arc::new(RwLock::new(CacheMetrics::default())),
            corpus_loader: CorpusLoader::new(),
            generator: ContentGenerator::new(),
        }
    }

    /// Load content for a specific level with performance optimization
    pub async fn load_level_content(&self, level: u32) -> Result<Vec<TextContent>, ContentError> {
        let start_time = Instant::now();

        // Check cache first
        if let Some(cached) = self.get_cached_content(level) {
            self.record_cache_hit();
            return Ok(cached.texts);
        }

        self.record_cache_miss();

        // Load from corpus or generate
        let content = if self.has_static_content(level) {
            self.load_static_content(level).await?
        } else {
            self.generate_dynamic_content(level).await?
        };

        // Validate content
        let validated_content = self.validate_and_process(content, level).await?;

        // Cache the result
        self.cache_content(level, validated_content.clone()).await;

        // Record performance metrics
        let load_time = start_time.elapsed();
        self.record_load_time(load_time);

        // Trigger preloading if configured
        self.trigger_preload(level).await;

        Ok(validated_content)
    }

    /// Preload content based on strategy
    pub async fn preload_content(&self, current_level: u32) {
        match &self.config.preload_strategy {
            PreloadStrategy::None => return,
            PreloadStrategy::Adjacent(range) => {
                let start = current_level.saturating_sub(*range);
                let end = (current_level + range).min(100);

                for level in start..=end {
                    if level != current_level && !self.is_cached(level) {
                        let _ = self.load_level_content(level).await;
                    }
                }
            },
            PreloadStrategy::Progressive(levels) => {
                for &level in levels {
                    if !self.is_cached(level) {
                        let _ = self.load_level_content(level).await;
                    }
                }
            },
            PreloadStrategy::Tier(tier_id) => {
                let tier_range = self.get_tier_range(*tier_id);
                for level in tier_range.0..=tier_range.1 {
                    if !self.is_cached(level) {
                        let _ = self.load_level_content(level).await;
                    }
                }
            },
            PreloadStrategy::Adaptive => {
                // Implement adaptive preloading based on user patterns
                self.adaptive_preload(current_level).await;
            },
        }
    }

    /// Get content from cache
    fn get_cached_content(&self, level: u32) -> Option<CachedContent> {
        let mut cache = self.cache.write().unwrap();
        if let Some(cached) = cache.content_cache.get_mut(&level) {
            cached.access_count += 1;
            Some(cached.clone())
        } else {
            None
        }
    }

    /// Check if level is cached
    fn is_cached(&self, level: u32) -> bool {
        let cache = self.cache.read().unwrap();
        cache.content_cache.contains(&level)
    }

    /// Load static content from corpus files
    async fn load_static_content(&self, level: u32) -> Result<Vec<TextContent>, ContentError> {
        let tier = self.get_tier_for_level(level);
        let file_path = self.corpus_loader.get_file_path(tier, level)?;

        // Load and parse content
        let content_data = self.corpus_loader.load_file(&file_path).await?;
        let parsed_content = self.parse_content_data(content_data, level)?;

        Ok(parsed_content)
    }

    /// Generate dynamic content
    async fn generate_dynamic_content(&self, level: u32) -> Result<Vec<TextContent>, ContentError> {
        let tier = self.get_tier_for_level(level);
        let seed = self.get_deterministic_seed(level);

        // Select appropriate generation algorithm
        let algorithm = self.select_generation_algorithm(tier, level);
        let generated = algorithm.generate(level, seed);

        // Convert to TextContent format
        let text_content = self.convert_generated_content(generated, level)?;

        Ok(vec![text_content])
    }

    /// Validate and process content
    async fn validate_and_process(
        &self,
        content: Vec<TextContent>,
        level: u32
    ) -> Result<Vec<TextContent>, ContentError> {
        let mut validated_content = Vec::new();

        for text in content {
            // Validate difficulty progression
            if !self.validate_difficulty_progression(&text, level) {
                continue; // Skip invalid content
            }

            // Validate character set compliance
            if !self.validate_character_set(&text, level) {
                continue;
            }

            // Validate content quality
            if !self.validate_content_quality(&text) {
                continue;
            }

            validated_content.push(text);
        }

        if validated_content.is_empty() {
            return Err(ContentError::NoValidContent(level));
        }

        Ok(validated_content)
    }

    /// Cache content with memory management
    async fn cache_content(&self, level: u32, content: Vec<TextContent>) {
        let mut cache = self.cache.write().unwrap();

        // Check memory pressure
        if self.check_memory_pressure() {
            self.perform_cache_cleanup(&mut cache);
        }

        // Create cached content entry
        let cached = CachedContent {
            level,
            texts: content.clone(),
            metadata: self.get_level_metadata(level),
            cached_at: Instant::now(),
            access_count: 1,
            compressed_size: self.calculate_compressed_size(&content),
        };

        // Store in cache
        if let Some(evicted) = cache.content_cache.put(level, cached) {
            self.record_eviction();
        }

        // Update cache metrics
        self.update_cache_metrics();
    }

    /// Check memory pressure and trigger cleanup if needed
    fn check_memory_pressure(&self) -> bool {
        let current_usage = self.get_current_memory_usage_mb();
        let threshold = self.config.max_cache_size_mb as f64 * self.config.memory_pressure_threshold;
        current_usage > threshold
    }

    /// Perform cache cleanup under memory pressure
    fn perform_cache_cleanup(&self, cache: &mut LevelCache) {
        // Remove least recently used items
        let target_size = (self.config.max_cached_levels as f64 * 0.7) as usize;

        while cache.content_cache.len() > target_size {
            cache.content_cache.pop_lru();
        }

        // Clean validation cache
        while cache.validation_cache.len() > 500 {
            cache.validation_cache.pop_lru();
        }
    }

    /// Adaptive preloading based on user patterns
    async fn adaptive_preload(&self, current_level: u32) {
        // Analyze user progression patterns
        let next_likely_levels = self.predict_next_levels(current_level);

        for level in next_likely_levels {
            if !self.is_cached(level) {
                tokio::spawn({
                    let manager = self.clone();
                    async move {
                        let _ = manager.load_level_content(level).await;
                    }
                });
            }
        }
    }

    /// Predict likely next levels based on patterns
    fn predict_next_levels(&self, current_level: u32) -> Vec<u32> {
        let mut next_levels = Vec::new();

        // Sequential progression
        if current_level < 100 {
            next_levels.push(current_level + 1);
        }

        // Retry patterns (users often retry current or previous levels)
        if current_level > 1 {
            next_levels.push(current_level - 1);
        }
        next_levels.push(current_level);

        // Skip ahead patterns (advanced users)
        if current_level < 95 {
            next_levels.push(current_level + 5);
        }

        // Tier boundaries (users often jump to tier starts)
        let tier_starts = vec![1, 41, 61, 81];
        for &start in &tier_starts {
            if start > current_level && start <= current_level + 10 {
                next_levels.push(start);
            }
        }

        next_levels
    }

    // Helper methods for content management
    fn has_static_content(&self, level: u32) -> bool {
        self.corpus_loader.has_content_for_level(level)
    }

    fn get_tier_for_level(&self, level: u32) -> u32 {
        match level {
            1..=40 => 1,
            41..=60 => 2,
            61..=80 => 3,
            81..=100 => 4,
            _ => 1,
        }
    }

    fn get_tier_range(&self, tier: u32) -> (u32, u32) {
        match tier {
            1 => (1, 40),
            2 => (41, 60),
            3 => (61, 80),
            4 => (81, 100),
            _ => (1, 40),
        }
    }

    fn get_deterministic_seed(&self, level: u32) -> u64 {
        // Create deterministic seed based on level for reproducible content
        let base_seed = 0x1A2B3C4D5E6F7890u64;
        base_seed.wrapping_mul(level as u64).wrapping_add(level as u64 * 31)
    }

    fn select_generation_algorithm(&self, tier: u32, level: u32) -> &dyn SeedAlgorithm {
        // Select appropriate algorithm based on tier and level
        match tier {
            1 => self.generator.seed_algorithms.get("letter_progression").unwrap().as_ref(),
            2 => self.generator.seed_algorithms.get("punctuation_integration").unwrap().as_ref(),
            3 => self.generator.seed_algorithms.get("number_patterns").unwrap().as_ref(),
            4 => self.generator.seed_algorithms.get("symbol_mastery").unwrap().as_ref(),
            _ => self.generator.seed_algorithms.get("default").unwrap().as_ref(),
        }
    }

    // Validation methods
    fn validate_difficulty_progression(&self, content: &TextContent, level: u32) -> bool {
        let expected_difficulty = self.calculate_expected_difficulty(level);
        let tolerance = 0.3;

        (content.difficulty_score - expected_difficulty).abs() <= tolerance
    }

    fn validate_character_set(&self, content: &TextContent, level: u32) -> bool {
        let allowed_chars = self.get_allowed_character_set(level);
        content.content.chars().all(|c| allowed_chars.contains(&c))
    }

    fn validate_content_quality(&self, content: &TextContent) -> bool {
        // Check for excessive repetition
        let repetition_ratio = self.calculate_repetition_ratio(&content.content);
        if repetition_ratio > 0.3 {
            return false;
        }

        // Check minimum length
        if content.content.len() < 40 {
            return false;
        }

        // Check character diversity
        let unique_chars = content.content.chars().collect::<std::collections::HashSet<_>>().len();
        if unique_chars < 4 {
            return false;
        }

        true
    }

    // Metric recording methods
    fn record_cache_hit(&self) {
        let mut metrics = self.metrics.write().unwrap();
        metrics.cache_hits += 1;
        metrics.total_requests += 1;
        metrics.hit_rate = metrics.cache_hits as f64 / metrics.total_requests as f64;
    }

    fn record_cache_miss(&self) {
        let mut metrics = self.metrics.write().unwrap();
        metrics.cache_misses += 1;
        metrics.total_requests += 1;
        metrics.miss_rate = metrics.cache_misses as f64 / metrics.total_requests as f64;
    }

    fn record_load_time(&self, duration: Duration) {
        let mut metrics = self.metrics.write().unwrap();
        let load_time_ms = duration.as_millis() as f64;

        // Calculate rolling average
        let alpha = 0.1; // Smoothing factor
        if metrics.average_load_time_ms == 0.0 {
            metrics.average_load_time_ms = load_time_ms;
        } else {
            metrics.average_load_time_ms =
                alpha * load_time_ms + (1.0 - alpha) * metrics.average_load_time_ms;
        }
    }

    fn record_eviction(&self) {
        let mut metrics = self.metrics.write().unwrap();
        metrics.eviction_count += 1;
    }

    fn update_cache_metrics(&self) {
        let mut metrics = self.metrics.write().unwrap();
        metrics.memory_usage_mb = self.get_current_memory_usage_mb();
    }

    // Utility methods
    fn get_current_memory_usage_mb(&self) -> f64 {
        // Implementation would use system memory monitoring
        // For now, estimate based on cache size
        let cache = self.cache.read().unwrap();
        let estimated_size = cache.content_cache.len() * 50 * 1024; // ~50KB per level
        estimated_size as f64 / (1024.0 * 1024.0)
    }

    fn calculate_compressed_size(&self, content: &[TextContent]) -> usize {
        // Estimate compressed size
        let total_size: usize = content.iter().map(|c| c.content.len()).sum();
        if self.config.compression_enabled {
            total_size * 60 / 100 // Assume 60% compression ratio
        } else {
            total_size
        }
    }

    fn calculate_expected_difficulty(&self, level: u32) -> f64 {
        // Implement difficulty curve calculation
        let tier = self.get_tier_for_level(level);
        let tier_base = match tier {
            1 => 1.0,
            2 => 5.0,
            3 => 8.0,
            4 => 11.0,
            _ => 1.0,
        };

        let level_in_tier = level - self.get_tier_range(tier).0 + 1;
        let tier_progression = (level_in_tier as f64 - 1.0) * 0.2;

        tier_base + tier_progression
    }

    fn get_allowed_character_set(&self, level: u32) -> std::collections::HashSet<char> {
        // Implementation would return appropriate character set for level
        std::collections::HashSet::new() // Placeholder
    }

    fn calculate_repetition_ratio(&self, content: &str) -> f64 {
        let words: Vec<&str> = content.split_whitespace().collect();
        if words.is_empty() {
            return 0.0;
        }

        let unique_words: std::collections::HashSet<&str> = words.iter().cloned().collect();
        1.0 - (unique_words.len() as f64 / words.len() as f64)
    }

    // Additional helper methods would be implemented here...
}

// Error types
#[derive(Debug, thiserror::Error)]
pub enum ContentError {
    #[error("No valid content available for level {0}")]
    NoValidContent(u32),

    #[error("Content loading failed: {0}")]
    LoadingFailed(String),

    #[error("Validation failed: {0}")]
    ValidationFailed(String),

    #[error("Cache error: {0}")]
    CacheError(String),

    #[error("Generation failed: {0}")]
    GenerationFailed(String),
}

// Default implementations
impl Default for CacheMetrics {
    fn default() -> Self {
        Self {
            hit_rate: 0.0,
            miss_rate: 0.0,
            total_requests: 0,
            cache_hits: 0,
            cache_misses: 0,
            memory_usage_mb: 0.0,
            average_load_time_ms: 0.0,
            eviction_count: 0,
            compression_ratio: 0.6,
        }
    }
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_cache_size_mb: 25, // Half of 50MB memory constraint
            max_cached_levels: 15,  // Balance between memory and performance
            preload_strategy: PreloadStrategy::Adjacent(2),
            cache_invalidation_ttl: Duration::from_secs(3600), // 1 hour
            compression_enabled: true,
            memory_pressure_threshold: 0.8,
        }
    }
}

impl CorpusLoader {
    fn new() -> Self {
        Self {
            corpus_paths: HashMap::new(),
            tier_configs: HashMap::new(),
            file_cache: HashMap::new(),
        }
    }

    fn has_content_for_level(&self, level: u32) -> bool {
        // Check if static content exists for this level
        level <= 80 // Tiers 1-3 have static content, Tier 4 is generated
    }

    fn get_file_path(&self, tier: u32, level: u32) -> Result<String, ContentError> {
        // Return path to content file for tier/level
        Ok(format!("content/tier_{}/level_{}.json", tier, level))
    }

    async fn load_file(&self, path: &str) -> Result<Vec<u8>, ContentError> {
        // Implementation would load file from disk
        // This is a placeholder
        Ok(Vec::new())
    }
}

impl ContentGenerator {
    fn new() -> Self {
        Self {
            seed_algorithms: HashMap::new(),
            templates: HashMap::new(),
            difficulty_calculator: DifficultyCalculator::new(),
        }
    }
}

impl DifficultyCalculator {
    fn new() -> Self {
        Self {
            base_weights: HashMap::new(),
            tier_multipliers: HashMap::new(),
            complexity_functions: HashMap::new(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cache_config_defaults() {
        let config = CacheConfig::default();
        assert!(config.max_cache_size_mb <= 50); // Within memory constraint
        assert!(config.max_cached_levels > 0);
    }

    #[test]
    fn test_tier_level_mapping() {
        let manager = ContentManager::new(CacheConfig::default());
        assert_eq!(manager.get_tier_for_level(1), 1);
        assert_eq!(manager.get_tier_for_level(40), 1);
        assert_eq!(manager.get_tier_for_level(41), 2);
        assert_eq!(manager.get_tier_for_level(100), 4);
    }

    #[test]
    fn test_difficulty_calculation() {
        let manager = ContentManager::new(CacheConfig::default());
        let diff1 = manager.calculate_expected_difficulty(1);
        let diff50 = manager.calculate_expected_difficulty(50);
        let diff100 = manager.calculate_expected_difficulty(100);

        assert!(diff1 < diff50);
        assert!(diff50 < diff100);
    }
}