//! Draw command processing
//!
//! This module handles processing draw commands from ZFFIState and converting
//! them into GPU rendering operations.

use super::ZGraphics;
use super::render_state::{BlendMode, CullMode, MatcapBlendMode, TextureHandle};

impl ZGraphics {
    /// Process all draw commands from ZFFIState and execute them
    ///
    /// This method consumes draw commands from the ZFFIState and executes them
    /// on the GPU, directly translating FFI state into graphics calls without
    /// an intermediate unpacking/repacking step.
    ///
    /// This replaces the previous execute_draw_commands() function in app.rs,
    /// eliminating redundant data translation and simplifying the architecture.
    pub fn process_draw_commands(
        &mut self,
        z_state: &mut crate::state::ZFFIState,
        texture_map: &hashbrown::HashMap<u32, TextureHandle>,
    ) {
        // Note: render mode is set once after init() in App::flush_post_init_resources()
        // No need to set it every frame

        // Note: texture_filter sync removed - filter is now per-draw via
        // PackedUnifiedShadingState.flags (bit 1) and sample_filtered() shader helper

        // 1. Swap the FFI-populated render pass into our command buffer
        // This efficiently transfers all immediate geometry (triangles, meshes)
        // without copying vectors. The old command buffer (now in z_state.render_pass)
        // will be cleared when z_state.clear_frame() is called.
        std::mem::swap(&mut self.command_buffer, &mut z_state.render_pass);

        // NOTE: Texture handle remapping was removed here.
        // Mesh/IndexedMesh commands now capture FFI texture handles at command creation time
        // (stored in `textures: [u32; 4]`), which are resolved to TextureHandle at render
        // time in frame.rs via texture_map. This fixes the bug where deferred remapping
        // used stale bound_textures state at frame end.
        //
        // Quad commands already work correctly - they capture textures via QuadBatch
        // and resolve them when creating VRPCommand::Quad in process_draw_commands().

        // Ensure default shading state exists for deferred commands.
        // Deferred commands (billboards, sprites, text) use ShadingStateIndex(0) by default.
        // If the game only uses deferred drawing and never calls draw_mesh/draw_triangles,
        // the shading state pool would be empty (cleared by clear_frame), causing panics
        // during command sorting/rendering when accessing state 0.
        //
        // This ensures state 0 always exists, using the current render state defaults
        // (color, blend mode, material properties, etc.) from z_state.current_shading_state.
        if z_state.shading_states.is_empty() {
            z_state.add_shading_state();
        }

        // 1.5. Process GPU-instanced quads (billboards, sprites)
        // Accumulate all instances and upload once, then create batched draw commands
        if !z_state.quad_batches.is_empty() {
            let total_instances: usize =
                z_state.quad_batches.iter().map(|b| b.instances.len()).sum();

            // Accumulate all instances into one buffer and track batch offsets
            let mut all_instances = Vec::with_capacity(total_instances);
            let mut batch_info = Vec::new(); // (base_instance, instance_count, textures, blend_mode)

            for batch in &z_state.quad_batches {
                if batch.instances.is_empty() {
                    continue;
                }

                let base_instance = all_instances.len() as u32;
                all_instances.extend_from_slice(&batch.instances);
                batch_info.push((
                    base_instance,
                    batch.instances.len() as u32,
                    batch.textures,
                    batch.blend_mode,
                ));
            }

            // Upload all instances once to GPU
            if !all_instances.is_empty() {
                self.buffer_manager
                    .upload_quad_instances(&self.device, &self.queue, &all_instances)
                    .expect("Failed to upload quad instances to GPU");
            }

            // Create draw commands for each batch with correct base_instance
            for (base_instance, instance_count, textures, batch_blend_mode) in batch_info {
                // Map FFI texture handles to graphics texture handles for this batch
                let texture_slots = [
                    texture_map
                        .get(&textures[0])
                        .copied()
                        .unwrap_or(TextureHandle::INVALID),
                    texture_map
                        .get(&textures[1])
                        .copied()
                        .unwrap_or(TextureHandle::INVALID),
                    texture_map
                        .get(&textures[2])
                        .copied()
                        .unwrap_or(TextureHandle::INVALID),
                    texture_map
                        .get(&textures[3])
                        .copied()
                        .unwrap_or(TextureHandle::INVALID),
                ];

                // Note: Quad instances contain their own shading_state_index in the instance data.
                // BufferSource::Quad has no buffer_index - quads read transforms and shading from instance data.
                // blend_mode comes from the batch (captured when quads were created), not current z_state
                self.command_buffer
                    .add_command(super::command_buffer::VRPCommand::Quad {
                        base_vertex: self.unit_quad_base_vertex,
                        first_index: self.unit_quad_first_index,
                        base_instance,
                        instance_count,
                        texture_slots,
                        blend_mode: BlendMode::from_u8(batch_blend_mode),
                        depth_test: z_state.depth_test,
                        cull_mode: CullMode::from_u8(z_state.cull_mode),
                    });
            }
        }

        // Note: All per-frame cleanup (model_matrices, audio_commands, render_pass)
        // happens AFTER render_frame completes in app.rs via z_state.clear_frame()
        // This keeps cleanup centralized and ensures matrices survive until GPU upload
    }

    /// Convert game matcap blend mode to graphics matcap blend mode
    #[allow(dead_code)] // Useful conversion helper
    pub(super) fn convert_matcap_blend_mode(mode: u8) -> MatcapBlendMode {
        match mode {
            0 => MatcapBlendMode::Multiply,
            1 => MatcapBlendMode::Add,
            2 => MatcapBlendMode::HsvModulate,
            _ => MatcapBlendMode::Multiply,
        }
    }

    /// Map game texture handles to graphics texture handles
    #[allow(dead_code)] // Useful conversion helper
    pub(super) fn map_texture_handles(
        texture_map: &hashbrown::HashMap<u32, TextureHandle>,
        bound_textures: &[u32; 4],
    ) -> [TextureHandle; 4] {
        let mut texture_slots = [TextureHandle::INVALID; 4];
        for (slot, &game_handle) in bound_textures.iter().enumerate() {
            if game_handle != 0 {
                if let Some(&graphics_handle) = texture_map.get(&game_handle) {
                    texture_slots[slot] = graphics_handle;
                }
            }
        }
        texture_slots
    }

    /// Convert game cull mode to graphics cull mode
    #[allow(dead_code)] // Useful conversion helper
    pub(super) fn convert_cull_mode(mode: u8) -> CullMode {
        match mode {
            0 => CullMode::None,
            1 => CullMode::Back,
            2 => CullMode::Front,
            _ => CullMode::None,
        }
    }

    /// Convert game blend mode to graphics blend mode
    #[allow(dead_code)] // Useful conversion helper
    pub(super) fn convert_blend_mode(mode: u8) -> BlendMode {
        match mode {
            0 => BlendMode::None,
            1 => BlendMode::Alpha,
            2 => BlendMode::Additive,
            3 => BlendMode::Multiply,
            _ => BlendMode::None,
        }
    }
}
