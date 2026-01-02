//! Render state FFI functions
//!
//! Functions for setting render state like color, depth testing, culling, and filtering.

use anyhow::Result;
use tracing::warn;
use wasmtime::{Caller, Linker};

use super::ZXGameContext;
use crate::graphics::{CullMode, StencilMode, TextureFilter};

/// Register render state FFI functions
pub fn register(linker: &mut Linker<ZXGameContext>) -> Result<()> {
    linker.func_wrap("env", "set_color", set_color)?;
    linker.func_wrap("env", "depth_test", depth_test)?;
    linker.func_wrap("env", "cull_mode", cull_mode)?;
    linker.func_wrap("env", "texture_filter", texture_filter)?;
    linker.func_wrap("env", "uniform_alpha", uniform_alpha)?;
    linker.func_wrap("env", "dither_offset", dither_offset)?;
    linker.func_wrap("env", "layer", layer)?;
    // Stencil functions for masked rendering (Tier 2)
    linker.func_wrap("env", "stencil_begin", stencil_begin)?;
    linker.func_wrap("env", "stencil_end", stencil_end)?;
    linker.func_wrap("env", "stencil_clear", stencil_clear)?;
    linker.func_wrap("env", "stencil_invert", stencil_invert)?;
    Ok(())
}

/// Set the uniform tint color
///
/// # Arguments
/// * `color` — Color in 0xRRGGBBAA format
///
/// This color is multiplied with vertex colors and textures.
fn set_color(mut caller: Caller<'_, ZXGameContext>, color: u32) {
    let state = &mut caller.data_mut().ffi;
    state.update_color(color);
}

/// Enable or disable depth testing
///
/// # Arguments
/// * `enabled` — 0 to disable, non-zero to enable
///
/// Default: enabled. Disable for 2D overlays or special effects.
fn depth_test(mut caller: Caller<'_, ZXGameContext>, enabled: u32) {
    let state = &mut caller.data_mut().ffi;
    state.depth_test = enabled != 0;
}

/// Set the face culling mode
///
/// # Arguments
/// * `mode` — 0=none (default, draw both sides), 1=back, 2=front
///
/// Default is none (no culling). Use back-face culling for solid 3D objects for performance.
fn cull_mode(mut caller: Caller<'_, ZXGameContext>, mode: u32) {
    let state = &mut caller.data_mut().ffi;

    if mode > 2 {
        warn!("cull_mode({}) invalid - must be 0-2, using 0 (none)", mode);
        state.cull_mode = CullMode::None;
        return;
    }

    state.cull_mode = CullMode::from_u32(mode);
}

/// Set the texture filtering mode
///
/// # Arguments
/// * `filter` — 0=nearest (pixelated, retro), 1=linear (smooth)
///
/// Default: nearest for retro aesthetic.
/// Note: Filter mode is stored in PackedUnifiedShadingState.flags for per-draw shader selection.
fn texture_filter(mut caller: Caller<'_, ZXGameContext>, filter: u32) {
    let state = &mut caller.data_mut().ffi;

    if filter > 1 {
        warn!(
            "texture_filter({}) invalid - must be 0-1, using 0 (nearest)",
            filter
        );
        state.texture_filter = TextureFilter::Nearest;
        state.update_texture_filter(false);
        return;
    }

    state.texture_filter = TextureFilter::from_u32(filter);
    state.update_texture_filter(filter == 1);
}

/// Set uniform alpha level for dither transparency
///
/// # Arguments
/// * `level` — 0-15 (0=fully transparent, 15=fully opaque, default=15)
///
/// This controls the dither pattern threshold for screen-door transparency.
/// The dither pattern is always active, but with level=15 (default) all fragments pass.
fn uniform_alpha(mut caller: Caller<'_, ZXGameContext>, level: u32) {
    let state = &mut caller.data_mut().ffi;

    if level > 15 {
        warn!(
            "uniform_alpha({}) invalid - must be 0-15, clamping to 15",
            level
        );
    }

    state.update_uniform_alpha(level.min(15) as u8);
}

/// Set dither offset for dither transparency
///
/// # Arguments
/// * `x` — 0-3 pixel shift in X axis
/// * `y` — 0-3 pixel shift in Y axis
///
/// Use different offsets for stacked dithered meshes to prevent pattern cancellation.
/// When two transparent objects overlap with the same alpha level and offset, their
/// dither patterns align and pixels cancel out. Different offsets shift the pattern
/// so both objects remain visible.
fn dither_offset(mut caller: Caller<'_, ZXGameContext>, x: u32, y: u32) {
    let state = &mut caller.data_mut().ffi;

    if x > 3 || y > 3 {
        warn!(
            "dither_offset({}, {}) invalid - values must be 0-3, clamping",
            x, y
        );
    }

    state.update_dither_offset(x.min(3) as u8, y.min(3) as u8);
}

/// Set the draw layer for 2D ordering control
///
/// # Arguments
/// * `n` — Layer value (0 = back, higher values = front)
///
/// Higher layer values are drawn on top of lower values.
/// Use this to ensure UI elements appear over game content
/// regardless of texture bindings or draw order.
///
/// Default: 0 (resets each frame)
fn layer(mut caller: Caller<'_, ZXGameContext>, n: u32) {
    let state = &mut caller.data_mut().ffi;
    state.current_layer = n;
}

// ============================================================================
// Stencil Functions (Tier 2 - Masked Rendering)
// ============================================================================

/// Begin writing to the stencil buffer (mask creation mode).
///
/// After calling this, subsequent draw calls will write to the stencil buffer
/// but NOT to the color buffer. Use this to create a mask shape.
///
/// # Example (circular scope mask)
/// ```rust,ignore
/// stencil_begin();           // Start mask creation
/// draw_mesh(circle_mesh);    // Draw circle to stencil only
/// stencil_end();             // Enable testing
/// draw_env();                // Only visible inside circle
/// draw_mesh(scene);          // Only visible inside circle
/// stencil_clear();           // Back to normal rendering
/// ```
fn stencil_begin(mut caller: Caller<'_, ZXGameContext>) {
    let state = &mut caller.data_mut().ffi;
    state.stencil_group += 1; // New stencil pass starts
    state.stencil_mode = StencilMode::Writing;
}

/// End stencil mask creation and begin stencil testing.
///
/// After calling this, subsequent draw calls will only render where
/// the stencil buffer was written (inside the mask).
///
/// Must be called after stencil_begin() has created a mask shape.
fn stencil_end(mut caller: Caller<'_, ZXGameContext>) {
    let state = &mut caller.data_mut().ffi;
    state.stencil_mode = StencilMode::Testing;
}

/// Clear stencil state and return to normal rendering.
///
/// Disables stencil operations. The stencil buffer itself is cleared
/// at the start of each frame during render pass creation.
///
/// Call this when finished with masked rendering to restore normal behavior.
fn stencil_clear(mut caller: Caller<'_, ZXGameContext>) {
    let state = &mut caller.data_mut().ffi;
    state.stencil_mode = StencilMode::Disabled;
    state.stencil_group += 1; // End of stencil pass, subsequent draws are separate
}

/// Enable inverted stencil testing.
///
/// After calling this, subsequent draw calls will only render where
/// the stencil buffer was NOT written (outside the mask).
///
/// Use this for effects like vignettes or rendering outside portals.
///
/// # Example (vignette effect)
/// ```rust,ignore
/// stencil_begin();           // Start mask creation
/// draw_mesh(rounded_rect);   // Draw center area to stencil
/// stencil_invert();          // Render OUTSIDE the mask
/// set_color(0x000000FF);     // Black vignette color
/// draw_rect(0.0, 0.0, 960.0, 540.0, 0x000000FF);  // Fill outside
/// stencil_clear();           // Back to normal
/// ```
fn stencil_invert(mut caller: Caller<'_, ZXGameContext>) {
    let state = &mut caller.data_mut().ffi;
    state.stencil_group += 1; // Separate group for inverted testing order control
    state.stencil_mode = StencilMode::TestingInverted;
}
