//! Environment drawing and utility functions
//!
//! This module contains functions for rendering configured environments,
//! setting blend modes, and binding matcap textures.

use tracing::warn;
use wasmtime::Caller;

use crate::ffi::ZXGameContext;

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
pub(crate) fn env_blend(mut caller: Caller<'_, ZXGameContext>, mode: u32) {
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
pub(crate) fn matcap_set(mut caller: Caller<'_, ZXGameContext>, slot: u32, texture: u32) {
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
///     env_gradient(0, 0x191970FF, 0x87CEEBFF, 0x228B22FF, 0x2F4F4FFF, 0.0, 0.0, 0.0, 0, 0, 0, 0, 0, 0, 0);
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
pub(crate) fn draw_env(mut caller: Caller<'_, ZXGameContext>) {
    let state = &mut caller.data_mut().ffi;

    // Capture current viewport for split-screen rendering
    let viewport = state.current_viewport;

    // Capture current pass_id for render pass ordering
    let pass_id = state.current_pass_id;

    // Get or create shading state index for current environment configuration
    // This ensures the environment data is uploaded to GPU
    let shading_idx = state.add_shading_state();

    // Add environment draw command to render pass
    state
        .render_pass
        .add_command(crate::graphics::VRPCommand::Environment {
            shading_state_index: shading_idx.0,
            viewport,
            pass_id,
        });
}
