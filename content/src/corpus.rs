//! Text corpus management and static content loading
//!
//! This module provides utilities for managing static text corpora
//! and loading pre-written content for typing training.

use centotype_core::types::*;
use std::collections::HashMap;

/// Static text corpus for training content
#[derive(Debug, Clone)]
pub struct TextCorpus {
    /// Corpus name/identifier
    pub name: String,
    /// Language of the corpus
    pub language: Language,
    /// Category of content
    pub category: ContentCategory,
    /// Text content entries
    pub entries: Vec<String>,
    /// Metadata about the corpus
    pub metadata: CorpusMetadata,
}

/// Metadata about a text corpus
#[derive(Debug, Clone)]
pub struct CorpusMetadata {
    /// Total number of entries
    pub entry_count: usize,
    /// Average entry length in characters
    pub avg_length: f64,
    /// Character distribution across all entries
    pub char_distribution: CharacterClassHistogram,
    /// Estimated difficulty range
    pub difficulty_range: (f64, f64),
}

/// Manager for multiple text corpora
pub struct CorpusManager {
    /// Available corpora organized by category and language
    corpora: HashMap<(ContentCategory, Language), Vec<TextCorpus>>,
}

impl CorpusManager {
    /// Create new corpus manager
    pub fn new() -> Self {
        Self {
            corpora: HashMap::new(),
        }
    }

    /// Add a corpus to the manager
    pub fn add_corpus(&mut self, corpus: TextCorpus) {
        let key = (corpus.category, corpus.language);
        self.corpora.entry(key).or_insert_with(Vec::new).push(corpus);
    }

    /// Get corpora for a specific category and language
    pub fn get_corpora(&self, category: ContentCategory, language: Language) -> Option<&Vec<TextCorpus>> {
        self.corpora.get(&(category, language))
    }

    /// Get random content from a corpus
    pub fn get_random_content(
        &self,
        category: ContentCategory,
        language: Language,
        length_target: usize,
    ) -> Option<String> {
        let corpora = self.get_corpora(category, language)?;

        // Find corpus with entries close to target length
        let mut best_corpus: Option<&TextCorpus> = None;
        let mut best_diff = usize::MAX;

        for corpus in corpora {
            let avg_len = corpus.metadata.avg_length as usize;
            let diff = if avg_len > length_target {
                avg_len - length_target
            } else {
                length_target - avg_len
            };

            if diff < best_diff {
                best_diff = diff;
                best_corpus = Some(corpus);
            }
        }

        // Get random entry from best corpus
        if let Some(corpus) = best_corpus {
            if !corpus.entries.is_empty() {
                use std::collections::hash_map::DefaultHasher;
                use std::hash::{Hash, Hasher};

                let mut hasher = DefaultHasher::new();
                std::thread::current().id().hash(&mut hasher);
                let index = (hasher.finish() as usize) % corpus.entries.len();

                return Some(corpus.entries[index].clone());
            }
        }

        None
    }

    /// Get all available categories and languages
    pub fn get_available_combinations(&self) -> Vec<(ContentCategory, Language)> {
        self.corpora.keys().cloned().collect()
    }
}

impl Default for CorpusManager {
    fn default() -> Self {
        Self::new()
    }
}

/// Load default corpora for basic content
pub fn load_default_corpora() -> CorpusManager {
    let mut manager = CorpusManager::new();

    // Basic English prose corpus
    let english_prose = TextCorpus {
        name: "English Basic Prose".to_string(),
        language: Language::English,
        category: ContentCategory::Prose,
        entries: vec![
            "The quick brown fox jumps over the lazy dog.".to_string(),
            "Programming is the art of telling a computer what to do.".to_string(),
            "Good code is written for humans to read, not just computers.".to_string(),
            "Practice makes perfect when learning to type faster.".to_string(),
        ],
        metadata: CorpusMetadata {
            entry_count: 4,
            avg_length: 45.0,
            char_distribution: CharacterClassHistogram::default(),
            difficulty_range: (5.0, 15.0),
        },
    };

    // Basic Indonesian prose corpus
    let indonesian_prose = TextCorpus {
        name: "Indonesian Basic Prose".to_string(),
        language: Language::Indonesian,
        category: ContentCategory::Prose,
        entries: vec![
            "Latihan mengetik sangat penting untuk meningkatkan kecepatan.".to_string(),
            "Programmer yang baik selalu menulis kode yang mudah dibaca.".to_string(),
            "Teknologi berkembang pesat di era digital ini.".to_string(),
            "Belajar bahasa pemrograman membutuhkan kesabaran dan latihan.".to_string(),
        ],
        metadata: CorpusMetadata {
            entry_count: 4,
            avg_length: 52.0,
            char_distribution: CharacterClassHistogram::default(),
            difficulty_range: (5.0, 15.0),
        },
    };

    // Basic code corpus
    let code_corpus = TextCorpus {
        name: "Basic Programming Code".to_string(),
        language: Language::English,
        category: ContentCategory::Code,
        entries: vec![
            "function add(a, b) { return a + b; }".to_string(),
            "let result = data.filter(x => x > 0);".to_string(),
            "const config = { debug: true, timeout: 5000 };".to_string(),
            "if (condition) { console.log('Success'); }".to_string(),
        ],
        metadata: CorpusMetadata {
            entry_count: 4,
            avg_length: 35.0,
            char_distribution: CharacterClassHistogram::default(),
            difficulty_range: (15.0, 30.0),
        },
    };

    manager.add_corpus(english_prose);
    manager.add_corpus(indonesian_prose);
    manager.add_corpus(code_corpus);

    manager
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_corpus_manager() {
        let manager = load_default_corpora();

        // Test getting available combinations
        let combinations = manager.get_available_combinations();
        assert!(!combinations.is_empty());

        // Test getting random content
        let content = manager.get_random_content(
            ContentCategory::Prose,
            Language::English,
            50
        );
        assert!(content.is_some());
    }

    #[test]
    fn test_corpus_creation() {
        let corpus = TextCorpus {
            name: "Test Corpus".to_string(),
            language: Language::English,
            category: ContentCategory::Technical,
            entries: vec!["test entry".to_string()],
            metadata: CorpusMetadata {
                entry_count: 1,
                avg_length: 10.0,
                char_distribution: CharacterClassHistogram::default(),
                difficulty_range: (5.0, 10.0),
            },
        };

        assert_eq!(corpus.name, "Test Corpus");
        assert_eq!(corpus.metadata.entry_count, 1);
    }
}
