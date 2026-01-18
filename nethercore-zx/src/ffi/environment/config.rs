//! Environment mode configuration functions
//!
//! This module contains the FFI functions for configuring each of the 8 environment modes.
//! Each function configures parameters for a specific procedural environment type.

use wasmtime::Caller;

use glam::Vec3;

use crate::ffi::ZXGameContext;
use crate::graphics::env_mode;
use crate::graphics::unified_shading_state::{
    CellsConfig, ENV_OVERLAY_OFFSET, GradientConfig, LinesConfig, NebulaConfig, RingsConfig,
    RoomConfig, SilhouetteConfig, VeilConfig,
};

#[inline]
fn layer_offset_words(layer: u32) -> usize {
    if layer == 0 { 0 } else { ENV_OVERLAY_OFFSET }
}

/// Configure gradient environment (Mode 0)
///
/// # Arguments
/// * `layer` — Target layer: 0 = base layer, 1 = overlay layer
/// * `zenith` — Color directly overhead (0xRRGGBBAA)
/// * `sky_horizon` — Sky color at horizon level (0xRRGGBBAA)
/// * `ground_horizon` — Ground color at horizon level (0xRRGGBBAA)
/// * `nadir` — Color directly below (0xRRGGBBAA)
/// * `rotation` — Sun azimuth around Y axis in radians (0 = +Z, π/2 = +X)
/// * `shift` — Horizon vertical shift (-1.0 to 1.0, 0.0 = equator)
/// * `sun_elevation` — Sun elevation in radians (0 = horizon, π/2 = zenith)
/// * `sun_disk` — Sun disc size (0-255)
/// * `sun_halo` — Sun halo size (0-255)
/// * `sun_intensity` — Sun intensity (0 disables sun)
/// * `horizon_haze` — Haze near the horizon (0-255)
/// * `sun_warmth` — Sun color warmth (0 = neutral/white, 255 = warm/orange)
/// * `cloudiness` — Stylized cloud bands (0 disables, 255 = strongest)
///
/// Sets the 4-color gradient for environment rendering. The gradient interpolates:
/// - zenith → sky_horizon (Y > 0)
/// - sky_horizon → ground_horizon (at Y = 0 + shift)
/// - ground_horizon → nadir (Y < 0)
///
/// You can configure the same mode on both layers with different parameters:
/// ```ignore
/// env_gradient(0, ...);  // Base layer
/// env_gradient(1, ...);  // Overlay layer
/// ```
///
/// **Examples:**
/// - Blue sky: `env_gradient(0, 0x191970FF, 0x87CEEBFF, 0x228B22FF, 0x2F4F4FFF, 0.0, 0.0, 0.0, 0, 0, 0, 0, 0, 0, 0)`
/// - Sunset: `env_gradient(0, 0x4A00E0FF, 0xFF6B6BFF, 0x8B4513FF, 0x2F2F2FFF, 0.0, 0.1, 0.0, 0, 0, 0, 0, 0, 0, 0)`
pub(crate) fn env_gradient(
    mut caller: Caller<'_, ZXGameContext>,
    layer: u32,
    zenith: u32,
    sky_horizon: u32,
    ground_horizon: u32,
    nadir: u32,
    rotation: f32,
    shift: f32,
    sun_elevation: f32,
    sun_disk: u32,
    sun_halo: u32,
    sun_intensity: u32,
    horizon_haze: u32,
    sun_warmth: u32,
    cloudiness: u32,
    cloud_phase: u32,
) {
    let state = &mut caller.data_mut().ffi;
    let offset = layer_offset_words(layer);

    // Pack gradient into specified layer (0 = base, 1 = overlay)
    state
        .current_environment_state
        .pack_gradient(GradientConfig {
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
        });

    // Set mode for the specified layer
    if layer == 0 {
        state
            .current_environment_state
            .set_base_mode(env_mode::GRADIENT);
    } else {
        state
            .current_environment_state
            .set_overlay_mode(env_mode::GRADIENT);
    }

    state.environment_dirty = true;
}

/// Configure cells environment (Mode 1).
///
/// Two families under one mode ID:
/// - Family 0: Particles (stars/snow/rain/embers/bubbles/warp)
/// - Family 1: Tiles/Lights (Mondrian/Truchet, buildings, bands, panels)
pub(crate) fn env_cells(
    mut caller: Caller<'_, ZXGameContext>,
    layer: u32,
    family: u32,
    variant: u32,
    density: u32,
    size_min: u32,
    size_max: u32,
    intensity: u32,
    shape: u32,
    motion: u32,
    parallax: u32,
    height_bias: u32,
    clustering: u32,
    color_a: u32,
    color_b: u32,
    axis_x: f32,
    axis_y: f32,
    axis_z: f32,
    phase: u32,
    seed: u32,
) {
    let state = &mut caller.data_mut().ffi;

    // Pack cells into specified layer
    state.current_environment_state.pack_cells(CellsConfig {
        offset: layer_offset_words(layer),
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
        axis: Vec3::new(axis_x, axis_y, axis_z),
        phase,
        seed,
    });

    // Set mode for the specified layer
    if layer == 0 {
        state
            .current_environment_state
            .set_base_mode(env_mode::CELLS);
    } else {
        state
            .current_environment_state
            .set_overlay_mode(env_mode::CELLS);
    }

    state.environment_dirty = true;
}

/// Configure lines environment (Mode 2)
///
/// # Arguments
/// * `layer` — Target layer: 0 = base layer, 1 = overlay layer
/// * `variant` — Surface type: 0=Floor, 1=Ceiling, 2=Sphere
/// * `line_type` — Line pattern: 0=Horizontal, 1=Vertical, 2=Grid
/// * `thickness` — Line thickness (0-255)
/// * `spacing` — Distance between lines (world units)
/// * `fade_distance` — Distance where lines start fading (world units)
/// * `parallax` — Horizon band perspective bias (0 disables, 255 = strongest)
/// * `color_primary` — Main line color (0xRRGGBBAA)
/// * `color_accent` — Accent line color (0xRRGGBBAA)
/// * `accent_every` — Make every Nth line use accent color
/// * `phase` — Scroll phase (0-65535, wraps for seamless looping)
///
/// Creates an infinite procedural grid. Good for:
/// - **Synthwave:** Floor grid with pink/cyan colors
/// - **Racing games:** Track lines scrolling with speed
/// - **Holographic:** Spherical grid overlay
///
/// **Animation:** Increment `phase` for scrolling:
/// - Racing: `phase = phase.wrapping_add((velocity.z * delta_time * 65535.0) as u16)`
/// - Synthwave: `phase = phase.wrapping_add((delta_time * 2.0 * 65535.0) as u16)`
pub(crate) fn env_lines(
    mut caller: Caller<'_, ZXGameContext>,
    layer: u32,
    variant: u32,
    line_type: u32,
    thickness: u32,
    spacing: f32,
    fade_distance: f32,
    parallax: u32,
    color_primary: u32,
    color_accent: u32,
    accent_every: u32,
    phase: u32,
    profile: u32,
    warp: u32,
    wobble: u32,
    glow: u32,
    axis_x: f32,
    axis_y: f32,
    axis_z: f32,
    seed: u32,
) {
    let state = &mut caller.data_mut().ffi;

    // Pack lines into specified layer
    state.current_environment_state.pack_lines(LinesConfig {
        offset: layer_offset_words(layer),
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
        axis: Vec3::new(axis_x, axis_y, axis_z),
        seed,
    });

    // Set mode for the specified layer
    if layer == 0 {
        state
            .current_environment_state
            .set_base_mode(env_mode::LINES);
    } else {
        state
            .current_environment_state
            .set_overlay_mode(env_mode::LINES);
    }

    state.environment_dirty = true;
}

/// Configure silhouette environment (Mode 3)
///
/// # Arguments
/// * `layer` — Target layer: 0 = base layer, 1 = overlay layer
/// * `jaggedness` — Terrain roughness (0-255, 0=smooth hills, 255=sharp peaks)
/// * `layer_count` — Number of depth layers (1-3)
/// * `color_near` — Nearest silhouette color (0xRRGGBBAA)
/// * `color_far` — Farthest silhouette color (0xRRGGBBAA)
/// * `sky_zenith` — Sky color at zenith behind silhouettes (0xRRGGBBAA)
/// * `sky_horizon` — Sky color at horizon behind silhouettes (0xRRGGBBAA)
/// * `parallax_rate` — Layer separation amount (0-255)
/// * `seed` — Noise seed for terrain shape
///
/// Creates layered terrain silhouettes with procedural noise.
/// Good for mountain ranges, city skylines, forest horizons.
///
/// **Use cases:**
/// - Mountains: jaggedness=200, layer_count=3, cool blue tones
/// - Cityscape: jaggedness=50, layer_count=2, dark grays
/// - Forest: jaggedness=100, layer_count=2, green-brown tones
pub(crate) fn env_silhouette(
    mut caller: Caller<'_, ZXGameContext>,
    layer: u32,
    family: u32,
    jaggedness: u32,
    layer_count: u32,
    color_near: u32,
    color_far: u32,
    sky_zenith: u32,
    sky_horizon: u32,
    parallax_rate: u32,
    seed: u32,
    phase: u32,
    fog: u32,
    wind: u32,
) {
    let state = &mut caller.data_mut().ffi;

    state
        .current_environment_state
        .pack_silhouette(SilhouetteConfig {
            offset: layer_offset_words(layer),
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
        });

    // Set mode for the specified layer
    if layer == 0 {
        state
            .current_environment_state
            .set_base_mode(env_mode::SILHOUETTE);
    } else {
        state
            .current_environment_state
            .set_overlay_mode(env_mode::SILHOUETTE);
    }

    state.environment_dirty = true;
}

/// Configure nebula environment (Mode 4).
///
/// Soft fields: fog/clouds/aurora/ink/plasma/kaleido.
pub(crate) fn env_nebula(
    mut caller: Caller<'_, ZXGameContext>,
    layer: u32,
    family: u32,
    coverage: u32,
    softness: u32,
    intensity: u32,
    scale: u32,
    detail: u32,
    warp: u32,
    flow: u32,
    parallax: u32,
    height_bias: u32,
    contrast: u32,
    color_a: u32,
    color_b: u32,
    axis_x: f32,
    axis_y: f32,
    axis_z: f32,
    phase: u32,
    seed: u32,
) {
    let state = &mut caller.data_mut().ffi;

    state.current_environment_state.pack_nebula(NebulaConfig {
        offset: layer_offset_words(layer),
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
        axis: Vec3::new(axis_x, axis_y, axis_z),
        phase,
        seed,
    });

    // Set mode for the specified layer
    if layer == 0 {
        state
            .current_environment_state
            .set_base_mode(env_mode::NEBULA);
    } else {
        state
            .current_environment_state
            .set_overlay_mode(env_mode::NEBULA);
    }

    state.environment_dirty = true;
}

/// Configure room environment (Mode 5)
///
/// # Arguments
/// * `layer` — Target layer: 0 = base layer, 1 = overlay layer
/// * `color_ceiling` — Ceiling color (0xRRGGBB00, alpha byte unused)
/// * `color_floor` — Floor color (0xRRGGBB00, alpha byte unused)
/// * `color_walls` — Wall color (0xRRGGBB00, alpha byte unused)
/// * `panel_size` — Size of wall panel pattern (world units)
/// * `panel_gap` — Gap between panels (0-255)
/// * `light_dir_x`, `light_dir_y`, `light_dir_z` — Light direction
/// * `light_intensity` — Directional light strength (0-255)
/// * `corner_darken` — Corner/edge darkening amount (0-255)
/// * `room_scale` — Room size multiplier
/// * `viewer_x`, `viewer_y`, `viewer_z` — Viewer position in room (-128 to 127 = -1.0 to 1.0)
///
/// Creates interior of a 3D box with directional lighting.
/// Viewer position affects which walls/ceiling/floor are visible.
///
/// **Position:** snorm8x3 where (0, 0, 0) = center of room:
/// ```ignore
/// let norm_x = (player.x / room_half_size).clamp(-1.0, 1.0);
/// env_room(layer, ..., (norm_x * 127.0) as i32, ...);
/// ```
///
/// **Use cases:** Hangars, corridors, dungeons, studios
/// **Note:** Can be safely layered with any other mode (uses no shared storage)
pub(crate) fn env_room(
    mut caller: Caller<'_, ZXGameContext>,
    layer: u32,
    color_ceiling: u32,
    color_floor: u32,
    color_walls: u32,
    panel_size: f32,
    panel_gap: u32,
    light_dir_x: f32,
    light_dir_y: f32,
    light_dir_z: f32,
    light_intensity: u32,
    light_tint: u32,
    corner_darken: u32,
    room_scale: f32,
    viewer_x: i32,
    viewer_y: i32,
    viewer_z: i32,
    accent: u32,
    accent_mode: u32,
    roughness: u32,
    phase: u32,
) {
    let state = &mut caller.data_mut().ffi;

    state.current_environment_state.pack_room(RoomConfig {
        offset: layer_offset_words(layer),
        color_ceiling,
        color_floor,
        color_walls,
        panel_size,
        panel_gap,
        light_direction: glam::Vec3::new(light_dir_x, light_dir_y, light_dir_z),
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
    });

    // Set mode for the specified layer
    if layer == 0 {
        state
            .current_environment_state
            .set_base_mode(env_mode::ROOM);
    } else {
        state
            .current_environment_state
            .set_overlay_mode(env_mode::ROOM);
    }

    state.environment_dirty = true;
}

/// Configure veil environment (Mode 6).
///
/// Axis-aligned SDF ribbons/pillars with bounded depth slices.
pub(crate) fn env_veil(
    mut caller: Caller<'_, ZXGameContext>,
    layer: u32,
    family: u32,
    density: u32,
    width: u32,
    taper: u32,
    curvature: u32,
    edge_soft: u32,
    height_min: u32,
    height_max: u32,
    color_near: u32,
    color_far: u32,
    glow: u32,
    parallax: u32,
    axis_x: f32,
    axis_y: f32,
    axis_z: f32,
    phase: u32,
    seed: u32,
) {
    let state = &mut caller.data_mut().ffi;

    state.current_environment_state.pack_veil(VeilConfig {
        offset: layer_offset_words(layer),
        family,
        density,
        height_min,
        height_max,
        width,
        taper,
        curvature,
        edge_soft,
        color_near,
        color_far,
        glow,
        parallax,
        axis: Vec3::new(axis_x, axis_y, axis_z),
        phase,
        seed,
    });

    // Set mode for the specified layer
    if layer == 0 {
        state
            .current_environment_state
            .set_base_mode(env_mode::VEIL);
    } else {
        state
            .current_environment_state
            .set_overlay_mode(env_mode::VEIL);
    }

    state.environment_dirty = true;
}

/// Configure rings environment (Mode 7)
///
/// # Arguments
/// * `layer` — Target layer: 0 = base layer, 1 = overlay layer
/// * `ring_count` — Number of rings (1-255)
/// * `thickness` — Ring thickness (0-255)
/// * `color_a` — First alternating color (0xRRGGBBAA)
/// * `color_b` — Second alternating color (0xRRGGBBAA)
/// * `center_color` — Bright center color (0xRRGGBBAA)
/// * `center_falloff` — Center glow falloff (0-255)
/// * `spiral_twist` — Spiral rotation in degrees (0=concentric)
/// * `axis_x`, `axis_y`, `axis_z` — Ring axis direction (normalized)
/// * `phase` — Rotation phase (0-65535 = 0°-360°, wraps for seamless)
///
/// Creates concentric rings for portals, tunnels, or vortex effects.
///
/// **Animation:** Increment `phase` for spinning:
/// - Portal spin: `phase = phase.wrapping_add((delta_time * 2.0 * 65535.0) as u16)`
/// - Hypnotic: `phase = phase.wrapping_add((delta_time * 5.0 * 65535.0) as u16)`
pub(crate) fn env_rings(
    mut caller: Caller<'_, ZXGameContext>,
    layer: u32,
    family: u32,
    ring_count: u32,
    thickness: u32,
    color_a: u32,
    color_b: u32,
    center_color: u32,
    center_falloff: u32,
    spiral_twist: f32,
    axis_x: f32,
    axis_y: f32,
    axis_z: f32,
    phase: u32,
    wobble: u32,
    noise: u32,
    dash: u32,
    glow: u32,
    seed: u32,
) {
    let state = &mut caller.data_mut().ffi;

    // Pack rings into specified layer
    state.current_environment_state.pack_rings(RingsConfig {
        offset: layer_offset_words(layer),
        family,
        ring_count,
        thickness,
        color_a,
        color_b,
        center_color,
        center_falloff,
        spiral_twist,
        axis: Vec3::new(axis_x, axis_y, axis_z),
        phase,
        wobble,
        noise,
        dash,
        glow,
        seed,
    });

    // Set mode for the specified layer
    if layer == 0 {
        state
            .current_environment_state
            .set_base_mode(env_mode::RINGS);
    } else {
        state
            .current_environment_state
            .set_overlay_mode(env_mode::RINGS);
    }

    state.environment_dirty = true;
}
