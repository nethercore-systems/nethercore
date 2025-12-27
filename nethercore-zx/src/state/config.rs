//! Init-time configuration for Nethercore ZX

/// Init-time configuration for Nethercore ZX
///
/// All config functions are **init-only** and **single-call** — calling the same
/// function twice during init() is an error and will trap.
#[derive(Debug, Clone)]
pub struct ZXInitConfig {
    /// Tick rate index (0-3 for Z: 24, 30, 60, 120 fps)
    pub tick_rate_index: u32,
    /// Clear/background color (RGBA: 0xRRGGBBAA)
    pub clear_color: u32,
    /// Render mode (0-3: Unlit, Matcap, PBR, Hybrid)
    pub render_mode: u8,
    /// Whether any config was changed during init
    pub modified: bool,

    // Duplicate call tracking — each config function can only be called once
    /// Whether set_tick_rate() has been called
    pub tick_rate_set: bool,
    /// Whether set_clear_color() has been called
    pub clear_color_set: bool,
    /// Whether render_mode() has been called
    pub render_mode_set: bool,
}

impl Default for ZXInitConfig {
    fn default() -> Self {
        Self {
            tick_rate_index: 2,      // Default 60 fps
            clear_color: 0x000000FF, // Black, fully opaque
            render_mode: 0,          // Unlit
            modified: false,
            // No config functions called yet
            tick_rate_set: false,
            clear_color_set: false,
            render_mode_set: false,
        }
    }
}
