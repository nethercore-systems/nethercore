//! Render state FFI functions
//!
//! Functions for setting render state like color, depth testing, culling, blending, and filtering.

use anyhow::Result;
use tracing::warn;
use wasmtime::{Caller, Linker};

use super::ZXGameContext;

/// Register render state FFI functions
pub fn register(linker: &mut Linker<ZXGameContext>) -> Result<()> {
    linker.func_wrap("env", "set_color", set_color)?;
    linker.func_wrap("env", "depth_test", depth_test)?;
    linker.func_wrap("env", "cull_mode", cull_mode)?;
    linker.func_wrap("env", "blend_mode", blend_mode)?;
    linker.func_wrap("env", "texture_filter", texture_filter)?;
    linker.func_wrap("env", "uniform_alpha", uniform_alpha)?;
    linker.func_wrap("env", "dither_offset", dither_offset)?;
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
/// * `mode` — 0=none (draw both sides), 1=back (default), 2=front
///
/// Back-face culling is the default for solid 3D objects.
fn cull_mode(mut caller: Caller<'_, ZXGameContext>, mode: u32) {
    let state = &mut caller.data_mut().ffi;

    if mode > 2 {
        warn!("cull_mode({}) invalid - must be 0-2, using 0 (none)", mode);
        state.cull_mode = 0;
        return;
    }

    state.cull_mode = mode as u8;
}

/// Set the blend mode for transparent rendering
///
/// # Arguments
/// * `mode` — 0=none (opaque), 1=alpha, 2=additive, 3=multiply
///
/// Default: none (opaque). Use alpha for transparent textures.
/// Note: Blend mode is stored per-draw command for pipeline selection, not in GPU shading state.
fn blend_mode(mut caller: Caller<'_, ZXGameContext>, mode: u32) {
    let state = &mut caller.data_mut().ffi;

    if mode > 3 {
        warn!("blend_mode({}) invalid - must be 0-3, using 0 (none)", mode);
        state.blend_mode = 0;
        return;
    }

    state.blend_mode = mode as u8;
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
        state.texture_filter = 0;
        state.update_texture_filter(false);
        return;
    }

    state.texture_filter = filter as u8;
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
