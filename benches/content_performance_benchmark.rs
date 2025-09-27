//! # Content System Performance Benchmark Suite
//!
//! Comprehensive benchmarking framework for validating content loading P99 <25ms target.
//! This benchmark suite provides precise measurement of content generation, caching,
//! and retrieval performance across all Centotype content management operations.
//!
//! ## Key Performance Targets
//!
//! - **Content Loading P99**: <25ms (including cache)
//! - **Cache Lookup Time**: <5ms (direct cache access)
//! - **Content Generation P95**: <50ms (cold generation)
//! - **Cache Hit Rate**: >90% (preloading effectiveness)
//! - **Memory Efficiency**: <1MB per cached level
//!
//! ## Benchmark Categories
//!
//! 1. **Cold Content Generation**: Level creation from scratch
//! 2. **Cache Performance**: Hit rates and lookup latency
//! 3. **Preloading Effectiveness**: Background content preparation
//! 4. **Large Level Loading**: 3000+ character performance
//! 5. **Concurrent Content Access**: Multi-threaded retrieval
//! 6. **Cache Eviction Performance**: Memory management efficiency
//! 7. **Content Validation Speed**: Text corpus verification

use centotype_content::{ContentManager, ContentConfig, CacheMetrics, LevelContent};
use centotype_core::{CentotypeCore, types::*};
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;
use tokio::sync::RwLock;
use tracing::{debug, info, warn, instrument};

/// Content performance benchmark configuration
#[derive(Debug, Clone)]
pub struct ContentBenchmarkConfig {
    /// Number of content loading samples
    pub content_sample_count: usize,
    /// Target P99 content loading time
    pub p99_target: Duration,
    /// Target cache lookup time
    pub cache_lookup_target: Duration,
    /// Target content generation time
    pub generation_target: Duration,
    /// Target cache hit rate percentage
    pub cache_hit_rate_target: f64,
    /// Large content size for stress testing
    pub large_content_size: usize,
    /// Concurrent access thread count
    pub concurrent_threads: usize,
    /// Cache capacity for testing
    pub cache_capacity: usize,
    /// Preloading test range
    pub preload_range: std::ops::Range<u8>,
}

impl Default for ContentBenchmarkConfig {
    fn default() -> Self {
        Self {
            content_sample_count: 5000,
            p99_target: Duration::from_millis(25),
            cache_lookup_target: Duration::from_millis(5),
            generation_target: Duration::from_millis(50),
            cache_hit_rate_target: 90.0,
            large_content_size: 3000,
            concurrent_threads: 16,
            cache_capacity: 100,
            preload_range: 1..21,
        }
    }
}

/// Content performance measurement result
#[derive(Debug, Clone)]
pub struct ContentPerformanceMeasurement {
    /// Content loading time samples
    pub loading_times: Vec<Duration>,
    /// Cache operation time samples
    pub cache_times: Vec<Duration>,
    /// Content performance statistics
    pub statistics: ContentStatistics,
    /// Performance target compliance
    pub target_compliance: ContentTargetCompliance,
    /// Cache performance metrics
    pub cache_metrics: CachePerformanceMetrics,
    /// Identified bottlenecks
    pub bottlenecks: Vec<ContentBottleneck>,
    /// Test metadata
    pub metadata: ContentBenchmarkMetadata,
}

/// Statistical analysis of content performance
#[derive(Debug, Clone)]
pub struct ContentStatistics {
    pub mean_loading_time: Duration,
    pub median_loading_time: Duration,
    pub p95_loading_time: Duration,
    pub p99_loading_time: Duration,
    pub min_loading_time: Duration,
    pub max_loading_time: Duration,
    pub loading_time_std_dev: f64,
    pub cache_hit_rate: f64,
    pub cache_miss_rate: f64,
    pub avg_cache_lookup_time: Duration,
    pub content_generation_rate: f64, // levels per second
    pub memory_efficiency_score: f64, // 0.0 to 1.0
    pub outlier_count: usize,
}

/// Content performance target compliance
#[derive(Debug, Clone)]
pub struct ContentTargetCompliance {
    pub p99_loading_compliant: bool,
    pub cache_lookup_compliant: bool,
    pub generation_speed_compliant: bool,
    pub cache_hit_rate_compliant: bool,
    pub memory_efficiency_compliant: bool,
    pub overall_compliant: bool,
    pub margin_p99_loading: i64, // Microseconds margin
    pub margin_cache_lookup: i64,
    pub margin_hit_rate: f64, // Percentage margin
}

/// Content system bottleneck identification
#[derive(Debug, Clone)]
pub struct ContentBottleneck {
    pub component: String,
    pub operation_phase: ContentPhase,
    pub avg_time: Duration,
    pub impact_percentage: f64,
    pub frequency: f64,
    pub suggested_optimization: String,
}

/// Content operation phases for analysis
#[derive(Debug, Clone)]
pub enum ContentPhase {
    CacheLookup,
    ContentGeneration,
    TextProcessing,
    ValidationCheck,
    CacheInsertion,
    MemoryAllocation,
    Serialization,
    Preloading,
}

/// Cache performance metrics
#[derive(Debug, Clone)]
pub struct CachePerformanceMetrics {
    pub hit_count: u64,
    pub miss_count: u64,
    pub hit_rate: f64,
    pub miss_rate: f64,
    pub lookup_latency_p50: Duration,
    pub lookup_latency_p95: Duration,
    pub lookup_latency_p99: Duration,
    pub cache_size_bytes: usize,
    pub eviction_count: u64,
    pub memory_usage_efficiency: f64,
}

/// Content benchmark execution metadata
#[derive(Debug, Clone)]
pub struct ContentBenchmarkMetadata {
    pub test_name: String,
    pub execution_timestamp: Instant,
    pub config: ContentBenchmarkConfig,
    pub content_manager_config: ContentManagerConfig,
    pub level_complexity_distribution: LevelComplexityDistribution,
    pub warmup_duration: Duration,
    pub actual_sample_count: usize,
}

/// Content manager configuration for benchmarking
#[derive(Debug, Clone)]
pub struct ContentManagerConfig {
    pub cache_capacity: usize,
    pub preload_enabled: bool,
    pub generation_algorithm: String,
    pub validation_enabled: bool,
}

/// Distribution of content complexity in test
#[derive(Debug, Clone)]
pub struct LevelComplexityDistribution {
    pub simple_levels: usize,    // <500 chars
    pub medium_levels: usize,    // 500-1500 chars
    pub complex_levels: usize,   // 1500-3000 chars
    pub large_levels: usize,     // >3000 chars
}

/// Content performance benchmark harness
pub struct ContentPerformanceBenchmarkHarness {
    config: ContentBenchmarkConfig,
    content_manager: Arc<ContentManager>,
    runtime: Runtime,
}

impl ContentPerformanceBenchmarkHarness {
    /// Create new content benchmark harness
    pub async fn new(config: ContentBenchmarkConfig) -> Result<Self> {
        let runtime = Runtime::new().map_err(|e| CentotypeError::Internal(format!("Failed to create runtime: {}", e)))?;

        // Configure content manager for benchmarking
        let content_config = ContentConfig {
            cache_capacity: config.cache_capacity,
            preload_enabled: true,
            validation_enabled: true,
            generation_seed: Some(42), // Deterministic for benchmarking
        };

        let content_manager = Arc::new(ContentManager::with_config(content_config).await?);

        Ok(Self {
            config,
            content_manager,
            runtime,
        })
    }

    /// Run comprehensive content performance benchmark suite
    #[instrument(skip(self))]
    pub async fn run_comprehensive_content_benchmark(&self) -> Result<Vec<ContentPerformanceMeasurement>> {
        info!("Starting comprehensive content performance benchmark suite");

        let mut results = Vec::new();

        // 1. Cold content generation benchmark
        let cold_generation = self.benchmark_cold_content_generation().await?;
        results.push(cold_generation);

        // 2. Cache performance benchmark
        let cache_performance = self.benchmark_cache_performance().await?;
        results.push(cache_performance);

        // 3. Preloading effectiveness benchmark
        let preloading = self.benchmark_preloading_effectiveness().await?;
        results.push(preloading);

        // 4. Large content loading benchmark
        let large_content = self.benchmark_large_content_loading().await?;
        results.push(large_content);

        // 5. Concurrent content access benchmark
        let concurrent_access = self.benchmark_concurrent_content_access().await?;
        results.push(concurrent_access);

        // 6. Cache eviction performance benchmark
        let cache_eviction = self.benchmark_cache_eviction_performance().await?;
        results.push(cache_eviction);

        // 7. Content validation speed benchmark
        let validation_speed = self.benchmark_content_validation_speed().await?;
        results.push(validation_speed);

        info!("Completed comprehensive content performance benchmark suite with {} test categories", results.len());
        Ok(results)
    }

    /// Benchmark cold content generation performance
    #[instrument(skip(self))]
    async fn benchmark_cold_content_generation(&self) -> Result<ContentPerformanceMeasurement> {
        info!("Benchmarking cold content generation");

        // Clear cache to ensure cold generation
        self.content_manager.clear_cache().await;

        let mut loading_times = Vec::with_capacity(self.config.content_sample_count);
        let mut cache_times = Vec::new();
        let warmup_start = Instant::now();

        // Warmup phase
        for level_num in 1..=10 {
            let level_id = LevelId::new(level_num)?;
            let _ = self.content_manager.get_level_content(level_id, None).await?;
        }
        self.content_manager.clear_cache().await;
        let warmup_duration = warmup_start.elapsed();

        // Main measurement phase
        for i in 0..self.config.content_sample_count {
            let level_num = ((i % 100) + 1) as u8; // Cycle through levels 1-100
            let level_id = LevelId::new(level_num)?;

            let loading_start = Instant::now();
            let _content = self.content_manager.get_level_content(level_id, None).await?;
            let loading_time = loading_start.elapsed();
            loading_times.push(loading_time);

            if i % 500 == 0 && i > 0 {
                debug!("Cold generation benchmark progress: {}/{}", i, self.config.content_sample_count);
            }

            // Clear cache periodically to maintain cold generation
            if i % 50 == 0 {
                self.content_manager.clear_cache().await;
            }
        }

        let statistics = Self::calculate_content_statistics(&loading_times, &cache_times, &self.content_manager.get_cache_metrics());
        let target_compliance = self.assess_content_target_compliance(&statistics);
        let cache_metrics = self.extract_cache_performance_metrics().await;
        let bottlenecks = self.identify_content_bottlenecks(&loading_times, ContentPhase::ContentGeneration).await;

        let metadata = ContentBenchmarkMetadata {
            test_name: "cold_content_generation".to_string(),
            execution_timestamp: Instant::now(),
            config: self.config.clone(),
            content_manager_config: ContentManagerConfig {
                cache_capacity: self.config.cache_capacity,
                preload_enabled: true,
                generation_algorithm: "default".to_string(),
                validation_enabled: true,
            },
            level_complexity_distribution: self.analyze_level_complexity_distribution().await,
            warmup_duration,
            actual_sample_count: loading_times.len(),
        };

        Ok(ContentPerformanceMeasurement {
            loading_times,
            cache_times,
            statistics,
            target_compliance,
            cache_metrics,
            bottlenecks,
            metadata,
        })
    }

    /// Benchmark cache performance (hit rates and lookup latency)
    #[instrument(skip(self))]
    async fn benchmark_cache_performance(&self) -> Result<ContentPerformanceMeasurement> {
        info!("Benchmarking cache performance");

        self.content_manager.clear_cache().await;

        let mut loading_times = Vec::new();
        let mut cache_times = Vec::with_capacity(self.config.content_sample_count);

        // Pre-populate cache with test content
        for level_num in self.config.preload_range.clone() {
            let level_id = LevelId::new(level_num)?;
            let _ = self.content_manager.get_level_content(level_id, None).await?;
        }

        // Measure cache lookup performance
        for i in 0..self.config.content_sample_count {
            let level_num = (i % (self.config.preload_range.end - self.config.preload_range.start) as usize + self.config.preload_range.start as usize) as u8;
            let level_id = LevelId::new(level_num)?;

            let cache_start = Instant::now();
            let _cached_content = self.content_manager.get_cached_content(level_id, None).await;
            let cache_time = cache_start.elapsed();
            cache_times.push(cache_time);

            if i % 500 == 0 && i > 0 {
                debug!("Cache performance benchmark progress: {}/{}", i, self.config.content_sample_count);
            }
        }

        let statistics = Self::calculate_content_statistics(&loading_times, &cache_times, &self.content_manager.get_cache_metrics());
        let target_compliance = self.assess_content_target_compliance(&statistics);
        let cache_metrics = self.extract_cache_performance_metrics().await;
        let bottlenecks = self.identify_content_bottlenecks(&cache_times, ContentPhase::CacheLookup).await;

        let metadata = ContentBenchmarkMetadata {
            test_name: "cache_performance".to_string(),
            execution_timestamp: Instant::now(),
            config: self.config.clone(),
            content_manager_config: ContentManagerConfig {
                cache_capacity: self.config.cache_capacity,
                preload_enabled: true,
                generation_algorithm: "default".to_string(),
                validation_enabled: true,
            },
            level_complexity_distribution: self.analyze_level_complexity_distribution().await,
            warmup_duration: Duration::ZERO,
            actual_sample_count: cache_times.len(),
        };

        Ok(ContentPerformanceMeasurement {
            loading_times,
            cache_times,
            statistics,
            target_compliance,
            cache_metrics,
            bottlenecks,
            metadata,
        })
    }

    /// Benchmark preloading effectiveness
    #[instrument(skip(self))]
    async fn benchmark_preloading_effectiveness(&self) -> Result<ContentPerformanceMeasurement> {
        info!("Benchmarking preloading effectiveness");

        self.content_manager.clear_cache().await;

        let mut loading_times = Vec::new();
        let mut cache_times = Vec::new();

        // Test preloading performance
        for base_level in 1..=50 {
            let level_id = LevelId::new(base_level)?;

            // Measure preloading time
            let preload_start = Instant::now();
            self.content_manager.preload_upcoming_levels(level_id).await?;
            let preload_time = preload_start.elapsed();
            loading_times.push(preload_time);

            // Measure subsequent access time for preloaded content
            for offset in 1..=5 {
                if base_level + offset <= 100 {
                    let next_level = LevelId::new(base_level + offset)?;
                    let access_start = Instant::now();
                    let _content = self.content_manager.get_level_content(next_level, None).await?;
                    let access_time = access_start.elapsed();
                    cache_times.push(access_time);
                }
            }

            if base_level % 10 == 0 {
                debug!("Preloading benchmark progress: level {}/50", base_level);
            }
        }

        let statistics = Self::calculate_content_statistics(&loading_times, &cache_times, &self.content_manager.get_cache_metrics());
        let target_compliance = self.assess_content_target_compliance(&statistics);
        let cache_metrics = self.extract_cache_performance_metrics().await;
        let bottlenecks = self.identify_content_bottlenecks(&loading_times, ContentPhase::Preloading).await;

        let metadata = ContentBenchmarkMetadata {
            test_name: "preloading_effectiveness".to_string(),
            execution_timestamp: Instant::now(),
            config: self.config.clone(),
            content_manager_config: ContentManagerConfig {
                cache_capacity: self.config.cache_capacity,
                preload_enabled: true,
                generation_algorithm: "default".to_string(),
                validation_enabled: true,
            },
            level_complexity_distribution: self.analyze_level_complexity_distribution().await,
            warmup_duration: Duration::ZERO,
            actual_sample_count: loading_times.len() + cache_times.len(),
        };

        Ok(ContentPerformanceMeasurement {
            loading_times,
            cache_times,
            statistics,
            target_compliance,
            cache_metrics,
            bottlenecks,
            metadata,
        })
    }

    /// Benchmark large content loading performance
    #[instrument(skip(self))]
    async fn benchmark_large_content_loading(&self) -> Result<ContentPerformanceMeasurement> {
        info!("Benchmarking large content loading ({}+ characters)", self.config.large_content_size);

        self.content_manager.clear_cache().await;

        let mut loading_times = Vec::with_capacity(self.config.content_sample_count / 10);
        let cache_times = Vec::new();

        // Test large content levels (typically higher level numbers)
        for i in 0..(self.config.content_sample_count / 10) {
            let level_num = (80 + (i % 20)) as u8; // Levels 80-99 are typically larger
            let level_id = LevelId::new(level_num)?;

            let loading_start = Instant::now();
            let content = self.content_manager.get_level_content(level_id, None).await?;
            let loading_time = loading_start.elapsed();

            // Only count if content is actually large
            if content.content.len() >= self.config.large_content_size {
                loading_times.push(loading_time);
            }

            if i % 50 == 0 && i > 0 {
                debug!("Large content benchmark progress: {}/{}", i, self.config.content_sample_count / 10);
            }
        }

        let statistics = Self::calculate_content_statistics(&loading_times, &cache_times, &self.content_manager.get_cache_metrics());
        let target_compliance = self.assess_content_target_compliance(&statistics);
        let cache_metrics = self.extract_cache_performance_metrics().await;
        let bottlenecks = self.identify_content_bottlenecks(&loading_times, ContentPhase::TextProcessing).await;

        let metadata = ContentBenchmarkMetadata {
            test_name: "large_content_loading".to_string(),
            execution_timestamp: Instant::now(),
            config: self.config.clone(),
            content_manager_config: ContentManagerConfig {
                cache_capacity: self.config.cache_capacity,
                preload_enabled: true,
                generation_algorithm: "default".to_string(),
                validation_enabled: true,
            },
            level_complexity_distribution: self.analyze_level_complexity_distribution().await,
            warmup_duration: Duration::ZERO,
            actual_sample_count: loading_times.len(),
        };

        Ok(ContentPerformanceMeasurement {
            loading_times,
            cache_times,
            statistics,
            target_compliance,
            cache_metrics,
            bottlenecks,
            metadata,
        })
    }

    /// Benchmark concurrent content access performance
    #[instrument(skip(self))]
    async fn benchmark_concurrent_content_access(&self) -> Result<ContentPerformanceMeasurement> {
        info!("Benchmarking concurrent content access with {} threads", self.config.concurrent_threads);

        self.content_manager.clear_cache().await;

        let loading_times = Arc::new(RwLock::new(Vec::new()));
        let cache_times = Arc::new(RwLock::new(Vec::new()));
        let mut handles = Vec::new();

        let operations_per_thread = self.config.content_sample_count / self.config.concurrent_threads;

        for thread_id in 0..self.config.concurrent_threads {
            let loading_times_clone = loading_times.clone();
            let cache_times_clone = cache_times.clone();
            let content_manager_clone = self.content_manager.clone();

            let handle = tokio::spawn(async move {
                for i in 0..operations_per_thread {
                    let level_num = ((thread_id * operations_per_thread + i) % 100 + 1) as u8;
                    let level_id = LevelId::new(level_num).unwrap();

                    let loading_start = Instant::now();
                    let _content = content_manager_clone.get_level_content(level_id, None).await.unwrap();
                    let loading_time = loading_start.elapsed();

                    loading_times_clone.write().await.push(loading_time);

                    // Also test cache lookup if content is likely cached
                    if i > 50 {
                        let cache_start = Instant::now();
                        let _cached = content_manager_clone.get_cached_content(level_id, None).await;
                        let cache_time = cache_start.elapsed();
                        cache_times_clone.write().await.push(cache_time);
                    }
                }
            });

            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            let _ = handle.await;
        }

        let loading_times_vec = loading_times.read().await.clone();
        let cache_times_vec = cache_times.read().await.clone();

        let statistics = Self::calculate_content_statistics(&loading_times_vec, &cache_times_vec, &self.content_manager.get_cache_metrics());
        let target_compliance = self.assess_content_target_compliance(&statistics);
        let cache_metrics = self.extract_cache_performance_metrics().await;
        let bottlenecks = self.identify_content_bottlenecks(&loading_times_vec, ContentPhase::ContentGeneration).await;

        let metadata = ContentBenchmarkMetadata {
            test_name: format!("concurrent_access_{}threads", self.config.concurrent_threads),
            execution_timestamp: Instant::now(),
            config: self.config.clone(),
            content_manager_config: ContentManagerConfig {
                cache_capacity: self.config.cache_capacity,
                preload_enabled: true,
                generation_algorithm: "default".to_string(),
                validation_enabled: true,
            },
            level_complexity_distribution: self.analyze_level_complexity_distribution().await,
            warmup_duration: Duration::ZERO,
            actual_sample_count: loading_times_vec.len() + cache_times_vec.len(),
        };

        Ok(ContentPerformanceMeasurement {
            loading_times: loading_times_vec,
            cache_times: cache_times_vec,
            statistics,
            target_compliance,
            cache_metrics,
            bottlenecks,
            metadata,
        })
    }

    /// Benchmark cache eviction performance
    #[instrument(skip(self))]
    async fn benchmark_cache_eviction_performance(&self) -> Result<ContentPerformanceMeasurement> {
        info!("Benchmarking cache eviction performance");

        self.content_manager.clear_cache().await;

        let mut loading_times = Vec::new();
        let cache_times = Vec::new();

        // Fill cache beyond capacity to trigger evictions
        let levels_to_load = self.config.cache_capacity * 2; // Load 2x cache capacity
        for i in 0..levels_to_load {
            let level_num = (i % 100 + 1) as u8;
            let level_id = LevelId::new(level_num)?;

            let loading_start = Instant::now();
            let _content = self.content_manager.get_level_content(level_id, None).await?;
            let loading_time = loading_start.elapsed();
            loading_times.push(loading_time);

            if i % 50 == 0 && i > 0 {
                debug!("Cache eviction benchmark progress: {}/{}", i, levels_to_load);
            }
        }

        let statistics = Self::calculate_content_statistics(&loading_times, &cache_times, &self.content_manager.get_cache_metrics());
        let target_compliance = self.assess_content_target_compliance(&statistics);
        let cache_metrics = self.extract_cache_performance_metrics().await;
        let bottlenecks = self.identify_content_bottlenecks(&loading_times, ContentPhase::CacheInsertion).await;

        let metadata = ContentBenchmarkMetadata {
            test_name: "cache_eviction_performance".to_string(),
            execution_timestamp: Instant::now(),
            config: self.config.clone(),
            content_manager_config: ContentManagerConfig {
                cache_capacity: self.config.cache_capacity,
                preload_enabled: true,
                generation_algorithm: "default".to_string(),
                validation_enabled: true,
            },
            level_complexity_distribution: self.analyze_level_complexity_distribution().await,
            warmup_duration: Duration::ZERO,
            actual_sample_count: loading_times.len(),
        };

        Ok(ContentPerformanceMeasurement {
            loading_times,
            cache_times,
            statistics,
            target_compliance,
            cache_metrics,
            bottlenecks,
            metadata,
        })
    }

    /// Benchmark content validation speed
    #[instrument(skip(self))]
    async fn benchmark_content_validation_speed(&self) -> Result<ContentPerformanceMeasurement> {
        info!("Benchmarking content validation speed");

        let mut loading_times = Vec::with_capacity(self.config.content_sample_count);
        let cache_times = Vec::new();

        // Generate content for validation testing
        for i in 0..self.config.content_sample_count {
            let level_num = (i % 100 + 1) as u8;
            let level_id = LevelId::new(level_num)?;

            let validation_start = Instant::now();
            let content = self.content_manager.get_level_content(level_id, None).await?;
            let _validation_result = self.validate_content_quality(&content).await;
            let validation_time = validation_start.elapsed();
            loading_times.push(validation_time);

            if i % 500 == 0 && i > 0 {
                debug!("Content validation benchmark progress: {}/{}", i, self.config.content_sample_count);
            }
        }

        let statistics = Self::calculate_content_statistics(&loading_times, &cache_times, &self.content_manager.get_cache_metrics());
        let target_compliance = self.assess_content_target_compliance(&statistics);
        let cache_metrics = self.extract_cache_performance_metrics().await;
        let bottlenecks = self.identify_content_bottlenecks(&loading_times, ContentPhase::ValidationCheck).await;

        let metadata = ContentBenchmarkMetadata {
            test_name: "content_validation_speed".to_string(),
            execution_timestamp: Instant::now(),
            config: self.config.clone(),
            content_manager_config: ContentManagerConfig {
                cache_capacity: self.config.cache_capacity,
                preload_enabled: true,
                generation_algorithm: "default".to_string(),
                validation_enabled: true,
            },
            level_complexity_distribution: self.analyze_level_complexity_distribution().await,
            warmup_duration: Duration::ZERO,
            actual_sample_count: loading_times.len(),
        };

        Ok(ContentPerformanceMeasurement {
            loading_times,
            cache_times,
            statistics,
            target_compliance,
            cache_metrics,
            bottlenecks,
            metadata,
        })
    }

    /// Calculate comprehensive content performance statistics
    fn calculate_content_statistics(loading_times: &[Duration], cache_times: &[Duration], cache_metrics: &CacheMetrics) -> ContentStatistics {
        let mean_loading_time = if loading_times.is_empty() {
            Duration::ZERO
        } else {
            loading_times.iter().sum::<Duration>() / loading_times.len() as u32
        };

        let mut sorted_loading = loading_times.to_vec();
        sorted_loading.sort();

        let len = sorted_loading.len();
        let median_loading_time = if len > 0 { sorted_loading[len / 2] } else { Duration::ZERO };
        let p95_loading_time = if len > 0 { sorted_loading[len * 95 / 100] } else { Duration::ZERO };
        let p99_loading_time = if len > 0 { sorted_loading[len * 99 / 100] } else { Duration::ZERO };
        let min_loading_time = if len > 0 { sorted_loading[0] } else { Duration::ZERO };
        let max_loading_time = if len > 0 { sorted_loading[len - 1] } else { Duration::ZERO };

        // Calculate standard deviation
        let mean_nanos = mean_loading_time.as_nanos() as f64;
        let variance: f64 = loading_times.iter()
            .map(|&duration| {
                let diff = duration.as_nanos() as f64 - mean_nanos;
                diff * diff
            })
            .sum::<f64>() / loading_times.len().max(1) as f64;
        let loading_time_std_dev = variance.sqrt();

        let cache_hit_rate = cache_metrics.hit_rate();
        let cache_miss_rate = 100.0 - cache_hit_rate;

        let avg_cache_lookup_time = if cache_times.is_empty() {
            Duration::ZERO
        } else {
            cache_times.iter().sum::<Duration>() / cache_times.len() as u32
        };

        let content_generation_rate = if mean_loading_time.as_secs_f64() > 0.0 {
            1.0 / mean_loading_time.as_secs_f64()
        } else {
            0.0
        };

        let memory_efficiency_score = if cache_metrics.memory_usage_bytes > 0 {
            (cache_metrics.hit_count as f64 * 1000.0) / cache_metrics.memory_usage_bytes as f64
        } else {
            0.0
        };

        // Count outliers (more than 2 standard deviations from mean)
        let outlier_threshold = 2.0 * loading_time_std_dev;
        let outlier_count = loading_times.iter()
            .filter(|&&duration| {
                let diff = (duration.as_nanos() as f64 - mean_nanos).abs();
                diff > outlier_threshold
            })
            .count();

        ContentStatistics {
            mean_loading_time,
            median_loading_time,
            p95_loading_time,
            p99_loading_time,
            min_loading_time,
            max_loading_time,
            loading_time_std_dev,
            cache_hit_rate,
            cache_miss_rate,
            avg_cache_lookup_time,
            content_generation_rate,
            memory_efficiency_score,
            outlier_count,
        }
    }

    /// Assess content performance target compliance
    fn assess_content_target_compliance(&self, statistics: &ContentStatistics) -> ContentTargetCompliance {
        let p99_loading_compliant = statistics.p99_loading_time <= self.config.p99_target;
        let cache_lookup_compliant = statistics.avg_cache_lookup_time <= self.config.cache_lookup_target;
        let generation_speed_compliant = statistics.p95_loading_time <= self.config.generation_target;
        let cache_hit_rate_compliant = statistics.cache_hit_rate >= self.config.cache_hit_rate_target;
        let memory_efficiency_compliant = statistics.memory_efficiency_score > 0.1; // Simplified threshold

        let overall_compliant = p99_loading_compliant && cache_lookup_compliant &&
                               generation_speed_compliant && cache_hit_rate_compliant &&
                               memory_efficiency_compliant;

        let margin_p99_loading = self.config.p99_target.as_micros() as i64 - statistics.p99_loading_time.as_micros() as i64;
        let margin_cache_lookup = self.config.cache_lookup_target.as_micros() as i64 - statistics.avg_cache_lookup_time.as_micros() as i64;
        let margin_hit_rate = statistics.cache_hit_rate - self.config.cache_hit_rate_target;

        ContentTargetCompliance {
            p99_loading_compliant,
            cache_lookup_compliant,
            generation_speed_compliant,
            cache_hit_rate_compliant,
            memory_efficiency_compliant,
            overall_compliant,
            margin_p99_loading,
            margin_cache_lookup,
            margin_hit_rate,
        }
    }

    /// Identify content system bottlenecks
    async fn identify_content_bottlenecks(&self, times: &[Duration], phase: ContentPhase) -> Vec<ContentBottleneck> {
        let mut bottlenecks = Vec::new();

        if times.is_empty() {
            return bottlenecks;
        }

        let mut sorted_times = times.to_vec();
        sorted_times.sort();
        let p95_time = sorted_times[sorted_times.len() * 95 / 100];

        let threshold = match phase {
            ContentPhase::CacheLookup => self.config.cache_lookup_target,
            ContentPhase::ContentGeneration => self.config.generation_target,
            _ => self.config.p99_target,
        };

        if p95_time > threshold {
            let impact_percentage = ((p95_time.as_micros() as f64 / threshold.as_micros() as f64) - 1.0) * 100.0;

            bottlenecks.push(ContentBottleneck {
                component: format!("{:?}", phase),
                operation_phase: phase.clone(),
                avg_time: times.iter().sum::<Duration>() / times.len() as u32,
                impact_percentage,
                frequency: 1.0,
                suggested_optimization: Self::get_content_optimization_suggestion(&phase),
            });
        }

        bottlenecks
    }

    /// Get optimization suggestions for content phases
    fn get_content_optimization_suggestion(phase: &ContentPhase) -> String {
        match phase {
            ContentPhase::CacheLookup => "Optimize cache lookup with better hash algorithms and reduce lock contention".to_string(),
            ContentPhase::ContentGeneration => "Implement parallel content generation and optimize text processing algorithms".to_string(),
            ContentPhase::TextProcessing => "Use SIMD instructions for text processing and reduce string allocations".to_string(),
            ContentPhase::ValidationCheck => "Implement incremental validation and cache validation results".to_string(),
            ContentPhase::CacheInsertion => "Optimize cache insertion with batch operations and better memory management".to_string(),
            ContentPhase::MemoryAllocation => "Use object pooling and reduce heap allocations during content generation".to_string(),
            ContentPhase::Serialization => "Implement zero-copy serialization and use more efficient formats".to_string(),
            ContentPhase::Preloading => "Implement smarter preloading strategies based on user patterns".to_string(),
        }
    }

    /// Extract cache performance metrics
    async fn extract_cache_performance_metrics(&self) -> CachePerformanceMetrics {
        let cache_metrics = self.content_manager.get_cache_metrics();

        // Simulate cache lookup latency measurements
        let mut lookup_times = Vec::new();
        for _ in 0..100 {
            let start = Instant::now();
            let _ = self.content_manager.get_cached_content(LevelId::new(1).unwrap(), None).await;
            lookup_times.push(start.elapsed());
        }

        lookup_times.sort();
        let len = lookup_times.len();

        CachePerformanceMetrics {
            hit_count: cache_metrics.hit_count,
            miss_count: cache_metrics.miss_count,
            hit_rate: cache_metrics.hit_rate(),
            miss_rate: 100.0 - cache_metrics.hit_rate(),
            lookup_latency_p50: lookup_times[len / 2],
            lookup_latency_p95: lookup_times[len * 95 / 100],
            lookup_latency_p99: lookup_times[len * 99 / 100],
            cache_size_bytes: cache_metrics.memory_usage_bytes,
            eviction_count: cache_metrics.eviction_count,
            memory_usage_efficiency: cache_metrics.hit_rate() / 100.0, // Simplified
        }
    }

    /// Analyze level complexity distribution
    async fn analyze_level_complexity_distribution(&self) -> LevelComplexityDistribution {
        let mut simple_levels = 0;
        let mut medium_levels = 0;
        let mut complex_levels = 0;
        let mut large_levels = 0;

        // Sample a subset of levels to analyze complexity
        for level_num in 1..=20 {
            if let Ok(level_id) = LevelId::new(level_num) {
                if let Ok(content) = self.content_manager.get_level_content(level_id, None).await {
                    let char_count = content.content.len();
                    match char_count {
                        0..=500 => simple_levels += 1,
                        501..=1500 => medium_levels += 1,
                        1501..=3000 => complex_levels += 1,
                        _ => large_levels += 1,
                    }
                }
            }
        }

        LevelComplexityDistribution {
            simple_levels,
            medium_levels,
            complex_levels,
            large_levels,
        }
    }

    /// Validate content quality (simulation)
    async fn validate_content_quality(&self, _content: &LevelContent) -> bool {
        // Simulate content validation latency
        tokio::time::sleep(Duration::from_micros(200)).await;
        true // Always pass for simulation
    }

    /// Generate comprehensive content benchmark report
    pub fn generate_content_benchmark_report(&self, measurements: &[ContentPerformanceMeasurement]) -> ContentPerformanceBenchmarkReport {
        let overall_compliance = measurements.iter().all(|m| m.target_compliance.overall_compliant);

        let avg_p99_loading_time = measurements.iter()
            .map(|m| m.statistics.p99_loading_time)
            .sum::<Duration>() / measurements.len() as u32;

        let avg_cache_hit_rate = measurements.iter()
            .map(|m| m.statistics.cache_hit_rate)
            .sum::<f64>() / measurements.len() as f64;

        let worst_case_p99 = measurements.iter()
            .map(|m| m.statistics.p99_loading_time)
            .max()
            .unwrap_or(Duration::ZERO);

        let all_bottlenecks: Vec<_> = measurements.iter()
            .flat_map(|m| &m.bottlenecks)
            .cloned()
            .collect();

        ContentPerformanceBenchmarkReport {
            overall_compliance,
            avg_p99_loading_time,
            avg_cache_hit_rate,
            worst_case_p99,
            target_margin_p99: self.config.p99_target.as_micros() as i64 - avg_p99_loading_time.as_micros() as i64,
            measurements: measurements.to_vec(),
            identified_bottlenecks: all_bottlenecks,
            optimization_recommendations: self.generate_content_optimization_recommendations(measurements),
            performance_grade: self.calculate_content_performance_grade(measurements),
        }
    }

    /// Generate content optimization recommendations
    fn generate_content_optimization_recommendations(&self, measurements: &[ContentPerformanceMeasurement]) -> Vec<String> {
        let mut recommendations = Vec::new();

        let avg_p99 = measurements.iter()
            .map(|m| m.statistics.p99_loading_time)
            .sum::<Duration>() / measurements.len() as u32;

        if avg_p99 > self.config.p99_target {
            recommendations.push("P99 content loading time exceeds target - optimize content generation pipeline".to_string());
        }

        let avg_hit_rate = measurements.iter()
            .map(|m| m.statistics.cache_hit_rate)
            .sum::<f64>() / measurements.len() as f64;

        if avg_hit_rate < self.config.cache_hit_rate_target {
            recommendations.push("Cache hit rate below target - improve preloading strategies and cache policies".to_string());
        }

        let avg_cache_lookup_time = measurements.iter()
            .map(|m| m.statistics.avg_cache_lookup_time)
            .sum::<Duration>() / measurements.len() as u32;

        if avg_cache_lookup_time > self.config.cache_lookup_target {
            recommendations.push("Cache lookup time too high - optimize cache data structures and reduce contention".to_string());
        }

        recommendations
    }

    /// Calculate content performance grade
    fn calculate_content_performance_grade(&self, measurements: &[ContentPerformanceMeasurement]) -> char {
        let compliance_rate = measurements.iter()
            .filter(|m| m.target_compliance.overall_compliant)
            .count() as f64 / measurements.len() as f64;

        match compliance_rate {
            rate if rate >= 0.95 => 'A',
            rate if rate >= 0.85 => 'B',
            rate if rate >= 0.75 => 'C',
            rate if rate >= 0.65 => 'D',
            _ => 'F',
        }
    }
}

/// Complete content performance benchmark report
#[derive(Debug, Clone)]
pub struct ContentPerformanceBenchmarkReport {
    pub overall_compliance: bool,
    pub avg_p99_loading_time: Duration,
    pub avg_cache_hit_rate: f64,
    pub worst_case_p99: Duration,
    pub target_margin_p99: i64,
    pub measurements: Vec<ContentPerformanceMeasurement>,
    pub identified_bottlenecks: Vec<ContentBottleneck>,
    pub optimization_recommendations: Vec<String>,
    pub performance_grade: char,
}

/// Criterion benchmark functions for CI integration
pub fn content_performance_benchmarks(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let config = ContentBenchmarkConfig::default();
    let harness = rt.block_on(async {
        ContentPerformanceBenchmarkHarness::new(config).await.unwrap()
    });

    let mut group = c.benchmark_group("content_performance");
    group.throughput(Throughput::Elements(1));

    group.bench_function("content_loading", |b| {
        b.to_async(&rt).iter(|| async {
            let level_id = LevelId::new(1).unwrap();
            black_box(harness.content_manager.get_level_content(level_id, None).await.unwrap())
        })
    });

    group.bench_function("cache_lookup", |b| {
        b.to_async(&rt).iter(|| async {
            let level_id = LevelId::new(1).unwrap();
            black_box(harness.content_manager.get_cached_content(level_id, None).await)
        })
    });

    group.bench_function("preloading", |b| {
        b.to_async(&rt).iter(|| async {
            let level_id = LevelId::new(1).unwrap();
            black_box(harness.content_manager.preload_upcoming_levels(level_id).await.unwrap())
        })
    });

    group.finish();
}

criterion_group!(content_benches, content_performance_benchmarks);
criterion_main!(content_benches);

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_content_harness_creation() {
        let config = ContentBenchmarkConfig::default();
        let harness = ContentPerformanceBenchmarkHarness::new(config).await.unwrap();

        assert!(harness.config.content_sample_count > 0);
    }

    #[tokio::test]
    async fn test_content_statistics_calculation() {
        let loading_times = vec![
            Duration::from_millis(10),
            Duration::from_millis(20),
            Duration::from_millis(30),
            Duration::from_millis(15),
            Duration::from_millis(25),
        ];

        let cache_times = vec![
            Duration::from_millis(2),
            Duration::from_millis(3),
            Duration::from_millis(1),
        ];

        let mock_cache_metrics = CacheMetrics {
            hit_count: 80,
            miss_count: 20,
            memory_usage_bytes: 1024 * 1024,
            avg_lookup_time_micros: 2000,
            eviction_count: 5,
        };

        let stats = ContentPerformanceBenchmarkHarness::calculate_content_statistics(
            &loading_times, &cache_times, &mock_cache_metrics
        );

        assert!(stats.cache_hit_rate > 70.0);
        assert!(stats.content_generation_rate > 0.0);
        assert!(stats.p99_loading_time <= Duration::from_millis(30));
    }

    #[tokio::test]
    async fn test_content_target_compliance() {
        let config = ContentBenchmarkConfig::default();
        let harness = ContentPerformanceBenchmarkHarness::new(config).await.unwrap();

        let statistics = ContentStatistics {
            mean_loading_time: Duration::from_millis(15),
            median_loading_time: Duration::from_millis(14),
            p95_loading_time: Duration::from_millis(22),
            p99_loading_time: Duration::from_millis(24),
            min_loading_time: Duration::from_millis(10),
            max_loading_time: Duration::from_millis(30),
            loading_time_std_dev: 5.0,
            cache_hit_rate: 92.0,
            cache_miss_rate: 8.0,
            avg_cache_lookup_time: Duration::from_millis(3),
            content_generation_rate: 50.0,
            memory_efficiency_score: 0.5,
            outlier_count: 1,
        };

        let compliance = harness.assess_content_target_compliance(&statistics);
        assert!(compliance.overall_compliant);
        assert!(compliance.p99_loading_compliant);
        assert!(compliance.cache_hit_rate_compliant);
    }
}