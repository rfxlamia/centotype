//! # Input Latency Benchmark Suite
//!
//! Comprehensive benchmarking framework for validating P99 input latency <25ms target.
//! This benchmark suite provides precise measurement of keystroke-to-processing latency
//! across all performance-critical paths in the Centotype application.
//!
//! ## Key Performance Targets
//!
//! - **P99 Input Latency**: <25ms (keystroke to processing)
//! - **P95 Input Latency**: <15ms (target for optimal user experience)
//! - **Mean Input Latency**: <10ms (target for smooth typing feel)
//! - **Input Processing Consistency**: <5ms standard deviation
//!
//! ## Benchmark Categories
//!
//! 1. **Single Keystroke Latency**: Individual key processing time
//! 2. **Rapid Typing Simulation**: High-frequency input handling (10+ KPS)
//! 3. **Error Detection Latency**: Time to identify and highlight errors
//! 4. **Input Event Propagation**: Cross-crate event handling latency
//! 5. **Real-time Feedback Latency**: Time to visual update after input
//! 6. **Concurrent Input Handling**: Performance under load

use centotype_content::ContentManager;
use centotype_core::{CentotypeCore, types::*};
use centotype_engine::CentotypeEngine;
use centotype_platform::PlatformManager;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;
use tokio::sync::RwLock;
use tracing::{debug, info, warn, instrument};

/// Input latency benchmark configuration
#[derive(Debug, Clone)]
pub struct InputLatencyBenchmarkConfig {
    /// Number of samples for latency measurement
    pub sample_count: usize,
    /// Target P99 latency threshold
    pub p99_target: Duration,
    /// Target P95 latency threshold
    pub p95_target: Duration,
    /// Target mean latency threshold
    pub mean_target: Duration,
    /// Maximum acceptable standard deviation
    pub max_std_dev: Duration,
    /// Rapid typing simulation rate (keystrokes per second)
    pub rapid_typing_kps: f64,
    /// Concurrent input thread count for stress testing
    pub concurrent_threads: usize,
    /// Test duration for sustained performance
    pub sustained_test_duration: Duration,
}

impl Default for InputLatencyBenchmarkConfig {
    fn default() -> Self {
        Self {
            sample_count: 10000,
            p99_target: Duration::from_millis(25),
            p95_target: Duration::from_millis(15),
            mean_target: Duration::from_millis(10),
            max_std_dev: Duration::from_millis(5),
            rapid_typing_kps: 10.0,
            concurrent_threads: 8,
            sustained_test_duration: Duration::from_secs(60),
        }
    }
}

/// High-precision input latency measurements
#[derive(Debug, Clone)]
pub struct InputLatencyMeasurement {
    /// Raw latency samples (nanosecond precision)
    pub samples: Vec<Duration>,
    /// Statistical analysis of latency distribution
    pub statistics: LatencyStatistics,
    /// Performance target compliance
    pub target_compliance: TargetCompliance,
    /// Identified performance bottlenecks
    pub bottlenecks: Vec<InputBottleneck>,
    /// Test metadata
    pub metadata: BenchmarkMetadata,
}

/// Statistical analysis of input latency
#[derive(Debug, Clone)]
pub struct LatencyStatistics {
    pub mean: Duration,
    pub median: Duration,
    pub p95: Duration,
    pub p99: Duration,
    pub p999: Duration,
    pub min: Duration,
    pub max: Duration,
    pub std_dev: f64,
    pub variance: f64,
    pub range: Duration,
    pub outlier_count: usize,
}

/// Performance target compliance assessment
#[derive(Debug, Clone)]
pub struct TargetCompliance {
    pub p99_compliant: bool,
    pub p95_compliant: bool,
    pub mean_compliant: bool,
    pub std_dev_compliant: bool,
    pub overall_compliant: bool,
    pub margin_p99: i64, // Margin in microseconds (negative if over target)
    pub margin_p95: i64,
    pub margin_mean: i64,
}

/// Input processing bottleneck identification
#[derive(Debug, Clone)]
pub struct InputBottleneck {
    pub component: String,
    pub phase: InputPhase,
    pub avg_latency: Duration,
    pub impact_percentage: f64,
    pub occurrence_rate: f64,
    pub suggested_optimization: String,
}

/// Input processing phases for bottleneck analysis
#[derive(Debug, Clone)]
pub enum InputPhase {
    KeystrokeCapture,
    EventParsing,
    ValidationLogic,
    StateUpdate,
    ErrorDetection,
    FeedbackGeneration,
    UIUpdate,
    CrossCrateBoundary,
}

/// Benchmark execution metadata
#[derive(Debug, Clone)]
pub struct BenchmarkMetadata {
    pub test_name: String,
    pub execution_timestamp: Instant,
    pub config: InputLatencyBenchmarkConfig,
    pub environment_info: EnvironmentInfo,
    pub warmup_duration: Duration,
    pub actual_sample_count: usize,
}

/// Environment information for benchmark context
#[derive(Debug, Clone)]
pub struct EnvironmentInfo {
    pub platform: String,
    pub cpu_cores: usize,
    pub available_memory_mb: u64,
    pub is_debug_build: bool,
    pub compiler_version: String,
    pub target_architecture: String,
}

/// Input latency benchmark harness
pub struct InputLatencyBenchmarkHarness {
    config: InputLatencyBenchmarkConfig,
    content_manager: Arc<ContentManager>,
    core: Arc<CentotypeCore>,
    platform: Arc<PlatformManager>,
    runtime: Runtime,
}

impl InputLatencyBenchmarkHarness {
    /// Create new benchmark harness with specified configuration
    pub async fn new(config: InputLatencyBenchmarkConfig) -> Result<Self> {
        let runtime = Runtime::new().map_err(|e| CentotypeError::Internal(format!("Failed to create runtime: {}", e)))?;

        let content_manager = Arc::new(ContentManager::new().await?);
        let core = Arc::new(CentotypeCore::new());
        let platform = Arc::new(PlatformManager::new()?);

        Ok(Self {
            config,
            content_manager,
            core,
            platform,
            runtime,
        })
    }

    /// Run comprehensive input latency benchmark suite
    #[instrument(skip(self))]
    pub async fn run_comprehensive_benchmark(&self) -> Result<Vec<InputLatencyMeasurement>> {
        info!("Starting comprehensive input latency benchmark suite");

        let mut results = Vec::new();

        // 1. Single keystroke latency benchmark
        let single_keystroke = self.benchmark_single_keystroke_latency().await?;
        results.push(single_keystroke);

        // 2. Rapid typing simulation
        let rapid_typing = self.benchmark_rapid_typing_latency().await?;
        results.push(rapid_typing);

        // 3. Error detection latency
        let error_detection = self.benchmark_error_detection_latency().await?;
        results.push(error_detection);

        // 4. Input event propagation across crates
        let cross_crate = self.benchmark_cross_crate_propagation().await?;
        results.push(cross_crate);

        // 5. Real-time feedback latency
        let feedback_latency = self.benchmark_feedback_latency().await?;
        results.push(feedback_latency);

        // 6. Concurrent input handling
        let concurrent_handling = self.benchmark_concurrent_input_handling().await?;
        results.push(concurrent_handling);

        // 7. Sustained performance under load
        let sustained_performance = self.benchmark_sustained_input_performance().await?;
        results.push(sustained_performance);

        info!("Completed comprehensive input latency benchmark suite with {} test categories", results.len());
        Ok(results)
    }

    /// Benchmark single keystroke processing latency
    #[instrument(skip(self))]
    async fn benchmark_single_keystroke_latency(&self) -> Result<InputLatencyMeasurement> {
        info!("Benchmarking single keystroke latency");

        let mut samples = Vec::with_capacity(self.config.sample_count);
        let warmup_start = Instant::now();

        // Warmup phase to eliminate cold start effects
        for _ in 0..100 {
            let _ = self.measure_single_keystroke('a').await?;
        }
        let warmup_duration = warmup_start.elapsed();

        // Main measurement phase
        for i in 0..self.config.sample_count {
            let char_to_type = char::from(b'a' + (i % 26) as u8);
            let latency = self.measure_single_keystroke(char_to_type).await?;
            samples.push(latency);

            // Progress logging
            if i % 1000 == 0 && i > 0 {
                debug!("Single keystroke benchmark progress: {}/{}", i, self.config.sample_count);
            }
        }

        let statistics = Self::calculate_latency_statistics(&samples);
        let target_compliance = self.assess_target_compliance(&statistics);
        let bottlenecks = self.identify_input_bottlenecks(&samples, InputPhase::KeystrokeCapture).await;

        let metadata = BenchmarkMetadata {
            test_name: "single_keystroke_latency".to_string(),
            execution_timestamp: Instant::now(),
            config: self.config.clone(),
            environment_info: self.gather_environment_info(),
            warmup_duration,
            actual_sample_count: samples.len(),
        };

        Ok(InputLatencyMeasurement {
            samples,
            statistics,
            target_compliance,
            bottlenecks,
            metadata,
        })
    }

    /// Measure single keystroke processing latency with high precision
    async fn measure_single_keystroke(&self, keystroke: char) -> Result<Duration> {
        let start = Instant::now();

        // Simulate complete keystroke processing pipeline:
        // 1. Input capture (platform layer)
        let capture_start = Instant::now();
        let _platform_input = self.platform.capture_input_event(keystroke)?;
        let _capture_duration = capture_start.elapsed();

        // 2. Event parsing (engine layer)
        let parse_start = Instant::now();
        let _parsed_event = InputEvent::Character(keystroke);
        let _parse_duration = parse_start.elapsed();

        // 3. Core processing (validation, state update)
        let core_start = Instant::now();
        let _core_result = self.core.process_character_input(keystroke);
        let _core_duration = core_start.elapsed();

        // 4. Feedback generation
        let feedback_start = Instant::now();
        let _feedback = self.generate_input_feedback(keystroke).await;
        let _feedback_duration = feedback_start.elapsed();

        let total_latency = start.elapsed();
        Ok(total_latency)
    }

    /// Benchmark rapid typing simulation (high-frequency input)
    #[instrument(skip(self))]
    async fn benchmark_rapid_typing_latency(&self) -> Result<InputLatencyMeasurement> {
        info!("Benchmarking rapid typing latency at {} KPS", self.config.rapid_typing_kps);

        let keystroke_interval = Duration::from_secs_f64(1.0 / self.config.rapid_typing_kps);
        let test_duration = Duration::from_secs(10); // 10 seconds of rapid typing
        let expected_keystrokes = (test_duration.as_secs_f64() * self.config.rapid_typing_kps) as usize;

        let mut samples = Vec::with_capacity(expected_keystrokes);
        let test_start = Instant::now();
        let mut last_keystroke = Instant::now();

        while test_start.elapsed() < test_duration {
            if last_keystroke.elapsed() >= keystroke_interval {
                let char_index = samples.len() % 26;
                let keystroke = char::from(b'a' + char_index as u8);

                let latency = self.measure_single_keystroke(keystroke).await?;
                samples.push(latency);

                last_keystroke = Instant::now();
            }

            // Small yield to prevent busy waiting
            tokio::task::yield_now().await;
        }

        let statistics = Self::calculate_latency_statistics(&samples);
        let target_compliance = self.assess_target_compliance(&statistics);
        let bottlenecks = self.identify_input_bottlenecks(&samples, InputPhase::StateUpdate).await;

        let metadata = BenchmarkMetadata {
            test_name: format!("rapid_typing_{}kps", self.config.rapid_typing_kps),
            execution_timestamp: Instant::now(),
            config: self.config.clone(),
            environment_info: self.gather_environment_info(),
            warmup_duration: Duration::ZERO,
            actual_sample_count: samples.len(),
        };

        Ok(InputLatencyMeasurement {
            samples,
            statistics,
            target_compliance,
            bottlenecks,
            metadata,
        })
    }

    /// Benchmark error detection latency
    #[instrument(skip(self))]
    async fn benchmark_error_detection_latency(&self) -> Result<InputLatencyMeasurement> {
        info!("Benchmarking error detection latency");

        let mut samples = Vec::with_capacity(self.config.sample_count / 10); // Fewer samples for error scenarios

        // Set up test content for error detection
        let test_text = "The quick brown fox jumps over the lazy dog";
        let _ = self.core.load_test_content(test_text);

        for i in 0..(self.config.sample_count / 10) {
            let start = Instant::now();

            // Simulate incorrect keystroke (intentional error)
            let correct_char = test_text.chars().nth(i % test_text.len()).unwrap_or('a');
            let incorrect_char = if correct_char == 'a' { 'b' } else { 'a' };

            // Measure error detection latency
            let detection_start = Instant::now();
            let _error_result = self.core.process_character_input(incorrect_char);
            let _detection_time = detection_start.elapsed();

            // Measure error highlighting latency
            let highlight_start = Instant::now();
            let _highlight_result = self.generate_error_highlight().await;
            let _highlight_time = highlight_start.elapsed();

            let total_latency = start.elapsed();
            samples.push(total_latency);
        }

        let statistics = Self::calculate_latency_statistics(&samples);
        let target_compliance = self.assess_target_compliance(&statistics);
        let bottlenecks = self.identify_input_bottlenecks(&samples, InputPhase::ErrorDetection).await;

        let metadata = BenchmarkMetadata {
            test_name: "error_detection_latency".to_string(),
            execution_timestamp: Instant::now(),
            config: self.config.clone(),
            environment_info: self.gather_environment_info(),
            warmup_duration: Duration::ZERO,
            actual_sample_count: samples.len(),
        };

        Ok(InputLatencyMeasurement {
            samples,
            statistics,
            target_compliance,
            bottlenecks,
            metadata,
        })
    }

    /// Benchmark cross-crate input event propagation
    #[instrument(skip(self))]
    async fn benchmark_cross_crate_propagation(&self) -> Result<InputLatencyMeasurement> {
        info!("Benchmarking cross-crate input event propagation");

        let mut samples = Vec::with_capacity(self.config.sample_count);

        for i in 0..self.config.sample_count {
            let start = Instant::now();

            // Simulate full cross-crate input pipeline
            let keystroke = char::from(b'a' + (i % 26) as u8);

            // Platform -> Engine boundary
            let platform_start = Instant::now();
            let _platform_event = self.platform.capture_input_event(keystroke)?;
            let _platform_latency = platform_start.elapsed();

            // Engine -> Core boundary
            let engine_start = Instant::now();
            let _core_result = self.core.process_character_input(keystroke);
            let _engine_latency = engine_start.elapsed();

            // Core -> Content boundary (if content needs to be loaded)
            let content_start = Instant::now();
            if i % 100 == 0 {
                let level_id = LevelId::new(1)?;
                let _ = self.content_manager.get_level_content(level_id, None).await?;
            }
            let _content_latency = content_start.elapsed();

            let total_latency = start.elapsed();
            samples.push(total_latency);
        }

        let statistics = Self::calculate_latency_statistics(&samples);
        let target_compliance = self.assess_target_compliance(&statistics);
        let bottlenecks = self.identify_input_bottlenecks(&samples, InputPhase::CrossCrateBoundary).await;

        let metadata = BenchmarkMetadata {
            test_name: "cross_crate_propagation".to_string(),
            execution_timestamp: Instant::now(),
            config: self.config.clone(),
            environment_info: self.gather_environment_info(),
            warmup_duration: Duration::ZERO,
            actual_sample_count: samples.len(),
        };

        Ok(InputLatencyMeasurement {
            samples,
            statistics,
            target_compliance,
            bottlenecks,
            metadata,
        })
    }

    /// Benchmark real-time feedback latency
    #[instrument(skip(self))]
    async fn benchmark_feedback_latency(&self) -> Result<InputLatencyMeasurement> {
        info!("Benchmarking real-time feedback latency");

        let mut samples = Vec::with_capacity(self.config.sample_count);

        for i in 0..self.config.sample_count {
            let keystroke = char::from(b'a' + (i % 26) as u8);
            let start = Instant::now();

            // Input processing
            let _input_result = self.core.process_character_input(keystroke);

            // Feedback generation (visual updates, sound, etc.)
            let feedback_start = Instant::now();
            let _feedback = self.generate_input_feedback(keystroke).await;
            let _feedback_time = feedback_start.elapsed();

            // UI update latency
            let ui_start = Instant::now();
            let _ui_update = self.simulate_ui_update().await;
            let _ui_time = ui_start.elapsed();

            let total_latency = start.elapsed();
            samples.push(total_latency);
        }

        let statistics = Self::calculate_latency_statistics(&samples);
        let target_compliance = self.assess_target_compliance(&statistics);
        let bottlenecks = self.identify_input_bottlenecks(&samples, InputPhase::FeedbackGeneration).await;

        let metadata = BenchmarkMetadata {
            test_name: "feedback_latency".to_string(),
            execution_timestamp: Instant::now(),
            config: self.config.clone(),
            environment_info: self.gather_environment_info(),
            warmup_duration: Duration::ZERO,
            actual_sample_count: samples.len(),
        };

        Ok(InputLatencyMeasurement {
            samples,
            statistics,
            target_compliance,
            bottlenecks,
            metadata,
        })
    }

    /// Benchmark concurrent input handling performance
    #[instrument(skip(self))]
    async fn benchmark_concurrent_input_handling(&self) -> Result<InputLatencyMeasurement> {
        info!("Benchmarking concurrent input handling with {} threads", self.config.concurrent_threads);

        let samples = Arc::new(RwLock::new(Vec::new()));
        let mut handles = Vec::new();

        let operations_per_thread = self.config.sample_count / self.config.concurrent_threads;

        for thread_id in 0..self.config.concurrent_threads {
            let samples_clone = samples.clone();
            let core_clone = self.core.clone();
            let platform_clone = self.platform.clone();

            let handle = tokio::spawn(async move {
                for i in 0..operations_per_thread {
                    let keystroke = char::from(b'a' + ((thread_id * operations_per_thread + i) % 26) as u8);
                    let start = Instant::now();

                    // Simulate concurrent input processing
                    let _platform_result = platform_clone.capture_input_event(keystroke);
                    let _core_result = core_clone.process_character_input(keystroke);

                    let latency = start.elapsed();
                    samples_clone.write().await.push(latency);
                }
            });

            handles.push(handle);
        }

        // Wait for all threads to complete
        for handle in handles {
            let _ = handle.await;
        }

        let samples_vec = samples.read().await.clone();
        let statistics = Self::calculate_latency_statistics(&samples_vec);
        let target_compliance = self.assess_target_compliance(&statistics);
        let bottlenecks = self.identify_input_bottlenecks(&samples_vec, InputPhase::StateUpdate).await;

        let metadata = BenchmarkMetadata {
            test_name: format!("concurrent_input_{}threads", self.config.concurrent_threads),
            execution_timestamp: Instant::now(),
            config: self.config.clone(),
            environment_info: self.gather_environment_info(),
            warmup_duration: Duration::ZERO,
            actual_sample_count: samples_vec.len(),
        };

        Ok(InputLatencyMeasurement {
            samples: samples_vec,
            statistics,
            target_compliance,
            bottlenecks,
            metadata,
        })
    }

    /// Benchmark sustained input performance over extended period
    #[instrument(skip(self))]
    async fn benchmark_sustained_input_performance(&self) -> Result<InputLatencyMeasurement> {
        info!("Benchmarking sustained input performance for {:?}", self.config.sustained_test_duration);

        let mut samples = Vec::new();
        let test_start = Instant::now();
        let mut keystroke_count = 0;

        while test_start.elapsed() < self.config.sustained_test_duration {
            let keystroke = char::from(b'a' + (keystroke_count % 26) as u8);
            let latency = self.measure_single_keystroke(keystroke).await?;
            samples.push(latency);

            keystroke_count += 1;

            // Simulate realistic typing pace (not maximum speed)
            tokio::time::sleep(Duration::from_millis(100)).await; // 10 characters per second
        }

        let statistics = Self::calculate_latency_statistics(&samples);
        let target_compliance = self.assess_target_compliance(&statistics);
        let bottlenecks = self.identify_input_bottlenecks(&samples, InputPhase::StateUpdate).await;

        let metadata = BenchmarkMetadata {
            test_name: format!("sustained_performance_{}s", self.config.sustained_test_duration.as_secs()),
            execution_timestamp: Instant::now(),
            config: self.config.clone(),
            environment_info: self.gather_environment_info(),
            warmup_duration: Duration::ZERO,
            actual_sample_count: samples.len(),
        };

        Ok(InputLatencyMeasurement {
            samples,
            statistics,
            target_compliance,
            bottlenecks,
            metadata,
        })
    }

    /// Calculate comprehensive latency statistics
    fn calculate_latency_statistics(samples: &[Duration]) -> LatencyStatistics {
        if samples.is_empty() {
            return LatencyStatistics {
                mean: Duration::ZERO,
                median: Duration::ZERO,
                p95: Duration::ZERO,
                p99: Duration::ZERO,
                p999: Duration::ZERO,
                min: Duration::ZERO,
                max: Duration::ZERO,
                std_dev: 0.0,
                variance: 0.0,
                range: Duration::ZERO,
                outlier_count: 0,
            };
        }

        let mut sorted_samples = samples.to_vec();
        sorted_samples.sort();

        let len = sorted_samples.len();
        let mean = samples.iter().sum::<Duration>() / samples.len() as u32;
        let median = sorted_samples[len / 2];
        let p95 = sorted_samples[len * 95 / 100];
        let p99 = sorted_samples[len * 99 / 100];
        let p999 = sorted_samples[len * 999 / 1000];
        let min = sorted_samples[0];
        let max = sorted_samples[len - 1];
        let range = max - min;

        // Calculate variance and standard deviation
        let mean_nanos = mean.as_nanos() as f64;
        let variance: f64 = samples.iter()
            .map(|&duration| {
                let diff = duration.as_nanos() as f64 - mean_nanos;
                diff * diff
            })
            .sum::<f64>() / samples.len() as f64;
        let std_dev = variance.sqrt();

        // Count outliers (values more than 2 standard deviations from mean)
        let outlier_threshold = 2.0 * std_dev;
        let outlier_count = samples.iter()
            .filter(|&&duration| {
                let diff = (duration.as_nanos() as f64 - mean_nanos).abs();
                diff > outlier_threshold
            })
            .count();

        LatencyStatistics {
            mean,
            median,
            p95,
            p99,
            p999,
            min,
            max,
            std_dev,
            variance,
            range,
            outlier_count,
        }
    }

    /// Assess compliance with performance targets
    fn assess_target_compliance(&self, statistics: &LatencyStatistics) -> TargetCompliance {
        let p99_compliant = statistics.p99 <= self.config.p99_target;
        let p95_compliant = statistics.p95 <= self.config.p95_target;
        let mean_compliant = statistics.mean <= self.config.mean_target;
        let std_dev_compliant = Duration::from_nanos(statistics.std_dev as u64) <= self.config.max_std_dev;

        let overall_compliant = p99_compliant && p95_compliant && mean_compliant && std_dev_compliant;

        let margin_p99 = self.config.p99_target.as_micros() as i64 - statistics.p99.as_micros() as i64;
        let margin_p95 = self.config.p95_target.as_micros() as i64 - statistics.p95.as_micros() as i64;
        let margin_mean = self.config.mean_target.as_micros() as i64 - statistics.mean.as_micros() as i64;

        TargetCompliance {
            p99_compliant,
            p95_compliant,
            mean_compliant,
            std_dev_compliant,
            overall_compliant,
            margin_p99,
            margin_p95,
            margin_mean,
        }
    }

    /// Identify performance bottlenecks in input processing
    async fn identify_input_bottlenecks(&self, samples: &[Duration], phase: InputPhase) -> Vec<InputBottleneck> {
        let mut bottlenecks = Vec::new();

        let statistics = Self::calculate_latency_statistics(samples);

        // Identify bottlenecks based on performance deviation
        if statistics.p99 > self.config.p99_target {
            let impact_percentage = ((statistics.p99.as_micros() as f64 / self.config.p99_target.as_micros() as f64) - 1.0) * 100.0;

            bottlenecks.push(InputBottleneck {
                component: format!("{:?}", phase),
                phase: phase.clone(),
                avg_latency: statistics.mean,
                impact_percentage,
                occurrence_rate: 1.0, // Simplified
                suggested_optimization: Self::get_optimization_suggestion(&phase),
            });
        }

        bottlenecks
    }

    /// Get optimization suggestions for specific input phases
    fn get_optimization_suggestion(phase: &InputPhase) -> String {
        match phase {
            InputPhase::KeystrokeCapture => "Optimize platform input capture with native event handling".to_string(),
            InputPhase::EventParsing => "Implement event parsing caching and reduce allocations".to_string(),
            InputPhase::ValidationLogic => "Streamline validation logic and use lookup tables".to_string(),
            InputPhase::StateUpdate => "Optimize state update with copy-on-write semantics".to_string(),
            InputPhase::ErrorDetection => "Implement incremental error detection algorithms".to_string(),
            InputPhase::FeedbackGeneration => "Use async feedback generation to avoid blocking".to_string(),
            InputPhase::UIUpdate => "Implement differential UI updates and virtual scrolling".to_string(),
            InputPhase::CrossCrateBoundary => "Reduce serialization overhead and async boundary costs".to_string(),
        }
    }

    /// Gather environment information for benchmark context
    fn gather_environment_info(&self) -> EnvironmentInfo {
        EnvironmentInfo {
            platform: std::env::consts::OS.to_string(),
            cpu_cores: num_cpus::get(),
            available_memory_mb: 1024, // Simplified - would use platform-specific APIs
            is_debug_build: cfg!(debug_assertions),
            compiler_version: "rustc 1.75.0".to_string(), // Would get actual version
            target_architecture: std::env::consts::ARCH.to_string(),
        }
    }

    /// Generate input feedback (simulation)
    async fn generate_input_feedback(&self, _keystroke: char) -> Result<()> {
        // Simulate feedback generation latency
        tokio::time::sleep(Duration::from_micros(100)).await;
        Ok(())
    }

    /// Generate error highlight (simulation)
    async fn generate_error_highlight(&self) -> Result<()> {
        // Simulate error highlighting latency
        tokio::time::sleep(Duration::from_micros(200)).await;
        Ok(())
    }

    /// Simulate UI update (simulation)
    async fn simulate_ui_update(&self) -> Result<()> {
        // Simulate UI update latency
        tokio::time::sleep(Duration::from_micros(500)).await;
        Ok(())
    }

    /// Generate comprehensive benchmark report
    pub fn generate_benchmark_report(&self, measurements: &[InputLatencyMeasurement]) -> InputLatencyBenchmarkReport {
        let overall_compliance = measurements.iter().all(|m| m.target_compliance.overall_compliant);

        let avg_p99_latency = measurements.iter()
            .map(|m| m.statistics.p99)
            .sum::<Duration>() / measurements.len() as u32;

        let worst_case_p99 = measurements.iter()
            .map(|m| m.statistics.p99)
            .max()
            .unwrap_or(Duration::ZERO);

        let all_bottlenecks: Vec<_> = measurements.iter()
            .flat_map(|m| &m.bottlenecks)
            .cloned()
            .collect();

        InputLatencyBenchmarkReport {
            overall_compliance,
            avg_p99_latency,
            worst_case_p99,
            target_margin_p99: self.config.p99_target.as_micros() as i64 - avg_p99_latency.as_micros() as i64,
            measurements: measurements.to_vec(),
            identified_bottlenecks: all_bottlenecks,
            optimization_recommendations: self.generate_optimization_recommendations(measurements),
            performance_grade: self.calculate_performance_grade(measurements),
        }
    }

    /// Generate optimization recommendations based on benchmark results
    fn generate_optimization_recommendations(&self, measurements: &[InputLatencyMeasurement]) -> Vec<String> {
        let mut recommendations = Vec::new();

        let avg_p99 = measurements.iter()
            .map(|m| m.statistics.p99)
            .sum::<Duration>() / measurements.len() as u32;

        if avg_p99 > self.config.p99_target {
            recommendations.push("P99 latency exceeds target - implement input processing optimizations".to_string());
        }

        let outlier_rate = measurements.iter()
            .map(|m| m.statistics.outlier_count as f64 / m.samples.len() as f64)
            .sum::<f64>() / measurements.len() as f64;

        if outlier_rate > 0.05 {
            recommendations.push("High outlier rate detected - investigate latency spikes and inconsistencies".to_string());
        }

        if measurements.iter().any(|m| !m.target_compliance.std_dev_compliant) {
            recommendations.push("Input latency variance too high - implement latency smoothing techniques".to_string());
        }

        recommendations
    }

    /// Calculate overall performance grade
    fn calculate_performance_grade(&self, measurements: &[InputLatencyMeasurement]) -> char {
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

/// Complete input latency benchmark report
#[derive(Debug, Clone)]
pub struct InputLatencyBenchmarkReport {
    pub overall_compliance: bool,
    pub avg_p99_latency: Duration,
    pub worst_case_p99: Duration,
    pub target_margin_p99: i64, // Microseconds margin (negative if over target)
    pub measurements: Vec<InputLatencyMeasurement>,
    pub identified_bottlenecks: Vec<InputBottleneck>,
    pub optimization_recommendations: Vec<String>,
    pub performance_grade: char,
}

/// Criterion benchmark functions for CI integration
pub fn input_latency_benchmarks(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let config = InputLatencyBenchmarkConfig::default();
    let harness = rt.block_on(async {
        InputLatencyBenchmarkHarness::new(config).await.unwrap()
    });

    let mut group = c.benchmark_group("input_latency");
    group.throughput(Throughput::Elements(1));

    group.bench_function("single_keystroke", |b| {
        b.to_async(&rt).iter(|| async {
            black_box(harness.measure_single_keystroke('a').await.unwrap())
        })
    });

    group.bench_function("error_detection", |b| {
        b.to_async(&rt).iter(|| async {
            let _ = harness.core.load_test_content("test");
            black_box(harness.core.process_character_input('x')) // Wrong character
        })
    });

    group.finish();
}

criterion_group!(benches, input_latency_benchmarks);
criterion_main!(benches);

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_input_latency_harness_creation() {
        let config = InputLatencyBenchmarkConfig::default();
        let harness = InputLatencyBenchmarkHarness::new(config).await.unwrap();

        // Verify harness was created successfully
        assert!(harness.config.sample_count > 0);
    }

    #[tokio::test]
    async fn test_single_keystroke_measurement() {
        let config = InputLatencyBenchmarkConfig {
            sample_count: 10,
            ..Default::default()
        };
        let harness = InputLatencyBenchmarkHarness::new(config).await.unwrap();

        let latency = harness.measure_single_keystroke('a').await.unwrap();
        assert!(latency > Duration::ZERO);
        assert!(latency < Duration::from_millis(100)); // Sanity check
    }

    #[tokio::test]
    async fn test_latency_statistics_calculation() {
        let samples = vec![
            Duration::from_millis(5),
            Duration::from_millis(10),
            Duration::from_millis(15),
            Duration::from_millis(20),
            Duration::from_millis(25),
        ];

        let stats = InputLatencyBenchmarkHarness::calculate_latency_statistics(&samples);
        assert_eq!(stats.median, Duration::from_millis(15));
        assert_eq!(stats.min, Duration::from_millis(5));
        assert_eq!(stats.max, Duration::from_millis(25));
    }

    #[tokio::test]
    async fn test_target_compliance_assessment() {
        let config = InputLatencyBenchmarkConfig::default();
        let harness = InputLatencyBenchmarkHarness::new(config).await.unwrap();

        let statistics = LatencyStatistics {
            mean: Duration::from_millis(5),
            median: Duration::from_millis(5),
            p95: Duration::from_millis(10),
            p99: Duration::from_millis(15),
            p999: Duration::from_millis(20),
            min: Duration::from_millis(1),
            max: Duration::from_millis(20),
            std_dev: 2.0,
            variance: 4.0,
            range: Duration::from_millis(19),
            outlier_count: 0,
        };

        let compliance = harness.assess_target_compliance(&statistics);
        assert!(compliance.overall_compliant);
        assert!(compliance.p99_compliant);
        assert!(compliance.margin_p99 > 0);
    }
}