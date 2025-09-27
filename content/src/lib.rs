//! # Centotype Content
//!
//! Text corpus management and dynamic content generation system.
//! This crate handles:
//!
//! - Static text corpus loading and caching
//! - Dynamic content generation with deterministic seeding
//! - Difficulty analysis and validation
//! - Multi-language content support
//! - Character class distribution analysis
//! - Performance-optimized LRU caching
//! - Security validation for generated content

pub mod cache;
pub mod corpus;
pub mod difficulty;
pub mod generator;
pub mod validation;

// Re-export main types for public API
pub use cache::{CacheConfig, CacheManager, CacheMetrics, ContentCache, PreloadStrategy};
pub use difficulty::{DifficultyAnalyzer, DifficultyConfig, DifficultyScore, ProgressionReport, TierRequirements};
pub use generator::{CentotypeContentGenerator, DifficultyParams, LevelGenerationParams, generate_cache_key};
pub use validation::{ContentValidator, ValidationResult, verify_difficulty_progression};

use centotype_core::types::*;
use std::sync::Arc;
use parking_lot::RwLock;
use tracing::{debug, info, warn};

/// Main content management system integrating all components
pub struct ContentManager {
    /// Content generator for creating level content
    generator: Arc<CentotypeContentGenerator>,
    /// High-performance cache for content storage
    cache: Arc<ContentCache>,
    /// Cache manager for coordination
    cache_manager: Arc<CacheManager>,
    /// Difficulty analyzer for progression validation
    difficulty_analyzer: Arc<DifficultyAnalyzer>,
    /// Thread-safe configuration
    config: Arc<RwLock<ContentConfig>>,
}

/// Configuration for the content management system
#[derive(Debug, Clone)]
pub struct ContentConfig {
    /// Enable content preloading
    pub enable_preloading: bool,
    /// Preloading strategy
    pub preload_strategy: PreloadStrategy,
    /// Cache configuration
    pub cache_config: CacheConfig,
    /// Difficulty analysis configuration
    pub difficulty_config: DifficultyConfig,
    /// Enable comprehensive content validation
    pub enable_validation: bool,
    /// Seed for deterministic content generation
    pub default_seed: Option<u64>,
}

impl Default for ContentConfig {
    fn default() -> Self {
        Self {
            enable_preloading: true,
            preload_strategy: PreloadStrategy::Sequential(3),
            cache_config: CacheConfig::default(),
            difficulty_config: DifficultyConfig::default(),
            enable_validation: true,
            default_seed: None, // Use random seeds by default
        }
    }
}

impl ContentManager {
    /// Create new content manager with default configuration
    pub async fn new() -> Result<Self> {
        Self::with_config(ContentConfig::default()).await
    }

    /// Create content manager with custom configuration
    pub async fn with_config(config: ContentConfig) -> Result<Self> {
        info!("Initializing Centotype content management system");

        // Initialize validation system
        let validator = Arc::new(validation::ContentValidator::new()
            .map_err(|e| CentotypeError::Content(format!("Failed to create validator: {}", e)))?);

        // Initialize content generator
        let generator = Arc::new(generator::CentotypeContentGenerator::new(validator));

        // Initialize cache manager
        let cache_manager = Arc::new(CacheManager::with_config(
            generator.clone(),
            config.cache_config.clone(),
        )?);

        // Get cache reference
        let cache = cache_manager.content_cache();

        // Initialize difficulty analyzer
        let difficulty_analyzer = Arc::new(DifficultyAnalyzer::with_config(
            config.difficulty_config.clone()
        ));

        info!("Content management system initialized successfully");

        Ok(Self {
            generator,
            cache,
            cache_manager,
            difficulty_analyzer,
            config: Arc::new(RwLock::new(config)),
        })
    }

    /// Get content for a specific level with caching and validation
    pub async fn get_level_content(&self, level_id: LevelId, seed: Option<u64>) -> Result<String> {
        let config = self.config.read();
        let effective_seed = seed.or(config.default_seed).unwrap_or_else(|| {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut hasher = DefaultHasher::new();
            level_id.hash(&mut hasher);
            hasher.finish()
        });

        debug!("Requesting content for level {} with seed {}", level_id.0, effective_seed);

        // Get content from cache (with generation on miss)
        let content = self.cache.get_content(level_id, effective_seed).await?;

        // Validate content if enabled
        if config.enable_validation {
            self.generator.validate_content_difficulty(&content, level_id);
        }

        // Trigger preloading for upcoming levels if enabled
        if config.enable_preloading {
            if let Err(e) = self.cache.preload_upcoming_levels(level_id).await {
                warn!("Failed to preload upcoming levels: {}", e);
                // Don't fail the request for preloading errors
            }
        }

        debug!("Successfully retrieved content for level {} ({} chars)", level_id.0, content.len());
        Ok(content)
    }

    /// Get cached content only (no generation on miss)
    pub async fn get_cached_content(&self, level_id: LevelId, seed: Option<u64>) -> Option<String> {
        let config = self.config.read();
        let effective_seed = seed.or(config.default_seed).unwrap_or_else(|| {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut hasher = DefaultHasher::new();
            level_id.hash(&mut hasher);
            hasher.finish()
        });

        self.cache.get_cached_content(level_id, effective_seed).await
    }

    /// Preload content for upcoming levels
    pub async fn preload_upcoming_levels(&self, current_level: LevelId) -> Result<()> {
        debug!("Preloading content for upcoming levels from level {}", current_level.0);
        self.cache.preload_upcoming_levels(current_level).await
    }

    /// Invalidate cached content for a specific level
    pub async fn invalidate_level(&self, level_id: LevelId, seed: Option<u64>) {
        let config = self.config.read();
        let effective_seed = seed.or(config.default_seed).unwrap_or_else(|| {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut hasher = DefaultHasher::new();
            level_id.hash(&mut hasher);
            hasher.finish()
        });

        self.cache.invalidate(level_id, effective_seed).await;
        debug!("Invalidated cached content for level {}", level_id.0);
    }

    /// Clear all cached content
    pub async fn clear_cache(&self) {
        self.cache.clear().await;
        info!("Cleared all cached content");
    }

    /// Get cache performance metrics
    pub fn get_cache_metrics(&self) -> CacheMetrics {
        self.cache_manager.get_aggregated_metrics()
    }

    /// Validate cache performance against targets
    pub fn validate_cache_performance(&self) -> Result<()> {
        self.cache_manager.validate_performance_targets()
    }

    /// Analyze difficulty of content
    pub fn analyze_difficulty(&self, content: &str) -> DifficultyScore {
        self.difficulty_analyzer.analyze_content(content)
    }

    /// Validate difficulty progression across multiple levels
    pub async fn validate_progression(&self, level_range: std::ops::Range<u8>) -> Result<()> {
        let mut contents = Vec::new();

        for level_num in level_range {
            if level_num < LevelId::MIN || level_num > LevelId::MAX {
                continue;
            }

            let level_id = LevelId::new(level_num)?;
            if let Some(content) = self.get_cached_content(level_id, None).await {
                contents.push((level_id, content));
            } else {
                // Generate content for validation
                let content = self.get_level_content(level_id, None).await?;
                contents.push((level_id, content));
            }
        }

        self.difficulty_analyzer.validate_progression(&contents)
    }

    /// Generate difficulty progression report
    pub async fn generate_progression_report(&self, level_range: std::ops::Range<u8>) -> Result<ProgressionReport> {
        let mut contents = Vec::new();

        for level_num in level_range {
            if level_num < LevelId::MIN || level_num > LevelId::MAX {
                continue;
            }

            let level_id = LevelId::new(level_num)?;
            let content = self.get_level_content(level_id, None).await?;
            contents.push((level_id, content));
        }

        Ok(self.difficulty_analyzer.generate_progression_report(&contents))
    }

    /// Update content configuration
    pub async fn update_config(&self, new_config: ContentConfig) -> Result<()> {
        let mut config = self.config.write();
        *config = new_config;
        info!("Updated content configuration");
        Ok(())
    }

    /// Get current configuration
    pub async fn get_config(&self) -> ContentConfig {
        self.config.read().clone()
    }

    /// Run maintenance tasks (cache cleanup, metrics update, etc.)
    pub async fn run_maintenance(&self) {
        debug!("Running content system maintenance");
        self.cache_manager.run_maintenance().await;
    }

    /// Generate deterministic content for testing
    pub async fn generate_deterministic_content(&self, level_id: LevelId, seed: u64) -> Result<String> {
        self.generator.generate_level_content(level_id, seed)
    }

    /// Validate that content meets security requirements
    pub fn validate_content_security(&self, content: &str) -> ValidationResult {
        // Access validator through generator
        // For now, we'll create a temporary validator for this operation
        let validator = validation::ContentValidator::new()
            .map_err(|e| CentotypeError::Content(format!("Failed to create validator: {}", e)))
            .unwrap();

        validator.validate_security(content)
    }

    /// Get tier requirements for a specific tier
    pub fn get_tier_requirements(&self, tier: Tier) -> TierRequirements {
        DifficultyAnalyzer::get_tier_requirements(tier)
    }

    /// Check if content meets difficulty requirements for a level
    pub fn validate_content_difficulty(&self, content: &str, level_id: LevelId) -> bool {
        self.generator.validate_content_difficulty(content, level_id)
    }
}

impl Default for ContentManager {
    fn default() -> Self {
        // This is a blocking operation, so we use a simple fallback
        // In practice, users should use ContentManager::new().await
        panic!("ContentManager::default() not supported - use ContentManager::new().await instead")
    }
}

/// Statistics for monitoring content system health
#[derive(Debug, Clone)]
pub struct CacheStatistics {
    pub total_items: usize,
    pub memory_usage_bytes: usize,
    pub hit_rate: f64,
    pub miss_count: u64,
}

impl From<CacheMetrics> for CacheStatistics {
    fn from(metrics: CacheMetrics) -> Self {
        Self {
            total_items: (metrics.hit_count + metrics.miss_count) as usize,
            memory_usage_bytes: metrics.memory_usage_bytes as usize,
            hit_rate: metrics.hit_rate(),
            miss_count: metrics.miss_count,
        }
    }
}

/// Quick validation helper for testing deterministic generation
pub async fn validate_deterministic_generation(
    manager: &ContentManager,
    level_id: LevelId,
    seed: u64,
    iterations: usize,
) -> Result<bool> {
    let mut previous_content: Option<String> = None;

    for i in 0..iterations {
        let content = manager.generate_deterministic_content(level_id, seed).await?;

        if let Some(prev) = &previous_content {
            if *prev != content {
                warn!(
                    "Deterministic generation failed at iteration {}: content differs",
                    i + 1
                );
                return Ok(false);
            }
        }

        previous_content = Some(content);
    }

    debug!(
        "Deterministic generation validated for level {} across {} iterations",
        level_id.0, iterations
    );
    Ok(true)
}

/// Performance testing helper
pub async fn benchmark_content_loading(
    manager: &ContentManager,
    level_id: LevelId,
    iterations: usize,
) -> Result<std::time::Duration> {
    let start = std::time::Instant::now();

    for _ in 0..iterations {
        let _content = manager.get_level_content(level_id, None).await?;
    }

    let total_duration = start.elapsed();
    let avg_duration = total_duration / iterations as u32;

    debug!(
        "Content loading benchmark: {} iterations, avg {}μs per load",
        iterations,
        avg_duration.as_micros()
    );

    Ok(avg_duration)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_content_manager_creation() {
        let manager = ContentManager::new().await.unwrap();
        assert!(!manager.get_cache_metrics().hit_rate().is_nan());
    }

    #[tokio::test]
    async fn test_level_content_generation() {
        let manager = ContentManager::new().await.unwrap();
        let level = LevelId::new(1).unwrap();

        let content1 = manager.get_level_content(level, Some(12345)).await.unwrap();
        let content2 = manager.get_level_content(level, Some(12345)).await.unwrap();

        assert_eq!(content1, content2, "Content should be deterministic with same seed");
        assert!(!content1.is_empty(), "Content should not be empty");
    }

    #[tokio::test]
    async fn test_cache_functionality() {
        let manager = ContentManager::new().await.unwrap();
        let level = LevelId::new(5).unwrap();
        let seed = 67890;

        // First access should be a cache miss
        let _content1 = manager.get_level_content(level, Some(seed)).await.unwrap();

        // Second access should be a cache hit
        let _content2 = manager.get_level_content(level, Some(seed)).await.unwrap();

        let metrics = manager.get_cache_metrics();
        assert!(metrics.hit_count > 0, "Should have cache hits");
    }

    #[tokio::test]
    async fn test_difficulty_analysis() {
        let manager = ContentManager::new().await.unwrap();

        let simple_text = "This is a simple sentence with basic words.";
        let complex_text = "&mut HashMap<String, Vec<Option<Box<dyn Iterator<Item=u32>>>>>";

        let simple_score = manager.analyze_difficulty(simple_text);
        let complex_score = manager.analyze_difficulty(complex_text);

        assert!(simple_score.overall < complex_score.overall,
               "Complex text should have higher difficulty score");
    }

    #[tokio::test]
    async fn test_content_validation() {
        let manager = ContentManager::new().await.unwrap();

        // Test security validation
        let safe_content = "function test() { return 42; }";
        let unsafe_content = "\x1b[31mRed text\x1b[0m";

        assert!(manager.validate_content_security(safe_content).is_valid());
        assert!(!manager.validate_content_security(unsafe_content).is_valid());
    }

    #[tokio::test]
    async fn test_deterministic_generation() {
        let manager = ContentManager::new().await.unwrap();
        let level = LevelId::new(10).unwrap();
        let seed = 98765;

        let is_deterministic = validate_deterministic_generation(&manager, level, seed, 5)
            .await
            .unwrap();

        assert!(is_deterministic, "Content generation should be deterministic");
    }

    #[tokio::test]
    async fn test_performance_targets() {
        let manager = ContentManager::new().await.unwrap();
        let level = LevelId::new(1).unwrap();

        // Warm up cache
        let _content = manager.get_level_content(level, None).await.unwrap();

        // Benchmark loading time
        let avg_duration = benchmark_content_loading(&manager, level, 10).await.unwrap();

        // Should meet P99 target of <25ms (being generous with test environment)
        assert!(avg_duration.as_millis() < 50,
               "Content loading should be fast: {}ms", avg_duration.as_millis());
    }

    #[tokio::test]
    async fn test_cache_invalidation() {
        let manager = ContentManager::new().await.unwrap();
        let level = LevelId::new(3).unwrap();
        let seed = 11111;

        // Generate and cache content
        let _content1 = manager.get_level_content(level, Some(seed)).await.unwrap();
        assert!(manager.get_cached_content(level, Some(seed)).await.is_some());

        // Invalidate cache
        manager.invalidate_level(level, Some(seed)).await;
        assert!(manager.get_cached_content(level, Some(seed)).await.is_none());
    }

    #[tokio::test]
    async fn test_progression_validation() {
        let manager = ContentManager::new().await.unwrap();

        // Test progression for first few levels
        let validation_result = manager.validate_progression(1..6).await;
        assert!(validation_result.is_ok(), "Progression validation should pass");
    }
}
