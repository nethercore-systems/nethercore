use bytemuck::{Pod, Zeroable};
use glam::Vec3;

use super::quantization::{pack_f16, pack_f16x2};
use zx_common::{pack_octahedral_u16, pack_octahedral_u32};

// ============================================================================
// Environment System (Multi-Environment v4)
// ============================================================================

/// Environment configuration (64 bytes, POD, hashable)
/// Supports 8 procedural modes with layering and blend modes.
///
/// # Header Layout (bits)
/// - 0-2:   base_mode (0-7)
/// - 3-5:   overlay_mode (0-7)
/// - 6-7:   blend_mode (0-3: Alpha, Add, Multiply, Screen)
/// - 8-31:  reserved
///
/// # Data Layout
/// - data[0..7]:   Base layer parameters (28 bytes)
/// - data[7..14]:  Overlay layer parameters (28 bytes)
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Pod, Zeroable)]
pub struct PackedEnvironmentState {
    /// Header: base_mode(3) + overlay_mode(3) + blend_mode(2) + reserved(24)
    pub header: u32,
    /// Padding (reserved for future flags; keeps the struct 16-byte aligned)
    pub _pad: u32,
    /// Mode parameters: base[0..7], overlay[7..14]
    pub data: [u32; 14],
}

// Compile-time size verification
const _: () = assert!(core::mem::size_of::<PackedEnvironmentState>() == 64);
const _: () = assert!(core::mem::align_of::<PackedEnvironmentState>() >= 4);

/// Number of packed `u32` words per layer payload.
pub const ENV_LAYER_WORDS: usize = 7;
/// Packed `u32` word offset for the overlay layer payload.
pub const ENV_OVERLAY_OFFSET: usize = ENV_LAYER_WORDS;

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
    pub const ALPHA: u32 = 0; // lerp(base, overlay, overlay.a)
    pub const ADD: u32 = 1; // base + overlay
    pub const MULTIPLY: u32 = 2; // base * overlay
    pub const SCREEN: u32 = 3; // 1 - (1-base) * (1-overlay)
}

/// Configuration for packing gradient parameters
pub struct GradientConfig {
    pub offset: usize,
    pub zenith: u32,
    pub sky_horizon: u32,
    pub ground_horizon: u32,
    pub nadir: u32,
    pub rotation: f32,
    pub shift: f32,
}

/// Configuration for packing scatter parameters
pub struct ScatterConfig {
    pub offset: usize,
    pub variant: u32,
    pub density: u32,
    pub size: u32,
    pub glow: u32,
    pub streak_length: u32,
    pub color_primary: u32,
    pub color_secondary: u32,
    pub parallax_rate: u32,
    pub parallax_size: u32,
    pub phase: u32,
    pub layer_count: u32,
}

/// Configuration for packing lines parameters
pub struct LinesConfig {
    pub offset: usize,
    pub variant: u32,
    pub line_type: u32,
    pub thickness: u32,
    pub spacing: f32,
    pub fade_distance: f32,
    pub color_primary: u32,
    pub color_accent: u32,
    pub accent_every: u32,
    pub phase: u32,
}

/// Configuration for packing silhouette parameters
pub struct SilhouetteConfig {
    pub offset: usize,
    pub jaggedness: u32,
    pub layer_count: u32,
    pub color_near: u32,
    pub color_far: u32,
    pub sky_zenith: u32,
    pub sky_horizon: u32,
    pub parallax_rate: u32,
    pub seed: u32,
}

/// Configuration for packing rectangles parameters
pub struct RectanglesConfig {
    pub offset: usize,
    pub variant: u32,
    pub density: u32,
    pub lit_ratio: u32,
    pub size_min: u32,
    pub size_max: u32,
    pub aspect: u32,
    pub color_primary: u32,
    pub color_variation: u32,
    pub parallax_rate: u32,
    pub phase: u32,
}

/// Configuration for packing room parameters
pub struct RoomConfig {
    pub offset: usize,
    pub color_ceiling: u32,
    pub color_floor: u32,
    pub color_walls: u32,
    pub panel_size: f32,
    pub panel_gap: u32,
    pub light_direction: Vec3,
    pub light_intensity: u32,
    pub corner_darken: u32,
    pub room_scale: f32,
    pub viewer_x: i32,
    pub viewer_y: i32,
    pub viewer_z: i32,
}

/// Configuration for packing curtains parameters
pub struct CurtainsConfig {
    pub offset: usize,
    pub layer_count: u32,
    pub density: u32,
    pub height_min: u32,
    pub height_max: u32,
    pub width: u32,
    pub spacing: u32,
    pub waviness: u32,
    pub color_near: u32,
    pub color_far: u32,
    pub glow: u32,
    pub parallax_rate: u32,
    pub phase: u32,
}

/// Configuration for packing rings parameters
pub struct RingsConfig {
    pub offset: usize,
    pub ring_count: u32,
    pub thickness: u32,
    pub color_a: u32,
    pub color_b: u32,
    pub center_color: u32,
    pub center_falloff: u32,
    pub spiral_twist: f32,
    pub axis: Vec3,
    pub phase: u32,
}

impl PackedEnvironmentState {
    /// Create header from mode and blend settings
    #[inline]
    pub fn make_header(base_mode: u32, overlay_mode: u32, blend_mode: u32) -> u32 {
        (base_mode & 0x7) | ((overlay_mode & 0x7) << 3) | ((blend_mode & 0x3) << 6)
    }

    /// Get base mode from header
    #[inline]
    pub fn base_mode(&self) -> u32 {
        self.header & 0x7
    }

    /// Get overlay mode from header
    #[inline]
    pub fn overlay_mode(&self) -> u32 {
        (self.header >> 3) & 0x7
    }

    /// Get blend mode from header
    #[inline]
    pub fn blend_mode(&self) -> u32 {
        (self.header >> 6) & 0x3
    }

    /// Set base mode
    #[inline]
    pub fn set_base_mode(&mut self, mode: u32) {
        self.header = (self.header & !0x7) | (mode & 0x7);
    }

    /// Set overlay mode
    #[inline]
    pub fn set_overlay_mode(&mut self, mode: u32) {
        self.header = (self.header & !(0x7 << 3)) | ((mode & 0x7) << 3);
    }

    /// Set blend mode
    #[inline]
    pub fn set_blend_mode(&mut self, mode: u32) {
        self.header = (self.header & !(0x3 << 6)) | ((mode & 0x3) << 6);
    }

    /// Pack gradient parameters into data[offset..offset+7]
    /// Mode 0: Gradient - 4-color sky/ground gradient
    pub fn pack_gradient(&mut self, config: GradientConfig) {
        let GradientConfig {
            offset,
            zenith,
            sky_horizon,
            ground_horizon,
            nadir,
            rotation,
            shift,
        } = config;
        self.data[offset] = zenith;
        self.data[offset + 1] = sky_horizon;
        self.data[offset + 2] = ground_horizon;
        self.data[offset + 3] = nadir;
        self.data[offset + 4] = pack_f16x2(rotation, shift);
        self.data[offset + 5] = 0;
        self.data[offset + 6] = 0;
    }

    /// Create a default gradient environment (blue sky)
    pub fn default_gradient() -> Self {
        let mut env = Self {
            header: Self::make_header(env_mode::GRADIENT, env_mode::GRADIENT, blend_mode::ALPHA),
            ..Default::default()
        };
        // Blue sky defaults
        env.pack_gradient(GradientConfig {
            offset: 0,
            zenith: 0x3366B2FF,         // darker blue
            sky_horizon: 0xB2D8F2FF,    // light blue
            ground_horizon: 0x8B7355FF, // tan/brown
            nadir: 0x4A3728FF,          // dark brown
            rotation: 0.0,
            shift: 0.0,
        });
        env
    }

    /// Pack scatter parameters into data[offset..offset+7]
    /// Mode 1: Scatter - Cellular noise particle field with parallax layers
    ///
    /// # Data Layout
    /// - data[0]: variant(2) + density(8) + size(8) + glow(8) + streak_len(6) = 32 bits
    /// - data[1]: color_primary(RGB8, 24) + parallax_rate(8) = 32 bits
    /// - data[2]: color_secondary(RGB8, 24) + parallax_size(8) = 32 bits
    /// - data[3]: phase(u16, 16) + layer_count(2) + reserved(14) = 32 bits
    /// - data[4]: reserved
    pub fn pack_scatter(&mut self, config: ScatterConfig) {
        let ScatterConfig {
            offset,
            variant,
            density,
            size,
            glow,
            streak_length,
            color_primary,
            color_secondary,
            parallax_rate,
            parallax_size,
            phase,
            layer_count,
        } = config;
        // data[0]: variant(2) + density(8) + size(8) + glow(8) + streak_len(6)
        self.data[offset] = (variant & 0x3)
            | ((density & 0xFF) << 2)
            | ((size & 0xFF) << 10)
            | ((glow & 0xFF) << 18)
            | ((streak_length & 0x3F) << 26);

        // data[1]: color_primary RGB (bits 31-8) + parallax_rate (bits 7-0)
        self.data[offset + 1] = (color_primary & 0xFFFFFF00) | (parallax_rate & 0xFF);

        // data[2]: color_secondary RGB (bits 31-8) + parallax_size (bits 7-0)
        self.data[offset + 2] = (color_secondary & 0xFFFFFF00) | (parallax_size & 0xFF);

        // data[3]: phase(16) + layer_count(2) + reserved(14)
        self.data[offset + 3] = (phase & 0xFFFF) | (((layer_count.clamp(1, 3) - 1) & 0x3) << 16);

        // data[4]: reserved
        self.data[offset + 4] = 0;
        self.data[offset + 5] = 0;
        self.data[offset + 6] = 0;
    }

    /// Pack lines parameters into data[offset..offset+7]
    /// Mode 2: Lines - Infinite grid lines projected onto a plane
    ///
    /// # Data Layout
    /// - data[0]: variant(2) + line_type(2) + thickness(8) + accent_every(8) + reserved(12)
    /// - data[1]: spacing(f16) + fade_distance(f16)
    /// - data[2]: color_primary (RGBA8)
    /// - data[3]: color_accent (RGBA8)
    /// - data[4]: phase(u16) + reserved(16)
    pub fn pack_lines(&mut self, config: LinesConfig) {
        let LinesConfig {
            offset,
            variant,
            line_type,
            thickness,
            spacing,
            fade_distance,
            color_primary,
            color_accent,
            accent_every,
            phase,
        } = config;
        // data[0]: variant(2) + line_type(2) + thickness(8) + accent_every(8) + reserved(12)
        self.data[offset] = (variant & 0x3)
            | ((line_type & 0x3) << 2)
            | ((thickness & 0xFF) << 4)
            | ((accent_every & 0xFF) << 12);

        // data[1]: spacing(f16) + fade_distance(f16)
        self.data[offset + 1] = pack_f16x2(spacing, fade_distance);

        // data[2]: color_primary (RGBA8)
        self.data[offset + 2] = color_primary;

        // data[3]: color_accent (RGBA8)
        self.data[offset + 3] = color_accent;

        // data[4]: phase(u16) + reserved(16)
        self.data[offset + 4] = phase & 0xFFFF;
        self.data[offset + 5] = 0;
        self.data[offset + 6] = 0;
    }

    /// Pack silhouette parameters into data[offset..offset+7]
    /// Mode 3: Silhouette - Layered terrain silhouettes with parallax
    ///
    /// # Data Layout
    /// - data[0]: jaggedness(8) + layer_count(2) + parallax_rate(8) + reserved(14)
    /// - data[1]: color_near (RGBA8)
    /// - data[2]: color_far (RGBA8)
    /// - data[3]: sky_zenith (RGBA8)
    /// - data[4]: sky_horizon (RGBA8)
    /// - data[5]: seed (u32)
    pub fn pack_silhouette(&mut self, config: SilhouetteConfig) {
        let SilhouetteConfig {
            offset,
            jaggedness,
            layer_count,
            color_near,
            color_far,
            sky_zenith,
            sky_horizon,
            parallax_rate,
            seed,
        } = config;
        // data[0]: jaggedness(8) + layer_count(2) + parallax_rate(8) + reserved(14)
        self.data[offset] = (jaggedness & 0xFF)
            | (((layer_count.clamp(1, 3) - 1) & 0x3) << 8)
            | ((parallax_rate & 0xFF) << 10);

        // data[1]: color_near (RGBA8)
        self.data[offset + 1] = color_near;

        // data[2]: color_far (RGBA8)
        self.data[offset + 2] = color_far;

        // data[3]: sky_zenith (RGBA8)
        self.data[offset + 3] = sky_zenith;

        // data[4]: sky_horizon (RGBA8)
        self.data[offset + 4] = sky_horizon;

        // data[5]: seed (full u32; previously limited to a shared 16-bit lane)
        self.data[offset + 5] = seed;
        self.data[offset + 6] = 0;
    }

    /// Pack rectangles parameters into data[offset..offset+7]
    /// Mode 4: Rectangles - Rectangular light sources (windows, screens, panels)
    ///
    /// # Data Layout
    /// - data[0]: variant(2) + density(8) + lit_ratio(8) + size_min(6) + size_max(6) + aspect(2)
    /// - data[1]: color_primary (RGBA8)
    /// - data[2]: color_variation (RGBA8)
    /// - data[3]: parallax_rate(8) + reserved(8) + phase(16)
    /// - data[4]: reserved
    pub fn pack_rectangles(&mut self, config: RectanglesConfig) {
        let RectanglesConfig {
            offset,
            variant,
            density,
            lit_ratio,
            size_min,
            size_max,
            aspect,
            color_primary,
            color_variation,
            parallax_rate,
            phase,
        } = config;
        // data[0]: variant(2) + density(8) + lit_ratio(8) + size_min(6) + size_max(6) + aspect(2)
        self.data[offset] = (variant & 0x3)
            | ((density & 0xFF) << 2)
            | ((lit_ratio & 0xFF) << 10)
            | ((size_min & 0x3F) << 18)
            | ((size_max & 0x3F) << 24)
            | ((aspect & 0x3) << 30);

        // data[1]: color_primary (RGBA8)
        self.data[offset + 1] = color_primary;

        // data[2]: color_variation (RGBA8)
        self.data[offset + 2] = color_variation;

        // data[3]: parallax_rate(8) + reserved(8) + phase(16)
        self.data[offset + 3] = (parallax_rate & 0xFF) | ((phase & 0xFFFF) << 16);

        // data[4]: reserved
        self.data[offset + 4] = 0;
        self.data[offset + 5] = 0;
        self.data[offset + 6] = 0;
    }

    /// Pack room parameters into data[offset..offset+7]
    /// Mode 5: Room - Interior of a 3D box with directional lighting
    ///
    /// # Data Layout (viewer packed into color alpha bytes - rooms are opaque)
    ///
    /// - data[0]: color_ceiling_RGB(24) + viewer_x_snorm8(8)
    /// - data[1]: color_floor_RGB(24) + viewer_y_snorm8(8)
    /// - data[2]: color_walls_RGB(24) + viewer_z_snorm8(8)
    /// - data[3]: panel_size(f16) + panel_gap(8) + corner_darken(8)
    /// - data[4]: light_dir_oct(16) + light_intensity(8) + room_scale(8)
    ///
    /// Note: Does NOT use per-layer extension words - can safely layer with other modes
    pub fn pack_room(&mut self, config: RoomConfig) {
        let RoomConfig {
            offset,
            color_ceiling,
            color_floor,
            color_walls,
            panel_size,
            panel_gap,
            light_direction,
            light_intensity,
            corner_darken,
            room_scale,
            viewer_x,
            viewer_y,
            viewer_z,
        } = config;
        // Convert viewer positions from i32 to snorm8 (clamp to -128..127, store as u8)
        let vx = ((viewer_x.clamp(-128, 127) as i8) as u8) as u32;
        let vy = ((viewer_y.clamp(-128, 127) as i8) as u8) as u32;
        let vz = ((viewer_z.clamp(-128, 127) as i8) as u8) as u32;

        // data[0]: color_ceiling RGB (bits 31-8) + viewer_x snorm8 (bits 7-0)
        self.data[offset] = (color_ceiling & 0xFFFFFF00) | vx;

        // data[1]: color_floor RGB (bits 31-8) + viewer_y snorm8 (bits 7-0)
        self.data[offset + 1] = (color_floor & 0xFFFFFF00) | vy;

        // data[2]: color_walls RGB (bits 31-8) + viewer_z snorm8 (bits 7-0)
        self.data[offset + 2] = (color_walls & 0xFFFFFF00) | vz;

        // data[3]: panel_size(f16, bits 0-15) + panel_gap(8, bits 16-23) + corner_darken(8, bits 24-31)
        let panel_size_bits = pack_f16(panel_size) as u32;
        self.data[offset + 3] =
            panel_size_bits | ((panel_gap & 0xFF) << 16) | ((corner_darken & 0xFF) << 24);

        // data[4]: light_dir_oct(16, bits 0-15) + light_intensity(8, bits 16-23) + room_scale(8, bits 24-31)
        let light_oct = pack_octahedral_u32(light_direction.normalize_or_zero());
        let room_scale_packed = ((room_scale.clamp(0.1, 25.5) * 10.0) as u32) & 0xFF;
        self.data[offset + 4] =
            (light_oct & 0xFFFF) | ((light_intensity & 0xFF) << 16) | (room_scale_packed << 24);
        self.data[offset + 5] = 0;
        self.data[offset + 6] = 0;
    }

    /// Pack curtains parameters into data[offset..offset+7]
    /// Mode 6: Curtains - Vertical structures (pillars, trees) around viewer
    ///
    /// # Data Layout
    /// - data[0]: layer_count(2) + density(8) + height_min(6) + height_max(6) + width(5) + spacing(5)
    /// - data[1]: waviness(8) + glow(8) + parallax_rate(8) + reserved(8)
    /// - data[2]: color_near (RGBA8)
    /// - data[3]: color_far (RGBA8)
    /// - data[4]: phase(u16) + reserved(16)
    pub fn pack_curtains(&mut self, config: CurtainsConfig) {
        let CurtainsConfig {
            offset,
            layer_count,
            density,
            height_min,
            height_max,
            width,
            spacing,
            waviness,
            color_near,
            color_far,
            glow,
            parallax_rate,
            phase,
        } = config;
        // data[0]: layer_count(2) + density(8) + height_min(6) + height_max(6) + width(5) + spacing(5)
        self.data[offset] = ((layer_count.clamp(1, 3) - 1) & 0x3)
            | ((density & 0xFF) << 2)
            | ((height_min & 0x3F) << 10)
            | ((height_max & 0x3F) << 16)
            | ((width & 0x1F) << 22)
            | ((spacing & 0x1F) << 27);

        // data[1]: waviness(8) + glow(8) + parallax_rate(8) + reserved(8)
        self.data[offset + 1] =
            (waviness & 0xFF) | ((glow & 0xFF) << 8) | ((parallax_rate & 0xFF) << 16);

        // data[2]: color_near (RGBA8)
        self.data[offset + 2] = color_near;

        // data[3]: color_far (RGBA8)
        self.data[offset + 3] = color_far;

        // data[4]: phase(u16) + reserved(16)
        self.data[offset + 4] = phase & 0xFFFF;
        self.data[offset + 5] = 0;
        self.data[offset + 6] = 0;
    }

    /// Pack rings parameters into data[offset..offset+7]
    /// Mode 7: Rings - Concentric rings around focal direction (tunnel/portal/vortex)
    ///
    /// # Data Layout
    ///
    /// - data[0]: ring_count(8) + thickness(8) + center_falloff(8) + reserved(8)
    /// - data[1]: color_a (RGBA8)
    /// - data[2]: color_b (RGBA8)
    /// - data[3]: center_color (RGBA8)
    /// - data[4]: spiral_twist(f16) + axis_oct(16)
    /// - data[5]: phase (u16 stored in low 16 bits)
    pub fn pack_rings(&mut self, config: RingsConfig) {
        let RingsConfig {
            offset,
            ring_count,
            thickness,
            color_a,
            color_b,
            center_color,
            center_falloff,
            spiral_twist,
            axis,
            phase,
        } = config;
        // data[0]: ring_count(8) + thickness(8) + center_falloff(8) + reserved(8)
        self.data[offset] =
            (ring_count & 0xFF) | ((thickness & 0xFF) << 8) | ((center_falloff & 0xFF) << 16);

        // data[1]: color_a (RGBA8)
        self.data[offset + 1] = color_a;

        // data[2]: color_b (RGBA8)
        self.data[offset + 2] = color_b;

        // data[3]: center_color (RGBA8)
        self.data[offset + 3] = center_color;

        // data[4]: spiral_twist(f16, bits 0-15) + axis_oct16(16, bits 16-31)
        // Using 16-bit octahedral (2x snorm8) for axis - ~1.4° precision, fits in 16 bits
        let axis_oct = pack_octahedral_u16(axis.normalize_or_zero()) as u32;
        let twist_bits = pack_f16(spiral_twist) as u32;
        self.data[offset + 4] = twist_bits | (axis_oct << 16);

        // data[5]: phase (previously stored in a shared 16-bit lane)
        self.data[offset + 5] = phase & 0xFFFF;
        self.data[offset + 6] = 0;
    }
}

/// Handle to interned environment state (newtype for type safety)
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub struct EnvironmentIndex(pub u32);

impl EnvironmentIndex {
    pub const INVALID: Self = Self(u32::MAX);
}

impl crate::state::PoolIndex for EnvironmentIndex {
    fn from_raw(value: u32) -> Self {
        EnvironmentIndex(value)
    }

    fn as_raw(&self) -> u32 {
        self.0
    }
}
