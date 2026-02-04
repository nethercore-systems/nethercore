//! EPU parameter structs and shape/pattern enums.
//!
//! This module contains all the parameter structures used to configure
//! EPU layers through the builder API, as well as the shape and pattern
//! enums for various effects.

use glam::Vec3;

use super::{EpuBlend, EpuRegion};

// =============================================================================
// Shape and Pattern Enums
// =============================================================================

/// Decal shape types
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum DecalShape {
    /// Circular disk
    #[default]
    Disk = 0,
    /// Ring/annulus
    Ring = 1,
    /// Rectangle
    Rect = 2,
    /// Vertical line
    Line = 3,
}

/// Grid pattern types
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum GridPattern {
    /// Vertical stripes
    #[default]
    Stripes = 0,
    /// Crosshatch grid
    Grid = 1,
    /// Checkerboard
    Checker = 2,
}

/// Flow pattern types
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum FlowPattern {
    /// Perlin-like noise
    #[default]
    Noise = 0,
    /// Directional streaks
    Streaks = 1,
    /// Underwater caustic
    Caustic = 2,
}

/// Waveforms for phase-driven modulation.
#[repr(u8)]
#[derive(Clone, Copy, Debug, PartialEq, Eq, Default)]
pub enum PhaseWaveform {
    /// No modulation (constant).
    #[default]
    Off = 0,
    /// Smooth sine modulation.
    Sine = 1,
    /// Linear up/down (triangle) modulation.
    Triangle = 2,
    /// Hard on/off modulation.
    Strobe = 3,
}

// =============================================================================
// Bounds Parameter Structs
// =============================================================================

/// Parameters for RAMP bounds.
#[derive(Clone, Copy, Debug)]
pub struct RampParams {
    /// Up vector defining the bounds orientation
    pub up: Vec3,
    /// RGB color for wall/horizon
    pub wall_color: [u8; 3],
    /// RGB color for sky/ceiling
    pub sky_color: [u8; 3],
    /// RGB color for floor/ground
    pub floor_color: [u8; 3],
    /// Ceiling threshold (0..15 maps to -1..1)
    pub ceil_q: u8,
    /// Floor threshold (0..15 maps to -1..1)
    pub floor_q: u8,
    /// Transition softness (0..255)
    pub softness: u8,
}

impl Default for RampParams {
    fn default() -> Self {
        Self {
            up: Vec3::Y,
            wall_color: [0, 0, 0],
            sky_color: [0, 0, 0],
            floor_color: [0, 0, 0],
            ceil_q: 8,
            floor_q: 8,
            softness: 128,
        }
    }
}

/// Parameters for SECTOR bounds.
#[derive(Clone, Copy, Debug)]
pub struct SectorParams {
    pub up: Vec3,
    pub sky_color: [u8; 3],
    pub wall_color: [u8; 3],
    pub strength: u8,
    pub center_u01: u8,
    pub width: u8,
    pub variant_id: u8,
}

impl Default for SectorParams {
    fn default() -> Self {
        Self {
            up: Vec3::Y,
            sky_color: [0, 0, 0],
            wall_color: [0, 0, 0],
            strength: 0,
            center_u01: 128,
            width: 64,
            variant_id: 0,
        }
    }
}

/// Parameters for SILHOUETTE bounds.
#[derive(Clone, Copy, Debug)]
pub struct SilhouetteParams {
    pub up: Vec3,
    pub silhouette_color: [u8; 3],
    pub background_color: [u8; 3],
    pub edge_softness: u8,
    pub horizon_bias: u8,
    pub roughness: u8,
    pub octaves_q: u8,
    pub drift_amount_q: u8,
    pub drift_speed: u8,
    pub strength: u8,
    pub variant_id: u8,
}

impl Default for SilhouetteParams {
    fn default() -> Self {
        Self {
            up: Vec3::Y,
            silhouette_color: [0, 0, 0],
            background_color: [0, 0, 0],
            edge_softness: 32,
            horizon_bias: 128,
            roughness: 128,
            octaves_q: 4,
            drift_amount_q: 0,
            drift_speed: 0,
            strength: 15,
            variant_id: 0,
        }
    }
}

/// Parameters for SPLIT bounds.
#[derive(Clone, Copy, Debug)]
pub struct SplitParams {
    pub axis: Vec3,
    pub sky_color: [u8; 3],
    pub wall_color: [u8; 3],
    pub blend_width: u8,
    pub wedge_angle: u8,
    pub count: u8,
    pub offset: u8,
    pub variant_id: u8,
}

impl Default for SplitParams {
    fn default() -> Self {
        Self {
            axis: Vec3::Y,
            sky_color: [0, 0, 0],
            wall_color: [0, 0, 0],
            blend_width: 16,
            wedge_angle: 128,
            count: 8,
            offset: 0,
            variant_id: 0,
        }
    }
}

/// Parameters for CELL bounds.
#[derive(Clone, Copy, Debug)]
pub struct CellParams {
    pub axis: Vec3,
    pub gap_color: [u8; 3],
    pub wall_color: [u8; 3],
    pub outline_brightness: u8,
    pub density: u8,
    pub fill_ratio: u8,
    pub gap_width: u8,
    pub seed: u8,
    pub gap_alpha: u8,
    pub outline_alpha: u8,
    pub variant_id: u8,
}

impl Default for CellParams {
    fn default() -> Self {
        Self {
            axis: Vec3::Y,
            gap_color: [0, 0, 0],
            wall_color: [0, 0, 0],
            outline_brightness: 0,
            density: 64,
            fill_ratio: 255,
            gap_width: 0,
            seed: 0,
            gap_alpha: 15,
            outline_alpha: 15,
            variant_id: 0,
        }
    }
}

/// Parameters for PATCHES bounds.
#[derive(Clone, Copy, Debug)]
pub struct PatchesParams {
    pub axis: Vec3,
    pub sky_color: [u8; 3],
    pub wall_color: [u8; 3],
    pub scale: u8,
    pub coverage: u8,
    pub sharpness: u8,
    pub seed: u8,
    pub sky_alpha: u8,
    pub wall_alpha: u8,
    pub domain_id: u8,
    pub variant_id: u8,
}

impl Default for PatchesParams {
    fn default() -> Self {
        Self {
            axis: Vec3::Y,
            sky_color: [0, 0, 0],
            wall_color: [0, 0, 0],
            scale: 32,
            coverage: 128,
            sharpness: 64,
            seed: 0,
            sky_alpha: 15,
            wall_alpha: 15,
            domain_id: 0,
            variant_id: 0,
        }
    }
}

/// Parameters for APERTURE bounds.
#[derive(Clone, Copy, Debug)]
pub struct ApertureParams {
    pub dir: Vec3,
    pub opening_color: [u8; 3],
    pub frame_color: [u8; 3],
    pub edge_softness: u8,
    pub half_width: u8,
    pub half_height: u8,
    pub frame_thickness: u8,
    pub variant_param: u8,
    pub variant_id: u8,
}

impl Default for ApertureParams {
    fn default() -> Self {
        Self {
            dir: Vec3::Z,
            opening_color: [0, 0, 0],
            frame_color: [0, 0, 0],
            edge_softness: 16,
            half_width: 128,
            half_height: 128,
            frame_thickness: 64,
            variant_param: 0,
            variant_id: 0,
        }
    }
}

// =============================================================================
// Feature Parameter Structs
// =============================================================================

/// Parameters for DECAL feature.
#[derive(Clone, Copy, Debug)]
pub struct DecalParams {
    /// Region mask
    pub region: EpuRegion,
    /// Blend mode
    pub blend: EpuBlend,
    /// Shape type
    pub shape: DecalShape,
    /// Shape center direction
    pub dir: Vec3,
    /// RGB color for shape (primary)
    pub color: [u8; 3],
    /// RGB color for outline/glow (secondary)
    pub color_b: [u8; 3],
    /// Brightness (0..255)
    pub intensity: u8,
    /// Edge softness (0..15)
    pub softness_q: u8,
    /// Size (0..255 maps to 0..0.5 rad)
    pub size: u8,
    /// Glow softness (0..255 maps to 0..0.2)
    pub glow_softness: u8,
    /// Looping animation phase (0..255 maps to 0..1).
    ///
    /// Advance this from your game (deterministic) to animate the decal.
    pub phase: u8,
    /// Alpha (0-15)
    pub alpha: u8,
}

impl Default for DecalParams {
    fn default() -> Self {
        Self {
            region: EpuRegion::All,
            blend: EpuBlend::Add,
            shape: DecalShape::Disk,
            dir: Vec3::Y,
            color: [255, 255, 255],
            color_b: [0, 0, 0],
            intensity: 255,
            softness_q: 2,
            size: 20,
            glow_softness: 64,
            phase: 0,
            alpha: 15,
        }
    }
}

/// Parameters for SCATTER feature.
#[derive(Clone, Copy, Debug)]
pub struct ScatterParams {
    /// Region mask
    pub region: EpuRegion,
    /// Blend mode
    pub blend: EpuBlend,
    /// RGB color for points
    pub color: [u8; 3],
    /// Brightness (0..255)
    pub intensity: u8,
    /// Point density (0..255 maps to 1..256)
    pub density: u8,
    /// Point size (0..255 maps to 0.001..0.05 rad)
    pub size: u8,
    /// Twinkle amount (0..15)
    pub twinkle_q: u8,
    /// Random seed (0..255)
    pub seed: u8,
}

impl Default for ScatterParams {
    fn default() -> Self {
        Self {
            region: EpuRegion::All,
            blend: EpuBlend::Add,
            color: [255, 255, 255],
            intensity: 255,
            density: 200,
            size: 20,
            twinkle_q: 8,
            seed: 0,
        }
    }
}

/// Parameters for GRID feature.
#[derive(Clone, Copy, Debug)]
pub struct GridParams {
    /// Region mask
    pub region: EpuRegion,
    /// Blend mode
    pub blend: EpuBlend,
    /// RGB color for lines
    pub color: [u8; 3],
    /// Brightness (0..255)
    pub intensity: u8,
    /// Grid scale (0..255 maps to 1..64)
    pub scale: u8,
    /// Line thickness (0..255 maps to 0.001..0.1)
    pub thickness: u8,
    /// Pattern type
    pub pattern: GridPattern,
    /// Scroll speed (0..15 maps to 0..2)
    pub scroll_q: u8,
    /// Looping animation phase (0..255 maps to 0..1).
    ///
    /// Advance this from your game (deterministic) to animate scrolling.
    pub phase: u8,
}

impl Default for GridParams {
    fn default() -> Self {
        Self {
            region: EpuRegion::Walls,
            blend: EpuBlend::Add,
            color: [64, 64, 64],
            intensity: 128,
            scale: 32,
            thickness: 20,
            pattern: GridPattern::Grid,
            scroll_q: 0,
            phase: 0,
        }
    }
}

/// Parameters for FLOW feature.
#[derive(Clone, Copy, Debug)]
pub struct FlowParams {
    /// Region mask
    pub region: EpuRegion,
    /// Blend mode
    pub blend: EpuBlend,
    /// Flow direction
    pub dir: Vec3,
    /// RGB color for flow
    pub color: [u8; 3],
    /// Brightness (0..255)
    pub intensity: u8,
    /// Noise scale (0..255 maps to 1..16)
    pub scale: u8,
    /// Looping animation phase (0..255 maps to 0..1).
    ///
    /// Advance this from your game (deterministic) to animate the pattern.
    pub phase: u8,
    /// Noise octaves (0..4)
    pub octaves: u8,
    /// Pattern type
    pub pattern: FlowPattern,
    /// Turbulence amount (0..255)
    pub turbulence: u8,
}

impl Default for FlowParams {
    fn default() -> Self {
        Self {
            region: EpuRegion::Sky,
            blend: EpuBlend::Lerp,
            dir: Vec3::X,
            color: [128, 128, 128],
            intensity: 60,
            scale: 32,
            phase: 0,
            octaves: 2,
            pattern: FlowPattern::Noise,
            turbulence: 0,
        }
    }
}

/// Parameters for LOBE_RADIANCE feature.
#[derive(Clone, Copy, Debug)]
pub struct LobeRadianceParams {
    pub region: EpuRegion,
    pub blend: EpuBlend,
    pub dir: Vec3,
    pub color: [u8; 3],
    pub edge_color: [u8; 3],
    pub intensity: u8,
    pub exponent: u8,
    pub falloff: u8,
    /// How to interpret [`phase`](Self::phase) for modulation.
    pub waveform: PhaseWaveform,
    /// Looping animation phase (0..255 maps to 0..1).
    ///
    /// Advance this from your game (deterministic) to animate the lobe.
    pub phase: u8,
    pub alpha: u8,
}

impl Default for LobeRadianceParams {
    fn default() -> Self {
        Self {
            region: EpuRegion::All,
            blend: EpuBlend::Add,
            dir: Vec3::Y,
            color: [255, 255, 255],
            edge_color: [0, 0, 0],
            intensity: 255,
            exponent: 64,
            falloff: 64,
            waveform: PhaseWaveform::Off,
            phase: 0,
            alpha: 15,
        }
    }
}

/// Parameters for BAND_RADIANCE feature.
#[derive(Clone, Copy, Debug)]
pub struct BandRadianceParams {
    pub region: EpuRegion,
    pub blend: EpuBlend,
    pub axis: Vec3,
    pub color: [u8; 3],
    pub edge_color: [u8; 3],
    pub intensity: u8,
    pub width: u8,
    pub offset: u8,
    pub softness: u8,
    /// Looping modulation phase (0..255 maps to 0..1).
    ///
    /// Advance this from your game (deterministic) to scroll the modulation around the band.
    pub phase: u8,
    pub alpha: u8,
}

impl Default for BandRadianceParams {
    fn default() -> Self {
        Self {
            region: EpuRegion::All,
            blend: EpuBlend::Add,
            axis: Vec3::Y,
            color: [255, 255, 255],
            edge_color: [0, 0, 0],
            intensity: 255,
            width: 64,
            offset: 128,
            softness: 0,
            phase: 0,
            alpha: 15,
        }
    }
}

/// Parameters for ATMOSPHERE feature.
#[derive(Clone, Copy, Debug)]
pub struct AtmosphereParams {
    pub region: EpuRegion,
    pub blend: EpuBlend,
    pub zenith_color: [u8; 3],
    pub horizon_color: [u8; 3],
    pub intensity: u8,
    pub falloff_exponent: u8,
    pub horizon_y: u8,
    pub mie_concentration: u8,
    pub mie_exponent: u8,
    pub sun_dir: Vec3,
    pub alpha: u8,
    pub variant_id: u8,
}

impl Default for AtmosphereParams {
    fn default() -> Self {
        Self {
            region: EpuRegion::All,
            blend: EpuBlend::Add,
            zenith_color: [0, 0, 0],
            horizon_color: [0, 0, 0],
            intensity: 0,
            falloff_exponent: 128,
            horizon_y: 128,
            mie_concentration: 0,
            mie_exponent: 64,
            sun_dir: Vec3::Y,
            alpha: 15,
            variant_id: 0,
        }
    }
}
