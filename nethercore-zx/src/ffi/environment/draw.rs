//! Environment drawing and utility functions
//!
//! This module contains functions for rendering configured environments
//! and binding matcap textures.

use tracing::warn;
use wasmtime::Caller;

use crate::ffi::ZXGameContext;

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
