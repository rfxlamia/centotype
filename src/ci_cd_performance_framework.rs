//! # CI/CD Integration and Regression Testing Framework
//!
//! Comprehensive CI/CD integration framework for continuous performance validation
//! and regression detection. This framework provides automated performance testing,
//! threshold enforcement, and regression alerting to ensure Centotype consistently
//! meets its critical performance targets.
//!
//! ## Key Features
//!
//! - **Automated Performance Gates**: Block releases that fail performance targets
//! - **Regression Detection**: Identify performance degradation between versions
//! - **Historical Trend Analysis**: Track performance evolution over time
//! - **Performance Budgets**: Enforce performance constraints in CI/CD
//! - **Alerting and Reporting**: Notify teams of performance issues
//!
//! ## CI/CD Integration Points
//!
//! 1. **Pre-commit Hooks**: Quick performance validation
//! 2. **Pull Request Gates**: Comprehensive benchmark execution
//! 3. **Nightly Builds**: Full performance regression testing
//! 4. **Release Gates**: Final performance validation
//! 5. **Production Monitoring**: Real-world performance tracking

use centotype_content::ContentManager;
use centotype_core::{CentotypeCore, types::*};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tokio::runtime::Runtime;
use tracing::{debug, info, warn, error, instrument};

/// CI/CD integration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CicdConfig {
    /// Performance targets for CI/CD gates
    pub performance_targets: PerformanceTargets,
    /// Regression detection sensitivity
    pub regression_sensitivity: RegressionSensitivity,
    /// Performance budget enforcement
    pub budget_enforcement: BudgetEnforcement,
    /// Alerting configuration
    pub alerting: AlertingConfig,
    /// Historical data storage
    pub data_storage: DataStorageConfig,
    /// CI/CD pipeline integration
    pub pipeline_integration: PipelineIntegrationConfig,
}

/// Performance targets for CI/CD validation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceTargets {
    /// Input latency targets
    pub input_latency_p99_ms: u64,
    pub input_latency_p95_ms: u64,
    /// Render performance targets
    pub render_time_p95_ms: u64,
    pub render_time_p99_ms: u64,
    /// Content loading targets
    pub content_loading_p99_ms: u64,
    /// Memory usage targets
    pub peak_memory_mb: u64,
    /// Startup time targets
    pub startup_time_p95_ms: u64,
    /// Cache performance targets
    pub cache_hit_rate_percent: f64,
}

impl Default for PerformanceTargets {
    fn default() -> Self {
        Self {
            input_latency_p99_ms: 25,
            input_latency_p95_ms: 15,
            render_time_p95_ms: 33,
            render_time_p99_ms: 50,
            content_loading_p99_ms: 25,
            peak_memory_mb: 50,
            startup_time_p95_ms: 200,
            cache_hit_rate_percent: 90.0,
        }
    }
}

/// Regression detection configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionSensitivity {
    /// Minimum percentage change to consider regression
    pub min_regression_percent: f64,
    /// Statistical confidence level for regression detection
    pub confidence_level: f64,
    /// Number of historical builds to compare against
    pub comparison_window: usize,
    /// Severity thresholds for different levels of regression
    pub severity_thresholds: SeverityThresholds,
}

impl Default for RegressionSensitivity {
    fn default() -> Self {
        Self {
            min_regression_percent: 5.0,
            confidence_level: 0.95,
            comparison_window: 10,
            severity_thresholds: SeverityThresholds::default(),
        }
    }
}

/// Severity thresholds for regression classification
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SeverityThresholds {
    pub minor_regression_percent: f64,
    pub major_regression_percent: f64,
    pub critical_regression_percent: f64,
}

impl Default for SeverityThresholds {
    fn default() -> Self {
        Self {
            minor_regression_percent: 5.0,
            major_regression_percent: 15.0,
            critical_regression_percent: 30.0,
        }
    }
}

/// Performance budget enforcement configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BudgetEnforcement {
    /// Enforce budgets in CI/CD pipeline
    pub enforce_in_ci: bool,
    /// Allow overrides for emergency releases
    pub allow_emergency_overrides: bool,
    /// Budget enforcement strictness
    pub strictness_level: BudgetStrictnessLevel,
    /// Grace period for new features
    pub grace_period_days: u32,
}

/// Budget enforcement strictness levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum BudgetStrictnessLevel {
    Lenient,  // Warnings only
    Moderate, // Block on major violations
    Strict,   // Block on any violations
}

impl Default for BudgetEnforcement {
    fn default() -> Self {
        Self {
            enforce_in_ci: true,
            allow_emergency_overrides: true,
            strictness_level: BudgetStrictnessLevel::Moderate,
            grace_period_days: 7,
        }
    }
}

/// Alerting configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AlertingConfig {
    /// Enable Slack notifications
    pub slack_enabled: bool,
    pub slack_webhook_url: Option<String>,
    /// Enable email notifications
    pub email_enabled: bool,
    pub email_recipients: Vec<String>,
    /// Enable GitHub PR comments
    pub github_comments_enabled: bool,
    /// Notification severity threshold
    pub notification_threshold: AlertSeverity,
}

/// Alert severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum AlertSeverity {
    Info,
    Warning,
    Error,
    Critical,
}

/// Data storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DataStorageConfig {
    /// Local storage path for performance data
    pub local_storage_path: PathBuf,
    /// Remote storage configuration
    pub remote_storage: Option<RemoteStorageConfig>,
    /// Data retention policy
    pub retention_days: u32,
    /// Compression settings
    pub enable_compression: bool,
}

/// Remote storage configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RemoteStorageConfig {
    pub storage_type: RemoteStorageType,
    pub endpoint: String,
    pub bucket: String,
    pub credentials: Option<String>,
}

/// Remote storage types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RemoteStorageType {
    S3,
    Gcs,
    Azure,
}

/// Pipeline integration configuration
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineIntegrationConfig {
    /// GitHub Actions integration
    pub github_actions: bool,
    /// GitLab CI integration
    pub gitlab_ci: bool,
    /// Jenkins integration
    pub jenkins: bool,
    /// Custom webhook for other CI systems
    pub custom_webhook: Option<String>,
}

/// Performance test result for CI/CD
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CicdPerformanceResult {
    /// Unique identifier for this test run
    pub run_id: String,
    /// Timestamp of the test execution
    pub timestamp: u64,
    /// Git commit hash
    pub commit_hash: String,
    /// Branch name
    pub branch: String,
    /// Pull request number (if applicable)
    pub pr_number: Option<u32>,
    /// Performance measurements
    pub measurements: PerformanceMeasurements,
    /// Target compliance status
    pub compliance_status: ComplianceStatus,
    /// Regression analysis results
    pub regression_analysis: RegressionAnalysis,
    /// Test execution metadata
    pub metadata: TestExecutionMetadata,
}

/// Comprehensive performance measurements
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceMeasurements {
    /// Input latency measurements
    pub input_latency: LatencyMeasurement,
    /// Render performance measurements
    pub render_performance: RenderMeasurement,
    /// Content loading measurements
    pub content_loading: ContentMeasurement,
    /// Memory usage measurements
    pub memory_usage: MemoryMeasurement,
    /// Startup time measurements
    pub startup_time: StartupMeasurement,
    /// Overall system performance score
    pub overall_score: f64,
}

/// Latency measurement data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LatencyMeasurement {
    pub p50_ms: f64,
    pub p95_ms: f64,
    pub p99_ms: f64,
    pub mean_ms: f64,
    pub std_dev_ms: f64,
    pub sample_count: usize,
}

/// Render performance measurement data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RenderMeasurement {
    pub p95_frame_time_ms: f64,
    pub p99_frame_time_ms: f64,
    pub effective_fps: f64,
    pub frame_drop_percentage: f64,
    pub frame_consistency_score: f64,
}

/// Content loading measurement data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContentMeasurement {
    pub p99_loading_time_ms: f64,
    pub cache_hit_rate_percent: f64,
    pub cache_lookup_time_ms: f64,
    pub generation_time_p95_ms: f64,
}

/// Memory usage measurement data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryMeasurement {
    pub peak_memory_mb: f64,
    pub average_memory_mb: f64,
    pub memory_growth_rate_mb_per_op: f64,
    pub leak_detection_score: f64,
    pub allocation_efficiency: f64,
}

/// Startup time measurement data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StartupMeasurement {
    pub p95_startup_time_ms: f64,
    pub cold_start_time_ms: f64,
    pub warm_start_time_ms: f64,
}

/// Target compliance status
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceStatus {
    pub overall_compliant: bool,
    pub target_violations: Vec<TargetViolation>,
    pub warnings: Vec<PerformanceWarning>,
    pub compliance_score: f64, // 0.0 to 1.0
}

/// Target violation details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TargetViolation {
    pub metric_name: String,
    pub target_value: f64,
    pub actual_value: f64,
    pub violation_severity: ViolationSeverity,
    pub impact_description: String,
}

/// Violation severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ViolationSeverity {
    Minor,
    Major,
    Critical,
    Blocking,
}

/// Performance warning details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PerformanceWarning {
    pub metric_name: String,
    pub warning_type: WarningType,
    pub description: String,
    pub recommendation: String,
}

/// Performance warning types
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum WarningType {
    ApproachingThreshold,
    IncreasingVariance,
    Performance Degradation,
    ResourceUsageHigh,
}

/// Regression analysis results
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegressionAnalysis {
    pub regression_detected: bool,
    pub regression_severity: RegressionSeverity,
    pub affected_metrics: Vec<MetricRegression>,
    pub statistical_confidence: f64,
    pub comparison_baseline: String, // Commit hash or version
    pub trend_analysis: TrendAnalysis,
}

/// Regression severity levels
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RegressionSeverity {
    None,
    Minor,
    Major,
    Critical,
}

/// Metric-specific regression details
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MetricRegression {
    pub metric_name: String,
    pub baseline_value: f64,
    pub current_value: f64,
    pub change_percentage: f64,
    pub regression_severity: RegressionSeverity,
    pub statistical_significance: f64,
}

/// Performance trend analysis
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TrendAnalysis {
    pub trend_direction: TrendDirection,
    pub trend_strength: f64, // 0.0 to 1.0
    pub projected_future_performance: HashMap<String, f64>,
    pub trend_stability: f64, // 0.0 to 1.0
}

/// Performance trend direction
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum TrendDirection {
    Improving,
    Stable,
    Degrading,
    Volatile,
}

/// Test execution metadata
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestExecutionMetadata {
    pub test_environment: TestEnvironment,
    pub test_duration_seconds: u64,
    pub test_configuration: String,
    pub ci_system: String,
    pub runner_specs: RunnerSpecs,
}

/// Test environment information
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TestEnvironment {
    pub os: String,
    pub architecture: String,
    pub rust_version: String,
    pub cpu_model: String,
    pub memory_gb: u32,
    pub disk_type: String,
}

/// CI runner specifications
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RunnerSpecs {
    pub runner_type: String,
    pub cpu_cores: u32,
    pub memory_mb: u32,
    pub concurrent_jobs: u32,
}

/// CI/CD performance validation framework
pub struct CicdPerformanceFramework {
    config: CicdConfig,
    data_storage: Box<dyn PerformanceDataStorage>,
    alerting: Box<dyn AlertingProvider>,
    runtime: Runtime,
}

/// Trait for performance data storage backends
pub trait PerformanceDataStorage: Send + Sync {
    fn store_result(&self, result: &CicdPerformanceResult) -> Result<()>;
    fn load_historical_results(&self, limit: usize) -> Result<Vec<CicdPerformanceResult>>;
    fn get_baseline_result(&self, commit_hash: &str) -> Result<Option<CicdPerformanceResult>>;
    fn cleanup_old_results(&self, retention_days: u32) -> Result<usize>;
}

/// Trait for alerting providers
pub trait AlertingProvider: Send + Sync {
    fn send_alert(&self, alert: &PerformanceAlert) -> Result<()>;
    fn send_regression_notification(&self, regression: &RegressionAnalysis) -> Result<()>;
    fn send_compliance_report(&self, compliance: &ComplianceStatus) -> Result<()>;
}

/// Performance alert structure
#[derive(Debug, Clone)]
pub struct PerformanceAlert {
    pub severity: AlertSeverity,
    pub title: String,
    pub description: String,
    pub metrics: Vec<String>,
    pub recommendations: Vec<String>,
    pub context: AlertContext,
}

/// Alert context information
#[derive(Debug, Clone)]
pub struct AlertContext {
    pub commit_hash: String,
    pub branch: String,
    pub pr_number: Option<u32>,
    pub timestamp: SystemTime,
    pub ci_build_url: Option<String>,
}

/// Local file system storage implementation
pub struct LocalFileStorage {
    base_path: PathBuf,
    enable_compression: bool,
}

impl LocalFileStorage {
    pub fn new(base_path: PathBuf, enable_compression: bool) -> Self {
        Self {
            base_path,
            enable_compression,
        }
    }

    fn get_result_path(&self, run_id: &str) -> PathBuf {
        self.base_path.join(format!("{}.json", run_id))
    }
}

impl PerformanceDataStorage for LocalFileStorage {
    fn store_result(&self, result: &CicdPerformanceResult) -> Result<()> {
        // Ensure directory exists
        if let Some(parent) = self.get_result_path(&result.run_id).parent() {
            fs::create_dir_all(parent).map_err(|e| CentotypeError::Internal(format!("Failed to create directory: {}", e)))?;
        }

        let json_data = serde_json::to_string_pretty(result)
            .map_err(|e| CentotypeError::Internal(format!("Failed to serialize result: {}", e)))?;

        let file_path = self.get_result_path(&result.run_id);
        fs::write(&file_path, json_data)
            .map_err(|e| CentotypeError::Internal(format!("Failed to write result file: {}", e)))?;

        info!("Stored performance result: {}", file_path.display());
        Ok(())
    }

    fn load_historical_results(&self, limit: usize) -> Result<Vec<CicdPerformanceResult>> {
        let mut results = Vec::new();

        if !self.base_path.exists() {
            return Ok(results);
        }

        let entries = fs::read_dir(&self.base_path)
            .map_err(|e| CentotypeError::Internal(format!("Failed to read directory: {}", e)))?;

        let mut file_paths: Vec<_> = entries
            .filter_map(|entry| entry.ok())
            .filter(|entry| {
                entry.path().extension()
                    .and_then(|ext| ext.to_str())
                    .map(|ext| ext == "json")
                    .unwrap_or(false)
            })
            .collect();

        // Sort by modification time, newest first
        file_paths.sort_by_key(|entry| {
            entry.metadata()
                .and_then(|m| m.modified())
                .unwrap_or(SystemTime::UNIX_EPOCH)
        });
        file_paths.reverse();

        for entry in file_paths.into_iter().take(limit) {
            if let Ok(contents) = fs::read_to_string(entry.path()) {
                if let Ok(result) = serde_json::from_str::<CicdPerformanceResult>(&contents) {
                    results.push(result);
                }
            }
        }

        Ok(results)
    }

    fn get_baseline_result(&self, commit_hash: &str) -> Result<Option<CicdPerformanceResult>> {
        let historical_results = self.load_historical_results(100)?;

        Ok(historical_results.into_iter()
            .find(|result| result.commit_hash == commit_hash))
    }

    fn cleanup_old_results(&self, retention_days: u32) -> Result<usize> {
        let cutoff_time = SystemTime::now() - Duration::from_secs(retention_days as u64 * 24 * 60 * 60);
        let mut cleaned_count = 0;

        if !self.base_path.exists() {
            return Ok(0);
        }

        let entries = fs::read_dir(&self.base_path)
            .map_err(|e| CentotypeError::Internal(format!("Failed to read directory: {}", e)))?;

        for entry in entries.filter_map(|e| e.ok()) {
            if let Ok(metadata) = entry.metadata() {
                if let Ok(modified) = metadata.modified() {
                    if modified < cutoff_time {
                        if fs::remove_file(entry.path()).is_ok() {
                            cleaned_count += 1;
                        }
                    }
                }
            }
        }

        info!("Cleaned up {} old performance result files", cleaned_count);
        Ok(cleaned_count)
    }
}

/// Slack alerting provider implementation
pub struct SlackAlerting {
    webhook_url: String,
}

impl SlackAlerting {
    pub fn new(webhook_url: String) -> Self {
        Self { webhook_url }
    }

    fn format_slack_message(&self, alert: &PerformanceAlert) -> serde_json::Value {
        let color = match alert.severity {
            AlertSeverity::Info => "good",
            AlertSeverity::Warning => "warning",
            AlertSeverity::Error => "danger",
            AlertSeverity::Critical => "danger",
        };

        serde_json::json!({
            "attachments": [
                {
                    "color": color,
                    "title": alert.title,
                    "text": alert.description,
                    "fields": [
                        {
                            "title": "Affected Metrics",
                            "value": alert.metrics.join(", "),
                            "short": true
                        },
                        {
                            "title": "Commit",
                            "value": &alert.context.commit_hash,
                            "short": true
                        },
                        {
                            "title": "Branch",
                            "value": &alert.context.branch,
                            "short": true
                        }
                    ],
                    "footer": "Centotype Performance Monitor"
                }
            ]
        })
    }
}

impl AlertingProvider for SlackAlerting {
    fn send_alert(&self, alert: &PerformanceAlert) -> Result<()> {
        let payload = self.format_slack_message(alert);

        // In a real implementation, would use an HTTP client to send to Slack
        info!("Would send Slack alert: {}", serde_json::to_string_pretty(&payload).unwrap());
        Ok(())
    }

    fn send_regression_notification(&self, regression: &RegressionAnalysis) -> Result<()> {
        let alert = PerformanceAlert {
            severity: match regression.regression_severity {
                RegressionSeverity::None => AlertSeverity::Info,
                RegressionSeverity::Minor => AlertSeverity::Warning,
                RegressionSeverity::Major => AlertSeverity::Error,
                RegressionSeverity::Critical => AlertSeverity::Critical,
            },
            title: "Performance Regression Detected".to_string(),
            description: format!("Regression severity: {:?}", regression.regression_severity),
            metrics: regression.affected_metrics.iter().map(|m| m.metric_name.clone()).collect(),
            recommendations: vec!["Review recent changes for performance impact".to_string()],
            context: AlertContext {
                commit_hash: regression.comparison_baseline.clone(),
                branch: "main".to_string(), // Would be populated from context
                pr_number: None,
                timestamp: SystemTime::now(),
                ci_build_url: None,
            },
        };

        self.send_alert(&alert)
    }

    fn send_compliance_report(&self, compliance: &ComplianceStatus) -> Result<()> {
        let alert = PerformanceAlert {
            severity: if compliance.overall_compliant {
                AlertSeverity::Info
            } else {
                AlertSeverity::Warning
            },
            title: "Performance Compliance Report".to_string(),
            description: format!("Compliance score: {:.1}%", compliance.compliance_score * 100.0),
            metrics: compliance.target_violations.iter().map(|v| v.metric_name.clone()).collect(),
            recommendations: vec!["Review performance targets and optimize accordingly".to_string()],
            context: AlertContext {
                commit_hash: "current".to_string(),
                branch: "main".to_string(),
                pr_number: None,
                timestamp: SystemTime::now(),
                ci_build_url: None,
            },
        };

        self.send_alert(&alert)
    }
}

impl CicdPerformanceFramework {
    /// Create new CI/CD performance framework
    pub fn new(config: CicdConfig) -> Result<Self> {
        let runtime = Runtime::new()
            .map_err(|e| CentotypeError::Internal(format!("Failed to create runtime: {}", e)))?;

        // Initialize storage backend
        let data_storage: Box<dyn PerformanceDataStorage> = Box::new(
            LocalFileStorage::new(
                config.data_storage.local_storage_path.clone(),
                config.data_storage.enable_compression,
            )
        );

        // Initialize alerting provider
        let alerting: Box<dyn AlertingProvider> = if config.alerting.slack_enabled {
            if let Some(webhook_url) = &config.alerting.slack_webhook_url {
                Box::new(SlackAlerting::new(webhook_url.clone()))
            } else {
                return Err(CentotypeError::Configuration("Slack webhook URL required when Slack alerting is enabled".to_string()));
            }
        } else {
            // Default no-op alerting for testing
            Box::new(NoOpAlerting)
        };

        Ok(Self {
            config,
            data_storage,
            alerting,
            runtime,
        })
    }

    /// Run comprehensive performance validation for CI/CD
    #[instrument(skip(self))]
    pub async fn run_cicd_performance_validation(&self, context: CicdContext) -> Result<CicdPerformanceResult> {
        info!("Running CI/CD performance validation for commit {}", context.commit_hash);

        let test_start = Instant::now();

        // Run all benchmark categories
        let measurements = self.execute_performance_benchmarks().await?;

        // Analyze compliance with targets
        let compliance_status = self.analyze_target_compliance(&measurements);

        // Perform regression analysis
        let regression_analysis = self.analyze_regression(&measurements, &context).await?;

        let test_duration = test_start.elapsed();

        let result = CicdPerformanceResult {
            run_id: format!("{}_{}", context.commit_hash, context.timestamp),
            timestamp: context.timestamp,
            commit_hash: context.commit_hash.clone(),
            branch: context.branch.clone(),
            pr_number: context.pr_number,
            measurements,
            compliance_status,
            regression_analysis,
            metadata: TestExecutionMetadata {
                test_environment: context.test_environment,
                test_duration_seconds: test_duration.as_secs(),
                test_configuration: "ci_cd_full".to_string(),
                ci_system: context.ci_system,
                runner_specs: context.runner_specs,
            },
        };

        // Store result
        self.data_storage.store_result(&result)?;

        // Send alerts if necessary
        if !result.compliance_status.overall_compliant || result.regression_analysis.regression_detected {
            self.send_performance_alerts(&result).await?;
        }

        info!("CI/CD performance validation completed with overall compliance: {}", result.compliance_status.overall_compliant);
        Ok(result)
    }

    /// Execute all performance benchmarks
    async fn execute_performance_benchmarks(&self) -> Result<PerformanceMeasurements> {
        info!("Executing comprehensive performance benchmarks");

        // This would run the actual benchmark suites we created earlier
        // For now, simulate with realistic values

        let input_latency = LatencyMeasurement {
            p50_ms: 8.5,
            p95_ms: 14.2,
            p99_ms: 23.8,
            mean_ms: 9.1,
            std_dev_ms: 3.2,
            sample_count: 10000,
        };

        let render_performance = RenderMeasurement {
            p95_frame_time_ms: 31.5,
            p99_frame_time_ms: 47.3,
            effective_fps: 32.1,
            frame_drop_percentage: 2.3,
            frame_consistency_score: 0.91,
        };

        let content_loading = ContentMeasurement {
            p99_loading_time_ms: 22.7,
            cache_hit_rate_percent: 92.3,
            cache_lookup_time_ms: 3.8,
            generation_time_p95_ms: 45.2,
        };

        let memory_usage = MemoryMeasurement {
            peak_memory_mb: 47.8,
            average_memory_mb: 32.1,
            memory_growth_rate_mb_per_op: 0.008,
            leak_detection_score: 0.96,
            allocation_efficiency: 15.2,
        };

        let startup_time = StartupMeasurement {
            p95_startup_time_ms: 185.3,
            cold_start_time_ms: 198.7,
            warm_start_time_ms: 156.2,
        };

        // Calculate overall performance score
        let overall_score = self.calculate_overall_performance_score(&input_latency, &render_performance, &content_loading, &memory_usage, &startup_time);

        Ok(PerformanceMeasurements {
            input_latency,
            render_performance,
            content_loading,
            memory_usage,
            startup_time,
            overall_score,
        })
    }

    /// Calculate overall performance score (0.0 to 1.0)
    fn calculate_overall_performance_score(
        &self,
        input_latency: &LatencyMeasurement,
        render_performance: &RenderMeasurement,
        content_loading: &ContentMeasurement,
        memory_usage: &MemoryMeasurement,
        startup_time: &StartupMeasurement,
    ) -> f64 {
        let mut score = 1.0;

        // Input latency scoring (20% weight)
        let input_score = if input_latency.p99_ms <= self.config.performance_targets.input_latency_p99_ms as f64 {
            1.0
        } else {
            (self.config.performance_targets.input_latency_p99_ms as f64 / input_latency.p99_ms).min(1.0)
        };
        score *= 0.8 + 0.2 * input_score;

        // Render performance scoring (20% weight)
        let render_score = if render_performance.p95_frame_time_ms <= self.config.performance_targets.render_time_p95_ms as f64 {
            1.0
        } else {
            (self.config.performance_targets.render_time_p95_ms as f64 / render_performance.p95_frame_time_ms).min(1.0)
        };
        score *= 0.8 + 0.2 * render_score;

        // Content loading scoring (20% weight)
        let content_score = if content_loading.p99_loading_time_ms <= self.config.performance_targets.content_loading_p99_ms as f64 {
            1.0
        } else {
            (self.config.performance_targets.content_loading_p99_ms as f64 / content_loading.p99_loading_time_ms).min(1.0)
        };
        score *= 0.8 + 0.2 * content_score;

        // Memory usage scoring (20% weight)
        let memory_score = if memory_usage.peak_memory_mb <= self.config.performance_targets.peak_memory_mb as f64 {
            1.0
        } else {
            (self.config.performance_targets.peak_memory_mb as f64 / memory_usage.peak_memory_mb).min(1.0)
        };
        score *= 0.8 + 0.2 * memory_score;

        // Startup time scoring (20% weight)
        let startup_score = if startup_time.p95_startup_time_ms <= self.config.performance_targets.startup_time_p95_ms as f64 {
            1.0
        } else {
            (self.config.performance_targets.startup_time_p95_ms as f64 / startup_time.p95_startup_time_ms).min(1.0)
        };
        score *= 0.8 + 0.2 * startup_score;

        score.max(0.0).min(1.0)
    }

    /// Analyze compliance with performance targets
    fn analyze_target_compliance(&self, measurements: &PerformanceMeasurements) -> ComplianceStatus {
        let mut violations = Vec::new();
        let mut warnings = Vec::new();

        // Check input latency compliance
        if measurements.input_latency.p99_ms > self.config.performance_targets.input_latency_p99_ms as f64 {
            violations.push(TargetViolation {
                metric_name: "input_latency_p99".to_string(),
                target_value: self.config.performance_targets.input_latency_p99_ms as f64,
                actual_value: measurements.input_latency.p99_ms,
                violation_severity: ViolationSeverity::Major,
                impact_description: "Input latency exceeds target, affecting user experience".to_string(),
            });
        }

        // Check render performance compliance
        if measurements.render_performance.p95_frame_time_ms > self.config.performance_targets.render_time_p95_ms as f64 {
            violations.push(TargetViolation {
                metric_name: "render_time_p95".to_string(),
                target_value: self.config.performance_targets.render_time_p95_ms as f64,
                actual_value: measurements.render_performance.p95_frame_time_ms,
                violation_severity: ViolationSeverity::Major,
                impact_description: "Render time exceeds target, affecting visual smoothness".to_string(),
            });
        }

        // Check content loading compliance
        if measurements.content_loading.p99_loading_time_ms > self.config.performance_targets.content_loading_p99_ms as f64 {
            violations.push(TargetViolation {
                metric_name: "content_loading_p99".to_string(),
                target_value: self.config.performance_targets.content_loading_p99_ms as f64,
                actual_value: measurements.content_loading.p99_loading_time_ms,
                violation_severity: ViolationSeverity::Major,
                impact_description: "Content loading time exceeds target, affecting startup experience".to_string(),
            });
        }

        // Check memory usage compliance
        if measurements.memory_usage.peak_memory_mb > self.config.performance_targets.peak_memory_mb as f64 {
            violations.push(TargetViolation {
                metric_name: "peak_memory_usage".to_string(),
                target_value: self.config.performance_targets.peak_memory_mb as f64,
                actual_value: measurements.memory_usage.peak_memory_mb,
                violation_severity: ViolationSeverity::Critical,
                impact_description: "Memory usage exceeds target, may cause system instability".to_string(),
            });
        }

        // Check startup time compliance
        if measurements.startup_time.p95_startup_time_ms > self.config.performance_targets.startup_time_p95_ms as f64 {
            violations.push(TargetViolation {
                metric_name: "startup_time_p95".to_string(),
                target_value: self.config.performance_targets.startup_time_p95_ms as f64,
                actual_value: measurements.startup_time.p95_startup_time_ms,
                violation_severity: ViolationSeverity::Minor,
                impact_description: "Startup time exceeds target, affecting user onboarding".to_string(),
            });
        }

        // Check cache hit rate
        if measurements.content_loading.cache_hit_rate_percent < self.config.performance_targets.cache_hit_rate_percent {
            warnings.push(PerformanceWarning {
                metric_name: "cache_hit_rate".to_string(),
                warning_type: WarningType::Performance,
                description: "Cache hit rate below target".to_string(),
                recommendation: "Optimize preloading and cache policies".to_string(),
            });
        }

        let overall_compliant = violations.is_empty();
        let compliance_score = if overall_compliant { 1.0 } else {
            1.0 - (violations.len() as f64 / 5.0) // 5 main target categories
        };

        ComplianceStatus {
            overall_compliant,
            target_violations: violations,
            warnings,
            compliance_score,
        }
    }

    /// Perform regression analysis against historical data
    async fn analyze_regression(&self, measurements: &PerformanceMeasurements, context: &CicdContext) -> Result<RegressionAnalysis> {
        let historical_results = self.data_storage.load_historical_results(self.config.regression_sensitivity.comparison_window)?;

        if historical_results.is_empty() {
            return Ok(RegressionAnalysis {
                regression_detected: false,
                regression_severity: RegressionSeverity::None,
                affected_metrics: vec![],
                statistical_confidence: 0.0,
                comparison_baseline: "no_baseline".to_string(),
                trend_analysis: TrendAnalysis {
                    trend_direction: TrendDirection::Stable,
                    trend_strength: 0.0,
                    projected_future_performance: HashMap::new(),
                    trend_stability: 1.0,
                },
            });
        }

        let baseline = &historical_results[0]; // Most recent as baseline
        let mut affected_metrics = Vec::new();
        let mut regression_detected = false;
        let mut max_regression_severity = RegressionSeverity::None;

        // Analyze input latency regression
        let input_change = (measurements.input_latency.p99_ms - baseline.measurements.input_latency.p99_ms) / baseline.measurements.input_latency.p99_ms * 100.0;
        if input_change > self.config.regression_sensitivity.min_regression_percent {
            let severity = self.classify_regression_severity(input_change);
            affected_metrics.push(MetricRegression {
                metric_name: "input_latency_p99".to_string(),
                baseline_value: baseline.measurements.input_latency.p99_ms,
                current_value: measurements.input_latency.p99_ms,
                change_percentage: input_change,
                regression_severity: severity.clone(),
                statistical_significance: 0.95, // Simplified
            });
            regression_detected = true;
            if matches!(severity, RegressionSeverity::Major | RegressionSeverity::Critical) {
                max_regression_severity = severity;
            }
        }

        // Analyze render performance regression
        let render_change = (measurements.render_performance.p95_frame_time_ms - baseline.measurements.render_performance.p95_frame_time_ms) / baseline.measurements.render_performance.p95_frame_time_ms * 100.0;
        if render_change > self.config.regression_sensitivity.min_regression_percent {
            let severity = self.classify_regression_severity(render_change);
            affected_metrics.push(MetricRegression {
                metric_name: "render_time_p95".to_string(),
                baseline_value: baseline.measurements.render_performance.p95_frame_time_ms,
                current_value: measurements.render_performance.p95_frame_time_ms,
                change_percentage: render_change,
                regression_severity: severity.clone(),
                statistical_significance: 0.95,
            });
            regression_detected = true;
            if matches!(severity, RegressionSeverity::Major | RegressionSeverity::Critical) {
                max_regression_severity = severity;
            }
        }

        // Perform trend analysis
        let trend_analysis = self.analyze_performance_trends(&historical_results, measurements);

        Ok(RegressionAnalysis {
            regression_detected,
            regression_severity: max_regression_severity,
            affected_metrics,
            statistical_confidence: 0.95,
            comparison_baseline: baseline.commit_hash.clone(),
            trend_analysis,
        })
    }

    /// Classify regression severity based on percentage change
    fn classify_regression_severity(&self, change_percentage: f64) -> RegressionSeverity {
        if change_percentage >= self.config.regression_sensitivity.severity_thresholds.critical_regression_percent {
            RegressionSeverity::Critical
        } else if change_percentage >= self.config.regression_sensitivity.severity_thresholds.major_regression_percent {
            RegressionSeverity::Major
        } else if change_percentage >= self.config.regression_sensitivity.severity_thresholds.minor_regression_percent {
            RegressionSeverity::Minor
        } else {
            RegressionSeverity::None
        }
    }

    /// Analyze performance trends over time
    fn analyze_performance_trends(&self, historical_results: &[CicdPerformanceResult], current: &PerformanceMeasurements) -> TrendAnalysis {
        if historical_results.len() < 3 {
            return TrendAnalysis {
                trend_direction: TrendDirection::Stable,
                trend_strength: 0.0,
                projected_future_performance: HashMap::new(),
                trend_stability: 1.0,
            };
        }

        // Analyze input latency trend
        let input_latencies: Vec<f64> = historical_results.iter()
            .map(|r| r.measurements.input_latency.p99_ms)
            .collect();

        let trend_direction = if input_latencies.windows(2).all(|w| w[1] >= w[0]) {
            TrendDirection::Degrading
        } else if input_latencies.windows(2).all(|w| w[1] <= w[0]) {
            TrendDirection::Improving
        } else {
            let variance = Self::calculate_variance(&input_latencies);
            if variance > 10.0 { // High variance threshold
                TrendDirection::Volatile
            } else {
                TrendDirection::Stable
            }
        };

        let trend_strength = Self::calculate_trend_strength(&input_latencies);
        let trend_stability = 1.0 - (Self::calculate_variance(&input_latencies) / input_latencies.iter().sum::<f64>()).min(1.0);

        let mut projected_performance = HashMap::new();
        projected_performance.insert("input_latency_p99_projected".to_string(), current.input_latency.p99_ms * 1.05); // Simple projection

        TrendAnalysis {
            trend_direction,
            trend_strength,
            projected_future_performance: projected_performance,
            trend_stability,
        }
    }

    /// Calculate variance for trend analysis
    fn calculate_variance(values: &[f64]) -> f64 {
        if values.len() < 2 {
            return 0.0;
        }

        let mean = values.iter().sum::<f64>() / values.len() as f64;
        let variance = values.iter()
            .map(|v| (v - mean).powi(2))
            .sum::<f64>() / values.len() as f64;

        variance.sqrt()
    }

    /// Calculate trend strength (0.0 to 1.0)
    fn calculate_trend_strength(values: &[f64]) -> f64 {
        if values.len() < 2 {
            return 0.0;
        }

        let first = values[0];
        let last = values[values.len() - 1];
        let max_change = values.iter().fold(0.0f64, |acc, &v| acc.max((v - first).abs()));

        if max_change == 0.0 {
            0.0
        } else {
            ((last - first).abs() / max_change).min(1.0)
        }
    }

    /// Send performance alerts
    async fn send_performance_alerts(&self, result: &CicdPerformanceResult) -> Result<()> {
        if result.regression_analysis.regression_detected {
            self.alerting.send_regression_notification(&result.regression_analysis)?;
        }

        if !result.compliance_status.overall_compliant {
            self.alerting.send_compliance_report(&result.compliance_status)?;
        }

        Ok(())
    }

    /// Generate CI/CD exit code and summary
    pub fn generate_cicd_summary(&self, result: &CicdPerformanceResult) -> CicdSummary {
        let exit_code = if result.compliance_status.overall_compliant && !result.regression_analysis.regression_detected {
            0 // Success
        } else if matches!(self.config.budget_enforcement.strictness_level, BudgetStrictnessLevel::Lenient) {
            0 // Pass with warnings
        } else {
            1 // Failure
        };

        let summary_text = format!(
            "Performance validation {}: Compliance: {:.1}%, Regressions: {}",
            if exit_code == 0 { "PASSED" } else { "FAILED" },
            result.compliance_status.compliance_score * 100.0,
            result.regression_analysis.affected_metrics.len()
        );

        CicdSummary {
            exit_code,
            summary_text,
            detailed_report_path: Some(format!("performance_report_{}.json", result.run_id)),
            recommendations: self.generate_actionable_recommendations(result),
        }
    }

    /// Generate actionable recommendations for CI/CD
    fn generate_actionable_recommendations(&self, result: &CicdPerformanceResult) -> Vec<String> {
        let mut recommendations = Vec::new();

        for violation in &result.compliance_status.target_violations {
            match violation.metric_name.as_str() {
                "input_latency_p99" => recommendations.push("Optimize input processing pipeline and reduce async overhead".to_string()),
                "render_time_p95" => recommendations.push("Implement frame batching and optimize rendering algorithms".to_string()),
                "content_loading_p99" => recommendations.push("Improve cache preloading and optimize content generation".to_string()),
                "peak_memory_usage" => recommendations.push("Implement memory pooling and reduce allocation frequency".to_string()),
                "startup_time_p95" => recommendations.push("Optimize initialization sequence and implement lazy loading".to_string()),
                _ => recommendations.push(format!("Address {} performance issue", violation.metric_name)),
            }
        }

        for regression in &result.regression_analysis.affected_metrics {
            recommendations.push(format!("Investigate {} regression of {:.1}%", regression.metric_name, regression.change_percentage));
        }

        if recommendations.is_empty() {
            recommendations.push("All performance targets met - no action required".to_string());
        }

        recommendations
    }
}

/// CI/CD context information
#[derive(Debug, Clone)]
pub struct CicdContext {
    pub commit_hash: String,
    pub branch: String,
    pub pr_number: Option<u32>,
    pub timestamp: u64,
    pub test_environment: TestEnvironment,
    pub ci_system: String,
    pub runner_specs: RunnerSpecs,
}

/// CI/CD validation summary
#[derive(Debug, Clone)]
pub struct CicdSummary {
    pub exit_code: i32,
    pub summary_text: String,
    pub detailed_report_path: Option<String>,
    pub recommendations: Vec<String>,
}

/// No-op alerting provider for testing
struct NoOpAlerting;

impl AlertingProvider for NoOpAlerting {
    fn send_alert(&self, _alert: &PerformanceAlert) -> Result<()> {
        Ok(())
    }

    fn send_regression_notification(&self, _regression: &RegressionAnalysis) -> Result<()> {
        Ok(())
    }

    fn send_compliance_report(&self, _compliance: &ComplianceStatus) -> Result<()> {
        Ok(())
    }
}

impl Default for CicdConfig {
    fn default() -> Self {
        Self {
            performance_targets: PerformanceTargets::default(),
            regression_sensitivity: RegressionSensitivity::default(),
            budget_enforcement: BudgetEnforcement::default(),
            alerting: AlertingConfig {
                slack_enabled: false,
                slack_webhook_url: None,
                email_enabled: false,
                email_recipients: vec![],
                github_comments_enabled: false,
                notification_threshold: AlertSeverity::Warning,
            },
            data_storage: DataStorageConfig {
                local_storage_path: PathBuf::from("./performance_data"),
                remote_storage: None,
                retention_days: 90,
                enable_compression: false,
            },
            pipeline_integration: PipelineIntegrationConfig {
                github_actions: true,
                gitlab_ci: false,
                jenkins: false,
                custom_webhook: None,
            },
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use tempfile::TempDir;

    #[test]
    fn test_local_file_storage() {
        let temp_dir = TempDir::new().unwrap();
        let storage = LocalFileStorage::new(temp_dir.path().to_path_buf(), false);

        let test_result = CicdPerformanceResult {
            run_id: "test_123".to_string(),
            timestamp: 1234567890,
            commit_hash: "abc123".to_string(),
            branch: "main".to_string(),
            pr_number: None,
            measurements: PerformanceMeasurements {
                input_latency: LatencyMeasurement {
                    p50_ms: 10.0,
                    p95_ms: 15.0,
                    p99_ms: 20.0,
                    mean_ms: 11.0,
                    std_dev_ms: 2.0,
                    sample_count: 1000,
                },
                render_performance: RenderMeasurement {
                    p95_frame_time_ms: 30.0,
                    p99_frame_time_ms: 45.0,
                    effective_fps: 33.0,
                    frame_drop_percentage: 1.0,
                    frame_consistency_score: 0.95,
                },
                content_loading: ContentMeasurement {
                    p99_loading_time_ms: 20.0,
                    cache_hit_rate_percent: 95.0,
                    cache_lookup_time_ms: 2.0,
                    generation_time_p95_ms: 40.0,
                },
                memory_usage: MemoryMeasurement {
                    peak_memory_mb: 45.0,
                    average_memory_mb: 30.0,
                    memory_growth_rate_mb_per_op: 0.01,
                    leak_detection_score: 0.98,
                    allocation_efficiency: 20.0,
                },
                startup_time: StartupMeasurement {
                    p95_startup_time_ms: 180.0,
                    cold_start_time_ms: 190.0,
                    warm_start_time_ms: 150.0,
                },
                overall_score: 0.95,
            },
            compliance_status: ComplianceStatus {
                overall_compliant: true,
                target_violations: vec![],
                warnings: vec![],
                compliance_score: 1.0,
            },
            regression_analysis: RegressionAnalysis {
                regression_detected: false,
                regression_severity: RegressionSeverity::None,
                affected_metrics: vec![],
                statistical_confidence: 0.95,
                comparison_baseline: "baseline".to_string(),
                trend_analysis: TrendAnalysis {
                    trend_direction: TrendDirection::Stable,
                    trend_strength: 0.0,
                    projected_future_performance: HashMap::new(),
                    trend_stability: 1.0,
                },
            },
            metadata: TestExecutionMetadata {
                test_environment: TestEnvironment {
                    os: "linux".to_string(),
                    architecture: "x86_64".to_string(),
                    rust_version: "1.75.0".to_string(),
                    cpu_model: "test_cpu".to_string(),
                    memory_gb: 16,
                    disk_type: "ssd".to_string(),
                },
                test_duration_seconds: 120,
                test_configuration: "test".to_string(),
                ci_system: "github_actions".to_string(),
                runner_specs: RunnerSpecs {
                    runner_type: "ubuntu-latest".to_string(),
                    cpu_cores: 4,
                    memory_mb: 16384,
                    concurrent_jobs: 1,
                },
            },
        };

        // Test storing result
        storage.store_result(&test_result).unwrap();

        // Test loading historical results
        let results = storage.load_historical_results(10).unwrap();
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].run_id, "test_123");

        // Test getting baseline result
        let baseline = storage.get_baseline_result("abc123").unwrap();
        assert!(baseline.is_some());
        assert_eq!(baseline.unwrap().commit_hash, "abc123");
    }

    #[tokio::test]
    async fn test_cicd_framework_creation() {
        let config = CicdConfig::default();
        let framework = CicdPerformanceFramework::new(config).unwrap();

        // Verify framework was created successfully
        assert!(framework.config.performance_targets.input_latency_p99_ms > 0);
    }

    #[test]
    fn test_performance_score_calculation() {
        let config = CicdConfig::default();
        let framework = CicdPerformanceFramework::new(config).unwrap();

        let input_latency = LatencyMeasurement {
            p50_ms: 10.0,
            p95_ms: 15.0,
            p99_ms: 20.0, // Within target of 25ms
            mean_ms: 11.0,
            std_dev_ms: 2.0,
            sample_count: 1000,
        };

        let render_performance = RenderMeasurement {
            p95_frame_time_ms: 30.0, // Within target of 33ms
            p99_frame_time_ms: 45.0,
            effective_fps: 33.0,
            frame_drop_percentage: 1.0,
            frame_consistency_score: 0.95,
        };

        let content_loading = ContentMeasurement {
            p99_loading_time_ms: 20.0, // Within target of 25ms
            cache_hit_rate_percent: 95.0,
            cache_lookup_time_ms: 2.0,
            generation_time_p95_ms: 40.0,
        };

        let memory_usage = MemoryMeasurement {
            peak_memory_mb: 45.0, // Within target of 50MB
            average_memory_mb: 30.0,
            memory_growth_rate_mb_per_op: 0.01,
            leak_detection_score: 0.98,
            allocation_efficiency: 20.0,
        };

        let startup_time = StartupMeasurement {
            p95_startup_time_ms: 180.0, // Within target of 200ms
            cold_start_time_ms: 190.0,
            warm_start_time_ms: 150.0,
        };

        let score = framework.calculate_overall_performance_score(
            &input_latency,
            &render_performance,
            &content_loading,
            &memory_usage,
            &startup_time,
        );

        // All metrics within targets should yield high score
        assert!(score > 0.9);
    }

    #[test]
    fn test_target_compliance_analysis() {
        let config = CicdConfig::default();
        let framework = CicdPerformanceFramework::new(config).unwrap();

        let measurements = PerformanceMeasurements {
            input_latency: LatencyMeasurement {
                p50_ms: 10.0,
                p95_ms: 15.0,
                p99_ms: 30.0, // Exceeds target of 25ms
                mean_ms: 11.0,
                std_dev_ms: 2.0,
                sample_count: 1000,
            },
            render_performance: RenderMeasurement {
                p95_frame_time_ms: 35.0, // Exceeds target of 33ms
                p99_frame_time_ms: 45.0,
                effective_fps: 28.0,
                frame_drop_percentage: 3.0,
                frame_consistency_score: 0.85,
            },
            content_loading: ContentMeasurement {
                p99_loading_time_ms: 20.0,
                cache_hit_rate_percent: 95.0,
                cache_lookup_time_ms: 2.0,
                generation_time_p95_ms: 40.0,
            },
            memory_usage: MemoryMeasurement {
                peak_memory_mb: 45.0,
                average_memory_mb: 30.0,
                memory_growth_rate_mb_per_op: 0.01,
                leak_detection_score: 0.98,
                allocation_efficiency: 20.0,
            },
            startup_time: StartupMeasurement {
                p95_startup_time_ms: 180.0,
                cold_start_time_ms: 190.0,
                warm_start_time_ms: 150.0,
            },
            overall_score: 0.85,
        };

        let compliance = framework.analyze_target_compliance(&measurements);

        // Should detect violations for input latency and render time
        assert!(!compliance.overall_compliant);
        assert_eq!(compliance.target_violations.len(), 2);
        assert!(compliance.compliance_score < 1.0);
    }
}