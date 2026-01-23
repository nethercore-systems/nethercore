//! EPU (Environment Processing Unit) FFI functions
//!
//! These functions provide the instruction-based EPU API for configuring
//! and rendering procedural environments using 128-byte packed configurations.
//!
//! # Format
//!
//! Each environment is 128 bytes (8 x 128-bit instructions). Each 128-bit
//! instruction is stored as two u64 values (hi, lo).

use tracing::warn;
use wasmtime::Caller;

use crate::ffi::ZXGameContext;
use crate::ffi::helpers::get_memory;
use crate::graphics::epu::EpuConfig;

/// Draw the background using an EPU config (push-only, stateless API).
///
/// Reads 128 bytes (8 x 128-bit = 16 x u64) from WASM memory and stores the
/// config as the frame's EPU environment. If called multiple times in a frame,
/// only the last call is used.
///
/// # Arguments
/// * `config_ptr` - Pointer to 16 u64 values (128 bytes) in WASM memory
pub(crate) fn epu_draw(mut caller: Caller<'_, ZXGameContext>, config_ptr: u32) {
    // Get WASM memory
    let Some(memory) = get_memory(&caller, "epu_draw") else {
        return;
    };

    // Read 128 bytes (16 x u64 = 8 x 128-bit) from WASM memory
    let mem_data = memory.data(&caller);
    let ptr = config_ptr as usize;
    let size = 128; // 16 x u64 = 128 bytes

    if ptr + size > mem_data.len() {
        warn!(
            "epu_draw: memory access ({} bytes at {}) exceeds bounds ({})",
            size,
            ptr,
            mem_data.len()
        );
        return;
    }

    // Read the 16 u64 values (8 pairs of [hi, lo])
    let bytes = &mem_data[ptr..ptr + size];
    let layers: [[u64; 2]; 8] = {
        let mut arr = [[0u64; 2]; 8];
        for (i, chunk) in bytes.chunks_exact(16).enumerate() {
            // Each 128-bit instruction is [hi, lo] in little-endian
            arr[i][0] = u64::from_le_bytes(chunk[0..8].try_into().unwrap());
            arr[i][1] = u64::from_le_bytes(chunk[8..16].try_into().unwrap());
        }
        arr
    };

    // Create EpuConfig from the layers
    let config = EpuConfig { layers };

    let state = &mut caller.data_mut().ffi;

    // Capture current viewport/pass for split-screen + pass ordering
    let viewport = state.current_viewport;
    let pass_id = state.current_pass_id;

    // Warn if multiple different configs are pushed in the same frame.
    if let Some(prev) = state.epu_frame_config
        && prev.layers != config.layers
        && !state.epu_frame_draws.is_empty()
    {
        warn!("epu_draw: multiple different configs pushed in the same frame; last call wins");
    }

    // Last call wins for the frame config.
    state.epu_frame_config = Some(config);

    // Capture view/proj + shading state for this environment draw.
    // Instance index for the environment shader is an index into mvp_shading_indices.
    let mvp_index = state.add_mvp_shading_state();

    // Record (and overwrite) the draw request for this viewport/pass.
    state.epu_frame_draws.insert((viewport, pass_id), mvp_index);
}
