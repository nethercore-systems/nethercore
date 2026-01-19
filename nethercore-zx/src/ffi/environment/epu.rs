//! EPU (Environment Processing Unit) FFI functions
//!
//! These functions provide the instruction-based EPU API for configuring
//! and rendering procedural environments using 128-byte packed configurations.
//!
//! # v2 Format
//!
//! Each environment is 128 bytes (8 x 128-bit instructions). Each 128-bit
//! instruction is stored as two u64 values (hi, lo).

use tracing::warn;
use wasmtime::Caller;

use crate::ffi::ZXGameContext;
use crate::ffi::helpers::get_memory;
use crate::graphics::epu::EpuConfig;

/// Maximum environment ID supported by the EPU runtime.
const MAX_ENV_ID: u32 = 255;

/// Set an EPU environment configuration.
///
/// Reads 128 bytes (8 x 128-bit = 16 x u64) from WASM memory and stores them
/// as an EPU configuration.
///
/// # Arguments
/// * `env_id` - Environment slot ID (0-255)
/// * `config_ptr` - Pointer to 16 u64 values (128 bytes) in WASM memory
pub(crate) fn epu_set(mut caller: Caller<'_, ZXGameContext>, env_id: u32, config_ptr: u32) {
    // Validate env_id
    if env_id > MAX_ENV_ID {
        warn!(
            "epu_set: env_id {} exceeds maximum {} - ignoring",
            env_id, MAX_ENV_ID
        );
        return;
    }

    // Get WASM memory
    let Some(memory) = get_memory(&caller, "epu_set") else {
        return;
    };

    // Read 128 bytes (16 x u64 = 8 x 128-bit) from WASM memory
    let mem_data = memory.data(&caller);
    let ptr = config_ptr as usize;
    let size = 128; // 16 x u64 = 128 bytes

    if ptr + size > mem_data.len() {
        warn!(
            "epu_set: memory access ({} bytes at {}) exceeds bounds ({})",
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

    // Store the configuration in the FFI state
    let state = &mut caller.data_mut().ffi;
    state.epu_configs.insert(env_id, config);

    // Note: epu_dirty_envs is currently vestigial - the actual dirty tracking
    // happens in EpuCache (graphics/epu/runtime.rs) via config hash comparison.
    // This field exists for potential future use but is not read by the render path.
    state.epu_dirty_envs.insert(env_id);
}

/// Draw the background using the specified EPU environment.
///
/// # Arguments
/// * `env_id` - Environment slot ID (0-255)
pub(crate) fn epu_draw(mut caller: Caller<'_, ZXGameContext>, env_id: u32) {
    // Validate env_id
    if env_id > MAX_ENV_ID {
        warn!(
            "epu_draw: env_id {} exceeds maximum {} - ignoring",
            env_id, MAX_ENV_ID
        );
        return;
    }

    let state = &mut caller.data_mut().ffi;

    // Check if environment has been configured
    if !state.epu_configs.contains_key(&env_id) {
        warn!(
            "epu_draw: env_id {} not configured - call epu_set first",
            env_id
        );
        return;
    }

    // Capture current viewport for split-screen rendering
    let viewport = state.current_viewport;

    // Capture current pass_id for render pass ordering
    let pass_id = state.current_pass_id;

    // Add EPU environment draw command to render pass
    state
        .render_pass
        .add_command(crate::graphics::VRPCommand::EpuEnvironment {
            env_id,
            viewport,
            pass_id,
        });
}

// NOTE: epu_get_ambient() was removed because GPU readback would break
// rollback determinism (GPUs are not deterministic). Ambient lighting is
// computed and applied entirely on the GPU side, which is stateless from
// the game logic perspective.
