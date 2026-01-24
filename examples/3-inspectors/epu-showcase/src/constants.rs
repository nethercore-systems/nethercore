//! EPU Constants and Helper Functions
//!
//! Contains opcode definitions, region flags, blend modes, direction constants,
//! and helper functions for building EPU 128-bit layer configurations.
//!
//! The meta5 field (bits 52..48) encodes domain/variant:
//! - meta5 = (meta_hi << 1) | meta_lo (5 bits total, 0..31)
//! - domain_id = (meta5 >> 3) & 0b11 (0..3)
//! - variant_id = meta5 & 0b111 (0..7)

// =============================================================================
// Helper Functions for 128-bit format
// =============================================================================

/// Build hi word (bits 127..64 of the 128-bit layer)
///
/// Layout:
/// - bits 63..59: opcode     (5)
/// - bits 58..56: region     (3)
/// - bits 55..53: blend      (3)
/// - bits 52..49: meta_hi    (4) - domain/variant (high bits)
/// - bit  48:     meta_lo    (1) - domain/variant (low bit)
/// - bits 47..24: color_a    (24) - RGB24 primary color
/// - bits 23..0:  color_b    (24) - RGB24 secondary color
///
/// For basic opcodes, set meta5 to 0 (pass 0 as the fourth parameter).
/// For opcodes with domain/variant, use hi_meta() or compute meta5 = (domain_id << 3) | variant_id.
pub const fn hi(
    opcode: u64,
    region: u64,
    blend: u64,
    meta5: u64,
    color_a: u64,
    color_b: u64,
) -> u64 {
    let meta_hi = (meta5 >> 1) & 0xF;
    let meta_lo = meta5 & 0x1;
    ((opcode & 0x1F) << 59)
        | ((region & 0x7) << 56)
        | ((blend & 0x7) << 53)
        | ((meta_hi & 0xF) << 49)
        | ((meta_lo & 0x1) << 48)
        | ((color_a & 0xFFFFFF) << 24)
        | (color_b & 0xFFFFFF)
}

/// Build hi word with explicit domain and variant
///
/// Convenience wrapper that packs domain_id and variant_id into meta5.
/// - domain_id: 0=DIRECT3D, 1=AXIS_CYL, 2=AXIS_POLAR, 3=TANGENT_LOCAL
/// - variant_id: opcode-specific variant (0..7)
#[allow(clippy::too_many_arguments)]
pub const fn hi_meta(
    opcode: u64,
    region: u64,
    blend: u64,
    domain_id: u64,
    variant_id: u64,
    color_a: u64,
    color_b: u64,
) -> u64 {
    let meta5 = ((domain_id & 0x3) << 3) | (variant_id & 0x7);
    hi(opcode, region, blend, meta5, color_a, color_b)
}

/// Pack direction from 0xUUVV (u high byte, v low byte) to engine format
///
/// Note: direction is authored as 0xUUVV (u in high byte, v in low byte),
/// but the engine/shader decode expects low byte = u, high byte = v.
/// This function swaps bytes during packing to match the runtime format.
pub const fn pack_dir16_uv(direction_uv: u64) -> u64 {
    let u = (direction_uv >> 8) & 0xFF;
    let v = direction_uv & 0xFF;
    (u & 0xFF) | ((v & 0xFF) << 8)
}

/// Build lo word (bits 63..0 of the 128-bit layer)
///
/// Layout:
/// - bits 63..56: intensity  (8)
/// - bits 55..48: param_a    (8)
/// - bits 47..40: param_b    (8)
/// - bits 39..32: param_c    (8)
/// - bits 31..24: param_d    (8)
/// - bits 23..8:  direction  (16) - Octahedral encoded
/// - bits 7..4:   alpha_a    (4) - color_a alpha (0-15)
/// - bits 3..0:   alpha_b    (4) - color_b alpha (0-15)
#[allow(clippy::too_many_arguments)]
pub const fn lo(
    intensity: u64,
    param_a: u64,
    param_b: u64,
    param_c: u64,
    param_d: u64,
    direction: u64,
    alpha_a: u64,
    alpha_b: u64,
) -> u64 {
    ((intensity & 0xFF) << 56)
        | ((param_a & 0xFF) << 48)
        | ((param_b & 0xFF) << 40)
        | ((param_c & 0xFF) << 32)
        | ((param_d & 0xFF) << 24)
        | ((pack_dir16_uv(direction) & 0xFFFF) << 8)
        | ((alpha_a & 0xF) << 4)
        | (alpha_b & 0xF)
}

// =============================================================================
// Opcodes (NOP=0; bounds=1..7; features=8..)
// =============================================================================

pub const OP_RAMP: u64 = 0x01;
// 0x02: SECTOR (enclosure)
// 0x03: SILHOUETTE (enclosure)
// 0x04: SPLIT (enclosure)
// 0x05: CELL (enclosure)
// 0x06: PATCHES (enclosure)
// 0x07: APERTURE (enclosure)
pub const OP_DECAL: u64 = 0x08;
pub const OP_GRID: u64 = 0x09;
pub const OP_SCATTER: u64 = 0x0A;
pub const OP_FLOW: u64 = 0x0B;

// =============================================================================
// Enclosure Opcodes (0x02-0x07)
// =============================================================================

/// SECTOR - Azimuthal opening sector / interior cues
/// Variants: 0=BOX, 1=TUNNEL, 2=CAVE
pub const OP_SECTOR: u64 = 0x02;

/// SILHOUETTE - Skyline/horizon cutout
/// Variants: 0=MOUNTAINS, 1=CITY, 2=FOREST, 3=DUNES, 4=WAVES, 5=RUINS, 6=INDUSTRIAL, 7=SPIRES
pub const OP_SILHOUETTE: u64 = 0x03;

/// SPLIT - Planar cut enclosure
/// Variants: 0=HALF, 1=WEDGE, 2=CORNER, 3=BANDS, 4=CROSS, 5=PRISM
pub const OP_SPLIT: u64 = 0x04;

/// CELL - Voronoi/mosaic cells
/// Variants: 0=GRID, 1=HEX, 2=VORONOI, 3=RADIAL, 4=SHATTER, 5=BRICK
pub const OP_CELL: u64 = 0x05;

/// PATCHES - Noise patches
/// Domains: 0=DIRECT3D, 1=AXIS_CYL, 2=AXIS_POLAR
/// Variants: 0=BLOBS, 1=ISLANDS, 2=DEBRIS, 3=MEMBRANE, 4=STATIC, 5=STREAKS
pub const OP_PATCHES: u64 = 0x06;

/// APERTURE - Shaped opening/viewport
/// Variants: 0=CIRCLE, 1=RECT, 2=ROUNDED_RECT, 3=ARCH, 4=BARS, 5=MULTI, 6=IRREGULAR
pub const OP_APERTURE: u64 = 0x07;

// =============================================================================
// Radiance Opcodes (0x0C-0x13)
// =============================================================================

/// TRACE - Line/crack patterns (lightning, cracks, lead lines, filaments)
/// Domains: 1=AXIS_CYL, 2=AXIS_POLAR, 3=TANGENT_LOCAL
/// Variants: 0=LIGHTNING, 1=CRACKS, 2=LEAD_LINES, 3=FILAMENTS
pub const OP_TRACE: u64 = 0x0C;

/// VEIL - Curtain/ribbon effects
/// Domains: 1=AXIS_CYL, 2=AXIS_POLAR
/// Variants: 0=CURTAINS, 1=PILLARS, 2=LASER_BARS, 3=RAIN_WALL, 4=SHARDS
pub const OP_VEIL: u64 = 0x0D;

/// ATMOSPHERE - Advanced fog/scattering
/// Variants: 0=ABSORPTION, 1=RAYLEIGH, 2=MIE, 3=FULL, 4=ALIEN
pub const OP_ATMOSPHERE: u64 = 0x0E;

/// PLANE - Ground/surface textures
/// Variants: 0=TILES, 1=HEX, 2=STONE, 3=SAND, 4=WATER, 5=GRATING, 6=GRASS, 7=PAVEMENT
pub const OP_PLANE: u64 = 0x0F;

/// CELESTIAL - Moon/sun/planet bodies
/// Variants: 0=MOON, 1=SUN, 2=PLANET, 3=GAS_GIANT, 4=RINGED, 5=BINARY, 6=ECLIPSE
pub const OP_CELESTIAL: u64 = 0x10;

/// PORTAL - Vortex/portal effects
/// Domains: 3=TANGENT_LOCAL (fixed)
/// Variants: 0=CIRCLE, 1=RECT, 2=TEAR, 3=VORTEX, 4=CRACK, 5=RIFT
pub const OP_PORTAL: u64 = 0x11;

/// LOBE - Directional glow
pub const OP_LOBE: u64 = 0x12;

/// BAND - Horizon band
pub const OP_BAND: u64 = 0x13;

// =============================================================================
// Domain ID Constants (for meta5 encoding)
// =============================================================================

/// DIRECT3D - Spherical/infinite 3D motifs (no chart)
pub const DOMAIN_DIRECT3D: u64 = 0;
/// AXIS_CYL - Cylindrical wrap-around (columns, curtains, rain)
pub const DOMAIN_AXIS_CYL: u64 = 1;
/// AXIS_POLAR - Radial/spoke patterns (starbursts, radial grids)
pub const DOMAIN_AXIS_POLAR: u64 = 2;
/// TANGENT_LOCAL - Local tangent-plane chart (portals, decals, SDFs)
pub const DOMAIN_TANGENT_LOCAL: u64 = 3;

// =============================================================================
// Variant ID Constants (opcode-specific)
// =============================================================================

// SECTOR variants
pub const SECTOR_BOX: u64 = 0;
pub const SECTOR_TUNNEL: u64 = 1;
pub const SECTOR_CAVE: u64 = 2;

// CELL variants
pub const CELL_GRID: u64 = 0;
pub const CELL_HEX: u64 = 1;
pub const CELL_VORONOI: u64 = 2;
pub const CELL_RADIAL: u64 = 3;
pub const CELL_SHATTER: u64 = 4;
pub const CELL_BRICK: u64 = 5;

// PATCHES variants
pub const PATCHES_BLOBS: u64 = 0;
pub const PATCHES_ISLANDS: u64 = 1;
pub const PATCHES_DEBRIS: u64 = 2;
pub const PATCHES_MEMBRANE: u64 = 3;
pub const PATCHES_STATIC: u64 = 4;
pub const PATCHES_STREAKS: u64 = 5;

// APERTURE variants
pub const APERTURE_CIRCLE: u64 = 0;
pub const APERTURE_RECT: u64 = 1;
pub const APERTURE_ROUNDED_RECT: u64 = 2;
pub const APERTURE_ARCH: u64 = 3;
pub const APERTURE_BARS: u64 = 4;
pub const APERTURE_MULTI: u64 = 5;
pub const APERTURE_IRREGULAR: u64 = 6;

// TRACE variants
pub const TRACE_LIGHTNING: u64 = 0;
pub const TRACE_CRACKS: u64 = 1;
pub const TRACE_LEAD_LINES: u64 = 2;
pub const TRACE_FILAMENTS: u64 = 3;

// VEIL variants
pub const VEIL_CURTAINS: u64 = 0;
pub const VEIL_PILLARS: u64 = 1;
pub const VEIL_LASER_BARS: u64 = 2;
pub const VEIL_RAIN_WALL: u64 = 3;
pub const VEIL_SHARDS: u64 = 4;

// ATMOSPHERE variants
pub const ATMO_ABSORPTION: u64 = 0;
pub const ATMO_RAYLEIGH: u64 = 1;
pub const ATMO_MIE: u64 = 2;
pub const ATMO_FULL: u64 = 3;
pub const ATMO_ALIEN: u64 = 4;

// PLANE variants
pub const PLANE_TILES: u64 = 0;
pub const PLANE_HEX: u64 = 1;
pub const PLANE_STONE: u64 = 2;
pub const PLANE_SAND: u64 = 3;
pub const PLANE_WATER: u64 = 4;
pub const PLANE_GRATING: u64 = 5;
pub const PLANE_GRASS: u64 = 6;
pub const PLANE_PAVEMENT: u64 = 7;

// CELESTIAL variants
pub const CELESTIAL_MOON: u64 = 0;
pub const CELESTIAL_SUN: u64 = 1;
pub const CELESTIAL_PLANET: u64 = 2;
pub const CELESTIAL_GAS_GIANT: u64 = 3;
pub const CELESTIAL_RINGED: u64 = 4;
pub const CELESTIAL_BINARY: u64 = 5;
pub const CELESTIAL_ECLIPSE: u64 = 6;

// PORTAL variants
pub const PORTAL_CIRCLE: u64 = 0;
pub const PORTAL_RECT: u64 = 1;
pub const PORTAL_TEAR: u64 = 2;
pub const PORTAL_VORTEX: u64 = 3;
pub const PORTAL_CRACK: u64 = 4;
pub const PORTAL_RIFT: u64 = 5;

// SPLIT variants
pub const SPLIT_HALF: u64 = 0;
pub const SPLIT_WEDGE: u64 = 1;
pub const SPLIT_CORNER: u64 = 2;
pub const SPLIT_BANDS: u64 = 3;
pub const SPLIT_CROSS: u64 = 4;
pub const SPLIT_PRISM: u64 = 5;

// SILHOUETTE variants
pub const SILHOUETTE_MOUNTAINS: u64 = 0;
pub const SILHOUETTE_CITY: u64 = 1;
pub const SILHOUETTE_FOREST: u64 = 2;
pub const SILHOUETTE_DUNES: u64 = 3;
pub const SILHOUETTE_WAVES: u64 = 4;
pub const SILHOUETTE_RUINS: u64 = 5;
pub const SILHOUETTE_INDUSTRIAL: u64 = 6;
pub const SILHOUETTE_SPIRES: u64 = 7;

// =============================================================================
// Region Flags
// =============================================================================

pub const REGION_ALL: u64 = 0b111;
pub const REGION_SKY: u64 = 0b100;
pub const REGION_WALLS: u64 = 0b010;
pub const REGION_FLOOR: u64 = 0b001;

// =============================================================================
// Blend Modes (ADD=0, MULTIPLY=1, MAX=2, LERP=3, SCREEN=4, HSV_MOD=5, MIN=6, OVERLAY=7)
// =============================================================================

pub const BLEND_ADD: u64 = 0;
pub const BLEND_MULTIPLY: u64 = 1;
pub const BLEND_MAX: u64 = 2;
pub const BLEND_LERP: u64 = 3;
pub const BLEND_SCREEN: u64 = 4;
pub const BLEND_HSV_MOD: u64 = 5;
pub const BLEND_MIN: u64 = 6;
pub const BLEND_OVERLAY: u64 = 7;

// =============================================================================
// Direction Constants (Octahedral Encoding)
// =============================================================================

/// Direction for +Y (up) in octahedral encoding: u=128, v=255
pub const DIR_UP: u64 = 0x80FF;
/// Direction for -Y (down) in octahedral encoding: u=128, v=0
pub const DIR_DOWN: u64 = 0x8000;
/// Direction for sun (0.5, 0.7, 0.3 normalized): approximately
pub const DIR_SUN: u64 = 0xC0A0;
/// Direction for a low sun near the horizon (setting sun)
pub const DIR_SUNSET: u64 = 0xC190;

// =============================================================================
// NOP Layer
// =============================================================================

/// NOP layer (disabled)
pub const NOP_LAYER: [u64; 2] = [0, 0];
