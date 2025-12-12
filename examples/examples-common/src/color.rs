//! Color constants
//!
//! Common color values as packed u32 (0xRRGGBBAA format).

pub const WHITE: u32 = 0xFFFFFFFF;
pub const BLACK: u32 = 0x000000FF;
pub const RED: u32 = 0xFF0000FF;
pub const GREEN: u32 = 0x00FF00FF;
pub const BLUE: u32 = 0x0000FFFF;
pub const YELLOW: u32 = 0xFFFF00FF;
pub const CYAN: u32 = 0x00FFFFFF;
pub const MAGENTA: u32 = 0xFF00FFFF;
pub const ORANGE: u32 = 0xFF8000FF;

// UI colors
pub const TEXT_WHITE: u32 = 0xFFFFFFFF;
pub const TEXT_GRAY: u32 = 0xCCCCCCFF;
pub const TEXT_DIM: u32 = 0x888888FF;

// Light colors
pub const WARM_WHITE: u32 = 0xFFF2E6FF;
pub const COOL_WHITE: u32 = 0xE6F2FFFF;

// Sky presets
pub const SKY_HORIZON: u32 = 0xB2D8F2FF;
pub const SKY_ZENITH: u32 = 0x3366B2FF;
pub const SUN_DEFAULT: u32 = 0xFFEEDDFF;
