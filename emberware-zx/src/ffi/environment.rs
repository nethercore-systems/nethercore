//! Environment system FFI functions (Multi-Environment v3)
//!
//! Functions for setting procedural environment rendering parameters.
//! Environments support 8 modes with layering (base + overlay) and blend modes.

use anyhow::Result;
use tracing::warn;
use wasmtime::{Caller, Linker};

use crate::graphics::env_mode;

use super::ZXGameContext;

/// Register environment system FFI functions
pub fn register(linker: &mut Linker<ZXGameContext>) -> Result<()> {
    linker.func_wrap("env", "env_gradient_set", env_gradient_set)?;
    linker.func_wrap("env", "env_scatter_set", env_scatter_set)?;
    linker.func_wrap("env", "env_lines_set", env_lines_set)?;
    linker.func_wrap("env", "env_silhouette_set", env_silhouette_set)?;
    linker.func_wrap("env", "env_rectangles_set", env_rectangles_set)?;
    linker.func_wrap("env", "env_room_set", env_room_set)?;
    linker.func_wrap("env", "env_curtains_set", env_curtains_set)?;
    linker.func_wrap("env", "env_rings_set", env_rings_set)?;
    linker.func_wrap("env", "env_select_pair", env_select_pair)?;
    linker.func_wrap("env", "env_blend_mode", env_blend_mode)?;
    Ok(())
}

/// Set gradient parameters for Mode 0 (Gradient)
///
/// # Arguments
/// * `zenith` — Color directly overhead (0xRRGGBBAA)
/// * `sky_horizon` — Sky color at horizon level (0xRRGGBBAA)
/// * `ground_horizon` — Ground color at horizon level (0xRRGGBBAA)
/// * `nadir` — Color directly below (0xRRGGBBAA)
/// * `rotation` — Rotation around Y axis in radians (for non-symmetric gradients)
/// * `shift` — Horizon vertical shift (-1.0 to 1.0, 0.0 = equator)
///
/// Sets the 4-color gradient for environment rendering. The gradient interpolates:
/// - zenith → sky_horizon (Y > 0)
/// - sky_horizon → ground_horizon (at Y = 0 + shift)
/// - ground_horizon → nadir (Y < 0)
///
/// This affects the BASE layer gradient. To set overlay gradient, call env_select_pair
/// first to configure overlay mode, then call this function again.
///
/// **Examples:**
/// - Blue sky: `env_gradient_set(0x191970FF, 0x87CEEBFF, 0x228B22FF, 0x2F4F4FFF, 0.0, 0.0)`
/// - Sunset: `env_gradient_set(0x4A00E0FF, 0xFF6B6BFF, 0x8B4513FF, 0x2F2F2FFF, 0.0, 0.1)`
fn env_gradient_set(
    mut caller: Caller<'_, ZXGameContext>,
    zenith: u32,
    sky_horizon: u32,
    ground_horizon: u32,
    nadir: u32,
    rotation: f32,
    shift: f32,
) {
    let state = &mut caller.data_mut().ffi;

    // Pack gradient into current environment state's base layer (offset 0)
    state.current_environment_state.pack_gradient(
        0, // Base layer offset
        zenith,
        sky_horizon,
        ground_horizon,
        nadir,
        rotation,
        shift,
    );

    // Ensure base mode is set to Gradient
    state
        .current_environment_state
        .set_base_mode(env_mode::GRADIENT);

    state.environment_dirty = true;
}

/// Set scatter parameters for Mode 1 (Scatter)
///
/// # Arguments
/// * `variant` — Scatter type: 0=Stars, 1=Vertical (rain), 2=Horizontal, 3=Warp
/// * `density` — Particle count (0-255)
/// * `size` — Particle size (0-255)
/// * `glow` — Glow/bloom intensity (0-255)
/// * `streak_length` — Elongation for streaks (0-63, 0=points)
/// * `color_primary` — Main particle color (0xRRGGBB00)
/// * `color_secondary` — Variation/twinkle color (0xRRGGBB00)
/// * `parallax_rate` — Layer separation amount (0-255, 0=flat)
/// * `parallax_size` — Size variation with depth (0-255)
/// * `phase` — Animation phase (0-65535, wraps for seamless looping)
///
/// Creates a procedural particle field. Variants:
/// - **Stars (0):** Static twinkling points, good for night skies
/// - **Vertical (1):** Rain/snow streaks falling downward
/// - **Horizontal (2):** Speed lines, good for motion blur effects
/// - **Warp (3):** Radial expansion from center (hyperspace effect)
///
/// **Animation:** Increment `phase` each frame for movement:
/// - Rain: `phase = phase.wrapping_add((delta_time * speed * 65535.0) as u16)`
/// - Twinkle: `phase = phase.wrapping_add((delta_time * 0.1 * 65535.0) as u16)`
fn env_scatter_set(
    mut caller: Caller<'_, ZXGameContext>,
    variant: u32,
    density: u32,
    size: u32,
    glow: u32,
    streak_length: u32,
    color_primary: u32,
    color_secondary: u32,
    parallax_rate: u32,
    parallax_size: u32,
    phase: u32,
) {
    let state = &mut caller.data_mut().ffi;

    // Pack scatter into current environment state's base layer (offset 0)
    // Layer count defaults to 1 for now
    state.current_environment_state.pack_scatter(
        0, // Base layer offset
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
        1, // layer_count default
    );

    // Ensure base mode is set to Scatter
    state
        .current_environment_state
        .set_base_mode(env_mode::SCATTER);

    state.environment_dirty = true;
}

/// Set lines parameters for Mode 2 (Lines)
///
/// # Arguments
/// * `variant` — Surface type: 0=Floor, 1=Ceiling, 2=Sphere
/// * `line_type` — Line pattern: 0=Horizontal, 1=Vertical, 2=Grid
/// * `thickness` — Line thickness (0-255)
/// * `spacing` — Distance between lines (world units)
/// * `fade_distance` — Distance where lines start fading (world units)
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
fn env_lines_set(
    mut caller: Caller<'_, ZXGameContext>,
    variant: u32,
    line_type: u32,
    thickness: u32,
    spacing: f32,
    fade_distance: f32,
    color_primary: u32,
    color_accent: u32,
    accent_every: u32,
    phase: u32,
) {
    let state = &mut caller.data_mut().ffi;

    // Pack lines into current environment state's base layer (offset 0)
    state.current_environment_state.pack_lines(
        0, // Base layer offset
        variant,
        line_type,
        thickness,
        spacing,
        fade_distance,
        color_primary,
        color_accent,
        accent_every,
        phase,
    );

    // Ensure base mode is set to Lines
    state
        .current_environment_state
        .set_base_mode(env_mode::LINES);

    state.environment_dirty = true;
}

/// Set silhouette parameters for Mode 3 (Silhouette)
///
/// # Arguments
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
fn env_silhouette_set(
    mut caller: Caller<'_, ZXGameContext>,
    jaggedness: u32,
    layer_count: u32,
    color_near: u32,
    color_far: u32,
    sky_zenith: u32,
    sky_horizon: u32,
    parallax_rate: u32,
    seed: u32,
) {
    let state = &mut caller.data_mut().ffi;

    state.current_environment_state.pack_silhouette(
        0, // Base layer offset
        jaggedness,
        layer_count,
        color_near,
        color_far,
        sky_zenith,
        sky_horizon,
        parallax_rate,
        seed,
    );

    state
        .current_environment_state
        .set_base_mode(env_mode::SILHOUETTE);

    state.environment_dirty = true;
}

/// Set rectangles parameters for Mode 4 (Rectangles)
///
/// # Arguments
/// * `variant` — Pattern type: 0=Scatter, 1=Buildings, 2=Bands, 3=Panels
/// * `density` — How many rectangles (0-255)
/// * `lit_ratio` — Percentage of rectangles lit (0-255, 128=50%)
/// * `size_min` — Minimum rectangle size (0-63)
/// * `size_max` — Maximum rectangle size (0-63)
/// * `aspect` — Aspect ratio bias (0-3, 0=square, 3=very tall)
/// * `color_primary` — Main window/panel color (0xRRGGBBAA)
/// * `color_variation` — Color variation for variety (0xRRGGBBAA)
/// * `parallax_rate` — Layer separation (0-255, for scatter variant)
/// * `phase` — Flicker phase (0-65535, wraps for seamless animation)
///
/// Creates rectangular light sources like windows, screens, or panels.
///
/// **Variants:**
/// - Scatter (0): Random scattered rectangles
/// - Buildings (1): Organized like building windows
/// - Bands (2): Horizontal bands of rectangles
/// - Panels (3): Grid-like control panel layout
///
/// **Animation:** Increment `phase` for window flicker:
/// - Slow flicker: `phase = phase.wrapping_add((delta_time * 0.5 * 65535.0) as u16)`
fn env_rectangles_set(
    mut caller: Caller<'_, ZXGameContext>,
    variant: u32,
    density: u32,
    lit_ratio: u32,
    size_min: u32,
    size_max: u32,
    aspect: u32,
    color_primary: u32,
    color_variation: u32,
    parallax_rate: u32,
    phase: u32,
) {
    let state = &mut caller.data_mut().ffi;

    state.current_environment_state.pack_rectangles(
        0, // Base layer offset
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
    );

    state
        .current_environment_state
        .set_base_mode(env_mode::RECTANGLES);

    state.environment_dirty = true;
}

/// Set room parameters for Mode 5 (Room)
///
/// # Arguments
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
/// ```
/// let norm_x = (player.x / room_half_size).clamp(-1.0, 1.0);
/// env_room_set(..., (norm_x * 127.0) as i32, ...);
/// ```
///
/// **Use cases:** Hangars, corridors, dungeons, studios
/// **Note:** Can be safely layered with any other mode (uses no shared storage)
fn env_room_set(
    mut caller: Caller<'_, ZXGameContext>,
    color_ceiling: u32,
    color_floor: u32,
    color_walls: u32,
    panel_size: f32,
    panel_gap: u32,
    light_dir_x: f32,
    light_dir_y: f32,
    light_dir_z: f32,
    light_intensity: u32,
    corner_darken: u32,
    room_scale: f32,
    viewer_x: i32,
    viewer_y: i32,
    viewer_z: i32,
) {
    let state = &mut caller.data_mut().ffi;

    state.current_environment_state.pack_room(
        0, // Base layer offset
        color_ceiling,
        color_floor,
        color_walls,
        panel_size,
        panel_gap,
        glam::Vec3::new(light_dir_x, light_dir_y, light_dir_z),
        light_intensity,
        corner_darken,
        room_scale,
        viewer_x,
        viewer_y,
        viewer_z,
    );

    state
        .current_environment_state
        .set_base_mode(env_mode::ROOM);

    state.environment_dirty = true;
}

/// Set curtains parameters for Mode 6 (Curtains)
///
/// # Arguments
/// * `layer_count` — Depth layers (1-3)
/// * `density` — Structures per cell (0-255)
/// * `height_min` — Minimum height (0-63)
/// * `height_max` — Maximum height (0-63)
/// * `width` — Structure width (0-31)
/// * `spacing` — Gap between structures (0-31)
/// * `waviness` — Organic wobble (0-255, 0=straight)
/// * `color_near` — Nearest structure color (0xRRGGBBAA)
/// * `color_far` — Farthest structure color (0xRRGGBBAA)
/// * `glow` — Neon/magical glow intensity (0-255)
/// * `parallax_rate` — Layer separation (0-255)
/// * `phase` — Horizontal scroll phase (0-65535, wraps for seamless)
///
/// Creates vertical structures (pillars, trees) arranged around the viewer.
///
/// **Use cases:**
/// - Forest: waviness=50, brown/green tones
/// - Pillars: waviness=0, white/gray, evenly spaced
/// - Neon: waviness=30, bright colors, high glow
///
/// **Animation:** Increment `phase` for side-scrolling parallax:
/// - Running: `phase = phase.wrapping_add((velocity.x * delta_time * 65535.0) as u16)`
fn env_curtains_set(
    mut caller: Caller<'_, ZXGameContext>,
    layer_count: u32,
    density: u32,
    height_min: u32,
    height_max: u32,
    width: u32,
    spacing: u32,
    waviness: u32,
    color_near: u32,
    color_far: u32,
    glow: u32,
    parallax_rate: u32,
    phase: u32,
) {
    let state = &mut caller.data_mut().ffi;

    state.current_environment_state.pack_curtains(
        0, // Base layer offset
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
    );

    state
        .current_environment_state
        .set_base_mode(env_mode::CURTAINS);

    state.environment_dirty = true;
}

/// Set rings parameters for Mode 7 (Rings)
///
/// # Arguments
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
fn env_rings_set(
    mut caller: Caller<'_, ZXGameContext>,
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
) {
    let state = &mut caller.data_mut().ffi;

    // Pack rings into current environment state's base layer (offset 0)
    state.current_environment_state.pack_rings(
        0, // Base layer offset
        ring_count,
        thickness,
        color_a,
        color_b,
        center_color,
        center_falloff,
        spiral_twist,
        glam::Vec3::new(axis_x, axis_y, axis_z),
        phase,
    );

    // Ensure base mode is set to Rings
    state
        .current_environment_state
        .set_base_mode(env_mode::RINGS);

    state.environment_dirty = true;
}

/// Select base and overlay modes for environment layering
///
/// # Arguments
/// * `base_mode` — Base layer mode (0-7)
/// * `overlay_mode` — Overlay layer mode (0-7)
///
/// # Mode Values
/// - 0: Gradient — 4-color procedural sky/ground gradient
/// - 1: Scatter — Random particles (stars, rain, hyperspace) [Phase 2]
/// - 2: Lines — Procedural line patterns (synthwave grid) [Phase 3]
/// - 3: Silhouette — Layered silhouettes (mountains, cityscape) [Phase 5]
/// - 4: Rectangles — Random rectangles (city windows) [Phase 7]
/// - 5: Room — Interior box environment [Phase 6]
/// - 6: Curtains — Vertical strips (forest, pillars) [Phase 8]
/// - 7: Rings — Concentric rings (portals, tunnels) [Phase 4]
///
/// When base_mode == overlay_mode, only the base layer is rendered (no layering).
/// Use env_blend_mode() to control how layers combine.
///
/// **Note:** Phase 1 only supports Mode 0 (Gradient). Other modes will display
/// the default gradient until implemented.
fn env_select_pair(
    mut caller: Caller<'_, ZXGameContext>,
    base_mode: u32,
    overlay_mode: u32,
) {
    let state = &mut caller.data_mut().ffi;

    // Validate modes
    if base_mode > 7 {
        warn!(
            "env_select_pair: invalid base_mode {} (must be 0-7), clamping",
            base_mode
        );
    }
    if overlay_mode > 7 {
        warn!(
            "env_select_pair: invalid overlay_mode {} (must be 0-7), clamping",
            overlay_mode
        );
    }

    state
        .current_environment_state
        .set_base_mode(base_mode.min(7));
    state
        .current_environment_state
        .set_overlay_mode(overlay_mode.min(7));

    state.environment_dirty = true;
}

/// Set the blend mode for combining base and overlay layers
///
/// # Arguments
/// * `mode` — Blend mode (0-3)
///
/// # Blend Modes
/// - 0: Alpha — Standard alpha blending: lerp(base, overlay, overlay.a)
/// - 1: Add — Additive blending: base + overlay
/// - 2: Multiply — Multiplicative: base * overlay
/// - 3: Screen — Screen blending: 1 - (1-base) * (1-overlay)
///
/// Only applies when base_mode != overlay_mode (layering enabled).
///
/// **Examples:**
/// - Alpha (default): Overlay covers base based on alpha
/// - Add: Bright overlays add to base (good for glow effects)
/// - Multiply: Dark overlays darken base (good for vignettes)
/// - Screen: Light overlays brighten base (good for fog/haze)
fn env_blend_mode(mut caller: Caller<'_, ZXGameContext>, mode: u32) {
    let state = &mut caller.data_mut().ffi;

    // Validate mode
    if mode > 3 {
        warn!(
            "env_blend_mode: invalid mode {} (must be 0-3), clamping",
            mode
        );
    }

    state
        .current_environment_state
        .set_blend_mode(mode.min(3));

    state.environment_dirty = true;
}
