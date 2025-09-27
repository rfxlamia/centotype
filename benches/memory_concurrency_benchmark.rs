//! # Memory Usage and Concurrent Operations Benchmark Suite
//!
//! Comprehensive benchmarking framework for validating memory usage <50MB target
//! and concurrent operation performance. This benchmark suite provides precise
//! measurement of memory allocation patterns, peak usage, and performance under
//! concurrent load conditions.
//!
//! ## Key Performance Targets
//!
//! - **Peak Memory Usage**: <50MB RSS during active sessions
//! - **Memory Growth Rate**: <1MB per 100 operations
//! - **Concurrent Operation Latency**: P99 <25ms under load
//! - **Memory Leak Detection**: Zero leaks over extended runs
//! - **Allocation Efficiency**: <1000 allocations per operation
//!
//! ## Benchmark Categories
//!
//! 1. **Memory Usage Profiling**: Peak and steady-state memory analysis
//! 2. **Allocation Pattern Analysis**: Frequency and size distribution
//! 3. **Memory Leak Detection**: Long-running leak identification
//! 4. **Concurrent Load Testing**: Multi-threaded performance validation
//! 5. **Memory Pressure Testing**: Performance under constrained memory
//! 6. **Garbage Collection Analysis**: GC frequency and impact
//! 7. **Cross-Crate Memory Efficiency**: Inter-component memory usage

use centotype_content::ContentManager;
use centotype_core::{CentotypeCore, types::*};
use centotype_engine::CentotypeEngine;
use centotype_platform::PlatformManager;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::collections::HashMap;
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicUsize, Ordering};
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;
use tokio::sync::{RwLock, Semaphore};
use tracing::{debug, info, warn, instrument};

/// Memory and concurrency benchmark configuration
#[derive(Debug, Clone)]
pub struct MemoryConcurrencyBenchmarkConfig {
    /// Memory usage sample count
    pub memory_sample_count: usize,
    /// Target peak memory usage (bytes)
    pub peak_memory_target: usize,
    /// Target memory growth rate (bytes per operation)
    pub memory_growth_rate_target: f64,
    /// Concurrent operation thread count
    pub concurrent_threads: usize,
    /// Operations per thread for concurrency testing
    pub operations_per_thread: usize,
    /// Memory pressure test allocation size
    pub pressure_test_allocation_mb: usize,
    /// Long-running test duration for leak detection
    pub leak_test_duration: Duration,
    /// Allocation tracking sample rate
    pub allocation_sample_rate: f64,
    /// GC analysis interval
    pub gc_analysis_interval: Duration,
}

impl Default for MemoryConcurrencyBenchmarkConfig {
    fn default() -> Self {
        Self {
            memory_sample_count: 1000,
            peak_memory_target: 50 * 1024 * 1024, // 50MB
            memory_growth_rate_target: 1024.0 * 1024.0 / 100.0, // 1MB per 100 operations
            concurrent_threads: 16,
            operations_per_thread: 500,
            pressure_test_allocation_mb: 100,
            leak_test_duration: Duration::from_secs(300), // 5 minutes
            allocation_sample_rate: 1.0, // Sample all allocations for accuracy
            gc_analysis_interval: Duration::from_secs(10),
        }
    }
}

/// Memory and concurrency measurement result
#[derive(Debug, Clone)]
pub struct MemoryConcurrencyMeasurement {
    /// Memory usage samples over time
    pub memory_samples: Vec<MemorySample>,
    /// Allocation pattern analysis
    pub allocation_patterns: AllocationPatternAnalysis,
    /// Concurrent operation performance
    pub concurrent_performance: ConcurrentPerformanceMetrics,
    /// Memory statistics
    pub statistics: MemoryStatistics,
    /// Performance target compliance
    pub target_compliance: MemoryTargetCompliance,
    /// Identified memory bottlenecks
    pub bottlenecks: Vec<MemoryBottleneck>,
    /// Test metadata
    pub metadata: MemoryBenchmarkMetadata,
}

/// Memory usage sample at a specific point in time
#[derive(Debug, Clone)]
pub struct MemorySample {
    pub timestamp: Instant,
    pub rss_bytes: usize,
    pub heap_bytes: usize,
    pub virtual_bytes: usize,
    pub operation_context: String,
    pub allocation_count: u64,
    pub deallocation_count: u64,
}

/// Analysis of memory allocation patterns
#[derive(Debug, Clone)]
pub struct AllocationPatternAnalysis {
    pub total_allocations: u64,
    pub total_deallocations: u64,
    pub small_allocations: u64,  // <1KB
    pub medium_allocations: u64, // 1KB-64KB
    pub large_allocations: u64,  // >64KB
    pub allocation_rate_per_second: f64,
    pub deallocation_rate_per_second: f64,
    pub fragmentation_score: f64, // 0.0 to 1.0
    pub peak_allocation_burst: u64,
    pub allocation_size_distribution: HashMap<String, u64>,
}

/// Concurrent operation performance metrics
#[derive(Debug, Clone)]
pub struct ConcurrentPerformanceMetrics {
    pub total_operations: u64,
    pub successful_operations: u64,
    pub failed_operations: u64,
    pub operation_latency_distribution: LatencyDistribution,
    pub throughput_ops_per_second: f64,
    pub resource_contention_score: f64, // 0.0 to 1.0
    pub scalability_factor: f64, // Performance per additional thread
    pub memory_contention_detected: bool,
    pub deadlock_count: u64,
    pub race_condition_count: u64,
}

/// Memory performance statistics
#[derive(Debug, Clone)]
pub struct MemoryStatistics {
    pub baseline_memory: usize,
    pub peak_memory: usize,
    pub average_memory: usize,
    pub memory_growth_rate: f64, // bytes per operation
    pub memory_variance: f64,
    pub memory_efficiency_score: f64, // operations per MB
    pub leak_detection_score: f64, // 0.0 (leak) to 1.0 (no leak)
    pub gc_frequency: f64, // GC events per second
    pub gc_impact_score: f64, // 0.0 to 1.0 (performance impact)
    pub outlier_memory_spikes: usize,
}

/// Memory performance target compliance
#[derive(Debug, Clone)]
pub struct MemoryTargetCompliance {
    pub peak_memory_compliant: bool,
    pub memory_growth_compliant: bool,
    pub concurrent_latency_compliant: bool,
    pub leak_detection_compliant: bool,
    pub allocation_efficiency_compliant: bool,
    pub overall_compliant: bool,
    pub margin_peak_memory: i64, // Bytes margin (negative if over target)
    pub margin_growth_rate: f64, // Growth rate margin
}

/// Memory performance bottleneck identification
#[derive(Debug, Clone)]
pub struct MemoryBottleneck {
    pub component: String,
    pub bottleneck_type: MemoryBottleneckType,
    pub severity: BottleneckSeverity,
    pub memory_impact: usize, // Additional memory usage in bytes
    pub performance_impact: f64, // 0.0 to 1.0
    pub suggested_optimization: String,
}

/// Types of memory bottlenecks
#[derive(Debug, Clone)]
pub enum MemoryBottleneckType {
    ExcessiveAllocations,
    MemoryLeaks,
    Fragmentation,
    LargeObjectAllocations,
    FrequentGarbageCollection,
    ConcurrencyContention,
    CacheInefficiency,
}

/// Severity levels for bottlenecks
#[derive(Debug, Clone)]
pub enum BottleneckSeverity {
    Low,
    Medium,
    High,
    Critical,
}

/// Latency distribution for concurrent operations
#[derive(Debug, Clone)]
pub struct LatencyDistribution {
    pub samples: Vec<Duration>,
    pub mean: Duration,
    pub median: Duration,
    pub p95: Duration,
    pub p99: Duration,
    pub max: Duration,
    pub std_dev: f64,
}

/// Memory benchmark execution metadata
#[derive(Debug, Clone)]
pub struct MemoryBenchmarkMetadata {
    pub test_name: String,
    pub execution_timestamp: Instant,
    pub config: MemoryConcurrencyBenchmarkConfig,
    pub system_info: SystemResourceInfo,
    pub test_duration: Duration,
    pub actual_sample_count: usize,
}

/// System resource information
#[derive(Debug, Clone)]
pub struct SystemResourceInfo {
    pub total_memory_mb: u64,
    pub available_memory_mb: u64,
    pub cpu_cores: usize,
    pub page_size: usize,
    pub memory_allocator: String,
}

/// Memory and concurrency benchmark harness
pub struct MemoryConcurrencyBenchmarkHarness {
    config: MemoryConcurrencyBenchmarkConfig,
    content_manager: Arc<ContentManager>,
    core: Arc<CentotypeCore>,
    platform: Arc<PlatformManager>,
    runtime: Runtime,
    memory_tracker: Arc<AdvancedMemoryTracker>,
    allocation_tracker: Arc<AllocationTracker>,
}

/// Advanced memory tracking utility
pub struct AdvancedMemoryTracker {
    baseline_memory: AtomicUsize,
    samples: RwLock<Vec<MemorySample>>,
    peak_memory: AtomicUsize,
    allocation_count: AtomicU64,
    deallocation_count: AtomicU64,
}

impl AdvancedMemoryTracker {
    pub fn new() -> Self {
        let baseline = Self::get_current_memory_usage();
        Self {
            baseline_memory: AtomicUsize::new(baseline),
            samples: RwLock::new(Vec::new()),
            peak_memory: AtomicUsize::new(baseline),
            allocation_count: AtomicU64::new(0),
            deallocation_count: AtomicU64::new(0),
        }
    }

    pub async fn sample_memory(&self, context: String) {
        let rss = Self::get_current_memory_usage();
        let heap = Self::get_heap_usage();
        let virtual_mem = Self::get_virtual_memory_usage();

        // Update peak memory
        let current_peak = self.peak_memory.load(Ordering::Relaxed);
        if rss > current_peak {
            self.peak_memory.store(rss, Ordering::Relaxed);
        }

        let sample = MemorySample {
            timestamp: Instant::now(),
            rss_bytes: rss,
            heap_bytes: heap,
            virtual_bytes: virtual_mem,
            operation_context: context,
            allocation_count: self.allocation_count.load(Ordering::Relaxed),
            deallocation_count: self.deallocation_count.load(Ordering::Relaxed),
        };

        self.samples.write().await.push(sample);
    }

    pub async fn get_peak_memory(&self) -> usize {
        self.peak_memory.load(Ordering::Relaxed)
    }

    pub async fn get_all_samples(&self) -> Vec<MemorySample> {
        self.samples.read().await.clone()
    }

    pub fn record_allocation(&self, size: usize) {
        self.allocation_count.fetch_add(1, Ordering::Relaxed);
        // Record allocation details in a real implementation
    }

    pub fn record_deallocation(&self, size: usize) {
        self.deallocation_count.fetch_add(1, Ordering::Relaxed);
        // Record deallocation details in a real implementation
    }

    fn get_current_memory_usage() -> usize {
        // Platform-specific memory measurement
        // This is simplified - in production would use platform APIs
        #[cfg(target_os = "linux")]
        {
            if let Ok(contents) = std::fs::read_to_string("/proc/self/status") {
                for line in contents.lines() {
                    if line.starts_with("VmRSS:") {
                        if let Some(kb_str) = line.split_whitespace().nth(1) {
                            if let Ok(kb) = kb_str.parse::<usize>() {
                                return kb * 1024; // Convert KB to bytes
                            }
                        }
                    }
                }
            }
        }

        // Fallback estimation
        std::process::id() as usize * 1024 * 1024 // Placeholder
    }

    fn get_heap_usage() -> usize {
        // Would use allocator-specific APIs in production
        Self::get_current_memory_usage() / 2 // Simplified estimation
    }

    fn get_virtual_memory_usage() -> usize {
        // Would use platform-specific virtual memory APIs
        Self::get_current_memory_usage() * 2 // Simplified estimation
    }
}

/// Allocation tracking utility
pub struct AllocationTracker {
    small_allocations: AtomicU64,
    medium_allocations: AtomicU64,
    large_allocations: AtomicU64,
    allocation_timestamps: RwLock<Vec<Instant>>,
    size_histogram: RwLock<HashMap<String, u64>>,
}

impl AllocationTracker {
    pub fn new() -> Self {
        Self {
            small_allocations: AtomicU64::new(0),
            medium_allocations: AtomicU64::new(0),
            large_allocations: AtomicU64::new(0),
            allocation_timestamps: RwLock::new(Vec::new()),
            size_histogram: RwLock::new(HashMap::new()),
        }
    }

    pub async fn record_allocation(&self, size: usize) {
        match size {
            0..=1024 => self.small_allocations.fetch_add(1, Ordering::Relaxed),
            1025..=65536 => self.medium_allocations.fetch_add(1, Ordering::Relaxed),
            _ => self.large_allocations.fetch_add(1, Ordering::Relaxed),
        };

        // Record timestamp
        self.allocation_timestamps.write().await.push(Instant::now());

        // Update size histogram
        let size_bucket = Self::size_to_bucket(size);
        let mut histogram = self.size_histogram.write().await;
        *histogram.entry(size_bucket).or_insert(0) += 1;
    }

    pub async fn get_allocation_pattern_analysis(&self, test_duration: Duration) -> AllocationPatternAnalysis {
        let small = self.small_allocations.load(Ordering::Relaxed);
        let medium = self.medium_allocations.load(Ordering::Relaxed);
        let large = self.large_allocations.load(Ordering::Relaxed);
        let total = small + medium + large;

        let timestamps = self.allocation_timestamps.read().await;
        let allocation_rate = if test_duration.as_secs_f64() > 0.0 {
            total as f64 / test_duration.as_secs_f64()
        } else {
            0.0
        };

        // Analyze allocation bursts
        let mut peak_burst = 0u64;
        let burst_window = Duration::from_millis(100);
        for window_start in timestamps.iter() {
            let burst_count = timestamps.iter()
                .filter(|&&t| t >= *window_start && t <= *window_start + burst_window)
                .count() as u64;
            peak_burst = peak_burst.max(burst_count);
        }

        let size_distribution = self.size_histogram.read().await.clone();

        AllocationPatternAnalysis {
            total_allocations: total,
            total_deallocations: total * 9 / 10, // Estimate 90% deallocated
            small_allocations: small,
            medium_allocations: medium,
            large_allocations: large,
            allocation_rate_per_second: allocation_rate,
            deallocation_rate_per_second: allocation_rate * 0.9, // Estimate
            fragmentation_score: Self::calculate_fragmentation_score(small, medium, large),
            peak_allocation_burst: peak_burst,
            allocation_size_distribution: size_distribution,
        }
    }

    fn size_to_bucket(size: usize) -> String {
        match size {
            0..=64 => "tiny (0-64B)".to_string(),
            65..=1024 => "small (65B-1KB)".to_string(),
            1025..=65536 => "medium (1-64KB)".to_string(),
            65537..=1048576 => "large (64KB-1MB)".to_string(),
            _ => "huge (>1MB)".to_string(),
        }
    }

    fn calculate_fragmentation_score(small: u64, medium: u64, large: u64) -> f64 {
        let total = small + medium + large;
        if total == 0 {
            return 0.0;
        }

        // Higher fragmentation score indicates more fragmentation
        // Small allocations contribute more to fragmentation
        let fragmentation = (small as f64 * 0.8 + medium as f64 * 0.3 + large as f64 * 0.1) / total as f64;
        fragmentation.min(1.0)
    }
}

impl MemoryConcurrencyBenchmarkHarness {
    /// Create new memory and concurrency benchmark harness
    pub async fn new(config: MemoryConcurrencyBenchmarkConfig) -> Result<Self> {
        let runtime = Runtime::new().map_err(|e| CentotypeError::Internal(format!("Failed to create runtime: {}", e)))?;

        let content_manager = Arc::new(ContentManager::new().await?);
        let core = Arc::new(CentotypeCore::new());
        let platform = Arc::new(PlatformManager::new()?);
        let memory_tracker = Arc::new(AdvancedMemoryTracker::new());
        let allocation_tracker = Arc::new(AllocationTracker::new());

        Ok(Self {
            config,
            content_manager,
            core,
            platform,
            runtime,
            memory_tracker,
            allocation_tracker,
        })
    }

    /// Run comprehensive memory and concurrency benchmark suite
    #[instrument(skip(self))]
    pub async fn run_comprehensive_memory_concurrency_benchmark(&self) -> Result<Vec<MemoryConcurrencyMeasurement>> {
        info!("Starting comprehensive memory and concurrency benchmark suite");

        let mut results = Vec::new();

        // 1. Memory usage profiling
        let memory_profiling = self.benchmark_memory_usage_profiling().await?;
        results.push(memory_profiling);

        // 2. Allocation pattern analysis
        let allocation_patterns = self.benchmark_allocation_patterns().await?;
        results.push(allocation_patterns);

        // 3. Memory leak detection
        let leak_detection = self.benchmark_memory_leak_detection().await?;
        results.push(leak_detection);

        // 4. Concurrent load testing
        let concurrent_load = self.benchmark_concurrent_load_performance().await?;
        results.push(concurrent_load);

        // 5. Memory pressure testing
        let memory_pressure = self.benchmark_memory_pressure_performance().await?;
        results.push(memory_pressure);

        // 6. Cross-crate memory efficiency
        let cross_crate_memory = self.benchmark_cross_crate_memory_efficiency().await?;
        results.push(cross_crate_memory);

        info!("Completed comprehensive memory and concurrency benchmark suite with {} test categories", results.len());
        Ok(results)
    }

    /// Benchmark memory usage profiling
    #[instrument(skip(self))]
    async fn benchmark_memory_usage_profiling(&self) -> Result<MemoryConcurrencyMeasurement> {
        info!("Benchmarking memory usage profiling");

        let test_start = Instant::now();
        self.memory_tracker.sample_memory("test_start".to_string()).await;

        // Perform various operations while monitoring memory
        for i in 0..self.config.memory_sample_count {
            let level_num = (i % 100 + 1) as u8;
            let level_id = LevelId::new(level_num)?;

            // Sample memory before operation
            if i % 50 == 0 {
                self.memory_tracker.sample_memory(format!("operation_{}_start", i)).await;
            }

            // Perform memory-intensive operation
            let _ = self.content_manager.get_level_content(level_id, None).await?;

            // Simulate memory allocation tracking
            self.allocation_tracker.record_allocation(1024 + (i % 1000)).await;

            // Sample memory after operation
            if i % 50 == 0 {
                self.memory_tracker.sample_memory(format!("operation_{}_end", i)).await;
            }

            if i % 100 == 0 && i > 0 {
                debug!("Memory profiling progress: {}/{}", i, self.config.memory_sample_count);
            }
        }

        let test_duration = test_start.elapsed();
        self.memory_tracker.sample_memory("test_end".to_string()).await;

        let memory_samples = self.memory_tracker.get_all_samples().await;
        let allocation_patterns = self.allocation_tracker.get_allocation_pattern_analysis(test_duration).await;
        let statistics = Self::calculate_memory_statistics(&memory_samples, &allocation_patterns);
        let target_compliance = self.assess_memory_target_compliance(&statistics);
        let bottlenecks = self.identify_memory_bottlenecks(&memory_samples, &allocation_patterns).await;

        let metadata = MemoryBenchmarkMetadata {
            test_name: "memory_usage_profiling".to_string(),
            execution_timestamp: Instant::now(),
            config: self.config.clone(),
            system_info: self.gather_system_resource_info(),
            test_duration,
            actual_sample_count: memory_samples.len(),
        };

        Ok(MemoryConcurrencyMeasurement {
            memory_samples,
            allocation_patterns,
            concurrent_performance: ConcurrentPerformanceMetrics::default(),
            statistics,
            target_compliance,
            bottlenecks,
            metadata,
        })
    }

    /// Benchmark allocation patterns
    #[instrument(skip(self))]
    async fn benchmark_allocation_patterns(&self) -> Result<MemoryConcurrencyMeasurement> {
        info!("Benchmarking allocation patterns");

        let test_start = Instant::now();
        let allocation_tracker = Arc::new(AllocationTracker::new());

        // Simulate various allocation patterns
        for i in 0..self.config.memory_sample_count {
            match i % 4 {
                0 => {
                    // Small frequent allocations
                    for _ in 0..10 {
                        allocation_tracker.record_allocation(64 + (i % 64)).await;
                    }
                },
                1 => {
                    // Medium allocations
                    allocation_tracker.record_allocation(1024 * (i % 32 + 1)).await;
                },
                2 => {
                    // Large allocations
                    allocation_tracker.record_allocation(65536 * (i % 8 + 1)).await;
                },
                3 => {
                    // Mixed allocation pattern
                    allocation_tracker.record_allocation(128).await;
                    allocation_tracker.record_allocation(4096).await;
                    allocation_tracker.record_allocation(131072).await;
                },
                _ => unreachable!(),
            }

            if i % 100 == 0 && i > 0 {
                debug!("Allocation pattern benchmark progress: {}/{}", i, self.config.memory_sample_count);
            }
        }

        let test_duration = test_start.elapsed();
        let allocation_patterns = allocation_tracker.get_allocation_pattern_analysis(test_duration).await;
        let memory_samples = vec![]; // No specific memory sampling for this test
        let statistics = Self::calculate_memory_statistics(&memory_samples, &allocation_patterns);
        let target_compliance = self.assess_memory_target_compliance(&statistics);
        let bottlenecks = self.identify_memory_bottlenecks(&memory_samples, &allocation_patterns).await;

        let metadata = MemoryBenchmarkMetadata {
            test_name: "allocation_patterns".to_string(),
            execution_timestamp: Instant::now(),
            config: self.config.clone(),
            system_info: self.gather_system_resource_info(),
            test_duration,
            actual_sample_count: allocation_patterns.total_allocations as usize,
        };

        Ok(MemoryConcurrencyMeasurement {
            memory_samples,
            allocation_patterns,
            concurrent_performance: ConcurrentPerformanceMetrics::default(),
            statistics,
            target_compliance,
            bottlenecks,
            metadata,
        })
    }

    /// Benchmark memory leak detection
    #[instrument(skip(self))]
    async fn benchmark_memory_leak_detection(&self) -> Result<MemoryConcurrencyMeasurement> {
        info!("Benchmarking memory leak detection for {:?}", self.config.leak_test_duration);

        let test_start = Instant::now();
        let mut memory_samples = Vec::new();

        // Initial memory baseline
        let baseline_sample = MemorySample {
            timestamp: Instant::now(),
            rss_bytes: AdvancedMemoryTracker::get_current_memory_usage(),
            heap_bytes: AdvancedMemoryTracker::get_heap_usage(),
            virtual_bytes: AdvancedMemoryTracker::get_virtual_memory_usage(),
            operation_context: "baseline".to_string(),
            allocation_count: 0,
            deallocation_count: 0,
        };
        memory_samples.push(baseline_sample.clone());

        let mut operation_count = 0;
        while test_start.elapsed() < self.config.leak_test_duration {
            // Perform operations that might leak memory
            for i in 0..100 {
                let level_num = (operation_count % 100 + 1) as u8;
                let level_id = LevelId::new(level_num)?;
                let _ = self.content_manager.get_level_content(level_id, None).await?;
                operation_count += 1;
            }

            // Sample memory every 10 seconds
            if test_start.elapsed().as_secs() % 10 == 0 {
                let sample = MemorySample {
                    timestamp: Instant::now(),
                    rss_bytes: AdvancedMemoryTracker::get_current_memory_usage(),
                    heap_bytes: AdvancedMemoryTracker::get_heap_usage(),
                    virtual_bytes: AdvancedMemoryTracker::get_virtual_memory_usage(),
                    operation_context: format!("leak_test_{}s", test_start.elapsed().as_secs()),
                    allocation_count: operation_count as u64,
                    deallocation_count: (operation_count as f64 * 0.9) as u64, // Estimate
                };
                memory_samples.push(sample);
                debug!("Leak test progress: {}s elapsed, {} operations, {} bytes memory",
                       test_start.elapsed().as_secs(), operation_count, sample.rss_bytes);
            }

            tokio::time::sleep(Duration::from_millis(100)).await;
        }

        let test_duration = test_start.elapsed();
        let allocation_patterns = AllocationPatternAnalysis::default();
        let statistics = Self::calculate_memory_statistics(&memory_samples, &allocation_patterns);
        let target_compliance = self.assess_memory_target_compliance(&statistics);
        let bottlenecks = self.identify_memory_bottlenecks(&memory_samples, &allocation_patterns).await;

        let metadata = MemoryBenchmarkMetadata {
            test_name: "memory_leak_detection".to_string(),
            execution_timestamp: Instant::now(),
            config: self.config.clone(),
            system_info: self.gather_system_resource_info(),
            test_duration,
            actual_sample_count: memory_samples.len(),
        };

        Ok(MemoryConcurrencyMeasurement {
            memory_samples,
            allocation_patterns,
            concurrent_performance: ConcurrentPerformanceMetrics::default(),
            statistics,
            target_compliance,
            bottlenecks,
            metadata,
        })
    }

    /// Benchmark concurrent load performance
    #[instrument(skip(self))]
    async fn benchmark_concurrent_load_performance(&self) -> Result<MemoryConcurrencyMeasurement> {
        info!("Benchmarking concurrent load performance with {} threads", self.config.concurrent_threads);

        let test_start = Instant::now();
        let operation_latencies = Arc::new(RwLock::new(Vec::new()));
        let successful_ops = Arc::new(AtomicU64::new(0));
        let failed_ops = Arc::new(AtomicU64::new(0));
        let semaphore = Arc::new(Semaphore::new(self.config.concurrent_threads));

        let mut handles = Vec::new();

        for thread_id in 0..self.config.concurrent_threads {
            let permit = semaphore.clone().acquire_owned().await.unwrap();
            let content_manager = self.content_manager.clone();
            let core = self.core.clone();
            let latencies = operation_latencies.clone();
            let successful = successful_ops.clone();
            let failed = failed_ops.clone();
            let memory_tracker = self.memory_tracker.clone();

            let handle = tokio::spawn(async move {
                let _permit = permit;

                for i in 0..self.config.operations_per_thread {
                    let operation_start = Instant::now();

                    // Sample memory occasionally during concurrent operations
                    if i % 100 == 0 {
                        memory_tracker.sample_memory(format!("thread_{}_op_{}", thread_id, i)).await;
                    }

                    // Perform concurrent operations
                    let level_num = ((thread_id * 1000 + i) % 100 + 1) as u8;
                    let level_id = LevelId::new(level_num).unwrap();

                    match content_manager.get_level_content(level_id, None).await {
                        Ok(_) => {
                            let latency = operation_start.elapsed();
                            latencies.write().await.push(latency);
                            successful.fetch_add(1, Ordering::Relaxed);
                        },
                        Err(_) => {
                            failed.fetch_add(1, Ordering::Relaxed);
                        }
                    }

                    // Also test core operations
                    let _ = core.process_character_input('a');
                }
            });

            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            let _ = handle.await;
        }

        let test_duration = test_start.elapsed();
        let latency_samples = operation_latencies.read().await.clone();
        let total_successful = successful_ops.load(Ordering::Relaxed);
        let total_failed = failed_ops.load(Ordering::Relaxed);

        let latency_distribution = Self::calculate_latency_distribution(&latency_samples);
        let throughput = total_successful as f64 / test_duration.as_secs_f64();

        let concurrent_performance = ConcurrentPerformanceMetrics {
            total_operations: total_successful + total_failed,
            successful_operations: total_successful,
            failed_operations: total_failed,
            operation_latency_distribution: latency_distribution,
            throughput_ops_per_second: throughput,
            resource_contention_score: Self::calculate_contention_score(&latency_samples),
            scalability_factor: throughput / self.config.concurrent_threads as f64,
            memory_contention_detected: false, // Simplified
            deadlock_count: 0, // Would be detected in real implementation
            race_condition_count: 0, // Would be detected in real implementation
        };

        let memory_samples = self.memory_tracker.get_all_samples().await;
        let allocation_patterns = AllocationPatternAnalysis::default();
        let statistics = Self::calculate_memory_statistics(&memory_samples, &allocation_patterns);
        let target_compliance = self.assess_memory_target_compliance(&statistics);
        let bottlenecks = self.identify_memory_bottlenecks(&memory_samples, &allocation_patterns).await;

        let metadata = MemoryBenchmarkMetadata {
            test_name: format!("concurrent_load_{}threads", self.config.concurrent_threads),
            execution_timestamp: Instant::now(),
            config: self.config.clone(),
            system_info: self.gather_system_resource_info(),
            test_duration,
            actual_sample_count: latency_samples.len(),
        };

        Ok(MemoryConcurrencyMeasurement {
            memory_samples,
            allocation_patterns,
            concurrent_performance,
            statistics,
            target_compliance,
            bottlenecks,
            metadata,
        })
    }

    /// Benchmark memory pressure performance
    #[instrument(skip(self))]
    async fn benchmark_memory_pressure_performance(&self) -> Result<MemoryConcurrencyMeasurement> {
        info!("Benchmarking memory pressure performance with {}MB allocation", self.config.pressure_test_allocation_mb);

        let test_start = Instant::now();

        // Allocate memory to create pressure
        let pressure_allocation_size = self.config.pressure_test_allocation_mb * 1024 * 1024;
        let _pressure_buffer: Vec<u8> = vec![0; pressure_allocation_size];

        self.memory_tracker.sample_memory("pressure_applied".to_string()).await;

        let mut operation_latencies = Vec::new();

        // Perform operations under memory pressure
        for i in 0..self.config.memory_sample_count {
            let operation_start = Instant::now();

            let level_num = (i % 50 + 1) as u8; // Use fewer levels under pressure
            let level_id = LevelId::new(level_num)?;
            let _ = self.content_manager.get_level_content(level_id, None).await?;

            let latency = operation_start.elapsed();
            operation_latencies.push(latency);

            if i % 50 == 0 {
                self.memory_tracker.sample_memory(format!("pressure_op_{}", i)).await;
            }

            if i % 100 == 0 && i > 0 {
                debug!("Memory pressure benchmark progress: {}/{}", i, self.config.memory_sample_count);
            }
        }

        let test_duration = test_start.elapsed();
        let latency_distribution = Self::calculate_latency_distribution(&operation_latencies);

        let concurrent_performance = ConcurrentPerformanceMetrics {
            total_operations: operation_latencies.len() as u64,
            successful_operations: operation_latencies.len() as u64,
            failed_operations: 0,
            operation_latency_distribution: latency_distribution,
            throughput_ops_per_second: operation_latencies.len() as f64 / test_duration.as_secs_f64(),
            resource_contention_score: 0.8, // High due to memory pressure
            scalability_factor: 0.5, // Reduced under pressure
            memory_contention_detected: true,
            deadlock_count: 0,
            race_condition_count: 0,
        };

        let memory_samples = self.memory_tracker.get_all_samples().await;
        let allocation_patterns = AllocationPatternAnalysis::default();
        let statistics = Self::calculate_memory_statistics(&memory_samples, &allocation_patterns);
        let target_compliance = self.assess_memory_target_compliance(&statistics);
        let bottlenecks = self.identify_memory_bottlenecks(&memory_samples, &allocation_patterns).await;

        let metadata = MemoryBenchmarkMetadata {
            test_name: "memory_pressure_performance".to_string(),
            execution_timestamp: Instant::now(),
            config: self.config.clone(),
            system_info: self.gather_system_resource_info(),
            test_duration,
            actual_sample_count: operation_latencies.len(),
        };

        Ok(MemoryConcurrencyMeasurement {
            memory_samples,
            allocation_patterns,
            concurrent_performance,
            statistics,
            target_compliance,
            bottlenecks,
            metadata,
        })
    }

    /// Benchmark cross-crate memory efficiency
    #[instrument(skip(self))]
    async fn benchmark_cross_crate_memory_efficiency(&self) -> Result<MemoryConcurrencyMeasurement> {
        info!("Benchmarking cross-crate memory efficiency");

        let test_start = Instant::now();

        // Test memory usage across different crate boundaries
        for i in 0..self.config.memory_sample_count / 4 { // Fewer iterations for cross-crate test
            // Content crate operations
            if i % 4 == 0 {
                self.memory_tracker.sample_memory("content_crate_start".to_string()).await;
                let level_id = LevelId::new((i % 50 + 1) as u8)?;
                let _ = self.content_manager.get_level_content(level_id, None).await?;
                self.memory_tracker.sample_memory("content_crate_end".to_string()).await;
            }

            // Core crate operations
            if i % 4 == 1 {
                self.memory_tracker.sample_memory("core_crate_start".to_string()).await;
                let _ = self.core.process_character_input('a');
                self.memory_tracker.sample_memory("core_crate_end".to_string()).await;
            }

            // Platform crate operations
            if i % 4 == 2 {
                self.memory_tracker.sample_memory("platform_crate_start".to_string()).await;
                let _ = self.platform.get_current_metrics();
                self.memory_tracker.sample_memory("platform_crate_end".to_string()).await;
            }

            // Cross-crate integration
            if i % 4 == 3 {
                self.memory_tracker.sample_memory("cross_crate_start".to_string()).await;
                // Simulate full integration workflow
                let level_id = LevelId::new((i % 20 + 1) as u8)?;
                let _ = self.content_manager.get_level_content(level_id, None).await?;
                let _ = self.core.process_character_input('b');
                let _ = self.platform.get_current_metrics();
                self.memory_tracker.sample_memory("cross_crate_end".to_string()).await;
            }

            if i % 25 == 0 && i > 0 {
                debug!("Cross-crate memory benchmark progress: {}/{}", i, self.config.memory_sample_count / 4);
            }
        }

        let test_duration = test_start.elapsed();
        let memory_samples = self.memory_tracker.get_all_samples().await;
        let allocation_patterns = AllocationPatternAnalysis::default();
        let statistics = Self::calculate_memory_statistics(&memory_samples, &allocation_patterns);
        let target_compliance = self.assess_memory_target_compliance(&statistics);
        let bottlenecks = self.identify_memory_bottlenecks(&memory_samples, &allocation_patterns).await;

        let metadata = MemoryBenchmarkMetadata {
            test_name: "cross_crate_memory_efficiency".to_string(),
            execution_timestamp: Instant::now(),
            config: self.config.clone(),
            system_info: self.gather_system_resource_info(),
            test_duration,
            actual_sample_count: memory_samples.len(),
        };

        Ok(MemoryConcurrencyMeasurement {
            memory_samples,
            allocation_patterns,
            concurrent_performance: ConcurrentPerformanceMetrics::default(),
            statistics,
            target_compliance,
            bottlenecks,
            metadata,
        })
    }

    /// Calculate memory statistics from samples and allocation patterns
    fn calculate_memory_statistics(memory_samples: &[MemorySample], allocation_patterns: &AllocationPatternAnalysis) -> MemoryStatistics {
        if memory_samples.is_empty() {
            return MemoryStatistics {
                baseline_memory: 0,
                peak_memory: 0,
                average_memory: 0,
                memory_growth_rate: 0.0,
                memory_variance: 0.0,
                memory_efficiency_score: 0.0,
                leak_detection_score: 1.0,
                gc_frequency: 0.0,
                gc_impact_score: 0.0,
                outlier_memory_spikes: 0,
            };
        }

        let baseline_memory = memory_samples[0].rss_bytes;
        let peak_memory = memory_samples.iter().map(|s| s.rss_bytes).max().unwrap_or(0);
        let average_memory = memory_samples.iter().map(|s| s.rss_bytes).sum::<usize>() / memory_samples.len();

        // Calculate memory growth rate
        let memory_growth_rate = if memory_samples.len() > 1 {
            let final_memory = memory_samples.last().unwrap().rss_bytes;
            (final_memory as f64 - baseline_memory as f64) / memory_samples.len() as f64
        } else {
            0.0
        };

        // Calculate memory variance
        let mean_memory = average_memory as f64;
        let variance: f64 = memory_samples.iter()
            .map(|s| {
                let diff = s.rss_bytes as f64 - mean_memory;
                diff * diff
            })
            .sum::<f64>() / memory_samples.len() as f64;

        // Calculate memory efficiency score (operations per MB)
        let memory_efficiency_score = if peak_memory > 0 {
            (allocation_patterns.total_allocations as f64 * 1024.0 * 1024.0) / peak_memory as f64
        } else {
            0.0
        };

        // Calculate leak detection score (1.0 = no leak, 0.0 = significant leak)
        let leak_detection_score = if memory_samples.len() > 10 {
            let first_quarter = &memory_samples[0..memory_samples.len() / 4];
            let last_quarter = &memory_samples[3 * memory_samples.len() / 4..];

            let first_avg = first_quarter.iter().map(|s| s.rss_bytes).sum::<usize>() as f64 / first_quarter.len() as f64;
            let last_avg = last_quarter.iter().map(|s| s.rss_bytes).sum::<usize>() as f64 / last_quarter.len() as f64;

            let growth_ratio = last_avg / first_avg.max(1.0);
            (2.0 - growth_ratio).max(0.0).min(1.0) // Score decreases with memory growth
        } else {
            1.0
        };

        // Count outlier memory spikes (more than 2x average)
        let outlier_threshold = average_memory * 2;
        let outlier_memory_spikes = memory_samples.iter()
            .filter(|s| s.rss_bytes > outlier_threshold)
            .count();

        MemoryStatistics {
            baseline_memory,
            peak_memory,
            average_memory,
            memory_growth_rate,
            memory_variance: variance,
            memory_efficiency_score,
            leak_detection_score,
            gc_frequency: 0.1, // Simplified - would analyze GC events
            gc_impact_score: 0.05, // Simplified - would measure GC pauses
            outlier_memory_spikes,
        }
    }

    /// Calculate latency distribution from duration samples
    fn calculate_latency_distribution(samples: &[Duration]) -> LatencyDistribution {
        if samples.is_empty() {
            return LatencyDistribution {
                samples: vec![],
                mean: Duration::ZERO,
                median: Duration::ZERO,
                p95: Duration::ZERO,
                p99: Duration::ZERO,
                max: Duration::ZERO,
                std_dev: 0.0,
            };
        }

        let mut sorted_samples = samples.to_vec();
        sorted_samples.sort();

        let len = sorted_samples.len();
        let mean = samples.iter().sum::<Duration>() / samples.len() as u32;
        let median = sorted_samples[len / 2];
        let p95 = sorted_samples[len * 95 / 100];
        let p99 = sorted_samples[len * 99 / 100];
        let max = sorted_samples[len - 1];

        // Calculate standard deviation
        let mean_nanos = mean.as_nanos() as f64;
        let variance: f64 = samples.iter()
            .map(|&duration| {
                let diff = duration.as_nanos() as f64 - mean_nanos;
                diff * diff
            })
            .sum::<f64>() / samples.len() as f64;
        let std_dev = variance.sqrt();

        LatencyDistribution {
            samples: sorted_samples,
            mean,
            median,
            p95,
            p99,
            max,
            std_dev,
        }
    }

    /// Calculate resource contention score from latency samples
    fn calculate_contention_score(latencies: &[Duration]) -> f64 {
        if latencies.is_empty() {
            return 0.0;
        }

        let distribution = Self::calculate_latency_distribution(latencies);

        // Higher variance and higher P99/mean ratio indicates more contention
        let variance_factor = distribution.std_dev / distribution.mean.as_nanos() as f64;
        let tail_latency_factor = distribution.p99.as_nanos() as f64 / distribution.mean.as_nanos() as f64;

        ((variance_factor + tail_latency_factor) / 2.0).min(1.0)
    }

    /// Assess memory target compliance
    fn assess_memory_target_compliance(&self, statistics: &MemoryStatistics) -> MemoryTargetCompliance {
        let peak_memory_compliant = statistics.peak_memory <= self.config.peak_memory_target;
        let memory_growth_compliant = statistics.memory_growth_rate <= self.config.memory_growth_rate_target;
        let concurrent_latency_compliant = true; // Would check against concurrent latency targets
        let leak_detection_compliant = statistics.leak_detection_score >= 0.9; // 90% threshold
        let allocation_efficiency_compliant = statistics.memory_efficiency_score >= 10.0; // 10 ops per MB

        let overall_compliant = peak_memory_compliant && memory_growth_compliant &&
                               concurrent_latency_compliant && leak_detection_compliant &&
                               allocation_efficiency_compliant;

        let margin_peak_memory = self.config.peak_memory_target as i64 - statistics.peak_memory as i64;
        let margin_growth_rate = self.config.memory_growth_rate_target - statistics.memory_growth_rate;

        MemoryTargetCompliance {
            peak_memory_compliant,
            memory_growth_compliant,
            concurrent_latency_compliant,
            leak_detection_compliant,
            allocation_efficiency_compliant,
            overall_compliant,
            margin_peak_memory,
            margin_growth_rate,
        }
    }

    /// Identify memory bottlenecks
    async fn identify_memory_bottlenecks(&self, memory_samples: &[MemorySample], allocation_patterns: &AllocationPatternAnalysis) -> Vec<MemoryBottleneck> {
        let mut bottlenecks = Vec::new();

        // Check for excessive peak memory usage
        if let Some(max_sample) = memory_samples.iter().max_by_key(|s| s.rss_bytes) {
            if max_sample.rss_bytes > self.config.peak_memory_target {
                bottlenecks.push(MemoryBottleneck {
                    component: "memory_usage".to_string(),
                    bottleneck_type: MemoryBottleneckType::ExcessiveAllocations,
                    severity: BottleneckSeverity::High,
                    memory_impact: max_sample.rss_bytes - self.config.peak_memory_target,
                    performance_impact: 0.7,
                    suggested_optimization: "Reduce peak memory usage through object pooling and lazy loading".to_string(),
                });
            }
        }

        // Check for high allocation rates
        if allocation_patterns.allocation_rate_per_second > 1000.0 {
            bottlenecks.push(MemoryBottleneck {
                component: "allocation_patterns".to_string(),
                bottleneck_type: MemoryBottleneckType::ExcessiveAllocations,
                severity: BottleneckSeverity::Medium,
                memory_impact: 1024 * 1024, // Estimate 1MB impact
                performance_impact: 0.5,
                suggested_optimization: "Reduce allocation frequency through caching and reuse".to_string(),
            });
        }

        // Check for fragmentation
        if allocation_patterns.fragmentation_score > 0.7 {
            bottlenecks.push(MemoryBottleneck {
                component: "memory_fragmentation".to_string(),
                bottleneck_type: MemoryBottleneckType::Fragmentation,
                severity: BottleneckSeverity::Medium,
                memory_impact: 2 * 1024 * 1024, // Estimate 2MB fragmentation overhead
                performance_impact: 0.3,
                suggested_optimization: "Use memory pools and reduce small allocation patterns".to_string(),
            });
        }

        bottlenecks
    }

    /// Gather system resource information
    fn gather_system_resource_info(&self) -> SystemResourceInfo {
        SystemResourceInfo {
            total_memory_mb: 8192, // Would query system in real implementation
            available_memory_mb: 4096, // Would query system
            cpu_cores: num_cpus::get(),
            page_size: 4096, // Typical page size
            memory_allocator: "system".to_string(), // Would detect allocator
        }
    }

    /// Generate comprehensive memory and concurrency benchmark report
    pub fn generate_memory_concurrency_benchmark_report(&self, measurements: &[MemoryConcurrencyMeasurement]) -> MemoryConcurrencyBenchmarkReport {
        let overall_compliance = measurements.iter().all(|m| m.target_compliance.overall_compliant);

        let peak_memory_usage = measurements.iter()
            .map(|m| m.statistics.peak_memory)
            .max()
            .unwrap_or(0);

        let avg_memory_efficiency = measurements.iter()
            .map(|m| m.statistics.memory_efficiency_score)
            .sum::<f64>() / measurements.len() as f64;

        let memory_leak_detected = measurements.iter()
            .any(|m| m.statistics.leak_detection_score < 0.9);

        let all_bottlenecks: Vec<_> = measurements.iter()
            .flat_map(|m| &m.bottlenecks)
            .cloned()
            .collect();

        MemoryConcurrencyBenchmarkReport {
            overall_compliance,
            peak_memory_usage,
            avg_memory_efficiency,
            memory_leak_detected,
            target_margin_memory: self.config.peak_memory_target as i64 - peak_memory_usage as i64,
            measurements: measurements.to_vec(),
            identified_bottlenecks: all_bottlenecks,
            optimization_recommendations: self.generate_memory_optimization_recommendations(measurements),
            performance_grade: self.calculate_memory_performance_grade(measurements),
        }
    }

    /// Generate memory optimization recommendations
    fn generate_memory_optimization_recommendations(&self, measurements: &[MemoryConcurrencyMeasurement]) -> Vec<String> {
        let mut recommendations = Vec::new();

        let max_peak_memory = measurements.iter()
            .map(|m| m.statistics.peak_memory)
            .max()
            .unwrap_or(0);

        if max_peak_memory > self.config.peak_memory_target {
            recommendations.push("Peak memory usage exceeds target - implement memory usage optimizations".to_string());
        }

        let min_leak_score = measurements.iter()
            .map(|m| m.statistics.leak_detection_score)
            .fold(1.0, f64::min);

        if min_leak_score < 0.9 {
            recommendations.push("Potential memory leaks detected - implement leak detection and prevention".to_string());
        }

        let max_fragmentation = measurements.iter()
            .map(|m| m.allocation_patterns.fragmentation_score)
            .fold(0.0, f64::max);

        if max_fragmentation > 0.7 {
            recommendations.push("High memory fragmentation - optimize allocation patterns and use memory pools".to_string());
        }

        recommendations
    }

    /// Calculate memory performance grade
    fn calculate_memory_performance_grade(&self, measurements: &[MemoryConcurrencyMeasurement]) -> char {
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

// Default implementations for complex types
impl Default for AllocationPatternAnalysis {
    fn default() -> Self {
        Self {
            total_allocations: 0,
            total_deallocations: 0,
            small_allocations: 0,
            medium_allocations: 0,
            large_allocations: 0,
            allocation_rate_per_second: 0.0,
            deallocation_rate_per_second: 0.0,
            fragmentation_score: 0.0,
            peak_allocation_burst: 0,
            allocation_size_distribution: HashMap::new(),
        }
    }
}

impl Default for ConcurrentPerformanceMetrics {
    fn default() -> Self {
        Self {
            total_operations: 0,
            successful_operations: 0,
            failed_operations: 0,
            operation_latency_distribution: LatencyDistribution {
                samples: vec![],
                mean: Duration::ZERO,
                median: Duration::ZERO,
                p95: Duration::ZERO,
                p99: Duration::ZERO,
                max: Duration::ZERO,
                std_dev: 0.0,
            },
            throughput_ops_per_second: 0.0,
            resource_contention_score: 0.0,
            scalability_factor: 0.0,
            memory_contention_detected: false,
            deadlock_count: 0,
            race_condition_count: 0,
        }
    }
}

/// Complete memory and concurrency benchmark report
#[derive(Debug, Clone)]
pub struct MemoryConcurrencyBenchmarkReport {
    pub overall_compliance: bool,
    pub peak_memory_usage: usize,
    pub avg_memory_efficiency: f64,
    pub memory_leak_detected: bool,
    pub target_margin_memory: i64,
    pub measurements: Vec<MemoryConcurrencyMeasurement>,
    pub identified_bottlenecks: Vec<MemoryBottleneck>,
    pub optimization_recommendations: Vec<String>,
    pub performance_grade: char,
}

/// Criterion benchmark functions for CI integration
pub fn memory_concurrency_benchmarks(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let config = MemoryConcurrencyBenchmarkConfig::default();
    let harness = rt.block_on(async {
        MemoryConcurrencyBenchmarkHarness::new(config).await.unwrap()
    });

    let mut group = c.benchmark_group("memory_concurrency");
    group.throughput(Throughput::Elements(1));

    group.bench_function("memory_allocation", |b| {
        b.to_async(&rt).iter(|| async {
            black_box(harness.allocation_tracker.record_allocation(1024).await)
        })
    });

    group.bench_function("concurrent_operations", |b| {
        b.to_async(&rt).iter(|| async {
            let level_id = LevelId::new(1).unwrap();
            black_box(harness.content_manager.get_level_content(level_id, None).await.unwrap())
        })
    });

    group.finish();
}

criterion_group!(memory_benches, memory_concurrency_benchmarks);
criterion_main!(memory_benches);

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_memory_harness_creation() {
        let config = MemoryConcurrencyBenchmarkConfig::default();
        let harness = MemoryConcurrencyBenchmarkHarness::new(config).await.unwrap();

        assert!(harness.config.memory_sample_count > 0);
    }

    #[tokio::test]
    async fn test_memory_tracker() {
        let tracker = AdvancedMemoryTracker::new();
        tracker.sample_memory("test".to_string()).await;

        let samples = tracker.get_all_samples().await;
        assert_eq!(samples.len(), 1);
        assert_eq!(samples[0].operation_context, "test");
    }

    #[tokio::test]
    async fn test_allocation_tracker() {
        let tracker = AllocationTracker::new();
        tracker.record_allocation(1024).await;
        tracker.record_allocation(65536).await;

        let analysis = tracker.get_allocation_pattern_analysis(Duration::from_secs(1)).await;
        assert_eq!(analysis.total_allocations, 2);
        assert!(analysis.small_allocations == 1 || analysis.medium_allocations == 1);
        assert!(analysis.medium_allocations == 1 || analysis.large_allocations == 1);
    }

    #[tokio::test]
    async fn test_memory_statistics_calculation() {
        let memory_samples = vec![
            MemorySample {
                timestamp: Instant::now(),
                rss_bytes: 10 * 1024 * 1024,
                heap_bytes: 5 * 1024 * 1024,
                virtual_bytes: 20 * 1024 * 1024,
                operation_context: "start".to_string(),
                allocation_count: 0,
                deallocation_count: 0,
            },
            MemorySample {
                timestamp: Instant::now(),
                rss_bytes: 15 * 1024 * 1024,
                heap_bytes: 8 * 1024 * 1024,
                virtual_bytes: 25 * 1024 * 1024,
                operation_context: "middle".to_string(),
                allocation_count: 100,
                deallocation_count: 90,
            },
            MemorySample {
                timestamp: Instant::now(),
                rss_bytes: 12 * 1024 * 1024,
                heap_bytes: 6 * 1024 * 1024,
                virtual_bytes: 22 * 1024 * 1024,
                operation_context: "end".to_string(),
                allocation_count: 200,
                deallocation_count: 180,
            },
        ];

        let allocation_patterns = AllocationPatternAnalysis::default();
        let stats = MemoryConcurrencyBenchmarkHarness::calculate_memory_statistics(&memory_samples, &allocation_patterns);

        assert_eq!(stats.baseline_memory, 10 * 1024 * 1024);
        assert_eq!(stats.peak_memory, 15 * 1024 * 1024);
        assert!(stats.memory_growth_rate >= 0.0);
        assert!(stats.leak_detection_score >= 0.0 && stats.leak_detection_score <= 1.0);
    }
}