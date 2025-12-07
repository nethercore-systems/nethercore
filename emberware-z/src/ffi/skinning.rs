//! GPU skinning FFI functions
//!
//! Functions for setting bone transformation matrices for skeletal animation.

use anyhow::Result;
use glam::Mat4;
use tracing::warn;
use wasmtime::{Caller, Linker};

use emberware_core::wasm::GameStateWithConsole;

use crate::console::ZInput;
use crate::state::{ZFFIState, MAX_BONES};

/// Register GPU skinning FFI functions
pub fn register(linker: &mut Linker<GameStateWithConsole<ZInput, ZFFIState>>) -> Result<()> {
    linker.func_wrap("env", "set_bones", set_bones)?;
    Ok(())
}

/// Set bone transform matrices for GPU skinning
///
/// # Arguments
/// * `matrices_ptr` — Pointer to array of bone matrices in WASM memory
/// * `count` — Number of bones (max 256)
///
/// Each bone matrix is 16 floats in column-major order (same as transform_set).
/// The vertex shader will use these matrices to deform skinned vertices based on
/// their bone indices and weights.
///
/// Call this before drawing skinned meshes (meshes with FORMAT_SKINNED flag).
/// The bone transforms are typically computed on CPU each frame for skeletal animation.
fn set_bones(
    mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>,
    matrices_ptr: u32,
    count: u32,
) {
    // Validate bone count
    if count > MAX_BONES as u32 {
        warn!(
            "set_bones: bone count {} exceeds maximum {} - clamping",
            count, MAX_BONES
        );
        return;
    }

    if count == 0 {
        // Clear bone data
        let state = &mut caller.data_mut().console;
        state.bone_matrices.clear();
        state.bone_count = 0;
        return;
    }

    // Calculate required memory size (16 floats per matrix × 4 bytes per float)
    let matrix_size = 16 * 4; // 64 bytes per matrix
    let total_size = count as usize * matrix_size;

    // Get WASM memory
    let memory = match caller.data().game.memory {
        Some(mem) => mem,
        None => {
            warn!("set_bones: WASM memory not initialized");
            return;
        }
    };

    // Read matrix data from WASM memory
    let data = memory.data(&caller);
    let start = matrices_ptr as usize;
    let end = start + total_size;

    if end > data.len() {
        warn!(
            "set_bones: memory access out of bounds (requested {}-{}, memory size {})",
            start,
            end,
            data.len()
        );
        return;
    }

    // Parse matrices from memory (column-major order)
    let mut matrices = Vec::with_capacity(count as usize);
    for i in 0..count as usize {
        let offset = start + i * matrix_size;
        let matrix_bytes = &data[offset..offset + matrix_size];

        // Convert bytes to f32 array (16 floats)
        let mut floats = [0.0f32; 16];
        for (j, float) in floats.iter_mut().enumerate() {
            let byte_offset = j * 4;
            let bytes = [
                matrix_bytes[byte_offset],
                matrix_bytes[byte_offset + 1],
                matrix_bytes[byte_offset + 2],
                matrix_bytes[byte_offset + 3],
            ];
            *float = f32::from_le_bytes(bytes);
        }

        // Create Mat4 from column-major floats
        let matrix = Mat4::from_cols_array(&floats);
        matrices.push(matrix);
    }

    // Store bone matrices in render state
    let state = &mut caller.data_mut().console;
    state.bone_matrices = matrices;
    state.bone_count = count;
}
