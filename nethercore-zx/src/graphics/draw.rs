//! Draw command processing
//!
//! This module handles processing draw commands from ZXFFIState and converting
//! them into GPU rendering operations.

use super::ZXGraphics;
use super::render_state::TextureHandle;

impl ZXGraphics {
    /// Process all draw commands from ZXFFIState and execute them
    ///
    /// This method consumes draw commands from the ZXFFIState and executes them
    /// on the GPU, directly translating FFI state into graphics calls without
    /// an intermediate unpacking/repacking step.
    ///
    /// This replaces the previous execute_draw_commands() function in app.rs,
    /// eliminating redundant data translation and simplifying the architecture.
    pub fn process_draw_commands(
        &mut self,
        z_state: &mut crate::state::ZXFFIState,
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
        if z_state.shading_pool.is_empty() {
            z_state.add_shading_state();
        }

        // 1.5. Process GPU-instanced quads (billboards, sprites)
        // Accumulate all instances and upload once, then create batched draw commands
        if !z_state.quad_batches.is_empty() {
            let total_instances: usize =
                z_state.quad_batches.iter().map(|b| b.instances.len()).sum();

            // Compute absolute offsets for view/proj matrices in unified_transforms
            // Layout: [models | views | projs]
            let view_offset = z_state.model_matrices.len() as u32;
            let proj_offset = (z_state.model_matrices.len() + z_state.view_matrices.len()) as u32;

            // Resolution is fixed at 540p (index 1) - pack into mode field (bits 8-9)
            let resolution_index: u32 = 1;

            // Clear and reuse scratch buffers (avoids per-frame allocation)
            self.quad_instance_scratch.clear();
            self.quad_batch_scratch.clear();

            // Reserve capacity only if needed (buffers grow but never shrink)
            if self.quad_instance_scratch.capacity() < total_instances {
                self.quad_instance_scratch
                    .reserve(total_instances - self.quad_instance_scratch.capacity());
            }
            if self.quad_batch_scratch.capacity() < z_state.quad_batches.len() {
                self.quad_batch_scratch
                    .reserve(z_state.quad_batches.len() - self.quad_batch_scratch.capacity());
            }

            for batch in &z_state.quad_batches {
                if batch.instances.is_empty() {
                    continue;
                }

                let base_instance = self.quad_instance_scratch.len() as u32;

                // Transform each instance:
                // - Convert logical view_index to absolute indices
                // - Pack resolution_index into mode (bits 8-9)
                self.quad_instance_scratch
                    .extend(batch.instances.iter().map(|instance| {
                        let mut transformed = *instance;
                        let logical_view_index = transformed.view_index;
                        transformed.view_index = view_offset + logical_view_index;
                        transformed.proj_index = proj_offset + logical_view_index;
                        // Pack resolution_index into bits 8-9 of mode
                        transformed.mode |= (resolution_index & 0x3) << 8;
                        transformed
                    }));

                self.quad_batch_scratch.push((
                    base_instance,
                    batch.instances.len() as u32,
                    batch.textures,
                    batch.is_screen_space,
                    batch.viewport,
                    batch.stencil_mode,
                    batch.layer,
                ));
            }

            // Upload all instances once to GPU
            if !self.quad_instance_scratch.is_empty() {
                self.buffer_manager
                    .upload_quad_instances(&self.device, &self.queue, &self.quad_instance_scratch)
                    .expect("Failed to upload quad instances to GPU");
            }

            // Create draw commands for each batch with correct base_instance
            for &(
                base_instance,
                instance_count,
                textures,
                is_screen_space,
                viewport,
                stencil_mode,
                layer,
            ) in &self.quad_batch_scratch
            {
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
                // Screen-space quads (2D) rely on layer-based sorting, not depth testing.
                // World-space quads (3D billboards) use depth testing for 3D occlusion.
                self.command_buffer
                    .add_command(super::command_buffer::VRPCommand::Quad {
                        base_vertex: self.unit_quad_base_vertex,
                        first_index: self.unit_quad_first_index,
                        base_instance,
                        instance_count,
                        texture_slots,
                        depth_test: !is_screen_space && z_state.depth_test,
                        cull_mode: z_state.cull_mode,
                        viewport,
                        stencil_mode,
                        layer,
                    });
            }
        }

        // Note: All per-frame cleanup (model_matrices, audio_commands, render_pass)
        // happens AFTER render_frame completes in app.rs via z_state.clear_frame()
        // This keeps cleanup centralized and ensures matrices survive until GPU upload
    }
}
