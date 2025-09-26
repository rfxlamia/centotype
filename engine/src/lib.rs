//! # Centotype Engine
//!
//! The engine crate provides the core event loop, input handling, and render system.

pub mod event;
pub mod input;
pub mod render;
pub mod tty;
pub mod performance;

// Re-export main types
pub use event::Event as EngineEvent;
pub use input::Input as InputProcessor;
pub use render::Render as Renderer;
pub use tty::Tty as TtyManager;
pub use performance::Performance as PerformanceMonitor;

use centotype_core::{types::*, CentotypeCore};
use centotype_platform::PlatformManager;
use std::sync::Arc;

/// Main engine coordinator that manages all subsystems
pub struct CentotypeEngine {
    core: Arc<CentotypeCore>,
    platform: Arc<PlatformManager>,
}

impl CentotypeEngine {
    pub fn new(core: Arc<CentotypeCore>, platform: Arc<PlatformManager>) -> Result<Self> {
        Ok(Self { core, platform })
    }

    /// Start the main engine loop - stub implementation
    pub async fn run(&mut self, _mode: TrainingMode, _target_text: String) -> Result<SessionResult> {
        // Stub implementation
        self.core.complete_session()
    }

    /// Emergency shutdown - restore terminal state immediately
    pub fn emergency_shutdown(&mut self) {
        // Stub implementation
    }
}