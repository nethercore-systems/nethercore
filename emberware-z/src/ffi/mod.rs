//! Emberware Z FFI host functions
//!
//! Console-specific FFI functions for the PS1/N64 aesthetic fantasy console.
//! These functions are registered with the WASM linker and called by games.

#![allow(clippy::too_many_arguments)]

mod assets;

// ============================================================================
// Color Utilities
// ============================================================================

/// Unpack a 0xRRGGBBAA color to normalized [r, g, b, a] floats (0.0-1.0)
#[inline]
pub fn unpack_rgba(color: u32) -> [f32; 4] {
    [
        ((color >> 24) & 0xFF) as f32 / 255.0,
        ((color >> 16) & 0xFF) as f32 / 255.0,
        ((color >> 8) & 0xFF) as f32 / 255.0,
        (color & 0xFF) as f32 / 255.0,
    ]
}

/// Unpack a 0xRRGGBBAA color to normalized [r, g, b] floats (0.0-1.0), ignoring alpha
#[inline]
pub(crate) fn unpack_rgb(color: u32) -> [f32; 3] {
    [
        ((color >> 24) & 0xFF) as f32 / 255.0,
        ((color >> 16) & 0xFF) as f32 / 255.0,
        ((color >> 8) & 0xFF) as f32 / 255.0,
    ]
}

// ============================================================================
// WASM Memory Utilities
// ============================================================================

/// Get WASM memory from a Caller
///
/// Returns `None` if the WASM module doesn't export memory (should never happen
/// for valid WASM modules).
#[inline]
pub(crate) fn get_wasm_memory<T>(caller: &mut Caller<'_, T>) -> Option<Memory> {
    match caller.get_export("memory") {
        Some(Extern::Memory(mem)) => Some(mem),
        _ => None,
    }
}

mod audio;
mod billboard;
mod camera;
mod config;
mod draw_2d;
mod draw_3d;
pub(crate) mod guards;
pub mod input;
mod keyframes;
mod lighting;
mod material;
mod mesh;
mod mesh_generators;
mod render_state;
mod rom;
mod skinning;
mod sky;
mod texture;
mod transform;

use anyhow::Result;
use wasmtime::{Caller, Extern, Linker, Memory};

use emberware_core::wasm::WasmGameContext;

use crate::console::ZInput;
use crate::state::ZFFIState;

/// Type alias for Emberware Z WASM game context
pub type ZGameContext = WasmGameContext<ZInput, ZFFIState, ()>;

/// Register all Emberware Z FFI functions with the linker
pub fn register_z_ffi(linker: &mut Linker<ZGameContext>) -> Result<()> {
    // Configuration functions (init-only)
    config::register(linker)?;

    // Camera functions
    camera::register(linker)?;

    // Transform functions
    transform::register(linker)?;

    // Input functions (from input submodule)
    input::register(linker)?;

    // Render state functions
    render_state::register(linker)?;

    // Texture functions
    texture::register(linker)?;

    // Mesh functions (retained mode)
    mesh::register(linker)?;

    // Procedural mesh generation
    mesh_generators::register(linker)?;

    // Immediate mode 3D drawing
    draw_3d::register(linker)?;

    // Billboard drawing
    billboard::register(linker)?;

    // 2D drawing (screen space)
    draw_2d::register(linker)?;

    // Sky system + matcap
    sky::register(linker)?;

    // Material functions
    material::register(linker)?;

    // Lighting functions (Mode 2 PBR)
    lighting::register(linker)?;

    // GPU skinning
    skinning::register(linker)?;

    // Keyframe animations
    keyframes::register(linker)?;

    // Audio functions
    audio::register(linker)?;

    // EmberZ format loading (load_zmesh, load_ztex, load_zsound)
    assets::register(linker)?;

    // ROM data pack loading (rom_texture, rom_mesh, rom_sound, etc.)
    rom::register(linker)?;

    Ok(())
}
