//! GPU skinning FFI functions
//!
//! Functions for setting bone transformation matrices for skeletal animation.

use anyhow::Result;
use glam::Vec4;
use tracing::warn;
use wasmtime::{Caller, Linker};

use emberware_core::wasm::GameStateWithConsole;

use crate::console::ZInput;
use crate::state::{BoneMatrix3x4, ZFFIState, MAX_BONES};

/// Register GPU skinning FFI functions
pub fn register(linker: &mut Linker<GameStateWithConsole<ZInput, ZFFIState>>) -> Result<()> {
    linker.func_wrap("env", "set_bones", set_bones)?;
    Ok(())
}

/// Set bone transform matrices for GPU skinning
///
/// # Arguments
/// * `matrices_ptr` — Pointer to array of 3x4 bone matrices in WASM memory
/// * `count` — Number of bones (max 256)
///
/// Each bone matrix is 12 floats in **column-major** order (consistent with transform_set):
/// ```text
/// [m00, m10, m20]  // col 0: X axis
/// [m01, m11, m21]  // col 1: Y axis
/// [m02, m12, m22]  // col 2: Z axis
/// [tx,  ty,  tz ]  // col 3: translation
/// // implicit 4th row [0, 0, 0, 1] (affine transform)
/// ```
///
/// This is the same convention as `transform_set`, `view_matrix_set`, etc.
/// Internally transposed to vec4 rows for GPU alignment efficiency.
///
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

    // Calculate required memory size (12 floats per 3x4 matrix × 4 bytes per float)
    let matrix_size = 12 * 4; // 48 bytes per bone
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

    // Parse 3x4 matrices from memory (column-major order, like transform_set)
    // Input layout: [col0.xyz, col1.xyz, col2.xyz, col3.xyz]
    // Output layout: vec4 rows (transposed for GPU alignment)
    let mut matrices = Vec::with_capacity(count as usize);
    for i in 0..count as usize {
        let offset = start + i * matrix_size;
        let matrix_bytes = &data[offset..offset + matrix_size];

        // Convert bytes to f32 array (12 floats in column-major order)
        let mut floats = [0.0f32; 12];
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

        // Transpose column-major input to row-major storage (for GPU alignment)
        // Input:  [col0.x, col0.y, col0.z, col1.x, col1.y, col1.z, col2.x, col2.y, col2.z, tx, ty, tz]
        // Output: row0 = [col0.x, col1.x, col2.x, tx]
        //         row1 = [col0.y, col1.y, col2.y, ty]
        //         row2 = [col0.z, col1.z, col2.z, tz]
        let matrix = BoneMatrix3x4 {
            row0: Vec4::new(floats[0], floats[3], floats[6], floats[9]),
            row1: Vec4::new(floats[1], floats[4], floats[7], floats[10]),
            row2: Vec4::new(floats[2], floats[5], floats[8], floats[11]),
        };
        matrices.push(matrix);
    }

    // Store bone matrices in render state
    let state = &mut caller.data_mut().console;
    state.bone_matrices = matrices;
    state.bone_count = count;
}
