//! Debug Environment - Environment controls with debug integration
//!
//! Provides procedural environment configuration with debug inspector integration.
//! Part of Multi-Environment v3 system.

use crate::ffi::*;

/// Environment mode constants
pub mod env_mode {
    pub const GRADIENT: u32 = 0;
    pub const SCATTER: u32 = 1;
    pub const LINES: u32 = 2;
    pub const SILHOUETTE: u32 = 3;
    pub const RECTANGLES: u32 = 4;
    pub const ROOM: u32 = 5;
    pub const CURTAINS: u32 = 6;
    pub const RINGS: u32 = 7;
}

/// Blend mode constants for environment layering
pub mod blend_mode {
    pub const ALPHA: u32 = 0;
    pub const ADD: u32 = 1;
    pub const MULTIPLY: u32 = 2;
    pub const SCREEN: u32 = 3;
}

/// Gradient parameters for Mode 0 (Gradient)
#[derive(Clone, Copy)]
pub struct GradientParams {
    /// Color directly overhead (u32 RGBA)
    pub zenith: u32,
    /// Sky color at horizon level (u32 RGBA)
    pub sky_horizon: u32,
    /// Ground color at horizon level (u32 RGBA)
    pub ground_horizon: u32,
    /// Color directly below (u32 RGBA)
    pub nadir: u32,
    /// Rotation around Y axis in radians
    pub rotation: f32,
    /// Horizon vertical shift (-1.0 to 1.0)
    pub shift: f32,
}

impl Default for GradientParams {
    fn default() -> Self {
        Self::blue_sky()
    }
}

impl GradientParams {
    /// Create a blue sky gradient (default)
    pub fn blue_sky() -> Self {
        Self {
            zenith: 0x191970FF,    // Midnight blue
            sky_horizon: 0x87CEEBFF, // Sky blue
            ground_horizon: 0x228B22FF, // Forest green
            nadir: 0x2F4F4FFF,    // Dark slate gray
            rotation: 0.0,
            shift: 0.0,
        }
    }

    /// Create a sunset gradient
    pub fn sunset() -> Self {
        Self {
            zenith: 0x4A00E0FF,    // Deep purple
            sky_horizon: 0xFF6B6BFF, // Salmon/coral
            ground_horizon: 0x8B4513FF, // Saddle brown
            nadir: 0x2F2F2FFF,    // Dark gray
            rotation: 0.0,
            shift: 0.1,
        }
    }

    /// Create an underwater gradient
    pub fn underwater() -> Self {
        Self {
            zenith: 0x006994FF,    // Dark teal
            sky_horizon: 0x40E0D0FF, // Turquoise
            ground_horizon: 0x20B2AAFF, // Light sea green
            nadir: 0x003366FF,    // Navy
            rotation: 0.0,
            shift: -0.1,
        }
    }

    /// Create a night sky gradient
    pub fn night() -> Self {
        Self {
            zenith: 0x03030DFF,    // Almost black
            sky_horizon: 0x0D0D1AFF, // Very dark blue
            ground_horizon: 0x0A0A14FF, // Slightly lighter
            nadir: 0x000000FF,    // Black
            rotation: 0.0,
            shift: 0.0,
        }
    }

    /// Create a vapor/synthwave gradient
    pub fn vapor() -> Self {
        Self {
            zenith: 0xFF00FFFF,    // Magenta
            sky_horizon: 0x00FFFFFF, // Cyan
            ground_horizon: 0x8800FFFF, // Purple
            nadir: 0x000033FF,    // Dark navy
            rotation: 0.0,
            shift: 0.0,
        }
    }

    /// Create a dusty desert gradient
    pub fn desert() -> Self {
        Self {
            zenith: 0x4682B4FF,    // Steel blue
            sky_horizon: 0xFFE4B5FF, // Moccasin (sandy)
            ground_horizon: 0xD2B48CFF, // Tan
            nadir: 0x8B7355FF,    // Burly wood dark
            rotation: 0.0,
            shift: 0.05,
        }
    }
}

/// Environment state for debug control (Multi-Environment v3)
#[derive(Clone, Copy)]
pub struct DebugEnvironment {
    /// Base layer mode (0-7)
    pub base_mode: u32,
    /// Overlay layer mode (0-7)
    pub overlay_mode: u32,
    /// Blend mode (0-3)
    pub blend_mode: u32,
    /// Gradient parameters (used when mode is Gradient)
    pub gradient: GradientParams,
    /// Current preset index (for cycling)
    pub preset_index: u32,
}

impl Default for DebugEnvironment {
    fn default() -> Self {
        Self {
            base_mode: env_mode::GRADIENT,
            overlay_mode: env_mode::GRADIENT, // Same = no layering
            blend_mode: blend_mode::ALPHA,
            gradient: GradientParams::blue_sky(),
            preset_index: 0,
        }
    }
}

/// Number of gradient presets available
pub const GRADIENT_PRESET_COUNT: u32 = 6;

/// Preset names (for UI display)
pub const GRADIENT_PRESET_NAMES: [&str; 6] = [
    "Blue Sky",
    "Sunset",
    "Underwater",
    "Night",
    "Vapor",
    "Desert",
];

impl DebugEnvironment {
    /// Create with default blue sky
    pub fn new() -> Self {
        Self::default()
    }

    /// Create from a preset index
    pub fn from_preset(index: u32) -> Self {
        let mut env = Self::default();
        env.load_preset(index);
        env
    }

    /// Load a gradient preset by index
    pub fn load_preset(&mut self, index: u32) {
        self.preset_index = index % GRADIENT_PRESET_COUNT;
        self.gradient = match self.preset_index {
            0 => GradientParams::blue_sky(),
            1 => GradientParams::sunset(),
            2 => GradientParams::underwater(),
            3 => GradientParams::night(),
            4 => GradientParams::vapor(),
            5 => GradientParams::desert(),
            _ => GradientParams::blue_sky(),
        };
        self.base_mode = env_mode::GRADIENT;
    }

    /// Cycle to next preset
    pub fn next_preset(&mut self) {
        self.load_preset(self.preset_index + 1);
    }

    /// Cycle to previous preset
    pub fn prev_preset(&mut self) {
        if self.preset_index == 0 {
            self.load_preset(GRADIENT_PRESET_COUNT - 1);
        } else {
            self.load_preset(self.preset_index - 1);
        }
    }

    /// Get current preset name
    pub fn preset_name(&self) -> &'static str {
        GRADIENT_PRESET_NAMES[self.preset_index as usize % GRADIENT_PRESET_NAMES.len()]
    }

    /// Apply environment settings (call in render())
    pub fn apply(&self) {
        unsafe {
            // Set modes
            env_select_pair(self.base_mode, self.overlay_mode);
            env_blend_mode(self.blend_mode);

            // Set gradient parameters (if using gradient mode)
            if self.base_mode == env_mode::GRADIENT {
                env_gradient_set(
                    self.gradient.zenith,
                    self.gradient.sky_horizon,
                    self.gradient.ground_horizon,
                    self.gradient.nadir,
                    self.gradient.rotation,
                    self.gradient.shift,
                );
            }
        }
    }

    /// Apply environment settings and draw sky
    pub fn apply_and_draw(&self) {
        self.apply();
        unsafe {
            draw_sky();
        }
    }
}

/// Register environment debug values
///
/// Call this in init() with pointers to your static environment state
pub unsafe fn register_environment_debug(
    zenith: *const u8,
    sky_horizon: *const u8,
    ground_horizon: *const u8,
    nadir: *const u8,
    rotation: *const f32,
    shift: *const f32,
) {
    debug_group_begin(b"environment".as_ptr(), 11);
    debug_register_color(b"zenith".as_ptr(), 6, zenith);
    debug_register_color(b"sky_horizon".as_ptr(), 11, sky_horizon);
    debug_register_color(b"ground_horizon".as_ptr(), 14, ground_horizon);
    debug_register_color(b"nadir".as_ptr(), 5, nadir);
    debug_register_f32(b"rotation".as_ptr(), 8, rotation);
    debug_register_f32(b"shift".as_ptr(), 5, shift);
    debug_group_end();
}
