//! # Optimized Cache Warming and Preloading Strategies
//!
//! This module implements intelligent preloading strategies designed to achieve >90% cache hit rates
//! and minimize the P99 content loading latency to <25ms by predicting user behavior and warming
//! the cache proactively.
//!
//! ## Key Strategies
//!
//! 1. **Adaptive Sequential Preloading**: Learns from user progression patterns
//! 2. **Difficulty-Based Preloading**: Preloads based on level difficulty patterns
//! 3. **Time-Based Preloading**: Uses session timing to predict next accesses
//! 4. **Background Cache Warming**: Continuously maintains hot cache entries
//! 5. **Failure Recovery Preloading**: Preloads alternative content on errors

use centotype_content::{ContentManager, PreloadStrategy};
use centotype_core::types::*;
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{RwLock, Semaphore};
use tokio::time::{interval, sleep};
use tracing::{debug, info, warn, instrument};

/// Intelligent preloading orchestrator that adapts to user behavior
pub struct AdaptivePreloadManager {
    content_manager: Arc<ContentManager>,
    user_behavior_tracker: Arc<RwLock<UserBehaviorTracker>>,
    preload_strategies: Vec<Box<dyn PreloadStrategy + Send + Sync>>,
    background_preloader: Arc<BackgroundPreloader>,
    config: PreloadConfig,
    metrics: Arc<RwLock<PreloadMetrics>>,
}

/// Configuration for preloading behavior
#[derive(Debug, Clone)]
pub struct PreloadConfig {
    /// Maximum levels to preload ahead
    pub max_preload_distance: u8,
    /// Maximum concurrent preload operations
    pub max_concurrent_preloads: usize,
    /// Minimum time between preload decisions
    pub preload_decision_interval: Duration,
    /// Cache warming interval for background tasks
    pub background_warming_interval: Duration,
    /// Enable failure recovery preloading
    pub enable_failure_recovery: bool,
    /// User behavior learning window (number of sessions)
    pub behavior_learning_window: usize,
}

impl Default for PreloadConfig {
    fn default() -> Self {
        Self {
            max_preload_distance: 5,
            max_concurrent_preloads: 3,
            preload_decision_interval: Duration::from_millis(100),
            background_warming_interval: Duration::from_secs(30),
            enable_failure_recovery: true,
            behavior_learning_window: 100,
        }
    }
}

/// Tracks user behavior patterns to improve preloading decisions
#[derive(Debug, Clone, Default)]
pub struct UserBehaviorTracker {
    /// Level access patterns (level -> frequency)
    access_patterns: HashMap<LevelId, AccessPattern>,
    /// Session progression patterns
    progression_patterns: VecDeque<ProgressionPattern>,
    /// Timing patterns for level transitions
    timing_patterns: HashMap<LevelId, TimingPattern>,
    /// Error recovery patterns
    error_recovery_patterns: HashMap<LevelId, Vec<LevelId>>,
    /// Current session state
    current_session: Option<SessionPattern>,
}

/// Access pattern for a specific level
#[derive(Debug, Clone)]
pub struct AccessPattern {
    pub total_accesses: u64,
    pub cache_hits: u64,
    pub cache_misses: u64,
    pub avg_session_duration: Duration,
    pub typical_next_levels: Vec<(LevelId, f64)>, // (level, probability)
    pub last_accessed: Instant,
}

/// User progression pattern within a session
#[derive(Debug, Clone)]
pub struct ProgressionPattern {
    pub session_id: uuid::Uuid,
    pub levels_attempted: Vec<LevelId>,
    pub total_duration: Duration,
    pub success_rate: f64,
    pub retry_patterns: HashMap<LevelId, u32>,
    pub timestamp: Instant,
}

/// Timing patterns for level transitions
#[derive(Debug, Clone)]
pub struct TimingPattern {
    pub avg_time_on_level: Duration,
    pub transition_times: HashMap<LevelId, Duration>,
    pub peak_usage_hours: Vec<u8>, // Hours of day when level is most accessed
}

/// Current session tracking
#[derive(Debug, Clone)]
pub struct SessionPattern {
    pub session_id: uuid::Uuid,
    pub start_time: Instant,
    pub current_level: LevelId,
    pub levels_this_session: Vec<LevelId>,
    pub predicted_next_levels: Vec<(LevelId, f64)>,
    pub last_action_time: Instant,
}

/// Preloading performance metrics
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct PreloadMetrics {
    pub total_preloads_triggered: u64,
    pub successful_preloads: u64,
    pub failed_preloads: u64,
    pub cache_hit_improvement: f64,
    pub avg_preload_latency: Duration,
    pub background_warming_cycles: u64,
    pub user_behavior_predictions_correct: u64,
    pub user_behavior_predictions_total: u64,
    pub memory_usage_for_preloading: usize,
}

impl PreloadMetrics {
    /// Calculate preload success rate
    pub fn success_rate(&self) -> f64 {
        if self.total_preloads_triggered == 0 {
            0.0
        } else {
            (self.successful_preloads as f64 / self.total_preloads_triggered as f64) * 100.0
        }
    }

    /// Calculate prediction accuracy
    pub fn prediction_accuracy(&self) -> f64 {
        if self.user_behavior_predictions_total == 0 {
            0.0
        } else {
            (self.user_behavior_predictions_correct as f64 / self.user_behavior_predictions_total as f64) * 100.0
        }
    }
}

/// Trait for different preloading strategies
pub trait PreloadStrategy: std::fmt::Debug + Send + Sync {
    /// Predict which levels should be preloaded based on current context
    async fn predict_preload_candidates(
        &self,
        current_level: LevelId,
        behavior_tracker: &UserBehaviorTracker,
        config: &PreloadConfig,
    ) -> Vec<PreloadCandidate>;

    /// Get the priority weight for this strategy
    fn get_priority_weight(&self) -> f64;

    /// Get strategy name for debugging
    fn name(&self) -> &'static str;
}

/// A candidate level for preloading with priority score
#[derive(Debug, Clone)]
pub struct PreloadCandidate {
    pub level_id: LevelId,
    pub priority_score: f64,
    pub predicted_access_time: Option<Duration>,
    pub reason: String,
}

/// Sequential preloading based on level progression
#[derive(Debug)]
pub struct SequentialPreloadStrategy {
    preload_distance: u8,
    priority_weight: f64,
}

impl SequentialPreloadStrategy {
    pub fn new(preload_distance: u8, priority_weight: f64) -> Self {
        Self {
            preload_distance,
            priority_weight,
        }
    }
}

impl PreloadStrategy for SequentialPreloadStrategy {
    async fn predict_preload_candidates(
        &self,
        current_level: LevelId,
        _behavior_tracker: &UserBehaviorTracker,
        _config: &PreloadConfig,
    ) -> Vec<PreloadCandidate> {
        let mut candidates = Vec::new();

        for i in 1..=self.preload_distance {
            let next_level_num = current_level.0 + i;
            if next_level_num <= LevelId::MAX {
                if let Ok(next_level) = LevelId::new(next_level_num) {
                    let priority = self.priority_weight * (1.0 - (i as f64 / self.preload_distance as f64));
                    candidates.push(PreloadCandidate {
                        level_id: next_level,
                        priority_score: priority,
                        predicted_access_time: Some(Duration::from_secs(i as u64 * 30)), // Estimate 30s per level
                        reason: format!("Sequential progression from level {}", current_level.0),
                    });
                }
            }
        }

        candidates
    }

    fn get_priority_weight(&self) -> f64 {
        self.priority_weight
    }

    fn name(&self) -> &'static str {
        "Sequential"
    }
}

/// Adaptive preloading based on learned user behavior
#[derive(Debug)]
pub struct AdaptivePreloadStrategy {
    priority_weight: f64,
}

impl AdaptivePreloadStrategy {
    pub fn new(priority_weight: f64) -> Self {
        Self { priority_weight }
    }
}

impl PreloadStrategy for AdaptivePreloadStrategy {
    async fn predict_preload_candidates(
        &self,
        current_level: LevelId,
        behavior_tracker: &UserBehaviorTracker,
        config: &PreloadConfig,
    ) -> Vec<PreloadCandidate> {
        let mut candidates = Vec::new();

        // Check if we have access patterns for the current level
        if let Some(access_pattern) = behavior_tracker.access_patterns.get(&current_level) {
            // Predict based on typical next levels
            for (next_level, probability) in &access_pattern.typical_next_levels {
                if *probability > 0.1 { // Only consider levels with >10% probability
                    let priority = self.priority_weight * probability;
                    candidates.push(PreloadCandidate {
                        level_id: *next_level,
                        priority_score: priority,
                        predicted_access_time: Some(access_pattern.avg_session_duration / 2),
                        reason: format!("Adaptive: {:.1}% probability based on history", probability * 100.0),
                    });
                }
            }
        }

        // Look at recent progression patterns
        if let Some(current_session) = &behavior_tracker.current_session {
            for (predicted_level, probability) in &current_session.predicted_next_levels {
                let priority = self.priority_weight * probability * 0.8; // Slightly lower than historical
                candidates.push(PreloadCandidate {
                    level_id: *predicted_level,
                    priority_score: priority,
                    predicted_access_time: Some(Duration::from_secs(60)), // Estimate 1 minute
                    reason: format!("Session pattern: {:.1}% probability", probability * 100.0),
                });
            }
        }

        // Limit to max preload distance
        candidates.retain(|c| {
            (c.level_id.0 as i16 - current_level.0 as i16).abs() <= config.max_preload_distance as i16
        });

        candidates
    }

    fn get_priority_weight(&self) -> f64 {
        self.priority_weight
    }

    fn name(&self) -> &'static str {
        "Adaptive"
    }
}

/// Difficulty-based preloading strategy
#[derive(Debug)]
pub struct DifficultyBasedPreloadStrategy {
    priority_weight: f64,
}

impl DifficultyBasedPreloadStrategy {
    pub fn new(priority_weight: f64) -> Self {
        Self { priority_weight }
    }

    /// Determine if a level is likely to be challenging based on number
    fn is_challenging_level(&self, level: LevelId) -> bool {
        // Levels ending in 0, 5 are often milestone/test levels
        level.0 % 5 == 0 || level.0 % 10 == 0
    }
}

impl PreloadStrategy for DifficultyBasedPreloadStrategy {
    async fn predict_preload_candidates(
        &self,
        current_level: LevelId,
        behavior_tracker: &UserBehaviorTracker,
        config: &PreloadConfig,
    ) -> Vec<PreloadCandidate> {
        let mut candidates = Vec::new();

        // Look for challenging levels nearby that users might retry
        for i in 1..=config.max_preload_distance {
            if let Ok(check_level) = LevelId::new(current_level.0 + i) {
                if self.is_challenging_level(check_level) {
                    let priority = self.priority_weight * 0.7; // Medium priority
                    candidates.push(PreloadCandidate {
                        level_id: check_level,
                        priority_score: priority,
                        predicted_access_time: Some(Duration::from_secs(i as u64 * 45)),
                        reason: format!("Difficulty milestone level {}", check_level.0),
                    });
                }
            }

            // Also check previous levels for retry patterns
            if current_level.0 >= i {
                if let Ok(check_level) = LevelId::new(current_level.0 - i) {
                    if let Some(error_pattern) = behavior_tracker.error_recovery_patterns.get(&current_level) {
                        if error_pattern.contains(&check_level) {
                            let priority = self.priority_weight * 0.5; // Lower priority for retries
                            candidates.push(PreloadCandidate {
                                level_id: check_level,
                                priority_score: priority,
                                predicted_access_time: Some(Duration::from_secs(120)), // Users may go back
                                reason: format!("Error recovery pattern for level {}", check_level.0),
                            });
                        }
                    }
                }
            }
        }

        candidates
    }

    fn get_priority_weight(&self) -> f64 {
        self.priority_weight
    }

    fn name(&self) -> &'static str {
        "DifficultyBased"
    }
}

/// Background cache warming for popular content
pub struct BackgroundPreloader {
    content_manager: Arc<ContentManager>,
    semaphore: Arc<Semaphore>,
    config: PreloadConfig,
    metrics: Arc<RwLock<PreloadMetrics>>,
    is_running: Arc<RwLock<bool>>,
}

impl BackgroundPreloader {
    pub fn new(
        content_manager: Arc<ContentManager>,
        config: PreloadConfig,
        metrics: Arc<RwLock<PreloadMetrics>>,
    ) -> Self {
        let semaphore = Arc::new(Semaphore::new(config.max_concurrent_preloads));

        Self {
            content_manager,
            semaphore,
            config,
            metrics,
            is_running: Arc::new(RwLock::new(false)),
        }
    }

    /// Start background cache warming task
    #[instrument(skip(self))]
    pub async fn start_background_warming(&self, popular_levels: Vec<LevelId>) {
        {
            let mut running = self.is_running.write().await;
            if *running {
                debug!("Background warming already running");
                return;
            }
            *running = true;
        }

        info!("Starting background cache warming for {} popular levels", popular_levels.len());

        let content_manager = self.content_manager.clone();
        let semaphore = self.semaphore.clone();
        let metrics = self.metrics.clone();
        let config = self.config.clone();
        let is_running = self.is_running.clone();

        tokio::spawn(async move {
            let mut warming_interval = interval(config.background_warming_interval);

            loop {
                // Check if we should continue running
                {
                    let running = is_running.read().await;
                    if !*running {
                        debug!("Background warming stopped");
                        break;
                    }
                }

                warming_interval.tick().await;

                // Warm cache for popular levels
                let mut warming_tasks = Vec::new();

                for level_id in &popular_levels {
                    let permit = match semaphore.clone().acquire_owned().await {
                        Ok(permit) => permit,
                        Err(_) => {
                            warn!("Failed to acquire semaphore for background warming");
                            continue;
                        }
                    };

                    let content_manager = content_manager.clone();
                    let metrics = metrics.clone();
                    let level_id = *level_id;

                    let task = tokio::spawn(async move {
                        let _permit = permit; // Hold permit until task completes

                        let start_time = Instant::now();

                        // Check if already cached
                        if content_manager.get_cached_content(level_id, None).await.is_some() {
                            return; // Already cached, skip
                        }

                        // Preload content
                        match content_manager.get_level_content(level_id, None).await {
                            Ok(_) => {
                                let preload_time = start_time.elapsed();
                                {
                                    let mut m = metrics.write().await;
                                    m.successful_preloads += 1;
                                    m.total_preloads_triggered += 1;
                                    m.background_warming_cycles += 1;

                                    // Update average preload latency
                                    if m.avg_preload_latency == Duration::ZERO {
                                        m.avg_preload_latency = preload_time;
                                    } else {
                                        m.avg_preload_latency = (m.avg_preload_latency + preload_time) / 2;
                                    }
                                }

                                debug!("Background warmed cache for level {} in {}ms",
                                      level_id.0, preload_time.as_millis());
                            },
                            Err(e) => {
                                {
                                    let mut m = metrics.write().await;
                                    m.failed_preloads += 1;
                                    m.total_preloads_triggered += 1;
                                }
                                warn!("Background warming failed for level {}: {}", level_id.0, e);
                            }
                        }
                    });

                    warming_tasks.push(task);
                }

                // Wait for all warming tasks to complete
                for task in warming_tasks {
                    let _ = task.await; // Ignore join errors
                }

                debug!("Background warming cycle completed");
            }
        });
    }

    /// Stop background cache warming
    pub async fn stop_background_warming(&self) {
        let mut running = self.is_running.write().await;
        *running = false;
        info!("Background cache warming stopped");
    }
}

impl AdaptivePreloadManager {
    /// Create new adaptive preload manager
    pub async fn new(content_manager: Arc<ContentManager>, config: PreloadConfig) -> Self {
        let user_behavior_tracker = Arc::new(RwLock::new(UserBehaviorTracker::default()));
        let metrics = Arc::new(RwLock::new(PreloadMetrics::default()));

        // Initialize preloading strategies with different weights
        let preload_strategies: Vec<Box<dyn PreloadStrategy + Send + Sync>> = vec![
            Box::new(SequentialPreloadStrategy::new(3, 1.0)),
            Box::new(AdaptivePreloadStrategy::new(0.8)),
            Box::new(DifficultyBasedPreloadStrategy::new(0.6)),
        ];

        let background_preloader = Arc::new(BackgroundPreloader::new(
            content_manager.clone(),
            config.clone(),
            metrics.clone(),
        ));

        Self {
            content_manager,
            user_behavior_tracker,
            preload_strategies,
            background_preloader,
            config,
            metrics,
        }
    }

    /// Execute intelligent preloading for a given level
    #[instrument(skip(self))]
    pub async fn execute_intelligent_preload(&self, current_level: LevelId) -> Result<PreloadResult> {
        debug!("Executing intelligent preload for level {}", current_level.0);

        let behavior_tracker = self.user_behavior_tracker.read().await;
        let mut all_candidates = Vec::new();

        // Gather candidates from all strategies
        for strategy in &self.preload_strategies {
            let candidates = strategy.predict_preload_candidates(
                current_level,
                &behavior_tracker,
                &self.config,
            ).await;

            debug!("Strategy '{}' generated {} candidates", strategy.name(), candidates.len());
            all_candidates.extend(candidates);
        }

        drop(behavior_tracker); // Release lock early

        // Sort candidates by priority score
        all_candidates.sort_by(|a, b| b.priority_score.partial_cmp(&a.priority_score).unwrap());

        // Take top candidates up to max concurrent limit
        let selected_candidates: Vec<_> = all_candidates
            .into_iter()
            .take(self.config.max_concurrent_preloads)
            .collect();

        debug!("Selected {} candidates for preloading", selected_candidates.len());

        // Execute preloading for selected candidates
        let mut preload_tasks = Vec::new();
        let mut preload_results = Vec::new();

        for candidate in selected_candidates {
            let content_manager = self.content_manager.clone();
            let metrics = self.metrics.clone();

            let task = tokio::spawn(async move {
                let start_time = Instant::now();

                match content_manager.get_level_content(candidate.level_id, None).await {
                    Ok(_) => {
                        let preload_time = start_time.elapsed();
                        {
                            let mut m = metrics.write().await;
                            m.successful_preloads += 1;
                            m.total_preloads_triggered += 1;
                        }

                        debug!("Successfully preloaded level {} ({}ms): {}",
                              candidate.level_id.0, preload_time.as_millis(), candidate.reason);

                        PreloadTaskResult::Success {
                            level_id: candidate.level_id,
                            preload_time,
                            reason: candidate.reason,
                        }
                    },
                    Err(e) => {
                        {
                            let mut m = metrics.write().await;
                            m.failed_preloads += 1;
                            m.total_preloads_triggered += 1;
                        }

                        warn!("Failed to preload level {}: {}", candidate.level_id.0, e);

                        PreloadTaskResult::Failure {
                            level_id: candidate.level_id,
                            error: e.to_string(),
                            reason: candidate.reason,
                        }
                    }
                }
            });

            preload_tasks.push(task);
        }

        // Collect all results
        for task in preload_tasks {
            match task.await {
                Ok(result) => preload_results.push(result),
                Err(e) => warn!("Preload task join error: {}", e),
            }
        }

        let successful_preloads = preload_results.iter()
            .filter(|r| matches!(r, PreloadTaskResult::Success { .. }))
            .count();

        let total_preloads = preload_results.len();

        info!("Preload execution completed: {}/{} successful", successful_preloads, total_preloads);

        Ok(PreloadResult {
            total_attempted: total_preloads,
            successful: successful_preloads,
            failed: total_preloads - successful_preloads,
            task_results: preload_results,
        })
    }

    /// Record user action to improve future predictions
    pub async fn record_user_action(&self, action: UserAction) {
        let mut behavior_tracker = self.user_behavior_tracker.write().await;

        match action {
            UserAction::LevelAccess { level_id, session_id, cache_hit } => {
                // Update access patterns
                let access_pattern = behavior_tracker.access_patterns
                    .entry(level_id)
                    .or_insert_with(|| AccessPattern {
                        total_accesses: 0,
                        cache_hits: 0,
                        cache_misses: 0,
                        avg_session_duration: Duration::from_secs(60),
                        typical_next_levels: Vec::new(),
                        last_accessed: Instant::now(),
                    });

                access_pattern.total_accesses += 1;
                access_pattern.last_accessed = Instant::now();

                if cache_hit {
                    access_pattern.cache_hits += 1;
                } else {
                    access_pattern.cache_misses += 1;
                }

                // Update current session
                if let Some(ref mut session) = behavior_tracker.current_session {
                    if session.session_id == session_id {
                        session.levels_this_session.push(level_id);
                        session.current_level = level_id;
                        session.last_action_time = Instant::now();
                    }
                } else {
                    behavior_tracker.current_session = Some(SessionPattern {
                        session_id,
                        start_time: Instant::now(),
                        current_level: level_id,
                        levels_this_session: vec![level_id],
                        predicted_next_levels: Vec::new(),
                        last_action_time: Instant::now(),
                    });
                }
            },

            UserAction::SessionComplete { session_id, levels_attempted, success_rate } => {
                // Record progression pattern
                if let Some(session) = behavior_tracker.current_session.take() {
                    if session.session_id == session_id {
                        let pattern = ProgressionPattern {
                            session_id,
                            levels_attempted,
                            total_duration: session.last_action_time.duration_since(session.start_time),
                            success_rate,
                            retry_patterns: HashMap::new(),
                            timestamp: Instant::now(),
                        };

                        behavior_tracker.progression_patterns.push_back(pattern);

                        // Keep only recent patterns
                        while behavior_tracker.progression_patterns.len() > self.config.behavior_learning_window {
                            behavior_tracker.progression_patterns.pop_front();
                        }
                    }
                }
            },

            UserAction::LevelRetry { level_id, from_level } => {
                // Record error recovery pattern
                behavior_tracker.error_recovery_patterns
                    .entry(from_level)
                    .or_insert_with(Vec::new)
                    .push(level_id);
            },
        }

        debug!("Recorded user action for behavioral learning");
    }

    /// Start background cache warming for popular levels
    pub async fn start_background_warming(&self) -> Result<()> {
        // Determine popular levels based on access patterns
        let behavior_tracker = self.user_behavior_tracker.read().await;
        let mut popular_levels: Vec<_> = behavior_tracker.access_patterns
            .iter()
            .filter(|(_, pattern)| pattern.total_accesses >= 5) // Levels accessed 5+ times
            .map(|(level_id, pattern)| (*level_id, pattern.total_accesses))
            .collect();

        popular_levels.sort_by(|a, b| b.1.cmp(&a.1)); // Sort by access count
        let popular_levels: Vec<_> = popular_levels.into_iter()
            .take(20) // Top 20 popular levels
            .map(|(level_id, _)| level_id)
            .collect();

        drop(behavior_tracker);

        if popular_levels.is_empty() {
            // Default popular levels if no data available
            let default_popular: Vec<_> = (1..=10)
                .filter_map(|i| LevelId::new(i).ok())
                .collect();
            self.background_preloader.start_background_warming(default_popular).await;
        } else {
            self.background_preloader.start_background_warming(popular_levels).await;
        }

        info!("Background cache warming started");
        Ok(())
    }

    /// Stop background cache warming
    pub async fn stop_background_warming(&self) {
        self.background_preloader.stop_background_warming().await;
    }

    /// Get current preloading metrics
    pub async fn get_metrics(&self) -> PreloadMetrics {
        self.metrics.read().await.clone()
    }

    /// Get user behavior insights
    pub async fn get_behavior_insights(&self) -> BehaviorInsights {
        let behavior_tracker = self.user_behavior_tracker.read().await;

        BehaviorInsights {
            total_levels_accessed: behavior_tracker.access_patterns.len(),
            most_popular_level: behavior_tracker.access_patterns
                .iter()
                .max_by_key(|(_, pattern)| pattern.total_accesses)
                .map(|(level_id, _)| *level_id),
            avg_session_length: if behavior_tracker.progression_patterns.is_empty() {
                Duration::ZERO
            } else {
                behavior_tracker.progression_patterns
                    .iter()
                    .map(|p| p.total_duration)
                    .sum::<Duration>() / behavior_tracker.progression_patterns.len() as u32
            },
            cache_hit_rate: {
                let total_hits: u64 = behavior_tracker.access_patterns.values().map(|p| p.cache_hits).sum();
                let total_accesses: u64 = behavior_tracker.access_patterns.values().map(|p| p.total_accesses).sum();
                if total_accesses > 0 {
                    (total_hits as f64 / total_accesses as f64) * 100.0
                } else {
                    0.0
                }
            },
            learning_window_utilization: (behavior_tracker.progression_patterns.len() as f64
                / self.config.behavior_learning_window as f64) * 100.0,
        }
    }
}

/// User action for behavioral learning
#[derive(Debug, Clone)]
pub enum UserAction {
    LevelAccess {
        level_id: LevelId,
        session_id: uuid::Uuid,
        cache_hit: bool,
    },
    SessionComplete {
        session_id: uuid::Uuid,
        levels_attempted: Vec<LevelId>,
        success_rate: f64,
    },
    LevelRetry {
        level_id: LevelId,
        from_level: LevelId,
    },
}

/// Result of a preload execution
#[derive(Debug, Clone)]
pub struct PreloadResult {
    pub total_attempted: usize,
    pub successful: usize,
    pub failed: usize,
    pub task_results: Vec<PreloadTaskResult>,
}

/// Individual preload task result
#[derive(Debug, Clone)]
pub enum PreloadTaskResult {
    Success {
        level_id: LevelId,
        preload_time: Duration,
        reason: String,
    },
    Failure {
        level_id: LevelId,
        error: String,
        reason: String,
    },
}

/// User behavior insights for monitoring
#[derive(Debug, Clone)]
pub struct BehaviorInsights {
    pub total_levels_accessed: usize,
    pub most_popular_level: Option<LevelId>,
    pub avg_session_length: Duration,
    pub cache_hit_rate: f64,
    pub learning_window_utilization: f64,
}

#[cfg(test)]
mod tests {
    use super::*;
    use centotype_content::ContentManager;

    #[tokio::test]
    async fn test_sequential_preload_strategy() {
        let strategy = SequentialPreloadStrategy::new(3, 1.0);
        let behavior_tracker = UserBehaviorTracker::default();
        let config = PreloadConfig::default();
        let current_level = LevelId::new(5).unwrap();

        let candidates = strategy.predict_preload_candidates(current_level, &behavior_tracker, &config).await;

        assert_eq!(candidates.len(), 3);
        assert_eq!(candidates[0].level_id, LevelId::new(6).unwrap());
        assert!(candidates[0].priority_score > candidates[1].priority_score);
    }

    #[tokio::test]
    async fn test_adaptive_preload_manager_creation() {
        let content_manager = Arc::new(ContentManager::new().await.unwrap());
        let config = PreloadConfig::default();

        let manager = AdaptivePreloadManager::new(content_manager, config).await;

        let metrics = manager.get_metrics().await;
        assert_eq!(metrics.total_preloads_triggered, 0);
    }

    #[tokio::test]
    async fn test_user_behavior_recording() {
        let content_manager = Arc::new(ContentManager::new().await.unwrap());
        let config = PreloadConfig::default();
        let manager = AdaptivePreloadManager::new(content_manager, config).await;

        let session_id = uuid::Uuid::new_v4();
        let level_id = LevelId::new(1).unwrap();

        manager.record_user_action(UserAction::LevelAccess {
            level_id,
            session_id,
            cache_hit: true,
        }).await;

        let insights = manager.get_behavior_insights().await;
        assert_eq!(insights.total_levels_accessed, 1);
    }

    #[tokio::test]
    async fn test_preload_execution() {
        let content_manager = Arc::new(ContentManager::new().await.unwrap());
        let config = PreloadConfig {
            max_concurrent_preloads: 2,
            ..PreloadConfig::default()
        };
        let manager = AdaptivePreloadManager::new(content_manager, config).await;

        let current_level = LevelId::new(1).unwrap();
        let result = manager.execute_intelligent_preload(current_level).await.unwrap();

        assert!(result.total_attempted > 0);
    }
}