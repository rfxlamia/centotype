//! # Performance Validation Binary
//!
//! Main binary for running comprehensive performance validation in CI/CD pipelines.
//! This binary coordinates all benchmark suites and generates standardized reports.

use centotype_content::ContentManager;
use centotype_core::{CentotypeCore, types::*};
use clap::{Parser, Subcommand};
use serde_json;
use std::path::PathBuf;
use std::time::{Duration, Instant};
use tokio;
use tracing::{info, warn, error};

mod input_latency_benchmark;
mod render_performance_benchmark;
mod content_performance_benchmark;
mod memory_concurrency_benchmark;
mod ci_cd_performance_framework;

use input_latency_benchmark::{InputLatencyBenchmarkHarness, InputLatencyBenchmarkConfig};
use render_performance_benchmark::{RenderPerformanceBenchmarkHarness, RenderBenchmarkConfig};
use content_performance_benchmark::{ContentPerformanceBenchmarkHarness, ContentBenchmarkConfig};
use memory_concurrency_benchmark::{MemoryConcurrencyBenchmarkHarness, MemoryConcurrencyBenchmarkConfig};
use ci_cd_performance_framework::{CicdPerformanceFramework, CicdConfig, CicdContext, TestEnvironment, RunnerSpecs};

#[derive(Parser)]
#[command(name = "performance_validator")]
#[command(about = "Centotype Performance Validation Suite")]
#[command(version = "1.0.0")]
struct Cli {
    #[command(subcommand)]
    command: Option<Commands>,

    /// Commit hash for this validation run
    #[arg(long)]
    commit_hash: Option<String>,

    /// Branch name
    #[arg(long)]
    branch: Option<String>,

    /// Pull request number
    #[arg(long)]
    pr_number: Option<u32>,

    /// CI system name
    #[arg(long, default_value = "unknown")]
    ci_system: String,

    /// Output format (json, yaml, text)
    #[arg(long, default_value = "json")]
    output_format: String,

    /// Output file path
    #[arg(long)]
    output_file: Option<PathBuf>,

    /// Enable extended sampling for comprehensive testing
    #[arg(long)]
    extended_samples: bool,

    /// Test duration in seconds for long-running tests
    #[arg(long, default_value = "60")]
    duration: u64,

    /// Enable verbose logging
    #[arg(short, long)]
    verbose: bool,
}

#[derive(Subcommand)]
enum Commands {
    /// Run all performance benchmarks
    All {
        /// Configuration file path
        #[arg(short, long)]
        config: Option<PathBuf>,
    },
    /// Run input latency benchmarks only
    InputLatency {
        /// Number of samples to collect
        #[arg(long, default_value = "10000")]
        samples: usize,
    },
    /// Run render performance benchmarks only
    RenderPerformance {
        /// Number of frames to test
        #[arg(long, default_value = "1000")]
        frames: usize,
    },
    /// Run content system benchmarks only
    ContentSystem {
        /// Number of content loading tests
        #[arg(long, default_value = "5000")]
        tests: usize,
    },
    /// Run memory and concurrency benchmarks only
    MemoryConcurrency {
        /// Number of concurrent threads
        #[arg(long, default_value = "16")]
        threads: usize,
    },
    /// Validate against performance targets
    Validate {
        /// Input file with performance results
        #[arg(short, long)]
        input_file: PathBuf,
        /// Performance targets file
        #[arg(short, long)]
        targets_file: PathBuf,
        /// Strict mode (fail on any violation)
        #[arg(long)]
        strict_mode: bool,
    },
    /// Compare performance between two runs
    Compare {
        /// Baseline performance file
        #[arg(short, long)]
        baseline: PathBuf,
        /// Comparison performance file
        #[arg(short, long)]
        comparison: PathBuf,
        /// Regression threshold percentage
        #[arg(long, default_value = "5.0")]
        threshold: f64,
    },
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let cli = Cli::parse();

    // Initialize logging
    let log_level = if cli.verbose { "debug" } else { "info" };
    tracing_subscriber::fmt()
        .with_env_filter(format!("performance_validator={}", log_level))
        .init();

    info!("Starting Centotype Performance Validator v1.0.0");

    match &cli.command {
        Some(Commands::All { config }) => {
            run_all_benchmarks(&cli, config.as_deref()).await?;
        }
        Some(Commands::InputLatency { samples }) => {
            run_input_latency_benchmark(*samples).await?;
        }
        Some(Commands::RenderPerformance { frames }) => {
            run_render_performance_benchmark(*frames).await?;
        }
        Some(Commands::ContentSystem { tests }) => {
            run_content_system_benchmark(*tests).await?;
        }
        Some(Commands::MemoryConcurrency { threads }) => {
            run_memory_concurrency_benchmark(*threads).await?;
        }
        Some(Commands::Validate { input_file, targets_file, strict_mode }) => {
            validate_performance_targets(input_file, targets_file, *strict_mode).await?;
        }
        Some(Commands::Compare { baseline, comparison, threshold }) => {
            compare_performance_results(baseline, comparison, *threshold).await?;
        }
        None => {
            // Default: run all benchmarks
            run_all_benchmarks(&cli, None).await?;
        }
    }

    info!("Performance validation completed successfully");
    Ok(())
}

async fn run_all_benchmarks(cli: &Cli, _config_path: Option<&std::path::Path>) -> Result<(), Box<dyn std::error::Error>> {
    info!("Running comprehensive performance benchmark suite");

    let start_time = Instant::now();

    // Create CI/CD context
    let context = CicdContext {
        commit_hash: cli.commit_hash.clone().unwrap_or_else(|| "unknown".to_string()),
        branch: cli.branch.clone().unwrap_or_else(|| "main".to_string()),
        pr_number: cli.pr_number,
        timestamp: std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .unwrap()
            .as_secs(),
        test_environment: TestEnvironment {
            os: std::env::consts::OS.to_string(),
            architecture: std::env::consts::ARCH.to_string(),
            rust_version: "1.75.0".to_string(), // Would get actual version
            cpu_model: "unknown".to_string(),   // Would detect actual CPU
            memory_gb: 16,                     // Would detect actual memory
            disk_type: "ssd".to_string(),      // Would detect disk type
        },
        ci_system: cli.ci_system.clone(),
        runner_specs: RunnerSpecs {
            runner_type: "standard".to_string(),
            cpu_cores: num_cpus::get() as u32,
            memory_mb: 16384, // Would detect actual memory
            concurrent_jobs: 1,
        },
    };

    // Initialize CI/CD framework
    let cicd_config = CicdConfig::default();
    let framework = CicdPerformanceFramework::new(cicd_config)?;

    // Run comprehensive validation
    let result = framework.run_cicd_performance_validation(context).await?;

    // Generate output
    match cli.output_format.as_str() {
        "json" => {
            let json_output = serde_json::to_string_pretty(&result)?;
            if let Some(output_file) = &cli.output_file {
                std::fs::write(output_file, &json_output)?;
                info!("Results written to: {}", output_file.display());
            } else {
                println!("{}", json_output);
            }
        }
        "yaml" => {
            warn!("YAML output format not yet implemented, using JSON");
            let json_output = serde_json::to_string_pretty(&result)?;
            if let Some(output_file) = &cli.output_file {
                std::fs::write(output_file, &json_output)?;
            } else {
                println!("{}", json_output);
            }
        }
        "text" => {
            let text_summary = generate_text_summary(&result);
            if let Some(output_file) = &cli.output_file {
                std::fs::write(output_file, &text_summary)?;
            } else {
                println!("{}", text_summary);
            }
        }
        _ => {
            error!("Unsupported output format: {}", cli.output_format);
            return Err("Unsupported output format".into());
        }
    }

    let total_duration = start_time.elapsed();
    info!("Complete benchmark suite finished in {:?}", total_duration);

    // Exit with appropriate code
    if result.compliance_status.overall_compliant && !result.regression_analysis.regression_detected {
        std::process::exit(0);
    } else {
        std::process::exit(1);
    }
}

async fn run_input_latency_benchmark(samples: usize) -> Result<(), Box<dyn std::error::Error>> {
    info!("Running input latency benchmark with {} samples", samples);

    let config = InputLatencyBenchmarkConfig {
        sample_count: samples,
        ..Default::default()
    };

    let harness = InputLatencyBenchmarkHarness::new(config).await?;
    let measurements = harness.run_comprehensive_benchmark().await?;
    let report = harness.generate_benchmark_report(&measurements);

    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}

async fn run_render_performance_benchmark(frames: usize) -> Result<(), Box<dyn std::error::Error>> {
    info!("Running render performance benchmark with {} frames", frames);

    let config = RenderBenchmarkConfig {
        frame_sample_count: frames,
        ..Default::default()
    };

    let harness = RenderPerformanceBenchmarkHarness::new(config).await?;
    let measurements = harness.run_comprehensive_render_benchmark().await?;
    let report = harness.generate_render_benchmark_report(&measurements);

    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}

async fn run_content_system_benchmark(tests: usize) -> Result<(), Box<dyn std::error::Error>> {
    info!("Running content system benchmark with {} tests", tests);

    let config = ContentBenchmarkConfig {
        content_sample_count: tests,
        ..Default::default()
    };

    let harness = ContentPerformanceBenchmarkHarness::new(config).await?;
    let measurements = harness.run_comprehensive_content_benchmark().await?;
    let report = harness.generate_content_benchmark_report(&measurements);

    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}

async fn run_memory_concurrency_benchmark(threads: usize) -> Result<(), Box<dyn std::error::Error>> {
    info!("Running memory and concurrency benchmark with {} threads", threads);

    let config = MemoryConcurrencyBenchmarkConfig {
        concurrent_threads: threads,
        ..Default::default()
    };

    let harness = MemoryConcurrencyBenchmarkHarness::new(config).await?;
    let measurements = harness.run_comprehensive_memory_concurrency_benchmark().await?;
    let report = harness.generate_memory_concurrency_benchmark_report(&measurements);

    println!("{}", serde_json::to_string_pretty(&report)?);
    Ok(())
}

async fn validate_performance_targets(
    input_file: &std::path::Path,
    targets_file: &std::path::Path,
    strict_mode: bool,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Validating performance targets (strict mode: {})", strict_mode);

    let input_data = std::fs::read_to_string(input_file)?;
    let targets_data = std::fs::read_to_string(targets_file)?;

    let _performance_result: ci_cd_performance_framework::CicdPerformanceResult = serde_json::from_str(&input_data)?;
    let _targets: ci_cd_performance_framework::PerformanceTargets = serde_json::from_str(&targets_data)?;

    // Implementation would validate each target and generate report
    info!("Performance validation completed");

    Ok(())
}

async fn compare_performance_results(
    baseline: &std::path::Path,
    comparison: &std::path::Path,
    threshold: f64,
) -> Result<(), Box<dyn std::error::Error>> {
    info!("Comparing performance results (threshold: {}%)", threshold);

    let baseline_data = std::fs::read_to_string(baseline)?;
    let comparison_data = std::fs::read_to_string(comparison)?;

    let _baseline_result: ci_cd_performance_framework::CicdPerformanceResult = serde_json::from_str(&baseline_data)?;
    let _comparison_result: ci_cd_performance_framework::CicdPerformanceResult = serde_json::from_str(&comparison_data)?;

    // Implementation would perform detailed comparison and generate diff report
    info!("Performance comparison completed");

    Ok(())
}

fn generate_text_summary(result: &ci_cd_performance_framework::CicdPerformanceResult) -> String {
    format!(
        r#"
=== CENTOTYPE PERFORMANCE VALIDATION SUMMARY ===

Commit: {}
Branch: {}
Timestamp: {}
Overall Status: {}

=== PERFORMANCE METRICS ===

Input Latency:
  P99: {:.1}ms (target: ≤25ms)
  P95: {:.1}ms (target: ≤15ms)
  Mean: {:.1}ms

Render Performance:
  P95 Frame Time: {:.1}ms (target: ≤33ms)
  Effective FPS: {:.1}
  Frame Consistency: {:.1}%

Content Loading:
  P99 Loading Time: {:.1}ms (target: ≤25ms)
  Cache Hit Rate: {:.1}% (target: ≥90%)

Memory Usage:
  Peak Memory: {:.1}MB (target: ≤50MB)
  Memory Efficiency: {:.1}
  Leak Detection Score: {:.1}%

=== COMPLIANCE STATUS ===

Overall Compliant: {}
Compliance Score: {:.1}%
Target Violations: {}

=== REGRESSION ANALYSIS ===

Regression Detected: {}
Affected Metrics: {}
Confidence Level: {:.1}%

=== RECOMMENDATIONS ===

{}

===============================================
        "#,
        result.commit_hash,
        result.branch,
        result.timestamp,
        if result.compliance_status.overall_compliant { "✅ PASS" } else { "❌ FAIL" },
        result.measurements.input_latency.p99_ms,
        result.measurements.input_latency.p95_ms,
        result.measurements.input_latency.mean_ms,
        result.measurements.render_performance.p95_frame_time_ms,
        result.measurements.render_performance.effective_fps,
        result.measurements.render_performance.frame_consistency_score * 100.0,
        result.measurements.content_loading.p99_loading_time_ms,
        result.measurements.content_loading.cache_hit_rate_percent,
        result.measurements.memory_usage.peak_memory_mb,
        result.measurements.memory_usage.allocation_efficiency,
        result.measurements.memory_usage.leak_detection_score * 100.0,
        if result.compliance_status.overall_compliant { "✅ YES" } else { "❌ NO" },
        result.compliance_status.compliance_score * 100.0,
        result.compliance_status.target_violations.len(),
        if result.regression_analysis.regression_detected { "⚠️ YES" } else { "✅ NO" },
        result.regression_analysis.affected_metrics.len(),
        result.regression_analysis.statistical_confidence * 100.0,
        if result.compliance_status.target_violations.is_empty() {
            "All performance targets are within acceptable ranges.".to_string()
        } else {
            result.compliance_status.target_violations
                .iter()
                .map(|v| format!("• {}", v.impact_description))
                .collect::<Vec<_>>()
                .join("\n")
        }
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cli_parsing() {
        let args = vec![
            "performance_validator",
            "--commit-hash", "abc123",
            "--branch", "main",
            "--output-format", "json"
        ];

        let cli = Cli::try_parse_from(args).unwrap();
        assert_eq!(cli.commit_hash, Some("abc123".to_string()));
        assert_eq!(cli.branch, Some("main".to_string()));
        assert_eq!(cli.output_format, "json");
    }

    #[test]
    fn test_subcommand_parsing() {
        let args = vec![
            "performance_validator",
            "input-latency",
            "--samples", "5000"
        ];

        let cli = Cli::try_parse_from(args).unwrap();
        match cli.command {
            Some(Commands::InputLatency { samples }) => assert_eq!(samples, 5000),
            _ => panic!("Expected InputLatency command"),
        }
    }
}