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
    linker.func_wrap("env", "env_gradient", env_gradient)?;
    linker.func_wrap("env", "env_scatter", env_scatter)?;
    linker.func_wrap("env", "env_lines", env_lines)?;
    linker.func_wrap("env", "env_silhouette", env_silhouette)?;
    linker.func_wrap("env", "env_rectangles", env_rectangles)?;
    linker.func_wrap("env", "env_room", env_room)?;
    linker.func_wrap("env", "env_curtains", env_curtains)?;
    linker.func_wrap("env", "env_rings", env_rings)?;
    linker.func_wrap("env", "env_blend", env_blend)?;
    linker.func_wrap("env", "matcap_set", matcap_set)?;
    linker.func_wrap("env", "draw_env", draw_env)?;
    Ok(())
}

/// Configure gradient environment (Mode 0)
///
/// # Arguments
/// * `layer` — Target layer: 0 = base layer, 1 = overlay layer
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
/// You can configure the same mode on both layers with different parameters:
/// ```ignore
/// env_gradient(0, ...);  // Base layer
/// env_gradient(1, ...);  // Overlay layer
/// ```
///
/// **Examples:**
/// - Blue sky: `env_gradient(0, 0x191970FF, 0x87CEEBFF, 0x228B22FF, 0x2F4F4FFF, 0.0, 0.0)`
/// - Sunset: `env_gradient(0, 0x4A00E0FF, 0xFF6B6BFF, 0x8B4513FF, 0x2F2F2FFF, 0.0, 0.1)`
fn env_gradient(
    mut caller: Caller<'_, ZXGameContext>,
    layer: u32,
    zenith: u32,
    sky_horizon: u32,
    ground_horizon: u32,
    nadir: u32,
    rotation: f32,
    shift: f32,
) {
    let state = &mut caller.data_mut().ffi;

    // Pack gradient into specified layer (0 = base, 1 = overlay)
    state.current_environment_state.pack_gradient(
        layer as usize,
        zenith,
        sky_horizon,
        ground_horizon,
        nadir,
        rotation,
        shift,
    );

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

/// Configure scatter environment (Mode 1)
///
/// # Arguments
/// * `layer` — Target layer: 0 = base layer, 1 = overlay layer
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
///
/// **Example same-mode layering:**
/// ```ignore
/// env_scatter(0, 0, 128, 3, 200, 0, 0xFFFFFF00, 0xCCCCFF00, 50, 100, 0);  // Stars
/// env_scatter(1, 1, 64, 2, 128, 30, 0x8888FF00, 0x4444FF00, 80, 50, 0);  // Rain overlay
/// ```
fn env_scatter(
    mut caller: Caller<'_, ZXGameContext>,
    layer: u32,
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

    // Pack scatter into specified layer
    state.current_environment_state.pack_scatter(
        layer as usize,
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

    // Set mode for the specified layer
    if layer == 0 {
        state
            .current_environment_state
            .set_base_mode(env_mode::SCATTER);
    } else {
        state
            .current_environment_state
            .set_overlay_mode(env_mode::SCATTER);
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
fn env_lines(
    mut caller: Caller<'_, ZXGameContext>,
    layer: u32,
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

    // Pack lines into specified layer
    state.current_environment_state.pack_lines(
        layer as usize,
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
fn env_silhouette(
    mut caller: Caller<'_, ZXGameContext>,
    layer: u32,
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
        layer as usize,
        jaggedness,
        layer_count,
        color_near,
        color_far,
        sky_zenith,
        sky_horizon,
        parallax_rate,
        seed,
    );

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

/// Configure rectangles environment (Mode 4)
///
/// # Arguments
/// * `layer` — Target layer: 0 = base layer, 1 = overlay layer
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
fn env_rectangles(
    mut caller: Caller<'_, ZXGameContext>,
    layer: u32,
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
        layer as usize,
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

    // Set mode for the specified layer
    if layer == 0 {
        state
            .current_environment_state
            .set_base_mode(env_mode::RECTANGLES);
    } else {
        state
            .current_environment_state
            .set_overlay_mode(env_mode::RECTANGLES);
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
fn env_room(
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
    corner_darken: u32,
    room_scale: f32,
    viewer_x: i32,
    viewer_y: i32,
    viewer_z: i32,
) {
    let state = &mut caller.data_mut().ffi;

    state.current_environment_state.pack_room(
        layer as usize,
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

/// Configure curtains environment (Mode 6)
///
/// # Arguments
/// * `layer` — Target layer: 0 = base layer, 1 = overlay layer
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
fn env_curtains(
    mut caller: Caller<'_, ZXGameContext>,
    layer: u32,
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
        layer as usize,
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

    // Set mode for the specified layer
    if layer == 0 {
        state
            .current_environment_state
            .set_base_mode(env_mode::CURTAINS);
    } else {
        state
            .current_environment_state
            .set_overlay_mode(env_mode::CURTAINS);
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
fn env_rings(
    mut caller: Caller<'_, ZXGameContext>,
    layer: u32,
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

    // Pack rings into specified layer
    state.current_environment_state.pack_rings(
        layer as usize,
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
/// Controls how the overlay layer composites onto the base layer.
///
/// **Examples:**
/// - Alpha (default): Overlay covers base based on alpha
/// - Add: Bright overlays add to base (good for glow effects)
/// - Multiply: Dark overlays darken base (good for vignettes)
/// - Screen: Light overlays brighten base (good for fog/haze)
fn env_blend(mut caller: Caller<'_, ZXGameContext>, mode: u32) {
    let state = &mut caller.data_mut().ffi;

    // Validate mode
    if mode > 3 {
        warn!("env_blend: invalid mode {} (must be 0-3), clamping", mode);
    }

    state.current_environment_state.set_blend_mode(mode.min(3));

    state.environment_dirty = true;
}

/// Bind a matcap texture to a slot (Mode 1 only)
///
/// # Arguments
/// * `slot` — Matcap slot (1-3)
/// * `texture` — Texture handle from load_texture
///
/// In Mode 1 (Matcap), slots 1-3 are used for matcap textures that multiply together.
/// Slot 0 is reserved for albedo texture.
/// Using this function in other modes is allowed but has no effect.
fn matcap_set(mut caller: Caller<'_, ZXGameContext>, slot: u32, texture: u32) {
    // Validate slot range (1-3 for matcaps)
    if !(1..=3).contains(&slot) {
        warn!("matcap_set: invalid slot {} (must be 1-3)", slot);
        return;
    }

    let state = &mut caller.data_mut().ffi;
    state.bound_textures[slot as usize] = texture;
}

/// Render the configured environment
///
/// Renders the procedural environment using the current configuration.
/// Always renders at far plane (depth=1.0) so geometry appears in front.
///
/// # Usage
/// Call this **first** in your `render()` function, before any 3D geometry:
/// ```rust,ignore
/// fn render() {
///     // Configure environment (e.g., gradient on base layer)
///     env_gradient(0, 0x191970FF, 0x87CEEBFF, 0x228B22FF, 0x2F4F4FFF, 0.0, 0.0);
///
///     // Draw environment first (before geometry)
///     draw_env();
///
///     // Then draw scene geometry
///     draw_mesh(terrain);
///     draw_mesh(player);
/// }
/// ```
///
/// # Notes
/// - Works in all render modes (0-3)
/// - Environment always renders behind all geometry
/// - Depth test is disabled for environment rendering
fn draw_env(mut caller: Caller<'_, ZXGameContext>) {
    let state = &mut caller.data_mut().ffi;

    // Capture current viewport for split-screen rendering
    let viewport = state.current_viewport;

    // Capture current pass_id for render pass ordering
    let pass_id = state.current_pass_id;

    // Get or create shading state index for current environment configuration
    // This ensures the environment data is uploaded to GPU
    let shading_idx = state.add_shading_state();

    // Add sky/environment draw command to render pass
    state
        .render_pass
        .add_command(crate::graphics::VRPCommand::Sky {
            shading_state_index: shading_idx.0,
            viewport,
            pass_id,
        });
}
