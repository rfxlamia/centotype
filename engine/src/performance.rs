//! Advanced performance monitoring and hot-path latency profiling for the engine
use std::time::{Duration, Instant};
use std::collections::VecDeque;
use tracing::{debug, warn};

/// CircularBuffer for efficient high-frequency latency measurements
#[derive(Debug, Clone)]
pub struct CircularBuffer<T> {
    data: Vec<T>,
    head: usize,
    size: usize,
    capacity: usize,
}

impl<T: Clone + Default> CircularBuffer<T> {
    pub fn new(capacity: usize) -> Self {
        Self {
            data: vec![T::default(); capacity],
            head: 0,
            size: 0,
            capacity,
        }
    }

    pub fn push(&mut self, item: T) {
        self.data[self.head] = item;
        self.head = (self.head + 1) % self.capacity;
        if self.size < self.capacity {
            self.size += 1;
        }
    }

    pub fn iter(&self) -> Box<dyn Iterator<Item = &T> + '_> {
        if self.size < self.capacity {
            Box::new(self.data[..self.size].iter())
        } else {
            Box::new(self.data[self.head..].iter().chain(self.data[..self.head].iter()))
        }
    }

    pub fn len(&self) -> usize {
        self.size
    }

    pub fn is_empty(&self) -> bool {
        self.size == 0
    }
}

/// Detailed latency profiler for hot-path analysis
#[derive(Debug)]
pub struct LatencyProfiler {
    /// Input capture timing (crossterm event polling)
    input_capture_times: CircularBuffer<Duration>,
    /// Event processing timing (key event → action conversion)
    event_processing_times: CircularBuffer<Duration>,
    /// Scoring engine timing (core keystroke processing)
    scoring_times: CircularBuffer<Duration>,
    /// State update timing (session state changes)
    state_update_times: CircularBuffer<Duration>,
    /// Render timing (frame generation and terminal output)
    render_times: CircularBuffer<Duration>,
    /// Total end-to-end latency (keystroke → visual update)
    total_times: CircularBuffer<Duration>,
    /// Async boundary overhead tracking
    async_boundary_times: CircularBuffer<Duration>,
    /// Memory allocation tracking
    allocation_times: CircularBuffer<Duration>,
}

impl LatencyProfiler {
    pub fn new() -> Self {
        let capacity = 10000; // 10k samples for precise percentile calculation
        Self {
            input_capture_times: CircularBuffer::new(capacity),
            event_processing_times: CircularBuffer::new(capacity),
            scoring_times: CircularBuffer::new(capacity),
            state_update_times: CircularBuffer::new(capacity),
            render_times: CircularBuffer::new(capacity),
            total_times: CircularBuffer::new(capacity),
            async_boundary_times: CircularBuffer::new(capacity),
            allocation_times: CircularBuffer::new(capacity),
        }
    }

    /// Measure complete input cycle with detailed breakdown
    pub fn measure_input_cycle<F, R>(&mut self, f: F) -> (R, InputCycleBreakdown)
    where F: FnOnce(&mut InputCycleTimer) -> R {
        let total_start = Instant::now();
        let mut timer = InputCycleTimer::new();

        let result = f(&mut timer);
        let total_elapsed = total_start.elapsed();

        // Store measurements
        self.input_capture_times.push(timer.input_capture);
        self.event_processing_times.push(timer.event_processing);
        self.scoring_times.push(timer.scoring);
        self.state_update_times.push(timer.state_update);
        self.render_times.push(timer.render);
        self.total_times.push(total_elapsed);
        self.async_boundary_times.push(timer.async_boundary);
        self.allocation_times.push(timer.allocation);

        let breakdown = InputCycleBreakdown {
            input_capture: timer.input_capture,
            event_processing: timer.event_processing,
            scoring: timer.scoring,
            state_update: timer.state_update,
            render: timer.render,
            async_boundary: timer.async_boundary,
            allocation: timer.allocation,
            total: total_elapsed,
        };

        // Warn if any component exceeds targets
        if timer.input_capture > Duration::from_millis(5) {
            warn!("Input capture exceeds 5ms target: {:?}", timer.input_capture);
        }
        if timer.scoring > Duration::from_millis(5) {
            warn!("Scoring exceeds 5ms target: {:?}", timer.scoring);
        }
        if timer.render > Duration::from_millis(15) {
            warn!("Render exceeds 15ms target: {:?}", timer.render);
        }
        if total_elapsed > Duration::from_millis(25) {
            warn!("Total input cycle exceeds 25ms P99 target: {:?}", total_elapsed);
        }

        (result, breakdown)
    }

    /// Get comprehensive latency report with percentiles
    pub fn report_latency(&self) -> LatencyReport {
        LatencyReport {
            total_p99: self.calculate_percentile(&self.total_times, 0.99),
            total_p95: self.calculate_percentile(&self.total_times, 0.95),
            total_mean: self.calculate_mean(&self.total_times),

            input_p99: self.calculate_percentile(&self.input_capture_times, 0.99),
            event_processing_p99: self.calculate_percentile(&self.event_processing_times, 0.99),
            scoring_p99: self.calculate_percentile(&self.scoring_times, 0.99),
            state_update_p99: self.calculate_percentile(&self.state_update_times, 0.99),
            render_p99: self.calculate_percentile(&self.render_times, 0.99),

            async_boundary_overhead: self.calculate_mean(&self.async_boundary_times),
            allocation_overhead: self.calculate_mean(&self.allocation_times),

            sample_count: self.total_times.len(),
            target_compliance: self.assess_target_compliance(),
        }
    }

    /// Calculate percentile from circular buffer
    fn calculate_percentile(&self, buffer: &CircularBuffer<Duration>, percentile: f64) -> Duration {
        if buffer.is_empty() {
            return Duration::ZERO;
        }

        let mut values: Vec<Duration> = buffer.iter().cloned().collect();
        values.sort();

        let index = ((values.len() as f64 - 1.0) * percentile) as usize;
        values[index.min(values.len() - 1)]
    }

    /// Calculate mean from circular buffer
    fn calculate_mean(&self, buffer: &CircularBuffer<Duration>) -> Duration {
        if buffer.is_empty() {
            return Duration::ZERO;
        }

        let sum: Duration = buffer.iter().sum();
        sum / buffer.len() as u32
    }

    /// Assess compliance with performance targets
    fn assess_target_compliance(&self) -> TargetCompliance {
        let total_p99 = self.calculate_percentile(&self.total_times, 0.99);
        let input_p99 = self.calculate_percentile(&self.input_capture_times, 0.99);
        let scoring_p99 = self.calculate_percentile(&self.scoring_times, 0.99);
        let render_p99 = self.calculate_percentile(&self.render_times, 0.99);

        TargetCompliance {
            total_target_met: total_p99 <= Duration::from_millis(25),
            input_target_met: input_p99 <= Duration::from_millis(5),
            scoring_target_met: scoring_p99 <= Duration::from_millis(5),
            render_target_met: render_p99 <= Duration::from_millis(15),
            total_margin_ms: 25 - total_p99.as_millis() as i64,
            critical_path_bottleneck: self.identify_bottleneck(),
        }
    }

    /// Identify the primary bottleneck in the hot path
    fn identify_bottleneck(&self) -> HotPathBottleneck {
        let input_p99 = self.calculate_percentile(&self.input_capture_times, 0.99);
        let scoring_p99 = self.calculate_percentile(&self.scoring_times, 0.99);
        let render_p99 = self.calculate_percentile(&self.render_times, 0.99);
        let async_p99 = self.calculate_percentile(&self.async_boundary_times, 0.99);

        if render_p99 > input_p99 && render_p99 > scoring_p99 && render_p99 > async_p99 {
            HotPathBottleneck::Render(render_p99)
        } else if scoring_p99 > input_p99 && scoring_p99 > async_p99 {
            HotPathBottleneck::Scoring(scoring_p99)
        } else if async_p99 > input_p99 {
            HotPathBottleneck::AsyncBoundary(async_p99)
        } else {
            HotPathBottleneck::InputCapture(input_p99)
        }
    }

    /// Reset all measurements
    pub fn reset(&mut self) {
        let capacity = 10000;
        self.input_capture_times = CircularBuffer::new(capacity);
        self.event_processing_times = CircularBuffer::new(capacity);
        self.scoring_times = CircularBuffer::new(capacity);
        self.state_update_times = CircularBuffer::new(capacity);
        self.render_times = CircularBuffer::new(capacity);
        self.total_times = CircularBuffer::new(capacity);
        self.async_boundary_times = CircularBuffer::new(capacity);
        self.allocation_times = CircularBuffer::new(capacity);
    }
}

/// Timer for measuring individual input cycle components
#[derive(Debug, Default)]
pub struct InputCycleTimer {
    pub input_capture: Duration,
    pub event_processing: Duration,
    pub scoring: Duration,
    pub state_update: Duration,
    pub render: Duration,
    pub async_boundary: Duration,
    pub allocation: Duration,
}

impl InputCycleTimer {
    pub fn new() -> Self {
        Self::default()
    }

    /// Time input capture phase
    pub fn time_input_capture<F, R>(&mut self, f: F) -> R
    where F: FnOnce() -> R {
        let start = Instant::now();
        let result = f();
        self.input_capture = start.elapsed();
        result
    }

    /// Time event processing phase
    pub fn time_event_processing<F, R>(&mut self, f: F) -> R
    where F: FnOnce() -> R {
        let start = Instant::now();
        let result = f();
        self.event_processing = start.elapsed();
        result
    }

    /// Time scoring engine phase
    pub fn time_scoring<F, R>(&mut self, f: F) -> R
    where F: FnOnce() -> R {
        let start = Instant::now();
        let result = f();
        self.scoring = start.elapsed();
        result
    }

    /// Time state update phase
    pub fn time_state_update<F, R>(&mut self, f: F) -> R
    where F: FnOnce() -> R {
        let start = Instant::now();
        let result = f();
        self.state_update = start.elapsed();
        result
    }

    /// Time render phase
    pub fn time_render<F, R>(&mut self, f: F) -> R
    where F: FnOnce() -> R {
        let start = Instant::now();
        let result = f();
        self.render = start.elapsed();
        result
    }

    /// Time async boundary crossing
    pub fn time_async_boundary<F, R>(&mut self, f: F) -> R
    where F: FnOnce() -> R {
        let start = Instant::now();
        let result = f();
        self.async_boundary = start.elapsed();
        result
    }

    /// Time memory allocation operations
    pub fn time_allocation<F, R>(&mut self, f: F) -> R
    where F: FnOnce() -> R {
        let start = Instant::now();
        let result = f();
        self.allocation = start.elapsed();
        result
    }
}

/// Performance monitor for engine operations with advanced profiling
pub struct Performance {
    /// Advanced latency profiler for hot-path analysis
    pub latency_profiler: LatencyProfiler,
    /// Legacy measurements for compatibility
    input_latencies: VecDeque<Duration>,
    processing_times: VecDeque<Duration>,
    render_times: VecDeque<Duration>,
    /// Total operations count
    total_operations: u64,
    /// Start time for overall performance tracking
    start_time: Instant,
}

impl Performance {
    /// Create new performance monitor with advanced profiling
    pub fn new() -> Self {
        Self {
            latency_profiler: LatencyProfiler::new(),
            input_latencies: VecDeque::new(),
            processing_times: VecDeque::new(),
            render_times: VecDeque::new(),
            total_operations: 0,
            start_time: Instant::now(),
        }
    }

    /// Record input latency measurement
    pub fn record_input_latency(&mut self, latency: Duration) {
        self.input_latencies.push_back(latency);
        if self.input_latencies.len() > 1000 {
            self.input_latencies.pop_front();
        }
        debug!("Input latency: {:?}", latency);
    }

    /// Record processing time measurement
    pub fn record_processing_time(&mut self, duration: Duration) {
        self.processing_times.push_back(duration);
        if self.processing_times.len() > 1000 {
            self.processing_times.pop_front();
        }
        self.total_operations += 1;
    }

    /// Record render time measurement
    pub fn record_render_time(&mut self, duration: Duration) {
        self.render_times.push_back(duration);
        if self.render_times.len() > 1000 {
            self.render_times.pop_front();
        }
    }

    /// Get comprehensive performance metrics including hot-path analysis
    pub fn get_metrics(&self) -> EnginePerformanceMetrics {
        let latency_report = self.latency_profiler.report_latency();

        EnginePerformanceMetrics {
            input_latency_p50: self.calculate_percentile(&self.input_latencies, 0.5),
            input_latency_p95: self.calculate_percentile(&self.input_latencies, 0.95),
            input_latency_p99: self.calculate_percentile(&self.input_latencies, 0.99),
            processing_time_avg: self.calculate_average(&self.processing_times),
            render_time_avg: self.calculate_average(&self.render_times),
            total_operations: self.total_operations,
            uptime: self.start_time.elapsed(),

            // Advanced metrics from latency profiler
            hot_path_p99: latency_report.total_p99,
            hot_path_compliance: latency_report.target_compliance.total_target_met,
            bottleneck_component: format!("{:?}", latency_report.target_compliance.critical_path_bottleneck),
            optimization_margin_ms: latency_report.target_compliance.total_margin_ms,
        }
    }

    /// Get detailed latency breakdown report
    pub fn get_latency_report(&self) -> LatencyReport {
        self.latency_profiler.report_latency()
    }

    /// Reset all measurements including advanced profiling
    pub fn reset(&mut self) {
        self.latency_profiler.reset();
        self.input_latencies.clear();
        self.processing_times.clear();
        self.render_times.clear();
        self.total_operations = 0;
        self.start_time = Instant::now();
    }

    // Private helper methods

    fn calculate_percentile(&self, data: &VecDeque<Duration>, percentile: f64) -> Duration {
        if data.is_empty() {
            return Duration::default();
        }

        let mut sorted: Vec<Duration> = data.iter().cloned().collect();
        sorted.sort();

        let index = ((data.len() as f64 - 1.0) * percentile) as usize;
        sorted[index.min(sorted.len() - 1)]
    }

    fn calculate_average(&self, data: &VecDeque<Duration>) -> Duration {
        if data.is_empty() {
            return Duration::default();
        }

        let sum: Duration = data.iter().sum();
        sum / data.len() as u32
    }
}

impl Default for Performance {
    fn default() -> Self {
        Self::new()
    }
}

impl Default for LatencyProfiler {
    fn default() -> Self {
        Self::new()
    }
}

/// Detailed input cycle breakdown timing
#[derive(Debug, Clone)]
pub struct InputCycleBreakdown {
    pub input_capture: Duration,
    pub event_processing: Duration,
    pub scoring: Duration,
    pub state_update: Duration,
    pub render: Duration,
    pub async_boundary: Duration,
    pub allocation: Duration,
    pub total: Duration,
}

/// Comprehensive latency report with percentiles and target analysis
#[derive(Debug, Clone)]
pub struct LatencyReport {
    pub total_p99: Duration,
    pub total_p95: Duration,
    pub total_mean: Duration,

    pub input_p99: Duration,
    pub event_processing_p99: Duration,
    pub scoring_p99: Duration,
    pub state_update_p99: Duration,
    pub render_p99: Duration,

    pub async_boundary_overhead: Duration,
    pub allocation_overhead: Duration,

    pub sample_count: usize,
    pub target_compliance: TargetCompliance,
}

/// Target compliance assessment
#[derive(Debug, Clone)]
pub struct TargetCompliance {
    pub total_target_met: bool,       // P99 ≤ 25ms
    pub input_target_met: bool,       // P99 ≤ 5ms
    pub scoring_target_met: bool,     // P99 ≤ 5ms
    pub render_target_met: bool,      // P99 ≤ 15ms
    pub total_margin_ms: i64,         // Margin from 25ms target (negative if over)
    pub critical_path_bottleneck: HotPathBottleneck,
}

/// Hot path bottleneck identification
#[derive(Debug, Clone)]
pub enum HotPathBottleneck {
    InputCapture(Duration),
    Scoring(Duration),
    Render(Duration),
    AsyncBoundary(Duration),
}

/// Enhanced engine performance metrics with hot-path analysis
#[derive(Debug, Clone)]
pub struct EnginePerformanceMetrics {
    // Legacy metrics for compatibility
    pub input_latency_p50: Duration,
    pub input_latency_p95: Duration,
    pub input_latency_p99: Duration,
    pub processing_time_avg: Duration,
    pub render_time_avg: Duration,
    pub total_operations: u64,
    pub uptime: Duration,

    // Advanced hot-path analysis
    pub hot_path_p99: Duration,
    pub hot_path_compliance: bool,
    pub bottleneck_component: String,
    pub optimization_margin_ms: i64,
}

impl EnginePerformanceMetrics {
    /// Check if performance targets are met
    pub fn meets_targets(&self) -> bool {
        self.hot_path_compliance && self.input_latency_p99 <= Duration::from_millis(25)
    }

    /// Get performance grade based on compliance
    pub fn performance_grade(&self) -> char {
        if self.hot_path_p99 <= Duration::from_millis(25) {
            'A'
        } else if self.hot_path_p99 <= Duration::from_millis(30) {
            'B'
        } else if self.hot_path_p99 <= Duration::from_millis(40) {
            'C'
        } else {
            'F'
        }
    }

    /// Get optimization recommendations
    pub fn optimization_recommendations(&self) -> Vec<String> {
        let mut recommendations = Vec::new();

        if !self.hot_path_compliance {
            recommendations.push(format!(
                "Critical: P99 latency {}ms exceeds 25ms target by {}ms",
                self.hot_path_p99.as_millis(),
                self.hot_path_p99.as_millis() as i64 - 25
            ));

            if self.bottleneck_component.contains("Render") {
                recommendations.push("Optimize render path: implement ANSI batching and line precomposition".to_string());
            } else if self.bottleneck_component.contains("Scoring") {
                recommendations.push("Optimize scoring engine: use arena allocation and reduce state copies".to_string());
            } else if self.bottleneck_component.contains("AsyncBoundary") {
                recommendations.push("Optimize async boundaries: reduce Arc cloning and RwLock contention".to_string());
            }
        }

        if self.optimization_margin_ms < 5 {
            recommendations.push("Margin too small: implement additional optimizations for consistency".to_string());
        }

        recommendations
    }
}
