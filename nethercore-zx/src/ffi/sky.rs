//! Sky system FFI functions
//!
//! Functions for setting procedural sky gradient. Sun-based lighting was removed
//! in Multi-Environment v3 - use light_set() for directional lighting instead.

use anyhow::Result;
use tracing::warn;
use wasmtime::{Caller, Linker};

use super::ZXGameContext;

/// Register sky system FFI functions
pub fn register(linker: &mut Linker<ZXGameContext>) -> Result<()> {
    linker.func_wrap("env", "sky_set_colors", sky_set_colors)?;
    linker.func_wrap("env", "matcap_set", matcap_set)?;
    linker.func_wrap("env", "draw_sky", draw_sky)?;
    Ok(())
}

/// Set sky gradient colors
///
/// # Arguments
/// * `horizon_color` — Horizon color (0xRRGGBBAA)
/// * `zenith_color` — Zenith color (0xRRGGBBAA)
///
/// Sets the procedural sky gradient. Horizon is the color at eye level,
/// zenith is the color directly overhead. The gradient interpolates smoothly between them.
/// Works in all render modes (provides ambient lighting in PBR/Hybrid modes).
///
/// **Examples:**
/// - `sky_set_colors(0x87CEEBFF, 0x191970FF)` — Light blue horizon, midnight blue zenith
/// - `sky_set_colors(0xFF6B6BFF, 0x4A00E0FF)` — Sunset gradient (red to purple)
fn sky_set_colors(mut caller: Caller<'_, ZXGameContext>, horizon_color: u32, zenith_color: u32) {
    let state = &mut caller.data_mut().ffi;
    state.update_sky_colors(horizon_color, zenith_color);
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

/// Draw the procedural sky
///
/// Renders a fullscreen gradient from horizon to zenith color with sun disc.
/// Uses current sky configuration set via `sky_set_colors()` and `sky_set_sun()`.
/// Always renders at far plane (depth=1.0) so geometry appears in front.
///
/// # Usage
/// Call this **first** in your `render()` function, before any 3D geometry:
/// ```rust,ignore
/// fn render() {
///     // Configure sky colors and sun
///     sky_set_colors(0xB2D8F2FF, 0x3366B2FF);  // Light blue → darker blue
///     sky_set_sun(-0.5, -0.707, -0.5, 0xFFF2E6FF, 0.98);  // Warm white sun (rays going down)
///
///     // Draw sky first (before geometry)
///     draw_sky();
///
///     // Then draw scene geometry
///     draw_mesh(terrain);
///     draw_mesh(player);
/// }
/// ```
///
/// # Notes
/// - Works in all render modes (0-3)
/// - Sky always renders behind all geometry
/// - Depth test is disabled for sky rendering
fn draw_sky(mut caller: Caller<'_, ZXGameContext>) {
    let state = &mut caller.data_mut().ffi;

    // Get or create shading state index for current sky configuration
    // This ensures the sky data is uploaded to GPU
    let shading_idx = state.add_shading_state();

    // Add sky draw command to render pass
    state
        .render_pass
        .add_command(crate::graphics::VRPCommand::Sky {
            shading_state_index: shading_idx.0,
            depth_test: false, // Sky always behind geometry
        });
}
