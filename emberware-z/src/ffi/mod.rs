//! Emberware Z FFI host functions
//!
//! Console-specific FFI functions for the PS1/N64 aesthetic fantasy console.
//! These functions are registered with the WASM linker and called by games.

#![allow(clippy::too_many_arguments)]

mod audio;
mod billboard;
mod camera;
mod config;
mod draw_2d;
mod draw_3d;
pub mod input;
mod lighting;
mod material;
mod mesh;
mod render_state;
mod skinning;
mod sky;
mod texture;
mod transform;

use anyhow::Result;
use wasmtime::Linker;

use emberware_core::wasm::GameStateWithConsole;

use crate::console::ZInput;
use crate::state::ZFFIState;

/// Register all Emberware Z FFI functions with the linker
pub fn register_z_ffi(linker: &mut Linker<GameStateWithConsole<ZInput, ZFFIState>>) -> Result<()> {
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

    // Audio functions
    audio::register(linker)?;

    Ok(())
}
