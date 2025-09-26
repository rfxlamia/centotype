//! Scoring engine with deterministic calculations for WPM, accuracy, and skill index
use crate::types::*;
use chrono::{DateTime, Utc};
use std::collections::VecDeque;
use std::time::Duration;
use tracing::debug;
use unicode_segmentation::UnicodeSegmentation;

/// Real-time and final scoring calculations with deterministic results
pub struct Scoring {
    performance_tracker: ScoringPerformanceTracker,
}

impl Scoring {
    pub fn new() -> Self {
        Self {
            performance_tracker: ScoringPerformanceTracker::new(),
        }
    }

    /// Calculate live metrics during an active session
    pub fn calculate_live_metrics(
        &mut self,
        session: &SessionState,
    ) -> Result<LiveMetrics> {
        let start_time = std::time::Instant::now();

        // Calculate session duration
        let elapsed_seconds = if session.is_paused {
            // When paused, use duration up to last keystroke
            self.calculate_active_duration(session)
        } else {
            // Active session - calculate current elapsed time
            let total_elapsed = (Utc::now() - session.started_at).to_std()
                .map_err(|_| CentotypeError::State("Invalid session timing".to_string()))?;
            total_elapsed.saturating_sub(session.paused_duration).as_secs_f64()
        };

        // Prevent division by zero
        let elapsed_seconds = elapsed_seconds.max(0.001); // Minimum 1ms

        // Calculate basic metrics
        let raw_wpm = self.calculate_wpm(session.typed_text.len(), Duration::from_secs_f64(elapsed_seconds));
        let accuracy = self.calculate_accuracy(&session.target_text, &session.typed_text);

        // Calculate streaks and errors
        let (current_streak, longest_streak, errors) = self.analyze_typing_patterns(session);

        // Calculate effective WPM (accounts for errors)
        let effective_wpm = raw_wpm * (accuracy / 100.0);

        let metrics = LiveMetrics {
            raw_wpm,
            effective_wpm,
            accuracy,
            current_streak,
            longest_streak,
            errors,
            elapsed_seconds,
        };

        // Track performance
        let calculation_time = start_time.elapsed();
        self.performance_tracker.record_calculation(calculation_time);

        debug!(
            raw_wpm = %raw_wpm,
            effective_wpm = %effective_wpm,
            accuracy = %accuracy,
            "Calculated live metrics"
        );

        Ok(metrics)
    }

    /// Calculate final metrics for a completed session
    pub fn calculate_final_metrics(
        &mut self,
        session: &SessionState,
    ) -> Result<FinalMetrics> {
        if !session.is_completed {
            return Err(CentotypeError::State("Session not completed".to_string()));
        }

        let start_time = std::time::Instant::now();

        // Calculate session duration
        let total_duration = (session.started_at + chrono::Duration::seconds(1)) - session.started_at;
        let active_duration = total_duration.to_std()
            .map_err(|_| CentotypeError::State("Invalid session duration".to_string()))?
            .saturating_sub(session.paused_duration);

        let duration_seconds = active_duration.as_secs_f64();

        // Basic metrics
        let raw_wpm = self.calculate_wpm(session.typed_text.len(), active_duration);
        let accuracy = self.calculate_accuracy(&session.target_text, &session.typed_text);
        let effective_wpm = raw_wpm * (accuracy / 100.0);

        // Advanced metrics
        let consistency = self.calculate_consistency(&session.keystrokes);
        let (_, longest_streak, errors) = self.analyze_typing_patterns(session);
        let latency_p99 = self.calculate_latency_p99(&session.keystrokes);

        let metrics = FinalMetrics {
            raw_wpm,
            effective_wpm,
            accuracy,
            consistency,
            longest_streak,
            errors,
            latency_p99,
        };

        // Track performance
        let calculation_time = start_time.elapsed();
        self.performance_tracker.record_calculation(calculation_time);

        debug!(
            duration_seconds = %duration_seconds,
            effective_wpm = %effective_wpm,
            accuracy = %accuracy,
            consistency = %consistency,
            "Calculated final metrics"
        );

        Ok(metrics)
    }

    /// Calculate skill index using the Centotype algorithm
    pub fn calculate_skill_index(&self, metrics: &FinalMetrics, tier: Tier) -> f64 {
        let tier_weight = tier.weight();

        // Base score from effective WPM (0-600 points)
        let wpm_score = (metrics.effective_wpm * 4.0).min(600.0);

        // Accuracy bonus/penalty (0-200 points)
        let accuracy_bonus = if metrics.accuracy >= 95.0 {
            (metrics.accuracy - 95.0) * 40.0 // Up to 200 points for 100% accuracy
        } else {
            // Significant penalty for < 95% accuracy
            (metrics.accuracy - 95.0) * 8.0 // Down to -200 points for 70% accuracy
        };

        // Consistency bonus (0-100 points)
        let consistency_bonus = metrics.consistency;

        // Error severity penalty
        let error_penalty = metrics.errors.severity_score() * 5.0;

        // Streak bonus (0-100 points)
        let streak_bonus = (metrics.longest_streak as f64 / 10.0).min(100.0);

        // Base skill index before tier adjustment
        let base_skill_index = wpm_score + accuracy_bonus + consistency_bonus + streak_bonus - error_penalty;

        // Apply tier weight (higher tiers are harder)
        let final_skill_index = base_skill_index * tier_weight;

        // Ensure non-negative result
        final_skill_index.max(0.0)
    }

    /// Get performance metrics for the scoring engine itself
    pub fn get_performance_metrics(&self) -> ScoringMetrics {
        self.performance_tracker.get_metrics()
    }

    // Private helper methods

    fn calculate_active_duration(&self, session: &SessionState) -> f64 {
        if session.keystrokes.is_empty() {
            return 0.001; // Minimum duration
        }

        let first_keystroke = session.keystrokes.first().unwrap().timestamp;
        let last_keystroke = session.keystrokes.last().unwrap().timestamp;

        (last_keystroke - first_keystroke)
            .to_std()
            .unwrap_or_default()
            .saturating_sub(session.paused_duration)
            .as_secs_f64()
            .max(0.001)
    }

    fn analyze_typing_patterns(&self, session: &SessionState) -> (u32, u32, ErrorStats) {
        let mut current_streak = 0u32;
        let mut longest_streak = 0u32;
        let mut errors = ErrorStats::default();

        // Analyze character by character
        let target_chars: Vec<char> = session.target_text.chars().collect();
        let typed_chars: Vec<char> = session.typed_text.chars().collect();

        let mut streak = 0u32;

        for (i, &target_char) in target_chars.iter().enumerate() {
            if let Some(&typed_char) = typed_chars.get(i) {
                if target_char == typed_char {
                    streak += 1;
                    longest_streak = longest_streak.max(streak);
                } else {
                    streak = 0;
                    errors.substitution += 1;
                }
            } else {
                // Character not yet typed or deleted
                streak = 0;
                if i < typed_chars.len() {
                    errors.deletion += 1;
                }
                break;
            }
        }

        // Handle extra characters (insertions)
        if typed_chars.len() > target_chars.len() {
            errors.insertion += (typed_chars.len() - target_chars.len()) as u32;
            streak = 0;
        }

        current_streak = streak;

        // Count backspace events from keystrokes
        errors.backspace_count = session.keystrokes
            .iter()
            .filter(|k| k.char_typed.is_none())
            .count() as u32;

        (current_streak, longest_streak, errors)
    }

    fn calculate_consistency(&self, keystrokes: &[Keystroke]) -> f64 {
        if keystrokes.len() < 10 {
            return 0.0; // Not enough data for consistency calculation
        }

        // Calculate inter-keystroke intervals
        let mut intervals = Vec::new();
        for i in 1..keystrokes.len() {
            let prev_ts = keystrokes[i-1].timestamp.timestamp_millis();
            let curr_ts = keystrokes[i].timestamp.timestamp_millis();
            let interval = (curr_ts - prev_ts) as f64;
            if interval > 0.0 && interval < 2000.0 { // Ignore pauses > 2 seconds
                intervals.push(interval);
            }
        }

        if intervals.is_empty() {
            return 0.0;
        }

        // Calculate coefficient of variation (lower is more consistent)
        let mean = intervals.iter().sum::<f64>() / intervals.len() as f64;
        let variance = intervals.iter()
            .map(|x| (x - mean).powi(2))
            .sum::<f64>() / intervals.len() as f64;
        let std_dev = variance.sqrt();

        if mean == 0.0 {
            return 0.0;
        }

        let coefficient_of_variation = std_dev / mean;

        // Convert to consistency score (0-100, higher is better)
        let consistency: f64 = (1.0 - coefficient_of_variation.min(1.0)) * 100.0;
        consistency.max(0.0).min(100.0)
    }

    fn calculate_latency_p99(&self, keystrokes: &[Keystroke]) -> Duration {
        if keystrokes.len() < 2 {
            return Duration::default();
        }

        // Calculate inter-keystroke intervals
        let mut intervals = Vec::new();
        for i in 1..keystrokes.len() {
            let prev_ts = keystrokes[i-1].timestamp.timestamp_millis();
            let curr_ts = keystrokes[i].timestamp.timestamp_millis();
            let interval_ms = (curr_ts - prev_ts) as u64;
            if interval_ms > 0 && interval_ms < 2000 { // Filter out pauses
                intervals.push(Duration::from_millis(interval_ms));
            }
        }

        if intervals.is_empty() {
            return Duration::default();
        }

        intervals.sort();
        let p99_index = ((intervals.len() as f64 - 1.0) * 0.99) as usize;
        intervals[p99_index.min(intervals.len() - 1)]
    }
}

impl Default for Scoring {
    fn default() -> Self {
        Self::new()
    }
}

impl ScoringEngine for Scoring {
    fn calculate_wpm(&self, chars_typed: usize, duration: Duration) -> f64 {
        let minutes = duration.as_secs_f64() / 60.0;
        if minutes <= 0.0 {
            return 0.0;
        }

        // Standard WPM calculation: (characters / 5) / minutes
        // Using characters instead of words for more accurate typing trainer metrics
        (chars_typed as f64 / 5.0) / minutes
    }

    fn calculate_accuracy(&self, target: &str, typed: &str) -> f64 {
        if target.is_empty() {
            return 100.0;
        }

        // Use grapheme clusters for accurate Unicode handling
        let target_graphemes: Vec<&str> = target.graphemes(true).collect();
        let typed_graphemes: Vec<&str> = typed.graphemes(true).collect();

        let mut correct = 0;
        let total = target_graphemes.len();

        // Count correct characters
        for (i, &target_grapheme) in target_graphemes.iter().enumerate() {
            if let Some(&typed_grapheme) = typed_graphemes.get(i) {
                if target_grapheme == typed_grapheme {
                    correct += 1;
                }
            }
        }

        // Calculate accuracy as percentage
        if total == 0 {
            100.0
        } else {
            (correct as f64 / total as f64) * 100.0
        }
    }

    fn calculate_skill_index(&self, metrics: &FinalMetrics, tier: Tier) -> f64 {
        self.calculate_skill_index(metrics, tier)
    }

    fn classify_errors(&self, target: &str, typed: &str) -> ErrorStats {
        // This is a simplified error classification
        // In practice, you might want to use a more sophisticated algorithm
        let mut errors = ErrorStats::default();

        let target_chars: Vec<char> = target.chars().collect();
        let typed_chars: Vec<char> = typed.chars().collect();

        // Simple character-by-character comparison
        let max_len = target_chars.len().max(typed_chars.len());

        for i in 0..max_len {
            match (target_chars.get(i), typed_chars.get(i)) {
                (Some(&target_char), Some(&typed_char)) => {
                    if target_char != typed_char {
                        errors.substitution += 1;
                    }
                }
                (Some(_), None) => {
                    errors.deletion += 1;
                }
                (None, Some(_)) => {
                    errors.insertion += 1;
                }
                (None, None) => break,
            }
        }

        errors
    }
}

/// Performance tracking for scoring calculations
#[derive(Debug, Clone)]
struct ScoringPerformanceTracker {
    calculation_times: VecDeque<Duration>,
    total_calculations: u64,
}

impl ScoringPerformanceTracker {
    fn new() -> Self {
        Self {
            calculation_times: VecDeque::new(),
            total_calculations: 0,
        }
    }

    fn record_calculation(&mut self, duration: Duration) {
        self.calculation_times.push_back(duration);
        self.total_calculations += 1;

        // Keep only recent measurements (last 1000)
        if self.calculation_times.len() > 1000 {
            self.calculation_times.pop_front();
        }
    }

    fn get_metrics(&self) -> ScoringMetrics {
        if self.calculation_times.is_empty() {
            return ScoringMetrics {
                avg_calculation_time: Duration::default(),
                p95_calculation_time: Duration::default(),
                p99_calculation_time: Duration::default(),
                total_calculations: self.total_calculations,
            };
        }

        let mut sorted_times: Vec<Duration> = self.calculation_times.iter().cloned().collect();
        sorted_times.sort();

        let avg_time = sorted_times.iter().sum::<Duration>() / sorted_times.len() as u32;
        let p95_index = ((sorted_times.len() as f64 - 1.0) * 0.95) as usize;
        let p99_index = ((sorted_times.len() as f64 - 1.0) * 0.99) as usize;

        ScoringMetrics {
            avg_calculation_time: avg_time,
            p95_calculation_time: sorted_times[p95_index.min(sorted_times.len() - 1)],
            p99_calculation_time: sorted_times[p99_index.min(sorted_times.len() - 1)],
            total_calculations: self.total_calculations,
        }
    }
}

/// Metrics for scoring engine performance
#[derive(Debug, Clone)]
pub struct ScoringMetrics {
    pub avg_calculation_time: Duration,
    pub p95_calculation_time: Duration,
    pub p99_calculation_time: Duration,
    pub total_calculations: u64,
}

impl ScoringMetrics {
    /// Check if performance targets are met (P95 < 2ms)
    pub fn meets_targets(&self) -> bool {
        self.p95_calculation_time <= Duration::from_millis(2)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_wpm_calculation() {
        let scoring = Scoring::new();

        // Test basic WPM calculation
        let chars_typed = 100; // 20 words at 5 chars per word
        let duration = Duration::from_secs(60); // 1 minute
        let wpm = scoring.calculate_wpm(chars_typed, duration);

        assert_eq!(wpm, 20.0);
    }

    #[test]
    fn test_accuracy_calculation() {
        let scoring = Scoring::new();

        // Perfect accuracy
        assert_eq!(scoring.calculate_accuracy("hello", "hello"), 100.0);

        // 80% accuracy (4 out of 5 correct)
        assert_eq!(scoring.calculate_accuracy("hello", "hallo"), 80.0);

        // Empty strings
        assert_eq!(scoring.calculate_accuracy("", ""), 100.0);
        assert_eq!(scoring.calculate_accuracy("test", ""), 0.0);
    }

    #[test]
    fn test_skill_index_calculation() {
        let scoring = Scoring::new();
        let tier = Tier(1); // Tier 1 (easiest)

        let metrics = FinalMetrics {
            raw_wpm: 50.0,
            effective_wpm: 45.0, // 90% accuracy
            accuracy: 90.0,
            consistency: 80.0,
            longest_streak: 100,
            errors: ErrorStats::default(),
            latency_p99: Duration::from_millis(20),
        };

        let skill_index = scoring.calculate_skill_index(&metrics, tier);
        assert!(skill_index > 0.0);

        // Higher tier should have higher skill index for same performance
        let tier_10 = Tier(10);
        let high_tier_skill_index = scoring.calculate_skill_index(&metrics, tier_10);
        assert!(high_tier_skill_index > skill_index);
    }

    #[test]
    fn test_error_classification() {
        let scoring = Scoring::new();

        // Test substitution errors
        let errors = scoring.classify_errors("hello", "hallo");
        assert_eq!(errors.substitution, 1);
        assert_eq!(errors.insertion, 0);
        assert_eq!(errors.deletion, 0);

        // Test insertion error
        let errors = scoring.classify_errors("hello", "helloo");
        assert_eq!(errors.insertion, 1);

        // Test deletion error
        let errors = scoring.classify_errors("hello", "hell");
        assert_eq!(errors.deletion, 1);
    }

    #[test]
    fn test_live_metrics_calculation() {
        let mut scoring = Scoring::new();

        let session = SessionState {
            session_id: uuid::Uuid::new_v4(),
            mode: TrainingMode::Arcade { level: LevelId::new(1).unwrap() },
            target_text: "hello world".to_string(),
            typed_text: "hello w".to_string(),
            cursor_position: 7,
            started_at: Utc::now() - chrono::Duration::seconds(60),
            paused_duration: Duration::default(),
            is_paused: false,
            is_completed: false,
            keystrokes: vec![
                Keystroke {
                    timestamp: Utc::now() - chrono::Duration::seconds(55),
                    char_typed: Some('h'),
                    is_correction: false,
                    cursor_pos: 0,
                },
                Keystroke {
                    timestamp: Utc::now() - chrono::Duration::seconds(50),
                    char_typed: Some('e'),
                    is_correction: false,
                    cursor_pos: 1,
                },
            ],
        };

        let metrics = scoring.calculate_live_metrics(&session).unwrap();
        assert!(metrics.raw_wpm > 0.0);
        assert!(metrics.accuracy > 0.0);
        assert!(metrics.elapsed_seconds > 0.0);
    }
}
