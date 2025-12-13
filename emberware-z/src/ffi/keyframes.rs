//! Keyframe animation FFI functions
//!
//! Functions for loading and accessing animation keyframes:
//! - `keyframes_load`: Load from WASM memory (init-only)
//! - `rom_keyframes`: Load from ROM data pack (init-only)
//! - `keyframes_bone_count`: Get bone count for a collection
//! - `keyframes_frame_count`: Get frame count for a collection
//! - `keyframe_read`: Decode and read a keyframe to WASM memory
//! - `keyframe_bind`: Bind a keyframe directly to GPU (bypass WASM)

use anyhow::{Result, bail};
use tracing::warn;
use wasmtime::{Caller, Linker};

use emberware_core::wasm::GameStateWithConsole;
use z_common::formats::{
    BoneTransform, EmberZAnimationHeader, PLATFORM_BONE_KEYFRAME_SIZE, PlatformBoneKeyframe,
    decode_bone_transform,
};

use super::guards::check_init_only;
use crate::console::ZInput;
use crate::state::{
    BoneMatrix3x4, KeyframeSource, MAX_BONES, MAX_KEYFRAME_COLLECTIONS, PendingKeyframes, ZFFIState,
};

/// Register keyframe animation FFI functions
pub fn register(linker: &mut Linker<GameStateWithConsole<ZInput, ZFFIState>>) -> Result<()> {
    // Init-only loading
    linker.func_wrap("env", "keyframes_load", keyframes_load)?;
    linker.func_wrap("env", "rom_keyframes", rom_keyframes)?;

    // Query functions
    linker.func_wrap("env", "keyframes_bone_count", keyframes_bone_count)?;
    linker.func_wrap("env", "keyframes_frame_count", keyframes_frame_count)?;

    // Access functions
    linker.func_wrap("env", "keyframe_read", keyframe_read)?;
    linker.func_wrap("env", "keyframe_bind", keyframe_bind)?;

    Ok(())
}

// ═══════════════════════════════════════════════════════════════════════════
// LOADING FUNCTIONS (init-only)
// ═══════════════════════════════════════════════════════════════════════════

/// Load keyframes from WASM memory
///
/// # Arguments
/// * `data_ptr` — Pointer to .ewzanim data in WASM memory
/// * `byte_size` — Total size of the data in bytes
///
/// # Returns
/// Keyframe collection handle (>0) on success. Traps on failure.
///
/// **Init-only:** Can only be called during `init()`.
fn keyframes_load(
    mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>,
    data_ptr: u32,
    byte_size: u32,
) -> Result<u32> {
    check_init_only(&caller, "keyframes_load")?;

    // Check keyframe collection limit
    let state = &caller.data().console;
    let total_keyframes = state.keyframes.len() + state.pending_keyframes.len();
    if total_keyframes >= MAX_KEYFRAME_COLLECTIONS {
        bail!(
            "keyframes_load: maximum keyframe collection count {} exceeded",
            MAX_KEYFRAME_COLLECTIONS
        );
    }

    // Get WASM memory
    let memory = caller
        .data()
        .game
        .memory
        .ok_or_else(|| anyhow::anyhow!("keyframes_load: no WASM memory available"))?;

    let data = memory.data(&caller);
    let start = data_ptr as usize;
    let size = byte_size as usize;

    if start + size > data.len() {
        bail!(
            "keyframes_load: memory access out of bounds ({} + {} > {})",
            start,
            size,
            data.len()
        );
    }

    // Parse header
    let header_bytes = &data[start..start + EmberZAnimationHeader::SIZE.min(size)];
    let header = EmberZAnimationHeader::from_bytes(header_bytes)
        .ok_or_else(|| anyhow::anyhow!("keyframes_load: invalid header"))?;

    if !header.validate() {
        bail!(
            "keyframes_load: invalid header (bone_count={}, frame_count={}, flags={})",
            header.bone_count,
            header.frame_count,
            header.flags
        );
    }

    // Validate data size
    let expected_size = header.file_size();
    if size < expected_size {
        bail!(
            "keyframes_load: data too small ({} bytes, expected {})",
            size,
            expected_size
        );
    }

    // Check bone count against limit
    if header.bone_count as usize > MAX_BONES {
        bail!(
            "keyframes_load: bone_count {} exceeds MAX_BONES {}",
            header.bone_count,
            MAX_BONES
        );
    }

    // Copy keyframe data (skip header)
    let data_start = start + EmberZAnimationHeader::SIZE;
    let data_len = header.data_size();
    let keyframe_data = data[data_start..data_start + data_len].to_vec();

    // Allocate handle and queue pending load
    let state = &mut caller.data_mut().console;
    let handle = state.next_keyframe_handle;
    state.next_keyframe_handle += 1;

    state.pending_keyframes.push(PendingKeyframes {
        handle,
        bone_count: header.bone_count,
        frame_count: header.frame_count,
        data: keyframe_data,
    });

    tracing::info!(
        "keyframes_load: queued handle {} ({} bones, {} frames)",
        handle,
        header.bone_count,
        header.frame_count
    );

    Ok(handle)
}

/// Load keyframes from ROM data pack by ID
///
/// # Arguments
/// * `id_ptr` — Pointer to asset ID string in WASM memory
/// * `id_len` — Length of asset ID string
///
/// # Returns
/// Keyframe collection handle (>0) on success. Traps on failure.
///
/// **Init-only:** Can only be called during `init()`.
fn rom_keyframes(
    mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>,
    id_ptr: u32,
    id_len: u32,
) -> Result<u32> {
    check_init_only(&caller, "rom_keyframes")?;

    // Read asset ID from WASM memory
    let id = {
        let memory = caller
            .data()
            .game
            .memory
            .ok_or_else(|| anyhow::anyhow!("rom_keyframes: no WASM memory available"))?;
        let data = memory.data(&caller);
        let start = id_ptr as usize;
        let len = id_len as usize;

        if start + len > data.len() {
            bail!("rom_keyframes: string ID access out of bounds");
        }

        String::from_utf8(data[start..start + len].to_vec())
            .map_err(|_| anyhow::anyhow!("rom_keyframes: invalid UTF-8 in asset ID"))?
    };

    // Check keyframe collection limit
    let state = &caller.data().console;
    let total_keyframes = state.keyframes.len() + state.pending_keyframes.len();
    if total_keyframes >= MAX_KEYFRAME_COLLECTIONS {
        bail!(
            "rom_keyframes: maximum keyframe collection count {} exceeded",
            MAX_KEYFRAME_COLLECTIONS
        );
    }

    // Get keyframe data from data pack
    let (bone_count, frame_count, keyframe_data) = {
        let state = &caller.data().console;
        let data_pack = state
            .data_pack
            .as_ref()
            .ok_or_else(|| anyhow::anyhow!("rom_keyframes: no data pack loaded"))?;

        let packed = data_pack.find_keyframes(&id).ok_or_else(|| {
            anyhow::anyhow!("rom_keyframes: keyframes '{}' not found in data pack", id)
        })?;

        // Validate
        if !packed.validate() {
            bail!("rom_keyframes: invalid keyframes '{}' in data pack", id);
        }

        (packed.bone_count, packed.frame_count, packed.data.clone())
    };

    // Allocate handle and queue pending load
    let state = &mut caller.data_mut().console;
    let handle = state.next_keyframe_handle;
    state.next_keyframe_handle += 1;

    state.pending_keyframes.push(PendingKeyframes {
        handle,
        bone_count,
        frame_count,
        data: keyframe_data,
    });

    tracing::info!(
        "rom_keyframes: queued '{}' as handle {} ({} bones, {} frames)",
        id,
        handle,
        bone_count,
        frame_count
    );

    Ok(handle)
}

// ═══════════════════════════════════════════════════════════════════════════
// QUERY FUNCTIONS
// ═══════════════════════════════════════════════════════════════════════════

/// Get the bone count for a keyframe collection
///
/// # Arguments
/// * `handle` — Keyframe collection handle from keyframes_load() or rom_keyframes()
///
/// # Returns
/// Bone count (0 on invalid handle)
///
/// # Note
/// Works during init() by also checking pending_keyframes.
fn keyframes_bone_count(
    caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>,
    handle: u32,
) -> u32 {
    if handle == 0 {
        warn!("keyframes_bone_count: invalid handle 0");
        return 0;
    }

    let state = &caller.data().console;
    let index = handle as usize - 1;

    // First check finalized keyframes
    if let Some(kf) = state.keyframes.get(index) {
        return kf.bone_count as u32;
    }

    // During init(), keyframes may still be in pending_keyframes
    // Search by handle since indices don't match during pending state
    for pending in &state.pending_keyframes {
        if pending.handle == handle {
            return pending.bone_count as u32;
        }
    }

    warn!(
        "keyframes_bone_count: handle {} not found (only {} loaded, {} pending)",
        handle,
        state.keyframes.len(),
        state.pending_keyframes.len()
    );
    0
}

/// Get the frame count for a keyframe collection
///
/// # Arguments
/// * `handle` — Keyframe collection handle from keyframes_load() or rom_keyframes()
///
/// # Returns
/// Frame count (0 on invalid handle)
///
/// # Note
/// Works during init() by also checking pending_keyframes.
fn keyframes_frame_count(
    caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>,
    handle: u32,
) -> u32 {
    if handle == 0 {
        warn!("keyframes_frame_count: invalid handle 0");
        return 0;
    }

    let state = &caller.data().console;
    let index = handle as usize - 1;

    // First check finalized keyframes
    if let Some(kf) = state.keyframes.get(index) {
        return kf.frame_count as u32;
    }

    // During init(), keyframes may still be in pending_keyframes
    // Search by handle since indices don't match during pending state
    for pending in &state.pending_keyframes {
        if pending.handle == handle {
            return pending.frame_count as u32;
        }
    }

    warn!(
        "keyframes_frame_count: handle {} not found (only {} loaded, {} pending)",
        handle,
        state.keyframes.len(),
        state.pending_keyframes.len()
    );
    0
}

// ═══════════════════════════════════════════════════════════════════════════
// ACCESS FUNCTIONS
// ═══════════════════════════════════════════════════════════════════════════

/// Read a decoded keyframe into WASM memory
///
/// Decodes the platform format (16 bytes/bone) to BoneTransform format (40 bytes/bone):
/// - rotation: [f32; 4] quaternion [x, y, z, w]
/// - position: [f32; 3]
/// - scale: [f32; 3]
///
/// # Arguments
/// * `handle` — Keyframe collection handle
/// * `index` — Frame index (0-based)
/// * `out_ptr` — Pointer to output buffer in WASM memory (must be bone_count × 40 bytes)
///
/// # Traps
/// - Invalid handle (0 or not loaded)
/// - Frame index out of bounds
/// - Output buffer out of bounds
fn keyframe_read(
    mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>,
    handle: u32,
    index: u32,
    out_ptr: u32,
) -> Result<()> {
    if handle == 0 {
        bail!("keyframe_read: invalid keyframe handle 0");
    }

    // Get keyframe collection
    let (bone_count, frame_data) = {
        let state = &caller.data().console;
        let handle_index = handle as usize - 1;

        match state.keyframes.get(handle_index) {
            Some(kf) => {
                if index >= kf.frame_count as u32 {
                    bail!(
                        "keyframe_read: frame index {} >= frame_count {}",
                        index,
                        kf.frame_count
                    );
                }

                // Get frame data
                let frame_size = kf.bone_count as usize * PLATFORM_BONE_KEYFRAME_SIZE;
                let start = index as usize * frame_size;
                let end = start + frame_size;
                (kf.bone_count, kf.data[start..end].to_vec())
            }
            None => {
                bail!(
                    "keyframe_read: invalid keyframe handle {} (only {} loaded)",
                    handle,
                    state.keyframes.len()
                );
            }
        }
    };

    // Calculate output size
    let output_size = bone_count as usize * BoneTransform::SIZE;

    // Get WASM memory and validate bounds
    let memory = caller
        .data()
        .game
        .memory
        .ok_or_else(|| anyhow::anyhow!("keyframe_read: WASM memory not initialized"))?;

    let data = memory.data_mut(&mut caller);
    let out_start = out_ptr as usize;
    let out_end = out_start + output_size;

    if out_end > data.len() {
        bail!(
            "keyframe_read: output buffer out of bounds ({}-{}, memory size {})",
            out_start,
            out_end,
            data.len()
        );
    }

    // Decode each bone and write to output
    for i in 0..bone_count as usize {
        let kf_offset = i * PLATFORM_BONE_KEYFRAME_SIZE;
        let kf_bytes = &frame_data[kf_offset..kf_offset + PLATFORM_BONE_KEYFRAME_SIZE];

        // Parse platform keyframe
        let platform_kf = PlatformBoneKeyframe::from_bytes(kf_bytes);

        // Decode to BoneTransform
        let transform = decode_bone_transform(&platform_kf);

        // Write to output buffer
        let out_offset = out_start + i * BoneTransform::SIZE;
        data[out_offset..out_offset + BoneTransform::SIZE].copy_from_slice(&transform.to_bytes());
    }

    tracing::trace!(
        "keyframe_read: decoded frame {} from handle {} ({} bones)",
        index,
        handle,
        bone_count
    );

    Ok(())
}

/// Bind a keyframe directly from the static GPU buffer (Animation System v2)
///
/// Points subsequent skinned draws to use pre-decoded matrices from @binding(7) all_keyframes.
/// No CPU decoding or data transfer needed at draw time.
///
/// # Arguments
/// * `handle` — Keyframe collection handle (0 to unbind)
/// * `index` — Frame index (0-based)
///
/// # Traps
/// - Invalid handle (not loaded)
/// - Frame index out of bounds
///
/// # Animation System v2
/// Unlike the legacy `keyframe_read() -> set_bones()` path, this uses pre-uploaded
/// static keyframe data. The GPU shader reads directly from the all_keyframes buffer
/// at the computed offset.
fn keyframe_bind(
    mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>,
    handle: u32,
    index: u32,
) -> Result<()> {
    if handle == 0 {
        // Unbind keyframes - reset to default static offset 0
        let state = &mut caller.data_mut().console;
        state.current_keyframe_source = KeyframeSource::Static { offset: 0 };
        state.bone_count = 0;
        state.shading_state_dirty = true;
        tracing::trace!("keyframe_bind: unbound (offset 0)");
        return Ok(());
    }

    // Extract values from immutable borrow first
    let (offset, bone_count) = {
        let state = &caller.data().console;
        let handle_index = handle as usize - 1;

        // Validate handle against loaded keyframes
        if handle_index >= state.keyframes.len() {
            bail!(
                "keyframe_bind: invalid keyframe handle {} (only {} loaded)",
                handle,
                state.keyframes.len()
            );
        }

        // Get GPU info for this keyframe collection
        if handle_index >= state.keyframe_gpu_info.len() {
            bail!(
                "keyframe_bind: keyframe {} has no GPU info (only {} uploaded)",
                handle,
                state.keyframe_gpu_info.len()
            );
        }

        let gpu_info = &state.keyframe_gpu_info[handle_index];

        // Validate frame index
        if index >= gpu_info.frame_count as u32 {
            bail!(
                "keyframe_bind: frame index {} >= frame_count {}",
                index,
                gpu_info.frame_count
            );
        }

        // Compute the global buffer offset for this specific frame
        // Layout in all_keyframes: [kf0_frame0_bones..., kf0_frame1_bones..., kf1_frame0_bones..., ...]
        let frame_offset = index * gpu_info.bone_count as u32;
        let offset = gpu_info.keyframe_base_offset + frame_offset;
        let bone_count = gpu_info.bone_count as u32;

        (offset, bone_count)
    };

    // Update state for this draw
    let state = &mut caller.data_mut().console;
    state.current_keyframe_source = KeyframeSource::Static { offset };
    state.bone_count = bone_count;
    state.shading_state_dirty = true;

    tracing::trace!(
        "keyframe_bind: bound handle {} frame {} -> offset {} ({} bones)",
        handle,
        index,
        offset,
        bone_count
    );

    Ok(())
}

/// Convert a BoneTransform to a 3x4 bone matrix
///
/// The BoneTransform contains:
/// - rotation: quaternion [x, y, z, w]
/// - position: [x, y, z]
/// - scale: [x, y, z]
///
/// The output is a 3x4 matrix in row-major format for GPU:
/// - row0: [m00, m01, m02, tx]
/// - row1: [m10, m11, m12, ty]
/// - row2: [m20, m21, m22, tz]
///
/// Note: This function is used by tests but not runtime code since Animation System v2
/// uses pre-decoded static keyframes. Keeping it for test coverage.
#[allow(dead_code)]
fn bone_transform_to_matrix(t: &BoneTransform) -> BoneMatrix3x4 {
    let [qx, qy, qz, qw] = t.rotation;
    let [px, py, pz] = t.position;
    let [sx, sy, sz] = t.scale;

    // Quaternion to rotation matrix
    let xx = qx * qx;
    let yy = qy * qy;
    let zz = qz * qz;
    let xy = qx * qy;
    let xz = qx * qz;
    let yz = qy * qz;
    let wx = qw * qx;
    let wy = qw * qy;
    let wz = qw * qz;

    // Rotation matrix elements (row-major)
    let r00 = 1.0 - 2.0 * (yy + zz);
    let r01 = 2.0 * (xy - wz);
    let r02 = 2.0 * (xz + wy);

    let r10 = 2.0 * (xy + wz);
    let r11 = 1.0 - 2.0 * (xx + zz);
    let r12 = 2.0 * (yz - wx);

    let r20 = 2.0 * (xz - wy);
    let r21 = 2.0 * (yz + wx);
    let r22 = 1.0 - 2.0 * (xx + yy);

    // Apply scale and build 3x4 matrix (row-major for GPU)
    BoneMatrix3x4 {
        row0: [r00 * sx, r01 * sy, r02 * sz, px],
        row1: [r10 * sx, r11 * sy, r12 * sz, py],
        row2: [r20 * sx, r21 * sy, r22 * sz, pz],
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_bone_transform_to_matrix_identity() {
        let t = BoneTransform {
            rotation: [0.0, 0.0, 0.0, 1.0],
            position: [0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
        };

        let m = bone_transform_to_matrix(&t);

        // Should be identity (approximately)
        assert!((m.row0[0] - 1.0).abs() < 0.001);
        assert!((m.row0[1]).abs() < 0.001);
        assert!((m.row0[2]).abs() < 0.001);
        assert!((m.row0[3]).abs() < 0.001);

        assert!((m.row1[0]).abs() < 0.001);
        assert!((m.row1[1] - 1.0).abs() < 0.001);
        assert!((m.row1[2]).abs() < 0.001);
        assert!((m.row1[3]).abs() < 0.001);

        assert!((m.row2[0]).abs() < 0.001);
        assert!((m.row2[1]).abs() < 0.001);
        assert!((m.row2[2] - 1.0).abs() < 0.001);
        assert!((m.row2[3]).abs() < 0.001);
    }

    #[test]
    fn test_bone_transform_to_matrix_translation() {
        let t = BoneTransform {
            rotation: [0.0, 0.0, 0.0, 1.0],
            position: [1.0, 2.0, 3.0],
            scale: [1.0, 1.0, 1.0],
        };

        let m = bone_transform_to_matrix(&t);

        // Translation should be in the last column
        assert!((m.row0[3] - 1.0).abs() < 0.001);
        assert!((m.row1[3] - 2.0).abs() < 0.001);
        assert!((m.row2[3] - 3.0).abs() < 0.001);
    }

    #[test]
    fn test_bone_transform_to_matrix_scale() {
        let t = BoneTransform {
            rotation: [0.0, 0.0, 0.0, 1.0],
            position: [0.0, 0.0, 0.0],
            scale: [2.0, 3.0, 4.0],
        };

        let m = bone_transform_to_matrix(&t);

        // Scale should be on the diagonal
        assert!((m.row0[0] - 2.0).abs() < 0.001);
        assert!((m.row1[1] - 3.0).abs() < 0.001);
        assert!((m.row2[2] - 4.0).abs() < 0.001);
    }

    #[test]
    fn test_bone_transform_to_matrix_90_x_rotation() {
        // 90° rotation around X axis: quat = [sin(45°), 0, 0, cos(45°)]
        let s = std::f32::consts::FRAC_1_SQRT_2; // sin(45°) = cos(45°) = 1/√2
        let t = BoneTransform {
            rotation: [s, 0.0, 0.0, s],
            position: [0.0, 0.0, 0.0],
            scale: [1.0, 1.0, 1.0],
        };

        let m = bone_transform_to_matrix(&t);

        // X axis should be unchanged
        assert!((m.row0[0] - 1.0).abs() < 0.001);
        assert!((m.row0[1]).abs() < 0.001);
        assert!((m.row0[2]).abs() < 0.001);

        // Y axis rotates to Z: (0, 1, 0) -> (0, 0, 1)
        assert!((m.row1[0]).abs() < 0.001);
        assert!((m.row1[1]).abs() < 0.001);
        assert!((m.row1[2] - 1.0).abs() < 0.001);

        // Z axis rotates to -Y: (0, 0, 1) -> (0, -1, 0)
        assert!((m.row2[0]).abs() < 0.001);
        assert!((m.row2[1] + 1.0).abs() < 0.001);
        assert!((m.row2[2]).abs() < 0.001);
    }
}
