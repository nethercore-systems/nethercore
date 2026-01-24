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
use crate::graphics::epu::{EpuConfig, MAX_ENV_STATES};

fn read_epu_config(
    caller: &Caller<'_, ZXGameContext>,
    config_ptr: u32,
    fn_name: &str,
) -> Option<EpuConfig> {
    // Get WASM memory
    let Some(memory) = get_memory(caller, fn_name) else {
        return None;
    };

    // Read 128 bytes (16 x u64 = 8 x 128-bit) from WASM memory
    let mem_data = memory.data(caller);
    let ptr = config_ptr as usize;
    let size = 128; // 16 x u64 = 128 bytes

    if ptr + size > mem_data.len() {
        warn!(
            "{fn_name}: memory access ({} bytes at {}) exceeds bounds ({})",
            size,
            ptr,
            mem_data.len()
        );
        return None;
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

    Some(EpuConfig { layers })
}

/// Store an EPU config for an environment ID without drawing a background.
///
/// Reads 128 bytes (8 x 128-bit = 16 x u64) from WASM memory and stores the
/// config for the given `env_id` for this frame. If called multiple times for
/// the same `env_id` in a frame, the last call wins.
///
/// # Arguments
/// * `env_id` - Environment ID (0..255)
/// * `config_ptr` - Pointer to 16 u64 values (128 bytes) in WASM memory
pub(crate) fn epu_set_env(mut caller: Caller<'_, ZXGameContext>, env_id: u32, config_ptr: u32) {
    let env_id_clamped = env_id.min(MAX_ENV_STATES.saturating_sub(1));
    if env_id != env_id_clamped {
        warn!(
            "epu_set_env: env_id {} out of range, clamped to {} (max {})",
            env_id,
            env_id_clamped,
            MAX_ENV_STATES.saturating_sub(1)
        );
    }

    let Some(config) = read_epu_config(&caller, config_ptr, "epu_set_env") else {
        return;
    };

    let state = &mut caller.data_mut().ffi;

    let layers = config.layers;
    if let Some(prev) = state.epu_frame_configs.insert(env_id_clamped, config)
        && prev.layers != layers
    {
        warn!(
            "epu_set_env: multiple different configs pushed for env_id {} in the same frame; last call wins",
            env_id_clamped
        );
    }
}

/// Draw the background using an EPU config (push-only, stateless API).
///
/// Reads 128 bytes (8 x 128-bit = 16 x u64) from WASM memory and stores the
/// config for the **currently selected** env_id (`environment_index(...)`),
/// then records a background draw request for the current viewport/pass.
///
/// If called multiple times for the same env_id in a frame, the last call wins.
///
/// # Arguments
/// * `config_ptr` - Pointer to 16 u64 values (128 bytes) in WASM memory
pub(crate) fn epu_draw(mut caller: Caller<'_, ZXGameContext>, config_ptr: u32) {
    let Some(config) = read_epu_config(&caller, config_ptr, "epu_draw") else {
        return;
    };

    let state = &mut caller.data_mut().ffi;

    // Capture current viewport/pass for split-screen + pass ordering
    let viewport = state.current_viewport;
    let pass_id = state.current_pass_id;

    // Store config for the current env_id (selected via environment_index()).
    let env_id = state
        .current_shading_state
        .environment_index
        .min(MAX_ENV_STATES.saturating_sub(1));
    let layers = config.layers;
    if let Some(prev) = state.epu_frame_configs.insert(env_id, config)
        && prev.layers != layers
    {
        warn!(
            "epu_draw: multiple different configs pushed for env_id {} in the same frame; last call wins",
            env_id
        );
    }

    // Capture view/proj + shading state for this environment draw.
    // Instance index for the environment shader is an index into mvp_shading_indices.
    let mvp_index = state.add_mvp_shading_state();

    // Record (and overwrite) the draw request for this viewport/pass.
    state.epu_frame_draws.insert((viewport, pass_id), mvp_index);
}
