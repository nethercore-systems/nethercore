//! Debug Environment helpers (Multi-Environment v4).
//!
//! This module is a small convenience layer for examples. It is not part of the
//! stable ZX FFI surface; the canonical env_* API lives in `nethercore/include/zx.rs`.

use crate::ffi::*;

/// Environment mode constants (0–7).
pub mod env_mode {
    pub const GRADIENT: u32 = 0;
    pub const CELLS: u32 = 1;
    pub const LINES: u32 = 2;
    pub const SILHOUETTE: u32 = 3;
    pub const NEBULA: u32 = 4;
    pub const ROOM: u32 = 5;
    pub const VEIL: u32 = 6;
    pub const RINGS: u32 = 7;

    // Backward-compat aliases (legacy names).
    pub const SCATTER: u32 = CELLS;
    pub const RECTANGLES: u32 = NEBULA;
    pub const CURTAINS: u32 = VEIL;
}

/// Blend mode constants for environment layering.
pub mod blend_mode {
    pub const ALPHA: u32 = 0;
    pub const ADD: u32 = 1;
    pub const MULTIPLY: u32 = 2;
    pub const SCREEN: u32 = 3;
}

#[derive(Clone, Copy)]
pub struct GradientParams {
    pub zenith: u32,
    pub sky_horizon: u32,
    pub ground_horizon: u32,
    pub nadir: u32,
    pub rotation: f32,
    pub shift: f32,
    pub sun_elevation: f32,
    pub sun_disk: u32,
    pub sun_halo: u32,
    pub sun_intensity: u32,
    pub horizon_haze: u32,
    pub sun_warmth: u32,
    pub cloudiness: u32,
    pub cloud_phase: u32,
}

impl GradientParams {
    pub fn clear_day() -> Self {
        Self {
            zenith: 0x2E65FFFF,
            sky_horizon: 0xA9D8FFFF,
            ground_horizon: 0x4D8B4DFF,
            nadir: 0x102010FF,
            rotation: 0.35,
            shift: 0.00,
            sun_elevation: 0.95,
            sun_disk: 10,
            sun_halo: 72,
            sun_intensity: 230,
            horizon_haze: 32,
            sun_warmth: 24,
            cloudiness: 40,
            cloud_phase: 0,
        }
    }
}

impl Default for GradientParams {
    fn default() -> Self {
        Self::clear_day()
    }
}

#[derive(Clone, Copy)]
pub struct CellsParams {
    pub family: u32,
    pub variant: u32,
    pub density: u32,
    pub size_min: u32,
    pub size_max: u32,
    pub intensity: u32,
    pub shape: u32,
    pub motion: u32,
    pub parallax: u32,
    pub height_bias: u32,
    pub clustering: u32,
    pub color_a: u32,
    pub color_b: u32,
    pub phase: u16,
    pub seed: u32,
}

impl CellsParams {
    pub fn disabled() -> Self {
        Self {
            family: 0,
            variant: 0,
            density: 0,
            size_min: 0,
            size_max: 0,
            intensity: 0,
            shape: 0,
            motion: 0,
            parallax: 0,
            height_bias: 0,
            clustering: 0,
            color_a: 0,
            color_b: 0,
            phase: 0,
            seed: 0,
        }
    }

    pub fn starfield_calm() -> Self {
        Self {
            family: 0,
            variant: 0,
            density: 120,
            size_min: 2,
            size_max: 10,
            intensity: 200,
            shape: 220,
            motion: 64,
            parallax: 140,
            height_bias: 100,
            clustering: 40,
            color_a: 0xDDE6FFFF,
            color_b: 0xFFF2C0FF,
            phase: 0,
            seed: 0,
        }
    }
}

impl Default for CellsParams {
    fn default() -> Self {
        Self::starfield_calm()
    }
}

#[derive(Clone, Copy)]
pub struct LinesParams {
    pub variant: u32,
    pub line_type: u32,
    pub thickness: u32,
    pub spacing: f32,
    pub fade_distance: f32,
    pub parallax: u32,
    pub color_primary: u32,
    pub color_accent: u32,
    pub accent_every: u32,
    pub phase: u16,
    pub profile: u32,
    pub warp: u32,
    pub wobble: u32,
    pub glow: u32,
    pub axis_x: f32,
    pub axis_y: f32,
    pub axis_z: f32,
    pub seed: u32,
}

impl LinesParams {
    pub fn synth_grid() -> Self {
        Self {
            variant: 0,
            line_type: 2,
            thickness: 18,
            spacing: 2.25,
            fade_distance: 80.0,
            parallax: 0,
            color_primary: 0x00FFB0C0,
            color_accent: 0xFF3AF0FF,
            accent_every: 8,
            phase: 0,
            profile: 0,
            warp: 24,
            wobble: 0,
            glow: 96,
            axis_x: 0.0,
            axis_y: 0.0,
            axis_z: 1.0,
            seed: 0x4D2F5A10,
        }
    }
}

impl Default for LinesParams {
    fn default() -> Self {
        Self::synth_grid()
    }
}

#[derive(Clone, Copy)]
pub struct RingsParams {
    pub family: u32,
    pub ring_count: u32,
    pub thickness: u32,
    pub color_a: u32,
    pub color_b: u32,
    pub center_color: u32,
    pub center_falloff: u32,
    pub spiral_twist: f32,
    pub axis_x: f32,
    pub axis_y: f32,
    pub axis_z: f32,
    pub phase: u16,
    pub wobble: u16,
    pub noise: u32,
    pub dash: u32,
    pub glow: u32,
    pub seed: u32,
}

impl RingsParams {
    pub fn stargate_portal() -> Self {
        Self {
            family: 0,
            ring_count: 48,
            thickness: 28,
            color_a: 0x2EE7FFFF,
            color_b: 0x0B2B4CFF,
            center_color: 0xE8FFFFFF,
            center_falloff: 190,
            spiral_twist: 25.0,
            axis_x: 0.0,
            axis_y: 0.0,
            axis_z: 1.0,
            phase: 0,
            wobble: 9000,
            noise: 32,
            dash: 24,
            glow: 160,
            seed: 41,
        }
    }
}

impl Default for RingsParams {
    fn default() -> Self {
        Self::stargate_portal()
    }
}

/// Small environment state for examples.
#[derive(Clone, Copy)]
pub struct DebugEnvironment {
    pub base_mode: u32,
    pub overlay_mode: u32,
    pub blend_mode: u32,
    pub gradient: GradientParams,
    pub cells: CellsParams,
    pub lines: LinesParams,
    pub rings: RingsParams,
}

impl Default for DebugEnvironment {
    fn default() -> Self {
        Self {
            base_mode: env_mode::GRADIENT,
            overlay_mode: env_mode::CELLS,
            blend_mode: blend_mode::SCREEN,
            gradient: GradientParams::default(),
            cells: CellsParams::starfield_calm(),
            lines: LinesParams::synth_grid(),
            rings: RingsParams::stargate_portal(),
        }
    }
}

impl DebugEnvironment {
    /// Advance loop phases (call in `update()`).
    pub fn tick(&mut self, delta_speed: f32) {
        let delta = (delta_speed * 100.0) as u16;
        self.cells.phase = self.cells.phase.wrapping_add(delta);
        self.lines.phase = self.lines.phase.wrapping_add(delta);
        self.rings.phase = self.rings.phase.wrapping_add(delta);
    }

    /// Apply environment settings (call in `render()` before `draw_env()`).
    pub fn apply(&self) {
        unsafe {
            env_blend(self.blend_mode);

            // Base layer
            match self.base_mode {
                env_mode::GRADIENT => {
                    env_gradient(
                        0,
                        self.gradient.zenith,
                        self.gradient.sky_horizon,
                        self.gradient.ground_horizon,
                        self.gradient.nadir,
                        self.gradient.rotation,
                        self.gradient.shift,
                        self.gradient.sun_elevation,
                        self.gradient.sun_disk,
                        self.gradient.sun_halo,
                        self.gradient.sun_intensity,
                        self.gradient.horizon_haze,
                        self.gradient.sun_warmth,
                        self.gradient.cloudiness,
                        self.gradient.cloud_phase,
                    );
                }
                env_mode::CELLS => {
                    env_cells(
                        0,
                        self.cells.family,
                        self.cells.variant,
                        self.cells.density,
                        self.cells.size_min,
                        self.cells.size_max,
                        self.cells.intensity,
                        self.cells.shape,
                        self.cells.motion,
                        self.cells.parallax,
                        self.cells.height_bias,
                        self.cells.clustering,
                        self.cells.color_a,
                        self.cells.color_b,
                        self.cells.phase as u32,
                        self.cells.seed,
                    );
                }
                env_mode::LINES => {
                    env_lines(
                        0,
                        self.lines.variant,
                        self.lines.line_type,
                        self.lines.thickness,
                        self.lines.spacing,
                        self.lines.fade_distance,
                        self.lines.parallax,
                        self.lines.color_primary,
                        self.lines.color_accent,
                        self.lines.accent_every,
                        self.lines.phase as u32,
                        self.lines.profile,
                        self.lines.warp,
                        self.lines.wobble,
                        self.lines.glow,
                        self.lines.axis_x,
                        self.lines.axis_y,
                        self.lines.axis_z,
                        self.lines.seed,
                    );
                }
                env_mode::RINGS => {
                    env_rings(
                        0,
                        self.rings.family,
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
                        self.rings.wobble as u32,
                        self.rings.noise,
                        self.rings.dash,
                        self.rings.glow,
                        self.rings.seed,
                    );
                }
                _ => {
                    // Not covered by this small helper.
                    env_gradient(
                        0,
                        self.gradient.zenith,
                        self.gradient.sky_horizon,
                        self.gradient.ground_horizon,
                        self.gradient.nadir,
                        self.gradient.rotation,
                        self.gradient.shift,
                        self.gradient.sun_elevation,
                        self.gradient.sun_disk,
                        self.gradient.sun_halo,
                        self.gradient.sun_intensity,
                        self.gradient.horizon_haze,
                        self.gradient.sun_warmth,
                        self.gradient.cloudiness,
                        self.gradient.cloud_phase,
                    );
                }
            }

            // Overlay layer (only the implemented subset; others fall back to "disabled")
            match self.overlay_mode {
                env_mode::CELLS => {
                    env_cells(
                        1,
                        self.cells.family,
                        self.cells.variant,
                        self.cells.density,
                        self.cells.size_min,
                        self.cells.size_max,
                        self.cells.intensity,
                        self.cells.shape,
                        self.cells.motion,
                        self.cells.parallax,
                        self.cells.height_bias,
                        self.cells.clustering,
                        self.cells.color_a,
                        self.cells.color_b,
                        self.cells.phase as u32,
                        self.cells.seed,
                    );
                }
                env_mode::LINES => {
                    env_lines(
                        1,
                        self.lines.variant,
                        self.lines.line_type,
                        self.lines.thickness,
                        self.lines.spacing,
                        self.lines.fade_distance,
                        self.lines.parallax,
                        self.lines.color_primary,
                        self.lines.color_accent,
                        self.lines.accent_every,
                        self.lines.phase as u32,
                        self.lines.profile,
                        self.lines.warp,
                        self.lines.wobble,
                        self.lines.glow,
                        self.lines.axis_x,
                        self.lines.axis_y,
                        self.lines.axis_z,
                        self.lines.seed,
                    );
                }
                env_mode::RINGS => {
                    env_rings(
                        1,
                        self.rings.family,
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
                        self.rings.wobble as u32,
                        self.rings.noise,
                        self.rings.dash,
                        self.rings.glow,
                        self.rings.seed,
                    );
                }
                _ => {
                    // Leave overlay as-is (caller can set it explicitly).
                }
            }
        }
    }

    pub fn apply_and_draw(&self) {
        self.apply();
        unsafe {
            draw_env();
        }
    }
}
