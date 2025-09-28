//! Centotype - CLI-based typing trainer with 100 progressive difficulty levels
//!
//! A high-performance typing trainer designed for developers and competitive typists.

use centotype_cli::{Cli, CliManager};
use centotype_content::ContentManager;
use centotype_core::CentotypeCore;
use centotype_engine::CentotypeEngine;
use centotype_persistence::PersistenceManager;
use centotype_platform::PlatformManager;
use clap::Parser;
use std::sync::Arc;
use tracing::{error, info};

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    // Initialize tracing
    tracing_subscriber::fmt::init();

    info!("Starting Centotype v{}", env!("CARGO_PKG_VERSION"));

    // Parse command line arguments
    let cli = Cli::parse();

    // Initialize platform manager
    let platform_manager = Arc::new(PlatformManager::new()?);

    // Validate platform compatibility
    let validation = platform_manager.validate_platform();
    if !validation.is_supported {
        error!("Platform not supported: {:?}", validation.issues);
        return Err(anyhow::anyhow!("Unsupported platform"));
    }

    if !validation.warnings.is_empty() {
        for warning in &validation.warnings {
            tracing::warn!("{}", warning);
        }
    }

    // Apply platform optimizations
    platform_manager.apply_optimizations()?;

    // Initialize core components
    let core = Arc::new(CentotypeCore::new());
    let content_manager = Arc::new(ContentManager::new().await?);
    let persistence_manager = Arc::new(PersistenceManager::new()?);

    // Initialize engine
    let mut engine = CentotypeEngine::new(Arc::clone(&core), Arc::clone(&platform_manager)).await?;

    // Initialize CLI manager
    let cli_manager = CliManager::new();

    // Load user configuration and profile
    let config = persistence_manager.load_config()?;
    let profile = persistence_manager.load_profile()?;

    info!(
        "User profile loaded: {} total sessions",
        profile.total_sessions
    );

    // Run the CLI command
    match cli_manager.run(cli) {
        Ok(_) => {
            info!("Centotype session completed successfully");
        }
        Err(e) => {
            error!("Error during session: {}", e);

            // Emergency shutdown to restore terminal state
            engine.emergency_shutdown();

            return Err(anyhow::anyhow!("Session failed: {}", e));
        }
    }

    // Save profile if it was modified
    persistence_manager.save_profile(&profile)?;

    info!("Centotype shutdown complete");
    Ok(())
}
