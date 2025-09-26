//! Error detection and classification using Damerau-Levenshtein algorithm
use crate::types::*;
use std::cmp::min;
use std::collections::HashMap;
use tracing::debug;
use unicode_segmentation::UnicodeSegmentation;

/// Error classifier implementing Damerau-Levenshtein distance algorithm
pub struct Error {
    cache: HashMap<(String, String), ErrorAnalysis>,
    cache_max_size: usize,
}

impl Error {
    pub fn new() -> Self {
        Self {
            cache: HashMap::new(),
            cache_max_size: 1000, // Limit cache size for memory efficiency
        }
    }

    /// Classify errors between target and typed text using Damerau-Levenshtein
    pub fn classify_errors(&mut self, target: &str, typed: &str) -> ErrorStats {
        let start_time = std::time::Instant::now();

        // Check cache first
        let cache_key = (target.to_string(), typed.to_string());
        if let Some(cached) = self.cache.get(&cache_key) {
            return cached.stats.clone();
        }

        // Perform full error analysis
        let analysis = self.analyze_text_differences(target, typed);
        let stats = analysis.stats.clone();

        // Cache result if we have space
        if self.cache.len() < self.cache_max_size {
            self.cache.insert(cache_key, analysis);
        } else if self.cache.len() >= self.cache_max_size {
            // Clear cache when it gets too large
            self.cache.clear();
        }

        let elapsed = start_time.elapsed();
        debug!(
            duration_ms = %elapsed.as_millis(),
            substitutions = %stats.substitution,
            insertions = %stats.insertion,
            deletions = %stats.deletion,
            transpositions = %stats.transposition,
            "Classified errors using Damerau-Levenshtein"
        );

        stats
    }

    /// Get detailed error analysis with position information
    pub fn analyze_errors(&mut self, target: &str, typed: &str) -> ErrorAnalysis {
        let cache_key = (target.to_string(), typed.to_string());
        if let Some(cached) = self.cache.get(&cache_key) {
            return cached.clone();
        }

        let analysis = self.analyze_text_differences(target, typed);

        // Cache result
        if self.cache.len() < self.cache_max_size {
            self.cache.insert(cache_key, analysis.clone());
        }

        analysis
    }

    /// Get error patterns for learning insights
    pub fn get_error_patterns(&self, session_results: &[SessionResult]) -> ErrorPatterns {
        let mut character_errors = HashMap::new();
        let mut bigram_errors = HashMap::new();
        let mut position_errors = Vec::new();

        for result in session_results {
            // Accumulate character-level errors
            for ch in 'a'..='z' {
                character_errors.entry(ch).or_insert(0);
            }
            for ch in 'A'..='Z' {
                character_errors.entry(ch).or_insert(0);
            }

            // This would need access to detailed session data to implement fully
            // For now, we'll create a placeholder implementation
        }

        ErrorPatterns {
            character_errors,
            bigram_errors,
            position_errors,
            common_mistakes: self.identify_common_mistakes(session_results),
        }
    }

    /// Clear error analysis cache
    pub fn clear_cache(&mut self) {
        self.cache.clear();
    }

    // Private implementation methods

    fn analyze_text_differences(&self, target: &str, typed: &str) -> ErrorAnalysis {
        // Use grapheme clusters for proper Unicode support
        let target_graphemes: Vec<&str> = target.graphemes(true).collect();
        let typed_graphemes: Vec<&str> = typed.graphemes(true).collect();

        let (distance_matrix, operations) = self.compute_damerau_levenshtein_with_operations(
            &target_graphemes,
            &typed_graphemes,
        );

        let total_distance = distance_matrix[target_graphemes.len()][typed_graphemes.len()];

        let mut stats = ErrorStats::default();
        let mut error_positions = Vec::new();

        // Analyze operations to classify error types
        for operation in &operations {
            match operation {
                EditOperation::Substitution { position, expected, actual } => {
                    stats.substitution += 1;
                    error_positions.push(ErrorPosition {
                        position: *position,
                        error_type: ErrorType::Substitution,
                        expected_char: Some(*expected),
                        actual_char: Some(*actual),
                    });
                }
                EditOperation::Insertion { position, char_inserted } => {
                    stats.insertion += 1;
                    error_positions.push(ErrorPosition {
                        position: *position,
                        error_type: ErrorType::Insertion,
                        expected_char: None,
                        actual_char: Some(*char_inserted),
                    });
                }
                EditOperation::Deletion { position, char_deleted } => {
                    stats.deletion += 1;
                    error_positions.push(ErrorPosition {
                        position: *position,
                        error_type: ErrorType::Deletion,
                        expected_char: Some(*char_deleted),
                        actual_char: None,
                    });
                }
                EditOperation::Transposition { position, char1, char2 } => {
                    stats.transposition += 1;
                    error_positions.push(ErrorPosition {
                        position: *position,
                        error_type: ErrorType::Transposition,
                        expected_char: Some(*char1),
                        actual_char: Some(*char2),
                    });
                }
                EditOperation::Match { .. } => {
                    // No error for matches
                }
            }
        }

        ErrorAnalysis {
            stats,
            total_distance,
            error_positions,
            accuracy: if target_graphemes.is_empty() {
                100.0
            } else {
                let correct_chars = target_graphemes.len() - total_distance;
                (correct_chars as f64 / target_graphemes.len() as f64) * 100.0
            },
        }
    }

    fn compute_damerau_levenshtein_with_operations(
        &self,
        source: &[&str],
        target: &[&str],
    ) -> (Vec<Vec<usize>>, Vec<EditOperation>) {
        let source_len = source.len();
        let target_len = target.len();

        // Initialize the distance matrix
        let mut matrix = vec![vec![0; target_len + 1]; source_len + 1];

        // Initialize first row and column
        for i in 0..=source_len {
            matrix[i][0] = i;
        }
        for j in 0..=target_len {
            matrix[0][j] = j;
        }

        // Fill the matrix using Damerau-Levenshtein algorithm
        for i in 1..=source_len {
            for j in 1..=target_len {
                let cost = if source[i - 1] == target[j - 1] { 0 } else { 1 };

                matrix[i][j] = min(
                    min(
                        matrix[i - 1][j] + 1,     // deletion
                        matrix[i][j - 1] + 1,     // insertion
                    ),
                    matrix[i - 1][j - 1] + cost,  // substitution
                );

                // Transposition
                if i > 1 && j > 1
                    && source[i - 1] == target[j - 2]
                    && source[i - 2] == target[j - 1]
                {
                    matrix[i][j] = min(matrix[i][j], matrix[i - 2][j - 2] + cost);
                }
            }
        }

        // Trace back to find operations
        let operations = self.trace_back_operations(source, target, &matrix);

        (matrix, operations)
    }

    fn trace_back_operations(
        &self,
        source: &[&str],
        target: &[&str],
        matrix: &[Vec<usize>],
    ) -> Vec<EditOperation> {
        let mut operations = Vec::new();
        let mut i = source.len();
        let mut j = target.len();

        while i > 0 || j > 0 {
            if i > 0 && j > 0 && source[i - 1] == target[j - 1] {
                // Match
                operations.push(EditOperation::Match {
                    position: i - 1,
                    character: source[i - 1].chars().next().unwrap_or('\0'),
                });
                i -= 1;
                j -= 1;
            } else if i > 0 && j > 0
                && matrix[i][j] == matrix[i - 1][j - 1] + 1
            {
                // Substitution
                operations.push(EditOperation::Substitution {
                    position: i - 1,
                    expected: source[i - 1].chars().next().unwrap_or('\0'),
                    actual: target[j - 1].chars().next().unwrap_or('\0'),
                });
                i -= 1;
                j -= 1;
            } else if i > 1 && j > 1
                && source[i - 1] == target[j - 2]
                && source[i - 2] == target[j - 1]
                && matrix[i][j] == matrix[i - 2][j - 2] + 1
            {
                // Transposition
                operations.push(EditOperation::Transposition {
                    position: i - 2,
                    char1: source[i - 2].chars().next().unwrap_or('\0'),
                    char2: source[i - 1].chars().next().unwrap_or('\0'),
                });
                i -= 2;
                j -= 2;
            } else if i > 0 && matrix[i][j] == matrix[i - 1][j] + 1 {
                // Deletion
                operations.push(EditOperation::Deletion {
                    position: i - 1,
                    char_deleted: source[i - 1].chars().next().unwrap_or('\0'),
                });
                i -= 1;
            } else if j > 0 && matrix[i][j] == matrix[i][j - 1] + 1 {
                // Insertion
                operations.push(EditOperation::Insertion {
                    position: j - 1,
                    char_inserted: target[j - 1].chars().next().unwrap_or('\0'),
                });
                j -= 1;
            } else {
                // This shouldn't happen with a correct algorithm
                break;
            }
        }

        operations.reverse(); // Reverse to get operations in forward order
        operations
    }

    fn identify_common_mistakes(&self, session_results: &[SessionResult]) -> Vec<CommonMistake> {
        // This is a simplified implementation
        // In practice, you'd analyze patterns across many sessions
        let mut mistakes = Vec::new();

        // Common typing mistakes patterns
        mistakes.push(CommonMistake {
            pattern: "th".to_string(),
            mistake: "teh".to_string(),
            frequency: 5,
            suggested_practice: "Practice 'th' combinations slowly".to_string(),
        });

        mistakes.push(CommonMistake {
            pattern: "and".to_string(),
            mistake: "nad".to_string(),
            frequency: 3,
            suggested_practice: "Focus on 'a' before 'n' in common words".to_string(),
        });

        mistakes
    }
}

impl Default for Error {
    fn default() -> Self {
        Self::new()
    }
}

/// Complete error analysis result
#[derive(Debug, Clone)]
pub struct ErrorAnalysis {
    pub stats: ErrorStats,
    pub total_distance: usize,
    pub error_positions: Vec<ErrorPosition>,
    pub accuracy: f64,
}

/// Individual error position with context
#[derive(Debug, Clone)]
pub struct ErrorPosition {
    pub position: usize,
    pub error_type: ErrorType,
    pub expected_char: Option<char>,
    pub actual_char: Option<char>,
}

/// Type of typing error
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ErrorType {
    Substitution,
    Insertion,
    Deletion,
    Transposition,
}

/// Edit operations identified during analysis
#[derive(Debug, Clone)]
enum EditOperation {
    Match {
        position: usize,
        character: char,
    },
    Substitution {
        position: usize,
        expected: char,
        actual: char,
    },
    Insertion {
        position: usize,
        char_inserted: char,
    },
    Deletion {
        position: usize,
        char_deleted: char,
    },
    Transposition {
        position: usize,
        char1: char,
        char2: char,
    },
}

/// Error patterns for learning insights
#[derive(Debug, Clone)]
pub struct ErrorPatterns {
    pub character_errors: HashMap<char, u32>,
    pub bigram_errors: HashMap<String, u32>,
    pub position_errors: Vec<PositionError>,
    pub common_mistakes: Vec<CommonMistake>,
}

/// Error frequency by position in text
#[derive(Debug, Clone)]
pub struct PositionError {
    pub position_range: (usize, usize), // Start and end of range
    pub error_rate: f64,                // Error rate in this range
    pub common_errors: Vec<char>,       // Most common errors in this range
}

/// Common typing mistake pattern
#[derive(Debug, Clone)]
pub struct CommonMistake {
    pub pattern: String,           // Correct pattern
    pub mistake: String,           // Common mistake
    pub frequency: u32,            // How often this mistake occurs
    pub suggested_practice: String, // Practice recommendation
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_basic_error_classification() {
        let mut classifier = Error::new();

        // Test substitution error
        let stats = classifier.classify_errors("hello", "hallo");
        assert_eq!(stats.substitution, 1);
        assert_eq!(stats.insertion, 0);
        assert_eq!(stats.deletion, 0);
        assert_eq!(stats.transposition, 0);

        // Test insertion error
        let stats = classifier.classify_errors("hello", "helllo");
        assert_eq!(stats.insertion, 1);

        // Test deletion error
        let stats = classifier.classify_errors("hello", "hell");
        assert_eq!(stats.deletion, 1);
    }

    #[test]
    fn test_transposition_detection() {
        let mut classifier = Error::new();

        // Test simple transposition
        let stats = classifier.classify_errors("hello", "ehllo");
        assert_eq!(stats.transposition, 1);

        // Test word with transposition
        let stats = classifier.classify_errors("the", "teh");
        assert_eq!(stats.transposition, 1);
    }

    #[test]
    fn test_complex_errors() {
        let mut classifier = Error::new();

        // Test multiple error types
        let stats = classifier.classify_errors("programming", "progamming"); // deletion of 'r'
        assert_eq!(stats.deletion, 1);

        let stats = classifier.classify_errors("function", "funtion"); // deletion of 'c'
        assert_eq!(stats.deletion, 1);
    }

    #[test]
    fn test_detailed_error_analysis() {
        let mut classifier = Error::new();

        let analysis = classifier.analyze_errors("hello", "hallo");
        assert_eq!(analysis.error_positions.len(), 1);
        assert_eq!(analysis.error_positions[0].error_type, ErrorType::Substitution);
        assert_eq!(analysis.error_positions[0].expected_char, Some('e'));
        assert_eq!(analysis.error_positions[0].actual_char, Some('a'));
    }

    #[test]
    fn test_empty_strings() {
        let mut classifier = Error::new();

        // Empty target and typed
        let stats = classifier.classify_errors("", "");
        assert_eq!(stats.total_errors(), 0);

        // Empty typed string
        let stats = classifier.classify_errors("hello", "");
        assert_eq!(stats.deletion, 5);

        // Empty target string
        let stats = classifier.classify_errors("", "hello");
        assert_eq!(stats.insertion, 5);
    }

    #[test]
    fn test_unicode_support() {
        let mut classifier = Error::new();

        // Test with Unicode characters
        let stats = classifier.classify_errors("café", "cafe");
        assert_eq!(stats.substitution, 1); // é -> e

        // Test with emoji
        let stats = classifier.classify_errors("hello 😊", "hello 😢");
        assert_eq!(stats.substitution, 1); // 😊 -> 😢
    }

    #[test]
    fn test_cache_functionality() {
        let mut classifier = Error::new();

        // First call should compute
        let stats1 = classifier.classify_errors("hello", "hallo");

        // Second call should use cache
        let stats2 = classifier.classify_errors("hello", "hallo");

        assert_eq!(stats1.substitution, stats2.substitution);
        assert_eq!(stats1.insertion, stats2.insertion);
        assert_eq!(stats1.deletion, stats2.deletion);
        assert_eq!(stats1.transposition, stats2.transposition);
    }

    #[test]
    fn test_error_severity() {
        let mut classifier = Error::new();

        // Transposition should have higher severity
        let transposition_stats = classifier.classify_errors("hello", "ehllo");
        let substitution_stats = classifier.classify_errors("hello", "hallo");

        assert!(transposition_stats.severity_score() > substitution_stats.severity_score());
    }
}
