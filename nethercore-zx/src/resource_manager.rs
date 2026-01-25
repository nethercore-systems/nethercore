//! Resource manager for Nethercore ZX
//!
//! Manages the mapping between game resource handles (u32) and
//! graphics backend handles (TextureHandle, MeshHandle).

use crate::graphics::epu::{EpuConfig, RampParams, epu_begin, epu_finish};
use crate::graphics::{MeshHandle, TextureHandle, ZXGraphics, pack_vertex_data};
use crate::state::{
    BoneMatrix3x4, KeyframeGpuInfo, LoadedKeyframeCollection, SkeletonData, SkeletonGpuInfo,
    ZXFFIState,
};
use nethercore_core::console::{Audio, ConsoleResourceManager};
use zx_common::formats::{
    BoneTransform, PLATFORM_BONE_KEYFRAME_SIZE, PlatformBoneKeyframe, decode_bone_transform,
};

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

/// Default environment configuration for the resource manager.
///
/// A simple cyan sky with gray walls and dark floor. This is used as a fallback
/// when games don't specify their own environment configuration via `epu_set()` / `epu_set_env()`.
///
/// Format: Layer 0 is a RAMP enclosure, layers 1-7 are empty.
/// For preset examples showing full EPU capabilities, see the epu-showcase example.
fn default_environment() -> EpuConfig {
    use glam::Vec3;

    let mut e = epu_begin();

    // Simple enclosure: cyan sky, gray walls, dark gray floor
    e.ramp_enclosure(RampParams {
        up: Vec3::Y,
        wall_color: [128, 128, 128], // gray walls
        sky_color: [100, 200, 220],  // cyan sky
        floor_color: [64, 64, 64],   // dark gray floor
        ceil_q: 10,                  // ceiling threshold
        floor_q: 5,                  // floor threshold
        softness: 180,               // soft transitions
    });

    // Layers 1-7 remain as NOP (empty [0, 0]) from epu_begin()

    epu_finish(e)
}

/// Resource manager for Nethercore ZX
///
/// Manages the mapping between game resource handles (u32) and
/// graphics backend handles (TextureHandle, MeshHandle).
pub struct ZResourceManager {
    /// Mapping from game texture handles to graphics texture handles
    pub texture_map: hashbrown::HashMap<u32, TextureHandle>,
    /// Mapping from game mesh handles to graphics mesh handles
    pub mesh_map: hashbrown::HashMap<u32, MeshHandle>,
}

impl Default for ZResourceManager {
    fn default() -> Self {
        Self {
            texture_map: hashbrown::HashMap::new(),
            mesh_map: hashbrown::HashMap::new(),
        }
    }
}

impl ZResourceManager {
    /// Create a new resource manager
    pub fn new() -> Self {
        Self::default()
    }
}

impl ConsoleResourceManager for ZResourceManager {
    type Graphics = ZXGraphics;
    type State = ZXFFIState;

    fn process_pending_resources(
        &mut self,
        graphics: &mut Self::Graphics,
        _audio: &mut dyn Audio,
        state: &mut Self::State,
    ) {
        // Process pending textures (RGBA8 or BC7)
        for pending in state.pending_textures.drain(..) {
            let result = graphics.load_texture_with_format(
                pending.width,
                pending.height,
                &pending.data,
                pending.format,
            );
            match result {
                Ok(handle) => {
                    self.texture_map.insert(pending.handle, handle);
                    tracing::debug!(
                        "Loaded texture: game_handle={} -> graphics_handle={:?} ({:?})",
                        pending.handle,
                        handle,
                        pending.format,
                    );
                }
                Err(e) => {
                    tracing::warn!("Failed to load texture {}: {}", pending.handle, e);
                }
            }
        }

        // Register built-in texture handles (font, white)
        // These are reserved handles used by draw_text and draw_rect
        self.texture_map.insert(u32::MAX, graphics.white_texture());
        self.texture_map
            .insert(u32::MAX - 1, graphics.font_texture());

        // Process pending unpacked meshes (f32 convenience API)
        // Convert to packed format before GPU upload for 37.5% memory savings
        for pending in state.pending_meshes.drain(..) {
            // Convert f32 vertex data to packed bytes
            let packed_data = pack_vertex_data(&pending.vertex_data, pending.format);

            let result = if let Some(ref indices) = pending.index_data {
                graphics.load_mesh_indexed_packed(&packed_data, indices, pending.format)
            } else {
                graphics.load_mesh_packed(&packed_data, pending.format)
            };

            match result {
                Ok(handle) => {
                    self.mesh_map.insert(pending.handle, handle);

                    // Also store RetainedMesh metadata in state.mesh_map for FFI access
                    if let Some(retained_mesh) = graphics.get_mesh(handle) {
                        state.mesh_map.insert(pending.handle, retained_mesh.clone());
                    }

                    tracing::debug!(
                        "Loaded mesh (f32→packed): game_handle={} -> graphics_handle={:?}",
                        pending.handle,
                        handle
                    );
                }
                Err(e) => {
                    tracing::warn!("Failed to load mesh {}: {}", pending.handle, e);
                }
            }
        }

        // Process pending packed meshes (procedural generation, power users)
        for pending in state.pending_meshes_packed.drain(..) {
            let result = if let Some(ref indices) = pending.index_data {
                graphics.load_mesh_indexed_packed(&pending.vertex_data, indices, pending.format)
            } else {
                graphics.load_mesh_packed(&pending.vertex_data, pending.format)
            };

            match result {
                Ok(handle) => {
                    self.mesh_map.insert(pending.handle, handle);

                    // Also store RetainedMesh metadata in state.mesh_map for FFI access
                    if let Some(retained_mesh) = graphics.get_mesh(handle) {
                        state.mesh_map.insert(pending.handle, retained_mesh.clone());
                    }

                    tracing::debug!(
                        "Loaded mesh (packed): game_handle={} -> graphics_handle={:?}",
                        pending.handle,
                        handle
                    );
                }
                Err(e) => {
                    tracing::warn!("Failed to load packed mesh {}: {}", pending.handle, e);
                }
            }
        }

        // Process pending skeletons (move to finalized storage)
        // Skeletons are stored by handle order (handle N is at index N-1)
        let had_skeletons = !state.pending_skeletons.is_empty();
        for pending in state.pending_skeletons.drain(..) {
            // Ensure skeletons vec is large enough for this handle
            let index = pending.handle as usize - 1;
            while state.skeletons.len() <= index {
                // Fill gaps with empty skeletons (shouldn't happen in practice)
                state.skeletons.push(SkeletonData {
                    inverse_bind: Vec::new(),
                    bone_count: 0,
                });
            }

            state.skeletons[index] = SkeletonData {
                inverse_bind: pending.inverse_bind,
                bone_count: pending.bone_count,
            };

            tracing::debug!(
                "Loaded skeleton: handle={} with {} bones",
                pending.handle,
                pending.bone_count
            );
        }

        // Process pending keyframes (move to finalized storage)
        // Keyframes are stored by handle order (handle N is at index N-1)
        let had_keyframes = !state.pending_keyframes.is_empty();
        for pending in state.pending_keyframes.drain(..) {
            let index = pending.handle as usize - 1;
            while state.keyframes.len() <= index {
                // Fill gaps with empty keyframe collections (shouldn't happen in practice)
                state.keyframes.push(LoadedKeyframeCollection {
                    bone_count: 0,
                    frame_count: 0,
                    data: Vec::new(),
                });
            }

            state.keyframes[index] = LoadedKeyframeCollection {
                bone_count: pending.bone_count,
                frame_count: pending.frame_count,
                data: pending.data,
            };

            tracing::debug!(
                "Loaded keyframes: handle={} with {} bones, {} frames",
                pending.handle,
                pending.bone_count,
                pending.frame_count
            );
        }

        // =====================================================================
        // Animation system: Upload static animation data to GPU
        // =====================================================================

        // Upload all skeleton inverse bind matrices to GPU
        if had_skeletons {
            let mut all_inverse_bind: Vec<BoneMatrix3x4> = Vec::new();

            // Ensure gpu_info vec is large enough
            while state.skeleton_gpu_info.len() < state.skeletons.len() {
                state.skeleton_gpu_info.push(SkeletonGpuInfo::default());
            }

            for (i, skeleton) in state.skeletons.iter().enumerate() {
                if skeleton.bone_count == 0 {
                    continue; // Skip empty placeholders
                }

                let offset = all_inverse_bind.len() as u32;
                state.skeleton_gpu_info[i] = SkeletonGpuInfo {
                    inverse_bind_offset: offset,
                    bone_count: skeleton.bone_count as u8,
                };

                all_inverse_bind.extend_from_slice(&skeleton.inverse_bind);
            }

            if !all_inverse_bind.is_empty() {
                graphics.upload_static_inverse_bind(&all_inverse_bind);
                // Sync inverse_bind_end to state for offset computation
                state.inverse_bind_end = graphics.inverse_bind_end as u32;
            }
        }

        // If we had skeletons but no keyframes, animation_static_end = inverse_bind_end
        // IMPORTANT: Must sync to graphics.animation_static_end so frame.rs uploads
        // immediate bones to the correct offset that matches what the shader expects
        if had_skeletons && !had_keyframes {
            state.animation_static_end = state.inverse_bind_end;
            graphics.animation_static_end = graphics.inverse_bind_end;
        }

        // Decode and upload all keyframe bone matrices to GPU
        if had_keyframes {
            let mut all_keyframes: Vec<BoneMatrix3x4> = Vec::new();

            // Ensure gpu_info vec is large enough
            while state.keyframe_gpu_info.len() < state.keyframes.len() {
                state.keyframe_gpu_info.push(KeyframeGpuInfo::default());
            }

            for (i, kf) in state.keyframes.iter().enumerate() {
                if kf.frame_count == 0 {
                    continue; // Skip empty placeholders
                }

                let base_offset = all_keyframes.len() as u32;
                state.keyframe_gpu_info[i] = KeyframeGpuInfo {
                    keyframe_base_offset: base_offset,
                    bone_count: kf.bone_count,
                    frame_count: kf.frame_count,
                };

                // Decode all frames for this animation
                for frame_idx in 0..kf.frame_count as usize {
                    let frame_start =
                        frame_idx * kf.bone_count as usize * PLATFORM_BONE_KEYFRAME_SIZE;

                    for bone_idx in 0..kf.bone_count as usize {
                        let kf_offset = frame_start + bone_idx * PLATFORM_BONE_KEYFRAME_SIZE;
                        let kf_bytes = &kf.data[kf_offset..kf_offset + PLATFORM_BONE_KEYFRAME_SIZE];

                        // Decode platform keyframe to BoneTransform
                        let platform_kf = PlatformBoneKeyframe::from_bytes(kf_bytes);
                        let transform = decode_bone_transform(&platform_kf);

                        // Convert to BoneMatrix3x4
                        let matrix = bone_transform_to_matrix(&transform);
                        all_keyframes.push(matrix);
                    }
                }
            }

            if !all_keyframes.is_empty() {
                graphics.upload_static_keyframes(&all_keyframes);
                // Sync animation_static_end to state for offset computation
                state.animation_static_end = graphics.animation_static_end as u32;
            } else {
                // No keyframes loaded - animation_static_end = inverse_bind_end
                // IMPORTANT: Must sync to graphics.animation_static_end so frame.rs uploads
                // immediate bones to the correct offset that matches what the shader expects
                state.animation_static_end = state.inverse_bind_end;
                graphics.animation_static_end = graphics.inverse_bind_end;
            }
        }

        // Invalidate frame bind group cache if buffers may have changed
        if had_skeletons || had_keyframes {
            graphics.invalidate_frame_bind_group();
        }

        // Apply init config to graphics (render mode from game's init() phase)
        // Resolution is fixed at 540p
        graphics.set_render_mode(state.init_config.render_mode);
        graphics.update_resolution();
    }

    fn execute_draw_commands(&mut self, graphics: &mut Self::Graphics, state: &mut Self::State) {
        // Process draw commands - ZXGraphics consumes draw commands directly
        graphics.process_draw_commands(state, &self.texture_map);
    }

    fn render_game_to_target(
        &self,
        graphics: &mut Self::Graphics,
        encoder: &mut wgpu::CommandEncoder,
        state: &Self::State,
        clear_color: [f32; 4],
    ) {
        // =====================================================================
        // EPU Compute Dispatch: Generate environment maps before rendering
        // =====================================================================
        //
        // The EPU (Environment Processing Unit) generates EnvRadiance (mip pyramid)
        // and SH9 for active environments. This must happen before
        // render_frame() so the textures are valid for sampling during rendering.

        // Collect active environment IDs from shading states used this frame.
        // This keeps the EPU compute workload proportional to what the frame actually references.
        let env_ids: Vec<u32> = state
            .shading_pool
            .iter()
            .map(|s| s.environment_index)
            .collect();
        let active = crate::graphics::epu::collect_active_envs(&env_ids);

        if !active.unique_ids.is_empty() {
            // Push-only API: epu_set(...) / epu_set_env(...) provide configs for one or more env_ids.
            // If no config is provided, fall back to the built-in default environment.
            let default_config: EpuConfig = state
                .epu_frame_configs
                .get(&0)
                .copied()
                .unwrap_or_else(default_environment);

            // Build all active env_ids with their own config when present, falling back to env_id=0.
            let mut config_refs: Vec<(u32, &EpuConfig)> =
                Vec::with_capacity(active.unique_ids.len());
            for &env_id in &active.unique_ids {
                let config = state
                    .epu_frame_configs
                    .get(&env_id)
                    .unwrap_or(&default_config);
                config_refs.push((env_id, config));
            }

            // Dispatch EPU compute shaders
            graphics.build_epu_environments(encoder, &config_refs);
        }

        // =====================================================================
        // Main Render Pass
        // =====================================================================
        graphics.render_frame(encoder, state, &self.texture_map, clear_color);
    }
}
