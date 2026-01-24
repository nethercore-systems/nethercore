//! Keyframe access functions

use anyhow::{Result, bail};
use wasmtime::Caller;

use zx_common::formats::{
    BoneTransform, PLATFORM_BONE_KEYFRAME_SIZE, PlatformBoneKeyframe, decode_bone_transform,
};

use crate::ffi::ZXGameContext;
use crate::state::{BoneMatrix3x4, KeyframeSource};

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
pub(super) fn keyframe_read(
    mut caller: Caller<'_, ZXGameContext>,
    handle: u32,
    index: u32,
    out_ptr: u32,
) -> Result<()> {
    if handle == 0 {
        bail!("keyframe_read: invalid keyframe handle 0");
    }

    // Get keyframe collection
    let (bone_count, frame_data) = {
        let state = &caller.data().ffi;
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

/// Bind a keyframe directly from the static GPU buffer (Animation System)
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
/// # Animation System
/// Unlike the legacy `keyframe_read() -> set_bones()` path, this uses pre-uploaded
/// static keyframe data. The GPU shader reads directly from the all_keyframes buffer
/// at the computed offset.
pub(super) fn keyframe_bind(
    mut caller: Caller<'_, ZXGameContext>,
    handle: u32,
    index: u32,
) -> Result<()> {
    if handle == 0 {
        // Unbind keyframes - reset to default static offset 0
        let state = &mut caller.data_mut().ffi;
        state.current_keyframe_source = KeyframeSource::Static { offset: 0 };
        state.bone_count = 0;
        state.shading_state_dirty = true;
        tracing::trace!("keyframe_bind: unbound (offset 0)");
        return Ok(());
    }

    // Extract values from immutable borrow first
    let (offset, bone_count) = {
        let state = &caller.data().ffi;
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
    let state = &mut caller.data_mut().ffi;
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
/// Note: This function is used by tests but not runtime code since the unified animation buffer
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

        // For 90° rotation around X, the rotation matrix is:
        // [1   0    0]
        // [0   0   -1]  (row 1)
        // [0   1    0]  (row 2)
        //
        // This transforms: Y axis → Z, Z axis → -Y

        // Row 0: X axis unchanged
        assert!((m.row0[0] - 1.0).abs() < 0.001);
        assert!((m.row0[1]).abs() < 0.001);
        assert!((m.row0[2]).abs() < 0.001);

        // Row 1: [0, 0, -1]
        assert!((m.row1[0]).abs() < 0.001);
        assert!((m.row1[1]).abs() < 0.001);
        assert!((m.row1[2] + 1.0).abs() < 0.001); // -1

        // Row 2: [0, 1, 0]
        assert!((m.row2[0]).abs() < 0.001);
        assert!((m.row2[1] - 1.0).abs() < 0.001); // +1
        assert!((m.row2[2]).abs() < 0.001);
    }
}
