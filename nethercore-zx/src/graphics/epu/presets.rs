//! EPU preset environment configurations
//!
//! Factory functions for common environment types from RFC Section 14.
//!
//! Each preset returns a complete, usable `EpuConfig` that can be used
//! directly with the EPU runtime. Presets follow the recommended slot
//! usage:
//!
//! - Slots 0-3: Bounds (RAMP, LOBE, BAND, FOG)
//! - Slots 4-7: Features (DECAL, GRID, SCATTER, FLOW)
//!
//! # Example
//!
//! ```ignore
//! use nethercore_zx::graphics::epu::presets;
//!
//! // Get a ready-to-use sunny meadow environment
//! let config = presets::sunny_meadow();
//!
//! // Or customize a void with stars
//! let stars = presets::void_with_stars();
//! ```

use super::{
    DecalParams, DecalShape, EpuBlend, EpuConfig, EpuRegion, FlowParams, FlowPattern, GridParams,
    GridPattern, RampParams, ScatterParams, epu_begin, epu_finish,
};
use glam::Vec3;

// =============================================================================
// Preset: Void with Stars
// =============================================================================

/// Void with stars - black background with twinkling stars.
///
/// RFC Section 14.2: Simple space environment with only stars providing light.
///
/// Layer structure:
/// - B0: RAMP (black enclosure)
/// - F0: SCATTER (emissive twinkling stars)
///
/// The stars use ADD blend mode, making them emissive light sources that
/// contribute to ambient lighting through the blur pyramid.
pub fn void_with_stars() -> EpuConfig {
    let mut e = epu_begin();

    // Fully closed "void": make everything black, minimal softness.
    e.ramp_enclosure(RampParams {
        up: Vec3::Y,
        wall_color: 0,  // black
        sky_color: 0,   // black
        floor_color: 0, // black
        ceil_q: 15,     // high ceiling threshold
        floor_q: 0,     // low floor threshold
        softness: 10,   // minimal softness
    });

    // Stars are the only light source: emissive by using blend Add.
    e.scatter(ScatterParams {
        region: EpuRegion::All,
        blend: EpuBlend::Add, // Emissive - contributes to lighting
        color: 15,            // white (grayscale palette index)
        intensity: 255,
        density: 200,
        size: 20,
        twinkle_q: 8, // moderate twinkling
        seed: 3,
    });

    epu_finish(e)
}

// =============================================================================
// Preset: Sunny Meadow
// =============================================================================

/// Sunny meadow - blue sky, green ground, sun disk with glow.
///
/// RFC Section 14.3: Classic outdoor daytime environment.
///
/// Layer structure:
/// - B0: RAMP (sky/horizon/ground gradient)
/// - B1: LOBE (sun glow)
/// - F0: DECAL (sun disk, emissive)
/// - F1: FLOW (slow cloud drift, visual-only)
///
/// The sun disk uses ADD blend (emissive), while clouds use LERP (visual-only)
/// to avoid clouds incorrectly lighting the scene.
pub fn sunny_meadow() -> EpuConfig {
    let mut e = epu_begin();

    let sun_dir = Vec3::new(0.5, 0.7, 0.3).normalize();

    // Open-ish sky enclosure with warm horizon
    e.ramp_enclosure(RampParams {
        up: Vec3::Y,
        wall_color: 24,  // warm horizon (palette index)
        sky_color: 40,   // blue sky (palette index)
        floor_color: 52, // green ground (palette index)
        ceil_q: 10,      // ceiling threshold
        floor_q: 5,      // floor threshold
        softness: 180,   // soft transitions
    });

    // Sun glow - warm directional light
    e.lobe(
        sun_dir, 20,  // warm white/yellow color
        180, // intensity
        32,  // exponent (moderate sharpness)
        0,   // no animation speed
        0,   // no animation mode
    );

    // Sun disk: emissive feature (blend Add)
    e.decal(DecalParams {
        region: EpuRegion::Sky,
        blend: EpuBlend::Add, // Emissive - contributes to lighting
        shape: DecalShape::Disk,
        dir: sun_dir,
        color: 15, // white
        intensity: 255,
        softness_q: 2,  // slight softness
        size: 12,       // small disk
        pulse_speed: 0, // no pulsing
    });

    // Slow cloud drift - visual only (LERP blend)
    e.flow(FlowParams {
        region: EpuRegion::Sky,
        blend: EpuBlend::Lerp, // Visual-only - does not light the scene
        dir: Vec3::X,          // drift direction
        color: 15,             // white clouds
        intensity: 60,         // subtle
        scale: 32,             // cloud scale
        speed: 20,             // slow drift
        octaves: 2,            // some detail
        pattern: FlowPattern::Noise,
    });

    epu_finish(e)
}

// =============================================================================
// Preset: Cyberpunk Alley
// =============================================================================

/// Cyberpunk alley - neon-lit urban environment with fog, rain, and glowing windows.
///
/// Layer structure:
/// - B0: RAMP (dark urban enclosure)
/// - B1: LOBE (left neon spill, magenta)
/// - B2: LOBE (right neon spill, cyan)
/// - B3: FOG (atmospheric haze)
/// - F0: GRID (building panels)
/// - F1: DECAL (neon sign)
/// - F2: FLOW (rain streaks)
/// - F3: SCATTER (lit windows)
///
/// Dual neon lobes create asymmetric lighting. Fog absorbs/tints the scene.
/// Grid panels and windows are emissive; rain is visual-only.
pub fn cyberpunk_alley() -> EpuConfig {
    let mut e = epu_begin();

    // Dark urban enclosure
    e.ramp_enclosure(RampParams {
        up: Vec3::Y,
        wall_color: 2,  // dark gray (palette index)
        sky_color: 1,   // near black sky
        floor_color: 3, // wet pavement
        ceil_q: 12,     // slightly open
        floor_q: 4,
        softness: 100,
    });

    // Left neon spill - magenta
    let left_neon_dir = Vec3::new(-0.7, 0.3, 0.2).normalize();
    e.lobe(
        left_neon_dir,
        72,  // magenta/pink (palette index)
        140, // intensity
        24,  // moderate spread
        0,
        0,
    );

    // Right neon spill - cyan
    let right_neon_dir = Vec3::new(0.7, 0.3, -0.2).normalize();
    e.lobe(
        right_neon_dir,
        36,  // cyan (palette index)
        120, // slightly less intense
        28,  // slightly tighter
        0,
        0,
    );

    // Atmospheric fog - absorbs and tints
    e.fog(
        Vec3::Y,
        80,  // fog tint color (palette index, muted purple-gray)
        80,  // density
        140, // vertical bias (concentrated lower)
        100, // falloff
    );

    // Building panels - emissive grid on walls
    e.grid(GridParams {
        region: EpuRegion::Walls,
        blend: EpuBlend::Add, // Emissive
        color: 64,            // neon accent (palette index)
        intensity: 80,
        scale: 48,
        thickness: 12,
        pattern: GridPattern::Grid,
        scroll_q: 0, // static
    });

    // Neon sign - emissive decal
    let sign_dir = Vec3::new(0.0, 0.2, 1.0).normalize();
    e.decal(DecalParams {
        region: EpuRegion::Walls,
        blend: EpuBlend::Add, // Emissive
        shape: DecalShape::Rect,
        dir: sign_dir,
        color: 72, // bright pink
        intensity: 200,
        softness_q: 4,
        size: 40,
        pulse_speed: 30, // subtle pulsing
    });

    // Rain streaks - visual only
    e.flow(FlowParams {
        region: EpuRegion::All,
        blend: EpuBlend::Lerp,                      // Visual-only
        dir: Vec3::new(0.1, -1.0, 0.0).normalize(), // falling
        color: 8,                                   // light gray
        intensity: 40,
        scale: 64,
        speed: 180, // fast rain
        octaves: 1,
        pattern: FlowPattern::Streaks,
    });

    // Lit windows - emissive scatter
    e.scatter(ScatterParams {
        region: EpuRegion::Walls,
        blend: EpuBlend::Add, // Emissive
        color: 20,            // warm light (palette index)
        intensity: 180,
        density: 120,
        size: 15,
        twinkle_q: 3, // subtle flicker
        seed: 7,
    });

    epu_finish(e)
}

// =============================================================================
// Preset: Underwater Cave
// =============================================================================

/// Underwater cave - caustics, bubbles, and ambient glow from above.
///
/// Layer structure:
/// - B0: RAMP (deep blue/teal enclosure)
/// - B1: LOBE (light from above)
/// - B2: FOG (water absorption)
/// - F0: FLOW (caustic patterns)
/// - F1: SCATTER (rising bubbles)
///
/// Light comes from above, fog creates depth, and caustics add underwater feel.
/// Bubbles are emissive (they catch and scatter light).
pub fn underwater_cave() -> EpuConfig {
    let mut e = epu_begin();

    // Deep underwater enclosure
    e.ramp_enclosure(RampParams {
        up: Vec3::Y,
        wall_color: 35,  // dark teal (palette index)
        sky_color: 38,   // lighter blue-green above
        floor_color: 33, // very dark blue floor
        ceil_q: 11,
        floor_q: 4,
        softness: 140,
    });

    // Light from above - filtered sunlight
    e.lobe(
        Vec3::Y, // straight up
        38,      // blue-green tint
        100,     // moderate intensity
        16,      // broad spread
        0,
        0,
    );

    // Water absorption fog
    e.fog(
        Vec3::Y,
        34,  // deep blue tint
        120, // dense
        100, // some vertical bias
        80,  // gradual falloff
    );

    // Caustic patterns - emissive (light dances on surfaces)
    e.flow(FlowParams {
        region: EpuRegion::All,
        blend: EpuBlend::Add, // Emissive - caustics are light
        dir: Vec3::new(0.2, -0.5, 0.3).normalize(),
        color: 40, // bright cyan
        intensity: 80,
        scale: 40,
        speed: 40, // slow undulation
        octaves: 2,
        pattern: FlowPattern::Caustic,
    });

    // Rising bubbles - emissive (catch light)
    e.scatter(ScatterParams {
        region: EpuRegion::All,
        blend: EpuBlend::Add, // Emissive
        color: 15,            // white highlights
        intensity: 140,
        density: 80,
        size: 25,
        twinkle_q: 6, // shimmer as they rise
        seed: 12,
    });

    epu_finish(e)
}

// =============================================================================
// Preset: Space Station
// =============================================================================

/// Space station - industrial panels, overhead lighting, warning indicators.
///
/// Layer structure:
/// - B0: RAMP (metallic gray enclosure)
/// - B1: LOBE (overhead fluorescent)
/// - B2: BAND (horizon accent strip)
/// - F0: GRID (wall panels)
/// - F1: DECAL (warning light)
/// - F2: SCATTER (indicator lights)
///
/// Industrial interior with structured lighting. Warning decal pulses,
/// indicator lights are scattered on walls.
pub fn space_station() -> EpuConfig {
    let mut e = epu_begin();

    // Industrial metallic enclosure
    e.ramp_enclosure(RampParams {
        up: Vec3::Y,
        wall_color: 6,  // medium gray (palette index)
        sky_color: 4,   // dark ceiling
        floor_color: 5, // slightly darker floor
        ceil_q: 10,
        floor_q: 6,
        softness: 60, // fairly crisp transitions
    });

    // Overhead fluorescent lighting
    e.lobe(
        Vec3::Y, // from above
        10,      // cool white (palette index)
        200,     // bright
        20,      // moderate spread
        0,
        0,
    );

    // Horizon accent band - subtle industrial strip
    e.band(
        Vec3::Y,
        64,  // accent color (palette index)
        60,  // subtle intensity
        40,  // width
        128, // centered at horizon
        0,   // no scroll
    );

    // Wall panels - emissive grid
    e.grid(GridParams {
        region: EpuRegion::Walls,
        blend: EpuBlend::Add, // Emissive
        color: 8,             // light gray
        intensity: 60,
        scale: 24,
        thickness: 8,
        pattern: GridPattern::Grid,
        scroll_q: 0, // static
    });

    // Warning light - pulsing emissive decal
    let warning_dir = Vec3::new(0.0, 0.3, 0.8).normalize();
    e.decal(DecalParams {
        region: EpuRegion::Walls,
        blend: EpuBlend::Add, // Emissive
        shape: DecalShape::Disk,
        dir: warning_dir,
        color: 18, // orange/red warning (palette index)
        intensity: 220,
        softness_q: 3,
        size: 20,
        pulse_speed: 60, // pulsing warning
    });

    // Indicator lights - scattered on walls
    e.scatter(ScatterParams {
        region: EpuRegion::Walls,
        blend: EpuBlend::Add, // Emissive
        color: 68,            // green indicator (palette index)
        intensity: 160,
        density: 60,
        size: 10,
        twinkle_q: 2, // occasional blink
        seed: 5,
    });

    epu_finish(e)
}

// =============================================================================
// Preset: Sunset Beach
// =============================================================================

/// Sunset beach - warm horizon, soft sky gradient, sun near horizon.
///
/// Layer structure:
/// - B0: RAMP (sunset gradient)
/// - B1: LOBE (sun glow at horizon)
/// - F0: DECAL (sun disk, low on horizon)
/// - F1: FLOW (gentle cloud wisps)
///
/// A peaceful sunset scene with warm colors and soft transitions.
pub fn sunset_beach() -> EpuConfig {
    let mut e = epu_begin();

    // Sunset gradient enclosure
    e.ramp_enclosure(RampParams {
        up: Vec3::Y,
        wall_color: 22,  // warm orange horizon (palette index)
        sky_color: 28,   // purple-blue upper sky
        floor_color: 48, // sandy beach
        ceil_q: 10,
        floor_q: 4,
        softness: 200, // very soft transitions
    });

    // Sun glow near horizon
    let sun_dir = Vec3::new(0.8, 0.15, 0.3).normalize();
    e.lobe(
        sun_dir, 20,  // warm yellow-orange
        220, // bright
        24,  // moderate spread
        0, 0,
    );

    // Sun disk - emissive, low on horizon
    e.decal(DecalParams {
        region: EpuRegion::All,
        blend: EpuBlend::Add, // Emissive
        shape: DecalShape::Disk,
        dir: sun_dir,
        color: 16, // bright orange-yellow
        intensity: 255,
        softness_q: 4, // soft edge (atmospheric)
        size: 18,
        pulse_speed: 0,
    });

    // Gentle cloud wisps - visual only
    e.flow(FlowParams {
        region: EpuRegion::Sky,
        blend: EpuBlend::Lerp, // Visual-only
        dir: Vec3::X,
        color: 22, // warm tinted clouds
        intensity: 50,
        scale: 48,
        speed: 15, // very slow drift
        octaves: 2,
        pattern: FlowPattern::Noise,
    });

    epu_finish(e)
}

// =============================================================================
// Preset: Haunted Forest
// =============================================================================

/// Haunted forest - dark, foggy, with eerie scattered lights.
///
/// Layer structure:
/// - B0: RAMP (dark forest enclosure)
/// - B1: LOBE (dim moonlight from above)
/// - B2: FOG (thick ground fog)
/// - F0: SCATTER (fireflies/will-o-wisps)
///
/// Oppressive atmosphere with fog and scattered glowing elements.
pub fn haunted_forest() -> EpuConfig {
    let mut e = epu_begin();

    // Dark forest enclosure
    e.ramp_enclosure(RampParams {
        up: Vec3::Y,
        wall_color: 49,  // dark green-brown (palette index)
        sky_color: 1,    // near black sky
        floor_color: 50, // dark earth
        ceil_q: 13,
        floor_q: 3,
        softness: 80,
    });

    // Dim moonlight
    let moon_dir = Vec3::new(-0.3, 0.8, 0.2).normalize();
    e.lobe(
        moon_dir, 9,  // pale blue-white
        60, // dim
        12, // broad, diffuse
        0, 0,
    );

    // Thick ground fog
    e.fog(
        Vec3::Y,
        82,  // pale gray-green tint
        160, // very dense
        80,  // concentrated low
        60,  // sharp falloff
    );

    // Will-o-wisps / fireflies - emissive
    e.scatter(ScatterParams {
        region: EpuRegion::All,
        blend: EpuBlend::Add, // Emissive
        color: 68,            // pale green glow
        intensity: 200,
        density: 40, // sparse
        size: 30,
        twinkle_q: 12, // strong flicker
        seed: 9,
    });

    epu_finish(e)
}

// =============================================================================
// Preset: Lava Cave
// =============================================================================

/// Lava cave - hot, glowing environment with flowing lava patterns.
///
/// Layer structure:
/// - B0: RAMP (dark rock with warm floor)
/// - B1: LOBE (glow from below)
/// - B2: FOG (heat haze)
/// - F0: FLOW (lava flow patterns)
/// - F1: SCATTER (embers)
///
/// Intense heat and glow from molten rock below.
pub fn lava_cave() -> EpuConfig {
    let mut e = epu_begin();

    // Dark rock with glowing floor
    e.ramp_enclosure(RampParams {
        up: Vec3::Y,
        wall_color: 3,   // dark rock
        sky_color: 2,    // very dark ceiling
        floor_color: 17, // bright orange-red lava
        ceil_q: 12,
        floor_q: 5,
        softness: 100,
    });

    // Glow from below
    e.lobe(
        -Vec3::Y, // from below
        18,       // bright orange
        180,
        20,
        0,
        0,
    );

    // Heat haze
    e.fog(
        Vec3::Y,
        17, // orange-red tint
        60,
        60, // bias towards floor
        120,
    );

    // Lava flow patterns - emissive
    e.flow(FlowParams {
        region: EpuRegion::Floor,
        blend: EpuBlend::Add, // Emissive
        dir: Vec3::X,
        color: 16, // bright orange-yellow
        intensity: 200,
        scale: 24,
        speed: 30, // slow flow
        octaves: 2,
        pattern: FlowPattern::Noise,
    });

    // Rising embers - emissive
    e.scatter(ScatterParams {
        region: EpuRegion::All,
        blend: EpuBlend::Add, // Emissive
        color: 17,            // orange
        intensity: 220,
        density: 50,
        size: 18,
        twinkle_q: 10, // flicker
        seed: 4,
    });

    epu_finish(e)
}

// =============================================================================
// Tests
// =============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graphics::epu::EpuOpcode;

    /// Helper to extract opcode from encoded layer
    fn layer_opcode(layer: u64) -> u8 {
        ((layer >> 60) & 0xF) as u8
    }

    /// Helper to extract blend mode from encoded layer
    fn layer_blend(layer: u64) -> u8 {
        ((layer >> 56) & 0x3) as u8
    }

    #[test]
    fn test_void_with_stars_structure() {
        let config = void_with_stars();

        // Should have RAMP in slot 0
        assert_eq!(layer_opcode(config.layers[0]), EpuOpcode::Ramp as u8);

        // Should have SCATTER in slot 4 (first feature slot)
        assert_eq!(layer_opcode(config.layers[4]), EpuOpcode::Scatter as u8);

        // Scatter should be emissive (ADD blend)
        assert_eq!(layer_blend(config.layers[4]), EpuBlend::Add as u8);

        // Other slots should be NOP
        assert_eq!(layer_opcode(config.layers[1]), EpuOpcode::Nop as u8);
        assert_eq!(layer_opcode(config.layers[2]), EpuOpcode::Nop as u8);
        assert_eq!(layer_opcode(config.layers[3]), EpuOpcode::Nop as u8);
    }

    #[test]
    fn test_sunny_meadow_structure() {
        let config = sunny_meadow();

        // B0: RAMP, B1: LOBE
        assert_eq!(layer_opcode(config.layers[0]), EpuOpcode::Ramp as u8);
        assert_eq!(layer_opcode(config.layers[1]), EpuOpcode::Lobe as u8);

        // F0: DECAL, F1: FLOW
        assert_eq!(layer_opcode(config.layers[4]), EpuOpcode::Decal as u8);
        assert_eq!(layer_opcode(config.layers[5]), EpuOpcode::Flow as u8);

        // Sun decal should be emissive
        assert_eq!(layer_blend(config.layers[4]), EpuBlend::Add as u8);

        // Clouds should be visual-only (LERP)
        assert_eq!(layer_blend(config.layers[5]), EpuBlend::Lerp as u8);
    }

    #[test]
    fn test_cyberpunk_alley_structure() {
        let config = cyberpunk_alley();

        // B0: RAMP, B1: LOBE, B2: LOBE, B3: FOG
        assert_eq!(layer_opcode(config.layers[0]), EpuOpcode::Ramp as u8);
        assert_eq!(layer_opcode(config.layers[1]), EpuOpcode::Lobe as u8);
        assert_eq!(layer_opcode(config.layers[2]), EpuOpcode::Lobe as u8);
        assert_eq!(layer_opcode(config.layers[3]), EpuOpcode::Fog as u8);

        // F0: GRID, F1: DECAL, F2: FLOW, F3: SCATTER
        assert_eq!(layer_opcode(config.layers[4]), EpuOpcode::Grid as u8);
        assert_eq!(layer_opcode(config.layers[5]), EpuOpcode::Decal as u8);
        assert_eq!(layer_opcode(config.layers[6]), EpuOpcode::Flow as u8);
        assert_eq!(layer_opcode(config.layers[7]), EpuOpcode::Scatter as u8);

        // FOG should use MULTIPLY blend
        assert_eq!(layer_blend(config.layers[3]), EpuBlend::Multiply as u8);

        // Rain (FLOW) should be visual-only (LERP)
        assert_eq!(layer_blend(config.layers[6]), EpuBlend::Lerp as u8);
    }

    #[test]
    fn test_underwater_cave_structure() {
        let config = underwater_cave();

        // B0: RAMP, B1: LOBE, B2: FOG
        assert_eq!(layer_opcode(config.layers[0]), EpuOpcode::Ramp as u8);
        assert_eq!(layer_opcode(config.layers[1]), EpuOpcode::Lobe as u8);
        assert_eq!(layer_opcode(config.layers[2]), EpuOpcode::Fog as u8);

        // F0: FLOW (caustics), F1: SCATTER (bubbles)
        assert_eq!(layer_opcode(config.layers[4]), EpuOpcode::Flow as u8);
        assert_eq!(layer_opcode(config.layers[5]), EpuOpcode::Scatter as u8);

        // Caustics should be emissive
        assert_eq!(layer_blend(config.layers[4]), EpuBlend::Add as u8);
    }

    #[test]
    fn test_space_station_structure() {
        let config = space_station();

        // B0: RAMP, B1: LOBE, B2: BAND
        assert_eq!(layer_opcode(config.layers[0]), EpuOpcode::Ramp as u8);
        assert_eq!(layer_opcode(config.layers[1]), EpuOpcode::Lobe as u8);
        assert_eq!(layer_opcode(config.layers[2]), EpuOpcode::Band as u8);

        // F0: GRID, F1: DECAL, F2: SCATTER
        assert_eq!(layer_opcode(config.layers[4]), EpuOpcode::Grid as u8);
        assert_eq!(layer_opcode(config.layers[5]), EpuOpcode::Decal as u8);
        assert_eq!(layer_opcode(config.layers[6]), EpuOpcode::Scatter as u8);
    }

    #[test]
    fn test_sunset_beach_returns_valid_config() {
        let config = sunset_beach();
        assert_eq!(layer_opcode(config.layers[0]), EpuOpcode::Ramp as u8);
        assert_eq!(layer_opcode(config.layers[1]), EpuOpcode::Lobe as u8);
        assert_eq!(layer_opcode(config.layers[4]), EpuOpcode::Decal as u8);
    }

    #[test]
    fn test_haunted_forest_returns_valid_config() {
        let config = haunted_forest();
        assert_eq!(layer_opcode(config.layers[0]), EpuOpcode::Ramp as u8);
        assert_eq!(layer_opcode(config.layers[1]), EpuOpcode::Lobe as u8);
        assert_eq!(layer_opcode(config.layers[2]), EpuOpcode::Fog as u8);
        assert_eq!(layer_opcode(config.layers[4]), EpuOpcode::Scatter as u8);
    }

    #[test]
    fn test_lava_cave_returns_valid_config() {
        let config = lava_cave();
        assert_eq!(layer_opcode(config.layers[0]), EpuOpcode::Ramp as u8);
        assert_eq!(layer_opcode(config.layers[1]), EpuOpcode::Lobe as u8);
        assert_eq!(layer_opcode(config.layers[2]), EpuOpcode::Fog as u8);
        assert_eq!(layer_opcode(config.layers[4]), EpuOpcode::Flow as u8);
        assert_eq!(layer_opcode(config.layers[5]), EpuOpcode::Scatter as u8);
    }

    #[test]
    fn test_all_presets_return_64_byte_configs() {
        let configs = [
            void_with_stars(),
            sunny_meadow(),
            cyberpunk_alley(),
            underwater_cave(),
            space_station(),
            sunset_beach(),
            haunted_forest(),
            lava_cave(),
        ];

        for config in &configs {
            assert_eq!(
                std::mem::size_of_val(config),
                64,
                "EpuConfig must be exactly 64 bytes"
            );
        }
    }

    #[test]
    fn test_presets_have_consistent_ramp_in_slot_0() {
        // All presets should have RAMP as their first layer (slot 0)
        let configs = [
            void_with_stars(),
            sunny_meadow(),
            cyberpunk_alley(),
            underwater_cave(),
            space_station(),
            sunset_beach(),
            haunted_forest(),
            lava_cave(),
        ];

        for config in &configs {
            assert_eq!(
                layer_opcode(config.layers[0]),
                EpuOpcode::Ramp as u8,
                "All presets should have RAMP in slot 0"
            );
        }
    }
}
