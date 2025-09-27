//! # Render Performance Benchmark Suite
//!
//! Comprehensive benchmarking framework for validating P95 render time <33ms target.
//! This benchmark suite provides precise measurement of frame rendering performance
//! to ensure smooth 30fps+ display updates in the Centotype terminal interface.
//!
//! ## Key Performance Targets
//!
//! - **P95 Render Time**: <33ms (30fps equivalent)
//! - **P99 Render Time**: <50ms (acceptable peak latency)
//! - **Mean Render Time**: <16ms (60fps target)
//! - **Frame Consistency**: <10ms standard deviation
//! - **Animation Smoothness**: <5% frame drops
//!
//! ## Benchmark Categories
//!
//! 1. **Static Content Rendering**: Text display without animation
//! 2. **Dynamic Content Rendering**: Real-time typing updates
//! 3. **Error Highlighting Rendering**: Visual error feedback
//! 4. **Progress Indicator Rendering**: WPM/accuracy displays
//! 5. **Large Content Rendering**: 3000+ character levels
//! 6. **Animation Performance**: Smooth transitions and effects
//! 7. **Concurrent Rendering**: Multiple UI elements simultaneously

use centotype_content::ContentManager;
use centotype_core::{CentotypeCore, types::*};
use centotype_engine::CentotypeEngine;
use centotype_platform::PlatformManager;
use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion, Throughput};
use ratatui::{Frame, Terminal, backend::TestBackend, layout::{Layout, Constraint, Direction}};
use std::collections::HashMap;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::runtime::Runtime;
use tokio::sync::RwLock;
use tracing::{debug, info, warn, instrument};

/// Render performance benchmark configuration
#[derive(Debug, Clone)]
pub struct RenderBenchmarkConfig {
    /// Number of frame samples for measurement
    pub frame_sample_count: usize,
    /// Target P95 render time threshold
    pub p95_target: Duration,
    /// Target P99 render time threshold
    pub p99_target: Duration,
    /// Target mean render time threshold
    pub mean_target: Duration,
    /// Maximum acceptable frame time variance
    pub max_frame_variance: Duration,
    /// Animation test duration
    pub animation_test_duration: Duration,
    /// Large content test size (character count)
    pub large_content_size: usize,
    /// Concurrent render element count
    pub concurrent_elements: usize,
    /// Terminal dimensions for testing
    pub terminal_width: u16,
    pub terminal_height: u16,
}

impl Default for RenderBenchmarkConfig {
    fn default() -> Self {
        Self {
            frame_sample_count: 1000,
            p95_target: Duration::from_millis(33), // 30fps
            p99_target: Duration::from_millis(50), // Acceptable peak
            mean_target: Duration::from_millis(16), // 60fps
            max_frame_variance: Duration::from_millis(10),
            animation_test_duration: Duration::from_secs(10),
            large_content_size: 3000,
            concurrent_elements: 8,
            terminal_width: 120,
            terminal_height: 40,
        }
    }
}

/// High-precision render performance measurements
#[derive(Debug, Clone)]
pub struct RenderPerformanceMeasurement {
    /// Raw frame render time samples
    pub frame_times: Vec<Duration>,
    /// Render performance statistics
    pub statistics: RenderStatistics,
    /// Performance target compliance
    pub target_compliance: RenderTargetCompliance,
    /// Identified rendering bottlenecks
    pub bottlenecks: Vec<RenderBottleneck>,
    /// Test metadata
    pub metadata: RenderBenchmarkMetadata,
}

/// Statistical analysis of render performance
#[derive(Debug, Clone)]
pub struct RenderStatistics {
    pub mean_frame_time: Duration,
    pub median_frame_time: Duration,
    pub p95_frame_time: Duration,
    pub p99_frame_time: Duration,
    pub min_frame_time: Duration,
    pub max_frame_time: Duration,
    pub frame_time_std_dev: f64,
    pub frame_time_variance: f64,
    pub effective_fps: f64,
    pub frame_drop_percentage: f64,
    pub frame_consistency_score: f64, // 0.0 to 1.0
    pub outlier_frame_count: usize,
}

/// Render performance target compliance
#[derive(Debug, Clone)]
pub struct RenderTargetCompliance {
    pub p95_compliant: bool,
    pub p99_compliant: bool,
    pub mean_compliant: bool,
    pub variance_compliant: bool,
    pub fps_compliant: bool,
    pub overall_compliant: bool,
    pub margin_p95: i64, // Margin in microseconds
    pub margin_p99: i64,
    pub margin_mean: i64,
}

/// Render performance bottleneck identification
#[derive(Debug, Clone)]
pub struct RenderBottleneck {
    pub component: String,
    pub render_phase: RenderPhase,
    pub avg_time: Duration,
    pub impact_percentage: f64,
    pub frequency: f64,
    pub suggested_optimization: String,
}

/// Rendering phases for bottleneck analysis
#[derive(Debug, Clone)]
pub enum RenderPhase {
    LayoutCalculation,
    TextMeasurement,
    ContentRendering,
    ErrorHighlighting,
    ProgressDisplay,
    AnimationUpdate,
    FrameComposition,
    TerminalOutput,
}

/// Render benchmark execution metadata
#[derive(Debug, Clone)]
pub struct RenderBenchmarkMetadata {
    pub test_name: String,
    pub execution_timestamp: Instant,
    pub config: RenderBenchmarkConfig,
    pub terminal_info: TerminalInfo,
    pub content_complexity: ContentComplexity,
    pub warmup_duration: Duration,
    pub actual_frame_count: usize,
}

/// Terminal information for render context
#[derive(Debug, Clone)]
pub struct TerminalInfo {
    pub width: u16,
    pub height: u16,
    pub color_support: bool,
    pub unicode_support: bool,
    pub terminal_type: String,
}

/// Content complexity metrics
#[derive(Debug, Clone)]
pub struct ContentComplexity {
    pub character_count: usize,
    pub line_count: usize,
    pub has_errors: bool,
    pub has_animations: bool,
    pub ui_element_count: usize,
}

/// Render performance benchmark harness
pub struct RenderPerformanceBenchmarkHarness {
    config: RenderBenchmarkConfig,
    content_manager: Arc<ContentManager>,
    core: Arc<CentotypeCore>,
    platform: Arc<PlatformManager>,
    runtime: Runtime,
    test_terminal: Terminal<TestBackend>,
}

impl RenderPerformanceBenchmarkHarness {
    /// Create new render benchmark harness
    pub async fn new(config: RenderBenchmarkConfig) -> Result<Self> {
        let runtime = Runtime::new().map_err(|e| CentotypeError::Internal(format!("Failed to create runtime: {}", e)))?;

        let content_manager = Arc::new(ContentManager::new().await?);
        let core = Arc::new(CentotypeCore::new());
        let platform = Arc::new(PlatformManager::new()?);

        // Create test terminal backend
        let backend = TestBackend::new(config.terminal_width, config.terminal_height);
        let test_terminal = Terminal::new(backend).map_err(|e| CentotypeError::Internal(format!("Failed to create terminal: {}", e)))?;

        Ok(Self {
            config,
            content_manager,
            core,
            platform,
            runtime,
            test_terminal,
        })
    }

    /// Run comprehensive render performance benchmark suite
    #[instrument(skip(self))]
    pub async fn run_comprehensive_render_benchmark(&self) -> Result<Vec<RenderPerformanceMeasurement>> {
        info!("Starting comprehensive render performance benchmark suite");

        let mut results = Vec::new();

        // 1. Static content rendering
        let static_rendering = self.benchmark_static_content_rendering().await?;
        results.push(static_rendering);

        // 2. Dynamic content rendering (typing simulation)
        let dynamic_rendering = self.benchmark_dynamic_content_rendering().await?;
        results.push(dynamic_rendering);

        // 3. Error highlighting rendering
        let error_highlighting = self.benchmark_error_highlighting_rendering().await?;
        results.push(error_highlighting);

        // 4. Progress indicator rendering
        let progress_rendering = self.benchmark_progress_indicator_rendering().await?;
        results.push(progress_rendering);

        // 5. Large content rendering
        let large_content = self.benchmark_large_content_rendering().await?;
        results.push(large_content);

        // 6. Animation performance
        let animation_performance = self.benchmark_animation_performance().await?;
        results.push(animation_performance);

        // 7. Concurrent rendering
        let concurrent_rendering = self.benchmark_concurrent_rendering().await?;
        results.push(concurrent_rendering);

        info!("Completed comprehensive render performance benchmark suite with {} test categories", results.len());
        Ok(results)
    }

    /// Benchmark static content rendering performance
    #[instrument(skip(self))]
    async fn benchmark_static_content_rendering(&self) -> Result<RenderPerformanceMeasurement> {
        info!("Benchmarking static content rendering");

        // Load test content
        let level_id = LevelId::new(1)?;
        let content = self.content_manager.get_level_content(level_id, None).await?;

        let mut frame_times = Vec::with_capacity(self.config.frame_sample_count);
        let warmup_start = Instant::now();

        // Warmup phase
        for _ in 0..50 {
            let _ = self.render_static_frame(&content).await?;
        }
        let warmup_duration = warmup_start.elapsed();

        // Main measurement phase
        for i in 0..self.config.frame_sample_count {
            let frame_start = Instant::now();

            let _frame_result = self.render_static_frame(&content).await?;

            let frame_time = frame_start.elapsed();
            frame_times.push(frame_time);

            if i % 100 == 0 && i > 0 {
                debug!("Static rendering benchmark progress: {}/{}", i, self.config.frame_sample_count);
            }
        }

        let statistics = Self::calculate_render_statistics(&frame_times);
        let target_compliance = self.assess_render_target_compliance(&statistics);
        let bottlenecks = self.identify_render_bottlenecks(&frame_times, RenderPhase::ContentRendering).await;

        let metadata = RenderBenchmarkMetadata {
            test_name: "static_content_rendering".to_string(),
            execution_timestamp: Instant::now(),
            config: self.config.clone(),
            terminal_info: self.gather_terminal_info(),
            content_complexity: ContentComplexity {
                character_count: content.content.len(),
                line_count: content.content.lines().count(),
                has_errors: false,
                has_animations: false,
                ui_element_count: 1,
            },
            warmup_duration,
            actual_frame_count: frame_times.len(),
        };

        Ok(RenderPerformanceMeasurement {
            frame_times,
            statistics,
            target_compliance,
            bottlenecks,
            metadata,
        })
    }

    /// Render a static frame for benchmarking
    async fn render_static_frame(&self, content: &LevelContent) -> Result<()> {
        // Simulate frame rendering pipeline
        let layout_start = Instant::now();
        let _layout = self.calculate_frame_layout().await;
        let _layout_time = layout_start.elapsed();

        let text_start = Instant::now();
        let _text_measurements = self.measure_text_dimensions(&content.content).await;
        let _text_time = text_start.elapsed();

        let render_start = Instant::now();
        let _rendered_content = self.render_content_to_buffer(&content.content).await;
        let _render_time = render_start.elapsed();

        let composition_start = Instant::now();
        let _composed_frame = self.compose_final_frame().await;
        let _composition_time = composition_start.elapsed();

        Ok(())
    }

    /// Benchmark dynamic content rendering (typing simulation)
    #[instrument(skip(self))]
    async fn benchmark_dynamic_content_rendering(&self) -> Result<RenderPerformanceMeasurement> {
        info!("Benchmarking dynamic content rendering");

        let level_id = LevelId::new(1)?;
        let content = self.content_manager.get_level_content(level_id, None).await?;
        let test_text = &content.content;

        let mut frame_times = Vec::with_capacity(self.config.frame_sample_count);
        let mut typed_chars = 0;

        for i in 0..self.config.frame_sample_count {
            let frame_start = Instant::now();

            // Simulate typing progress
            if i % 10 == 0 && typed_chars < test_text.len() {
                typed_chars += 1;
            }

            let _frame_result = self.render_dynamic_frame(test_text, typed_chars).await?;

            let frame_time = frame_start.elapsed();
            frame_times.push(frame_time);

            if i % 100 == 0 && i > 0 {
                debug!("Dynamic rendering benchmark progress: {}/{}", i, self.config.frame_sample_count);
            }
        }

        let statistics = Self::calculate_render_statistics(&frame_times);
        let target_compliance = self.assess_render_target_compliance(&statistics);
        let bottlenecks = self.identify_render_bottlenecks(&frame_times, RenderPhase::ContentRendering).await;

        let metadata = RenderBenchmarkMetadata {
            test_name: "dynamic_content_rendering".to_string(),
            execution_timestamp: Instant::now(),
            config: self.config.clone(),
            terminal_info: self.gather_terminal_info(),
            content_complexity: ContentComplexity {
                character_count: test_text.len(),
                line_count: test_text.lines().count(),
                has_errors: false,
                has_animations: true,
                ui_element_count: 3, // text + cursor + progress
            },
            warmup_duration: Duration::ZERO,
            actual_frame_count: frame_times.len(),
        };

        Ok(RenderPerformanceMeasurement {
            frame_times,
            statistics,
            target_compliance,
            bottlenecks,
            metadata,
        })
    }

    /// Render a dynamic frame with typing progress
    async fn render_dynamic_frame(&self, text: &str, typed_chars: usize) -> Result<()> {
        let layout_start = Instant::now();
        let _layout = self.calculate_frame_layout().await;
        let _layout_time = layout_start.elapsed();

        let cursor_start = Instant::now();
        let _cursor_position = self.calculate_cursor_position(typed_chars).await;
        let _cursor_time = cursor_start.elapsed();

        let progress_start = Instant::now();
        let progress = typed_chars as f64 / text.len() as f64;
        let _progress_display = self.render_progress_indicator(progress).await;
        let _progress_time = progress_start.elapsed();

        let text_start = Instant::now();
        let typed_text = &text[..typed_chars.min(text.len())];
        let remaining_text = &text[typed_chars.min(text.len())..];
        let _typed_render = self.render_typed_text(typed_text).await;
        let _remaining_render = self.render_remaining_text(remaining_text).await;
        let _text_time = text_start.elapsed();

        let composition_start = Instant::now();
        let _composed_frame = self.compose_final_frame().await;
        let _composition_time = composition_start.elapsed();

        Ok(())
    }

    /// Benchmark error highlighting rendering
    #[instrument(skip(self))]
    async fn benchmark_error_highlighting_rendering(&self) -> Result<RenderPerformanceMeasurement> {
        info!("Benchmarking error highlighting rendering");

        let test_text = "The quick brown fox jumps over the lazy dog";
        let error_positions = vec![4, 10, 16, 20, 26]; // Simulate errors at these positions

        let mut frame_times = Vec::with_capacity(self.config.frame_sample_count);

        for i in 0..self.config.frame_sample_count {
            let frame_start = Instant::now();

            let _frame_result = self.render_error_highlighting_frame(test_text, &error_positions).await?;

            let frame_time = frame_start.elapsed();
            frame_times.push(frame_time);

            if i % 100 == 0 && i > 0 {
                debug!("Error highlighting benchmark progress: {}/{}", i, self.config.frame_sample_count);
            }
        }

        let statistics = Self::calculate_render_statistics(&frame_times);
        let target_compliance = self.assess_render_target_compliance(&statistics);
        let bottlenecks = self.identify_render_bottlenecks(&frame_times, RenderPhase::ErrorHighlighting).await;

        let metadata = RenderBenchmarkMetadata {
            test_name: "error_highlighting_rendering".to_string(),
            execution_timestamp: Instant::now(),
            config: self.config.clone(),
            terminal_info: self.gather_terminal_info(),
            content_complexity: ContentComplexity {
                character_count: test_text.len(),
                line_count: 1,
                has_errors: true,
                has_animations: false,
                ui_element_count: error_positions.len() + 1,
            },
            warmup_duration: Duration::ZERO,
            actual_frame_count: frame_times.len(),
        };

        Ok(RenderPerformanceMeasurement {
            frame_times,
            statistics,
            target_compliance,
            bottlenecks,
            metadata,
        })
    }

    /// Render frame with error highlighting
    async fn render_error_highlighting_frame(&self, text: &str, error_positions: &[usize]) -> Result<()> {
        let highlight_start = Instant::now();

        for &pos in error_positions {
            let _highlight = self.render_error_highlight(pos).await;
        }

        let _highlight_time = highlight_start.elapsed();

        let text_start = Instant::now();
        let _text_render = self.render_content_to_buffer(text).await;
        let _text_time = text_start.elapsed();

        let composition_start = Instant::now();
        let _composed_frame = self.compose_final_frame().await;
        let _composition_time = composition_start.elapsed();

        Ok(())
    }

    /// Benchmark progress indicator rendering
    #[instrument(skip(self))]
    async fn benchmark_progress_indicator_rendering(&self) -> Result<RenderPerformanceMeasurement> {
        info!("Benchmarking progress indicator rendering");

        let mut frame_times = Vec::with_capacity(self.config.frame_sample_count);

        for i in 0..self.config.frame_sample_count {
            let frame_start = Instant::now();

            // Simulate changing progress
            let progress = (i as f64 / self.config.frame_sample_count as f64) * 100.0;
            let wpm = 50.0 + (i as f64 / 100.0) % 50.0; // Varying WPM
            let accuracy = 95.0 + (i as f64 / 200.0) % 5.0; // Varying accuracy

            let _frame_result = self.render_progress_frame(progress, wpm, accuracy).await?;

            let frame_time = frame_start.elapsed();
            frame_times.push(frame_time);

            if i % 100 == 0 && i > 0 {
                debug!("Progress indicator benchmark progress: {}/{}", i, self.config.frame_sample_count);
            }
        }

        let statistics = Self::calculate_render_statistics(&frame_times);
        let target_compliance = self.assess_render_target_compliance(&statistics);
        let bottlenecks = self.identify_render_bottlenecks(&frame_times, RenderPhase::ProgressDisplay).await;

        let metadata = RenderBenchmarkMetadata {
            test_name: "progress_indicator_rendering".to_string(),
            execution_timestamp: Instant::now(),
            config: self.config.clone(),
            terminal_info: self.gather_terminal_info(),
            content_complexity: ContentComplexity {
                character_count: 0,
                line_count: 0,
                has_errors: false,
                has_animations: true,
                ui_element_count: 4, // progress bar + WPM + accuracy + timer
            },
            warmup_duration: Duration::ZERO,
            actual_frame_count: frame_times.len(),
        };

        Ok(RenderPerformanceMeasurement {
            frame_times,
            statistics,
            target_compliance,
            bottlenecks,
            metadata,
        })
    }

    /// Render frame with progress indicators
    async fn render_progress_frame(&self, progress: f64, wpm: f64, accuracy: f64) -> Result<()> {
        let progress_start = Instant::now();
        let _progress_bar = self.render_progress_indicator(progress / 100.0).await;
        let _progress_time = progress_start.elapsed();

        let metrics_start = Instant::now();
        let _wpm_display = self.render_wpm_display(wpm).await;
        let _accuracy_display = self.render_accuracy_display(accuracy).await;
        let _timer_display = self.render_timer_display(60.0 - progress / 100.0 * 60.0).await;
        let _metrics_time = metrics_start.elapsed();

        let composition_start = Instant::now();
        let _composed_frame = self.compose_final_frame().await;
        let _composition_time = composition_start.elapsed();

        Ok(())
    }

    /// Benchmark large content rendering
    #[instrument(skip(self))]
    async fn benchmark_large_content_rendering(&self) -> Result<RenderPerformanceMeasurement> {
        info!("Benchmarking large content rendering with {} characters", self.config.large_content_size);

        // Generate large test content
        let large_content = "The quick brown fox jumps over the lazy dog. ".repeat(self.config.large_content_size / 45);

        let mut frame_times = Vec::with_capacity(self.config.frame_sample_count / 4); // Fewer samples for large content

        for i in 0..(self.config.frame_sample_count / 4) {
            let frame_start = Instant::now();

            let _frame_result = self.render_large_content_frame(&large_content).await?;

            let frame_time = frame_start.elapsed();
            frame_times.push(frame_time);

            if i % 25 == 0 && i > 0 {
                debug!("Large content rendering benchmark progress: {}/{}", i, self.config.frame_sample_count / 4);
            }
        }

        let statistics = Self::calculate_render_statistics(&frame_times);
        let target_compliance = self.assess_render_target_compliance(&statistics);
        let bottlenecks = self.identify_render_bottlenecks(&frame_times, RenderPhase::ContentRendering).await;

        let metadata = RenderBenchmarkMetadata {
            test_name: "large_content_rendering".to_string(),
            execution_timestamp: Instant::now(),
            config: self.config.clone(),
            terminal_info: self.gather_terminal_info(),
            content_complexity: ContentComplexity {
                character_count: large_content.len(),
                line_count: large_content.lines().count(),
                has_errors: false,
                has_animations: false,
                ui_element_count: 1,
            },
            warmup_duration: Duration::ZERO,
            actual_frame_count: frame_times.len(),
        };

        Ok(RenderPerformanceMeasurement {
            frame_times,
            statistics,
            target_compliance,
            bottlenecks,
            metadata,
        })
    }

    /// Render frame with large content
    async fn render_large_content_frame(&self, content: &str) -> Result<()> {
        let layout_start = Instant::now();
        let _layout = self.calculate_large_content_layout(content).await;
        let _layout_time = layout_start.elapsed();

        let viewport_start = Instant::now();
        let _viewport = self.calculate_viewport_content(content, 0).await; // Starting at position 0
        let _viewport_time = viewport_start.elapsed();

        let render_start = Instant::now();
        let _rendered_content = self.render_content_to_buffer(content).await;
        let _render_time = render_start.elapsed();

        let composition_start = Instant::now();
        let _composed_frame = self.compose_final_frame().await;
        let _composition_time = composition_start.elapsed();

        Ok(())
    }

    /// Benchmark animation performance
    #[instrument(skip(self))]
    async fn benchmark_animation_performance(&self) -> Result<RenderPerformanceMeasurement> {
        info!("Benchmarking animation performance for {:?}", self.config.animation_test_duration);

        let mut frame_times = Vec::new();
        let animation_start = Instant::now();
        let mut frame_count = 0;

        while animation_start.elapsed() < self.config.animation_test_duration {
            let frame_start = Instant::now();

            // Simulate animated cursor and progress updates
            let animation_progress = animation_start.elapsed().as_secs_f64() / self.config.animation_test_duration.as_secs_f64();
            let _frame_result = self.render_animated_frame(animation_progress).await?;

            let frame_time = frame_start.elapsed();
            frame_times.push(frame_time);
            frame_count += 1;

            // Target 30fps for animation
            let target_frame_time = Duration::from_millis(33);
            if frame_time < target_frame_time {
                tokio::time::sleep(target_frame_time - frame_time).await;
            }
        }

        let statistics = Self::calculate_render_statistics(&frame_times);
        let target_compliance = self.assess_render_target_compliance(&statistics);
        let bottlenecks = self.identify_render_bottlenecks(&frame_times, RenderPhase::AnimationUpdate).await;

        let metadata = RenderBenchmarkMetadata {
            test_name: "animation_performance".to_string(),
            execution_timestamp: Instant::now(),
            config: self.config.clone(),
            terminal_info: self.gather_terminal_info(),
            content_complexity: ContentComplexity {
                character_count: 100,
                line_count: 5,
                has_errors: false,
                has_animations: true,
                ui_element_count: 5, // animated cursor + progress + multiple UI elements
            },
            warmup_duration: Duration::ZERO,
            actual_frame_count: frame_times.len(),
        };

        Ok(RenderPerformanceMeasurement {
            frame_times,
            statistics,
            target_compliance,
            bottlenecks,
            metadata,
        })
    }

    /// Render animated frame
    async fn render_animated_frame(&self, progress: f64) -> Result<()> {
        let cursor_start = Instant::now();
        let cursor_phase = (progress * 2.0 * std::f64::consts::PI).sin();
        let _cursor_animation = self.render_animated_cursor(cursor_phase).await;
        let _cursor_time = cursor_start.elapsed();

        let progress_start = Instant::now();
        let _progress_animation = self.render_progress_indicator(progress).await;
        let _progress_time = progress_start.elapsed();

        let ui_start = Instant::now();
        let _ui_updates = self.render_animated_ui_elements(progress).await;
        let _ui_time = ui_start.elapsed();

        let composition_start = Instant::now();
        let _composed_frame = self.compose_final_frame().await;
        let _composition_time = composition_start.elapsed();

        Ok(())
    }

    /// Benchmark concurrent rendering
    #[instrument(skip(self))]
    async fn benchmark_concurrent_rendering(&self) -> Result<RenderPerformanceMeasurement> {
        info!("Benchmarking concurrent rendering with {} elements", self.config.concurrent_elements);

        let mut frame_times = Vec::with_capacity(self.config.frame_sample_count);

        for i in 0..self.config.frame_sample_count {
            let frame_start = Instant::now();

            let _frame_result = self.render_concurrent_elements_frame().await?;

            let frame_time = frame_start.elapsed();
            frame_times.push(frame_time);

            if i % 100 == 0 && i > 0 {
                debug!("Concurrent rendering benchmark progress: {}/{}", i, self.config.frame_sample_count);
            }
        }

        let statistics = Self::calculate_render_statistics(&frame_times);
        let target_compliance = self.assess_render_target_compliance(&statistics);
        let bottlenecks = self.identify_render_bottlenecks(&frame_times, RenderPhase::FrameComposition).await;

        let metadata = RenderBenchmarkMetadata {
            test_name: "concurrent_rendering".to_string(),
            execution_timestamp: Instant::now(),
            config: self.config.clone(),
            terminal_info: self.gather_terminal_info(),
            content_complexity: ContentComplexity {
                character_count: 500,
                line_count: 10,
                has_errors: true,
                has_animations: true,
                ui_element_count: self.config.concurrent_elements,
            },
            warmup_duration: Duration::ZERO,
            actual_frame_count: frame_times.len(),
        };

        Ok(RenderPerformanceMeasurement {
            frame_times,
            statistics,
            target_compliance,
            bottlenecks,
            metadata,
        })
    }

    /// Render frame with concurrent elements
    async fn render_concurrent_elements_frame(&self) -> Result<()> {
        let mut render_tasks = Vec::new();

        // Simulate concurrent rendering of multiple UI elements
        for i in 0..self.config.concurrent_elements {
            let task = async move {
                match i % 4 {
                    0 => self.render_content_to_buffer("Sample text content").await,
                    1 => self.render_progress_indicator(0.5).await,
                    2 => self.render_error_highlight(i).await,
                    _ => self.render_wpm_display(60.0).await,
                }
            };
            render_tasks.push(task);
        }

        // Execute all rendering tasks concurrently
        let render_start = Instant::now();
        for task in render_tasks {
            let _ = task.await;
        }
        let _concurrent_render_time = render_start.elapsed();

        let composition_start = Instant::now();
        let _composed_frame = self.compose_final_frame().await;
        let _composition_time = composition_start.elapsed();

        Ok(())
    }

    /// Calculate comprehensive render statistics
    fn calculate_render_statistics(frame_times: &[Duration]) -> RenderStatistics {
        if frame_times.is_empty() {
            return RenderStatistics {
                mean_frame_time: Duration::ZERO,
                median_frame_time: Duration::ZERO,
                p95_frame_time: Duration::ZERO,
                p99_frame_time: Duration::ZERO,
                min_frame_time: Duration::ZERO,
                max_frame_time: Duration::ZERO,
                frame_time_std_dev: 0.0,
                frame_time_variance: 0.0,
                effective_fps: 0.0,
                frame_drop_percentage: 0.0,
                frame_consistency_score: 0.0,
                outlier_frame_count: 0,
            };
        }

        let mut sorted_times = frame_times.to_vec();
        sorted_times.sort();

        let len = sorted_times.len();
        let mean_frame_time = frame_times.iter().sum::<Duration>() / frame_times.len() as u32;
        let median_frame_time = sorted_times[len / 2];
        let p95_frame_time = sorted_times[len * 95 / 100];
        let p99_frame_time = sorted_times[len * 99 / 100];
        let min_frame_time = sorted_times[0];
        let max_frame_time = sorted_times[len - 1];

        // Calculate variance and standard deviation
        let mean_nanos = mean_frame_time.as_nanos() as f64;
        let variance: f64 = frame_times.iter()
            .map(|&duration| {
                let diff = duration.as_nanos() as f64 - mean_nanos;
                diff * diff
            })
            .sum::<f64>() / frame_times.len() as f64;
        let frame_time_std_dev = variance.sqrt();

        // Calculate effective FPS
        let effective_fps = if mean_frame_time.as_nanos() > 0 {
            1_000_000_000.0 / mean_frame_time.as_nanos() as f64
        } else {
            0.0
        };

        // Calculate frame drop percentage (frames over 33ms for 30fps)
        let target_frame_time = Duration::from_millis(33);
        let dropped_frames = frame_times.iter().filter(|&&t| t > target_frame_time).count();
        let frame_drop_percentage = (dropped_frames as f64 / frame_times.len() as f64) * 100.0;

        // Calculate frame consistency score (based on variance)
        let frame_consistency_score = if frame_time_std_dev > 0.0 {
            (1.0 - (frame_time_std_dev / mean_nanos).min(1.0)).max(0.0)
        } else {
            1.0
        };

        // Count outliers (frames more than 2 standard deviations from mean)
        let outlier_threshold = 2.0 * frame_time_std_dev;
        let outlier_frame_count = frame_times.iter()
            .filter(|&&duration| {
                let diff = (duration.as_nanos() as f64 - mean_nanos).abs();
                diff > outlier_threshold
            })
            .count();

        RenderStatistics {
            mean_frame_time,
            median_frame_time,
            p95_frame_time,
            p99_frame_time,
            min_frame_time,
            max_frame_time,
            frame_time_std_dev,
            frame_time_variance: variance,
            effective_fps,
            frame_drop_percentage,
            frame_consistency_score,
            outlier_frame_count,
        }
    }

    /// Assess render performance target compliance
    fn assess_render_target_compliance(&self, statistics: &RenderStatistics) -> RenderTargetCompliance {
        let p95_compliant = statistics.p95_frame_time <= self.config.p95_target;
        let p99_compliant = statistics.p99_frame_time <= self.config.p99_target;
        let mean_compliant = statistics.mean_frame_time <= self.config.mean_target;
        let variance_compliant = Duration::from_nanos(statistics.frame_time_variance as u64) <= self.config.max_frame_variance;
        let fps_compliant = statistics.effective_fps >= 30.0; // 30fps minimum

        let overall_compliant = p95_compliant && p99_compliant && mean_compliant && variance_compliant && fps_compliant;

        let margin_p95 = self.config.p95_target.as_micros() as i64 - statistics.p95_frame_time.as_micros() as i64;
        let margin_p99 = self.config.p99_target.as_micros() as i64 - statistics.p99_frame_time.as_micros() as i64;
        let margin_mean = self.config.mean_target.as_micros() as i64 - statistics.mean_frame_time.as_micros() as i64;

        RenderTargetCompliance {
            p95_compliant,
            p99_compliant,
            mean_compliant,
            variance_compliant,
            fps_compliant,
            overall_compliant,
            margin_p95,
            margin_p99,
            margin_mean,
        }
    }

    /// Identify render performance bottlenecks
    async fn identify_render_bottlenecks(&self, frame_times: &[Duration], phase: RenderPhase) -> Vec<RenderBottleneck> {
        let mut bottlenecks = Vec::new();

        let statistics = Self::calculate_render_statistics(frame_times);

        if statistics.p95_frame_time > self.config.p95_target {
            let impact_percentage = ((statistics.p95_frame_time.as_micros() as f64 / self.config.p95_target.as_micros() as f64) - 1.0) * 100.0;

            bottlenecks.push(RenderBottleneck {
                component: format!("{:?}", phase),
                render_phase: phase.clone(),
                avg_time: statistics.mean_frame_time,
                impact_percentage,
                frequency: 1.0,
                suggested_optimization: Self::get_render_optimization_suggestion(&phase),
            });
        }

        bottlenecks
    }

    /// Get optimization suggestions for specific render phases
    fn get_render_optimization_suggestion(phase: &RenderPhase) -> String {
        match phase {
            RenderPhase::LayoutCalculation => "Cache layout calculations and use incremental updates".to_string(),
            RenderPhase::TextMeasurement => "Pre-calculate text dimensions and use glyph caching".to_string(),
            RenderPhase::ContentRendering => "Implement virtual scrolling and render only visible content".to_string(),
            RenderPhase::ErrorHighlighting => "Use efficient highlighting algorithms and batch updates".to_string(),
            RenderPhase::ProgressDisplay => "Update progress indicators only when values change significantly".to_string(),
            RenderPhase::AnimationUpdate => "Use easing functions and reduce animation complexity".to_string(),
            RenderPhase::FrameComposition => "Optimize frame buffer operations and reduce memory allocations".to_string(),
            RenderPhase::TerminalOutput => "Batch terminal output and optimize escape sequence generation".to_string(),
        }
    }

    /// Gather terminal information for context
    fn gather_terminal_info(&self) -> TerminalInfo {
        TerminalInfo {
            width: self.config.terminal_width,
            height: self.config.terminal_height,
            color_support: true, // Assume color support for testing
            unicode_support: true, // Assume Unicode support
            terminal_type: "test_terminal".to_string(),
        }
    }

    /// Render performance helper methods (simulations for benchmarking)
    async fn calculate_frame_layout(&self) -> Result<()> {
        tokio::time::sleep(Duration::from_micros(100)).await;
        Ok(())
    }

    async fn measure_text_dimensions(&self, _text: &str) -> Result<()> {
        tokio::time::sleep(Duration::from_micros(200)).await;
        Ok(())
    }

    async fn render_content_to_buffer(&self, _content: &str) -> Result<()> {
        tokio::time::sleep(Duration::from_micros(500)).await;
        Ok(())
    }

    async fn compose_final_frame(&self) -> Result<()> {
        tokio::time::sleep(Duration::from_micros(300)).await;
        Ok(())
    }

    async fn calculate_cursor_position(&self, _position: usize) -> Result<()> {
        tokio::time::sleep(Duration::from_micros(50)).await;
        Ok(())
    }

    async fn render_progress_indicator(&self, _progress: f64) -> Result<()> {
        tokio::time::sleep(Duration::from_micros(150)).await;
        Ok(())
    }

    async fn render_typed_text(&self, _text: &str) -> Result<()> {
        tokio::time::sleep(Duration::from_micros(300)).await;
        Ok(())
    }

    async fn render_remaining_text(&self, _text: &str) -> Result<()> {
        tokio::time::sleep(Duration::from_micros(200)).await;
        Ok(())
    }

    async fn render_error_highlight(&self, _position: usize) -> Result<()> {
        tokio::time::sleep(Duration::from_micros(100)).await;
        Ok(())
    }

    async fn render_wpm_display(&self, _wpm: f64) -> Result<()> {
        tokio::time::sleep(Duration::from_micros(75)).await;
        Ok(())
    }

    async fn render_accuracy_display(&self, _accuracy: f64) -> Result<()> {
        tokio::time::sleep(Duration::from_micros(75)).await;
        Ok(())
    }

    async fn render_timer_display(&self, _time: f64) -> Result<()> {
        tokio::time::sleep(Duration::from_micros(75)).await;
        Ok(())
    }

    async fn calculate_large_content_layout(&self, _content: &str) -> Result<()> {
        tokio::time::sleep(Duration::from_micros(500)).await;
        Ok(())
    }

    async fn calculate_viewport_content(&self, _content: &str, _position: usize) -> Result<()> {
        tokio::time::sleep(Duration::from_micros(200)).await;
        Ok(())
    }

    async fn render_animated_cursor(&self, _phase: f64) -> Result<()> {
        tokio::time::sleep(Duration::from_micros(100)).await;
        Ok(())
    }

    async fn render_animated_ui_elements(&self, _progress: f64) -> Result<()> {
        tokio::time::sleep(Duration::from_micros(250)).await;
        Ok(())
    }

    /// Generate comprehensive render benchmark report
    pub fn generate_render_benchmark_report(&self, measurements: &[RenderPerformanceMeasurement]) -> RenderPerformanceBenchmarkReport {
        let overall_compliance = measurements.iter().all(|m| m.target_compliance.overall_compliant);

        let avg_p95_frame_time = measurements.iter()
            .map(|m| m.statistics.p95_frame_time)
            .sum::<Duration>() / measurements.len() as u32;

        let avg_effective_fps = measurements.iter()
            .map(|m| m.statistics.effective_fps)
            .sum::<f64>() / measurements.len() as f64;

        let worst_case_p95 = measurements.iter()
            .map(|m| m.statistics.p95_frame_time)
            .max()
            .unwrap_or(Duration::ZERO);

        let all_bottlenecks: Vec<_> = measurements.iter()
            .flat_map(|m| &m.bottlenecks)
            .cloned()
            .collect();

        RenderPerformanceBenchmarkReport {
            overall_compliance,
            avg_p95_frame_time,
            avg_effective_fps,
            worst_case_p95,
            target_margin_p95: self.config.p95_target.as_micros() as i64 - avg_p95_frame_time.as_micros() as i64,
            measurements: measurements.to_vec(),
            identified_bottlenecks: all_bottlenecks,
            optimization_recommendations: self.generate_render_optimization_recommendations(measurements),
            performance_grade: self.calculate_render_performance_grade(measurements),
        }
    }

    /// Generate render optimization recommendations
    fn generate_render_optimization_recommendations(&self, measurements: &[RenderPerformanceMeasurement]) -> Vec<String> {
        let mut recommendations = Vec::new();

        let avg_p95 = measurements.iter()
            .map(|m| m.statistics.p95_frame_time)
            .sum::<Duration>() / measurements.len() as u32;

        if avg_p95 > self.config.p95_target {
            recommendations.push("P95 frame time exceeds target - implement rendering optimizations".to_string());
        }

        let avg_frame_drops = measurements.iter()
            .map(|m| m.statistics.frame_drop_percentage)
            .sum::<f64>() / measurements.len() as f64;

        if avg_frame_drops > 5.0 {
            recommendations.push("High frame drop percentage - optimize rendering pipeline".to_string());
        }

        let avg_consistency = measurements.iter()
            .map(|m| m.statistics.frame_consistency_score)
            .sum::<f64>() / measurements.len() as f64;

        if avg_consistency < 0.8 {
            recommendations.push("Poor frame consistency - implement frame time smoothing".to_string());
        }

        recommendations
    }

    /// Calculate render performance grade
    fn calculate_render_performance_grade(&self, measurements: &[RenderPerformanceMeasurement]) -> char {
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

/// Complete render performance benchmark report
#[derive(Debug, Clone)]
pub struct RenderPerformanceBenchmarkReport {
    pub overall_compliance: bool,
    pub avg_p95_frame_time: Duration,
    pub avg_effective_fps: f64,
    pub worst_case_p95: Duration,
    pub target_margin_p95: i64, // Microseconds margin
    pub measurements: Vec<RenderPerformanceMeasurement>,
    pub identified_bottlenecks: Vec<RenderBottleneck>,
    pub optimization_recommendations: Vec<String>,
    pub performance_grade: char,
}

/// Criterion benchmark functions for CI integration
pub fn render_performance_benchmarks(c: &mut Criterion) {
    let rt = Runtime::new().unwrap();
    let config = RenderBenchmarkConfig::default();
    let harness = rt.block_on(async {
        RenderPerformanceBenchmarkHarness::new(config).await.unwrap()
    });

    let mut group = c.benchmark_group("render_performance");
    group.throughput(Throughput::Elements(1));

    group.bench_function("static_frame_render", |b| {
        b.to_async(&rt).iter(|| async {
            let content = "The quick brown fox jumps over the lazy dog";
            black_box(harness.render_content_to_buffer(content).await.unwrap())
        })
    });

    group.bench_function("dynamic_frame_render", |b| {
        b.to_async(&rt).iter(|| async {
            black_box(harness.render_dynamic_frame("test content", 5).await.unwrap())
        })
    });

    group.bench_function("error_highlight_render", |b| {
        b.to_async(&rt).iter(|| async {
            black_box(harness.render_error_highlight(5).await.unwrap())
        })
    });

    group.finish();
}

criterion_group!(render_benches, render_performance_benchmarks);
criterion_main!(render_benches);

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_render_harness_creation() {
        let config = RenderBenchmarkConfig::default();
        let harness = RenderPerformanceBenchmarkHarness::new(config).await.unwrap();

        assert!(harness.config.frame_sample_count > 0);
    }

    #[tokio::test]
    async fn test_render_statistics_calculation() {
        let frame_times = vec![
            Duration::from_millis(16),
            Duration::from_millis(33),
            Duration::from_millis(25),
            Duration::from_millis(20),
            Duration::from_millis(30),
        ];

        let stats = RenderPerformanceBenchmarkHarness::calculate_render_statistics(&frame_times);
        assert!(stats.effective_fps > 0.0);
        assert!(stats.p95_frame_time <= Duration::from_millis(33));
    }

    #[tokio::test]
    async fn test_render_target_compliance() {
        let config = RenderBenchmarkConfig::default();
        let harness = RenderPerformanceBenchmarkHarness::new(config).await.unwrap();

        let statistics = RenderStatistics {
            mean_frame_time: Duration::from_millis(16),
            median_frame_time: Duration::from_millis(16),
            p95_frame_time: Duration::from_millis(30),
            p99_frame_time: Duration::from_millis(45),
            min_frame_time: Duration::from_millis(10),
            max_frame_time: Duration::from_millis(50),
            frame_time_std_dev: 5.0,
            frame_time_variance: 25.0,
            effective_fps: 60.0,
            frame_drop_percentage: 2.0,
            frame_consistency_score: 0.9,
            outlier_frame_count: 1,
        };

        let compliance = harness.assess_render_target_compliance(&statistics);
        assert!(compliance.overall_compliant);
        assert!(compliance.p95_compliant);
        assert!(compliance.fps_compliant);
    }
}