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
    linker.func_wrap("env", "draw_env", draw_env)?;
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

    // Get or create shading state index for current environment configuration
    // This ensures the environment data is uploaded to GPU
    let shading_idx = state.add_shading_state();

    // Add sky/environment draw command to render pass
    state
        .render_pass
        .add_command(crate::graphics::VRPCommand::Sky {
            shading_state_index: shading_idx.0,
            depth_test: false, // Environment always behind geometry
        });
}
