//! GPU buffer upload logic for frame rendering
//!
//! This module handles uploading per-frame data to GPU buffers:
//! - Vertex and index data from command buffer
//! - Unified transforms (model, view, projection matrices)
//! - Shading states
//! - MVP indices with absolute offsets
//! - Bone matrices for animation

use super::super::ZXGraphics;
use super::super::vertex::VERTEX_FORMAT_COUNT;
use crate::state::ZXFFIState;
use std::time::Instant;

impl ZXGraphics {
    /// Upload vertex and index data from command buffer to GPU buffers.
    /// Returns the elapsed time in nanoseconds if perf tracking is enabled.
    pub(super) fn upload_immediate_buffers(&mut self, perf_enabled: bool) -> Option<u64> {
        let t0 = perf_enabled.then(Instant::now);

        for format in 0..VERTEX_FORMAT_COUNT as u8 {
            let vertex_data = self.command_buffer.vertex_data(format);
            if !vertex_data.is_empty() {
                // Vertex data is already packed at record time.
                self.buffer_manager
                    .vertex_buffer_mut(format)
                    .ensure_capacity(&self.device, &self.queue, vertex_data.len() as u64);
                self.buffer_manager
                    .vertex_buffer(format)
                    .write_at(&self.queue, 0, vertex_data);
            }

            let index_data = self.command_buffer.index_data(format);
            if !index_data.is_empty() {
                let index_bytes: &[u8] = bytemuck::cast_slice(index_data);
                self.buffer_manager
                    .index_buffer_mut(format)
                    .ensure_capacity(&self.device, &self.queue, index_bytes.len() as u64);
                self.buffer_manager
                    .index_buffer(format)
                    .write_at(&self.queue, 0, index_bytes);
            }
        }

        t0.map(|t| t.elapsed().as_nanos() as u64)
    }

    /// Upload unified transforms (model, view, projection matrices) to GPU buffer.
    /// Returns the elapsed time in nanoseconds if perf tracking is enabled.
    pub(super) fn upload_transforms(
        &mut self,
        z_state: &ZXFFIState,
        perf_enabled: bool,
    ) -> Option<u64> {
        let model_count = z_state.model_matrices.len();
        let view_count = z_state.view_matrices.len();
        let proj_count = z_state.proj_matrices.len();
        let total_transforms = model_count + view_count + proj_count;

        if total_transforms == 0 {
            return None;
        }

        let t0 = perf_enabled.then(Instant::now);
        self.ensure_unified_transforms_capacity(total_transforms);

        // Write models, then views, then projs directly (avoids per-frame staging alloc).
        let model_bytes: &[u8] = bytemuck::cast_slice(&z_state.model_matrices);
        let view_bytes: &[u8] = bytemuck::cast_slice(&z_state.view_matrices);
        let proj_bytes: &[u8] = bytemuck::cast_slice(&z_state.proj_matrices);

        self.queue
            .write_buffer(&self.unified_transforms_buffer, 0, model_bytes);
        self.queue.write_buffer(
            &self.unified_transforms_buffer,
            model_bytes.len() as u64,
            view_bytes,
        );
        self.queue.write_buffer(
            &self.unified_transforms_buffer,
            (model_bytes.len() + view_bytes.len()) as u64,
            proj_bytes,
        );

        t0.map(|t| t.elapsed().as_nanos() as u64)
    }

    /// Upload shading states to GPU buffer.
    /// Returns the elapsed time in nanoseconds if perf tracking is enabled.
    pub(super) fn upload_shading_states(
        &mut self,
        z_state: &ZXFFIState,
        perf_enabled: bool,
    ) -> Option<u64> {
        if z_state.shading_pool.is_empty() {
            return None;
        }

        let t0 = perf_enabled.then(Instant::now);
        self.ensure_shading_state_buffer_capacity(z_state.shading_pool.len());
        let data = bytemuck::cast_slice(z_state.shading_pool.as_slice());
        self.queue.write_buffer(&self.shading_state_buffer, 0, data);

        t0.map(|t| t.elapsed().as_nanos() as u64)
    }

    /// Upload MVP indices with absolute offsets into unified_transforms buffer.
    /// CPU pre-computes absolute indices so shader does direct lookup without offset arithmetic.
    /// Returns the elapsed time in nanoseconds if perf tracking is enabled.
    pub(super) fn upload_mvp_indices(
        &mut self,
        z_state: &ZXFFIState,
        perf_enabled: bool,
    ) -> Option<u64> {
        let state_count = z_state.mvp_shading_states.len();
        if state_count == 0 {
            return None;
        }

        let t0 = perf_enabled.then(Instant::now);
        self.ensure_mvp_indices_buffer_capacity(state_count);

        // Transform relative indices to absolute indices
        let model_count = z_state.model_matrices.len();
        let view_count = z_state.view_matrices.len();
        let view_offset = model_count as u32;
        let proj_offset = (model_count + view_count) as u32;

        self.mvp_indices_scratch.clear();
        self.mvp_indices_scratch.reserve(state_count);
        for idx in &z_state.mvp_shading_states {
            self.mvp_indices_scratch
                .push(super::super::MvpShadingIndices {
                    model_idx: idx.model_idx,
                    view_idx: idx.view_idx + view_offset,
                    proj_idx: idx.proj_idx + proj_offset,
                    shading_idx: idx.shading_idx,
                });
        }

        let data = bytemuck::cast_slice(self.mvp_indices_scratch.as_slice());
        self.queue.write_buffer(&self.mvp_indices_buffer, 0, data);

        t0.map(|t| t.elapsed().as_nanos() as u64)
    }

    /// Upload bone matrices to unified_animation buffer (dynamic section).
    /// Bones are appended after static data (inverse_bind + keyframes).
    /// Returns the elapsed time in nanoseconds if perf tracking is enabled.
    pub(super) fn upload_bone_matrices(
        &mut self,
        z_state: &ZXFFIState,
        perf_enabled: bool,
    ) -> Option<u64> {
        if z_state.bone_matrices.is_empty() {
            return None;
        }

        let t0 = perf_enabled.then(Instant::now);
        let bone_count = z_state.bone_matrices.len().min(256);
        let bone_bytes: &[u8] = bytemuck::cast_slice(&z_state.bone_matrices[..bone_count]);
        // Write after static sections (inverse_bind + keyframes)
        let byte_offset = (self.animation_static_end * 48) as u64;
        self.queue
            .write_buffer(&self.unified_animation_buffer, byte_offset, bone_bytes);

        t0.map(|t| t.elapsed().as_nanos() as u64)
    }
}
