//! Resource management and frame cleanup methods for ZXFFIState

use super::{DEFAULT_Z_INDEX, ZXFFIState};
use crate::state::QuadBatch;
use glam::{Mat4, Vec3};

impl ZXFFIState {
    /// Add a quad instance to the appropriate batch (auto-batches by texture and viewport)
    ///
    /// This automatically groups quads by texture, viewport, z-index, and pass to minimize draw calls.
    /// When bound_textures, current_viewport, z_index, or pass_id changes, a new batch is created.
    pub fn add_quad_instance(&mut self, instance: crate::graphics::QuadInstance, z_index: u32) {
        // Determine if this is a screen-space quad (2D)
        let is_screen_space = instance.mode == crate::graphics::QuadMode::ScreenSpace as u32;

        // Check if we can add to the current batch or need a new one
        if let Some(last_batch) = self.quad_batches.last_mut()
            && last_batch.textures == self.bound_textures
            && last_batch.is_screen_space == is_screen_space
            && last_batch.viewport == self.current_viewport
            && last_batch.pass_id == self.current_pass_id
            && last_batch.z_index == z_index
        {
            // Same textures, mode, viewport, pass, and z_index - add to current batch
            last_batch.instances.push(instance);
            return;
        }

        // Need a new batch (first batch, textures changed, mode changed, viewport changed, pass changed, or z_index changed)
        self.quad_batches.push(QuadBatch {
            is_screen_space,
            textures: self.bound_textures,
            instances: vec![instance],
            viewport: self.current_viewport,
            pass_id: self.current_pass_id,
            z_index,
        });
    }

    /// Clear all per-frame commands and reset for next frame
    ///
    /// Called once per frame in app.rs after render_frame() completes.
    /// This is the centralized cleanup point for all per-frame resources.
    ///
    /// This clears only the resources that accumulate per-frame:
    /// - render_pass (immediate draw commands)
    /// - model_matrices (per-draw transforms)
    /// - deferred_commands (billboards, sprites, text, environment)
    ///
    /// Note: Audio playback state is in ZRollbackState, not here.
    ///
    /// One-time init resources (pending_textures, pending_meshes) are NOT cleared here.
    /// They are drained once after init() in app.rs and never accumulate again.
    pub fn clear_frame(&mut self) {
        self.render_pass.reset();

        // Clear matrix pools and re-add defaults
        self.model_matrices.clear();
        self.model_matrices.push(Mat4::IDENTITY); // Re-add identity matrix at index 0

        self.view_matrices.clear();
        self.view_matrices.push(Mat4::look_at_rh(
            Vec3::new(0.0, 0.0, 5.0),
            Vec3::ZERO,
            Vec3::Y,
        )); // Re-add default view matrix

        self.proj_matrices.clear();
        self.proj_matrices.push(Mat4::perspective_rh(
            45.0_f32.to_radians(),
            16.0 / 9.0,
            0.1,
            1000.0,
        )); // Re-add default projection matrix

        // Reset current MVP state to defaults
        self.current_model_matrix = None; // Will use last in pool (IDENTITY)
        self.current_view_matrix = None; // Will use last in pool (default view)
        self.current_proj_matrix = None; // Will use last in pool (default proj)

        // Clear combined MVP+shading state pool
        self.mvp_shading_states.clear();
        self.mvp_shading_map.clear();
        self.mvp_shading_overflowed_this_frame = false;
        self.mvp_shading_overflow_count = 0;

        // Reset shading state pool for next frame
        self.shading_pool.clear();
        self.shading_state_dirty = true; // Mark dirty so first draw creates state 0

        // Clear GPU-instanced quad batches for next frame
        self.quad_batches.clear();

        // Clear immediate bone matrices for next frame
        // The bone_matrices buffer accumulates during the frame and must be reset
        self.bone_matrices.clear();

        // Reset render state to defaults each frame (immediate-mode consistency)
        self.cull_mode = crate::graphics::CullMode::None;
        self.texture_filter = crate::graphics::TextureFilter::Nearest;
        self.current_z_index = DEFAULT_Z_INDEX; // Reset z-index to background
        self.current_viewport = crate::graphics::Viewport::FULLSCREEN; // Reset viewport to fullscreen

        // Reset render pass system - pass 0 is always the default pass
        self.current_pass_id = 0;
        self.pass_configs.clear();
        self.pass_configs
            .push(crate::graphics::PassConfig::default());

        // Clear EPU per-frame requests
        self.epu_frame_config = None;
        self.epu_frame_draws.clear();

        // Note: color and shading state already rebuild each frame via add_shading_state()
    }
}
