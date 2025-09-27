//! Platform detection and system information

use centotype_core::types::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OSType {
    Linux,
    MacOS,
    Windows,
    Unknown,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Architecture {
    X86_64,
    ARM64,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct PlatformInfo {
    pub os_type: OSType,
    pub architecture: Architecture,
    pub os_version: String,
    pub cpu_cores: usize,
    pub total_memory_mb: u64,
}

pub struct PlatformDetector;

impl Default for PlatformDetector {
    fn default() -> Self {
        Self::new()
    }
}

impl PlatformDetector {
    pub fn new() -> Self {
        Self
    }

    pub fn detect_platform(&self) -> Result<PlatformInfo> {
        let os_type = if cfg!(target_os = "linux") {
            OSType::Linux
        } else if cfg!(target_os = "macos") {
            OSType::MacOS
        } else if cfg!(target_os = "windows") {
            OSType::Windows
        } else {
            OSType::Unknown
        };

        let architecture = if cfg!(target_arch = "x86_64") {
            Architecture::X86_64
        } else if cfg!(target_arch = "aarch64") {
            Architecture::ARM64
        } else {
            Architecture::Unknown
        };

        Ok(PlatformInfo {
            os_type,
            architecture,
            os_version: self.get_os_version(),
            cpu_cores: num_cpus::get(),
            total_memory_mb: self.get_total_memory_mb(),
        })
    }

    pub fn detect_terminal_capabilities(&self) -> Result<super::TerminalCapabilities> {
        // Implementation would detect terminal type and capabilities
        Ok(super::TerminalCapabilities {
            terminal_type: super::TerminalType::Unknown,
            supports_color: true,
            supports_raw_mode: true,
            supports_mouse: false,
            max_colors: 256,
        })
    }

    fn get_os_version(&self) -> String {
        "Unknown".to_string()
    }

    fn get_total_memory_mb(&self) -> u64 {
        8192 // Default 8GB
    }
}
