//! GPU skinning FFI functions
//!
//! Functions for skeletal animation including:
//! - `load_skeleton`: Upload inverse bind matrices to create a skeleton
//! - `skeleton_bind`: Bind a skeleton for inverse bind mode rendering
//! - `set_bones`: Upload bone transforms for GPU skinning

use anyhow::Result;
use tracing::warn;
use wasmtime::{Caller, Linker};

use emberware_core::wasm::GameStateWithConsole;

use super::guards::check_init_only;
use crate::console::ZInput;
use crate::state::{BoneMatrix3x4, MAX_BONES, MAX_SKELETONS, PendingSkeleton, ZFFIState};

/// Register GPU skinning FFI functions
pub fn register(linker: &mut Linker<GameStateWithConsole<ZInput, ZFFIState>>) -> Result<()> {
    linker.func_wrap("env", "load_skeleton", load_skeleton)?;
    linker.func_wrap("env", "skeleton_bind", skeleton_bind)?;
    linker.func_wrap("env", "set_bones", set_bones)?;
    linker.func_wrap("env", "set_bones_4x4", set_bones_4x4)?;
    Ok(())
}

/// Load a skeleton's inverse bind matrices to GPU.
///
/// Call once during init() after loading skinned meshes.
/// The inverse bind matrices transform vertices from model space
/// to bone-local space at bind time.
///
/// # Arguments
/// * `inverse_bind_ptr` — Pointer to array of 3×4 matrices in WASM memory
///   (12 floats per matrix, column-major order)
/// * `bone_count` — Number of bones (maximum 256)
///
/// # Returns
/// * Skeleton handle (non-zero) on success
/// * 0 on error (logged to console)
///
/// # Errors
/// * bone_count exceeds 256
/// * inverse_bind_ptr is null or out of bounds
/// * Maximum skeleton count exceeded
fn load_skeleton(
    mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>,
    inverse_bind_ptr: u32,
    bone_count: u32,
) -> u32 {
    // Guard: init-only
    if let Err(e) = check_init_only(&caller, "load_skeleton") {
        warn!("{}", e);
        return 0;
    }

    // Validate bone count
    if bone_count == 0 {
        warn!("load_skeleton: bone_count is 0");
        return 0;
    }
    if bone_count > MAX_BONES as u32 {
        warn!(
            "load_skeleton: bone_count {} exceeds maximum {}",
            bone_count, MAX_BONES
        );
        return 0;
    }

    // Check skeleton limit (pending + already loaded)
    let state = &caller.data().console;
    let total_skeletons = state.skeletons.len() + state.pending_skeletons.len();
    if total_skeletons >= MAX_SKELETONS {
        warn!(
            "load_skeleton: maximum skeleton count {} exceeded",
            MAX_SKELETONS
        );
        return 0;
    }

    // Calculate required memory size (12 floats per 3x4 matrix × 4 bytes per float)
    let matrix_size = 12 * 4; // 48 bytes per bone
    let total_size = bone_count as usize * matrix_size;

    // Get WASM memory
    let memory = match caller.data().game.memory {
        Some(mem) => mem,
        None => {
            warn!("load_skeleton: WASM memory not initialized");
            return 0;
        }
    };

    // Read matrix data from WASM memory
    let data = memory.data(&caller);
    let start = inverse_bind_ptr as usize;
    let end = start + total_size;

    if end > data.len() {
        warn!(
            "load_skeleton: memory access out of bounds (requested {}-{}, memory size {})",
            start,
            end,
            data.len()
        );
        return 0;
    }

    // Parse 3x4 matrices from memory (column-major order, like transform_set)
    // Input layout: [col0.xyz, col1.xyz, col2.xyz, col3.xyz]
    // Output layout: vec4 rows (transposed for GPU alignment)
    let mut inverse_bind = Vec::with_capacity(bone_count as usize);
    for i in 0..bone_count as usize {
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
            row0: [floats[0], floats[3], floats[6], floats[9]],
            row1: [floats[1], floats[4], floats[7], floats[10]],
            row2: [floats[2], floats[5], floats[8], floats[11]],
        };
        inverse_bind.push(matrix);
    }

    // Store pending skeleton and allocate handle
    let state = &mut caller.data_mut().console;
    let handle = state.next_skeleton_handle;
    state.next_skeleton_handle += 1;

    state.pending_skeletons.push(PendingSkeleton {
        handle,
        inverse_bind,
        bone_count,
    });

    tracing::info!(
        "load_skeleton: queued skeleton {} with {} bones",
        handle,
        bone_count
    );

    handle
}

/// Bind a skeleton for subsequent skinned mesh rendering.
///
/// When a skeleton is bound, set_bones() expects model-space transforms
/// and the GPU automatically applies the inverse bind matrices.
///
/// # Arguments
/// * `skeleton` — Skeleton handle from load_skeleton(), or 0 to unbind
///
/// # Behavior
/// * skeleton > 0: Enable inverse bind mode. set_bones() receives model transforms.
/// * skeleton = 0: Disable inverse bind mode (raw). set_bones() receives final matrices.
///
/// # Notes
/// * Binding persists until changed (not reset per frame)
/// * Call multiple times per frame to render different skeletons
/// * Invalid handles are ignored with a warning
fn skeleton_bind(mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>, skeleton: u32) {
    let state = &mut caller.data_mut().console;

    if skeleton == 0 {
        // Unbind skeleton (raw mode)
        state.bound_skeleton = 0;
        state.update_skinning_mode(false);
        tracing::trace!("skeleton_bind: unbound (raw mode)");
        return;
    }

    // Validate handle (handles are 1-indexed, stored 0-indexed)
    let index = skeleton as usize - 1;
    if index >= state.skeletons.len() {
        warn!(
            "skeleton_bind: invalid skeleton handle {} (only {} skeletons loaded)",
            skeleton,
            state.skeletons.len()
        );
        return;
    }

    state.bound_skeleton = skeleton;
    state.update_skinning_mode(true);
    tracing::trace!("skeleton_bind: bound skeleton {}", skeleton);
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
            row0: [floats[0], floats[3], floats[6], floats[9]],
            row1: [floats[1], floats[4], floats[7], floats[10]],
            row2: [floats[2], floats[5], floats[8], floats[11]],
        };
        matrices.push(matrix);
    }

    // Store bone matrices in render state
    let state = &mut caller.data_mut().console;
    state.bone_matrices = matrices;
    state.bone_count = count;
}

/// Set bone transform matrices for GPU skinning (4x4 format)
///
/// # Arguments
/// * `matrices_ptr` — Pointer to array of 4x4 bone matrices in WASM memory
/// * `count` — Number of bones (max 256)
///
/// Each bone matrix is 16 floats in **column-major** order:
/// ```text
/// [m00, m10, m20, m30]  // col 0
/// [m01, m11, m21, m31]  // col 1
/// [m02, m12, m22, m32]  // col 2
/// [tx,  ty,  tz,  m33]  // col 3
/// ```
///
/// The 4th row is assumed to be [0, 0, 0, 1] (affine transforms) and is discarded.
/// This function converts the 4x4 matrices to the internal 3x4 format.
///
/// Use this when you have 4x4 matrices from a math library like glam.
/// For 3x4 matrices, use `set_bones()` instead for better performance.
fn set_bones_4x4(
    mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>,
    matrices_ptr: u32,
    count: u32,
) {
    // Validate bone count
    if count > MAX_BONES as u32 {
        warn!(
            "set_bones_4x4: bone count {} exceeds maximum {} - clamping",
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

    // Calculate required memory size (16 floats per 4x4 matrix × 4 bytes per float)
    let matrix_size = 16 * 4; // 64 bytes per bone
    let total_size = count as usize * matrix_size;

    // Get WASM memory
    let memory = match caller.data().game.memory {
        Some(mem) => mem,
        None => {
            warn!("set_bones_4x4: WASM memory not initialized");
            return;
        }
    };

    // Read matrix data from WASM memory
    let data = memory.data(&caller);
    let start = matrices_ptr as usize;
    let end = start + total_size;

    if end > data.len() {
        warn!(
            "set_bones_4x4: memory access out of bounds (requested {}-{}, memory size {})",
            start,
            end,
            data.len()
        );
        return;
    }

    // Parse 4x4 matrices from memory (column-major order)
    // Input layout: [col0.xyzw, col1.xyzw, col2.xyzw, col3.xyzw]
    // Output layout: 3x4 row-major for GPU (drop 4th row)
    let mut matrices = Vec::with_capacity(count as usize);
    for i in 0..count as usize {
        let offset = start + i * matrix_size;
        let matrix_bytes = &data[offset..offset + matrix_size];

        // Convert bytes to f32 array (16 floats in column-major order)
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

        // 4x4 column-major layout:
        // col0: [0, 1, 2, 3]    -> m00, m10, m20, m30
        // col1: [4, 5, 6, 7]    -> m01, m11, m21, m31
        // col2: [8, 9, 10, 11]  -> m02, m12, m22, m32
        // col3: [12, 13, 14, 15] -> tx, ty, tz, m33
        //
        // Convert to 3x4 row-major (drop 4th row: indices 3, 7, 11, 15):
        // row0: [m00, m01, m02, tx]  = [floats[0], floats[4], floats[8], floats[12]]
        // row1: [m10, m11, m12, ty]  = [floats[1], floats[5], floats[9], floats[13]]
        // row2: [m20, m21, m22, tz]  = [floats[2], floats[6], floats[10], floats[14]]
        let matrix = BoneMatrix3x4 {
            row0: [floats[0], floats[4], floats[8], floats[12]],
            row1: [floats[1], floats[5], floats[9], floats[13]],
            row2: [floats[2], floats[6], floats[10], floats[14]],
        };
        matrices.push(matrix);
    }

    // Store bone matrices in render state
    let state = &mut caller.data_mut().console;
    state.bone_matrices = matrices;
    state.bone_count = count;
}
