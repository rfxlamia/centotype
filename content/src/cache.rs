//! High-performance LRU cache system for content caching
//!
//! This module implements a thread-safe, performance-optimized cache using Moka
//! to meet the <25ms P99 content loading requirement. The cache supports
//! preloading, metrics collection, and graceful degradation.

use crate::generator::{generate_cache_key, CentotypeContentGenerator};
use centotype_core::types::*;
use moka::future::Cache;
use parking_lot::RwLock;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::Semaphore;
use tracing::{debug, instrument, warn};

/// Cache configuration parameters
#[derive(Debug, Clone)]
pub struct CacheConfig {
    /// Maximum number of cached content entries
    pub max_capacity: u64,
    /// Time to live for cache entries
    pub ttl: Duration,
    /// Time to idle before eviction
    pub tti: Duration,
    /// Maximum concurrent preloading operations
    pub max_preload_concurrent: usize,
}

impl Default for CacheConfig {
    fn default() -> Self {
        Self {
            max_capacity: 1000,             // Cache up to 1000 levels of content
            ttl: Duration::from_secs(3600), // 1 hour TTL
            tti: Duration::from_secs(1800), // 30 minute TTI
            max_preload_concurrent: 3,      // Preload max 3 levels concurrently
        }
    }
}

/// Cache metrics for monitoring performance
#[derive(Debug, Clone, Default)]
pub struct CacheMetrics {
    pub hit_count: u64,
    pub miss_count: u64,
    pub total_requests: u64,
    pub avg_lookup_time_micros: u64,
    pub preload_count: u64,
    pub eviction_count: u64,
    pub memory_usage_bytes: u64,
    pub error_count: u64,
}

impl CacheMetrics {
    /// Calculate cache hit rate as percentage
    pub fn hit_rate(&self) -> f64 {
        if self.total_requests == 0 {
            0.0
        } else {
            (self.hit_count as f64 / self.total_requests as f64) * 100.0
        }
    }

    /// Get average lookup time in milliseconds
    pub fn avg_lookup_time_ms(&self) -> f64 {
        self.avg_lookup_time_micros as f64 / 1000.0
    }

    /// Validate performance targets from master prompt
    pub fn validate_performance_targets(&self) -> Result<()> {
        // Cache hit rate should be >90%
        if self.hit_rate() < 90.0 {
            return Err(CentotypeError::Content(format!(
                "Cache hit rate too low: {:.2}% (target: >90%)",
                self.hit_rate()
            )));
        }

        // Average lookup time should be <5ms
        if self.avg_lookup_time_ms() > 5.0 {
            return Err(CentotypeError::Content(format!(
                "Cache lookup too slow: {:.2}ms (target: <5ms)",
                self.avg_lookup_time_ms()
            )));
        }

        Ok(())
    }

    /// Reset metrics (for testing)
    pub fn reset(&mut self) {
        *self = CacheMetrics::default();
    }
}

/// Preloading strategy for upcoming levels
#[derive(Debug, Clone)]
pub enum PreloadStrategy {
    /// Preload next N levels sequentially
    Sequential(u8),
    /// Preload levels based on user's historical access pattern
    Adaptive(Vec<LevelId>),
    /// Disable preloading
    None,
}

impl Default for PreloadStrategy {
    fn default() -> Self {
        PreloadStrategy::Sequential(3) // Preload next 3 levels by default
    }
}

/// Thread-safe content cache with LRU eviction and performance monitoring
pub struct ContentCache {
    /// Moka LRU cache for fast content lookup
    cache: Cache<String, String>,
    /// Content generator for cache misses
    generator: Arc<CentotypeContentGenerator>,
    /// Metrics collection
    metrics: Arc<RwLock<CacheMetrics>>,
    /// Preloading configuration
    preload_strategy: PreloadStrategy,
    /// Semaphore for limiting concurrent preloads
    preload_semaphore: Arc<Semaphore>,
    /// Cache configuration
    config: CacheConfig,
}

impl ContentCache {
    /// Create new content cache with specified configuration
    pub fn new(generator: Arc<CentotypeContentGenerator>, config: CacheConfig) -> Self {
        let cache = Cache::builder()
            .max_capacity(config.max_capacity)
            .time_to_live(config.ttl)
            .time_to_idle(config.tti)
            .build();

        let preload_semaphore = Arc::new(Semaphore::new(config.max_preload_concurrent));

        Self {
            cache,
            generator,
            metrics: Arc::new(RwLock::new(CacheMetrics::default())),
            preload_strategy: PreloadStrategy::default(),
            preload_semaphore,
            config,
        }
    }

    /// Get content for a level with caching (primary interface)
    #[instrument(skip(self), fields(level = %level_id.0, seed = %seed))]
    pub async fn get_content(&self, level_id: LevelId, seed: u64) -> Result<String> {
        let start_time = Instant::now();
        let cache_key = generate_cache_key(level_id, seed);

        // Increment total requests
        {
            let mut metrics = self.metrics.write();
            metrics.total_requests += 1;
        }

        // Try cache lookup first
        if let Some(content) = self.cache.get(&cache_key).await {
            let lookup_time = start_time.elapsed();
            self.update_metrics_hit(lookup_time);

            debug!(
                "Cache hit for level {} ({}μs)",
                level_id.0,
                lookup_time.as_micros()
            );

            return Ok(content);
        }

        // Cache miss - generate content
        debug!("Cache miss for level {}, generating content", level_id.0);

        let content = self
            .generator
            .generate_level_content(level_id, seed)
            .map_err(|e| {
                self.update_metrics_error();
                e
            })?;

        // Store in cache
        self.cache.insert(cache_key, content.clone()).await;

        let lookup_time = start_time.elapsed();
        self.update_metrics_miss(lookup_time);

        debug!(
            "Generated and cached content for level {} ({}μs)",
            level_id.0,
            lookup_time.as_micros()
        );

        Ok(content)
    }

    /// Get cached content only (no generation on miss)
    pub async fn get_cached_content(&self, level_id: LevelId, seed: u64) -> Option<String> {
        let cache_key = generate_cache_key(level_id, seed);
        self.cache.get(&cache_key).await
    }

    /// Preload content for upcoming levels based on strategy
    #[instrument(skip(self))]
    pub async fn preload_upcoming_levels(&self, current_level: LevelId) -> Result<()> {
        match &self.preload_strategy {
            PreloadStrategy::None => Ok(()),

            PreloadStrategy::Sequential(count) => {
                self.preload_sequential(current_level, *count).await
            }

            PreloadStrategy::Adaptive(levels) => self.preload_adaptive(levels.clone()).await,
        }
    }

    /// Preload next N levels sequentially
    async fn preload_sequential(&self, current_level: LevelId, count: u8) -> Result<()> {
        let mut preload_tasks = Vec::new();

        for i in 1..=count {
            let next_level = current_level.0 + i;
            if next_level > LevelId::MAX {
                break; // Don't exceed max level
            }

            let level_id = LevelId::new(next_level)?;
            let seed = self.generate_default_seed(level_id);
            let cache_key = generate_cache_key(level_id, seed);

            // Skip if already cached
            if self.cache.get(&cache_key).await.is_some() {
                continue;
            }

            // Acquire semaphore permit for concurrent limit
            let permit = self
                .preload_semaphore
                .clone()
                .acquire_owned()
                .await
                .map_err(|e| CentotypeError::Content(format!("Preload semaphore error: {}", e)))?;

            let generator = self.generator.clone();
            let cache = self.cache.clone();
            let metrics = self.metrics.clone();

            let task = tokio::spawn(async move {
                let _permit = permit; // Hold permit until task completes

                match generator.generate_level_content(level_id, seed) {
                    Ok(content) => {
                        cache.insert(cache_key, content).await;

                        // Update preload metrics
                        {
                            let mut m = metrics.write();
                            m.preload_count += 1;
                        }

                        debug!("Preloaded content for level {}", level_id.0);
                    }
                    Err(e) => {
                        warn!("Failed to preload level {}: {}", level_id.0, e);

                        // Update error metrics
                        {
                            let mut m = metrics.write();
                            m.error_count += 1;
                        }
                    }
                }
            });

            preload_tasks.push(task);
        }

        // Wait for all preload tasks to complete
        for task in preload_tasks {
            let _ = task.await; // Ignore join errors
        }

        Ok(())
    }

    /// Preload specific levels based on adaptive strategy
    async fn preload_adaptive(&self, levels: Vec<LevelId>) -> Result<()> {
        let mut preload_tasks = Vec::new();

        for level_id in levels {
            let seed = self.generate_default_seed(level_id);
            let cache_key = generate_cache_key(level_id, seed);

            // Skip if already cached
            if self.cache.get(&cache_key).await.is_some() {
                continue;
            }

            // Acquire semaphore permit
            let permit = self
                .preload_semaphore
                .clone()
                .acquire_owned()
                .await
                .map_err(|e| CentotypeError::Content(format!("Preload semaphore error: {}", e)))?;

            let generator = self.generator.clone();
            let cache = self.cache.clone();
            let metrics = self.metrics.clone();

            let task = tokio::spawn(async move {
                let _permit = permit;

                match generator.generate_level_content(level_id, seed) {
                    Ok(content) => {
                        cache.insert(cache_key, content).await;

                        {
                            let mut m = metrics.write();
                            m.preload_count += 1;
                        }

                        debug!("Preloaded content for level {}", level_id.0);
                    }
                    Err(e) => {
                        warn!("Failed to preload level {}: {}", level_id.0, e);

                        {
                            let mut m = metrics.write();
                            m.error_count += 1;
                        }
                    }
                }
            });

            preload_tasks.push(task);
        }

        // Wait for all adaptive preload tasks
        for task in preload_tasks {
            let _ = task.await;
        }

        Ok(())
    }

    /// Invalidate cache entry for a specific level
    pub async fn invalidate(&self, level_id: LevelId, seed: u64) {
        let cache_key = generate_cache_key(level_id, seed);
        self.cache.invalidate(&cache_key).await;
        debug!("Invalidated cache for level {}", level_id.0);
    }

    /// Clear entire cache
    pub async fn clear(&self) {
        self.cache.invalidate_all();
        self.cache.run_pending_tasks().await;
        {
            let mut metrics = self.metrics.write();
            metrics.reset();
        }
        debug!("Cleared entire content cache");
    }

    /// Get current cache metrics
    pub fn get_metrics(&self) -> CacheMetrics {
        let metrics = self.metrics.read();
        let mut result = metrics.clone();

        // Update memory usage from cache
        result.memory_usage_bytes = self.cache.weighted_size();

        result
    }

    /// Set preload strategy
    pub fn set_preload_strategy(&mut self, strategy: PreloadStrategy) {
        self.preload_strategy = strategy;
        debug!("Updated preload strategy: {:?}", self.preload_strategy);
    }

    /// Update hit metrics
    fn update_metrics_hit(&self, lookup_time: Duration) {
        let mut metrics = self.metrics.write();
        metrics.hit_count += 1;
        self.update_avg_lookup_time(&mut metrics, lookup_time);
    }

    /// Update miss metrics
    fn update_metrics_miss(&self, lookup_time: Duration) {
        let mut metrics = self.metrics.write();
        metrics.miss_count += 1;
        self.update_avg_lookup_time(&mut metrics, lookup_time);
    }

    /// Update error metrics
    fn update_metrics_error(&self) {
        let mut metrics = self.metrics.write();
        metrics.error_count += 1;
    }

    /// Update average lookup time with exponential moving average
    fn update_avg_lookup_time(&self, metrics: &mut CacheMetrics, lookup_time: Duration) {
        let new_time_micros = lookup_time.as_micros() as u64;

        if metrics.avg_lookup_time_micros == 0 {
            metrics.avg_lookup_time_micros = new_time_micros;
        } else {
            // Exponential moving average with α=0.1
            metrics.avg_lookup_time_micros =
                (metrics.avg_lookup_time_micros * 9 + new_time_micros) / 10;
        }
    }

    /// Generate default seed for a level (consistent across runs)
    fn generate_default_seed(&self, level_id: LevelId) -> u64 {
        use std::collections::hash_map::DefaultHasher;
        use std::hash::{Hash, Hasher};

        let mut hasher = DefaultHasher::new();
        level_id.hash(&mut hasher);
        // Add a constant to distinguish from user-provided seeds
        hasher.write_u64(0xCEF7074E_CAFE_BABE);
        hasher.finish()
    }
}

/// Cache manager for coordinating multiple cache instances
pub struct CacheManager {
    content_cache: Arc<ContentCache>,
}

impl CacheManager {
    /// Create new cache manager with content generator
    pub fn new(generator: Arc<CentotypeContentGenerator>) -> Result<Self> {
        let config = CacheConfig::default();
        let content_cache = Arc::new(ContentCache::new(generator, config));

        Ok(Self { content_cache })
    }

    /// Create cache manager with custom configuration
    pub fn with_config(
        generator: Arc<CentotypeContentGenerator>,
        config: CacheConfig,
    ) -> Result<Self> {
        let content_cache = Arc::new(ContentCache::new(generator, config));

        Ok(Self { content_cache })
    }

    /// Get content cache reference
    pub fn content_cache(&self) -> Arc<ContentCache> {
        self.content_cache.clone()
    }

    /// Get aggregated cache metrics
    pub fn get_aggregated_metrics(&self) -> CacheMetrics {
        self.content_cache.get_metrics()
    }

    /// Validate all cache performance targets
    pub fn validate_performance_targets(&self) -> Result<()> {
        let metrics = self.get_aggregated_metrics();
        metrics.validate_performance_targets()
    }

    /// Periodic maintenance task (should be called regularly)
    pub async fn run_maintenance(&self) {
        // Run pending cache tasks (cleanup, eviction, etc.)
        self.content_cache.cache.run_pending_tasks().await;

        // Update eviction count
        {
            let mut metrics = self.content_cache.metrics.write();
            // Moka doesn't directly expose eviction count, so we estimate it
            let current_size = self.content_cache.cache.entry_count();
            let max_capacity = self.content_cache.config.max_capacity;

            if current_size >= max_capacity {
                metrics.eviction_count += 1;
            }
        }

        debug!("Cache maintenance completed");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validation::ContentValidator;

    async fn create_test_cache() -> ContentCache {
        let validator = Arc::new(ContentValidator::new().unwrap());
        let generator = Arc::new(CentotypeContentGenerator::new(validator));
        let config = CacheConfig {
            max_capacity: 10,
            ttl: Duration::from_secs(60),
            tti: Duration::from_secs(30),
            max_preload_concurrent: 2,
        };

        ContentCache::new(generator, config)
    }

    #[tokio::test]
    async fn test_cache_hit_miss() {
        let cache = create_test_cache().await;
        let level = LevelId::new(1).unwrap();
        let seed = 12345;

        // First access should be a miss
        let content1 = cache.get_content(level, seed).await.unwrap();
        let metrics = cache.get_metrics();
        assert_eq!(metrics.miss_count, 1);
        assert_eq!(metrics.hit_count, 0);

        // Second access should be a hit
        let content2 = cache.get_content(level, seed).await.unwrap();
        let metrics = cache.get_metrics();
        assert_eq!(metrics.miss_count, 1);
        assert_eq!(metrics.hit_count, 1);

        // Content should be identical
        assert_eq!(content1, content2);
    }

    #[tokio::test]
    async fn test_cache_preloading() {
        let cache = create_test_cache().await;
        let level = LevelId::new(1).unwrap();

        // Preload upcoming levels
        cache.preload_upcoming_levels(level).await.unwrap();

        // Check if next levels are cached
        let level_2 = LevelId::new(2).unwrap();
        let seed_2 = cache.generate_default_seed(level_2);
        assert!(cache.get_cached_content(level_2, seed_2).await.is_some());

        let metrics = cache.get_metrics();
        assert!(metrics.preload_count > 0);
    }

    #[tokio::test]
    async fn test_cache_invalidation() {
        let cache = create_test_cache().await;
        let level = LevelId::new(1).unwrap();
        let seed = 12345;

        // Cache content
        let _content = cache.get_content(level, seed).await.unwrap();
        assert!(cache.get_cached_content(level, seed).await.is_some());

        // Invalidate
        cache.invalidate(level, seed).await;
        assert!(cache.get_cached_content(level, seed).await.is_none());
    }

    #[tokio::test]
    async fn test_cache_metrics() {
        let cache = create_test_cache().await;
        let level = LevelId::new(1).unwrap();
        let seed = 12345;

        // Generate some cache activity
        let _content1 = cache.get_content(level, seed).await.unwrap();
        let _content2 = cache.get_content(level, seed).await.unwrap();

        let metrics = cache.get_metrics();
        assert_eq!(metrics.total_requests, 2);
        assert_eq!(metrics.hit_count, 1);
        assert_eq!(metrics.miss_count, 1);
        assert_eq!(metrics.hit_rate(), 50.0);
        assert!(metrics.avg_lookup_time_micros > 0);
    }

    #[tokio::test]
    async fn test_cache_manager() {
        let validator = Arc::new(ContentValidator::new().unwrap());
        let generator = Arc::new(CentotypeContentGenerator::new(validator));
        let manager = CacheManager::new(generator).unwrap();

        let level = LevelId::new(1).unwrap();
        let seed = 12345;

        let cache = manager.content_cache();
        let _content = cache.get_content(level, seed).await.unwrap();

        let metrics = manager.get_aggregated_metrics();
        assert!(metrics.total_requests > 0);
    }
}
