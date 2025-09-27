#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminalType {
    XTerm,
    GnomeTerminal,
    ITerm2,
    WindowsTerminal,
    CMD,
    Unknown,
}

#[derive(Debug, Clone)]
pub struct TerminalCapabilities {
    pub terminal_type: TerminalType,
    pub supports_color: bool,
    pub supports_raw_mode: bool,
    pub supports_mouse: bool,
    pub max_colors: u16,
}

impl TerminalCapabilities {
    pub fn has_limitations(&self) -> bool {
        false
    }
    pub fn configure_terminal(&self) -> centotype_core::types::Result<()> {
        Ok(())
    }
}

pub struct TerminalManager;
impl Default for TerminalManager {
    fn default() -> Self {
        Self::new()
    }
}

impl TerminalManager {
    pub fn new() -> Self {
        Self
    }
}
