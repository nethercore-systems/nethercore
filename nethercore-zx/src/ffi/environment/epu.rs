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

    // Mark environment as dirty so it gets rebuilt
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

/// Sample the ambient cube for diffuse lighting from an EPU environment.
///
/// Returns the diffuse irradiance approximation for a given surface normal direction.
///
/// # Arguments
/// * `env_id` - Environment slot ID (0-255)
/// * `normal_x`, `normal_y`, `normal_z` - Surface normal direction (normalized)
///
/// # Returns
/// Packed RGB color as u32 in 0xRRGGBB00 format (alpha channel unused).
pub(crate) fn epu_get_ambient(
    caller: Caller<'_, ZXGameContext>,
    env_id: u32,
    normal_x: f32,
    normal_y: f32,
    normal_z: f32,
) -> u32 {
    // Validate env_id
    if env_id > MAX_ENV_ID {
        warn!(
            "epu_get_ambient: env_id {} exceeds maximum {} - returning black",
            env_id, MAX_ENV_ID
        );
        return 0;
    }

    let state = &caller.data().ffi;

    // Check if ambient cube data is available
    let Some(ambient_cube) = state.epu_ambient_cubes.get(&env_id) else {
        // Environment not configured or ambient not yet computed
        return 0;
    };

    // Normalize the normal vector
    let len_sq = normal_x * normal_x + normal_y * normal_y + normal_z * normal_z;
    if len_sq < 0.0001 {
        // Near-zero normal, return average of all directions
        let avg_r = (ambient_cube.pos_x[0]
            + ambient_cube.neg_x[0]
            + ambient_cube.pos_y[0]
            + ambient_cube.neg_y[0]
            + ambient_cube.pos_z[0]
            + ambient_cube.neg_z[0])
            / 6.0;
        let avg_g = (ambient_cube.pos_x[1]
            + ambient_cube.neg_x[1]
            + ambient_cube.pos_y[1]
            + ambient_cube.neg_y[1]
            + ambient_cube.pos_z[1]
            + ambient_cube.neg_z[1])
            / 6.0;
        let avg_b = (ambient_cube.pos_x[2]
            + ambient_cube.neg_x[2]
            + ambient_cube.pos_y[2]
            + ambient_cube.neg_y[2]
            + ambient_cube.pos_z[2]
            + ambient_cube.neg_z[2])
            / 6.0;
        return pack_rgb(avg_r, avg_g, avg_b);
    }

    let inv_len = 1.0 / len_sq.sqrt();
    let nx = normal_x * inv_len;
    let ny = normal_y * inv_len;
    let nz = normal_z * inv_len;

    // 6-direction ambient cube interpolation
    // Same algorithm as the GPU shader: weight by max(0, n.component) for each direction
    let pos_x_w = nx.max(0.0);
    let neg_x_w = (-nx).max(0.0);
    let pos_y_w = ny.max(0.0);
    let neg_y_w = (-ny).max(0.0);
    let pos_z_w = nz.max(0.0);
    let neg_z_w = (-nz).max(0.0);

    let r = ambient_cube.pos_x[0] * pos_x_w
        + ambient_cube.neg_x[0] * neg_x_w
        + ambient_cube.pos_y[0] * pos_y_w
        + ambient_cube.neg_y[0] * neg_y_w
        + ambient_cube.pos_z[0] * pos_z_w
        + ambient_cube.neg_z[0] * neg_z_w;

    let g = ambient_cube.pos_x[1] * pos_x_w
        + ambient_cube.neg_x[1] * neg_x_w
        + ambient_cube.pos_y[1] * pos_y_w
        + ambient_cube.neg_y[1] * neg_y_w
        + ambient_cube.pos_z[1] * pos_z_w
        + ambient_cube.neg_z[1] * neg_z_w;

    let b = ambient_cube.pos_x[2] * pos_x_w
        + ambient_cube.neg_x[2] * neg_x_w
        + ambient_cube.pos_y[2] * pos_y_w
        + ambient_cube.neg_y[2] * neg_y_w
        + ambient_cube.pos_z[2] * pos_z_w
        + ambient_cube.neg_z[2] * neg_z_w;

    pack_rgb(r, g, b)
}

/// Pack RGB floats (0.0-1.0+) to 0xRRGGBB00 format.
#[inline]
fn pack_rgb(r: f32, g: f32, b: f32) -> u32 {
    let r_u8 = (r.clamp(0.0, 1.0) * 255.0).round() as u32;
    let g_u8 = (g.clamp(0.0, 1.0) * 255.0).round() as u32;
    let b_u8 = (b.clamp(0.0, 1.0) * 255.0).round() as u32;
    (r_u8 << 24) | (g_u8 << 16) | (b_u8 << 8)
}
