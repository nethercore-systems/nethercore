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

/// Scatter variant constants for Mode 1
pub mod scatter_variant {
    pub const STARS: u32 = 0;
    pub const VERTICAL: u32 = 1; // Rain/snow
    pub const HORIZONTAL: u32 = 2; // Speed lines
    pub const WARP: u32 = 3; // Hyperspace
}

/// Lines variant constants for Mode 2
pub mod lines_variant {
    pub const FLOOR: u32 = 0;
    pub const CEILING: u32 = 1;
    pub const SPHERE: u32 = 2;
}

/// Lines type constants for Mode 2
pub mod lines_type {
    pub const HORIZONTAL: u32 = 0;
    pub const VERTICAL: u32 = 1;
    pub const GRID: u32 = 2;
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

/// Scatter parameters for Mode 1 (Scatter)
#[derive(Clone, Copy)]
pub struct ScatterParams {
    /// Scatter type: 0=Stars, 1=Vertical, 2=Horizontal, 3=Warp
    pub variant: u32,
    /// Particle count (0-255)
    pub density: u32,
    /// Particle size (0-255)
    pub size: u32,
    /// Glow/bloom intensity (0-255)
    pub glow: u32,
    /// Streak elongation (0-63, 0=points)
    pub streak_length: u32,
    /// Main particle color (0xRRGGBB00)
    pub color_primary: u32,
    /// Variation color (0xRRGGBB00)
    pub color_secondary: u32,
    /// Layer separation amount (0-255)
    pub parallax_rate: u32,
    /// Size variation with depth (0-255)
    pub parallax_size: u32,
    /// Animation phase (0-65535)
    pub phase: u16,
}

impl Default for ScatterParams {
    fn default() -> Self {
        Self::starfield()
    }
}

impl ScatterParams {
    /// Create a starfield (night sky with twinkling stars)
    pub fn starfield() -> Self {
        Self {
            variant: scatter_variant::STARS,
            density: 128,
            size: 64,
            glow: 32,
            streak_length: 0,
            color_primary: 0xFFFFFF00, // White
            color_secondary: 0xAABBFF00, // Slight blue
            parallax_rate: 0,
            parallax_size: 0,
            phase: 0,
        }
    }

    /// Create rain effect
    pub fn rain() -> Self {
        Self {
            variant: scatter_variant::VERTICAL,
            density: 180,
            size: 32,
            glow: 16,
            streak_length: 40,
            color_primary: 0x8899AA00, // Gray-blue
            color_secondary: 0xAABBCC00,
            parallax_rate: 64,
            parallax_size: 32,
            phase: 0,
        }
    }

    /// Create hyperspace warp effect
    pub fn hyperspace() -> Self {
        Self {
            variant: scatter_variant::WARP,
            density: 200,
            size: 48,
            glow: 128,
            streak_length: 63,
            color_primary: 0xFFFFFF00, // White
            color_secondary: 0x88CCFF00, // Light blue
            parallax_rate: 128,
            parallax_size: 64,
            phase: 0,
        }
    }
}

/// Lines parameters for Mode 2 (Lines)
#[derive(Clone, Copy)]
pub struct LinesParams {
    /// Surface type: 0=Floor, 1=Ceiling, 2=Sphere
    pub variant: u32,
    /// Line pattern: 0=Horizontal, 1=Vertical, 2=Grid
    pub line_type: u32,
    /// Line thickness (0-255)
    pub thickness: u32,
    /// Distance between lines (world units)
    pub spacing: f32,
    /// Distance where lines start fading
    pub fade_distance: f32,
    /// Main line color (0xRRGGBBAA)
    pub color_primary: u32,
    /// Accent line color (0xRRGGBBAA)
    pub color_accent: u32,
    /// Make every Nth line accent
    pub accent_every: u32,
    /// Scroll phase (0-65535)
    pub phase: u16,
}

impl Default for LinesParams {
    fn default() -> Self {
        Self::synthwave()
    }
}

impl LinesParams {
    /// Create synthwave floor grid
    pub fn synthwave() -> Self {
        Self {
            variant: lines_variant::FLOOR,
            line_type: lines_type::GRID,
            thickness: 32,
            spacing: 1.0,
            fade_distance: 20.0,
            color_primary: 0xFF00FFFF, // Magenta
            color_accent: 0x00FFFFFF,  // Cyan
            accent_every: 5,
            phase: 0,
        }
    }

    /// Create racing track lines
    pub fn racing() -> Self {
        Self {
            variant: lines_variant::FLOOR,
            line_type: lines_type::HORIZONTAL,
            thickness: 48,
            spacing: 2.0,
            fade_distance: 50.0,
            color_primary: 0xFFFFFFFF, // White
            color_accent: 0xFFFF00FF,  // Yellow
            accent_every: 5,
            phase: 0,
        }
    }

    /// Create holographic spherical grid
    pub fn hologram() -> Self {
        Self {
            variant: lines_variant::SPHERE,
            line_type: lines_type::GRID,
            thickness: 16,
            spacing: 0.5,
            fade_distance: 100.0,
            color_primary: 0x00AAFF80, // Blue, semi-transparent
            color_accent: 0x00FFFF80,  // Cyan, semi-transparent
            accent_every: 4,
            phase: 0,
        }
    }
}

/// Rings parameters for Mode 7 (Rings)
#[derive(Clone, Copy)]
pub struct RingsParams {
    /// Number of rings (1-255)
    pub ring_count: u32,
    /// Ring thickness (0-255)
    pub thickness: u32,
    /// First alternating color (0xRRGGBBAA)
    pub color_a: u32,
    /// Second alternating color (0xRRGGBBAA)
    pub color_b: u32,
    /// Center glow color (0xRRGGBBAA)
    pub center_color: u32,
    /// Center glow falloff (0-255)
    pub center_falloff: u32,
    /// Spiral rotation in degrees
    pub spiral_twist: f32,
    /// Ring axis direction X
    pub axis_x: f32,
    /// Ring axis direction Y
    pub axis_y: f32,
    /// Ring axis direction Z
    pub axis_z: f32,
    /// Rotation phase (0-65535 = 0°-360°)
    pub phase: u16,
}

impl Default for RingsParams {
    fn default() -> Self {
        Self::portal()
    }
}

impl RingsParams {
    /// Create spinning portal effect
    pub fn portal() -> Self {
        Self {
            ring_count: 12,
            thickness: 128,
            color_a: 0xFF00FFFF, // Magenta
            color_b: 0x8800FFFF, // Purple
            center_color: 0xFFFFFFFF, // White
            center_falloff: 32,
            spiral_twist: 0.0,
            axis_x: 0.0,
            axis_y: 0.0,
            axis_z: -1.0,
            phase: 0,
        }
    }

    /// Create forward-moving tunnel
    pub fn tunnel() -> Self {
        Self {
            ring_count: 20,
            thickness: 200,
            color_a: 0x333333FF, // Dark gray
            color_b: 0x666666FF, // Gray
            center_color: 0xFFFFFFFF, // White
            center_falloff: 16,
            spiral_twist: 0.0,
            axis_x: 0.0,
            axis_y: 0.0,
            axis_z: -1.0,
            phase: 0,
        }
    }

    /// Create hypnotic spiral pattern
    pub fn hypnotic() -> Self {
        Self {
            ring_count: 8,
            thickness: 180,
            color_a: 0x000000FF, // Black
            color_b: 0xFFFFFFFF, // White
            center_color: 0xFF0000FF, // Red
            center_falloff: 64,
            spiral_twist: 90.0,
            axis_x: 0.0,
            axis_y: 0.0,
            axis_z: -1.0,
            phase: 0,
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
    /// Scatter parameters (used when mode is Scatter)
    pub scatter: ScatterParams,
    /// Lines parameters (used when mode is Lines)
    pub lines: LinesParams,
    /// Rings parameters (used when mode is Rings)
    pub rings: RingsParams,
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
            scatter: ScatterParams::starfield(),
            lines: LinesParams::synthwave(),
            rings: RingsParams::portal(),
            preset_index: 0,
        }
    }
}

/// Number of gradient presets available
pub const GRADIENT_PRESET_COUNT: u32 = 6;
/// Number of scatter presets available
pub const SCATTER_PRESET_COUNT: u32 = 3;
/// Number of lines presets available
pub const LINES_PRESET_COUNT: u32 = 3;
/// Number of rings presets available
pub const RINGS_PRESET_COUNT: u32 = 3;
/// Total number of presets (gradient + scatter + lines + rings)
pub const TOTAL_PRESET_COUNT: u32 =
    GRADIENT_PRESET_COUNT + SCATTER_PRESET_COUNT + LINES_PRESET_COUNT + RINGS_PRESET_COUNT;

/// Preset names (for UI display)
pub const GRADIENT_PRESET_NAMES: [&str; 6] = [
    "Blue Sky",
    "Sunset",
    "Underwater",
    "Night",
    "Vapor",
    "Desert",
];

/// Scatter preset names
pub const SCATTER_PRESET_NAMES: [&str; 3] = ["Starfield", "Rain", "Hyperspace"];

/// Lines preset names
pub const LINES_PRESET_NAMES: [&str; 3] = ["Synthwave", "Racing", "Hologram"];

/// Rings preset names
pub const RINGS_PRESET_NAMES: [&str; 3] = ["Portal", "Tunnel", "Hypnotic"];

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

    /// Load a preset by index (cycles through all modes)
    /// Presets 0-5: Gradient, 6-8: Scatter, 9-11: Lines, 12-14: Rings
    pub fn load_preset(&mut self, index: u32) {
        self.preset_index = index % TOTAL_PRESET_COUNT;

        if self.preset_index < GRADIENT_PRESET_COUNT {
            // Gradient presets (0-5)
            self.base_mode = env_mode::GRADIENT;
            self.gradient = match self.preset_index {
                0 => GradientParams::blue_sky(),
                1 => GradientParams::sunset(),
                2 => GradientParams::underwater(),
                3 => GradientParams::night(),
                4 => GradientParams::vapor(),
                5 => GradientParams::desert(),
                _ => GradientParams::blue_sky(),
            };
        } else if self.preset_index < GRADIENT_PRESET_COUNT + SCATTER_PRESET_COUNT {
            // Scatter presets (6-8)
            self.base_mode = env_mode::SCATTER;
            let scatter_idx = self.preset_index - GRADIENT_PRESET_COUNT;
            self.scatter = match scatter_idx {
                0 => ScatterParams::starfield(),
                1 => ScatterParams::rain(),
                2 => ScatterParams::hyperspace(),
                _ => ScatterParams::starfield(),
            };
        } else if self.preset_index < GRADIENT_PRESET_COUNT + SCATTER_PRESET_COUNT + LINES_PRESET_COUNT
        {
            // Lines presets (9-11)
            self.base_mode = env_mode::LINES;
            let lines_idx = self.preset_index - GRADIENT_PRESET_COUNT - SCATTER_PRESET_COUNT;
            self.lines = match lines_idx {
                0 => LinesParams::synthwave(),
                1 => LinesParams::racing(),
                2 => LinesParams::hologram(),
                _ => LinesParams::synthwave(),
            };
        } else {
            // Rings presets (12-14)
            self.base_mode = env_mode::RINGS;
            let rings_idx =
                self.preset_index - GRADIENT_PRESET_COUNT - SCATTER_PRESET_COUNT - LINES_PRESET_COUNT;
            self.rings = match rings_idx {
                0 => RingsParams::portal(),
                1 => RingsParams::tunnel(),
                2 => RingsParams::hypnotic(),
                _ => RingsParams::portal(),
            };
        }
        self.overlay_mode = self.base_mode; // No layering by default
    }

    /// Cycle to next preset
    pub fn next_preset(&mut self) {
        self.load_preset(self.preset_index + 1);
    }

    /// Cycle to previous preset
    pub fn prev_preset(&mut self) {
        if self.preset_index == 0 {
            self.load_preset(TOTAL_PRESET_COUNT - 1);
        } else {
            self.load_preset(self.preset_index - 1);
        }
    }

    /// Get current preset name
    pub fn preset_name(&self) -> &'static str {
        if self.preset_index < GRADIENT_PRESET_COUNT {
            GRADIENT_PRESET_NAMES[self.preset_index as usize]
        } else if self.preset_index < GRADIENT_PRESET_COUNT + SCATTER_PRESET_COUNT {
            SCATTER_PRESET_NAMES[(self.preset_index - GRADIENT_PRESET_COUNT) as usize]
        } else if self.preset_index < GRADIENT_PRESET_COUNT + SCATTER_PRESET_COUNT + LINES_PRESET_COUNT
        {
            LINES_PRESET_NAMES
                [(self.preset_index - GRADIENT_PRESET_COUNT - SCATTER_PRESET_COUNT) as usize]
        } else {
            RINGS_PRESET_NAMES[(self.preset_index
                - GRADIENT_PRESET_COUNT
                - SCATTER_PRESET_COUNT
                - LINES_PRESET_COUNT) as usize]
        }
    }

    /// Get current mode name
    pub fn mode_name(&self) -> &'static str {
        match self.base_mode {
            0 => "Gradient",
            1 => "Scatter",
            2 => "Lines",
            3 => "Silhouette",
            4 => "Rectangles",
            5 => "Room",
            6 => "Curtains",
            7 => "Rings",
            _ => "Unknown",
        }
    }

    /// Advance animation phase (call in update())
    pub fn tick(&mut self, delta_speed: f32) {
        let delta = (delta_speed * 100.0) as u16;
        self.scatter.phase = self.scatter.phase.wrapping_add(delta);
        self.lines.phase = self.lines.phase.wrapping_add(delta);
        self.rings.phase = self.rings.phase.wrapping_add(delta);
    }

    /// Apply environment settings (call in render())
    pub fn apply(&self) {
        unsafe {
            // Set modes
            env_select_pair(self.base_mode, self.overlay_mode);
            env_blend_mode(self.blend_mode);

            // Apply parameters based on current mode
            match self.base_mode {
                0 => {
                    // Gradient
                    env_gradient_set(
                        self.gradient.zenith,
                        self.gradient.sky_horizon,
                        self.gradient.ground_horizon,
                        self.gradient.nadir,
                        self.gradient.rotation,
                        self.gradient.shift,
                    );
                }
                1 => {
                    // Scatter
                    env_scatter_set(
                        self.scatter.variant,
                        self.scatter.density,
                        self.scatter.size,
                        self.scatter.glow,
                        self.scatter.streak_length,
                        self.scatter.color_primary,
                        self.scatter.color_secondary,
                        self.scatter.parallax_rate,
                        self.scatter.parallax_size,
                        self.scatter.phase as u32,
                    );
                }
                2 => {
                    // Lines
                    env_lines_set(
                        self.lines.variant,
                        self.lines.line_type,
                        self.lines.thickness,
                        self.lines.spacing,
                        self.lines.fade_distance,
                        self.lines.color_primary,
                        self.lines.color_accent,
                        self.lines.accent_every,
                        self.lines.phase as u32,
                    );
                }
                7 => {
                    // Rings
                    env_rings_set(
                        self.rings.ring_count,
                        self.rings.thickness,
                        self.rings.color_a,
                        self.rings.color_b,
                        self.rings.center_color,
                        self.rings.center_falloff,
                        self.rings.spiral_twist,
                        self.rings.axis_x,
                        self.rings.axis_y,
                        self.rings.axis_z,
                        self.rings.phase as u32,
                    );
                }
                _ => {
                    // Modes 3-6 not yet implemented, fall back to gradient
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
