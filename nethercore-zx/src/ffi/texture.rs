//! Texture FFI functions
//!
//! Functions for loading and binding textures.

use anyhow::Result;
use tracing::warn;
use wasmtime::{Caller, Linker};

use super::helpers::{checked_mul, read_wasm_bytes, validate_dimensions_nonzero};
use super::{ZXGameContext, guards::guard_init_only};
use crate::graphics::MatcapBlendMode;
use crate::state::PendingTexture;
use zx_common::TextureFormat;

/// Register texture FFI functions
pub fn register(linker: &mut Linker<ZXGameContext>) -> Result<()> {
    linker.func_wrap("env", "load_texture", load_texture)?;
    linker.func_wrap("env", "texture_bind", texture_bind)?;
    linker.func_wrap("env", "texture_bind_slot", texture_bind_slot)?;
    linker.func_wrap("env", "matcap_blend_mode", matcap_blend_mode)?;
    Ok(())
}

/// Load a texture from RGBA pixel data
///
/// # Arguments
/// * `width` — Texture width in pixels
/// * `height` — Texture height in pixels
/// * `pixels_ptr` — Pointer to RGBA8 pixel data (width × height × 4 bytes)
///
/// Returns texture handle (>0) on success, 0 on failure.
/// Validates VRAM budget before allocation.
fn load_texture(
    mut caller: Caller<'_, ZXGameContext>,
    width: u32,
    height: u32,
    pixels_ptr: u32,
) -> u32 {
    const FN_NAME: &str = "load_texture";

    guard_init_only!(caller, FN_NAME);

    // Validate dimensions
    if !validate_dimensions_nonzero(width, height, FN_NAME) {
        return 0;
    }

    // Calculate size with overflow checking
    let Some(pixels) = checked_mul(width, height, FN_NAME, "dimensions") else {
        return 0;
    };
    let Some(size) = checked_mul(pixels, 4, FN_NAME, "size") else {
        return 0;
    };

    // Read pixel data from WASM memory
    let Some(pixel_data) = read_wasm_bytes(&caller, pixels_ptr, size as usize, FN_NAME) else {
        return 0;
    };

    // Now we can mutably borrow state
    let state = &mut caller.data_mut().ffi;

    // Allocate a texture handle
    let handle = state.next_texture_handle;
    state.next_texture_handle += 1;

    // Store the texture data for the graphics backend
    // load_texture() always creates RGBA8 textures (from WASM memory)
    state.pending_textures.push(PendingTexture {
        handle,
        width,
        height,
        format: TextureFormat::Rgba8,
        data: pixel_data,
    });

    handle
}

/// Bind a texture to slot 0 (albedo)
///
/// # Arguments
/// * `handle` — Texture handle from load_texture
///
/// Equivalent to texture_bind_slot(handle, 0).
fn texture_bind(mut caller: Caller<'_, ZXGameContext>, handle: u32) {
    let state = &mut caller.data_mut().ffi;
    state.bound_textures[0] = handle;
}

/// Bind a texture to a specific slot
///
/// # Arguments
/// * `handle` — Texture handle from load_texture
/// * `slot` — Slot index (0-3)
///
/// Slots: 0=albedo, 1=MRE/matcap, 2=env matcap, 3=matcap
fn texture_bind_slot(mut caller: Caller<'_, ZXGameContext>, handle: u32, slot: u32) {
    if slot > 3 {
        warn!("texture_bind_slot: invalid slot {} (max 3)", slot);
        return;
    }

    let state = &mut caller.data_mut().ffi;
    state.bound_textures[slot as usize] = handle;
}

/// Set matcap blend mode for a texture slot (Mode 1 only)
///
/// # Arguments
/// * `slot` — Matcap slot index (1-3, slot 0 is albedo and does not support blend modes)
/// * `mode` — Blend mode (0=Multiply, 1=Add, 2=HSV Modulate)
///
/// Mode 0 (Multiply): Standard matcap blending (default)
/// Mode 1 (Add): Additive blending for glow/emission effects
/// Mode 2 (HSV Modulate): Hue shifting and iridescence effects
fn matcap_blend_mode(mut caller: Caller<'_, ZXGameContext>, slot: u32, mode: u32) {
    if !(1..=3).contains(&slot) {
        warn!("matcap_blend_mode: invalid slot {} (must be 1-3)", slot);
        return;
    }

    let blend_mode = match MatcapBlendMode::from_u32(mode) {
        Some(m) => m,
        None => {
            warn!("matcap_blend_mode: invalid mode {} (must be 0-2)", mode);
            return;
        }
    };

    let state = &mut caller.data_mut().ffi;
    state.update_matcap_blend_mode(slot as usize, blend_mode); // Update single slot in unified state
}
