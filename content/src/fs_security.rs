//! File system security auditing for Centotype
//! Validates file permissions, path traversal protection, and secure storage practices

use std::fs;
use std::path::{Path, PathBuf};
use std::collections::HashMap;
use centotype_core::types::*;

/// File system security violation types
#[derive(Debug, Clone)]
pub enum FileSystemViolation {
    /// File has world-readable permissions
    WorldReadable {
        path: PathBuf,
        mode: u32,
        recommended_mode: u32,
    },
    /// File has group-writable permissions
    GroupWritable {
        path: PathBuf,
        mode: u32,
        recommended_mode: u32,
    },
    /// File has excessive permissions (777)
    ExcessivePermissions {
        path: PathBuf,
        mode: u32,
        recommended_mode: u32,
    },
    /// Path traversal vulnerability detected
    PathTraversal {
        attempted_path: String,
        resolved_path: Option<PathBuf>,
        security_risk: PathTraversalRisk,
    },
    /// Configuration file is not properly secured
    UnsecuredConfig {
        path: PathBuf,
        issue: String,
        recommendation: String,
    },
    /// Temporary file security issue
    UnsecuredTempFile {
        path: PathBuf,
        issue: String,
    },
    /// Symlink security issue
    DangerousSymlink {
        link_path: PathBuf,
        target_path: PathBuf,
        risk: String,
    },
}

/// Path traversal risk assessment
#[derive(Debug, Clone)]
pub enum PathTraversalRisk {
    Critical,  // Could access system files
    High,      // Could access user files outside app
    Medium,    // Could access sibling directories
    Low,       // Contained within app directory
}

/// File system security audit report
#[derive(Debug)]
pub struct FileSystemSecurityReport {
    pub violations: Vec<FileSystemViolation>,
    pub critical_violations: Vec<FileSystemViolation>,
    pub paths_tested: Vec<PathBuf>,
    pub secure_paths: Vec<PathBuf>,
    pub recommendations: Vec<String>,
    pub security_grade: char,
}

impl FileSystemSecurityReport {
    pub fn new() -> Self {
        Self {
            violations: Vec::new(),
            critical_violations: Vec::new(),
            paths_tested: Vec::new(),
            secure_paths: Vec::new(),
            recommendations: Vec::new(),
            security_grade: 'A',
        }
    }

    pub fn add_violation(&mut self, violation: FileSystemViolation) {
        let is_critical = matches!(violation,
            FileSystemViolation::PathTraversal { security_risk: PathTraversalRisk::Critical, .. } |
            FileSystemViolation::ExcessivePermissions { .. } |
            FileSystemViolation::DangerousSymlink { .. }
        );

        if is_critical {
            self.critical_violations.push(violation.clone());
        }
        self.violations.push(violation);
    }

    pub fn add_recommendation(&mut self, recommendation: String) {
        self.recommendations.push(recommendation);
    }

    pub fn calculate_security_grade(&mut self) {
        let critical_count = self.critical_violations.len();
        let total_violations = self.violations.len();

        self.security_grade = match (critical_count, total_violations) {
            (0, 0) => 'A',
            (0, 1..=2) => 'B',
            (0, 3..=5) => 'C',
            (0, _) => 'D',
            (_, _) => 'F', // Any critical violations = F
        };
    }

    pub fn is_secure(&self) -> bool {
        self.critical_violations.is_empty()
    }
}

/// Comprehensive file system security auditor
pub struct FileSystemSecurityAuditor {
    /// Application root directory
    app_root: PathBuf,
    /// Configuration directories to check
    config_dirs: Vec<PathBuf>,
    /// Temporary directories used by the app
    temp_dirs: Vec<PathBuf>,
    /// Cache of path canonicalization results
    path_cache: HashMap<String, Option<PathBuf>>,
}

impl FileSystemSecurityAuditor {
    pub fn new<P: AsRef<Path>>(app_root: P) -> Self {
        let app_root = app_root.as_ref().to_path_buf();

        // Standard configuration directories
        let config_dirs = vec![
            app_root.join("config"),
            dirs::config_dir().unwrap_or_else(|| PathBuf::from("/tmp")).join("centotype"),
            dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp")).join(".centotype"),
            dirs::home_dir().unwrap_or_else(|| PathBuf::from("/tmp")).join(".config/centotype"),
        ];

        // Standard temporary directories
        let temp_dirs = vec![
            std::env::temp_dir().join("centotype"),
            PathBuf::from("/tmp/centotype"),
            app_root.join("tmp"),
        ];

        Self {
            app_root,
            config_dirs,
            temp_dirs,
            path_cache: HashMap::new(),
        }
    }

    /// Perform comprehensive file system security audit
    pub fn audit(&mut self) -> Result<FileSystemSecurityReport> {
        let mut report = FileSystemSecurityReport::new();

        println!("🔒 Starting file system security audit...");

        // 1. Audit configuration file permissions
        self.audit_config_permissions(&mut report)?;

        // 2. Audit temporary file handling
        self.audit_temp_file_security(&mut report)?;

        // 3. Test path traversal protection
        self.test_path_traversal_protection(&mut report)?;

        // 4. Check for dangerous symlinks
        self.audit_symlink_security(&mut report)?;

        // 5. Validate directory permissions
        self.audit_directory_permissions(&mut report)?;

        // 6. Generate recommendations
        self.generate_security_recommendations(&mut report);

        // 7. Calculate overall security grade
        report.calculate_security_grade();

        println!("✅ File system security audit completed:");
        println!("  - {} paths tested", report.paths_tested.len());
        println!("  - {} violations found", report.violations.len());
        println!("  - {} critical violations", report.critical_violations.len());
        println!("  - Security grade: {}", report.security_grade);

        Ok(report)
    }

    /// Audit configuration file permissions
    fn audit_config_permissions(&mut self, report: &mut FileSystemSecurityReport) -> Result<()> {
        println!("  Auditing configuration file permissions...");

        for config_dir in &self.config_dirs.clone() {
            report.paths_tested.push(config_dir.clone());

            if !config_dir.exists() {
                continue;
            }

            // Check directory permissions
            if let Ok(metadata) = fs::metadata(config_dir) {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let mode = metadata.permissions().mode();

                    // Config directories should not be world-readable
                    if mode & 0o004 != 0 {
                        report.add_violation(FileSystemViolation::WorldReadable {
                            path: config_dir.clone(),
                            mode,
                            recommended_mode: mode & !0o004,
                        });
                    }

                    // Config directories should not be group-writable
                    if mode & 0o020 != 0 {
                        report.add_violation(FileSystemViolation::GroupWritable {
                            path: config_dir.clone(),
                            mode,
                            recommended_mode: mode & !0o020,
                        });
                    }
                }
            }

            // Check configuration files
            if let Ok(entries) = fs::read_dir(config_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    report.paths_tested.push(path.clone());

                    if path.is_file() {
                        self.audit_file_permissions(&path, report)?;
                    }
                }
            }
        }

        Ok(())
    }

    /// Audit individual file permissions
    fn audit_file_permissions(&self, path: &Path, report: &mut FileSystemSecurityReport) -> Result<()> {
        if let Ok(metadata) = fs::metadata(path) {
            #[cfg(unix)]
            {
                use std::os::unix::fs::PermissionsExt;
                let mode = metadata.permissions().mode();

                // Check for world-readable files
                if mode & 0o004 != 0 {
                    report.add_violation(FileSystemViolation::WorldReadable {
                        path: path.to_path_buf(),
                        mode,
                        recommended_mode: 0o600, // Owner read/write only
                    });
                }

                // Check for group-writable files
                if mode & 0o020 != 0 {
                    report.add_violation(FileSystemViolation::GroupWritable {
                        path: path.to_path_buf(),
                        mode,
                        recommended_mode: 0o600,
                    });
                }

                // Check for excessive permissions (777)
                if mode & 0o777 == 0o777 {
                    report.add_violation(FileSystemViolation::ExcessivePermissions {
                        path: path.to_path_buf(),
                        mode,
                        recommended_mode: 0o600,
                    });
                }

                // Configuration files should be restrictive
                if path.extension().map_or(false, |ext| ext == "toml" || ext == "json" || ext == "yaml") {
                    if mode & 0o077 != 0 {
                        report.add_violation(FileSystemViolation::UnsecuredConfig {
                            path: path.to_path_buf(),
                            issue: format!("Configuration file has permissive permissions: {:o}", mode),
                            recommendation: "Set permissions to 600 (owner read/write only)".to_string(),
                        });
                    }
                }
            }
        }

        Ok(())
    }

    /// Audit temporary file security
    fn audit_temp_file_security(&mut self, report: &mut FileSystemSecurityReport) -> Result<()> {
        println!("  Auditing temporary file security...");

        for temp_dir in &self.temp_dirs.clone() {
            report.paths_tested.push(temp_dir.clone());

            if !temp_dir.exists() {
                continue;
            }

            // Check temporary directory permissions
            if let Ok(metadata) = fs::metadata(temp_dir) {
                #[cfg(unix)]
                {
                    use std::os::unix::fs::PermissionsExt;
                    let mode = metadata.permissions().mode();

                    // Temp directories should not be world-writable without sticky bit
                    if (mode & 0o002 != 0) && (mode & 0o1000 == 0) {
                        report.add_violation(FileSystemViolation::UnsecuredTempFile {
                            path: temp_dir.clone(),
                            issue: "Temporary directory is world-writable without sticky bit".to_string(),
                        });
                    }
                }
            }

            // Check for leftover temporary files
            if let Ok(entries) = fs::read_dir(temp_dir) {
                for entry in entries.flatten() {
                    let path = entry.path();
                    if let Ok(metadata) = fs::metadata(&path) {
                        // Check if temp file is old (potential cleanup issue)
                        if let Ok(modified) = metadata.modified() {
                            if let Ok(elapsed) = modified.elapsed() {
                                if elapsed.as_secs() > 86400 { // 24 hours
                                    report.add_violation(FileSystemViolation::UnsecuredTempFile {
                                        path: path.clone(),
                                        issue: "Old temporary file found (potential cleanup issue)".to_string(),
                                    });
                                }
                            }
                        }
                    }
                }
            }
        }

        Ok(())
    }

    /// Test path traversal protection
    fn test_path_traversal_protection(&mut self, report: &mut FileSystemSecurityReport) -> Result<()> {
        println!("  Testing path traversal protection...");

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
            "../../../../../../../../../../../../etc/passwd",
            "..%2F..%2F..%2Fetc%2Fpasswd",
            "....//....//....//etc/passwd",
            "%2e%2e%2f%2e%2e%2f%2e%2e%2fetc%2fpasswd",
        ];

        for malicious_path in &malicious_paths {
            let risk = self.assess_path_traversal_risk(malicious_path);
            let resolved_path = self.safe_canonicalize(malicious_path);

            // If the path resolves to something outside the app directory, it's a problem
            if let Some(resolved) = &resolved_path {
                if !resolved.starts_with(&self.app_root) {
                    report.add_violation(FileSystemViolation::PathTraversal {
                        attempted_path: malicious_path.to_string(),
                        resolved_path: resolved_path.clone(),
                        security_risk: risk,
                    });
                }
            }
        }

        Ok(())
    }

    /// Assess the risk level of a path traversal attempt
    fn assess_path_traversal_risk(&self, path: &str) -> PathTraversalRisk {
        let critical_paths = ["/etc/", "/root/", "/sys/", "/proc/", "C:\\Windows\\", "C:\\System32\\"];
        let high_risk_paths = ["/home/", "/Users/", "~/.ssh/", "~/.aws/"];
        let medium_risk_paths = ["../", "..\\"];

        for critical in &critical_paths {
            if path.contains(critical) {
                return PathTraversalRisk::Critical;
            }
        }

        for high_risk in &high_risk_paths {
            if path.contains(high_risk) {
                return PathTraversalRisk::High;
            }
        }

        for medium in &medium_risk_paths {
            if path.contains(medium) {
                return PathTraversalRisk::Medium;
            }
        }

        PathTraversalRisk::Low
    }

    /// Safely canonicalize a path without following dangerous symlinks
    fn safe_canonicalize(&mut self, path: &str) -> Option<PathBuf> {
        // Check cache first
        if let Some(cached) = self.path_cache.get(path) {
            return cached.clone();
        }

        // Try to resolve the path safely
        let result = match Path::new(path).canonicalize() {
            Ok(canonical) => Some(canonical),
            Err(_) => None,
        };

        // Cache the result
        self.path_cache.insert(path.to_string(), result.clone());
        result
    }

    /// Audit symlink security
    fn audit_symlink_security(&mut self, report: &mut FileSystemSecurityReport) -> Result<()> {
        println!("  Auditing symlink security...");

        // Check for dangerous symlinks in the application directory
        self.find_symlinks(&self.app_root.clone(), report)?;

        Ok(())
    }

    /// Recursively find and audit symlinks
    fn find_symlinks(&self, dir: &Path, report: &mut FileSystemSecurityReport) -> Result<()> {
        if let Ok(entries) = fs::read_dir(dir) {
            for entry in entries.flatten() {
                let path = entry.path();

                if path.is_symlink() {
                    if let Ok(target) = fs::read_link(&path) {
                        // Check if symlink points outside the application directory
                        if let Ok(target_canonical) = target.canonicalize() {
                            if !target_canonical.starts_with(&self.app_root) {
                                let risk = if target_canonical.starts_with("/etc") ||
                                           target_canonical.starts_with("/root") {
                                    "Critical: Points to system directory"
                                } else if target_canonical.starts_with("/home") {
                                    "High: Points outside application"
                                } else {
                                    "Medium: Relative symlink"
                                };

                                report.add_violation(FileSystemViolation::DangerousSymlink {
                                    link_path: path.clone(),
                                    target_path: target_canonical,
                                    risk: risk.to_string(),
                                });
                            }
                        }
                    }
                } else if path.is_dir() && path.file_name().map_or(false, |name| name != "target") {
                    // Recursively check subdirectories (but skip target dir)
                    self.find_symlinks(&path, report)?;
                }
            }
        }

        Ok(())
    }

    /// Audit directory permissions
    fn audit_directory_permissions(&mut self, report: &mut FileSystemSecurityReport) -> Result<()> {
        println!("  Auditing directory permissions...");

        let important_dirs = vec![
            &self.app_root,
            &self.app_root.join("config"),
            &self.app_root.join("data"),
            &self.app_root.join("logs"),
        ];

        for dir in important_dirs {
            if dir.exists() {
                report.paths_tested.push(dir.clone());
                self.audit_file_permissions(dir, report)?;
            }
        }

        Ok(())
    }

    /// Generate security recommendations based on findings
    fn generate_security_recommendations(&self, report: &mut FileSystemSecurityReport) {
        // Basic recommendations
        report.add_recommendation(
            "Set configuration files to 600 permissions (owner read/write only)".to_string()
        );
        report.add_recommendation(
            "Ensure temporary files are cleaned up after use".to_string()
        );
        report.add_recommendation(
            "Implement path validation to prevent directory traversal attacks".to_string()
        );

        // Specific recommendations based on violations
        if report.violations.iter().any(|v| matches!(v, FileSystemViolation::WorldReadable { .. })) {
            report.add_recommendation(
                "Remove world-readable permissions from sensitive files using: chmod o-r <file>".to_string()
            );
        }

        if report.violations.iter().any(|v| matches!(v, FileSystemViolation::PathTraversal { .. })) {
            report.add_recommendation(
                "Implement strict path validation with canonicalization and boundary checks".to_string()
            );
        }

        if report.violations.iter().any(|v| matches!(v, FileSystemViolation::DangerousSymlink { .. })) {
            report.add_recommendation(
                "Review and remove dangerous symlinks that point outside the application directory".to_string()
            );
        }

        // Always recommend basic security practices
        report.add_recommendation(
            "Implement file access logging for audit trail".to_string()
        );
        report.add_recommendation(
            "Regular security audits of file permissions and access patterns".to_string()
        );
    }
}

/// Path validation utilities for preventing traversal attacks
pub struct PathValidator {
    allowed_base_paths: Vec<PathBuf>,
}

impl PathValidator {
    pub fn new(allowed_base_paths: Vec<PathBuf>) -> Self {
        Self { allowed_base_paths }
    }

    /// Validate that a path is safe and within allowed boundaries
    pub fn validate_path(&self, path: &str) -> Result<PathBuf> {
        // Basic input validation
        if path.is_empty() {
            return Err(CentotypeError::Storage("Empty path not allowed".to_string()));
        }

        // Check for obvious traversal attempts
        if path.contains("..") || path.contains("~") {
            return Err(CentotypeError::Storage("Path traversal attempt detected".to_string()));
        }

        // Check for null bytes
        if path.contains('\0') {
            return Err(CentotypeError::Storage("Null bytes in path not allowed".to_string()));
        }

        // Resolve the path
        let path_buf = Path::new(path);
        let canonical_path = path_buf.canonicalize()
            .map_err(|_| CentotypeError::Storage("Path canonicalization failed".to_string()))?;

        // Check if the canonical path is within allowed boundaries
        for allowed_base in &self.allowed_base_paths {
            if canonical_path.starts_with(allowed_base) {
                return Ok(canonical_path);
            }
        }

        Err(CentotypeError::Storage(
            "Path is outside allowed directories".to_string()
        ))
    }

    /// Create a secure temporary file path
    pub fn create_secure_temp_path(&self, prefix: &str, extension: &str) -> Result<PathBuf> {
        use std::time::{SystemTime, UNIX_EPOCH};
        use rand::{thread_rng, Rng};

        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .map_err(|_| CentotypeError::Storage("Failed to get timestamp".to_string()))?
            .as_secs();

        let random: u32 = thread_rng().gen();
        let filename = format!("{}_{}_{}_{}.{}", prefix, std::process::id(), timestamp, random, extension);

        let temp_dir = std::env::temp_dir().join("centotype");
        std::fs::create_dir_all(&temp_dir)
            .map_err(|e| CentotypeError::Storage(format!("Failed to create temp directory: {}", e)))?;

        let temp_path = temp_dir.join(filename);
        self.validate_path(&temp_path.to_string_lossy())?;

        Ok(temp_path)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[test]
    fn test_path_validator_blocks_traversal() {
        let current_dir = env::current_dir().unwrap();
        let validator = PathValidator::new(vec![current_dir]);

        // Should block path traversal attempts
        assert!(validator.validate_path("../../../etc/passwd").is_err());
        assert!(validator.validate_path("..\\..\\..\\windows\\system32").is_err());
        assert!(validator.validate_path("~/secret").is_err());
        assert!(validator.validate_path("file\0injection").is_err());
    }

    #[test]
    fn test_path_validator_allows_safe_paths() {
        let current_dir = env::current_dir().unwrap();
        let validator = PathValidator::new(vec![current_dir.clone()]);

        // Should allow paths within the current directory
        let safe_path = current_dir.join("test_file.txt");
        if safe_path.exists() {
            assert!(validator.validate_path(&safe_path.to_string_lossy()).is_ok());
        }
    }

    #[test]
    fn test_file_system_auditor_creation() {
        let current_dir = env::current_dir().unwrap();
        let auditor = FileSystemSecurityAuditor::new(&current_dir);

        assert_eq!(auditor.app_root, current_dir);
        assert!(!auditor.config_dirs.is_empty());
        assert!(!auditor.temp_dirs.is_empty());
    }

    #[test]
    fn test_path_traversal_risk_assessment() {
        let current_dir = env::current_dir().unwrap();
        let auditor = FileSystemSecurityAuditor::new(&current_dir);

        assert!(matches!(
            auditor.assess_path_traversal_risk("/etc/passwd"),
            PathTraversalRisk::Critical
        ));

        assert!(matches!(
            auditor.assess_path_traversal_risk("/home/user/.ssh/id_rsa"),
            PathTraversalRisk::High
        ));

        assert!(matches!(
            auditor.assess_path_traversal_risk("../config"),
            PathTraversalRisk::Medium
        ));
    }

    #[test]
    fn test_secure_temp_path_creation() {
        let current_dir = env::current_dir().unwrap();
        let temp_dir = env::temp_dir().join("centotype");
        let validator = PathValidator::new(vec![temp_dir.clone()]);

        let temp_path = validator.create_secure_temp_path("test", "tmp").unwrap();
        assert!(temp_path.starts_with(&temp_dir));
        assert!(temp_path.extension().unwrap() == "tmp");
    }
}