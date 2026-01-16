//! Rendering state management methods for ZXFFIState

use super::{SkeletonData, ZXFFIState};
use crate::graphics::unified_shading_state::GradientConfig;

impl ZXFFIState {
    /// Update sky colors in current environment state (backwards compatibility)
    ///
    /// Both colors are 0xRRGGBBAA format.
    /// Maps to env_gradient_set() internally for Multi-Environment v4 compatibility.
    /// Ground colors are derived as darker versions of sky colors.
    pub fn update_sky_colors(&mut self, horizon_rgba: u32, zenith_rgba: u32) {
        use crate::graphics::{blend_mode, env_mode};

        // Create ground colors by darkening the sky colors
        let darken = |color: u32| -> u32 {
            let r = ((color >> 24) & 0xFF) * 6 / 10;
            let g = ((color >> 16) & 0xFF) * 6 / 10;
            let b = ((color >> 8) & 0xFF) * 6 / 10;
            let a = color & 0xFF;
            (r << 24) | (g << 16) | (b << 8) | a
        };

        let ground_horizon = darken(horizon_rgba);
        let nadir = darken(zenith_rgba);

        // Set up gradient mode
        self.current_environment_state
            .set_base_mode(env_mode::GRADIENT);
        self.current_environment_state
            .set_overlay_mode(env_mode::GRADIENT);
        self.current_environment_state
            .set_blend_mode(blend_mode::ALPHA);

        // Pack gradient colors
        self.current_environment_state
            .pack_gradient(GradientConfig {
                offset: 0, // base mode offset
                zenith: zenith_rgba,
                sky_horizon: horizon_rgba,
                ground_horizon,
                nadir,
                rotation: 0.0,
                shift: 0.0,
                sun_elevation: 0.0,
                sun_disk: 0,
                sun_halo: 0,
                sun_intensity: 0,
                horizon_haze: 0,
                sun_warmth: 0,
                cloudiness: 0,
                cloud_phase: 0,
            });

        self.environment_dirty = true;
    }

    /// Sync animation state (Animation System v2 - Unified Buffer) to current_shading_state
    ///
    /// Computes absolute keyframe_base into unified_animation buffer:
    /// - Static keyframes: inverse_bind_end + section_offset
    /// - Immediate bones: animation_static_end + offset
    ///
    /// Also copies current_inverse_bind_base (already absolute since inverse_bind is at offset 0).
    /// Called before add_shading_state().
    pub fn sync_animation_state(&mut self) {
        use super::KeyframeSource;

        // Compute absolute keyframe_base into unified_animation buffer
        let keyframe_base = match self.current_keyframe_source {
            // Static keyframes are at [inverse_bind_end..animation_static_end)
            KeyframeSource::Static { offset } => self.inverse_bind_end + offset,
            // Immediate bones are at [animation_static_end..)
            KeyframeSource::Immediate { offset } => self.animation_static_end + offset,
        };

        // Update shading state if changed
        if self.current_shading_state.keyframe_base != keyframe_base {
            self.current_shading_state.keyframe_base = keyframe_base;
            self.shading_state_dirty = true;
        }

        if self.current_shading_state.inverse_bind_base != self.current_inverse_bind_base {
            self.current_shading_state.inverse_bind_base = self.current_inverse_bind_base;
            self.shading_state_dirty = true;
        }

        // Note: _pad field is unused - shader uses unified_animation with pre-computed offsets
    }

    /// Check if a skeleton is currently bound (inverse bind mode enabled)
    pub fn is_skeleton_bound(&self) -> bool {
        self.bound_skeleton != 0
    }

    /// Get the currently bound skeleton data, if any
    pub fn get_bound_skeleton(&self) -> Option<&SkeletonData> {
        if self.bound_skeleton == 0 {
            return None;
        }
        let index = self.bound_skeleton as usize - 1;
        self.skeletons.get(index)
    }

    /// Add current environment state to the pool if dirty, returning its index
    ///
    /// Uses deduplication via StatePool - if this exact state already exists, returns existing index.
    /// Otherwise adds a new entry.
    pub fn add_environment_state(&mut self) -> crate::graphics::EnvironmentIndex {
        // If not dirty, return the last added state
        if !self.environment_dirty && !self.environment_pool.is_empty() {
            return self
                .environment_pool
                .last_index()
                .unwrap_or(crate::graphics::EnvironmentIndex(0));
        }

        // Add to pool (handles deduplication and overflow internally)
        let env_idx = self.environment_pool.add(self.current_environment_state);
        self.environment_dirty = false;

        env_idx
    }

    /// Add current shading state to the pool if dirty, returning its index
    ///
    /// Uses deduplication via StatePool - if this exact state already exists, returns existing index.
    /// Otherwise adds a new entry.
    /// Also syncs the current environment state index into the shading state.
    pub fn add_shading_state(&mut self) -> crate::graphics::ShadingStateIndex {
        // Sync animation state before checking (Animation System v2)
        self.sync_animation_state();

        // Sync environment state (Multi-Environment v4)
        let env_idx = self.add_environment_state();
        if self.current_shading_state.environment_index != env_idx.0 {
            self.current_shading_state.environment_index = env_idx.0;
            self.shading_state_dirty = true;
        }

        // If not dirty, return the last added state
        if !self.shading_state_dirty && !self.shading_pool.is_empty() {
            return self
                .shading_pool
                .last_index()
                .unwrap_or(crate::graphics::ShadingStateIndex(0));
        }

        // Add to pool (handles deduplication and overflow internally)
        let shading_idx = self.shading_pool.add(self.current_shading_state);
        self.shading_state_dirty = false;

        shading_idx
    }

    /// Add current MVP matrices + shading state to combined pool, returning buffer index
    ///
    /// Uses lazy allocation and deduplication - only allocates when draws happen.
    /// Similar to add_shading_state() but for combined MVP+shading state.
    pub fn add_mvp_shading_state(&mut self) -> u32 {
        // First, ensure shading state is added
        let shading_idx = self.add_shading_state();

        // Get or push model matrix: Some = pending (push it), None = use last in pool
        let model_idx = if let Some(mat) = self.current_model_matrix.take() {
            self.model_matrices.push(mat);
            (self.model_matrices.len() - 1) as u32
        } else {
            (self.model_matrices.len() - 1) as u32
        };

        // Get or push view matrix
        let view_idx = if let Some(mat) = self.current_view_matrix.take() {
            self.view_matrices.push(mat);
            (self.view_matrices.len() - 1) as u32
        } else {
            (self.view_matrices.len() - 1) as u32
        };

        // Get or push projection matrix
        let proj_idx = if let Some(mat) = self.current_proj_matrix.take() {
            self.proj_matrices.push(mat);
            (self.proj_matrices.len() - 1) as u32
        } else {
            (self.proj_matrices.len() - 1) as u32
        };

        // Create unpacked indices struct (no bit-packing!)
        let indices = crate::graphics::MvpShadingIndices {
            model_idx,
            view_idx,
            proj_idx,
            shading_idx: shading_idx.0,
        };

        // Check if this exact combination already exists
        if let Some(&existing_idx) = self.mvp_shading_map.get(&indices) {
            return existing_idx;
        }

        // Add new combined state
        let buffer_idx = self.mvp_shading_states.len() as u32;
        if buffer_idx >= 65536 {
            self.mvp_shading_overflow_count = self.mvp_shading_overflow_count.saturating_add(1);
            if !self.mvp_shading_overflowed_this_frame {
                self.mvp_shading_overflowed_this_frame = true;
                tracing::error!(
                    "MVP+Shading state pool overflow (max 65,536 unique states per frame). Dropping new unique MVP+Shading states for this frame."
                );
            }
            // Fallback: reuse the last valid index rather than crashing the player.
            return self.mvp_shading_states.len().saturating_sub(1) as u32;
        }

        self.mvp_shading_states.push(indices);
        self.mvp_shading_map.insert(indices, buffer_idx);

        buffer_idx
    }
}
