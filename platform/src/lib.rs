//! # Centotype Platform
//!
//! Platform-specific integrations and optimizations for different operating systems
//! and terminal environments. This crate provides:
//!
//! - Terminal capability detection and optimization
//! - OS-specific performance optimizations
//! - Cross-platform terminal handling abstractions
//! - Hardware detection and feature availability
//!
//! ## Supported Platforms
//!
//! - **Linux**: Full support with optimizations for common terminals
//! - **macOS**: Native terminal integration with performance tuning
//! - **Windows**: Windows Terminal, PowerShell, and cmd.exe support
//!
//! ## Performance Optimizations
//!
//! - Terminal-specific render optimizations
//! - Platform-native input handling
//! - Memory management tuning per OS
//! - CPU architecture detection

pub mod detection;
pub mod input;
pub mod performance;
pub mod terminal;

// Re-export main types
pub use detection::{Architecture, OSType, PlatformDetector, PlatformInfo};
pub use input::{InputOptimizations, PlatformInput};
pub use performance::{PlatformPerformance, SystemMetrics};
pub use terminal::{TerminalCapabilities, TerminalManager, TerminalType};

use centotype_core::types::*;
use once_cell::sync::Lazy;
use std::sync::Arc;

/// Main platform manager that coordinates all platform-specific functionality
pub struct PlatformManager {
    info: PlatformInfo,
    terminal_caps: TerminalCapabilities,
    input_optimizations: InputOptimizations,
    performance_settings: PlatformPerformance,
}

impl PlatformManager {
    pub fn new() -> Result<Self> {
        let detector = PlatformDetector::new();
        let info = detector.detect_platform()?;
        let terminal_caps = detector.detect_terminal_capabilities()?;
        let input_optimizations = InputOptimizations::for_platform(&info)?;
        let performance_settings = PlatformPerformance::optimize_for_platform(&info)?;

        tracing::info!(
            "Platform detected: OS={:?}, Terminal={:?}, Arch={:?}",
            info.os_type,
            terminal_caps.terminal_type,
            info.architecture
        );

        Ok(Self {
            info,
            terminal_caps,
            input_optimizations,
            performance_settings,
        })
    }

    pub fn platform_info(&self) -> &PlatformInfo {
        &self.info
    }

    pub fn terminal_capabilities(&self) -> &TerminalCapabilities {
        &self.terminal_caps
    }

    pub fn input_optimizations(&self) -> &InputOptimizations {
        &self.input_optimizations
    }

    pub fn performance_settings(&self) -> &PlatformPerformance {
        &self.performance_settings
    }

    /// Check if the platform meets minimum requirements
    pub fn validate_platform(&self) -> PlatformValidation {
        let mut issues = Vec::new();
        let mut warnings = Vec::new();

        // Check terminal capabilities
        if !self.terminal_caps.supports_raw_mode {
            issues.push("Terminal does not support raw mode".to_string());
        }

        if !self.terminal_caps.supports_color {
            warnings.push("Terminal does not support color - using monochrome mode".to_string());
        }

        // Check performance capabilities
        if !self.performance_settings.can_meet_targets {
            warnings.push("Platform may not meet performance targets".to_string());
        }

        // Check input handling
        if !self.input_optimizations.high_precision_timing {
            warnings.push("High precision input timing not available".to_string());
        }

        PlatformValidation {
            is_supported: issues.is_empty(),
            issues,
            warnings,
            recommended_settings: self.get_recommended_settings(),
        }
    }

    fn get_recommended_settings(&self) -> RecommendedSettings {
        RecommendedSettings {
            use_reduced_effects: !self.performance_settings.can_meet_targets,
            buffer_size: self.input_optimizations.recommended_buffer_size,
            render_frequency: self.performance_settings.recommended_fps,
            memory_limit_mb: self.performance_settings.memory_limit_mb,
        }
    }

    /// Apply platform-specific optimizations
    pub fn apply_optimizations(&self) -> Result<()> {
        // Apply input optimizations
        self.input_optimizations.configure_input_system()?;

        // Apply performance optimizations
        self.performance_settings.configure_performance()?;

        // Configure terminal-specific settings
        self.terminal_caps.configure_terminal()?;

        tracing::info!("Platform optimizations applied successfully");
        Ok(())
    }

    /// Get system resource information
    pub fn get_system_metrics(&self) -> SystemMetrics {
        self.performance_settings.get_current_metrics()
    }

    /// Check if graceful degradation is needed
    pub fn should_use_fallback_mode(&self) -> bool {
        !self.performance_settings.can_meet_targets || self.terminal_caps.has_limitations()
    }

    /// Get platform-specific error recovery strategies
    pub fn get_error_recovery(&self) -> ErrorRecoveryStrategy {
        match self.info.os_type {
            OSType::Windows => ErrorRecoveryStrategy {
                restore_console_mode: true,
                flush_input_buffer: true,
                reset_cursor_visibility: true,
                clear_screen_on_exit: false,
            },
            OSType::MacOS | OSType::Linux => ErrorRecoveryStrategy {
                restore_console_mode: true,
                flush_input_buffer: false,
                reset_cursor_visibility: true,
                clear_screen_on_exit: true,
            },
            OSType::Unknown => ErrorRecoveryStrategy::default(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct PlatformValidation {
    pub is_supported: bool,
    pub issues: Vec<String>,
    pub warnings: Vec<String>,
    pub recommended_settings: RecommendedSettings,
}

#[derive(Debug, Clone)]
pub struct RecommendedSettings {
    pub use_reduced_effects: bool,
    pub buffer_size: usize,
    pub render_frequency: u32,
    pub memory_limit_mb: u64,
}

#[derive(Debug, Clone)]
pub struct ErrorRecoveryStrategy {
    pub restore_console_mode: bool,
    pub flush_input_buffer: bool,
    pub reset_cursor_visibility: bool,
    pub clear_screen_on_exit: bool,
}

impl Default for ErrorRecoveryStrategy {
    fn default() -> Self {
        Self {
            restore_console_mode: true,
            flush_input_buffer: true,
            reset_cursor_visibility: true,
            clear_screen_on_exit: true,
        }
    }
}

/// Global platform manager instance
static PLATFORM_MANAGER: Lazy<Arc<PlatformManager>> =
    Lazy::new(|| Arc::new(PlatformManager::new().expect("Failed to initialize platform manager")));

/// Get the global platform manager instance
pub fn get_platform_manager() -> Arc<PlatformManager> {
    Arc::clone(&PLATFORM_MANAGER)
}

impl Default for PlatformManager {
    fn default() -> Self {
        Self::new().expect("Failed to create default platform manager")
    }
}
