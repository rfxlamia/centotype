//! Content generation system for progressive 100-level typing training
//!
//! This module implements the ContentGenerator trait that creates deterministic,
//! progressively difficult content for each level with proper security validation.

use crate::validation::ContentValidator;
use centotype_core::types::*;
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use std::collections::HashMap;
use std::sync::Arc;
use tracing::{debug, instrument};

/// Main content generator implementing the ContentGenerator trait
pub struct CentotypeContentGenerator {
    validator: Arc<ContentValidator>,
    corpus_data: CorpusData,
}

/// Content generation parameters for a specific level
#[derive(Debug, Clone)]
pub struct LevelGenerationParams {
    pub level_id: LevelId,
    pub seed: u64,
    pub tier: Tier,
    pub tier_progress: u8,
}

impl LevelGenerationParams {
    pub fn new(level_id: LevelId, seed: u64) -> Self {
        let tier = level_id.tier();
        let tier_progress = ((level_id.0 - 1) % 10) + 1;

        Self {
            level_id,
            seed,
            tier,
            tier_progress,
        }
    }
}

/// Difficulty parameters calculated from level progression formulas
#[derive(Debug, Clone)]
pub struct DifficultyParams {
    pub symbol_ratio: f64,
    pub number_ratio: f64,
    pub tech_ratio: f64,
    pub content_length: usize,
    pub switch_freq: usize,
}

impl DifficultyParams {
    /// Calculate difficulty parameters using the mathematical formulas from master prompt
    pub fn calculate(level_id: LevelId) -> Self {
        let tier = level_id.tier().0 as f64;
        let tier_progress = (((level_id.0 - 1) % 10) + 1) as f64;

        // Symbol density: 5% → 30% (Level 1 → 100)
        let symbol_ratio = (5.0 + (tier - 1.0) * 2.5 + (tier_progress - 1.0) * 0.3) / 100.0;

        // Number density: 3% → 20% (Level 1 → 100)
        let number_ratio = (3.0 + (tier - 1.0) * 1.7 + (tier_progress - 1.0) * 0.2) / 100.0;

        // Technical terms: 2% → 15% (Level 1 → 100)
        let tech_ratio = (2.0 + (tier - 1.0) * 1.3 + (tier_progress - 1.0) * 0.2) / 100.0;

        // Content length: 300 → 3000 chars (Level 1 → 100)
        let content_length = 300 + ((tier - 1.0) * 270.0 + (tier_progress - 1.0) * 30.0) as usize;

        // Language switching frequency: 200 → 50 chars
        let switch_freq = (200.0 - (tier - 1.0) * 15.0).max(50.0) as usize;

        Self {
            symbol_ratio,
            number_ratio,
            tech_ratio,
            content_length,
            switch_freq,
        }
    }
}

/// Static corpus data for content generation
#[derive(Debug, Clone)]
pub struct CorpusData {
    pub basic_words: HashMap<Language, Vec<String>>,
    pub tech_terms: HashMap<Language, Vec<String>>,
    pub symbols: Vec<char>,
    pub programming_patterns: Vec<String>,
    pub common_numbers: Vec<String>,
}

impl Default for CorpusData {
    fn default() -> Self {
        let mut basic_words = HashMap::new();
        let mut tech_terms = HashMap::new();

        // English basic vocabulary
        basic_words.insert(Language::English, vec![
            "the".to_string(), "and".to_string(), "you".to_string(), "that".to_string(),
            "was".to_string(), "for".to_string(), "are".to_string(), "with".to_string(),
            "his".to_string(), "they".to_string(), "have".to_string(), "this".to_string(),
            "will".to_string(), "can".to_string(), "had".to_string(), "her".to_string(),
            "what".to_string(), "said".to_string(), "each".to_string(), "which".to_string(),
            "she".to_string(), "how".to_string(), "when".to_string(), "them".to_string(),
            "these".to_string(), "way".to_string(), "many".to_string(), "then".to_string(),
            "write".to_string(), "code".to_string(), "program".to_string(), "function".to_string(),
            "data".to_string(), "system".to_string(), "computer".to_string(), "software".to_string(),
        ]);

        // Indonesian basic vocabulary
        basic_words.insert(Language::Indonesian, vec![
            "dan".to_string(), "yang".to_string(), "di".to_string(), "dengan".to_string(),
            "untuk".to_string(), "dari".to_string(), "pada".to_string(), "ini".to_string(),
            "dalam".to_string(), "tidak".to_string(), "adalah".to_string(), "atau".to_string(),
            "akan".to_string(), "ada".to_string(), "oleh".to_string(), "dapat".to_string(),
            "juga".to_string(), "sebagai".to_string(), "ke".to_string(), "kode".to_string(),
            "program".to_string(), "fungsi".to_string(), "data".to_string(), "sistem".to_string(),
            "komputer".to_string(), "perangkat".to_string(), "lunak".to_string(), "aplikasi".to_string(),
        ]);

        // Technical terms
        tech_terms.insert(Language::English, vec![
            "function".to_string(), "variable".to_string(), "array".to_string(), "object".to_string(),
            "method".to_string(), "class".to_string(), "interface".to_string(), "struct".to_string(),
            "enum".to_string(), "trait".to_string(), "impl".to_string(), "match".to_string(),
            "async".to_string(), "await".to_string(), "Result".to_string(), "Option".to_string(),
            "HashMap".to_string(), "Vector".to_string(), "String".to_string(), "iterator".to_string(),
            "closure".to_string(), "lifetime".to_string(), "borrowing".to_string(), "ownership".to_string(),
        ]);

        tech_terms.insert(Language::Indonesian, vec![
            "fungsi".to_string(), "variabel".to_string(), "array".to_string(), "objek".to_string(),
            "metode".to_string(), "kelas".to_string(), "antarmuka".to_string(), "struktur".to_string(),
            "tipe".to_string(), "sifat".to_string(), "implementasi".to_string(), "cocok".to_string(),
            "asinkron".to_string(), "menunggu".to_string(), "hasil".to_string(), "pilihan".to_string(),
        ]);

        Self {
            basic_words,
            tech_terms,
            symbols: vec![
                '!', '@', '#', '$', '%', '^', '&', '*', '(', ')', '-', '_', '=', '+',
                '[', ']', '{', '}', '\\', '|', ';', ':', '\'', '"', ',', '.', '<', '>',
                '/', '?', '~', '`'
            ],
            programming_patterns: vec![
                "function {}()".to_string(),
                "const {} = {}".to_string(),
                "let mut {} = {}".to_string(),
                "impl {} for {}".to_string(),
                "match {} {{ {} }}".to_string(),
                "if {} {{ {} }}".to_string(),
                "for {} in {} {{ {} }}".to_string(),
                "while {} {{ {} }}".to_string(),
                "{}.map(|{}| {})".to_string(),
                "{}.filter(|{}| {})".to_string(),
            ],
            common_numbers: vec![
                "0".to_string(), "1".to_string(), "2".to_string(), "3".to_string(), "4".to_string(),
                "5".to_string(), "6".to_string(), "7".to_string(), "8".to_string(), "9".to_string(),
                "10".to_string(), "100".to_string(), "1000".to_string(), "2023".to_string(), "2024".to_string(),
                "0x1F".to_string(), "0xFF".to_string(), "42".to_string(), "3.14".to_string(), "1.0".to_string(),
            ],
        }
    }
}

impl CentotypeContentGenerator {
    /// Create new content generator with validation
    pub fn new(validator: Arc<ContentValidator>) -> Self {
        Self {
            validator,
            corpus_data: CorpusData::default(),
        }
    }

    /// Generate content for a specific level with deterministic seeding
    #[instrument(skip(self), fields(level = %params.level_id.0, seed = %params.seed))]
    fn generate_level_content_internal(&self, params: &LevelGenerationParams) -> Result<String> {
        let difficulty = DifficultyParams::calculate(params.level_id);
        let mut rng = ChaCha8Rng::seed_from_u64(params.seed);

        debug!(
            "Generating content for level {} with difficulty params: symbols={:.2}%, numbers={:.2}%, tech={:.2}%, length={}",
            params.level_id.0, difficulty.symbol_ratio * 100.0, difficulty.number_ratio * 100.0,
            difficulty.tech_ratio * 100.0, difficulty.content_length
        );

        let content = match params.tier.0 {
            1..=2 => self.generate_foundation_content(&mut rng, &difficulty)?,
            3..=4 => self.generate_programming_basics_content(&mut rng, &difficulty)?,
            5..=6 => self.generate_intermediate_content(&mut rng, &difficulty)?,
            7..=8 => self.generate_advanced_content(&mut rng, &difficulty)?,
            9..=10 => self.generate_expert_content(&mut rng, &difficulty)?,
            _ => return Err(CentotypeError::Content(format!("Invalid tier: {}", params.tier.0))),
        };

        // Validate the generated content
        self.validator.validate(&content, params.level_id)?;

        Ok(content)
    }

    /// Generate foundation level content (Tier 1-2, Levels 1-20)
    fn generate_foundation_content(&self, rng: &mut ChaCha8Rng, difficulty: &DifficultyParams) -> Result<String> {
        let mut content = String::with_capacity(difficulty.content_length + 100);
        let mut current_lang = if rng.gen_bool(0.5) { Language::English } else { Language::Indonesian };
        let mut chars_written = 0;

        while chars_written < difficulty.content_length {
            // Alternate languages based on switch frequency
            if chars_written > 0 && chars_written % difficulty.switch_freq == 0 {
                current_lang = match current_lang {
                    Language::English => Language::Indonesian,
                    Language::Indonesian => Language::English,
                };
                content.push_str("\n");
                chars_written += 1;
            }

            // Add language indicator
            let lang_prefix = match current_lang {
                Language::English => "EN: ",
                Language::Indonesian => "ID: ",
            };
            content.push_str(lang_prefix);
            chars_written += lang_prefix.len();

            // Generate sentence with basic words
            let sentence = self.generate_basic_sentence(rng, current_lang, difficulty)?;
            content.push_str(&sentence);
            chars_written += sentence.len();

            // Add punctuation
            let punct = if rng.gen_bool(0.7) { ". " } else { "! " };
            content.push_str(punct);
            chars_written += punct.len();

            // Inject numbers based on number_ratio
            if rng.gen_bool(difficulty.number_ratio) {
                let number = &self.corpus_data.common_numbers[rng.gen_range(0..self.corpus_data.common_numbers.len())];
                content.push(' ');
                content.push_str(number);
                chars_written += number.len() + 1;
            }

            content.push(' ');
            chars_written += 1;
        }

        // Trim to exact length
        content.truncate(difficulty.content_length);
        Ok(content)
    }

    /// Generate programming basics content (Tier 3-4, Levels 21-40)
    fn generate_programming_basics_content(&self, rng: &mut ChaCha8Rng, difficulty: &DifficultyParams) -> Result<String> {
        let mut content = String::with_capacity(difficulty.content_length + 100);
        let mut chars_written = 0;

        while chars_written < difficulty.content_length {
            // Mix prose and code
            if rng.gen_bool(0.6) {
                // Generate code snippet
                let code = self.generate_basic_code_snippet(rng, difficulty)?;
                content.push_str(&code);
                chars_written += code.len();
            } else {
                // Generate technical prose
                let prose = self.generate_technical_prose(rng, difficulty)?;
                content.push_str(&prose);
                chars_written += prose.len();
            }

            content.push_str("\n");
            chars_written += 1;
        }

        content.truncate(difficulty.content_length);
        Ok(content)
    }

    /// Generate intermediate complexity content (Tier 5-6, Levels 41-60)
    fn generate_intermediate_content(&self, rng: &mut ChaCha8Rng, difficulty: &DifficultyParams) -> Result<String> {
        let mut content = String::with_capacity(difficulty.content_length + 100);
        let mut chars_written = 0;

        while chars_written < difficulty.content_length {
            // Higher symbol density and complexity
            if rng.gen_bool(0.7) {
                let code = self.generate_complex_code_snippet(rng, difficulty)?;
                content.push_str(&code);
                chars_written += code.len();
            } else {
                let mixed = self.generate_mixed_content(rng, difficulty)?;
                content.push_str(&mixed);
                chars_written += mixed.len();
            }

            content.push_str("\n");
            chars_written += 1;
        }

        content.truncate(difficulty.content_length);
        Ok(content)
    }

    /// Generate advanced programming content (Tier 7-8, Levels 61-80)
    fn generate_advanced_content(&self, rng: &mut ChaCha8Rng, difficulty: &DifficultyParams) -> Result<String> {
        let mut content = String::with_capacity(difficulty.content_length + 100);
        let mut chars_written = 0;

        while chars_written < difficulty.content_length {
            // Heavy symbol usage and complex patterns
            let advanced = self.generate_advanced_code_pattern(rng, difficulty)?;
            content.push_str(&advanced);
            chars_written += advanced.len();
            content.push_str("\n");
            chars_written += 1;
        }

        content.truncate(difficulty.content_length);
        Ok(content)
    }

    /// Generate expert mastery content (Tier 9-10, Levels 81-100)
    fn generate_expert_content(&self, rng: &mut ChaCha8Rng, difficulty: &DifficultyParams) -> Result<String> {
        let mut content = String::with_capacity(difficulty.content_length + 100);
        let mut chars_written = 0;

        while chars_written < difficulty.content_length {
            // Maximum complexity with edge cases
            let expert = self.generate_expert_pattern(rng, difficulty)?;
            content.push_str(&expert);
            chars_written += expert.len();
            content.push_str("\n");
            chars_written += 1;
        }

        content.truncate(difficulty.content_length);
        Ok(content)
    }

    /// Generate basic sentence with common words
    fn generate_basic_sentence(&self, rng: &mut ChaCha8Rng, lang: Language, _difficulty: &DifficultyParams) -> Result<String> {
        let words = &self.corpus_data.basic_words[&lang];
        let sentence_len = rng.gen_range(4..8);
        let mut sentence = Vec::with_capacity(sentence_len);

        for _ in 0..sentence_len {
            let word = &words[rng.gen_range(0..words.len())];
            sentence.push(word.clone());
        }

        Ok(sentence.join(" "))
    }

    /// Generate basic code snippet for programming basics tiers
    fn generate_basic_code_snippet(&self, rng: &mut ChaCha8Rng, difficulty: &DifficultyParams) -> Result<String> {
        let patterns = &self.corpus_data.programming_patterns;
        let pattern = &patterns[rng.gen_range(0..patterns.len())];

        // Simple variable names and basic symbols
        let var_name = format!("var{}", rng.gen_range(1..10));
        let value = if rng.gen_bool(difficulty.number_ratio) {
            self.corpus_data.common_numbers[rng.gen_range(0..self.corpus_data.common_numbers.len())].clone()
        } else {
            format!("\"{}\"", self.generate_basic_sentence(rng, Language::English, difficulty)?)
        };

        Ok(pattern.replace("{}", &var_name).replace("{}", &value))
    }

    /// Generate complex code snippet for intermediate tiers
    fn generate_complex_code_snippet(&self, rng: &mut ChaCha8Rng, difficulty: &DifficultyParams) -> Result<String> {
        let mut code = String::new();

        // Add symbols based on symbol_ratio
        for _ in 0..(difficulty.symbol_ratio * 20.0) as usize {
            if rng.gen_bool(0.5) {
                let symbol = self.corpus_data.symbols[rng.gen_range(0..self.corpus_data.symbols.len())];
                code.push(symbol);
            }
        }

        // Add technical terms
        if rng.gen_bool(difficulty.tech_ratio) {
            let tech_words = &self.corpus_data.tech_terms[&Language::English];
            let tech = &tech_words[rng.gen_range(0..tech_words.len())];
            code.push_str(tech);
        }

        // Add brackets and operators
        code.push_str(" { [( )] } ");

        Ok(code)
    }

    /// Generate mixed content for intermediate levels
    fn generate_mixed_content(&self, rng: &mut ChaCha8Rng, difficulty: &DifficultyParams) -> Result<String> {
        let mut content = String::new();

        // Mix of prose and symbols
        let prose = self.generate_basic_sentence(rng, Language::English, difficulty)?;
        content.push_str(&prose);

        // Inject symbols
        if rng.gen_bool(difficulty.symbol_ratio) {
            content.push(' ');
            for _ in 0..3 {
                let symbol = self.corpus_data.symbols[rng.gen_range(0..self.corpus_data.symbols.len())];
                content.push(symbol);
            }
        }

        Ok(content)
    }

    /// Generate advanced code patterns for high tiers
    fn generate_advanced_code_pattern(&self, rng: &mut ChaCha8Rng, difficulty: &DifficultyParams) -> Result<String> {
        let mut pattern = String::new();

        // Complex symbol combinations
        pattern.push_str("&mut HashMap<String, Vec<Option<Box<dyn Iterator<Item=u32>>>>>");

        // Bitwise operations
        if rng.gen_bool(0.7) {
            pattern.push_str(" | (mask << 8) ^ ~(flags & 0xFF) >> 2");
        }

        // Hex values
        if rng.gen_bool(difficulty.number_ratio) {
            pattern.push_str(&format!(" 0x{:X}", rng.gen_range(0x1000..0xFFFF)));
        }

        Ok(pattern)
    }

    /// Generate expert-level patterns for mastery tiers
    fn generate_expert_pattern(&self, rng: &mut ChaCha8Rng, _difficulty: &DifficultyParams) -> Result<String> {
        let patterns = vec![
            "async fn process<T: Send + Sync + 'static>() -> Result<impl Iterator<Item=T>, Box<dyn Error>>",
            "0x1F3F4E40 | (mask << 8) ^ ~(flags & 0xFF) >> 2",
            "where Self: Clone + Debug + PartialEq<T> + Into<U> + From<V>",
            "macro_rules! impl_trait { ($($t:ty),*) => { $(impl Trait for $t {})* }; }",
            "pin_project! { struct Future<T> { #[pin] inner: T, state: State } }",
        ];

        Ok(patterns[rng.gen_range(0..patterns.len())].to_string())
    }

    /// Generate technical prose with domain terminology
    fn generate_technical_prose(&self, rng: &mut ChaCha8Rng, difficulty: &DifficultyParams) -> Result<String> {
        let lang = if rng.gen_bool(0.6) { Language::English } else { Language::Indonesian };
        let tech_words = &self.corpus_data.tech_terms[&lang];
        let basic_words = &self.corpus_data.basic_words[&lang];

        let mut prose = Vec::new();

        // Mix technical and basic words
        for _ in 0..rng.gen_range(5..10) {
            if rng.gen_bool(difficulty.tech_ratio) {
                prose.push(tech_words[rng.gen_range(0..tech_words.len())].clone());
            } else {
                prose.push(basic_words[rng.gen_range(0..basic_words.len())].clone());
            }
        }

        Ok(prose.join(" "))
    }
}

/// Implementation of the ContentGenerator trait from core
impl ContentGenerator for CentotypeContentGenerator {
    fn generate(&self, params: ContentParams) -> Result<TextContent> {
        let level_id = params.level.ok_or_else(|| {
            CentotypeError::Content("Level ID required for content generation".to_string())
        })?;

        let seed = params.seed.unwrap_or_else(|| {
            use std::collections::hash_map::DefaultHasher;
            use std::hash::{Hash, Hasher};
            let mut hasher = DefaultHasher::new();
            level_id.hash(&mut hasher);
            hasher.finish()
        });

        let gen_params = LevelGenerationParams::new(level_id, seed);
        let text = self.generate_level_content_internal(&gen_params)?;

        // Calculate metadata
        let difficulty_score = DifficultyParams::calculate(level_id);
        let char_classes = self.calculate_character_histogram(&text);

        let content = TextContent {
            text,
            metadata: ContentMetadata {
                level: Some(level_id),
                category: params.category,
                language: params.language,
                difficulty_score: difficulty_score.symbol_ratio + difficulty_score.number_ratio + difficulty_score.tech_ratio,
                char_classes,
                estimated_duration_secs: (params.length_chars / 50).max(30) as u32, // Assume 50 CPM baseline
            },
        };

        Ok(content)
    }

    fn validate(&self, content: &TextContent) -> Result<()> {
        if let Some(level_id) = content.metadata.level {
            self.validator.validate(&content.text, level_id)?;
        }
        Ok(())
    }
}

impl CentotypeContentGenerator {
    /// Calculate character class distribution for content
    fn calculate_character_histogram(&self, text: &str) -> CharacterClassHistogram {
        let mut histogram = CharacterClassHistogram::default();

        for ch in text.chars() {
            match ch {
                'a'..='z' => histogram.lowercase += 1,
                'A'..='Z' => histogram.uppercase += 1,
                '0'..='9' => histogram.digits += 1,
                ' ' | '\t' | '\n' | '\r' => histogram.whitespace += 1,
                '.' | ',' | ';' | ':' | '!' | '?' | '\'' | '"' => histogram.punctuation += 1,
                _ => histogram.symbols += 1,
            }
        }

        histogram
    }

    /// Generate content for a specific level (public interface for caching)
    pub fn generate_level_content(&self, level_id: LevelId, seed: u64) -> Result<String> {
        let params = LevelGenerationParams::new(level_id, seed);
        self.generate_level_content_internal(&params)
    }

    /// Validate that content meets difficulty requirements for a level
    pub fn validate_content_difficulty(&self, content: &str, level_id: LevelId) -> bool {
        let expected = DifficultyParams::calculate(level_id);
        let histogram = self.calculate_character_histogram(content);
        let total_chars = content.len() as f64;

        if total_chars == 0.0 {
            return false;
        }

        // Calculate actual ratios
        let actual_symbol_ratio = histogram.symbols as f64 / total_chars;
        let actual_number_ratio = histogram.digits as f64 / total_chars;

        // Allow ±5% tolerance for difficulty matching
        let symbol_diff = (actual_symbol_ratio - expected.symbol_ratio).abs();
        let number_diff = (actual_number_ratio - expected.number_ratio).abs();

        symbol_diff <= 0.05 && number_diff <= 0.05
    }
}

/// Cache key generation for deterministic content
pub fn generate_cache_key(level_id: LevelId, seed: u64) -> String {
    format!("content_v1_{}_{}", level_id.0, seed)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::validation::ContentValidator;

    #[test]
    fn test_difficulty_params_calculation() {
        let level_1 = DifficultyParams::calculate(LevelId::new(1).unwrap());
        assert!((level_1.symbol_ratio - 0.05).abs() < 0.001); // 5% with tolerance
        assert_eq!(level_1.content_length, 300);

        let level_100 = DifficultyParams::calculate(LevelId::new(100).unwrap());
        assert!((level_100.symbol_ratio - 0.30).abs() < 0.01); // 30% with tolerance
        assert_eq!(level_100.content_length, 3000);
    }

    #[test]
    fn test_deterministic_generation() {
        let validator = Arc::new(ContentValidator::new().unwrap());
        let generator = CentotypeContentGenerator::new(validator);

        let level = LevelId::new(1).unwrap();
        let seed = 12345;

        let content1 = generator.generate_level_content(level, seed).unwrap();
        let content2 = generator.generate_level_content(level, seed).unwrap();

        assert_eq!(content1, content2, "Content should be deterministic for same seed");
    }

    #[test]
    fn test_cache_key_generation() {
        let level = LevelId::new(42).unwrap();
        let seed = 67890;
        let key = generate_cache_key(level, seed);

        assert_eq!(key, "content_v1_42_67890");
    }
}
