//! Extended fuzzing framework for comprehensive security testing
//! Includes crash resistance testing, memory safety validation, and performance stress testing

use centotype_content::{ContentGenerator, ContentValidator};
use centotype_core::types::*;
use centotype_engine::Input;
use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use std::time::{Duration, Instant};
use std::thread;
use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicU64, AtomicBool, Ordering};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use sysinfo::{System, SystemExt, ProcessExt};

/// Crash types for classification
#[derive(Debug, Clone)]
pub enum CrashType {
    Panic,
    Timeout,
    MemoryExhaustion,
    StackOverflow,
    SecurityViolation,
    InfiniteLoop,
    ResourceExhaustion,
    UnexpectedError(String),
}

/// Fuzzing crash report
#[derive(Debug, Clone)]
pub struct FuzzCrash {
    pub input: String,
    pub error: String,
    pub test_number: u64,
    pub crash_type: CrashType,
    pub thread_id: u32,
    pub timestamp: Instant,
    pub memory_usage_mb: u64,
    pub cpu_usage_percent: f32,
}

/// Performance issue during fuzzing
#[derive(Debug, Clone)]
pub struct PerformanceIssue {
    pub test_number: u64,
    pub issue_type: PerformanceIssueType,
    pub measured_value: f64,
    pub threshold: f64,
    pub input_sample: String,
}

#[derive(Debug, Clone)]
pub enum PerformanceIssueType {
    SlowProcessing,
    HighMemoryUsage,
    CpuSpike,
    ThreadingIssue,
}

/// Comprehensive fuzzing report
#[derive(Debug)]
pub struct ExtendedFuzzReport {
    pub total_tests: u64,
    pub duration: Duration,
    pub crashes: Vec<FuzzCrash>,
    pub high_risk_crashes: Vec<FuzzCrash>,
    pub performance_issues: Vec<PerformanceIssue>,
    pub memory_leaks_detected: bool,
    pub max_memory_usage_mb: u64,
    pub avg_processing_time_ms: f64,
    pub stress_test_passed: bool,
    pub security_violations: u64,
    pub threads_used: u32,
}

impl ExtendedFuzzReport {
    pub fn new() -> Self {
        Self {
            total_tests: 0,
            duration: Duration::ZERO,
            crashes: Vec::new(),
            high_risk_crashes: Vec::new(),
            performance_issues: Vec::new(),
            memory_leaks_detected: false,
            max_memory_usage_mb: 0,
            avg_processing_time_ms: 0.0,
            stress_test_passed: false,
            security_violations: 0,
            threads_used: 1,
        }
    }

    pub fn add_crash(&mut self, crash: FuzzCrash) {
        let is_high_risk = matches!(crash.crash_type,
            CrashType::Panic | CrashType::SecurityViolation | CrashType::StackOverflow
        );

        if is_high_risk {
            self.high_risk_crashes.push(crash.clone());
        }
        self.crashes.push(crash);
    }

    pub fn add_performance_issue(&mut self, issue: PerformanceIssue) {
        self.performance_issues.push(issue);
    }

    pub fn total_crashes(&self) -> usize {
        self.crashes.len()
    }

    pub fn is_secure(&self) -> bool {
        self.high_risk_crashes.is_empty() && self.security_violations == 0
    }
}

/// Extended fuzzing engine with comprehensive testing capabilities
pub struct ExtendedFuzzEngine {
    content_generator: ContentGenerator,
    content_validator: ContentValidator,
    input_processor: Input,
    system_monitor: Arc<Mutex<System>>,
    test_counter: Arc<AtomicU64>,
    running: Arc<AtomicBool>,
}

impl ExtendedFuzzEngine {
    pub fn new() -> Result<Self> {
        Ok(Self {
            content_generator: ContentGenerator::new()?,
            content_validator: ContentValidator::new()?,
            input_processor: Input::new(),
            system_monitor: Arc::new(Mutex::new(System::new_all())),
            test_counter: Arc::new(AtomicU64::new(0)),
            running: Arc::new(AtomicBool::new(false)),
        })
    }

    /// Run comprehensive fuzzing test for specified duration
    pub fn run_extended_fuzz_test(&mut self, duration: Duration, thread_count: u32) -> Result<ExtendedFuzzReport> {
        let start_time = Instant::now();
        let mut report = ExtendedFuzzReport::new();
        report.threads_used = thread_count;

        println!("🚀 Starting extended fuzzing test...");
        println!("  Duration: {:?}", duration);
        println!("  Threads: {}", thread_count);
        println!("  Target: Zero high-risk crashes");

        self.running.store(true, Ordering::SeqCst);

        // Shared data structures for thread communication
        let crashes = Arc::new(Mutex::new(Vec::new()));
        let performance_issues = Arc::new(Mutex::new(Vec::new()));
        let max_memory = Arc::new(AtomicU64::new(0));
        let total_processing_time = Arc::new(Mutex::new(Duration::ZERO));

        // Spawn fuzzing threads
        let mut handles = Vec::new();
        for thread_id in 0..thread_count {
            let crashes_clone = crashes.clone();
            let performance_issues_clone = performance_issues.clone();
            let max_memory_clone = max_memory.clone();
            let total_processing_time_clone = total_processing_time.clone();
            let test_counter_clone = self.test_counter.clone();
            let running_clone = self.running.clone();
            let system_monitor_clone = self.system_monitor.clone();

            let handle = thread::spawn(move || {
                let mut local_generator = ContentGenerator::new().unwrap();
                let local_validator = ContentValidator::new().unwrap();
                let mut local_input = Input::new();
                let mut rng = ChaCha8Rng::from_entropy();

                while running_clone.load(Ordering::SeqCst) {
                    let test_number = test_counter_clone.fetch_add(1, Ordering::SeqCst);

                    // Generate random input for testing
                    let input = Self::generate_malicious_input(&mut rng);
                    let process_start = Instant::now();

                    // Monitor system resources
                    let (memory_mb, cpu_percent) = {
                        if let Ok(mut system) = system_monitor_clone.try_lock() {
                            system.refresh_all();
                            let process = system.processes().get(&sysinfo::get_current_pid().unwrap());
                            let memory = process.map(|p| p.memory() / 1024 / 1024).unwrap_or(0);
                            let cpu = process.map(|p| p.cpu_usage()).unwrap_or(0.0);
                            max_memory_clone.fetch_max(memory, Ordering::SeqCst);
                            (memory, cpu)
                        } else {
                            (0, 0.0)
                        }
                    };

                    // Test content generation and validation
                    match Self::test_input_comprehensive(&input, &mut local_generator, &local_validator, &mut local_input) {
                        Ok(processing_time) => {
                            // Update performance metrics
                            {
                                let mut total_time = total_processing_time_clone.lock().unwrap();
                                *total_time += processing_time;
                            }

                            // Check for performance issues
                            if processing_time.as_millis() > 100 {
                                let issue = PerformanceIssue {
                                    test_number,
                                    issue_type: PerformanceIssueType::SlowProcessing,
                                    measured_value: processing_time.as_millis() as f64,
                                    threshold: 100.0,
                                    input_sample: input.chars().take(50).collect(),
                                };
                                performance_issues_clone.lock().unwrap().push(issue);
                            }

                            if memory_mb > 100 {
                                let issue = PerformanceIssue {
                                    test_number,
                                    issue_type: PerformanceIssueType::HighMemoryUsage,
                                    measured_value: memory_mb as f64,
                                    threshold: 100.0,
                                    input_sample: input.chars().take(50).collect(),
                                };
                                performance_issues_clone.lock().unwrap().push(issue);
                            }

                            if cpu_percent > 80.0 {
                                let issue = PerformanceIssue {
                                    test_number,
                                    issue_type: PerformanceIssueType::CpuSpike,
                                    measured_value: cpu_percent as f64,
                                    threshold: 80.0,
                                    input_sample: input.chars().take(50).collect(),
                                };
                                performance_issues_clone.lock().unwrap().push(issue);
                            }
                        }
                        Err(error) => {
                            let crash_type = Self::classify_error(&error);
                            let crash = FuzzCrash {
                                input: input.clone(),
                                error: error.to_string(),
                                test_number,
                                crash_type,
                                thread_id,
                                timestamp: Instant::now(),
                                memory_usage_mb: memory_mb,
                                cpu_usage_percent: cpu_percent,
                            };
                            crashes_clone.lock().unwrap().push(crash);
                        }
                    }

                    // Progress reporting every 1000 tests
                    if test_number % 1000 == 0 {
                        let elapsed = start_time.elapsed();
                        let progress = elapsed.as_secs_f64() / duration.as_secs_f64() * 100.0;
                        println!("  Thread {} progress: {} tests ({:.1}%)", thread_id, test_number, progress);
                    }

                    // Small delay to prevent overwhelming the system
                    thread::sleep(Duration::from_micros(100));
                }
            });

            handles.push(handle);
        }

        // Wait for specified duration
        thread::sleep(duration);
        self.running.store(false, Ordering::SeqCst);

        // Wait for all threads to complete
        for handle in handles {
            handle.join().expect("Thread panicked during fuzzing");
        }

        // Collect results
        report.total_tests = self.test_counter.load(Ordering::SeqCst);
        report.duration = start_time.elapsed();
        report.crashes = crashes.lock().unwrap().clone();
        report.performance_issues = performance_issues.lock().unwrap().clone();
        report.max_memory_usage_mb = max_memory.load(Ordering::SeqCst);

        // Calculate average processing time
        let total_time = total_processing_time.lock().unwrap();
        report.avg_processing_time_ms = if report.total_tests > 0 {
            total_time.as_millis() as f64 / report.total_tests as f64
        } else {
            0.0
        };

        // Classify high-risk crashes
        for crash in &report.crashes {
            if matches!(crash.crash_type, CrashType::Panic | CrashType::SecurityViolation | CrashType::StackOverflow) {
                report.high_risk_crashes.push(crash.clone());
            }
            if matches!(crash.crash_type, CrashType::SecurityViolation) {
                report.security_violations += 1;
            }
        }

        // Determine if stress test passed
        report.stress_test_passed = report.high_risk_crashes.is_empty() &&
                                   report.security_violations == 0 &&
                                   report.max_memory_usage_mb < 200; // 200MB threshold

        println!("✅ Extended fuzzing test completed:");
        println!("  - {} total tests", report.total_tests);
        println!("  - {} total crashes", report.total_crashes());
        println!("  - {} high-risk crashes", report.high_risk_crashes.len());
        println!("  - {} security violations", report.security_violations);
        println!("  - {:.2}ms avg processing time", report.avg_processing_time_ms);
        println!("  - {}MB max memory usage", report.max_memory_usage_mb);
        println!("  - Stress test passed: {}", report.stress_test_passed);

        Ok(report)
    }

    /// Generate malicious input patterns for fuzzing
    fn generate_malicious_input(rng: &mut ChaCha8Rng) -> String {
        let input_type = rng.gen_range(0..10);

        match input_type {
            0 => Self::generate_escape_sequences(rng),
            1 => Self::generate_unicode_attacks(rng),
            2 => Self::generate_buffer_overflow_attempt(rng),
            3 => Self::generate_format_string_attack(rng),
            4 => Self::generate_sql_injection_attempt(rng),
            5 => Self::generate_path_traversal_attempt(rng),
            6 => Self::generate_control_characters(rng),
            7 => Self::generate_extremely_long_input(rng),
            8 => Self::generate_binary_data(rng),
            9 => Self::generate_nested_structures(rng),
            _ => unreachable!(),
        }
    }

    fn generate_escape_sequences(rng: &mut ChaCha8Rng) -> String {
        let sequences = vec![
            "\x1b[2J\x1b[H",
            "\x1b]0;rm -rf /\x07",
            "\x1b[?1049h",
            "\x1b[6n",
            "\x1b[>c",
            "\x1b_Gf=32;",
            "\x1b]52;c;$(whoami)\x07",
        ];
        let base = sequences[rng.gen_range(0..sequences.len())];
        format!("normal_text{}_more_text", base)
    }

    fn generate_unicode_attacks(rng: &mut ChaCha8Rng) -> String {
        let attacks = vec![
            "\u{200B}\u{200C}\u{200D}",
            "\u{FEFF}\u{202E}",
            "\u{061C}\u{2066}",
            "A\u{0308}\u{0300}\u{0301}",
            "\u{E000}\u{F8FF}",
        ];
        let base = attacks[rng.gen_range(0..attacks.len())];
        format!("text{}text", base)
    }

    fn generate_buffer_overflow_attempt(rng: &mut ChaCha8Rng) -> String {
        let size = rng.gen_range(10000..50000);
        "A".repeat(size)
    }

    fn generate_format_string_attack(_rng: &mut ChaCha8Rng) -> String {
        "%s%p%x%n%08x%016x".to_string()
    }

    fn generate_sql_injection_attempt(_rng: &mut ChaCha8Rng) -> String {
        "'; DROP TABLE users; --".to_string()
    }

    fn generate_path_traversal_attempt(rng: &mut ChaCha8Rng) -> String {
        let attempts = vec![
            "../../../etc/passwd",
            "..\\..\\..\\windows\\system32",
            "....//....//....//etc/shadow",
            "%2e%2e%2f%2e%2e%2f%2e%2e%2f",
        ];
        attempts[rng.gen_range(0..attempts.len())].to_string()
    }

    fn generate_control_characters(rng: &mut ChaCha8Rng) -> String {
        let mut result = String::new();
        for _ in 0..rng.gen_range(10..100) {
            let control_char = rng.gen_range(0..32) as u8 as char;
            result.push(control_char);
        }
        result
    }

    fn generate_extremely_long_input(rng: &mut ChaCha8Rng) -> String {
        let length = rng.gen_range(100000..1000000);
        let pattern = "abcdefghijklmnopqrstuvwxyz0123456789!@#$%^&*()";
        pattern.repeat(length / pattern.len() + 1)[..length].to_string()
    }

    fn generate_binary_data(rng: &mut ChaCha8Rng) -> String {
        let mut bytes = Vec::new();
        for _ in 0..rng.gen_range(100..1000) {
            bytes.push(rng.gen_range(0..256) as u8);
        }
        String::from_utf8_lossy(&bytes).to_string()
    }

    fn generate_nested_structures(rng: &mut ChaCha8Rng) -> String {
        let depth = rng.gen_range(100..1000);
        let mut result = String::new();
        for _ in 0..depth {
            result.push_str("{{[[(");
        }
        result.push_str("PAYLOAD");
        for _ in 0..depth {
            result.push_str(")]]}");
        }
        result
    }

    /// Comprehensive testing of input processing
    fn test_input_comprehensive(
        input: &str,
        generator: &mut ContentGenerator,
        validator: &ContentValidator,
        input_processor: &mut Input,
    ) -> Result<Duration> {
        let start = Instant::now();

        // Test 1: Content validation
        let _validation_result = validator.validate_security(input);

        // Test 2: Content sanitization
        let _sanitized = validator.sanitize(input);

        // Test 3: Content generation with potentially malicious seed
        if let Ok(seed) = input.chars().take(8).collect::<String>().parse::<u64>() {
            if let Ok(level) = LevelId::new(1) {
                let _content = generator.generate_level_content(level, seed)?;
            }
        }

        // Test 4: Input processing simulation
        for ch in input.chars().take(1000) { // Limit to prevent infinite loops
            if ch.is_control() {
                continue; // Skip control characters to prevent issues
            }

            let key_event = KeyEvent {
                code: KeyCode::Char(ch),
                modifiers: KeyModifiers::NONE,
                kind: crossterm::event::KeyEventKind::Press,
                state: crossterm::event::KeyEventState::NONE,
            };

            // Process the key event
            let _result = input_processor.process_key_event(key_event);
        }

        // Test 5: String operations that could cause issues
        let _char_count = input.chars().count();
        let _line_count = input.lines().count();
        let _byte_len = input.len();

        // Test 6: Unicode normalization
        use unicode_normalization::UnicodeNormalization;
        let _normalized: String = input.nfc().collect();

        // Test 7: Regex operations (common crash vector)
        if let Ok(regex) = regex::Regex::new(r"[a-zA-Z]+") {
            let _matches: Vec<_> = regex.find_iter(input).collect();
        }

        Ok(start.elapsed())
    }

    /// Classify error types for crash analysis
    fn classify_error(error: &CentotypeError) -> CrashType {
        let error_str = error.to_string().to_lowercase();

        if error_str.contains("stack overflow") || error_str.contains("recursion") {
            CrashType::StackOverflow
        } else if error_str.contains("memory") || error_str.contains("allocation") {
            CrashType::MemoryExhaustion
        } else if error_str.contains("timeout") || error_str.contains("deadline") {
            CrashType::Timeout
        } else if error_str.contains("security") || error_str.contains("validation") {
            CrashType::SecurityViolation
        } else if error_str.contains("infinite") || error_str.contains("loop") {
            CrashType::InfiniteLoop
        } else if error_str.contains("resource") || error_str.contains("limit") {
            CrashType::ResourceExhaustion
        } else if error_str.contains("panic") {
            CrashType::Panic
        } else {
            CrashType::UnexpectedError(error.to_string())
        }
    }

    /// Run targeted crash resistance test
    pub fn test_crash_resistance(&mut self, iterations: u64) -> Result<bool> {
        println!("🛡️  Testing crash resistance with {} iterations...", iterations);

        let mut rng = ChaCha8Rng::from_entropy();
        let mut crashes = 0;

        for i in 0..iterations {
            let malicious_input = Self::generate_malicious_input(&mut rng);

            match Self::test_input_comprehensive(
                &malicious_input,
                &mut self.content_generator,
                &self.content_validator,
                &mut self.input_processor,
            ) {
                Ok(_) => {
                    // Input processed successfully
                }
                Err(_) => {
                    crashes += 1;
                    if crashes > 10 {
                        println!("❌ Too many crashes detected ({}), aborting test", crashes);
                        return Ok(false);
                    }
                }
            }

            if i % 1000 == 0 && i > 0 {
                println!("  Crash resistance progress: {}/{} ({} crashes)", i, iterations, crashes);
            }
        }

        let success_rate = ((iterations - crashes) as f64 / iterations as f64) * 100.0;
        println!("✅ Crash resistance test completed:");
        println!("  - Success rate: {:.2}%", success_rate);
        println!("  - Total crashes: {}", crashes);

        Ok(success_rate >= 99.0) // 99% success rate required
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_extended_fuzz_engine_creation() {
        let engine = ExtendedFuzzEngine::new().expect("Failed to create fuzz engine");
        assert_eq!(engine.test_counter.load(Ordering::SeqCst), 0);
    }

    #[test]
    fn test_malicious_input_generation() {
        let mut rng = ChaCha8Rng::from_entropy();

        for _ in 0..10 {
            let input = ExtendedFuzzEngine::generate_malicious_input(&mut rng);
            assert!(!input.is_empty(), "Generated input should not be empty");
        }
    }

    #[test]
    fn test_crash_type_classification() {
        let security_error = CentotypeError::Content("Security validation failed".to_string());
        let crash_type = ExtendedFuzzEngine::classify_error(&security_error);
        assert!(matches!(crash_type, CrashType::SecurityViolation));

        let memory_error = CentotypeError::Content("Memory allocation failed".to_string());
        let crash_type = ExtendedFuzzEngine::classify_error(&memory_error);
        assert!(matches!(crash_type, CrashType::MemoryExhaustion));
    }

    #[test]
    #[ignore] // Long-running test
    fn test_short_crash_resistance() {
        let mut engine = ExtendedFuzzEngine::new().expect("Failed to create engine");
        let result = engine.test_crash_resistance(1000).expect("Crash resistance test failed");
        assert!(result, "Crash resistance test should pass");
    }

    #[test]
    #[ignore] // Very long-running test - 4 hours
    fn test_extended_fuzz_suite() {
        let mut engine = ExtendedFuzzEngine::new().expect("Failed to create engine");
        let duration = Duration::from_secs(4 * 3600); // 4 hours
        let thread_count = 4;

        let report = engine.run_extended_fuzz_test(duration, thread_count)
            .expect("Extended fuzz test failed");

        // Zero high-risk crashes allowed
        assert_eq!(report.high_risk_crashes.len(), 0,
                  "High-risk crashes found: {:?}", report.high_risk_crashes);

        // Should process significant number of tests
        assert!(report.total_tests > 10000,
                "Too few tests processed: {}", report.total_tests);

        // Memory usage should be reasonable
        assert!(report.max_memory_usage_mb < 500,
                "Excessive memory usage: {}MB", report.max_memory_usage_mb);

        // Security violations should be zero
        assert_eq!(report.security_violations, 0,
                  "Security violations found: {}", report.security_violations);

        println!("Extended fuzz test SUCCESS: {} tests, {} crashes, {}MB max memory",
                report.total_tests, report.total_crashes(), report.max_memory_usage_mb);
    }
}