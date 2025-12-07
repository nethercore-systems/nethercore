//! Sky system FFI functions
//!
//! Functions for setting procedural sky gradient and sun properties.

use anyhow::Result;
use tracing::warn;
use wasmtime::{Caller, Linker};

use emberware_core::wasm::GameStateWithConsole;

use crate::console::ZInput;
use crate::state::ZFFIState;

/// Register sky system FFI functions
pub fn register(linker: &mut Linker<GameStateWithConsole<ZInput, ZFFIState>>) -> Result<()> {
    linker.func_wrap("env", "sky_set_colors", sky_set_colors)?;
    linker.func_wrap("env", "sky_set_sun", sky_set_sun)?;
    linker.func_wrap("env", "matcap_set", matcap_set)?;
    Ok(())
}

/// Set sky gradient colors
///
/// # Arguments
/// * `horizon_r` — Horizon color red (0.0-1.0)
/// * `horizon_g` — Horizon color green (0.0-1.0)
/// * `horizon_b` — Horizon color blue (0.0-1.0)
/// * `zenith_r` — Zenith color red (0.0-1.0)
/// * `zenith_g` — Zenith color green (0.0-1.0)
/// * `zenith_b` — Zenith color blue (0.0-1.0)
///
/// Sets the procedural sky gradient. Horizon is the color at eye level,
/// zenith is the color directly overhead. The gradient interpolates smoothly between them.
/// Works in all render modes (provides ambient lighting in PBR/Hybrid modes).
fn sky_set_colors(
    mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>,
    horizon_r: f32,
    horizon_g: f32,
    horizon_b: f32,
    zenith_r: f32,
    zenith_g: f32,
    zenith_b: f32,
) {
    let state = &mut caller.data_mut().console;
    state.update_sky_colors(
        [horizon_r, horizon_g, horizon_b],
        [zenith_r, zenith_g, zenith_b],
    );
}

/// Set sky sun properties
///
/// # Arguments
/// * `dir_x` — Sun direction X component (will be normalized)
/// * `dir_y` — Sun direction Y component (will be normalized)
/// * `dir_z` — Sun direction Z component (will be normalized)
/// * `color_r` — Sun color red (0.0-1.0)
/// * `color_g` — Sun color green (0.0-1.0)
/// * `color_b` — Sun color blue (0.0-1.0)
/// * `sharpness` — Sun sharpness (0.0-1.0, higher = smaller/sharper sun disc)
///
/// Sets the procedural sky sun. The sun appears as a bright disc in the sky gradient
/// and provides specular highlights in PBR/Hybrid modes.
/// Direction will be automatically normalized by the graphics backend.
fn sky_set_sun(
    mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>,
    dir_x: f32,
    dir_y: f32,
    dir_z: f32,
    color_r: f32,
    color_g: f32,
    color_b: f32,
    sharpness: f32,
) {
    let state = &mut caller.data_mut().console;

    // Validate direction vector (warn if zero-length)
    let len_sq = dir_x * dir_x + dir_y * dir_y + dir_z * dir_z;
    if len_sq < 1e-10 {
        warn!("sky_set_sun: zero-length direction vector, using default (0, 1, 0)");
        state.update_sky_sun([0.0, 1.0, 0.0], [color_r, color_g, color_b], sharpness);
        return;
    }

    state.update_sky_sun(
        [dir_x, dir_y, dir_z],
        [color_r, color_g, color_b],
        sharpness,
    );
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
fn matcap_set(
    mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>,
    slot: u32,
    texture: u32,
) {
    // Validate slot range (1-3 for matcaps)
    if !(1..=3).contains(&slot) {
        warn!("matcap_set: invalid slot {} (must be 1-3)", slot);
        return;
    }

    let state = &mut caller.data_mut().console;
    state.bound_textures[slot as usize] = texture;
}
