//! Keyframe animation FFI functions
//!
//! Functions for loading and accessing animation keyframes:
//! - `keyframes_load`: Load from WASM memory (init-only)
//! - `rom_keyframes`: Load from ROM data pack (init-only)
//! - `keyframes_bone_count`: Get bone count for a collection
//! - `keyframes_frame_count`: Get frame count for a collection
//! - `keyframe_read`: Decode and read a keyframe to WASM memory
//! - `keyframe_bind`: Bind a keyframe directly to GPU (bypass WASM)

use anyhow::{bail, Result};
use tracing::warn;
use wasmtime::{Caller, Linker};

use emberware_core::wasm::GameStateWithConsole;
use z_common::formats::{
    decode_bone_transform, BoneTransform, EmberZAnimationHeader, PlatformBoneKeyframe,
    PLATFORM_BONE_KEYFRAME_SIZE,
};

use crate::console::ZInput;
use crate::state::{
    BoneMatrix3x4, LoadedKeyframeCollection, PendingKeyframes, ZFFIState, MAX_BONES,
    MAX_KEYFRAME_COLLECTIONS,
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

/// Check if we're in init phase (init-only function guard)
fn check_init_only(
    caller: &Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>,
    fn_name: &str,
) -> Result<()> {
    if !caller.data().game.in_init {
        bail!("{}: can only be called during init()", fn_name);
    }
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
fn keyframes_bone_count(
    caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>,
    handle: u32,
) -> u8 {
    if handle == 0 {
        warn!("keyframes_bone_count: invalid handle 0");
        return 0;
    }

    let state = &caller.data().console;
    let index = handle as usize - 1;

    if let Some(kf) = state.keyframes.get(index) {
        kf.bone_count
    } else {
        warn!(
            "keyframes_bone_count: handle {} not found (only {} loaded)",
            handle,
            state.keyframes.len()
        );
        0
    }
}

/// Get the frame count for a keyframe collection
///
/// # Arguments
/// * `handle` — Keyframe collection handle from keyframes_load() or rom_keyframes()
///
/// # Returns
/// Frame count (0 on invalid handle)
fn keyframes_frame_count(
    caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>,
    handle: u32,
) -> u16 {
    if handle == 0 {
        warn!("keyframes_frame_count: invalid handle 0");
        return 0;
    }

    let state = &caller.data().console;
    let index = handle as usize - 1;

    if let Some(kf) = state.keyframes.get(index) {
        kf.frame_count
    } else {
        warn!(
            "keyframes_frame_count: handle {} not found (only {} loaded)",
            handle,
            state.keyframes.len()
        );
        0
    }
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
/// * `frame_index` — Frame to read (0-based)
/// * `out_ptr` — Pointer to output buffer in WASM memory (must be bone_count × 40 bytes)
///
/// # Behavior
/// Writes `bone_count` BoneTransform structs to the output buffer.
/// Returns 1 on success, 0 on failure.
fn keyframe_read(
    mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>,
    handle: u32,
    frame_index: u16,
    out_ptr: u32,
) -> u32 {
    if handle == 0 {
        warn!("keyframe_read: invalid handle 0");
        return 0;
    }

    // Get keyframe collection
    let (bone_count, frame_count, frame_data) = {
        let state = &caller.data().console;
        let index = handle as usize - 1;

        match state.keyframes.get(index) {
            Some(kf) => {
                if frame_index >= kf.frame_count {
                    warn!(
                        "keyframe_read: frame {} out of bounds (collection has {} frames)",
                        frame_index, kf.frame_count
                    );
                    return 0;
                }

                // Get frame data
                let frame_size = kf.bone_count as usize * PLATFORM_BONE_KEYFRAME_SIZE;
                let start = frame_index as usize * frame_size;
                let end = start + frame_size;
                (kf.bone_count, kf.frame_count, kf.data[start..end].to_vec())
            }
            None => {
                warn!(
                    "keyframe_read: handle {} not found (only {} loaded)",
                    handle,
                    state.keyframes.len()
                );
                return 0;
            }
        }
    };

    // Calculate output size
    let output_size = bone_count as usize * BoneTransform::SIZE;

    // Get WASM memory and validate bounds
    let memory = match caller.data().game.memory {
        Some(mem) => mem,
        None => {
            warn!("keyframe_read: WASM memory not initialized");
            return 0;
        }
    };

    let data = memory.data_mut(&mut caller);
    let out_start = out_ptr as usize;
    let out_end = out_start + output_size;

    if out_end > data.len() {
        warn!(
            "keyframe_read: output buffer out of bounds ({}-{}, memory size {})",
            out_start,
            out_end,
            data.len()
        );
        return 0;
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
        frame_index,
        handle,
        bone_count
    );

    1 // Success
}

/// Bind a keyframe directly to GPU bone matrices
///
/// Decodes the platform format and uploads directly to the GPU bone buffer,
/// bypassing WASM memory. Use this for the "stamp" path when no blending is needed.
///
/// # Arguments
/// * `handle` — Keyframe collection handle
/// * `frame_index` — Frame to bind (0-based)
///
/// # Behavior
/// Sets up bone matrices for subsequent skinned mesh draws.
/// This is equivalent to calling keyframe_read() + converting to matrices + set_bones().
fn keyframe_bind(
    mut caller: Caller<'_, GameStateWithConsole<ZInput, ZFFIState>>,
    handle: u32,
    frame_index: u16,
) {
    if handle == 0 {
        warn!("keyframe_bind: invalid handle 0");
        return;
    }

    // Get keyframe collection and decode
    let bone_matrices: Vec<BoneMatrix3x4> = {
        let state = &caller.data().console;
        let index = handle as usize - 1;

        match state.keyframes.get(index) {
            Some(kf) => {
                if frame_index >= kf.frame_count {
                    warn!(
                        "keyframe_bind: frame {} out of bounds (collection has {} frames)",
                        frame_index, kf.frame_count
                    );
                    return;
                }

                // Get frame data
                let frame_size = kf.bone_count as usize * PLATFORM_BONE_KEYFRAME_SIZE;
                let start = frame_index as usize * frame_size;

                // Decode each bone
                let mut matrices = Vec::with_capacity(kf.bone_count as usize);
                for i in 0..kf.bone_count as usize {
                    let kf_offset = start + i * PLATFORM_BONE_KEYFRAME_SIZE;
                    let kf_bytes = &kf.data[kf_offset..kf_offset + PLATFORM_BONE_KEYFRAME_SIZE];

                    // Parse and decode platform keyframe
                    let platform_kf = PlatformBoneKeyframe::from_bytes(kf_bytes);
                    let transform = decode_bone_transform(&platform_kf);

                    // Convert BoneTransform to 3x4 matrix
                    let matrix = bone_transform_to_matrix(&transform);
                    matrices.push(matrix);
                }

                matrices
            }
            None => {
                warn!(
                    "keyframe_bind: handle {} not found (only {} loaded)",
                    handle,
                    state.keyframes.len()
                );
                return;
            }
        }
    };

    // Set bone matrices
    let bone_count = bone_matrices.len() as u32;
    let state = &mut caller.data_mut().console;
    state.bone_matrices = bone_matrices;
    state.bone_count = bone_count;

    tracing::trace!(
        "keyframe_bind: bound frame {} from handle {} ({} bones)",
        frame_index,
        handle,
        bone_count
    );
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
