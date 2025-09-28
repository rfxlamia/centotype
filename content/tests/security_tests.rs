//! Comprehensive security test suite for Centotype
//! Tests all critical security vectors including escape sequences, input sanitization,
//! file system permissions, and fuzzing resistance.

use centotype_content::{ContentGenerator, ContentValidator};
use centotype_core::types::*;
use std::time::{Duration, Instant};
use rand::{Rng, SeedableRng};
use rand_chacha::ChaCha8Rng;
use std::fs;
use std::path::Path;

/// Security violation types for reporting
#[derive(Debug, Clone)]
pub enum SecurityViolation {
    EscapeSequence {
        level_id: u32,
        content: String,
        sequence: String,
    },
    ShellInjection {
        level_id: u32,
        pattern: String,
        content: String,
    },
    ControlCharacter {
        level_id: u32,
        character: char,
        content: String,
    },
    UnicodeAnomaly {
        level_id: u32,
        description: String,
        content: String,
    },
    ContentIntegrity {
        level_id: u32,
        issue: String,
    },
}

/// Security audit report
#[derive(Debug)]
pub struct SecurityReport {
    pub violations: Vec<SecurityViolation>,
    pub high_risk_count: usize,
    pub medium_risk_count: usize,
    pub low_risk_count: usize,
    pub total_content_tested: usize,
    pub test_duration: Duration,
}

impl SecurityReport {
    pub fn new() -> Self {
        Self {
            violations: Vec::new(),
            high_risk_count: 0,
            medium_risk_count: 0,
            low_risk_count: 0,
            total_content_tested: 0,
            test_duration: Duration::ZERO,
        }
    }

    pub fn add_violation(&mut self, violation: SecurityViolation) {
        // Classify risk level
        match &violation {
            SecurityViolation::EscapeSequence { .. } => self.high_risk_count += 1,
            SecurityViolation::ShellInjection { .. } => self.high_risk_count += 1,
            SecurityViolation::ControlCharacter { .. } => self.medium_risk_count += 1,
            SecurityViolation::UnicodeAnomaly { .. } => self.low_risk_count += 1,
            SecurityViolation::ContentIntegrity { .. } => self.medium_risk_count += 1,
        }
        self.violations.push(violation);
    }

    pub fn high_risk_violations(&self) -> Vec<&SecurityViolation> {
        self.violations.iter().filter(|v| matches!(v,
            SecurityViolation::EscapeSequence { .. } |
            SecurityViolation::ShellInjection { .. }
        )).collect()
    }

    pub fn is_secure(&self) -> bool {
        self.high_risk_count == 0
    }
}

/// Fuzz test crash report
#[derive(Debug)]
pub struct FuzzCrash {
    pub input: String,
    pub error: String,
    pub test_number: u64,
    pub crash_type: CrashType,
}

#[derive(Debug)]
pub enum CrashType {
    Panic,
    Timeout,
    MemoryExhaustion,
    InfiniteLoop,
    SecurityViolation,
}

/// Fuzz testing report
#[derive(Debug)]
pub struct FuzzReport {
    pub total_tests: u64,
    pub duration: Duration,
    pub crashes: Vec<FuzzCrash>,
    pub high_risk_crashes: Vec<FuzzCrash>,
    pub performance_issues: Vec<String>,
}

impl FuzzReport {
    pub fn new() -> Self {
        Self {
            total_tests: 0,
            duration: Duration::ZERO,
            crashes: Vec::new(),
            high_risk_crashes: Vec::new(),
            performance_issues: Vec::new(),
        }
    }

    pub fn add_crash(&mut self, crash: FuzzCrash) {
        let is_high_risk = matches!(crash.crash_type,
            CrashType::Panic | CrashType::SecurityViolation
        );

        if is_high_risk {
            self.high_risk_crashes.push(crash.clone());
        }
        self.crashes.push(crash);
    }

    pub fn total_crashes(&self) -> usize {
        self.crashes.len()
    }
}

/// File system permission audit
#[derive(Debug)]
pub struct PermissionReport {
    pub issues: Vec<PermissionIssue>,
    pub critical_issues: Vec<PermissionIssue>,
    pub paths_tested: Vec<String>,
}

#[derive(Debug)]
pub enum PermissionIssue {
    WorldReadable { path: String, mode: u32 },
    GroupWritable { path: String, mode: u32 },
    PathTraversal { attempted_path: String },
    ExcessivePermissions { path: String, mode: u32 },
    ConfigFileUnsecured { path: String },
}

impl PermissionReport {
    pub fn new() -> Self {
        Self {
            issues: Vec::new(),
            critical_issues: Vec::new(),
            paths_tested: Vec::new(),
        }
    }

    pub fn add_issue(&mut self, issue: PermissionIssue) {
        let is_critical = matches!(issue,
            PermissionIssue::PathTraversal { .. } |
            PermissionIssue::ExcessivePermissions { .. }
        );

        if is_critical {
            self.critical_issues.push(issue.clone());
        }
        self.issues.push(issue);
    }

    pub fn critical_issues(&self) -> &[PermissionIssue] {
        &self.critical_issues
    }
}

/// Main security validator for comprehensive testing
pub struct SecurityAuditor {
    validator: ContentValidator,
    generator: ContentGenerator,
}

impl SecurityAuditor {
    pub fn new() -> Result<Self> {
        Ok(Self {
            validator: ContentValidator::new()?,
            generator: ContentGenerator::new()?,
        })
    }

    /// Comprehensive security audit of all content generation
    pub fn audit_content_security(&mut self) -> Result<SecurityReport> {
        let start_time = Instant::now();
        let mut report = SecurityReport::new();

        println!("🔍 Starting comprehensive content security audit...");

        // Test all 100 levels for security violations
        for level_id in 1..=100 {
            let level = LevelId::new(level_id)?;

            // Generate content with deterministic seed for reproducible testing
            let content = self.generator.generate_level_content(level, 12345)?;
            report.total_content_tested += 1;

            // Test 1: Escape sequence detection
            self.check_escape_sequences(&content, level_id, &mut report);

            // Test 2: Shell injection patterns
            self.check_shell_injection(&content, level_id, &mut report);

            // Test 3: Control character detection
            self.check_control_characters(&content, level_id, &mut report);

            // Test 4: Unicode anomalies
            self.check_unicode_anomalies(&content, level_id, &mut report);

            // Test 5: Content integrity
            self.check_content_integrity(&content, level, &mut report);

            if level_id % 10 == 0 {
                println!("  Completed level {} security audit", level_id);
            }
        }

        report.test_duration = start_time.elapsed();

        println!("✅ Content security audit completed:");
        println!("  - {} levels tested", report.total_content_tested);
        println!("  - {} high-risk violations", report.high_risk_count);
        println!("  - {} medium-risk violations", report.medium_risk_count);
        println!("  - {} low-risk violations", report.low_risk_count);
        println!("  - Duration: {:?}", report.test_duration);

        Ok(report)
    }

    /// Test for terminal escape sequences
    fn check_escape_sequences(&self, content: &str, level_id: u32, report: &mut SecurityReport) {
        let dangerous_sequences = vec![
            "\x1b]0;",          // Terminal title manipulation
            "\x1b[2J",          // Clear screen
            "\x1b[H",           // Cursor home
            "\x1b[?1049h",      // Alt screen buffer
            "\x1b[?25l",        // Hide cursor
            "\x1b[6n",          // Device status report
            "\x1b[>c",          // Device attributes
            "\x1b]52;",         // Clipboard manipulation
            "\x1b_G",           // Graphics protocol
            "\x07",             // BEL character
            "\x0c",             // Form feed
        ];

        for sequence in &dangerous_sequences {
            if content.contains(sequence) {
                report.add_violation(SecurityViolation::EscapeSequence {
                    level_id,
                    content: content.clone(),
                    sequence: sequence.to_string(),
                });
            }
        }

        // Check for any ESC character
        if content.contains('\x1b') {
            report.add_violation(SecurityViolation::EscapeSequence {
                level_id,
                content: content.clone(),
                sequence: "Generic ESC character".to_string(),
            });
        }
    }

    /// Test for shell injection patterns
    fn check_shell_injection(&self, content: &str, level_id: u32, report: &mut SecurityReport) {
        let injection_patterns = vec![
            "$(", "`", "&&", "||", ";", " | ",
            "rm ", "cat ", "curl ", "wget ", "chmod ",
            "sudo ", "su ", "exec ", "eval ",
            "/bin/", "/usr/bin/", "cmd.exe", "powershell",
        ];

        for pattern in &injection_patterns {
            if content.contains(pattern) {
                report.add_violation(SecurityViolation::ShellInjection {
                    level_id,
                    pattern: pattern.to_string(),
                    content: content.clone(),
                });
            }
        }
    }

    /// Test for dangerous control characters
    fn check_control_characters(&self, content: &str, level_id: u32, report: &mut SecurityReport) {
        for ch in content.chars() {
            if ch.is_control() && ch != '\n' && ch != '\r' && ch != '\t' {
                report.add_violation(SecurityViolation::ControlCharacter {
                    level_id,
                    character: ch,
                    content: content.clone(),
                });
            }
        }

        // Check for null bytes specifically
        if content.contains('\0') {
            report.add_violation(SecurityViolation::ControlCharacter {
                level_id,
                character: '\0',
                content: content.clone(),
            });
        }
    }

    /// Test for Unicode anomalies and suspicious patterns
    fn check_unicode_anomalies(&self, content: &str, level_id: u32, report: &mut SecurityReport) {
        // Check for suspicious Unicode characters
        let suspicious_chars = vec![
            '\u{200B}', // Zero-width space
            '\u{FEFF}', // Byte order mark
            '\u{202E}', // Right-to-left override
            '\u{2066}', // Left-to-right isolate
            '\u{061C}', // Arabic letter mark
            '\u{200D}', // Zero-width joiner
            '\u{200C}', // Zero-width non-joiner
        ];

        for &suspicious_char in &suspicious_chars {
            if content.contains(suspicious_char) {
                report.add_violation(SecurityViolation::UnicodeAnomaly {
                    level_id,
                    description: format!("Suspicious Unicode character: U+{:04X}", suspicious_char as u32),
                    content: content.clone(),
                });
            }
        }

        // Check for characters in private use areas
        for ch in content.chars() {
            let code_point = ch as u32;
            if (0xE000..=0xF8FF).contains(&code_point) ||  // Private Use Area
               (0xF0000..=0xFFFFD).contains(&code_point) ||  // Supplementary Private Use Area-A
               (0x100000..=0x10FFFD).contains(&code_point) { // Supplementary Private Use Area-B
                report.add_violation(SecurityViolation::UnicodeAnomaly {
                    level_id,
                    description: format!("Private use area character: U+{:04X}", code_point),
                    content: content.clone(),
                });
            }
        }
    }

    /// Test content integrity and validation compliance
    fn check_content_integrity(&self, content: &str, level: LevelId, report: &mut SecurityReport) {
        // Use the existing validator to check security compliance
        match self.validator.validate_security(content) {
            validation if !validation.is_valid() => {
                report.add_violation(SecurityViolation::ContentIntegrity {
                    level_id: level.0,
                    issue: validation.error_message().unwrap_or("Unknown validation error").to_string(),
                });
            }
            _ => {} // Content passed validation
        }

        // Additional integrity checks
        if content.is_empty() {
            report.add_violation(SecurityViolation::ContentIntegrity {
                level_id: level.0,
                issue: "Empty content generated".to_string(),
            });
        }

        if content.len() > 10_000 {
            report.add_violation(SecurityViolation::ContentIntegrity {
                level_id: level.0,
                issue: format!("Excessive content length: {}", content.len()),
            });
        }
    }

    /// Extended fuzzing test for crash resistance
    pub fn fuzz_input_handling(&mut self, duration: Duration) -> Result<FuzzReport> {
        let mut report = FuzzReport::new();
        let mut rng = ChaCha8Rng::from_entropy();
        let end_time = Instant::now() + duration;

        println!("🚀 Starting extended fuzz testing (duration: {:?})...", duration);
        let mut test_count = 0;

        while Instant::now() < end_time {
            // Generate random input sequences with various attack patterns
            let input = self.generate_fuzz_input(&mut rng);

            // Test input processing without crashing
            match self.test_input_processing(&input) {
                Ok(_) => {
                    // Input processed successfully
                }
                Err(e) => {
                    let crash_type = self.classify_error(&e);
                    report.add_crash(FuzzCrash {
                        input,
                        error: e.to_string(),
                        test_number: test_count,
                        crash_type,
                    });
                }
            }

            test_count += 1;

            if test_count % 1000 == 0 {
                let elapsed = Instant::now().duration_since(end_time - duration);
                let progress = elapsed.as_secs_f64() / duration.as_secs_f64() * 100.0;
                println!("  Fuzz progress: {} tests ({:.1}%)", test_count, progress);
            }
        }

        report.total_tests = test_count;
        report.duration = duration;

        println!("✅ Fuzz testing completed:");
        println!("  - {} total tests", report.total_tests);
        println!("  - {} total crashes", report.total_crashes());
        println!("  - {} high-risk crashes", report.high_risk_crashes.len());
        println!("  - Duration: {:?}", report.duration);

        Ok(report)
    }

    /// Generate malicious input for fuzzing
    fn generate_fuzz_input(&self, rng: &mut ChaCha8Rng) -> String {
        let input_length = rng.gen_range(1..=1000);
        let mut input_sequence = Vec::with_capacity(input_length);

        for _ in 0..input_length {
            let char_class = rng.gen_range(0..20);
            let ch = match char_class {
                0..=10 => rng.gen_range(b' '..=b'~') as char, // Printable ASCII
                11..=12 => rng.gen_range(0..=31) as char,     // Control characters
                13 => '\x1b',                                  // Escape character
                14 => '\0',                                    // Null character
                15 => '\x7f',                                  // DEL character
                16 => char::from_u32(rng.gen_range(128..=255)).unwrap_or('\0'), // Extended ASCII
                17 => char::from_u32(rng.gen_range(0x1000..=0x1FFFF)).unwrap_or('\0'), // Unicode
                18 => char::from_u32(rng.gen_range(0xE000..=0xF8FF)).unwrap_or('\0'), // Private use
                19 => '\u{FEFF}', // BOM
                _ => unreachable!(),
            };
            input_sequence.push(ch);
        }

        input_sequence.into_iter().collect()
    }

    /// Test input processing for crashes
    fn test_input_processing(&self, input: &str) -> Result<()> {
        // Test security validation
        let _validation_result = self.validator.validate_security(input);

        // Test sanitization
        let _sanitized = self.validator.sanitize(input);

        // Test string operations that could cause issues
        let _char_count = input.chars().count();
        let _line_count = input.lines().count();
        let _byte_len = input.len();

        // Test Unicode normalization
        use unicode_normalization::UnicodeNormalization;
        let _normalized: String = input.nfc().collect();

        // Test regex operations (common crash vector)
        if let Ok(regex) = regex::Regex::new(r"[a-zA-Z]+") {
            let _matches: Vec<_> = regex.find_iter(input).collect();
        }

        Ok(())
    }

    /// Classify error types for crash analysis
    fn classify_error(&self, error: &CentotypeError) -> CrashType {
        match error {
            CentotypeError::Content(msg) if msg.contains("validation") => CrashType::SecurityViolation,
            CentotypeError::Content(msg) if msg.contains("timeout") => CrashType::Timeout,
            CentotypeError::Content(msg) if msg.contains("memory") => CrashType::MemoryExhaustion,
            _ => CrashType::Panic,
        }
    }
}

/// File system permission auditing
pub struct FileSystemAuditor;

impl FileSystemAuditor {
    /// Audit file permissions and path traversal protection
    pub fn audit_file_permissions() -> Result<PermissionReport> {
        let mut report = PermissionReport::new();

        println!("🔒 Starting file system permission audit...");

        // Test configuration file permissions
        let config_paths = vec![
            "/home/v/.config/centotype/config.toml",
            "/home/v/.centotype.toml",
            "./config.toml",
            "./centotype.toml",
        ];

        for path_str in &config_paths {
            report.paths_tested.push(path_str.to_string());

            if let Ok(metadata) = fs::metadata(path_str) {
                let permissions = metadata.permissions();

                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let mode = permissions.mode();

                    // Check for world-readable files (security risk)
                    if mode & 0o004 != 0 {
                        report.add_issue(PermissionIssue::WorldReadable {
                            path: path_str.to_string(),
                            mode,
                        });
                    }

                    // Check for group-writable files
                    if mode & 0o020 != 0 {
                        report.add_issue(PermissionIssue::GroupWritable {
                            path: path_str.to_string(),
                            mode,
                        });
                    }

                    // Check for overly permissive files
                    if mode & 0o777 == 0o777 {
                        report.add_issue(PermissionIssue::ExcessivePermissions {
                            path: path_str.to_string(),
                            mode,
                        });
                    }
                }
            }
        }

        // Test path traversal protection
        let malicious_paths = vec![
            "../../../etc/passwd",
            "..\\..\\..\\windows\\system32\\config\\sam",
            "/etc/shadow",
            "~/.ssh/id_rsa",
            "/dev/null",
            "/proc/self/mem",
            "C:\\Windows\\System32\\drivers\\etc\\hosts",
            "file:///etc/passwd",
            "\\\\server\\share\\file.txt",
        ];

        for path in &malicious_paths {
            // Test if the application would try to access these paths
            // In a real implementation, this would test the actual path resolution logic
            if Path::new(path).is_absolute() && !path.starts_with("/tmp") {
                report.add_issue(PermissionIssue::PathTraversal {
                    attempted_path: path.to_string(),
                });
            }
        }

        println!("✅ File system audit completed:");
        println!("  - {} paths tested", report.paths_tested.len());
        println!("  - {} total issues", report.issues.len());
        println!("  - {} critical issues", report.critical_issues.len());

        Ok(report)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_comprehensive_security_suite() {
        let mut auditor = SecurityAuditor::new().expect("Failed to create security auditor");

        // 1. Content security audit
        let content_report = auditor.audit_content_security().expect("Content audit failed");
        assert_eq!(content_report.high_risk_violations().len(), 0,
                  "High-risk content violations found: {:?}", content_report.high_risk_violations());

        // 2. File system permission audit
        let perm_report = FileSystemAuditor::audit_file_permissions().expect("Permission audit failed");
        assert_eq!(perm_report.critical_issues().len(), 0,
                  "Critical permission issues found: {:?}", perm_report.critical_issues());
    }

    #[test]
    fn test_escape_sequence_detection() {
        let mut auditor = SecurityAuditor::new().expect("Failed to create auditor");
        let mut report = SecurityReport::new();

        // Test malicious escape sequences
        let malicious_content = "Hello \x1b[31mworld\x1b[0m";
        auditor.check_escape_sequences(malicious_content, 1, &mut report);

        assert!(report.high_risk_count > 0, "Escape sequences not detected");
    }

    #[test]
    fn test_shell_injection_detection() {
        let mut auditor = SecurityAuditor::new().expect("Failed to create auditor");
        let mut report = SecurityReport::new();

        // Test shell injection patterns
        let malicious_content = "$(rm -rf /) && echo 'pwned'";
        auditor.check_shell_injection(malicious_content, 1, &mut report);

        assert!(report.high_risk_count > 0, "Shell injection not detected");
    }

    #[test]
    fn test_unicode_anomaly_detection() {
        let mut auditor = SecurityAuditor::new().expect("Failed to create auditor");
        let mut report = SecurityReport::new();

        // Test suspicious Unicode
        let malicious_content = "Hello\u{200B}World\u{FEFF}";
        auditor.check_unicode_anomalies(malicious_content, 1, &mut report);

        assert!(report.low_risk_count > 0, "Unicode anomalies not detected");
    }

    #[test]
    #[ignore] // Long-running test - run with: cargo test test_short_fuzz_suite -- --ignored
    fn test_short_fuzz_suite() {
        let mut auditor = SecurityAuditor::new().expect("Failed to create auditor");
        let duration = Duration::from_secs(30); // 30 second test

        let fuzz_report = auditor.fuzz_input_handling(duration).expect("Fuzz test failed");

        // Should have no high-risk crashes
        assert_eq!(fuzz_report.high_risk_crashes.len(), 0,
                  "High-risk crashes found: {:?}", fuzz_report.high_risk_crashes);

        // Should process reasonable number of tests
        assert!(fuzz_report.total_tests > 1000,
                "Too few tests processed: {}", fuzz_report.total_tests);
    }

    #[test]
    #[ignore] // Very long-running test - run with: cargo test test_extended_fuzz_suite -- --ignored
    fn test_extended_fuzz_suite() {
        let mut auditor = SecurityAuditor::new().expect("Failed to create auditor");
        let duration = Duration::from_secs(4 * 3600); // 4 hours

        let fuzz_report = auditor.fuzz_input_handling(duration).expect("Fuzz test failed");

        // Zero high-risk crashes allowed
        assert_eq!(fuzz_report.high_risk_crashes.len(), 0,
                  "High-risk crashes found: {:?}", fuzz_report.high_risk_crashes);

        println!("Extended fuzz test completed: {} tests, {} crashes",
                fuzz_report.total_tests, fuzz_report.total_crashes());
    }

    #[test]
    fn test_content_sanitization() {
        let auditor = SecurityAuditor::new().expect("Failed to create auditor");

        let dirty_content = "\x1b[31mRed text\x1b[0m with \0null bytes";
        let clean_content = auditor.validator.sanitize(dirty_content);

        assert!(!clean_content.contains('\x1b'), "Escape sequences not sanitized");
        assert!(!clean_content.contains('\0'), "Null bytes not sanitized");
        assert!(auditor.validator.validate_security(&clean_content).is_valid(),
                "Sanitized content still fails validation");
    }

    #[test]
    fn test_malicious_patterns() {
        let mut auditor = SecurityAuditor::new().expect("Failed to create auditor");

        let malicious_inputs = vec![
            "\x1b]0;rm -rf /\x07",           // Terminal title injection
            "\x1b[2J\x1b[H\x1b[3J",         // Screen clearing
            "\x1b[?1049h",                   // Alt screen manipulation
            "\x1b[?25l",                     // Cursor hiding
            "\x1b[6n",                       // Cursor position request
            "\x1b[>c",                       // Device attributes request
            "hello\x1b[A\x1b[2Kworld",      // Cursor manipulation
            "$(whoami)",                     // Command substitution
            "`cat /etc/passwd`",             // Backtick command execution
            "test && rm -rf /",              // Command chaining
            "/bin/sh -c 'echo pwned'",       // Direct shell execution
        ];

        for (i, input) in malicious_inputs.iter().enumerate() {
            let mut report = SecurityReport::new();

            auditor.check_escape_sequences(input, i as u32, &mut report);
            auditor.check_shell_injection(input, i as u32, &mut report);
            auditor.check_control_characters(input, i as u32, &mut report);

            assert!(report.high_risk_count > 0 || report.medium_risk_count > 0,
                    "Malicious pattern not detected: {:?}", input);
        }
    }
}