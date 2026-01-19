//! Retained mesh storage and loading
//!
//! Handles mesh upload and storage for retained mode rendering.

use super::growable_buffer::GrowableBuffer;
use crate::graphics::vertex::{
    VERTEX_FORMAT_COUNT, VertexFormatInfo, vertex_stride, vertex_stride_packed,
};
use anyhow::Result;
use std::borrow::Cow;

/// Handle to a retained mesh
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MeshHandle(pub u32);

/// Stored mesh data for retained mode drawing
#[derive(Debug, Clone)]
pub struct RetainedMesh {
    /// Vertex format flags
    pub format: u8,
    /// Number of vertices
    pub vertex_count: u32,
    /// Number of indices (0 for non-indexed)
    pub index_count: u32,
    /// Byte offset into the format's vertex buffer
    pub vertex_offset: u64,
    /// Byte offset into the format's index buffer (if indexed)
    pub index_offset: u64,
}

/// Mesh loading and management operations
///
/// This is separated from BufferManager to keep mesh-specific logic isolated.
pub struct MeshLoader<'a> {
    retained_vertex_buffers: &'a mut [GrowableBuffer; VERTEX_FORMAT_COUNT],
    retained_index_buffers: &'a mut [GrowableBuffer; VERTEX_FORMAT_COUNT],
    next_mesh_id: &'a mut u32,
}

impl<'a> MeshLoader<'a> {
    /// Create a new mesh loader
    pub(super) fn new(
        retained_vertex_buffers: &'a mut [GrowableBuffer; VERTEX_FORMAT_COUNT],
        retained_index_buffers: &'a mut [GrowableBuffer; VERTEX_FORMAT_COUNT],
        next_mesh_id: &'a mut u32,
    ) -> Self {
        Self {
            retained_vertex_buffers,
            retained_index_buffers,
            next_mesh_id,
        }
    }

    /// Load a non-indexed mesh (retained mode)
    ///
    /// Convenience wrapper that accepts unpacked f32 vertex data and packs it internally.
    /// For procedural meshes or power users with pre-packed data, use load_mesh_packed() instead.
    ///
    /// The mesh is stored in the appropriate vertex buffer based on format.
    /// Returns a MeshHandle and RetainedMesh for storage.
    pub fn load_mesh(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        data: &[f32],
        format: u8,
    ) -> Result<(MeshHandle, RetainedMesh)> {
        use zx_common::pack_vertex_data;

        let format_idx = format as usize;
        if format_idx >= VERTEX_FORMAT_COUNT {
            anyhow::bail!("Invalid vertex format: {}", format);
        }

        // Validate unpacked stride
        let unpacked_stride_floats = (vertex_stride(format) / 4) as usize;
        if !data.len().is_multiple_of(unpacked_stride_floats) {
            anyhow::bail!(
                "Vertex data size {} is not a multiple of stride {} floats",
                data.len(),
                unpacked_stride_floats
            );
        }

        let vertex_count = data.len() / unpacked_stride_floats;

        // Pack f32 data to GPU format (f16/snorm16/unorm8)
        let packed_data = pack_vertex_data(data, format);

        // Ensure retained buffer has capacity
        self.retained_vertex_buffers[format_idx].ensure_capacity(
            device,
            queue,
            packed_data.len() as u64,
        );

        // Write packed data to retained buffer
        let vertex_offset = self.retained_vertex_buffers[format_idx].write(queue, &packed_data);

        // Create mesh handle
        let handle = MeshHandle(*self.next_mesh_id);
        *self.next_mesh_id += 1;

        let mesh = RetainedMesh {
            format,
            vertex_count: vertex_count as u32,
            index_count: 0,
            vertex_offset,
            index_offset: 0,
        };

        tracing::debug!(
            "Loaded mesh {}: {} vertices, format {} (f32→packed)",
            handle.0,
            vertex_count,
            VertexFormatInfo::for_format(format).name
        );

        Ok((handle, mesh))
    }

    /// Load an indexed mesh (retained mode)
    ///
    /// Convenience wrapper that accepts unpacked f32 vertex data and packs it internally.
    /// For procedural meshes or power users with pre-packed data, use load_mesh_indexed_packed() instead.
    ///
    /// The mesh is stored in the appropriate vertex and index buffers based on format.
    /// Returns a MeshHandle and RetainedMesh for storage.
    pub fn load_mesh_indexed(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        data: &[f32],
        indices: &[u16],
        format: u8,
    ) -> Result<(MeshHandle, RetainedMesh)> {
        use zx_common::pack_vertex_data;

        let format_idx = format as usize;
        if format_idx >= VERTEX_FORMAT_COUNT {
            anyhow::bail!("Invalid vertex format: {}", format);
        }

        // Validate unpacked stride
        let unpacked_stride_floats = (vertex_stride(format) / 4) as usize;
        if !data.len().is_multiple_of(unpacked_stride_floats) {
            anyhow::bail!(
                "Vertex data size {} is not a multiple of stride {} floats",
                data.len(),
                unpacked_stride_floats
            );
        }

        let vertex_count = data.len() / unpacked_stride_floats;

        // Pack f32 data to GPU format (f16/snorm16/unorm8)
        let packed_data = pack_vertex_data(data, format);

        // Ensure retained vertex buffer has capacity
        self.retained_vertex_buffers[format_idx].ensure_capacity(
            device,
            queue,
            packed_data.len() as u64,
        );

        // Ensure retained index buffer has capacity
        // Pad index data to 4-byte alignment for wgpu COPY_BUFFER_ALIGNMENT (only if needed)
        let index_byte_data: &[u8] = bytemuck::cast_slice(indices);
        let index_data_to_write: Cow<[u8]> = if index_byte_data.len().is_multiple_of(4) {
            Cow::Borrowed(index_byte_data)
        } else {
            let padded_len = (index_byte_data.len() + 3) & !3;
            let mut padded = index_byte_data.to_vec();
            padded.resize(padded_len, 0);
            Cow::Owned(padded)
        };

        self.retained_index_buffers[format_idx].ensure_capacity(
            device,
            queue,
            index_data_to_write.len() as u64,
        );

        // Write to retained buffers (packed vertex data + indices)
        let vertex_offset = self.retained_vertex_buffers[format_idx].write(queue, &packed_data);
        let index_offset =
            self.retained_index_buffers[format_idx].write(queue, &index_data_to_write);

        // Create mesh handle
        let handle = MeshHandle(*self.next_mesh_id);
        *self.next_mesh_id += 1;

        let mesh = RetainedMesh {
            format,
            vertex_count: vertex_count as u32,
            index_count: indices.len() as u32,
            vertex_offset,
            index_offset,
        };

        tracing::debug!(
            "Loaded indexed mesh {}: {} vertices, {} indices, format {} (f32→packed)",
            handle.0,
            vertex_count,
            indices.len(),
            VertexFormatInfo::for_format(format).name
        );

        Ok((handle, mesh))
    }

    /// Load a packed mesh (retained mode)
    ///
    /// The mesh data is already packed (f16/snorm16/unorm8 format).
    /// The mesh is stored in the appropriate vertex buffer based on format.
    /// Returns a MeshHandle and RetainedMesh for storage.
    pub fn load_mesh_packed(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        data: &[u8],
        format: u8,
    ) -> Result<(MeshHandle, RetainedMesh)> {
        let format_idx = format as usize;
        if format_idx >= VERTEX_FORMAT_COUNT {
            anyhow::bail!("Invalid vertex format: {}", format);
        }

        let stride = vertex_stride_packed(format) as usize;
        let vertex_count = data.len() / stride;

        if !data.len().is_multiple_of(stride) {
            anyhow::bail!(
                "Vertex data size {} is not a multiple of stride {}",
                data.len(),
                stride
            );
        }

        // Ensure retained buffer has capacity
        self.retained_vertex_buffers[format_idx].ensure_capacity(device, queue, data.len() as u64);

        // Write to retained buffer
        let vertex_offset = self.retained_vertex_buffers[format_idx].write(queue, data);

        // Create mesh handle
        let handle = MeshHandle(*self.next_mesh_id);
        *self.next_mesh_id += 1;

        let mesh = RetainedMesh {
            format,
            vertex_count: vertex_count as u32,
            index_count: 0,
            vertex_offset,
            index_offset: 0,
        };

        tracing::debug!(
            "Loaded PACKED mesh {}: {} vertices, format {}",
            handle.0,
            vertex_count,
            VertexFormatInfo::for_format(format).name
        );

        Ok((handle, mesh))
    }

    /// Load a packed indexed mesh (retained mode)
    ///
    /// The mesh data is already packed (f16/snorm16/unorm8 format).
    /// The mesh is stored in the appropriate vertex and index buffers based on format.
    /// Returns a MeshHandle and RetainedMesh for storage.
    pub fn load_mesh_indexed_packed(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        data: &[u8],
        indices: &[u16],
        format: u8,
    ) -> Result<(MeshHandle, RetainedMesh)> {
        let format_idx = format as usize;
        if format_idx >= VERTEX_FORMAT_COUNT {
            anyhow::bail!("Invalid vertex format: {}", format);
        }

        let stride = vertex_stride_packed(format) as usize;
        let vertex_count = data.len() / stride;

        if !data.len().is_multiple_of(stride) {
            anyhow::bail!(
                "Vertex data size {} is not a multiple of stride {}",
                data.len(),
                stride
            );
        }

        // Ensure retained vertex buffer has capacity
        self.retained_vertex_buffers[format_idx].ensure_capacity(device, queue, data.len() as u64);

        // Ensure retained index buffer has capacity
        // Pad index data to 4-byte alignment for wgpu COPY_BUFFER_ALIGNMENT (only if needed)
        let index_byte_data: &[u8] = bytemuck::cast_slice(indices);
        let index_data_to_write: Cow<[u8]> = if index_byte_data.len().is_multiple_of(4) {
            Cow::Borrowed(index_byte_data)
        } else {
            let padded_len = (index_byte_data.len() + 3) & !3;
            let mut padded = index_byte_data.to_vec();
            padded.resize(padded_len, 0);
            Cow::Owned(padded)
        };

        self.retained_index_buffers[format_idx].ensure_capacity(
            device,
            queue,
            index_data_to_write.len() as u64,
        );

        // Write to retained buffers
        let vertex_offset = self.retained_vertex_buffers[format_idx].write(queue, data);
        let index_offset =
            self.retained_index_buffers[format_idx].write(queue, &index_data_to_write);

        // Create mesh handle
        let handle = MeshHandle(*self.next_mesh_id);
        *self.next_mesh_id += 1;

        let mesh = RetainedMesh {
            format,
            vertex_count: vertex_count as u32,
            index_count: indices.len() as u32,
            vertex_offset,
            index_offset,
        };

        tracing::debug!(
            "Loaded PACKED indexed mesh {}: {} vertices, {} indices, format {}",
            handle.0,
            vertex_count,
            indices.len(),
            VertexFormatInfo::for_format(format).name
        );

        Ok((handle, mesh))
    }
}
