//! # Inter-Crate Metrics Collection and Monitoring
//!
//! This module provides comprehensive metrics collection across all centotype crates
//! to monitor performance, identify bottlenecks, and validate the <25ms P99 target.
//! It tracks cross-crate communication latency, memory usage patterns, and async
//! boundary performance in real-time.
//!
//! ## Key Monitoring Areas
//!
//! 1. **Cross-Crate Communication Latency**
//! 2. **Memory Allocation Patterns**
//! 3. **Async Boundary Performance**
//! 4. **Cache Effectiveness**
//! 5. **Error Handling Impact**
//! 6. **Resource Contention**

use centotype_content::{ContentManager, CacheMetrics};
use centotype_core::{CentotypeCore, types::*};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::sync::{RwLock, mpsc};
use tokio::time::{interval, sleep};
use tracing::{debug, info, warn, error, instrument, Span};

/// Central metrics coordinator that aggregates data from all crates
pub struct InterCrateMetricsCollector {
    /// Metrics storage
    metrics_storage: Arc<RwLock<MetricsStorage>>,
    /// Real-time metric channels
    metric_channels: MetricChannels,
    /// Performance alert system
    alert_system: Arc<PerformanceAlertSystem>,
    /// Configuration
    config: MetricsConfig,
    /// Background collection task handle
    collection_task: Option<tokio::task::JoinHandle<()>>,
}

/// Configuration for metrics collection
#[derive(Debug, Clone)]
pub struct MetricsConfig {
    /// How often to collect and aggregate metrics
    pub collection_interval: Duration,
    /// Maximum number of metric samples to retain
    pub max_samples_retained: usize,
    /// Performance thresholds for alerting
    pub performance_thresholds: PerformanceThresholds,
    /// Enable detailed async boundary tracking
    pub enable_async_boundary_tracking: bool,
    /// Enable memory allocation tracking
    pub enable_memory_tracking: bool,
    /// Enable error impact analysis
    pub enable_error_impact_tracking: bool,
}

impl Default for MetricsConfig {
    fn default() -> Self {
        Self {
            collection_interval: Duration::from_millis(100),
            max_samples_retained: 1000,
            performance_thresholds: PerformanceThresholds::default(),
            enable_async_boundary_tracking: true,
            enable_memory_tracking: true,
            enable_error_impact_tracking: true,
        }
    }
}

/// Performance thresholds for alerting
#[derive(Debug, Clone)]
pub struct PerformanceThresholds {
    /// P99 latency warning threshold
    pub p99_latency_warning: Duration,
    /// P99 latency critical threshold
    pub p99_latency_critical: Duration,
    /// Memory usage warning threshold (bytes)
    pub memory_warning: usize,
    /// Memory usage critical threshold (bytes)
    pub memory_critical: usize,
    /// Cache hit rate warning threshold (percentage)
    pub cache_hit_rate_warning: f64,
    /// Error rate warning threshold (errors per minute)
    pub error_rate_warning: f64,
}

impl Default for PerformanceThresholds {
    fn default() -> Self {
        Self {
            p99_latency_warning: Duration::from_millis(20),
            p99_latency_critical: Duration::from_millis(25),
            memory_warning: 40 * 1024 * 1024,      // 40MB
            memory_critical: 50 * 1024 * 1024,     // 50MB
            cache_hit_rate_warning: 85.0,           // 85%
            error_rate_warning: 10.0,              // 10 errors/minute
        }
    }
}

/// Channels for real-time metric collection
#[derive(Debug)]
pub struct MetricChannels {
    /// Cross-crate latency measurements
    pub latency_tx: mpsc::UnboundedSender<LatencyMeasurement>,
    pub latency_rx: Arc<RwLock<mpsc::UnboundedReceiver<LatencyMeasurement>>>,

    /// Memory usage measurements
    pub memory_tx: mpsc::UnboundedSender<MemoryMeasurement>,
    pub memory_rx: Arc<RwLock<mpsc::UnboundedReceiver<MemoryMeasurement>>>,

    /// Error impact measurements
    pub error_tx: mpsc::UnboundedSender<ErrorMeasurement>,
    pub error_rx: Arc<RwLock<mpsc::UnboundedReceiver<ErrorMeasurement>>>,

    /// Cache performance measurements
    pub cache_tx: mpsc::UnboundedSender<CacheMeasurement>,
    pub cache_rx: Arc<RwLock<mpsc::UnboundedReceiver<CacheMeasurement>>>,
}

impl MetricChannels {
    pub fn new() -> Self {
        let (latency_tx, latency_rx) = mpsc::unbounded_channel();
        let (memory_tx, memory_rx) = mpsc::unbounded_channel();
        let (error_tx, error_rx) = mpsc::unbounded_channel();
        let (cache_tx, cache_rx) = mpsc::unbounded_channel();

        Self {
            latency_tx,
            latency_rx: Arc::new(RwLock::new(latency_rx)),
            memory_tx,
            memory_rx: Arc::new(RwLock::new(memory_rx)),
            error_tx,
            error_rx: Arc::new(RwLock::new(error_rx)),
            cache_tx,
            cache_rx: Arc::new(RwLock::new(cache_rx)),
        }
    }
}

/// Individual latency measurement across crate boundaries
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyMeasurement {
    pub timestamp: SystemTime,
    pub operation: String,
    pub from_crate: CrateName,
    pub to_crate: CrateName,
    pub latency: Duration,
    pub context: HashMap<String, String>,
    pub span_id: Option<String>,
}

/// Memory usage measurement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryMeasurement {
    pub timestamp: SystemTime,
    pub crate_name: CrateName,
    pub operation: String,
    pub rss_bytes: usize,
    pub heap_bytes: usize,
    pub allocation_delta: i64,
    pub context: HashMap<String, String>,
}

/// Error impact measurement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ErrorMeasurement {
    pub timestamp: SystemTime,
    pub crate_name: CrateName,
    pub error_type: String,
    pub recovery_latency: Duration,
    pub impact_scope: ErrorImpactScope,
    pub context: HashMap<String, String>,
}

/// Cache performance measurement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheMeasurement {
    pub timestamp: SystemTime,
    pub operation: CacheOperation,
    pub latency: Duration,
    pub hit: bool,
    pub cache_size: usize,
    pub memory_usage: usize,
    pub context: HashMap<String, String>,
}

/// Crate names for tracking
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Hash)]
pub enum CrateName {
    Cli,
    Engine,
    Core,
    Content,
    Analytics,
    Persistence,
    Platform,
}

impl std::fmt::Display for CrateName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            CrateName::Cli => write!(f, "cli"),
            CrateName::Engine => write!(f, "engine"),
            CrateName::Core => write!(f, "core"),
            CrateName::Content => write!(f, "content"),
            CrateName::Analytics => write!(f, "analytics"),
            CrateName::Persistence => write!(f, "persistence"),
            CrateName::Platform => write!(f, "platform"),
        }
    }
}

/// Error impact scope
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ErrorImpactScope {
    Local,        // Error contained within single crate
    CrossCrate,   // Error affects multiple crates
    Session,      // Error affects entire session
    System,       // Error affects system stability
}

/// Cache operation types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum CacheOperation {
    Lookup,
    Store,
    Eviction,
    Preload,
    Invalidation,
}

/// Aggregated metrics storage
#[derive(Debug, Default)]
pub struct MetricsStorage {
    /// Time-series latency data
    pub latency_series: VecDeque<AggregatedLatencyMetrics>,
    /// Memory usage time-series
    pub memory_series: VecDeque<AggregatedMemoryMetrics>,
    /// Error impact time-series
    pub error_series: VecDeque<AggregatedErrorMetrics>,
    /// Cache performance time-series
    pub cache_series: VecDeque<AggregatedCacheMetrics>,
    /// Cross-crate dependency analysis
    pub dependency_analysis: CrossCrateDependencyAnalysis,
    /// Real-time alerts
    pub active_alerts: Vec<PerformanceAlert>,
}

/// Aggregated latency metrics for a time window
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedLatencyMetrics {
    pub timestamp: SystemTime,
    pub window_duration: Duration,
    pub total_operations: u64,
    pub p50_latency: Duration,
    pub p95_latency: Duration,
    pub p99_latency: Duration,
    pub max_latency: Duration,
    pub cross_crate_breakdown: HashMap<(CrateName, CrateName), LatencyStats>,
    pub operation_breakdown: HashMap<String, LatencyStats>,
}

/// Latency statistics for a specific category
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyStats {
    pub count: u64,
    pub mean: Duration,
    pub p95: Duration,
    pub p99: Duration,
    pub max: Duration,
}

/// Aggregated memory metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedMemoryMetrics {
    pub timestamp: SystemTime,
    pub total_rss_bytes: usize,
    pub total_heap_bytes: usize,
    pub peak_usage: usize,
    pub allocation_rate: f64, // Allocations per second
    pub per_crate_usage: HashMap<CrateName, MemoryUsageStats>,
    pub memory_pressure_score: f64, // 0.0 to 1.0
}

/// Memory usage statistics per crate
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryUsageStats {
    pub current_rss: usize,
    pub current_heap: usize,
    pub peak_rss: usize,
    pub allocation_delta: i64,
    pub allocation_rate: f64,
}

/// Aggregated error metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedErrorMetrics {
    pub timestamp: SystemTime,
    pub total_errors: u64,
    pub error_rate_per_minute: f64,
    pub avg_recovery_latency: Duration,
    pub error_distribution: HashMap<String, u64>,
    pub impact_distribution: HashMap<ErrorImpactScope, u64>,
    pub error_correlation_score: f64, // How errors correlate with latency
}

/// Aggregated cache metrics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AggregatedCacheMetrics {
    pub timestamp: SystemTime,
    pub total_operations: u64,
    pub hit_rate: f64,
    pub miss_rate: f64,
    pub avg_lookup_latency: Duration,
    pub cache_efficiency_score: f64, // Overall cache effectiveness
    pub operation_breakdown: HashMap<CacheOperation, CacheOpStats>,
}

/// Cache operation statistics
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CacheOpStats {
    pub count: u64,
    pub avg_latency: Duration,
    pub success_rate: f64,
}

/// Cross-crate dependency analysis
#[derive(Debug, Default, Clone)]
pub struct CrossCrateDependencyAnalysis {
    /// Call frequency between crates
    pub call_matrix: HashMap<(CrateName, CrateName), u64>,
    /// Average latency between crates
    pub latency_matrix: HashMap<(CrateName, CrateName), Duration>,
    /// Critical path analysis
    pub critical_paths: Vec<CriticalPath>,
    /// Bottleneck identification
    pub bottlenecks: Vec<PerformanceBottleneck>,
}

/// Critical path through the system
#[derive(Debug, Clone)]
pub struct CriticalPath {
    pub path: Vec<CrateName>,
    pub total_latency: Duration,
    pub bottleneck_crate: CrateName,
    pub frequency: u64,
}

/// Performance bottleneck identification
#[derive(Debug, Clone)]
pub struct PerformanceBottleneck {
    pub crate_name: CrateName,
    pub operation: String,
    pub avg_latency: Duration,
    pub frequency: u64,
    pub impact_score: f64, // 0.0 to 1.0
}

/// Performance alert system
pub struct PerformanceAlertSystem {
    config: MetricsConfig,
    alert_history: Arc<RwLock<VecDeque<PerformanceAlert>>>,
}

/// Performance alert
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceAlert {
    pub timestamp: SystemTime,
    pub severity: AlertSeverity,
    pub category: AlertCategory,
    pub message: String,
    pub metric_value: f64,
    pub threshold_value: f64,
    pub suggested_action: String,
    pub context: HashMap<String, String>,
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Critical,
}

/// Alert categories
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertCategory {
    Latency,
    Memory,
    CachePerformance,
    ErrorRate,
    ResourceContention,
}

impl InterCrateMetricsCollector {
    /// Create new metrics collector
    pub fn new(config: MetricsConfig) -> Self {
        let metrics_storage = Arc::new(RwLock::new(MetricsStorage::default()));
        let metric_channels = MetricChannels::new();
        let alert_system = Arc::new(PerformanceAlertSystem::new(config.clone()));

        Self {
            metrics_storage,
            metric_channels,
            alert_system,
            config,
            collection_task: None,
        }
    }

    /// Start background metrics collection
    #[instrument(skip(self))]
    pub async fn start_collection(&mut self) -> Result<()> {
        info!("Starting inter-crate metrics collection");

        let metrics_storage = self.metrics_storage.clone();
        let latency_rx = self.metric_channels.latency_rx.clone();
        let memory_rx = self.metric_channels.memory_rx.clone();
        let error_rx = self.metric_channels.error_rx.clone();
        let cache_rx = self.metric_channels.cache_rx.clone();
        let alert_system = self.alert_system.clone();
        let config = self.config.clone();

        let collection_task = tokio::spawn(async move {
            let mut collection_interval = interval(config.collection_interval);
            let mut latency_buffer = Vec::new();
            let mut memory_buffer = Vec::new();
            let mut error_buffer = Vec::new();
            let mut cache_buffer = Vec::new();

            loop {
                collection_interval.tick().await;

                // Collect all pending measurements
                Self::collect_latency_measurements(&latency_rx, &mut latency_buffer).await;
                Self::collect_memory_measurements(&memory_rx, &mut memory_buffer).await;
                Self::collect_error_measurements(&error_rx, &mut error_buffer).await;
                Self::collect_cache_measurements(&cache_rx, &mut cache_buffer).await;

                // Aggregate and store metrics
                if !latency_buffer.is_empty() || !memory_buffer.is_empty()
                   || !error_buffer.is_empty() || !cache_buffer.is_empty() {

                    Self::aggregate_and_store_metrics(
                        &metrics_storage,
                        &alert_system,
                        &config,
                        latency_buffer.clone(),
                        memory_buffer.clone(),
                        error_buffer.clone(),
                        cache_buffer.clone(),
                    ).await;

                    // Clear buffers after processing
                    latency_buffer.clear();
                    memory_buffer.clear();
                    error_buffer.clear();
                    cache_buffer.clear();
                }

                debug!("Metrics collection cycle completed");
            }
        });

        self.collection_task = Some(collection_task);
        info!("Metrics collection started successfully");
        Ok(())
    }

    /// Collect latency measurements from channel
    async fn collect_latency_measurements(
        rx: &Arc<RwLock<mpsc::UnboundedReceiver<LatencyMeasurement>>>,
        buffer: &mut Vec<LatencyMeasurement>,
    ) {
        let mut rx_guard = rx.write().await;
        while let Ok(measurement) = rx_guard.try_recv() {
            buffer.push(measurement);
        }
    }

    /// Collect memory measurements from channel
    async fn collect_memory_measurements(
        rx: &Arc<RwLock<mpsc::UnboundedReceiver<MemoryMeasurement>>>,
        buffer: &mut Vec<MemoryMeasurement>,
    ) {
        let mut rx_guard = rx.write().await;
        while let Ok(measurement) = rx_guard.try_recv() {
            buffer.push(measurement);
        }
    }

    /// Collect error measurements from channel
    async fn collect_error_measurements(
        rx: &Arc<RwLock<mpsc::UnboundedReceiver<ErrorMeasurement>>>,
        buffer: &mut Vec<ErrorMeasurement>,
    ) {
        let mut rx_guard = rx.write().await;
        while let Ok(measurement) = rx_guard.try_recv() {
            buffer.push(measurement);
        }
    }

    /// Collect cache measurements from channel
    async fn collect_cache_measurements(
        rx: &Arc<RwLock<mpsc::UnboundedReceiver<CacheMeasurement>>>,
        buffer: &mut Vec<CacheMeasurement>,
    ) {
        let mut rx_guard = rx.write().await;
        while let Ok(measurement) = rx_guard.try_recv() {
            buffer.push(measurement);
        }
    }

    /// Aggregate measurements and store in time-series
    async fn aggregate_and_store_metrics(
        storage: &Arc<RwLock<MetricsStorage>>,
        alert_system: &Arc<PerformanceAlertSystem>,
        config: &MetricsConfig,
        latency_measurements: Vec<LatencyMeasurement>,
        memory_measurements: Vec<MemoryMeasurement>,
        error_measurements: Vec<ErrorMeasurement>,
        cache_measurements: Vec<CacheMeasurement>,
    ) {
        let timestamp = SystemTime::now();
        let window_duration = config.collection_interval;

        // Aggregate latency metrics
        let latency_metrics = Self::aggregate_latency_metrics(
            timestamp,
            window_duration,
            latency_measurements,
        );

        // Aggregate memory metrics
        let memory_metrics = Self::aggregate_memory_metrics(
            timestamp,
            memory_measurements,
        );

        // Aggregate error metrics
        let error_metrics = Self::aggregate_error_metrics(
            timestamp,
            error_measurements,
        );

        // Aggregate cache metrics
        let cache_metrics = Self::aggregate_cache_metrics(
            timestamp,
            cache_measurements,
        );

        // Store aggregated metrics
        {
            let mut storage_guard = storage.write().await;

            if let Some(latency) = latency_metrics {
                storage_guard.latency_series.push_back(latency.clone());

                // Check for latency alerts
                alert_system.check_latency_alerts(&latency).await;
            }

            if let Some(memory) = memory_metrics {
                storage_guard.memory_series.push_back(memory.clone());

                // Check for memory alerts
                alert_system.check_memory_alerts(&memory).await;
            }

            if let Some(error) = error_metrics {
                storage_guard.error_series.push_back(error.clone());

                // Check for error rate alerts
                alert_system.check_error_alerts(&error).await;
            }

            if let Some(cache) = cache_metrics {
                storage_guard.cache_series.push_back(cache.clone());

                // Check for cache performance alerts
                alert_system.check_cache_alerts(&cache).await;
            }

            // Trim old data to stay within retention limits
            Self::trim_old_metrics(&mut storage_guard, config.max_samples_retained);

            // Update dependency analysis
            Self::update_dependency_analysis(&mut storage_guard);
        }
    }

    /// Aggregate latency measurements
    fn aggregate_latency_metrics(
        timestamp: SystemTime,
        window_duration: Duration,
        measurements: Vec<LatencyMeasurement>,
    ) -> Option<AggregatedLatencyMetrics> {
        if measurements.is_empty() {
            return None;
        }

        let mut latencies: Vec<Duration> = measurements.iter().map(|m| m.latency).collect();
        latencies.sort();

        let len = latencies.len();
        let p50_latency = latencies[len * 50 / 100];
        let p95_latency = latencies[len * 95 / 100];
        let p99_latency = latencies[len * 99 / 100];
        let max_latency = latencies[len - 1];

        // Cross-crate breakdown
        let mut cross_crate_breakdown = HashMap::new();
        for measurement in &measurements {
            let key = (measurement.from_crate.clone(), measurement.to_crate.clone());
            let stats = cross_crate_breakdown.entry(key).or_insert_with(|| LatencyStats {
                count: 0,
                mean: Duration::ZERO,
                p95: Duration::ZERO,
                p99: Duration::ZERO,
                max: Duration::ZERO,
            });
            stats.count += 1;
            stats.max = stats.max.max(measurement.latency);
        }

        // Operation breakdown
        let mut operation_breakdown = HashMap::new();
        for measurement in &measurements {
            let stats = operation_breakdown.entry(measurement.operation.clone()).or_insert_with(|| LatencyStats {
                count: 0,
                mean: Duration::ZERO,
                p95: Duration::ZERO,
                p99: Duration::ZERO,
                max: Duration::ZERO,
            });
            stats.count += 1;
            stats.max = stats.max.max(measurement.latency);
        }

        Some(AggregatedLatencyMetrics {
            timestamp,
            window_duration,
            total_operations: measurements.len() as u64,
            p50_latency,
            p95_latency,
            p99_latency,
            max_latency,
            cross_crate_breakdown,
            operation_breakdown,
        })
    }

    /// Aggregate memory measurements
    fn aggregate_memory_metrics(
        timestamp: SystemTime,
        measurements: Vec<MemoryMeasurement>,
    ) -> Option<AggregatedMemoryMetrics> {
        if measurements.is_empty() {
            return None;
        }

        let total_rss_bytes = measurements.iter().map(|m| m.rss_bytes).sum();
        let total_heap_bytes = measurements.iter().map(|m| m.heap_bytes).sum();
        let peak_usage = measurements.iter().map(|m| m.rss_bytes).max().unwrap_or(0);

        // Calculate allocation rate (allocations per second)
        let total_allocations: i64 = measurements.iter().map(|m| m.allocation_delta.abs()).sum();
        let allocation_rate = total_allocations as f64; // Simplified calculation

        // Per-crate usage
        let mut per_crate_usage = HashMap::new();
        for measurement in &measurements {
            let stats = per_crate_usage.entry(measurement.crate_name.clone()).or_insert_with(|| MemoryUsageStats {
                current_rss: 0,
                current_heap: 0,
                peak_rss: 0,
                allocation_delta: 0,
                allocation_rate: 0.0,
            });
            stats.current_rss = measurement.rss_bytes;
            stats.current_heap = measurement.heap_bytes;
            stats.peak_rss = stats.peak_rss.max(measurement.rss_bytes);
            stats.allocation_delta += measurement.allocation_delta;
        }

        // Memory pressure score (simplified)
        let memory_pressure_score = (peak_usage as f64 / (50.0 * 1024.0 * 1024.0)).min(1.0);

        Some(AggregatedMemoryMetrics {
            timestamp,
            total_rss_bytes,
            total_heap_bytes,
            peak_usage,
            allocation_rate,
            per_crate_usage,
            memory_pressure_score,
        })
    }

    /// Aggregate error measurements
    fn aggregate_error_metrics(
        timestamp: SystemTime,
        measurements: Vec<ErrorMeasurement>,
    ) -> Option<AggregatedErrorMetrics> {
        if measurements.is_empty() {
            return None;
        }

        let total_errors = measurements.len() as u64;
        let error_rate_per_minute = total_errors as f64; // Simplified - would need time window

        let avg_recovery_latency = measurements.iter()
            .map(|m| m.recovery_latency)
            .sum::<Duration>() / measurements.len() as u32;

        // Error distribution
        let mut error_distribution = HashMap::new();
        for measurement in &measurements {
            *error_distribution.entry(measurement.error_type.clone()).or_insert(0) += 1;
        }

        // Impact distribution
        let mut impact_distribution = HashMap::new();
        for measurement in &measurements {
            *impact_distribution.entry(measurement.impact_scope.clone()).or_insert(0) += 1;
        }

        // Error correlation score (simplified)
        let error_correlation_score = 0.5; // Would calculate correlation with latency

        Some(AggregatedErrorMetrics {
            timestamp,
            total_errors,
            error_rate_per_minute,
            avg_recovery_latency,
            error_distribution,
            impact_distribution,
            error_correlation_score,
        })
    }

    /// Aggregate cache measurements
    fn aggregate_cache_metrics(
        timestamp: SystemTime,
        measurements: Vec<CacheMeasurement>,
    ) -> Option<AggregatedCacheMetrics> {
        if measurements.is_empty() {
            return None;
        }

        let total_operations = measurements.len() as u64;
        let hits = measurements.iter().filter(|m| m.hit).count() as u64;
        let hit_rate = (hits as f64 / total_operations as f64) * 100.0;
        let miss_rate = 100.0 - hit_rate;

        let avg_lookup_latency = measurements.iter()
            .map(|m| m.latency)
            .sum::<Duration>() / measurements.len() as u32;

        // Cache efficiency score
        let cache_efficiency_score = hit_rate / 100.0; // Simplified

        // Operation breakdown
        let mut operation_breakdown = HashMap::new();
        for measurement in &measurements {
            let stats = operation_breakdown.entry(measurement.operation.clone()).or_insert_with(|| CacheOpStats {
                count: 0,
                avg_latency: Duration::ZERO,
                success_rate: 0.0,
            });
            stats.count += 1;
            stats.avg_latency = (stats.avg_latency + measurement.latency) / 2; // Simplified average
        }

        Some(AggregatedCacheMetrics {
            timestamp,
            total_operations,
            hit_rate,
            miss_rate,
            avg_lookup_latency,
            cache_efficiency_score,
            operation_breakdown,
        })
    }

    /// Trim old metrics to stay within retention limits
    fn trim_old_metrics(storage: &mut MetricsStorage, max_samples: usize) {
        while storage.latency_series.len() > max_samples {
            storage.latency_series.pop_front();
        }
        while storage.memory_series.len() > max_samples {
            storage.memory_series.pop_front();
        }
        while storage.error_series.len() > max_samples {
            storage.error_series.pop_front();
        }
        while storage.cache_series.len() > max_samples {
            storage.cache_series.pop_front();
        }
    }

    /// Update cross-crate dependency analysis
    fn update_dependency_analysis(storage: &mut MetricsStorage) {
        // This would analyze the latest metrics to update dependency patterns
        // Simplified implementation for now
        debug!("Updated cross-crate dependency analysis");
    }

    /// Get metric collector for a specific crate
    pub fn get_crate_collector(&self, crate_name: CrateName) -> CrateMetricsCollector {
        CrateMetricsCollector::new(
            crate_name,
            self.metric_channels.latency_tx.clone(),
            self.metric_channels.memory_tx.clone(),
            self.metric_channels.error_tx.clone(),
            self.metric_channels.cache_tx.clone(),
        )
    }

    /// Get current performance summary
    pub async fn get_performance_summary(&self) -> PerformanceSummary {
        let storage = self.metrics_storage.read().await;

        let latest_latency = storage.latency_series.back();
        let latest_memory = storage.memory_series.back();
        let latest_cache = storage.cache_series.back();
        let latest_error = storage.error_series.back();

        PerformanceSummary {
            current_p99_latency: latest_latency.map(|l| l.p99_latency).unwrap_or(Duration::ZERO),
            current_memory_usage: latest_memory.map(|m| m.total_rss_bytes).unwrap_or(0),
            current_cache_hit_rate: latest_cache.map(|c| c.hit_rate).unwrap_or(0.0),
            current_error_rate: latest_error.map(|e| e.error_rate_per_minute).unwrap_or(0.0),
            active_alerts: storage.active_alerts.len(),
            target_compliance: latest_latency
                .map(|l| l.p99_latency < self.config.performance_thresholds.p99_latency_critical)
                .unwrap_or(false),
        }
    }

    /// Stop metrics collection
    pub async fn stop_collection(&mut self) {
        if let Some(task) = self.collection_task.take() {
            task.abort();
            info!("Metrics collection stopped");
        }
    }
}

/// Per-crate metrics collector
pub struct CrateMetricsCollector {
    crate_name: CrateName,
    latency_tx: mpsc::UnboundedSender<LatencyMeasurement>,
    memory_tx: mpsc::UnboundedSender<MemoryMeasurement>,
    error_tx: mpsc::UnboundedSender<ErrorMeasurement>,
    cache_tx: mpsc::UnboundedSender<CacheMeasurement>,
}

impl CrateMetricsCollector {
    pub fn new(
        crate_name: CrateName,
        latency_tx: mpsc::UnboundedSender<LatencyMeasurement>,
        memory_tx: mpsc::UnboundedSender<MemoryMeasurement>,
        error_tx: mpsc::UnboundedSender<ErrorMeasurement>,
        cache_tx: mpsc::UnboundedSender<CacheMeasurement>,
    ) -> Self {
        Self {
            crate_name,
            latency_tx,
            memory_tx,
            error_tx,
            cache_tx,
        }
    }

    /// Record cross-crate latency measurement
    #[instrument(skip(self))]
    pub fn record_cross_crate_latency(
        &self,
        operation: impl Into<String>,
        to_crate: CrateName,
        latency: Duration,
        context: HashMap<String, String>,
    ) {
        let measurement = LatencyMeasurement {
            timestamp: SystemTime::now(),
            operation: operation.into(),
            from_crate: self.crate_name.clone(),
            to_crate,
            latency,
            context,
            span_id: self.get_current_span_id(),
        };

        if let Err(e) = self.latency_tx.send(measurement) {
            error!("Failed to send latency measurement: {}", e);
        }
    }

    /// Record memory usage measurement
    pub fn record_memory_usage(
        &self,
        operation: impl Into<String>,
        rss_bytes: usize,
        heap_bytes: usize,
        allocation_delta: i64,
        context: HashMap<String, String>,
    ) {
        let measurement = MemoryMeasurement {
            timestamp: SystemTime::now(),
            crate_name: self.crate_name.clone(),
            operation: operation.into(),
            rss_bytes,
            heap_bytes,
            allocation_delta,
            context,
        };

        if let Err(e) = self.memory_tx.send(measurement) {
            error!("Failed to send memory measurement: {}", e);
        }
    }

    /// Record error impact measurement
    pub fn record_error_impact(
        &self,
        error_type: impl Into<String>,
        recovery_latency: Duration,
        impact_scope: ErrorImpactScope,
        context: HashMap<String, String>,
    ) {
        let measurement = ErrorMeasurement {
            timestamp: SystemTime::now(),
            crate_name: self.crate_name.clone(),
            error_type: error_type.into(),
            recovery_latency,
            impact_scope,
            context,
        };

        if let Err(e) = self.error_tx.send(measurement) {
            error!("Failed to send error measurement: {}", e);
        }
    }

    /// Record cache performance measurement
    pub fn record_cache_performance(
        &self,
        operation: CacheOperation,
        latency: Duration,
        hit: bool,
        cache_size: usize,
        memory_usage: usize,
        context: HashMap<String, String>,
    ) {
        let measurement = CacheMeasurement {
            timestamp: SystemTime::now(),
            operation,
            latency,
            hit,
            cache_size,
            memory_usage,
            context,
        };

        if let Err(e) = self.cache_tx.send(measurement) {
            error!("Failed to send cache measurement: {}", e);
        }
    }

    /// Get current tracing span ID
    fn get_current_span_id(&self) -> Option<String> {
        // Would integrate with tracing to get current span
        Some(format!("span-{}", uuid::Uuid::new_v4()))
    }
}

impl PerformanceAlertSystem {
    pub fn new(config: MetricsConfig) -> Self {
        Self {
            config,
            alert_history: Arc::new(RwLock::new(VecDeque::new())),
        }
    }

    /// Check for latency-based alerts
    async fn check_latency_alerts(&self, metrics: &AggregatedLatencyMetrics) {
        if metrics.p99_latency > self.config.performance_thresholds.p99_latency_critical {
            self.create_alert(
                AlertSeverity::Critical,
                AlertCategory::Latency,
                format!("P99 latency critical: {}ms", metrics.p99_latency.as_millis()),
                metrics.p99_latency.as_millis() as f64,
                self.config.performance_thresholds.p99_latency_critical.as_millis() as f64,
                "Investigate async boundary overhead and cache performance".to_string(),
                HashMap::new(),
            ).await;
        } else if metrics.p99_latency > self.config.performance_thresholds.p99_latency_warning {
            self.create_alert(
                AlertSeverity::Warning,
                AlertCategory::Latency,
                format!("P99 latency warning: {}ms", metrics.p99_latency.as_millis()),
                metrics.p99_latency.as_millis() as f64,
                self.config.performance_thresholds.p99_latency_warning.as_millis() as f64,
                "Monitor preloading effectiveness and consider optimization".to_string(),
                HashMap::new(),
            ).await;
        }
    }

    /// Check for memory-based alerts
    async fn check_memory_alerts(&self, metrics: &AggregatedMemoryMetrics) {
        if metrics.peak_usage > self.config.performance_thresholds.memory_critical {
            self.create_alert(
                AlertSeverity::Critical,
                AlertCategory::Memory,
                format!("Memory usage critical: {}MB", metrics.peak_usage / 1024 / 1024),
                metrics.peak_usage as f64,
                self.config.performance_thresholds.memory_critical as f64,
                "Investigate memory leaks and optimize cache size".to_string(),
                HashMap::new(),
            ).await;
        }
    }

    /// Check for error rate alerts
    async fn check_error_alerts(&self, metrics: &AggregatedErrorMetrics) {
        if metrics.error_rate_per_minute > self.config.performance_thresholds.error_rate_warning {
            self.create_alert(
                AlertSeverity::Warning,
                AlertCategory::ErrorRate,
                format!("Error rate elevated: {:.1}/min", metrics.error_rate_per_minute),
                metrics.error_rate_per_minute,
                self.config.performance_thresholds.error_rate_warning,
                "Review error patterns and improve error handling".to_string(),
                HashMap::new(),
            ).await;
        }
    }

    /// Check for cache performance alerts
    async fn check_cache_alerts(&self, metrics: &AggregatedCacheMetrics) {
        if metrics.hit_rate < self.config.performance_thresholds.cache_hit_rate_warning {
            self.create_alert(
                AlertSeverity::Warning,
                AlertCategory::CachePerformance,
                format!("Cache hit rate low: {:.1}%", metrics.hit_rate),
                metrics.hit_rate,
                self.config.performance_thresholds.cache_hit_rate_warning,
                "Optimize preloading strategy and cache capacity".to_string(),
                HashMap::new(),
            ).await;
        }
    }

    /// Create and store performance alert
    async fn create_alert(
        &self,
        severity: AlertSeverity,
        category: AlertCategory,
        message: String,
        metric_value: f64,
        threshold_value: f64,
        suggested_action: String,
        context: HashMap<String, String>,
    ) {
        let alert = PerformanceAlert {
            timestamp: SystemTime::now(),
            severity,
            category,
            message: message.clone(),
            metric_value,
            threshold_value,
            suggested_action,
            context,
        };

        {
            let mut history = self.alert_history.write().await;
            history.push_back(alert);

            // Keep only recent alerts
            while history.len() > 100 {
                history.pop_front();
            }
        }

        info!("Performance alert: {}", message);
    }
}

/// Performance summary for monitoring dashboards
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceSummary {
    pub current_p99_latency: Duration,
    pub current_memory_usage: usize,
    pub current_cache_hit_rate: f64,
    pub current_error_rate: f64,
    pub active_alerts: usize,
    pub target_compliance: bool,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_metrics_collector_creation() {
        let config = MetricsConfig::default();
        let collector = InterCrateMetricsCollector::new(config);

        let summary = collector.get_performance_summary().await;
        assert_eq!(summary.current_p99_latency, Duration::ZERO);
    }

    #[tokio::test]
    async fn test_crate_metrics_collector() {
        let config = MetricsConfig::default();
        let collector = InterCrateMetricsCollector::new(config);

        let crate_collector = collector.get_crate_collector(CrateName::Content);

        // Test recording latency
        crate_collector.record_cross_crate_latency(
            "test_operation",
            CrateName::Core,
            Duration::from_millis(10),
            HashMap::new(),
        );

        // Test recording memory usage
        crate_collector.record_memory_usage(
            "test_operation",
            1024 * 1024, // 1MB
            512 * 1024,  // 512KB
            100,         // 100 bytes allocated
            HashMap::new(),
        );
    }

    #[tokio::test]
    async fn test_alert_system() {
        let config = MetricsConfig::default();
        let alert_system = PerformanceAlertSystem::new(config.clone());

        // Create test latency metrics that exceed threshold
        let metrics = AggregatedLatencyMetrics {
            timestamp: SystemTime::now(),
            window_duration: Duration::from_secs(1),
            total_operations: 100,
            p50_latency: Duration::from_millis(15),
            p95_latency: Duration::from_millis(22),
            p99_latency: Duration::from_millis(30), // Exceeds critical threshold of 25ms
            max_latency: Duration::from_millis(35),
            cross_crate_breakdown: HashMap::new(),
            operation_breakdown: HashMap::new(),
        };

        alert_system.check_latency_alerts(&metrics).await;

        let history = alert_system.alert_history.read().await;
        assert_eq!(history.len(), 1);
        assert!(matches!(history[0].severity, AlertSeverity::Critical));
    }
}