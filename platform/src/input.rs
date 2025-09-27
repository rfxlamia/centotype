use super::detection::PlatformInfo;
use centotype_core::types::*;

#[derive(Debug, Clone)]
pub struct InputOptimizations {
    pub high_precision_timing: bool,
    pub recommended_buffer_size: usize,
    pub use_platform_events: bool,
}

impl InputOptimizations {
    pub fn for_platform(_info: &PlatformInfo) -> Result<Self> {
        Ok(Self {
            high_precision_timing: true,
            recommended_buffer_size: 1024,
            use_platform_events: false,
        })
    }

    pub fn configure_input_system(&self) -> Result<()> {
        Ok(())
    }
}

pub struct PlatformInput;
impl Default for PlatformInput {
    fn default() -> Self {
        Self::new()
    }
}

impl PlatformInput {
    pub fn new() -> Self {
        Self
    }
}
