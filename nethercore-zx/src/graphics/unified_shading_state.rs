use bytemuck::{Pod, Zeroable};
use glam::Vec3;
use half::f16;
use zx_common::{pack_octahedral_u16, pack_octahedral_u32, unpack_octahedral_u32};

use super::render_state::MatcapBlendMode;

// ============================================================================
// Environment System (Multi-Environment v3)
// ============================================================================

/// Environment configuration (48 bytes, POD, hashable)
/// Supports 8 procedural modes with layering and blend modes.
///
/// # Header Layout (bits)
/// - 0-2:   base_mode (0-7)
/// - 3-5:   overlay_mode (0-7)
/// - 6-7:   blend_mode (0-3: Alpha, Add, Multiply, Screen)
/// - 8-31:  reserved
///
/// # Data Layout
/// - data[0..5]:  Base mode parameters (20 bytes)
/// - data[5..10]: Overlay mode parameters (20 bytes)
/// - data[10]:    Shared/overflow (4 bytes)
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Pod, Zeroable)]
pub struct PackedEnvironmentState {
    /// Header: base_mode(3) + overlay_mode(3) + blend_mode(2) + reserved(24)
    pub header: u32,
    /// Mode parameters: base[0..5], overlay[5..10], shared[10]
    pub data: [u32; 11],
}

// Compile-time size verification
const _: () = assert!(core::mem::size_of::<PackedEnvironmentState>() == 48);
const _: () = assert!(core::mem::align_of::<PackedEnvironmentState>() >= 4);

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

    /// Pack gradient parameters into data[offset..offset+5]
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

    /// Pack scatter parameters into data[offset..offset+5]
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
    }

    /// Pack lines parameters into data[offset..offset+5]
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
    }

    /// Pack silhouette parameters into data[offset..offset+5]
    /// Mode 3: Silhouette - Layered terrain silhouettes with parallax
    ///
    /// # Data Layout
    /// - data[0]: jaggedness(8) + layer_count(2) + parallax_rate(8) + reserved(14)
    /// - data[1]: color_near (RGBA8)
    /// - data[2]: color_far (RGBA8)
    /// - data[3]: sky_zenith (RGBA8)
    /// - data[4]: sky_horizon (RGBA8) - seed stored in shared data[10]
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

        // Store seed in shared data[10]
        if offset == 0 {
            self.data[10] = (self.data[10] & 0x0000FFFF) | ((seed & 0xFFFF) << 16);
        } else {
            self.data[10] = (self.data[10] & 0xFFFF0000) | (seed & 0xFFFF);
        }
    }

    /// Pack rectangles parameters into data[offset..offset+5]
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
    }

    /// Pack room parameters into data[offset..offset+5]
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
    /// Note: Does NOT use shared data[10] - can safely layer with other modes
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
    }

    /// Pack curtains parameters into data[offset..offset+5]
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
    }

    /// Pack rings parameters into data[offset..offset+5]
    /// Mode 7: Rings - Concentric rings around focal direction (tunnel/portal/vortex)
    ///
    /// # Data Layout
    ///
    /// - data[0]: ring_count(8) + thickness(8) + center_falloff(8) + reserved(8)
    /// - data[1]: color_a (RGBA8)
    /// - data[2]: color_b (RGBA8)
    /// - data[3]: center_color (RGBA8)
    /// - data[4]: spiral_twist(f16) + axis_oct(16)
    ///
    /// Note: phase is stored in shared data[10] upper 16 bits
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

        // Store phase in shared data[10] - upper 16 bits for rings
        // Note: Only works when rings is base mode (offset=0) or we use a different storage
        // For now, store in data[10] which is shared
        if offset == 0 {
            self.data[10] = (self.data[10] & 0x0000FFFF) | ((phase & 0xFFFF) << 16);
        } else {
            // For overlay at offset 5, store phase in data[10] lower 16 bits
            self.data[10] = (self.data[10] & 0xFFFF0000) | (phase & 0xFFFF);
        }
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

/// Light type stored in bit 7 of data1
#[repr(u8)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash)]
pub enum LightType {
    #[default]
    Directional = 0,
    Point = 1,
}

impl LightType {
    pub fn from_bit(bit: bool) -> Self {
        if bit {
            LightType::Point
        } else {
            LightType::Directional
        }
    }

    pub fn to_bit(self) -> bool {
        matches!(self, LightType::Point)
    }
}

/// One packed light (12 bytes) - supports directional and point lights
///
/// # Format
///
/// **data0:**
///
/// - Directional: octahedral direction (snorm16x2)
/// - Point: position XY (f16x2)
///
/// **data1:** RGB8 (bits 31-8) + type (bit 7) + intensity (bits 6-0)
///
/// - Format: 0xRRGGBB_TI where T=type(1bit), I=intensity(7bits)
/// - Intensity maps 0-127 -> 0.0-8.0 for HDR support
///
/// **data2:**
///
/// - Directional: unused (0)
/// - Point: position Z (f16, bits 15-0) + range (f16, bits 31-16)
#[repr(C)]
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Hash, Pod, Zeroable)]
pub struct PackedLight {
    /// Directional: octahedral direction (snorm16x2)
    /// Point: position XY (f16x2)
    pub data0: u32,

    /// RGB8 (bits 31-8) + type (bit 7) + intensity (bits 6-0)
    /// Format: 0xRRGGBB_TI where T=type(1bit), I=intensity(7bits)
    pub data1: u32,

    /// Directional: unused (0)
    /// Point: position Z (f16, bits 15-0) + range (f16, bits 31-16)
    pub data2: u32,
}

/// Unified per-draw shading state (80 bytes, POD, hashable)
/// Size breakdown: 16 bytes (header) + 48 bytes (lights) + 16 bytes (animation/environment)
///
/// # Mode-Specific Field Interpretation
///
/// The `uniform_set_0` and `uniform_set_1` fields are interpreted differently per render mode.
/// Each is a u32 containing 4 packed u8 values: [byte0, byte1, byte2, byte3].
///
/// Field layout per render mode:
///
/// | Mode | uniform_set_0 [b0, b1, b2, b3]                   | uniform_set_1 [b0, b1, b2, b3]           |
/// |------|--------------------------------------------------|------------------------------------------|
/// | 0    | [unused, unused, unused, Rim Intensity]          | [unused, unused, unused, Rim Power]      |
/// | 1    | [BlendMode0, BlendMode1, BlendMode2, BlendMode3] | [unused, unused, unused, unused]         |
/// | 2    | [Metallic, Roughness, Emissive, Rim Intensity]   | [unused, unused, unused, Rim Power]      |
/// | 3    | [SpecDamping*, Shininess, Emissive, RimIntens]   | [Spec R, Spec G, Spec B, Rim Power]      |
///
/// *SpecDamping is INVERTED: 0=full specular (default), 255=no specular.
/// This is beginner-friendly since the default of 0 gives visible highlights.
#[repr(C)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash, Pod, Zeroable)]
pub struct PackedUnifiedShadingState {
    /// Material color (RGBA8 packed)
    pub color_rgba8: u32,

    /// Mode-specific uniform data (4 bytes packed as 4 × u8)
    /// - Mode 0: [unused, unused, unused, rim_intensity]
    /// - Mode 1: [blend_mode_0, blend_mode_1, blend_mode_2, blend_mode_3]
    /// - Mode 2: [metallic, roughness, emissive, rim_intensity]
    /// - Mode 3: [spec_damping*, shininess, emissive, rim_intensity]
    /// * spec_damping is inverted: 0=full specular, 255=no specular
    pub uniform_set_0: u32,

    /// Mode-specific extended data (4 bytes packed as 4 × u8)
    /// - Mode 0: [unused, unused, unused, rim_power]
    /// - Mode 1: unused
    /// - Mode 2: [unused, unused, unused, rim_power]
    /// - Mode 3: [spec_r, spec_g, spec_b, rim_power]
    pub uniform_set_1: u32,

    /// Flags and reserved bits
    /// - Bit 0: skinning_mode (0 = raw, 1 = inverse bind mode)
    /// - Bits 1-31: reserved for future use
    pub flags: u32,

    pub lights: [PackedLight; 4], // 48 bytes (4 × 12-byte lights)

    // Animation System v2 fields (12 bytes)
    /// Base offset into @binding(7) all_keyframes buffer
    /// Shader reads: all_keyframes[keyframe_base + bone_index]
    /// 0 = no keyframes bound (use bones buffer directly)
    pub keyframe_base: u32,

    /// Base offset into @binding(6) all_inverse_bind buffer
    /// Shader reads: inverse_bind[inverse_bind_base + bone_index]
    /// 0 = no skeleton bound (raw bone mode)
    pub inverse_bind_base: u32,

    /// Padding for struct alignment (animation_flags slot unused)
    pub _pad: u32,

    /// Index into environment_states buffer for sky/environment rendering
    /// References a PackedEnvironmentState in the GPU buffer
    pub environment_index: u32,
}

impl Default for PackedUnifiedShadingState {
    fn default() -> Self {
        // uniform_set_0: [metallic=0, roughness=128, emissive=0, rim_intensity=0]
        let uniform_set_0 = pack_uniform_set_0(0, 128, 0, 0);
        // uniform_set_1: [spec_r=255, spec_g=255, spec_b=255, rim_power=0] (white specular)
        let uniform_set_1 = pack_uniform_set_1(255, 255, 255, 0);

        Self {
            color_rgba8: 0xFFFFFFFF, // White
            uniform_set_0,
            uniform_set_1,
            flags: DEFAULT_FLAGS, // uniform_alpha = 15 (opaque), other flags = 0
            lights: [PackedLight::default(); 4], // All lights disabled
            // Animation System v2 fields (default to no animation)
            keyframe_base: 0,     // No keyframes bound
            inverse_bind_base: 0, // No skeleton bound (raw bone mode)
            _pad: 0,
            environment_index: 0, // Index 0 = default environment
        }
    }
}

/// Handle to interned shading state (newtype for clarity and type safety)
#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct ShadingStateIndex(pub u32);

impl ShadingStateIndex {
    pub const INVALID: Self = Self(u32::MAX);
}

impl crate::state::PoolIndex for ShadingStateIndex {
    fn from_raw(value: u32) -> Self {
        ShadingStateIndex(value)
    }

    fn as_raw(&self) -> u32 {
        self.0
    }
}

// ============================================================================
// Quantization Helper Functions
// ============================================================================

/// Pack an f32 color channel [0.0, 1.0] to u8 [0, 255]
#[inline]
pub fn pack_unorm8(value: f32) -> u8 {
    (value.clamp(0.0, 1.0) * 255.0).round() as u8
}

/// Unpack u8 [0, 255] to f32 [0.0, 1.0]
#[inline]
pub fn unpack_unorm8(value: u8) -> f32 {
    value as f32 / 255.0
}

/// Pack an f32 normalized value [-1.0, 1.0] to i16 snorm16 [-32767, 32767]
#[inline]
#[allow(dead_code)] // Used in tests
pub fn pack_snorm16(value: f32) -> i16 {
    (value.clamp(-1.0, 1.0) * 32767.0).round() as i16
}

/// Unpack snorm16 [-32767, 32767] to f32 [-1.0, 1.0]
#[inline]
#[allow(dead_code)] // Reserved for future use
pub fn unpack_snorm16(value: i16) -> f32 {
    value as f32 / 32767.0
}

/// Pack f32 to IEEE 754 half-precision float (f16) stored as u16
#[inline]
pub fn pack_f16(value: f32) -> u16 {
    f16::from_f32(value).to_bits()
}

/// Unpack IEEE 754 half-precision float (f16) from u16 to f32
#[inline]
pub fn unpack_f16(bits: u16) -> f32 {
    f16::from_bits(bits).to_f32()
}

/// Pack two f32 values into a u32 as f16x2
#[inline]
pub fn pack_f16x2(x: f32, y: f32) -> u32 {
    let x_bits = pack_f16(x) as u32;
    let y_bits = pack_f16(y) as u32;
    x_bits | (y_bits << 16)
}

/// Unpack u32 to two f32 values from f16x2
#[inline]
pub fn unpack_f16x2(packed: u32) -> (f32, f32) {
    let x = unpack_f16((packed & 0xFFFF) as u16);
    let y = unpack_f16((packed >> 16) as u16);
    (x, y)
}

/// Pack RGBA f32 [0.0, 1.0] to u32 RGBA8
/// Format: 0xRRGGBBAA (R in highest byte, A in lowest)
#[inline]
pub fn pack_rgba8(r: f32, g: f32, b: f32, a: f32) -> u32 {
    let r = pack_unorm8(r);
    let g = pack_unorm8(g);
    let b = pack_unorm8(b);
    let a = pack_unorm8(a);
    ((r as u32) << 24) | ((g as u32) << 16) | ((b as u32) << 8) | (a as u32)
}

/// Pack Vec3 color [0.0, 1.0] to u32 RGB8 (alpha = 255)
#[inline]
pub fn pack_rgb8(color: Vec3) -> u32 {
    pack_rgba8(color.x, color.y, color.z, 1.0)
}

/// Pack 4x MatcapBlendMode into u32 (4 bytes)
#[inline]
pub fn pack_matcap_blend_modes(modes: [MatcapBlendMode; 4]) -> u32 {
    (modes[0] as u32)
        | ((modes[1] as u32) << 8)
        | ((modes[2] as u32) << 16)
        | ((modes[3] as u32) << 24)
}

// ============================================================================
// Uniform Set Packing Helpers
// ============================================================================

/// Pack 4 u8 values into uniform_set_0
/// Layout: [byte0, byte1, byte2, byte3] where byte0 is in low bits
/// - Mode 0: [unused, unused, unused, rim_intensity]
/// - Mode 1: [blend_mode_0, blend_mode_1, blend_mode_2, blend_mode_3]
/// - Mode 2: [metallic, roughness, emissive, rim_intensity]
/// - Mode 3: [spec_intensity, shininess, emissive, rim_intensity]
#[inline]
pub fn pack_uniform_set_0(byte0: u8, byte1: u8, byte2: u8, byte3: u8) -> u32 {
    (byte0 as u32) | ((byte1 as u32) << 8) | ((byte2 as u32) << 16) | ((byte3 as u32) << 24)
}

/// Pack 4 u8 values into uniform_set_1
/// Layout: [byte0, byte1, byte2, byte3] where byte0 is in low bits
/// - Mode 0: [unused, unused, unused, rim_power]
/// - Mode 1: unused
/// - Mode 2: [unused, unused, unused, rim_power]
/// - Mode 3: [spec_r, spec_g, spec_b, rim_power]
#[inline]
pub fn pack_uniform_set_1(byte0: u8, byte1: u8, byte2: u8, byte3: u8) -> u32 {
    (byte0 as u32) | ((byte1 as u32) << 8) | ((byte2 as u32) << 16) | ((byte3 as u32) << 24)
}

/// Update a specific byte in a packed u32 value
///
/// # Arguments
/// * `current` - The current packed u32 value
/// * `byte_index` - Which byte to update (0-3, where 0 is lowest byte)
/// * `value` - The new byte value
#[inline]
pub fn update_u32_byte(current: u32, byte_index: u8, value: u8) -> u32 {
    let shift = byte_index as u32 * 8;
    let mask = !(0xFFu32 << shift);
    (current & mask) | ((value as u32) << shift)
}

// Backwards compatibility aliases
#[inline]
pub fn update_uniform_set_0_byte(current: u32, byte_index: u8, value: u8) -> u32 {
    update_u32_byte(current, byte_index, value)
}

#[inline]
pub fn update_uniform_set_1_byte(current: u32, byte_index: u8, value: u8) -> u32 {
    update_u32_byte(current, byte_index, value)
}

/// Unpack matcap blend modes from u32
pub fn unpack_matcap_blend_modes(packed: u32) -> [MatcapBlendMode; 4] {
    [
        MatcapBlendMode::from_u8((packed & 0xFF) as u8),
        MatcapBlendMode::from_u8(((packed >> 8) & 0xFF) as u8),
        MatcapBlendMode::from_u8(((packed >> 16) & 0xFF) as u8),
        MatcapBlendMode::from_u8(((packed >> 24) & 0xFF) as u8),
    ]
}

// ============================================================================
// PackedLight Helpers
// ============================================================================

impl PackedLight {
    /// Create a directional light
    pub fn directional(direction: Vec3, color: Vec3, intensity: f32, enabled: bool) -> Self {
        let data0 = pack_octahedral_u32(direction.normalize_or_zero());
        let data1 =
            Self::pack_color_type_intensity(color, LightType::Directional, intensity, enabled);
        Self {
            data0,
            data1,
            data2: 0,
        }
    }

    /// Create a point light
    pub fn point(position: Vec3, color: Vec3, intensity: f32, range: f32, enabled: bool) -> Self {
        let data0 = pack_f16x2(position.x, position.y);
        let data1 = Self::pack_color_type_intensity(color, LightType::Point, intensity, enabled);
        let data2 = pack_f16x2(position.z, range);
        Self {
            data0,
            data1,
            data2,
        }
    }

    /// Pack color, type, and intensity into data1
    /// Format: 0xRRGGBB_TI where T=type(1bit), I=intensity(7bits)
    fn pack_color_type_intensity(
        color: Vec3,
        light_type: LightType,
        intensity: f32,
        enabled: bool,
    ) -> u32 {
        let r = pack_unorm8(color.x);
        let g = pack_unorm8(color.y);
        let b = pack_unorm8(color.z);

        // Intensity: 0.0-8.0 -> 0-127 (7 bits)
        // If disabled, set to 0
        let intensity_7bit = if enabled {
            ((intensity / 8.0).clamp(0.0, 1.0) * 127.0).round() as u8
        } else {
            0
        };

        // Type in bit 7, intensity in bits 0-6
        let type_intensity = ((light_type as u8) << 7) | (intensity_7bit & 0x7F);

        ((r as u32) << 24) | ((g as u32) << 16) | ((b as u32) << 8) | (type_intensity as u32)
    }

    /// Create a PackedLight from f32 parameters (directional light)
    /// Backward compatibility: delegates to directional()
    /// If enabled=false, intensity is set to 0 (which indicates disabled light)
    pub fn from_floats(direction: Vec3, color: Vec3, intensity: f32, enabled: bool) -> Self {
        Self::directional(direction, color, intensity, enabled)
    }

    /// Create a disabled light (all zeros)
    pub fn disabled() -> Self {
        Self::default()
    }

    /// Get light type (directional or point)
    pub fn get_type(&self) -> LightType {
        LightType::from_bit((self.data1 & 0x80) != 0)
    }

    /// Extract direction as f32 array (only valid for directional lights)
    /// Decodes the octahedral-encoded direction stored in data0.
    pub fn get_direction(&self) -> [f32; 3] {
        let dir = unpack_octahedral_u32(self.data0);
        [dir.x, dir.y, dir.z]
    }

    /// Get position (only valid for point lights)
    pub fn get_position(&self) -> [f32; 3] {
        let (x, y) = unpack_f16x2(self.data0);
        let (z, _) = unpack_f16x2(self.data2);
        [x, y, z]
    }

    /// Get range (only valid for point lights)
    pub fn get_range(&self) -> f32 {
        let (_, range) = unpack_f16x2(self.data2);
        range
    }

    /// Extract color as f32 array
    /// Format: 0xRRGGBB_TI (R in highest byte, type+intensity in lowest byte)
    pub fn get_color(&self) -> [f32; 3] {
        let r = unpack_unorm8(((self.data1 >> 24) & 0xFF) as u8);
        let g = unpack_unorm8(((self.data1 >> 16) & 0xFF) as u8);
        let b = unpack_unorm8(((self.data1 >> 8) & 0xFF) as u8);
        [r, g, b]
    }

    /// Extract intensity as f32 (0.0-8.0 range)
    /// Intensity is stored in bits 0-6 of data1
    pub fn get_intensity(&self) -> f32 {
        let intensity_7bit = (self.data1 & 0x7F) as f32;
        intensity_7bit / 127.0 * 8.0
    }

    /// Check if light is enabled (intensity > 0)
    pub fn is_enabled(&self) -> bool {
        (self.data1 & 0x7F) != 0
    }
}

// ============================================================================
// PackedUnifiedShadingState Helpers
// ============================================================================

/// Flag bit for skinning mode in PackedUnifiedShadingState.flags
/// 0 = raw mode (matrices used as-is), 1 = inverse bind mode
pub const FLAG_SKINNING_MODE: u32 = 1 << 0;

/// Flag bit for texture filter mode in PackedUnifiedShadingState.flags
/// 0 = nearest (pixelated), 1 = linear (smooth)
pub const FLAG_TEXTURE_FILTER_LINEAR: u32 = 1 << 1;

// ============================================================================
// Animation System v2 (Unified Buffer)
// ============================================================================
// NOTE: ANIMATION_FLAG_USE_IMMEDIATE removed - unified_animation buffer uses
// pre-computed offsets. The shader just reads from unified_animation[keyframe_base + bone_idx].

// ============================================================================
// Material Override Flags (bits 2-7)
// ============================================================================

/// Flag bit for uniform color override (bit 2)
pub const FLAG_USE_UNIFORM_COLOR: u32 = 1 << 2;
/// Flag bit for uniform metallic override (bit 3)
pub const FLAG_USE_UNIFORM_METALLIC: u32 = 1 << 3;
/// Flag bit for uniform roughness override (bit 4)
pub const FLAG_USE_UNIFORM_ROUGHNESS: u32 = 1 << 4;
/// Flag bit for uniform emissive override (bit 5)
pub const FLAG_USE_UNIFORM_EMISSIVE: u32 = 1 << 5;
/// Flag bit for uniform specular override (bit 6, Mode 3 only)
pub const FLAG_USE_UNIFORM_SPECULAR: u32 = 1 << 6;
/// Flag bit for matcap vs sky reflection (bit 7, Mode 1 only)
pub const FLAG_USE_MATCAP_REFLECTION: u32 = 1 << 7;

// ============================================================================
// Dither Transparency Flags (Bits 8-15)
// ============================================================================

/// Mask for uniform alpha level in flags (bits 8-11)
/// Values 0-15: 0 = fully transparent, 15 = fully opaque (default)
pub const FLAG_UNIFORM_ALPHA_MASK: u32 = 0xF << 8;
/// Bit shift for uniform alpha level
pub const FLAG_UNIFORM_ALPHA_SHIFT: u32 = 8;

/// Mask for dither offset X in flags (bits 12-13)
/// Values 0-3: pixel shift in X axis
pub const FLAG_DITHER_OFFSET_X_MASK: u32 = 0x3 << 12;
/// Bit shift for dither offset X
pub const FLAG_DITHER_OFFSET_X_SHIFT: u32 = 12;

/// Mask for dither offset Y in flags (bits 14-15)
/// Values 0-3: pixel shift in Y axis
pub const FLAG_DITHER_OFFSET_Y_MASK: u32 = 0x3 << 14;
/// Bit shift for dither offset Y
pub const FLAG_DITHER_OFFSET_Y_SHIFT: u32 = 14;

/// Default flags value with uniform_alpha = 15 (opaque)
pub const DEFAULT_FLAGS: u32 = 0xF << 8;

// ============================================================================
// Normal Mapping Flags (Bit 16)
// ============================================================================

/// Flag bit to disable normal map sampling in PackedUnifiedShadingState.flags
/// When NOT set (default) and mesh has tangent data: slot 3 is sampled as normal map
/// When SET: normal map sampling is skipped, vertex normal is used instead
/// This is an opt-out flag - normal mapping is enabled by default when tangent data exists
pub const FLAG_SKIP_NORMAL_MAP: u32 = 1 << 16;

impl PackedUnifiedShadingState {
    /// Create from all f32 parameters (used during FFI calls)
    /// For Mode 2: metallic, roughness, emissive packed into uniform_set_0
    /// rim_intensity defaults to 0, can be set via update methods
    pub fn from_floats(
        metallic: f32,
        roughness: f32,
        emissive: f32,
        color: u32,
        matcap_blend_modes: [MatcapBlendMode; 4],
        lights: [PackedLight; 4],
        environment_index: u32,
    ) -> Self {
        // Pack Mode 2 style: [metallic, roughness, emissive, rim_intensity=0]
        let uniform_set_0 = pack_uniform_set_0(
            pack_unorm8(metallic),
            pack_unorm8(roughness),
            pack_unorm8(emissive),
            0, // rim_intensity default
        );
        // uniform_set_1: for Mode 1 use matcap blend modes, Mode 3 use specular RGB
        let uniform_set_1 = pack_matcap_blend_modes(matcap_blend_modes);

        Self {
            uniform_set_0,
            color_rgba8: color,
            flags: DEFAULT_FLAGS, // uniform_alpha = 15 (opaque), other flags = 0
            uniform_set_1,
            lights,
            // Animation System v2 fields - defaults
            keyframe_base: 0,
            inverse_bind_base: 0,
            _pad: 0,
            environment_index,
        }
    }

    /// Set skinning mode flag
    /// - false: raw mode (matrices used as-is)
    /// - true: inverse bind mode (GPU applies inverse bind matrices)
    #[inline]
    pub fn set_skinning_mode(&mut self, inverse_bind: bool) {
        if inverse_bind {
            self.flags |= FLAG_SKINNING_MODE;
        } else {
            self.flags &= !FLAG_SKINNING_MODE;
        }
    }

    /// Get skinning mode flag
    #[inline]
    pub fn skinning_mode(&self) -> bool {
        (self.flags & FLAG_SKINNING_MODE) != 0
    }

    /// Set skip normal map flag (opt-out)
    /// When set to true: normal map sampling is disabled, vertex normal is used
    /// When set to false (default): normal map is sampled from slot 3 (if tangent data exists)
    #[inline]
    pub fn set_skip_normal_map(&mut self, skip: bool) {
        if skip {
            self.flags |= FLAG_SKIP_NORMAL_MAP;
        } else {
            self.flags &= !FLAG_SKIP_NORMAL_MAP;
        }
    }

    /// Check if normal map sampling is skipped
    /// Returns true if normal map is disabled, false if enabled (default)
    #[inline]
    pub fn skips_normal_map(&self) -> bool {
        (self.flags & FLAG_SKIP_NORMAL_MAP) != 0
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use zx_common::encode_octahedral;

    #[test]
    fn test_packed_sizes() {
        assert_eq!(std::mem::size_of::<PackedLight>(), 12); // 12 bytes for point light support
        assert_eq!(std::mem::size_of::<PackedEnvironmentState>(), 48); // 4 (header) + 44 (data)
        assert_eq!(std::mem::size_of::<PackedUnifiedShadingState>(), 80); // 16 (header) + 48 (lights) + 16 (animation/env)
    }

    #[test]
    fn test_quantization() {
        assert_eq!(pack_unorm8(0.0), 0);
        assert_eq!(pack_unorm8(1.0), 255);
        assert_eq!(pack_unorm8(0.5), 128);

        assert_eq!(pack_snorm16(0.0), 0);
        assert_eq!(pack_snorm16(1.0), 32767);
        assert_eq!(pack_snorm16(-1.0), -32767);
    }

    #[test]
    fn test_octahedral_cardinals() {
        // Test that cardinal directions encode/decode correctly
        let tests = [
            Vec3::new(1.0, 0.0, 0.0),  // +X
            Vec3::new(-1.0, 0.0, 0.0), // -X
            Vec3::new(0.0, 1.0, 0.0),  // +Y
            Vec3::new(0.0, -1.0, 0.0), // -Y
            Vec3::new(0.0, 0.0, 1.0),  // +Z
            Vec3::new(0.0, 0.0, -1.0), // -Z
        ];

        for dir in &tests {
            let (u, v) = encode_octahedral(*dir);
            assert!((-1.0..=1.0).contains(&u), "u out of range for {:?}", dir);
            assert!((-1.0..=1.0).contains(&v), "v out of range for {:?}", dir);

            // Verify packing doesn't panic and produces valid output
            let packed = pack_octahedral_u32(*dir);
            assert_ne!(packed, 0xFFFFFFFF, "invalid pack for {:?}", dir);
        }
    }

    #[test]
    fn test_octahedral_zero_vector() {
        let zero = Vec3::new(0.0, 0.0, 0.0);
        let (u, v) = encode_octahedral(zero);
        assert_eq!(u, 0.0);
        assert_eq!(v, 0.0);
    }

    #[test]
    fn test_octahedral_diagonal() {
        // Test diagonal directions (challenging for octahedral)
        let diag = Vec3::new(0.577, 0.577, 0.577).normalize();
        let (u, v) = encode_octahedral(diag);
        assert!((-1.0..=1.0).contains(&u));
        assert!((-1.0..=1.0).contains(&v));
    }

    #[test]
    fn test_pack_rgba8() {
        // Format: 0xRRGGBBAA (R in highest byte, A in lowest)
        let packed = pack_rgba8(1.0, 0.5, 0.25, 1.0);
        assert_eq!((packed >> 24) & 0xFF, 255); // R
        assert_eq!((packed >> 16) & 0xFF, 128); // G
        assert_eq!((packed >> 8) & 0xFF, 64); // B
        assert_eq!(packed & 0xFF, 255); // A
    }

    #[test]
    fn test_disabled_light() {
        let light = PackedLight::disabled();
        assert_eq!(light.data0, 0);
        assert_eq!(light.data1, 0);
        assert_eq!(light.data2, 0);
        assert!(!light.is_enabled());
    }

    #[test]
    fn test_directional_light_roundtrip() {
        let dir = Vec3::new(0.5, -0.7, 0.3).normalize();
        let color = Vec3::new(1.0, 0.5, 0.25);
        let intensity = 2.5;

        let light = PackedLight::directional(dir, color, intensity, true);

        assert_eq!(light.get_type(), LightType::Directional);
        assert!(light.is_enabled());

        let unpacked_dir = light.get_direction();
        assert!((unpacked_dir[0] - dir.x).abs() < 0.01);
        assert!((unpacked_dir[1] - dir.y).abs() < 0.01);
        assert!((unpacked_dir[2] - dir.z).abs() < 0.01);

        let unpacked_color = light.get_color();
        assert!((unpacked_color[0] - color.x).abs() < 0.01);
        assert!((unpacked_color[1] - color.y).abs() < 0.01);
        assert!((unpacked_color[2] - color.z).abs() < 0.01);

        // Intensity with 7-bit precision in 0-8 range
        let unpacked_intensity = light.get_intensity();
        assert!((unpacked_intensity - intensity).abs() < 0.1);
    }

    #[test]
    fn test_point_light_roundtrip() {
        let pos = Vec3::new(10.5, -5.25, 100.0);
        let color = Vec3::new(0.8, 0.6, 0.4);
        let intensity = 4.0;
        let range = 25.0;

        let light = PackedLight::point(pos, color, intensity, range, true);

        assert_eq!(light.get_type(), LightType::Point);
        assert!(light.is_enabled());

        let unpacked_pos = light.get_position();
        // f16 precision is about 3 decimal digits
        assert!((unpacked_pos[0] - pos.x).abs() < 0.1);
        assert!((unpacked_pos[1] - pos.y).abs() < 0.1);
        assert!((unpacked_pos[2] - pos.z).abs() < 1.0);

        let unpacked_range = light.get_range();
        assert!((unpacked_range - range).abs() < 0.5);
    }

    #[test]
    fn test_f16_packing() {
        let values = [0.0, 1.0, -1.0, 100.0, 0.001, 65504.0];
        for v in values {
            let packed = pack_f16(v);
            let unpacked = unpack_f16(packed);
            let error = (unpacked - v).abs() / v.abs().max(1.0);
            assert!(
                error < 0.01,
                "f16 roundtrip failed for {}: got {}",
                v,
                unpacked
            );
        }
    }

    #[test]
    fn test_f16x2_packing() {
        let (x, y) = (42.5, -17.25);
        let packed = pack_f16x2(x, y);
        let (ux, uy) = unpack_f16x2(packed);
        assert!((ux - x).abs() < 0.1);
        assert!((uy - y).abs() < 0.1);
    }

    #[test]
    fn test_intensity_range() {
        // Test intensity at various points in 0-8 range
        for intensity in [0.0, 1.0, 2.0, 4.0, 7.9] {
            let light =
                PackedLight::directional(Vec3::new(0.0, -1.0, 0.0), Vec3::ONE, intensity, true);
            let unpacked = light.get_intensity();
            assert!(
                (unpacked - intensity).abs() < 0.1,
                "intensity {} unpacked to {}",
                intensity,
                unpacked
            );
        }
    }

    #[test]
    fn test_disabled_directional_light() {
        let light = PackedLight::directional(
            Vec3::new(0.0, -1.0, 0.0),
            Vec3::ONE,
            1.0,
            false, // disabled
        );
        assert!(!light.is_enabled());
        assert_eq!(light.get_intensity(), 0.0);
    }

    #[test]
    fn test_texture_filter_flag() {
        let mut state = PackedUnifiedShadingState::default();
        // Default: nearest (flag not set)
        assert_eq!(state.flags & FLAG_TEXTURE_FILTER_LINEAR, 0);

        // Set to linear
        state.flags |= FLAG_TEXTURE_FILTER_LINEAR;
        assert_ne!(state.flags & FLAG_TEXTURE_FILTER_LINEAR, 0);

        // Set back to nearest
        state.flags &= !FLAG_TEXTURE_FILTER_LINEAR;
        assert_eq!(state.flags & FLAG_TEXTURE_FILTER_LINEAR, 0);
    }

    #[test]
    fn test_flags_independence() {
        // Verify texture_filter and skinning_mode flags don't interfere with each other
        let mut state = PackedUnifiedShadingState::default();

        // Set both flags
        state.flags = FLAG_SKINNING_MODE | FLAG_TEXTURE_FILTER_LINEAR;
        assert!(state.skinning_mode());
        assert_ne!(state.flags & FLAG_TEXTURE_FILTER_LINEAR, 0);

        // Clear skinning_mode, texture_filter should remain
        state.flags &= !FLAG_SKINNING_MODE;
        assert!(!state.skinning_mode());
        assert_ne!(state.flags & FLAG_TEXTURE_FILTER_LINEAR, 0);

        // Clear texture_filter, both should be clear
        state.flags &= !FLAG_TEXTURE_FILTER_LINEAR;
        assert!(!state.skinning_mode());
        assert_eq!(state.flags & FLAG_TEXTURE_FILTER_LINEAR, 0);
    }

    #[test]
    fn test_texture_filter_flag_bit_position() {
        // Verify the flag is at bit 1 (value 2)
        assert_eq!(FLAG_TEXTURE_FILTER_LINEAR, 2);
        assert_eq!(FLAG_TEXTURE_FILTER_LINEAR, 1 << 1);

        // Verify it's different from skinning_mode (bit 0)
        assert_ne!(FLAG_TEXTURE_FILTER_LINEAR, FLAG_SKINNING_MODE);
    }

    // ========================================================================
    // Dither Transparency Tests
    // ========================================================================

    #[test]
    fn test_uniform_alpha_packing() {
        // Test all 16 values pack/unpack correctly
        for alpha in 0..=15u32 {
            let flags = alpha << FLAG_UNIFORM_ALPHA_SHIFT;
            let unpacked = (flags & FLAG_UNIFORM_ALPHA_MASK) >> FLAG_UNIFORM_ALPHA_SHIFT;
            assert_eq!(unpacked, alpha);
        }
    }

    #[test]
    fn test_dither_offset_packing() {
        // Test all 16 offset combinations
        for x in 0..=3u32 {
            for y in 0..=3u32 {
                let flags = (x << FLAG_DITHER_OFFSET_X_SHIFT) | (y << FLAG_DITHER_OFFSET_Y_SHIFT);
                let unpacked_x = (flags & FLAG_DITHER_OFFSET_X_MASK) >> FLAG_DITHER_OFFSET_X_SHIFT;
                let unpacked_y = (flags & FLAG_DITHER_OFFSET_Y_MASK) >> FLAG_DITHER_OFFSET_Y_SHIFT;
                assert_eq!(unpacked_x, x);
                assert_eq!(unpacked_y, y);
            }
        }
    }

    #[test]
    fn test_default_flags_are_opaque() {
        let state = PackedUnifiedShadingState::default();
        let alpha = (state.flags & FLAG_UNIFORM_ALPHA_MASK) >> FLAG_UNIFORM_ALPHA_SHIFT;
        assert_eq!(alpha, 15, "Default uniform_alpha must be 15 (opaque)");
    }

    #[test]
    fn test_bayer_threshold_values() {
        // Verify Bayer matrix produces values in expected range
        const BAYER_4X4: [f32; 16] = [
            0.0 / 16.0,
            8.0 / 16.0,
            2.0 / 16.0,
            10.0 / 16.0,
            12.0 / 16.0,
            4.0 / 16.0,
            14.0 / 16.0,
            6.0 / 16.0,
            3.0 / 16.0,
            11.0 / 16.0,
            1.0 / 16.0,
            9.0 / 16.0,
            15.0 / 16.0,
            7.0 / 16.0,
            13.0 / 16.0,
            5.0 / 16.0,
        ];

        for (i, &threshold) in BAYER_4X4.iter().enumerate() {
            assert!(threshold >= 0.0, "Threshold {} is negative", i);
            assert!(threshold < 1.0, "Threshold {} >= 1.0", i);
        }

        // Verify we have 16 unique values
        let mut sorted = BAYER_4X4.to_vec();
        sorted.sort_by(|a, b| a.partial_cmp(b).unwrap());
        for i in 0..15 {
            assert_ne!(sorted[i], sorted[i + 1], "Duplicate threshold values");
        }
    }

    #[test]
    fn test_dither_flags_independence() {
        // Verify dither flags don't interfere with other flags
        let mut state = PackedUnifiedShadingState::default();

        // Set skinning_mode and texture_filter
        state.flags |= FLAG_SKINNING_MODE | FLAG_TEXTURE_FILTER_LINEAR;

        // Set uniform_alpha to 8 (50% transparency)
        state.flags = (state.flags & !FLAG_UNIFORM_ALPHA_MASK) | (8u32 << FLAG_UNIFORM_ALPHA_SHIFT);

        // Set dither offset to (2, 3)
        state.flags = (state.flags & !FLAG_DITHER_OFFSET_X_MASK & !FLAG_DITHER_OFFSET_Y_MASK)
            | (2u32 << FLAG_DITHER_OFFSET_X_SHIFT)
            | (3u32 << FLAG_DITHER_OFFSET_Y_SHIFT);

        // Verify all flags are independent
        assert!(state.skinning_mode());
        assert_ne!(state.flags & FLAG_TEXTURE_FILTER_LINEAR, 0);
        assert_eq!(
            (state.flags & FLAG_UNIFORM_ALPHA_MASK) >> FLAG_UNIFORM_ALPHA_SHIFT,
            8
        );
        assert_eq!(
            (state.flags & FLAG_DITHER_OFFSET_X_MASK) >> FLAG_DITHER_OFFSET_X_SHIFT,
            2
        );
        assert_eq!(
            (state.flags & FLAG_DITHER_OFFSET_Y_MASK) >> FLAG_DITHER_OFFSET_Y_SHIFT,
            3
        );
    }

    // ========================================================================
    // Normal Mapping Tests
    // ========================================================================

    #[test]
    fn test_skip_normal_map_flag() {
        let mut state = PackedUnifiedShadingState::default();
        // Default: normal map NOT skipped (i.e., normal mapping is enabled)
        assert!(!state.skips_normal_map());

        // Opt-out: skip normal map
        state.set_skip_normal_map(true);
        assert!(state.skips_normal_map());

        // Re-enable normal map (clear skip flag)
        state.set_skip_normal_map(false);
        assert!(!state.skips_normal_map());
    }

    #[test]
    fn test_skip_normal_map_flag_bit_position() {
        // Verify the flag is at bit 16 (value 0x10000)
        assert_eq!(FLAG_SKIP_NORMAL_MAP, 0x10000);
        assert_eq!(FLAG_SKIP_NORMAL_MAP, 1 << 16);

        // Verify it doesn't overlap with other flags
        assert_ne!(
            FLAG_SKIP_NORMAL_MAP & FLAG_SKINNING_MODE,
            FLAG_SKINNING_MODE
        );
        assert_ne!(
            FLAG_SKIP_NORMAL_MAP & FLAG_TEXTURE_FILTER_LINEAR,
            FLAG_TEXTURE_FILTER_LINEAR
        );
        assert_ne!(
            FLAG_SKIP_NORMAL_MAP & FLAG_UNIFORM_ALPHA_MASK,
            FLAG_UNIFORM_ALPHA_MASK
        );
    }

    #[test]
    fn test_skip_normal_map_flag_independence() {
        // Verify skip normal map flag doesn't interfere with other flags
        let mut state = PackedUnifiedShadingState::default();

        // Set multiple flags
        state.flags |= FLAG_SKINNING_MODE | FLAG_TEXTURE_FILTER_LINEAR;
        state.set_skip_normal_map(true);

        // Verify all flags are set correctly
        assert!(state.skinning_mode());
        assert_ne!(state.flags & FLAG_TEXTURE_FILTER_LINEAR, 0);
        assert!(state.skips_normal_map());

        // Clear skip flag, others should remain
        state.set_skip_normal_map(false);
        assert!(state.skinning_mode());
        assert_ne!(state.flags & FLAG_TEXTURE_FILTER_LINEAR, 0);
        assert!(!state.skips_normal_map());
    }

    // ========================================================================
    // Environment System Tests
    // ========================================================================

    #[test]
    fn test_environment_header_packing() {
        // Test all combinations of modes and blend modes
        for base in 0..8u32 {
            for overlay in 0..8u32 {
                for blend in 0..4u32 {
                    let header = PackedEnvironmentState::make_header(base, overlay, blend);
                    let mut env = PackedEnvironmentState::default();
                    env.header = header;
                    assert_eq!(env.base_mode(), base);
                    assert_eq!(env.overlay_mode(), overlay);
                    assert_eq!(env.blend_mode(), blend);
                }
            }
        }
    }

    #[test]
    fn test_environment_mode_setters() {
        let mut env = PackedEnvironmentState::default();

        env.set_base_mode(env_mode::GRADIENT);
        env.set_overlay_mode(env_mode::SCATTER);
        env.set_blend_mode(blend_mode::ADD);

        assert_eq!(env.base_mode(), env_mode::GRADIENT);
        assert_eq!(env.overlay_mode(), env_mode::SCATTER);
        assert_eq!(env.blend_mode(), blend_mode::ADD);

        // Change individual values without affecting others
        env.set_base_mode(env_mode::RINGS);
        assert_eq!(env.base_mode(), env_mode::RINGS);
        assert_eq!(env.overlay_mode(), env_mode::SCATTER); // unchanged
        assert_eq!(env.blend_mode(), blend_mode::ADD); // unchanged
    }

    #[test]
    fn test_environment_gradient_packing() {
        let mut env = PackedEnvironmentState::default();
        env.pack_gradient(GradientConfig {
            offset: 0,
            zenith: 0x3366B2FF,
            sky_horizon: 0xB2D8F2FF,
            ground_horizon: 0x8B7355FF,
            nadir: 0x4A3728FF,
            rotation: 45.0,
            shift: 0.25,
        });

        assert_eq!(env.data[0], 0x3366B2FF);
        assert_eq!(env.data[1], 0xB2D8F2FF);
        assert_eq!(env.data[2], 0x8B7355FF);
        assert_eq!(env.data[3], 0x4A3728FF);

        // Verify f16x2 packing of rotation and shift
        let (rotation, shift) = unpack_f16x2(env.data[4]);
        assert!((rotation - 45.0).abs() < 0.1);
        assert!((shift - 0.25).abs() < 0.01);
    }

    #[test]
    fn test_environment_default_gradient() {
        let env = PackedEnvironmentState::default_gradient();
        assert_eq!(env.base_mode(), env_mode::GRADIENT);
        assert_eq!(env.overlay_mode(), env_mode::GRADIENT);
        assert_eq!(env.blend_mode(), blend_mode::ALPHA);
        // Verify colors are set
        assert_ne!(env.data[0], 0); // zenith
        assert_ne!(env.data[1], 0); // sky_horizon
    }

    #[test]
    fn test_environment_index() {
        assert_eq!(EnvironmentIndex::default(), EnvironmentIndex(0));
        assert_eq!(EnvironmentIndex::INVALID, EnvironmentIndex(u32::MAX));
    }
}
