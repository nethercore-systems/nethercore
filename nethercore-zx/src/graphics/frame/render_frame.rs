//! Main frame rendering logic
//!
//! This module orchestrates the frame rendering process by coordinating:
//! - Performance metrics collection (perf_tracking)
//! - GPU buffer uploads (buffer_upload)
//! - Frame bind group management (frame_bind_group)
//! - Render pass execution (pass_execution)

use super::super::TextureHandleTable;
use super::super::ZXGraphics;
use super::super::command_buffer::VRPCommand;
use std::time::Instant;

impl ZXGraphics {
    pub fn render_frame(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        z_state: &crate::state::ZXFFIState,
        texture_table: &TextureHandleTable,
        clear_color: [f32; 4],
    ) {
        let perf_enabled = self.perf.enabled();
        let perf_frame_t0 = perf_enabled.then(Instant::now);

        // Collect performance metrics if enabled
        if perf_enabled {
            self.collect_frame_perf_metrics();
        }

        // If no commands, just clear render target
        // (blit is handled separately via blit_to_window())
        if self.command_buffer.commands().is_empty() {
            self.execute_clear_pass(encoder, clear_color);
            if let Some(t0) = perf_frame_t0 {
                self.perf.render_frame_ns = self
                    .perf
                    .render_frame_ns
                    .wrapping_add(t0.elapsed().as_nanos() as u64);
                self.perf.maybe_log();
            }
            return;
        }

        // Upload vertex/index data from command buffer to GPU buffers
        if let Some(elapsed) = self.upload_immediate_buffers(perf_enabled) {
            self.perf.upload_immediate_ns = self.perf.upload_immediate_ns.wrapping_add(elapsed);
        }

        // Sort draw commands IN-PLACE by CommandSortKey to minimize state changes
        // Commands are reset at the start of next frame, so no need to preserve original order
        let sort_t0 = perf_enabled.then(Instant::now);
        // Sort order: pass_id -> viewport -> z_index -> render_type -> cull -> textures
        self.command_buffer
            .commands_mut()
            .sort_unstable_by_key(VRPCommand::sort_key);
        if let Some(t0) = sort_t0 {
            self.perf.sort_ns = self
                .perf
                .sort_ns
                .wrapping_add(t0.elapsed().as_nanos() as u64);
        }

        // =================================================================
        // UNIFIED BUFFER UPLOADS
        // =================================================================

        // 1. Upload unified transforms: [models | views | projs]
        if let Some(elapsed) = self.upload_transforms(z_state, perf_enabled) {
            self.perf.upload_transforms_ns = self.perf.upload_transforms_ns.wrapping_add(elapsed);
        }

        // 2. Upload shading states
        if let Some(elapsed) = self.upload_shading_states(z_state, perf_enabled) {
            self.perf.upload_shading_ns = self.perf.upload_shading_ns.wrapping_add(elapsed);
        }

        // 3. Upload MVP + shading indices with ABSOLUTE offsets into unified_transforms
        if let Some(elapsed) = self.upload_mvp_indices(z_state, perf_enabled) {
            self.perf.upload_mvp_ns = self.perf.upload_mvp_ns.wrapping_add(elapsed);
        }

        // 4. Upload bone matrices to unified_animation (dynamic section)
        if let Some(elapsed) = self.upload_bone_matrices(z_state, perf_enabled) {
            self.perf.upload_bones_ns = self.perf.upload_bones_ns.wrapping_add(elapsed);
        }

        // NOTE: Inverse bind matrices are uploaded once during init via upload_static_inverse_bind()
        // They live in unified_animation[0..inverse_bind_end]

        // Take texture cache out temporarily to avoid nested mutable borrows during render pass.
        // Cache is persistent across frames - entries are reused when keys match.
        let mut texture_bind_groups = std::mem::take(&mut self.texture_bind_groups);

        // Create or reuse cached frame bind group (same for all draws)
        let frame_bind_group = match self.get_or_create_frame_bind_group(z_state) {
            Some(bg) => bg,
            None => {
                // No commands to render (should not happen since we checked above)
                return;
            }
        };

        // Execute render passes
        let encode_t0 = perf_enabled.then(Instant::now);
        self.execute_render_passes(
            encoder,
            z_state,
            texture_table,
            clear_color,
            &frame_bind_group,
            &mut texture_bind_groups,
            perf_enabled,
        );
        if let Some(t0) = encode_t0 {
            self.perf.encode_ns = self
                .perf
                .encode_ns
                .wrapping_add(t0.elapsed().as_nanos() as u64);
        }

        // Move texture cache back into self (preserving allocations for next frame)
        self.texture_bind_groups = texture_bind_groups;

        // NOTE: Blit is handled separately via blit_to_window()
        // This allows us to re-blit the last rendered frame on high refresh rate monitors
        // without re-rendering the game content

        if let Some(t0) = perf_frame_t0 {
            self.perf.render_frame_ns = self
                .perf
                .render_frame_ns
                .wrapping_add(t0.elapsed().as_nanos() as u64);
            self.perf.maybe_log();
        }
    }
}
