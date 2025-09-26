//! # Centotype Persistence
//!
//! Profile storage, configuration management, and data persistence.

pub mod profile;
pub mod config;
pub mod storage;

use centotype_core::types::*;
use std::path::PathBuf;

pub struct PersistenceManager {
    config_dir: PathBuf,
    data_dir: PathBuf,
}

impl PersistenceManager {
    pub fn new() -> Result<Self> {
        let config_dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("centotype");
        let data_dir = dirs::data_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("centotype");

        std::fs::create_dir_all(&config_dir)?;
        std::fs::create_dir_all(&data_dir)?;

        Ok(Self { config_dir, data_dir })
    }

    pub fn load_config(&self) -> Result<Config> {
        let config_path = self.config_dir.join("config.toml");
        if config_path.exists() {
            let content = std::fs::read_to_string(config_path)?;
            toml::from_str(&content).map_err(|e| {
                CentotypeError::Config(format!("Failed to parse config: {}", e))
            })
        } else {
            Ok(Config::default())
        }
    }

    pub fn save_config(&self, config: &Config) -> Result<()> {
        let config_path = self.config_dir.join("config.toml");
        let content = toml::to_string_pretty(config).map_err(|e| {
            CentotypeError::Config(format!("Failed to serialize config: {}", e))
        })?;
        std::fs::write(config_path, content)?;
        Ok(())
    }

    pub fn load_profile(&self) -> Result<UserProgress> {
        let profile_path = self.data_dir.join("profile.json");
        if profile_path.exists() {
            let content = std::fs::read_to_string(profile_path)?;
            serde_json::from_str(&content).map_err(|e| {
                CentotypeError::Persistence(format!("Failed to parse profile: {}", e))
            })
        } else {
            Ok(UserProgress::default())
        }
    }

    pub fn save_profile(&self, profile: &UserProgress) -> Result<()> {
        let profile_path = self.data_dir.join("profile.json");
        let temp_path = profile_path.with_extension("json.tmp");
        
        // Atomic write: write to temp file then rename
        let content = serde_json::to_string_pretty(profile)?;
        std::fs::write(&temp_path, content)?;
        std::fs::rename(temp_path, profile_path)?;
        
        Ok(())
    }

    pub fn save_session_result(&self, result: &SessionResult) -> Result<()> {
        // Implementation for saving individual session results
        Ok(())
    }
}