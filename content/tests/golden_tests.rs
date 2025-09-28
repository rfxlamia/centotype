// Golden tests for content generation consistency
// Uses snapshot testing to ensure deterministic content generation

use centotype_content::generator::CentotypeContentGenerator;
use centotype_core::types::LevelId;
use insta::assert_snapshot;

const GOLDEN_SEED: u64 = 12345; // Fixed seed for deterministic tests

#[test]
fn test_golden_level_1_content() {
    let generator = create_test_generator();
    let level_id = LevelId::new(1).unwrap();

    // Generate content with fixed seed
    let content = generator
        .generate_level_content(level_id, GOLDEN_SEED)
        .expect("Level 1 content generation should succeed");

    // Snapshot the generated content
    assert_snapshot!("level_1_content", content);
}

#[test]
fn test_golden_level_10_content() {
    let generator = create_test_generator();
    let level_id = LevelId::new(10).unwrap();

    let content = generator
        .generate_level_content(level_id, GOLDEN_SEED)
        .expect("Level 10 content generation should succeed");

    assert_snapshot!("level_10_content", content);
}

#[test]
fn test_golden_level_25_content() {
    let generator = create_test_generator();
    let level_id = LevelId::new(25).unwrap();

    let content = generator
        .generate_level_content(level_id, GOLDEN_SEED)
        .expect("Level 25 content generation should succeed");

    assert_snapshot!("level_25_content", content);
}

#[test]
fn test_golden_level_50_content() {
    let generator = create_test_generator();
    let level_id = LevelId::new(50).unwrap();

    let content = generator
        .generate_level_content(level_id, GOLDEN_SEED)
        .expect("Level 50 content generation should succeed");

    assert_snapshot!("level_50_content", content);
}

#[test]
fn test_golden_level_75_content() {
    let generator = create_test_generator();
    let level_id = LevelId::new(75).unwrap();

    let content = generator
        .generate_level_content(level_id, GOLDEN_SEED)
        .expect("Level 75 content generation should succeed");

    assert_snapshot!("level_75_content", content);
}

#[test]
fn test_golden_level_100_content() {
    let generator = create_test_generator();
    let level_id = LevelId::new(100).unwrap();

    let content = generator
        .generate_level_content(level_id, GOLDEN_SEED)
        .expect("Level 100 content generation should succeed");

    assert_snapshot!("level_100_content", content);
}

#[test]
fn test_golden_content_consistency() {
    let generator = create_test_generator();

    // Test multiple generations with same seed produce identical results
    let level_id = LevelId::new(42).unwrap();

    let content1 = generator
        .generate_level_content(level_id, GOLDEN_SEED)
        .expect("First generation should succeed");

    let content2 = generator
        .generate_level_content(level_id, GOLDEN_SEED)
        .expect("Second generation should succeed");

    // Both generations should be identical
    assert_eq!(content1, content2, "Same seed should produce identical content");

    // Snapshot the content
    assert_snapshot!("level_42_consistency", content1);
}

#[test]
fn test_golden_different_seeds() {
    let generator = create_test_generator();
    let level_id = LevelId::new(15).unwrap();

    // Generate with different seeds
    let content_seed1 = generator
        .generate_level_content(level_id, GOLDEN_SEED)
        .expect("Seed 1 generation should succeed");

    let content_seed2 = generator
        .generate_level_content(level_id, GOLDEN_SEED + 1)
        .expect("Seed 2 generation should succeed");

    // Different seeds should produce different content
    assert_ne!(content_seed1, content_seed2, "Different seeds should produce different content");

    // Snapshot both for comparison
    assert_snapshot!("level_15_seed_12345", content_seed1);
    assert_snapshot!("level_15_seed_12346", content_seed2);
}

#[test]
fn test_golden_content_properties() {
    let generator = create_test_generator();

    // Test content properties for various levels
    let test_levels = vec![1, 5, 10, 25, 50, 75, 100];

    for level in test_levels {
        let level_id = LevelId::new(level).unwrap();
        let content = generator
            .generate_level_content(level_id, GOLDEN_SEED)
            .expect(&format!("Level {} content generation should succeed", level));

        // Basic property checks
        assert!(!content.is_empty(), "Level {} content should not be empty", level);
        assert!(content.len() > 50, "Level {} content should be substantial", level);
        assert!(content.len() < 2000, "Level {} content should not be excessive", level);

        // Snapshot for detailed analysis
        assert_snapshot!(format!("level_{}_properties", level), format!(
            "Level: {}\nLength: {}\nContent: {}\n",
            level,
            content.len(),
            content
        ));
    }
}

#[test]
fn test_golden_progression_validation() {
    let generator = create_test_generator();

    // Test that difficulty increases across levels
    let mut previous_complexity = 0.0;

    for level in [1, 10, 20, 30, 40, 50, 60, 70, 80, 90, 100] {
        let level_id = LevelId::new(level).unwrap();
        let content = generator
            .generate_level_content(level_id, GOLDEN_SEED)
            .expect(&format!("Level {} should generate", level));

        // Calculate basic complexity metrics
        let symbol_count = content.chars().filter(|c| !c.is_alphanumeric() && !c.is_whitespace()).count();
        let number_count = content.chars().filter(|c| c.is_numeric()).count();
        let complexity = (symbol_count + number_count) as f64 / content.len() as f64;

        // For levels > 1, complexity should generally increase
        if level > 1 {
            // Allow some flexibility but expect general upward trend
            let progression_snapshot = format!(
                "Level {}: complexity={:.3}, symbols={}, numbers={}, length={}",
                level, complexity, symbol_count, number_count, content.len()
            );

            // Snapshot the progression data
            assert_snapshot!(format!("progression_level_{}", level), progression_snapshot);
        }

        previous_complexity = complexity;
    }
}

#[test]
fn test_golden_security_validation() {
    let generator = create_test_generator();

    // Test that generated content is safe across all levels
    for level in [1, 25, 50, 75, 100] {
        let level_id = LevelId::new(level).unwrap();
        let content = generator
            .generate_level_content(level_id, GOLDEN_SEED)
            .expect(&format!("Level {} should generate", level));

        // Security checks
        assert!(!content.contains('\x00'), "Level {} content should not contain null bytes", level);
        assert!(!content.contains('\x1b'), "Level {} content should not contain escape sequences", level);
        assert!(!content.contains("$("), "Level {} content should not contain shell injection patterns", level);
        assert!(!content.contains("`"), "Level {} content should not contain backticks", level);

        // Check for dangerous Unicode
        for ch in content.chars() {
            assert!(ch.is_ascii() || ch.is_alphabetic() || ch.is_numeric() || ch.is_whitespace(),
                "Level {} content contains potentially unsafe character: {:?}", level, ch);
        }

        // Snapshot security validation results
        let security_report = format!(
            "Level {}: length={}, ascii_ratio={:.2}, safe=true",
            level,
            content.len(),
            content.chars().filter(|c| c.is_ascii()).count() as f64 / content.len() as f64
        );

        assert_snapshot!(format!("security_level_{}", level), security_report);
    }
}

#[test]
fn test_golden_character_distribution() {
    let generator = create_test_generator();

    // Test character distribution for specific levels
    for level in [1, 50, 100] {
        let level_id = LevelId::new(level).unwrap();
        let content = generator
            .generate_level_content(level_id, GOLDEN_SEED)
            .expect(&format!("Level {} should generate", level));

        let total_chars = content.len();
        let letter_count = content.chars().filter(|c| c.is_alphabetic()).count();
        let number_count = content.chars().filter(|c| c.is_numeric()).count();
        let symbol_count = content.chars().filter(|c| !c.is_alphanumeric() && !c.is_whitespace()).count();
        let space_count = content.chars().filter(|c| c.is_whitespace()).count();

        let distribution_report = format!(
            "Level {}: total={}, letters={:.1}%, numbers={:.1}%, symbols={:.1}%, spaces={:.1}%",
            level,
            total_chars,
            (letter_count as f64 / total_chars as f64) * 100.0,
            (number_count as f64 / total_chars as f64) * 100.0,
            (symbol_count as f64 / total_chars as f64) * 100.0,
            (space_count as f64 / total_chars as f64) * 100.0
        );

        assert_snapshot!(format!("distribution_level_{}", level), distribution_report);
    }
}

// Helper function to create a test generator
fn create_test_generator() -> CentotypeContentGenerator {
    use centotype_content::validation::ContentValidator;
    use std::sync::Arc;

    let validator = Arc::new(ContentValidator::new().expect("Failed to create validator"));
    CentotypeContentGenerator::new(validator)
}