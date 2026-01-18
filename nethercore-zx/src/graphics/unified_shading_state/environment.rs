use bytemuck::{Pod, Zeroable};
use glam::Vec3;

use super::quantization::{pack_f16, pack_f16x2};
use zx_common::pack_octahedral_u16;

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
    pub const CELLS: u32 = 1;
    pub const LINES: u32 = 2;
    pub const SILHOUETTE: u32 = 3;
    pub const NEBULA: u32 = 4;
    pub const ROOM: u32 = 5;
    pub const VEIL: u32 = 6;
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
    pub sun_elevation: f32,
    pub sun_disk: u32,
    pub sun_halo: u32,
    pub sun_intensity: u32,
    pub horizon_haze: u32,
    pub sun_warmth: u32,
    pub cloudiness: u32,
    pub cloud_phase: u32,
}

/// Configuration for packing cells parameters
pub struct CellsConfig {
    pub offset: usize,
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
    pub axis: Vec3,
    pub phase: u32,
    pub seed: u32,
}

/// Configuration for packing lines parameters
pub struct LinesConfig {
    pub offset: usize,
    pub variant: u32,
    pub line_type: u32,
    pub thickness: u32,
    pub spacing: f32,
    pub fade_distance: f32,
    pub parallax: u32,
    pub color_primary: u32,
    pub color_accent: u32,
    pub accent_every: u32,
    pub phase: u32,
    pub profile: u32,
    pub warp: u32,
    pub wobble: u32,
    pub glow: u32,
    pub axis: Vec3,
    pub seed: u32,
}

/// Configuration for packing silhouette parameters
pub struct SilhouetteConfig {
    pub offset: usize,
    pub family: u32,
    pub jaggedness: u32,
    pub layer_count: u32,
    pub color_near: u32,
    pub color_far: u32,
    pub sky_zenith: u32,
    pub sky_horizon: u32,
    pub parallax_rate: u32,
    pub seed: u32,
    pub phase: u32,
    pub fog: u32,
    pub wind: u32,
}

/// Configuration for packing nebula parameters
pub struct NebulaConfig {
    pub offset: usize,
    pub family: u32,
    pub coverage: u32,
    pub softness: u32,
    pub intensity: u32,
    pub scale: u32,
    pub detail: u32,
    pub warp: u32,
    pub flow: u32,
    pub parallax: u32,
    pub height_bias: u32,
    pub contrast: u32,
    pub color_a: u32,
    pub color_b: u32,
    pub axis: Vec3,
    pub phase: u32,
    pub seed: u32,
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
    pub light_tint: u32,
    pub corner_darken: u32,
    pub room_scale: f32,
    pub viewer_x: i32,
    pub viewer_y: i32,
    pub viewer_z: i32,
    pub accent: u32,
    pub accent_mode: u32,
    pub roughness: u32,
    pub phase: u32,
}

/// Configuration for packing veil parameters
pub struct VeilConfig {
    pub offset: usize,
    pub family: u32,
    pub density: u32,
    pub height_min: u32,
    pub height_max: u32,
    pub width: u32,
    pub taper: u32,
    pub curvature: u32,
    pub edge_soft: u32,
    pub color_near: u32,
    pub color_far: u32,
    pub glow: u32,
    pub parallax: u32,
    pub axis: Vec3,
    pub phase: u32,
    pub seed: u32,
}

/// Configuration for packing rings parameters
pub struct RingsConfig {
    pub offset: usize,
    pub family: u32,
    pub ring_count: u32,
    pub thickness: u32,
    pub color_a: u32,
    pub color_b: u32,
    pub center_color: u32,
    pub center_falloff: u32,
    pub spiral_twist: f32,
    pub axis: Vec3,
    pub phase: u32,
    pub wobble: u32,
    pub noise: u32,
    pub dash: u32,
    pub glow: u32,
    pub seed: u32,
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
    /// Mode 0: Gradient - 4-color sky/ground gradient + featured sky controls
    pub fn pack_gradient(&mut self, config: GradientConfig) {
        let GradientConfig {
            offset,
            zenith,
            sky_horizon,
            ground_horizon,
            nadir,
            rotation,
            shift,
            sun_elevation,
            sun_disk,
            sun_halo,
            sun_intensity,
            horizon_haze,
            sun_warmth,
            cloudiness,
            cloud_phase,
        } = config;
        self.data[offset] = zenith;
        self.data[offset + 1] = sky_horizon;
        self.data[offset + 2] = ground_horizon;
        self.data[offset + 3] = nadir;

        // w4: cloud_phase:u16 (low16) | shift:f16 (high16)
        self.data[offset + 4] = (cloud_phase & 0xFFFF) | ((pack_f16(shift) as u32) << 16);

        // w5: sun_dir_oct16 (low16) | sun_disk:u8 | sun_halo:u8
        // Sun direction is defined in absolute world-space (+Y up; 0 azimuth = +Z, π/2 = +X).
        let (sin_az, cos_az) = rotation.sin_cos();
        let (sin_el, cos_el) = sun_elevation.sin_cos();
        let sun_dir = Vec3::new(cos_el * sin_az, sin_el, cos_el * cos_az).normalize_or_zero();
        let sun_oct16 = pack_octahedral_u16(sun_dir) as u32;
        self.data[offset + 5] =
            (sun_oct16 & 0xFFFF) | ((sun_disk & 0xFF) << 16) | ((sun_halo & 0xFF) << 24);

        // w6: sun_intensity:u8 | horizon_haze:u8 | sun_warmth:u8 | cloudiness:u8
        self.data[offset + 6] = (sun_intensity & 0xFF)
            | ((horizon_haze & 0xFF) << 8)
            | ((sun_warmth & 0xFF) << 16)
            | ((cloudiness & 0xFF) << 24);
    }

    /// Create a default gradient environment (blue sky)
    pub fn default_gradient() -> Self {
        let mut env = Self {
            // Default overlay is a transparent Cells layer (all-zero payload), so a base-only
            // environment does not get overridden by an unconfigured overlay.
            header: Self::make_header(env_mode::GRADIENT, env_mode::CELLS, blend_mode::ALPHA),
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
            sun_elevation: 0.0,
            sun_disk: 0,
            sun_halo: 0,
            sun_intensity: 0,
            horizon_haze: 0,
            sun_warmth: 0,
            cloudiness: 0,
            cloud_phase: 0,
        });
        env
    }

    /// Pack cells parameters into data[offset..offset+7]
    /// Mode 1: Cells - Particles (family 0) + Tiles/Lights (family 1)
    ///
    /// Packing matches the design sheets under `nethercore-design/specs/epu-modes/`.
    pub fn pack_cells(&mut self, config: CellsConfig) {
        let CellsConfig {
            offset,
            family,
            variant,
            density,
            size_min,
            size_max,
            intensity,
            shape,
            motion,
            parallax,
            height_bias,
            clustering,
            color_a,
            color_b,
            axis,
            phase,
            seed,
        } = config;

        // w0: family:u8 | variant:u8 | density:u8 | intensity:u8
        self.data[offset] = (family & 0xFF)
            | ((variant & 0xFF) << 8)
            | ((density & 0xFF) << 16)
            | ((intensity & 0xFF) << 24);

        // w1: size_min:u8 | size_max:u8 | shape:u8 | motion:u8
        self.data[offset + 1] = (size_min & 0xFF)
            | ((size_max & 0xFF) << 8)
            | ((shape & 0xFF) << 16)
            | ((motion & 0xFF) << 24);

        // w2/w3: palette endpoints
        self.data[offset + 2] = color_a;
        self.data[offset + 3] = color_b;

        // w4: parallax:u8 | reserved:u8 | axis_oct16:u16
        let axis_default = if family == 0 && variant == 1 {
            Vec3::new(0.0, -1.0, 0.0)
        } else {
            Vec3::Y
        };
        let axis_safe = if axis.length_squared() < 1e-6 {
            axis_default
        } else {
            axis.normalize()
        };
        let axis_oct16 = pack_octahedral_u16(axis_safe) as u32;
        self.data[offset + 4] = (parallax & 0xFF) | ((axis_oct16 & 0xFFFF) << 16);

        // w5: phase:u16 (low) | height_bias:u8 | clustering:u8
        self.data[offset + 5] =
            (phase & 0xFFFF) | ((height_bias & 0xFF) << 16) | ((clustering & 0xFF) << 24);

        // w6: seed:u32 (0 means derive from params)
        self.data[offset + 6] = seed;
    }

    /// Pack lines parameters into data[offset..offset+7]
    /// Mode 2: Lines - Infinite grid lines projected onto a plane
    ///
    /// # Data Layout
    /// - data[0]: variant(2) + line_type(2) + thickness(8) + accent_every(8) + parallax(8) + reserved(4)
    /// - data[1]: spacing(f16) + fade_distance(f16)
    /// - data[2]: color_primary (RGBA8)
    /// - data[3]: color_accent (RGBA8)
    /// - data[4]: phase(u16) + axis_oct16(u16)
    pub fn pack_lines(&mut self, config: LinesConfig) {
        let LinesConfig {
            offset,
            variant,
            line_type,
            thickness,
            spacing,
            fade_distance,
            parallax,
            color_primary,
            color_accent,
            accent_every,
            phase,
            profile,
            warp,
            wobble,
            glow,
            axis,
            seed,
        } = config;
        // data[0]: variant(2) + line_type(2) + thickness(8) + accent_every(8) + parallax(8) + reserved(4)
        self.data[offset] = (variant & 0x3)
            | ((line_type & 0x3) << 2)
            | ((thickness & 0xFF) << 4)
            | (((accent_every.max(1)) & 0xFF) << 12)
            | ((parallax & 0xFF) << 20);

        // data[1]: spacing(f16) + fade_distance(f16)
        self.data[offset + 1] = pack_f16x2(spacing, fade_distance);

        // data[2]: color_primary (RGBA8)
        self.data[offset + 2] = color_primary;

        // data[3]: color_accent (RGBA8)
        self.data[offset + 3] = color_accent;

        // w4: phase:u16 (low16) | axis_oct16:u16 (high16)
        let axis_safe = if axis.length_squared() < 1e-6 {
            Vec3::Z
        } else {
            axis.normalize()
        };
        let axis_oct16 = pack_octahedral_u16(axis_safe) as u32;
        self.data[offset + 4] = (phase & 0xFFFF) | ((axis_oct16 & 0xFFFF) << 16);

        // w5: warp:u8 | glow:u8 | wobble:u8 | profile:u8
        self.data[offset + 5] = (warp & 0xFF)
            | ((glow & 0xFF) << 8)
            | ((wobble & 0xFF) << 16)
            | ((profile & 0xFF) << 24);

        // w6: seed:u32 (0 means derive from params)
        self.data[offset + 6] = seed;
    }

    /// Pack silhouette parameters into data[offset..offset+7]
    /// Mode 3: Silhouette - Layered terrain silhouettes with parallax
    ///
    /// # Data Layout
    /// - data[0]: family:u8 | jaggedness:u8 | layer_count:u8 | parallax_rate:u8
    /// - data[1]: color_near (RGBA8)
    /// - data[2]: color_far (RGBA8)
    /// - data[3]: sky_zenith (RGBA8)
    /// - data[4]: sky_horizon (RGBA8)
    /// - data[5]: seed (u32)
    /// - data[6]: phase:u16 | fog:u8 | wind:u8
    pub fn pack_silhouette(&mut self, config: SilhouetteConfig) {
        let SilhouetteConfig {
            offset,
            family,
            jaggedness,
            layer_count,
            color_near,
            color_far,
            sky_zenith,
            sky_horizon,
            parallax_rate,
            seed,
            phase,
            fog,
            wind,
        } = config;
        // w0: family:u8 | jaggedness:u8 | layer_count:u8 | parallax_rate:u8
        self.data[offset] = (family & 0xFF)
            | ((jaggedness & 0xFF) << 8)
            | (((layer_count.clamp(1, 3)) & 0xFF) << 16)
            | ((parallax_rate & 0xFF) << 24);

        // data[1]: color_near (RGBA8)
        self.data[offset + 1] = color_near;

        // data[2]: color_far (RGBA8)
        self.data[offset + 2] = color_far;

        // data[3]: sky_zenith (RGBA8)
        self.data[offset + 3] = sky_zenith;

        // data[4]: sky_horizon (RGBA8)
        self.data[offset + 4] = sky_horizon;

        // w5: seed:u32 (0 means derive from params)
        self.data[offset + 5] = seed;

        // w6: phase:u16 (low) | fog:u8 | wind:u8
        self.data[offset + 6] = (phase & 0xFFFF) | ((fog & 0xFF) << 16) | ((wind & 0xFF) << 24);
    }

    /// Pack nebula parameters into data[offset..offset+7]
    /// Mode 4: Nebula - Soft fields (fog/clouds/aurora/ink/plasma/kaleido)
    pub fn pack_nebula(&mut self, config: NebulaConfig) {
        let NebulaConfig {
            offset,
            family,
            coverage,
            softness,
            intensity,
            scale,
            detail,
            warp,
            flow,
            parallax,
            height_bias,
            contrast,
            color_a,
            color_b,
            axis,
            phase,
            seed,
        } = config;

        // w0: family:u8 | coverage:u8 | softness:u8 | intensity:u8
        self.data[offset] = (family & 0xFF)
            | ((coverage & 0xFF) << 8)
            | ((softness & 0xFF) << 16)
            | ((intensity & 0xFF) << 24);

        // w1: scale:u8 | detail:u8 | warp:u8 | flow:u8
        self.data[offset + 1] =
            (scale & 0xFF) | ((detail & 0xFF) << 8) | ((warp & 0xFF) << 16) | ((flow & 0xFF) << 24);

        self.data[offset + 2] = color_a;
        self.data[offset + 3] = color_b;

        // w4: height_bias:u8 | contrast:u8 | parallax:u8 | reserved:u8
        self.data[offset + 4] =
            (height_bias & 0xFF) | ((contrast & 0xFF) << 8) | ((parallax & 0xFF) << 16);

        // w5: axis_oct16 (low16) | phase:u16 (high16)
        let axis_safe = if axis.length_squared() < 1e-6 {
            Vec3::Y
        } else {
            axis.normalize()
        };
        let axis_oct16 = pack_octahedral_u16(axis_safe) as u32;
        self.data[offset + 5] = (axis_oct16 & 0xFFFF) | ((phase & 0xFFFF) << 16);

        // w6: seed:u32 (0 means derive from params)
        self.data[offset + 6] = seed;
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
    /// - data[5]: accent_mode:u8 | accent:u8 | phase:u16
    /// - data[6]: light_tint_RGB(24) | roughness:u8
    ///
    /// Uses per-layer extension words `w5/w6` for loopable accents + light tint + roughness.
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
            light_tint,
            corner_darken,
            room_scale,
            viewer_x,
            viewer_y,
            viewer_z,
            accent,
            accent_mode,
            roughness,
            phase,
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

        // data[4]: light_dir_oct16 (low16) + light_intensity:u8 + room_scale_u8
        let light_dir_safe = if light_direction.length_squared() < 1e-6 {
            Vec3::new(0.0, -1.0, 0.0)
        } else {
            light_direction.normalize()
        };
        let light_oct16 = pack_octahedral_u16(light_dir_safe) as u32;
        let room_scale_packed = ((room_scale.clamp(0.1, 25.5) * 10.0) as u32) & 0xFF;
        self.data[offset + 4] =
            (light_oct16 & 0xFFFF) | ((light_intensity & 0xFF) << 16) | (room_scale_packed << 24);

        // w5: accent_mode:u8 | accent:u8 | phase:u16 (high16)
        self.data[offset + 5] =
            (accent_mode & 0xFF) | ((accent & 0xFF) << 8) | ((phase & 0xFFFF) << 16);

        // w6: light_tint_RGB(24) | roughness:u8
        self.data[offset + 6] = (light_tint & 0xFFFFFF00) | (roughness & 0xFF);
    }

    /// Pack veil parameters into data[offset..offset+7]
    /// Mode 6: Veil - Direction-based SDF ribbons/pillars (bounded 1–3 depth slices)
    pub fn pack_veil(&mut self, config: VeilConfig) {
        let VeilConfig {
            offset,
            family,
            density,
            mut height_min,
            mut height_max,
            width,
            taper,
            curvature,
            edge_soft,
            color_near,
            color_far,
            glow,
            parallax,
            axis,
            phase,
            seed,
        } = config;

        if height_min > height_max {
            core::mem::swap(&mut height_min, &mut height_max);
        }

        // w0: family:u8 | density:u8 | width:u8 | taper:u8
        self.data[offset] = (family & 0xFF)
            | ((density & 0xFF) << 8)
            | ((width & 0xFF) << 16)
            | ((taper & 0xFF) << 24);

        // w1: curvature:u8 | edge_soft:u8 | height_min:u8 | height_max:u8
        self.data[offset + 1] = (curvature & 0xFF)
            | ((edge_soft & 0xFF) << 8)
            | ((height_min & 0xFF) << 16)
            | ((height_max & 0xFF) << 24);

        self.data[offset + 2] = color_near;
        self.data[offset + 3] = color_far;

        // w4: glow:u8 | parallax:u8 | reserved:u16 (must be zero)
        self.data[offset + 4] = (glow & 0xFF) | ((parallax & 0xFF) << 8);

        // w5: axis_oct16 (low16) | phase:u16 (high16)
        let axis_safe = if axis.length_squared() < 1e-6 {
            Vec3::Y
        } else {
            axis.normalize()
        };
        let axis_oct16 = pack_octahedral_u16(axis_safe) as u32;
        self.data[offset + 5] = (axis_oct16 & 0xFFFF) | ((phase & 0xFFFF) << 16);

        // w6: seed:u32 (0 means derive from params)
        self.data[offset + 6] = seed;
    }

    /// Pack rings parameters into data[offset..offset+7]
    /// Mode 7: Rings - Concentric rings around focal direction (tunnel/portal/vortex)
    ///
    /// # Data Layout
    ///
    /// - data[0]: ring_count(8) + thickness(8) + center_falloff(8) + family(8)
    /// - data[1]: color_a (RGBA8)
    /// - data[2]: color_b (RGBA8)
    /// - data[3]: center_color (RGBA8)
    /// - data[4]: spiral_twist(f16) + axis_oct(16)
    /// - data[5]: phase:u16 + wobble:u16
    /// - data[6]: noise:u8 + dash:u8 + glow:u8 + seed:u8
    pub fn pack_rings(&mut self, config: RingsConfig) {
        let RingsConfig {
            offset,
            family,
            ring_count,
            thickness,
            color_a,
            color_b,
            center_color,
            center_falloff,
            spiral_twist,
            axis,
            phase,
            wobble,
            noise,
            dash,
            glow,
            seed,
        } = config;
        // w0: ring_count:u8 | thickness:u8 | center_falloff:u8 | family:u8
        self.data[offset] = (ring_count & 0xFF)
            | ((thickness & 0xFF) << 8)
            | ((center_falloff & 0xFF) << 16)
            | ((family & 0xFF) << 24);

        // data[1]: color_a (RGBA8)
        self.data[offset + 1] = color_a;

        // data[2]: color_b (RGBA8)
        self.data[offset + 2] = color_b;

        // data[3]: center_color (RGBA8)
        self.data[offset + 3] = center_color;

        // w4: spiral_twist:f16 (low16) + axis_oct16 (high16)
        // Using 16-bit octahedral (2x snorm8) for axis - ~1.4° precision, fits in 16 bits
        let axis_safe = if axis.length_squared() < 1e-6 {
            Vec3::Z
        } else {
            axis.normalize()
        };
        let axis_oct = pack_octahedral_u16(axis_safe) as u32;
        let twist_bits = pack_f16(spiral_twist) as u32;
        self.data[offset + 4] = twist_bits | (axis_oct << 16);

        // w5: phase:u16 (low16) | wobble:u16 (high16)
        self.data[offset + 5] = (phase & 0xFFFF) | ((wobble & 0xFFFF) << 16);

        // w6: noise:u8 | dash:u8 | glow:u8 | seed:u8
        self.data[offset + 6] =
            (noise & 0xFF) | ((dash & 0xFF) << 8) | ((glow & 0xFF) << 16) | ((seed & 0xFF) << 24);
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
