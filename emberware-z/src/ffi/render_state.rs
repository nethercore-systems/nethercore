//! Render state FFI functions
//!
//! Functions for setting render state like color, depth testing, culling, blending, and filtering.

use anyhow::Result;
use tracing::warn;
use wasmtime::{Caller, Linker};

use emberware_core::wasm::GameStateWithConsole;

use crate::console::ZInput;
use crate::state::ZFFIState;

/// Register render state FFI functions
pub fn register(linker: &mut Linker<GameStateWithConsole<ZInput, ZFFIState>>) -> Result<()> {
    linker.func_wrap("env", "set_color", set_color)?;
    linker.func_wrap("env", "depth_test", depth_test)?;
    linker.func_wrap("env", "cull_mode", cull_mode)?;
    linker.func_wrap("env", "blend_mode", blend_mode)?;
    linker.func_wrap("env", "texture_filter", texture_filter)?;
    Ok(())
}

/// Set the uniform tint color
///
/// # Arguments
/// * `color` — Color in 0xRRGGBBAA format
///
/// This color is multiplied with vertex colors and textures.
fn set_color(mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>, color: u32) {
    let state = &mut caller.data_mut().console;
    state.update_color(color);
}

/// Enable or disable depth testing
///
/// # Arguments
/// * `enabled` — 0 to disable, non-zero to enable
///
/// Default: enabled. Disable for 2D overlays or special effects.
fn depth_test(mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>, enabled: u32) {
    let state = &mut caller.data_mut().console;
    state.depth_test = enabled != 0;
}

/// Set the face culling mode
///
/// # Arguments
/// * `mode` — 0=none (draw both sides), 1=back (default), 2=front
///
/// Back-face culling is the default for solid 3D objects.
fn cull_mode(mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>, mode: u32) {
    let state = &mut caller.data_mut().console;

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
fn blend_mode(mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>, mode: u32) {
    let state = &mut caller.data_mut().console;

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
fn texture_filter(mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>, filter: u32) {
    let state = &mut caller.data_mut().console;

    if filter > 1 {
        warn!(
            "texture_filter({}) invalid - must be 0-1, using 0 (nearest)",
            filter
        );
        state.texture_filter = 0;
        return;
    }

    state.texture_filter = filter as u8;
}
