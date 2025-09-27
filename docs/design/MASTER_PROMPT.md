# Centotype Master Prompt System for 100-Level Content Generation

## Content Generation Master Prompt

You are generating typing training content for Centotype CLI, a progressive 100-level typing trainer. Create content that follows these specifications:

### Level Parameters
- **Level ID**: {level_id} (1-100)
- **Tier**: {tier} (1-10, calculated as ceil(level_id/10))
- **Tier Progress**: {tier_progress} (1-10, level_id % 10 or 10 if divisible)
- **Random Seed**: {seed} (for deterministic generation)

### Difficulty Progression Formulas

#### Symbol Density (%)
```
symbol_ratio = 5 + (tier - 1) * 2.5 + (tier_progress - 1) * 0.3
// Level 1: 5.0%, Level 100: 30.0%
```

#### Number Density (%)
```
number_ratio = 3 + (tier - 1) * 1.7 + (tier_progress - 1) * 0.2
// Level 1: 3.0%, Level 100: 20.0%
```

#### Technical Terms Density (%)
```
tech_ratio = 2 + (tier - 1) * 1.3 + (tier_progress - 1) * 0.2
// Level 1: 2.0%, Level 100: 15.0%
```

#### Content Length (characters)
```
content_length = 300 + (tier - 1) * 270 + (tier_progress - 1) * 30
// Level 1: 300 chars, Level 100: 3000 chars
```

#### Language Switching Frequency
```
switch_freq = max(200 - (tier - 1) * 15, 50)
// Level 1: 200 chars, Level 100: 50 chars
```

### Content Composition Requirements

#### Tier 1-2 (Levels 1-20): Foundation
- Basic alphabetic text with common words
- Simple punctuation (., , ! ?)
- Numbers in contexts (dates, simple math)
- Indonesian/English common vocabulary

#### Tier 3-4 (Levels 21-40): Programming Basics
- Introduction of programming symbols: {} [] ()
- Basic operators: + - * / =
- camelCase and snake_case patterns
- Simple code snippets and technical terms

#### Tier 5-6 (Levels 41-60): Intermediate Complexity
- Advanced symbols: | & ^ ~ % $ #
- Nested brackets and complex expressions
- Mixed-case scenarios with technical jargon
- Code documentation patterns

#### Tier 7-8 (Levels 61-80): Advanced Programming
- Complex symbol combinations: <> |&| ^^^ ~~~
- Bitwise operations and hex values
- Multi-language code switching
- Advanced technical terminology

#### Tier 9-10 (Levels 81-100): Expert Mastery
- Maximum symbol density with edge cases
- Unicode characters and special encodings
- Rapid language switching with no warnings
- Professional technical writing complexity

### Security Validation Requirements

#### Input Sanitization Rules
1. **No Escape Sequences**: Content must not contain terminal escape sequences (\x1b, \033)
2. **No Shell Commands**: No executable commands or shell metacharacters in inappropriate contexts
3. **No File Paths**: Avoid absolute file paths that could reveal system information
4. **Safe Unicode**: Only use printable Unicode characters in Basic Latin + common extensions
5. **Length Bounds**: Respect minimum/maximum content length parameters

#### Validation Tests (Must Pass)
```rust
// Test 1: No escape sequences
assert!(!content.contains('\x1b'));
assert!(!content.contains("\033"));

// Test 2: No shell injection patterns
assert!(!content.contains("$("));
assert!(!content.contains("`"));
assert!(!content.contains("&&"));

// Test 3: Length validation
assert!(content.len() >= min_length);
assert!(content.len() <= max_length);

// Test 4: Character composition
assert!(validate_symbol_ratio(content, target_symbol_ratio));
```

### Performance Constraints

#### Generation Requirements
- **Deterministic**: Same seed + level must produce identical content
- **Fast Generation**: <10ms generation time per level
- **Memory Efficient**: <1MB memory footprint during generation
- **Cacheable**: Content structure supports efficient LRU caching

#### Caching Integration
- Generate cache key: `content_v1_{level_id}_{seed}`
- Support cache invalidation for content updates
- Optimize for <25ms content loading (including cache lookup)

### Content Templates by Tier

#### Template Structure
```
{language_indicator}:{content_type}:{difficulty_markers}
{generated_content}
{validation_checksum}
```

#### Tier-Specific Patterns

**Tier 1-2**: Simple prose with basic punctuation
```
ID: Saya suka menulis kode yang bersih dan mudah dibaca.
EN: I love writing clean and readable code every day.
```

**Tier 5-6**: Mixed programming content
```
ID: function calculateSum(arr) { return arr.reduce((a,b) => a+b, 0); }
EN: const config = { debug: true, timeout: 5000, retries: 3 };
```

**Tier 9-10**: Expert-level complexity
```
ID: &mut HashMap<String, Vec<Option<Box<dyn Iterator<Item=u32>>>>>
EN: 0x1F3F4E40 | (mask << 8) ^ ~(flags & 0xFF) >> 2
```

### Quality Metrics

#### Measurable Success Criteria
1. **Progression Smoothness**: Each level 3-7% harder than previous
2. **Completion Rate**: >85% of generated levels meet difficulty targets
3. **Performance Compliance**: 100% of content loads within 25ms
4. **Security Pass Rate**: 100% pass security validation
5. **Deterministic Success**: Identical output for same seed+level across 100 runs

#### Content Balance Validation
```python
def validate_content_balance(content, level_id):
    actual_symbols = count_symbols(content) / len(content)
    expected_symbols = calculate_symbol_ratio(level_id)
    return abs(actual_symbols - expected_symbols) < 0.02  # ±2% tolerance
```

## Agent Coordination Specifications

### For rust-pro Agent
**Context**: Integrate generated corpus into content/ crate
**Requirements**:
- Implement `ContentGenerator` trait with deterministic seeding
- Add LRU cache with Moka crate for <25ms loading
- Connect to core/ crate's `LevelManager` for progression
- Support async loading for better UX

**Key Integration Points**:
```rust
pub trait ContentGenerator {
    fn generate_level_content(&self, level_id: LevelId, seed: u64) -> Result<String>;
    fn get_cached_content(&self, level_id: LevelId) -> Option<String>;
    fn validate_content_difficulty(&self, content: &str, level_id: LevelId) -> bool;
}
```

### For backend-architect Agent
**Context**: Validate inter-crate flow for performance targets
**Requirements**:
- Ensure content/ → core/ → engine/ data flow <25ms
- Design content preloading strategy for next 3 levels
- Implement graceful degradation for cache misses
- Optimize cross-crate communication patterns

### For security-auditor Agent
**Context**: Comprehensive security validation of corpus content
**Requirements**:
- Implement terminal escape sequence detection
- Add malicious pattern recognition (injection attempts)
- Validate Unicode character safety
- Create security test suite for content validation

### For performance-engineer Agent
**Context**: Benchmark system against P99 latency targets
**Requirements**:
- Measure content generation latency distribution
- Benchmark cache hit/miss performance
- Profile memory usage during content loading
- Validate <25ms P99 input latency with content system

### For test-automator Agent
**Context**: Comprehensive testing for content system
**Requirements**:
- Unit tests for deterministic generation
- Integration tests for cache performance
- Snapshot testing for content consistency
- Property-based testing for security validation

### For docs-architect Agent
**Context**: User and developer documentation
**Requirements**:
- Document content generation API
- Create troubleshooting guide for content issues
- Write quickstart for custom content development
- Explain difficulty progression system

## Validation System Implementation

### Content Validation Pipeline
```rust
pub struct ContentValidator {
    security_validator: SecurityValidator,
    difficulty_validator: DifficultyValidator,
    performance_validator: PerformanceValidator,
}

impl ContentValidator {
    pub fn validate_generated_content(
        &self,
        content: &str,
        level_id: LevelId
    ) -> ValidationResult {
        // 1. Security validation
        self.security_validator.validate(content)?;

        // 2. Difficulty progression validation
        self.difficulty_validator.validate(content, level_id)?;

        // 3. Performance impact validation
        self.performance_validator.validate(content)?;

        ValidationResult::Valid
    }
}
```

### Progressive Difficulty Verification
```rust
pub fn verify_difficulty_progression(contents: &[String]) -> Result<()> {
    for (i, content) in contents.iter().enumerate() {
        let current_difficulty = calculate_difficulty_score(content);
        if i > 0 {
            let prev_difficulty = calculate_difficulty_score(&contents[i-1]);
            let increase = (current_difficulty - prev_difficulty) / prev_difficulty;

            // Ensure 3-7% difficulty increase per level
            assert!(increase >= 0.03 && increase <= 0.07,
                "Level {} difficulty increase {:.2}% outside target range",
                i + 1, increase * 100.0);
        }
    }
    Ok(())
}
```

### Cache Performance Monitoring
```rust
pub struct CacheMetrics {
    hit_rate: f64,
    avg_lookup_time: Duration,
    memory_usage: usize,
    eviction_count: u64,
}

impl CacheMetrics {
    pub fn validate_performance_targets(&self) -> Result<()> {
        assert!(self.hit_rate >= 0.90, "Cache hit rate too low: {:.2}%", self.hit_rate * 100.0);
        assert!(self.avg_lookup_time.as_millis() < 5, "Cache lookup too slow: {}ms", self.avg_lookup_time.as_millis());
        Ok(())
    }
}
```

## Usage Instructions

### For Content Generation
1. Use the master prompt template with specific level parameters
2. Apply security validation to all generated content
3. Verify difficulty progression meets mathematical formulas
4. Test cache integration and performance targets
5. Document any edge cases or special handling required

### For Agent Coordination
1. Each agent should use their specific context section
2. Implement required traits and interfaces as specified
3. Coordinate with other agents through well-defined APIs
4. Validate integration points through comprehensive testing
5. Maintain performance targets throughout development

### Success Validation
- All 100 levels generate successfully with unique, appropriate content
- Content loading consistently meets <25ms P99 target
- Security validation passes 100% of test cases
- Difficulty progression shows measurable, consistent improvement
- Cross-platform compatibility verified on Linux/macOS/Windows

This master prompt system ensures consistent, secure, and performant content generation for the Centotype typing trainer while enabling effective coordination between specialized development agents.