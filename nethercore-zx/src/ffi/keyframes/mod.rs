//! Keyframe animation FFI functions
//!
//! Functions for loading and accessing animation keyframes:
//! - `keyframes_load`: Load from WASM memory (init-only)
//! - `rom_keyframes`: Load from ROM data pack (init-only)
//! - `keyframes_bone_count`: Get bone count for a collection
//! - `keyframes_frame_count`: Get frame count for a collection
//! - `keyframe_read`: Decode and read a keyframe to WASM memory
//! - `keyframe_bind`: Bind a keyframe directly to GPU (bypass WASM)

use anyhow::Result;
use wasmtime::Linker;

use super::ZXGameContext;

mod loading;
mod query;
mod access;

/// Register keyframe animation FFI functions
pub fn register(linker: &mut Linker<ZXGameContext>) -> Result<()> {
    // Init-only loading
    linker.func_wrap("env", "keyframes_load", loading::keyframes_load)?;
    linker.func_wrap("env", "rom_keyframes", loading::rom_keyframes)?;

    // Query functions
    linker.func_wrap("env", "keyframes_bone_count", query::keyframes_bone_count)?;
    linker.func_wrap("env", "keyframes_frame_count", query::keyframes_frame_count)?;

    // Access functions
    linker.func_wrap("env", "keyframe_read", access::keyframe_read)?;
    linker.func_wrap("env", "keyframe_bind", access::keyframe_bind)?;

    Ok(())
}
