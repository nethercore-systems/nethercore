//! Resource manager for Emberware Z
//!
//! Manages the mapping between game resource handles (u32) and
//! graphics backend handles (TextureHandle, MeshHandle).

use crate::graphics::{MeshHandle, TextureHandle, ZGraphics};
use crate::state::ZFFIState;
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

        // Process pending meshes
        for pending in state.pending_meshes.drain(..) {
            let result = if let Some(ref indices) = pending.index_data {
                graphics.load_mesh_indexed(&pending.vertex_data, indices, pending.format)
            } else {
                graphics.load_mesh(&pending.vertex_data, pending.format)
            };

            match result {
                Ok(handle) => {
                    self.mesh_map.insert(pending.handle, handle);

                    // Also store RetainedMesh metadata in state.mesh_map for FFI access
                    if let Some(retained_mesh) = graphics.get_mesh(handle) {
                        state.mesh_map.insert(pending.handle, retained_mesh.clone());
                    }

                    tracing::debug!(
                        "Loaded mesh: game_handle={} -> graphics_handle={:?}",
                        pending.handle,
                        handle
                    );
                }
                Err(e) => {
                    tracing::warn!("Failed to load mesh {}: {}", pending.handle, e);
                }
            }
        }
    }

    fn execute_draw_commands(&mut self, graphics: &mut Self::Graphics, state: &mut Self::State) {
        // Process draw commands - ZGraphics consumes draw commands directly
        graphics.process_draw_commands(state, &self.texture_map);
    }
}
