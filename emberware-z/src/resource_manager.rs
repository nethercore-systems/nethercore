//! Resource manager for Emberware Z
//!
//! Manages the mapping between game resource handles (u32) and
//! graphics backend handles (TextureHandle, MeshHandle).

use crate::graphics::{
    pack_color_rgba_unorm8, pack_normal_snorm16, pack_position_f16, pack_uv_f16,
    vertex_stride_packed, MeshHandle, TextureHandle, ZGraphics, FORMAT_COLOR, FORMAT_NORMAL,
    FORMAT_SKINNED, FORMAT_UV,
};
use crate::state::ZFFIState;
use bytemuck::cast_slice;
use emberware_core::console::{Audio, ConsoleResourceManager};

/// Resource manager for Emberware Z
///
/// Manages the mapping between game resource handles (u32) and
/// graphics backend handles (TextureHandle, MeshHandle).
pub struct ZResourceManager {
    /// Mapping from game texture handles to graphics texture handles
    pub texture_map: hashbrown::HashMap<u32, TextureHandle>,
    /// Mapping from game mesh handles to graphics mesh handles
    pub mesh_map: hashbrown::HashMap<u32, MeshHandle>,
}

impl ZResourceManager {
    /// Create a new resource manager
    pub fn new() -> Self {
        Self {
            texture_map: hashbrown::HashMap::new(),
            mesh_map: hashbrown::HashMap::new(),
        }
    }

    /// Convert unpacked f32 vertex data to packed format (f16, snorm16, unorm8)
    ///
    /// This ensures all GPU uploads use packed formats for 37.5% memory savings.
    fn pack_vertex_data(data: &[f32], format: u8) -> Vec<u8> {
        let has_uv = format & FORMAT_UV != 0;
        let has_color = format & FORMAT_COLOR != 0;
        let has_normal = format & FORMAT_NORMAL != 0;
        let has_skinning = format & FORMAT_SKINNED != 0;

        // Calculate unpacked stride (how many f32s per vertex)
        let mut f32_stride = 3; // Position (x, y, z)
        if has_uv {
            f32_stride += 2; // UV (u, v)
        }
        if has_color {
            f32_stride += 3; // Color (r, g, b) - alpha added as 1.0
        }
        if has_normal {
            f32_stride += 3; // Normal (nx, ny, nz)
        }
        if has_skinning {
            f32_stride += 9; // 4 bone indices (as f32) + 4 weights + padding (?)
                             // NOTE: Skinning layout needs verification
        }

        let vertex_count = data.len() / f32_stride;
        let packed_stride = vertex_stride_packed(format) as usize;
        let mut packed = Vec::with_capacity(vertex_count * packed_stride);

        for i in 0..vertex_count {
            let base = i * f32_stride;
            let mut offset = base;

            // Position: f32x3 → f16x4 (8 bytes)
            let pos = pack_position_f16(data[offset], data[offset + 1], data[offset + 2]);
            packed.extend_from_slice(cast_slice(&pos));
            offset += 3;

            // UV: f32x2 → f16x2 (4 bytes)
            if has_uv {
                let uv = pack_uv_f16(data[offset], data[offset + 1]);
                packed.extend_from_slice(cast_slice(&uv));
                offset += 2;
            }

            // Color: f32x3 → unorm8x4 (4 bytes, alpha=255)
            if has_color {
                let color = pack_color_rgba_unorm8(
                    data[offset],
                    data[offset + 1],
                    data[offset + 2],
                    1.0,
                );
                packed.extend_from_slice(cast_slice(&color));
                offset += 3;
            }

            // Normal: f32x3 → snorm16x4 (8 bytes)
            if has_normal {
                let normal = pack_normal_snorm16(data[offset], data[offset + 1], data[offset + 2]);
                packed.extend_from_slice(cast_slice(&normal));
                offset += 3;
            }

            // Skinning: Keep as-is (not packed)
            if has_skinning {
                // TODO: Implement skinning data packing when skinning is used
                // For now, this is a placeholder
                tracing::warn!("Skinning data packing not yet implemented");
            }
        }

        packed
    }
}

impl ConsoleResourceManager for ZResourceManager {
    type Graphics = ZGraphics;
    type State = ZFFIState;

    fn process_pending_resources(
        &mut self,
        graphics: &mut Self::Graphics,
        _audio: &mut dyn Audio,
        state: &mut Self::State,
    ) {
        // Process pending textures
        for pending in state.pending_textures.drain(..) {
            match graphics.load_texture(pending.width, pending.height, &pending.data) {
                Ok(handle) => {
                    self.texture_map.insert(pending.handle, handle);
                    tracing::debug!(
                        "Loaded texture: game_handle={} -> graphics_handle={:?}",
                        pending.handle,
                        handle
                    );
                }
                Err(e) => {
                    tracing::warn!("Failed to load texture {}: {}", pending.handle, e);
                }
            }
        }

        // Process pending unpacked meshes (f32 convenience API)
        // Convert to packed format before GPU upload for 37.5% memory savings
        for pending in state.pending_meshes.drain(..) {
            // Convert f32 vertex data to packed bytes
            let packed_data = Self::pack_vertex_data(&pending.vertex_data, pending.format);

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
    }

    fn execute_draw_commands(&mut self, graphics: &mut Self::Graphics, state: &mut Self::State) {
        // Process draw commands - ZGraphics consumes draw commands directly
        graphics.process_draw_commands(state, &self.texture_map);
    }
}
