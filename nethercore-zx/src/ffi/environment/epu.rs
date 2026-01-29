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

use std::sync::atomic::{AtomicU32, Ordering};

use crate::ffi::ZXGameContext;
use crate::ffi::helpers::get_memory;
use crate::graphics::epu::{EpuConfig, MAX_ENV_STATES};

static EPU_SET_DEBUG_COUNT: AtomicU32 = AtomicU32::new(0);

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

/// Store an EPU config for the currently selected environment index without drawing a background.
///
/// Reads 128 bytes (8 x 128-bit = 16 x u64) from WASM memory and stores the
/// config for the **currently selected** env_id (`environment_index(...)`) for
/// this frame. If called multiple times for the same env_id in a frame, the
/// last call wins.
///
/// # Arguments
/// * `config_ptr` - Pointer to 16 u64 values (128 bytes) in WASM memory
pub(crate) fn epu_set(mut caller: Caller<'_, ZXGameContext>, config_ptr: u32) {
    let Some(config) = read_epu_config(&caller, config_ptr, "epu_set") else {
        return;
    };

    let state = &mut caller.data_mut().ffi;

    // Store config for the current env_id (selected via environment_index()).
    let env_id = state
        .current_shading_state
        .environment_index
        .min(MAX_ENV_STATES.saturating_sub(1));
    let layers = config.layers;

    if std::env::var("NETHERCORE_EPU_DEBUG_SET").as_deref() == Ok("1") {
        let n = EPU_SET_DEBUG_COUNT.fetch_add(1, Ordering::Relaxed);
        if n < 32 {
            let d0 = ((layers[0][1] >> 24) & 0xFF) as u8;
            let d3 = ((layers[3][1] >> 24) & 0xFF) as u8;
            let d4 = ((layers[4][1] >> 24) & 0xFF) as u8;
            let d6 = ((layers[6][1] >> 24) & 0xFF) as u8;

            let op0 = ((layers[0][0] >> 59) & 0x1F) as u8;
            let op3 = ((layers[3][0] >> 59) & 0x1F) as u8;
            let op4 = ((layers[4][0] >> 59) & 0x1F) as u8;
            let op6 = ((layers[6][0] >> 59) & 0x1F) as u8;

            tracing::info!(
                "epu_set debug: call={}, env_id={}, (op,d)[0]=({},{}), [3]=({},{}), [4]=({},{}), [6]=({},{}), state_hash=0x{:016x}",
                n,
                env_id,
                op0,
                d0,
                op3,
                d3,
                op4,
                d4,
                op6,
                d6,
                config.state_hash()
            );
        }
    }

    if let Some(prev) = state.epu_frame_configs.insert(env_id, config)
        && prev.layers != layers
    {
        warn!(
            "epu_set: multiple different configs pushed for env_id {} in the same frame; last call wins",
            env_id
        );
    }
}

/// Draw the environment background for the current viewport/pass.
///
/// Records a background draw request using the current view/proj + shading state.
/// The environment selected is the current `environment_index(...)`.
///
/// For best results:
/// - call `epu_set(...)` earlier in `render()`
/// - call `draw_epu()` at the end of `render()` so it fills only background pixels
pub(crate) fn draw_epu(mut caller: Caller<'_, ZXGameContext>) {
    let state = &mut caller.data_mut().ffi;

    // Capture current viewport/pass for split-screen + pass ordering
    let viewport = state.current_viewport;
    let pass_id = state.current_pass_id;

    // Capture view/proj + shading state for this environment draw.
    // Instance index for the environment shader is an index into mvp_shading_indices.
    let mvp_index = state.add_mvp_shading_state();

    // Record (and overwrite) the draw request for this viewport/pass.
    state.epu_frame_draws.insert((viewport, pass_id), mvp_index);
}
