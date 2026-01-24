//! Rendering state management methods for ZXFFIState

use super::{SkeletonData, ZXFFIState};

impl ZXFFIState {
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

    /// Add current shading state to the pool if dirty, returning its index
    ///
    /// Uses deduplication via StatePool - if this exact state already exists, returns existing index.
    /// Otherwise adds a new entry.
    pub fn add_shading_state(&mut self) -> crate::graphics::ShadingStateIndex {
        // Sync animation state before checking (Animation System v2)
        self.sync_animation_state();

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
