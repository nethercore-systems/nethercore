//! Buffer capacity management for dynamic GPU buffers
//!
//! Handles automatic growth of storage buffers when more space is needed.
//! Buffers grow by powers of two to minimize reallocation frequency.

use super::super::ZXGraphics;
use glam::Mat4;

impl ZXGraphics {
    /// Ensure unified transforms buffer has sufficient capacity
    pub(super) fn ensure_unified_transforms_capacity(&mut self, count: usize) {
        if count <= self.unified_transforms_capacity {
            return;
        }

        let new_capacity = (count * 2).next_power_of_two();
        tracing::debug!(
            "Growing unified transforms buffer: {} → {}",
            self.unified_transforms_capacity,
            new_capacity
        );

        self.unified_transforms_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Unified Transforms (@binding(0))"),
            size: (new_capacity * std::mem::size_of::<Mat4>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.unified_transforms_capacity = new_capacity;
        self.invalidate_frame_bind_group();
    }

    /// Ensure MVP indices buffer has sufficient capacity
    pub(super) fn ensure_mvp_indices_buffer_capacity(&mut self, count: usize) {
        if count <= self.mvp_indices_capacity {
            return;
        }

        let new_capacity = (count * 2).next_power_of_two();
        tracing::debug!(
            "Growing MVP indices buffer: {} → {}",
            self.mvp_indices_capacity,
            new_capacity
        );

        self.mvp_indices_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("MVP Indices"),
            size: (new_capacity * 2 * std::mem::size_of::<u32>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.mvp_indices_capacity = new_capacity;
    }

    /// Ensure shading state buffer has sufficient capacity
    pub(super) fn ensure_shading_state_buffer_capacity(&mut self, count: usize) {
        if count <= self.shading_state_capacity {
            return;
        }

        let new_capacity = (count * 2).next_power_of_two();
        tracing::debug!(
            "Growing shading state buffer: {} → {}",
            self.shading_state_capacity,
            new_capacity
        );

        self.shading_state_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Shading States"),
            size: (new_capacity * std::mem::size_of::<super::super::PackedUnifiedShadingState>())
                as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.shading_state_capacity = new_capacity;
    }

    /// Ensure environment state buffer has sufficient capacity (Multi-Environment v4)
    pub(super) fn ensure_environment_states_buffer_capacity(&mut self, count: usize) {
        if count <= self.environment_states_capacity {
            return;
        }

        let new_capacity = (count * 2).next_power_of_two();
        tracing::debug!(
            "Growing environment state buffer: {} → {}",
            self.environment_states_capacity,
            new_capacity
        );

        self.environment_states_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Environment States"),
            size: (new_capacity * std::mem::size_of::<super::super::PackedEnvironmentState>())
                as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.environment_states_capacity = new_capacity;
    }
}
