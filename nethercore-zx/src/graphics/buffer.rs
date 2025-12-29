//! GPU buffer management for vertex and index data
//!
//! Provides auto-growing buffers for efficient GPU memory management
//! and retained mesh storage.

use std::borrow::Cow;

use hashbrown::HashMap;

use anyhow::Result;

use super::vertex::{VERTEX_FORMAT_COUNT, VertexFormatInfo, vertex_stride, vertex_stride_packed};

/// Initial buffer size (64KB)
const INITIAL_BUFFER_SIZE: u64 = 64 * 1024;

/// Growth factor when buffer needs to expand (2x)
const BUFFER_GROWTH_FACTOR: u64 = 2;

/// Auto-growing GPU buffer for vertex/index data
///
/// Grows dynamically during init phase when more data is needed.
/// Avoids frequent reallocation by doubling capacity on growth.
pub struct GrowableBuffer {
    /// The wgpu buffer
    buffer: wgpu::Buffer,
    /// Buffer usage flags
    usage: wgpu::BufferUsages,
    /// Current capacity in bytes
    capacity: u64,
    /// Current used size in bytes
    used: u64,
    /// Debug label
    label: String,
}

impl GrowableBuffer {
    /// Create a new growable buffer with initial capacity
    pub fn new(device: &wgpu::Device, usage: wgpu::BufferUsages, label: &str) -> Self {
        let buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(label),
            size: INITIAL_BUFFER_SIZE,
            usage: usage | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        Self {
            buffer,
            usage,
            capacity: INITIAL_BUFFER_SIZE,
            used: 0,
            label: label.to_string(),
        }
    }

    /// Ensure the buffer has enough capacity for additional bytes
    ///
    /// If the buffer needs to grow, creates a new larger buffer and returns true.
    /// The old buffer contents are NOT preserved (call this before writing).
    pub fn ensure_capacity(&mut self, device: &wgpu::Device, additional_bytes: u64) -> bool {
        let required = self.used + additional_bytes;
        if required <= self.capacity {
            return false;
        }

        // Calculate new capacity (at least double, or enough for required)
        let mut new_capacity = self.capacity * BUFFER_GROWTH_FACTOR;
        while new_capacity < required {
            new_capacity *= BUFFER_GROWTH_FACTOR;
        }

        tracing::debug!(
            "Growing buffer '{}': {} -> {} bytes",
            self.label,
            self.capacity,
            new_capacity
        );

        // Create new buffer
        self.buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some(&self.label),
            size: new_capacity,
            usage: self.usage | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.capacity = new_capacity;
        // Reset used since data needs to be re-uploaded
        self.used = 0;

        true
    }

    /// Write data to the buffer at the current position
    ///
    /// Returns the byte offset where data was written.
    /// Panics if there's not enough capacity (call ensure_capacity first).
    pub fn write(&mut self, queue: &wgpu::Queue, data: &[u8]) -> u64 {
        let offset = self.used;
        assert!(
            offset + data.len() as u64 <= self.capacity,
            "Buffer overflow: {} + {} > {}",
            offset,
            data.len(),
            self.capacity
        );

        queue.write_buffer(&self.buffer, offset, data);
        self.used += data.len() as u64;

        offset
    }

    /// Write data to the buffer at a specific offset
    ///
    /// Updates the used counter if this write extends past the current end.
    /// Panics if offset + data.len > capacity.
    pub fn write_at(&self, queue: &wgpu::Queue, offset: u64, data: &[u8]) {
        assert!(
            offset + data.len() as u64 <= self.capacity,
            "Buffer overflow: {} + {} > {}",
            offset,
            data.len(),
            self.capacity
        );
        queue.write_buffer(&self.buffer, offset, data);
    }

    /// Reset the used counter (for per-frame immediate mode buffers)
    pub fn reset(&mut self) {
        self.used = 0;
    }

    /// Get the underlying wgpu buffer, if it exists
    pub fn buffer(&self) -> Option<&wgpu::Buffer> {
        Some(&self.buffer)
    }

    /// Get current used bytes
    pub fn used(&self) -> u64 {
        self.used
    }

    /// Get current capacity in bytes
    pub fn capacity(&self) -> u64 {
        self.capacity
    }
}

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

/// Manages vertex/index buffers and retained meshes
///
/// Handles buffer allocation, growth, and mesh storage.
pub struct BufferManager {
    /// Per-format vertex buffers for immediate mode (rewritten each frame)
    vertex_buffers: [GrowableBuffer; VERTEX_FORMAT_COUNT],
    /// Per-format index buffers for immediate mode (rewritten each frame)
    index_buffers: [GrowableBuffer; VERTEX_FORMAT_COUNT],
    /// Per-format vertex buffers for retained meshes (uploaded once)
    retained_vertex_buffers: [GrowableBuffer; VERTEX_FORMAT_COUNT],
    /// Per-format index buffers for retained meshes (uploaded once)
    retained_index_buffers: [GrowableBuffer; VERTEX_FORMAT_COUNT],
    /// Retained mesh storage
    retained_meshes: HashMap<u32, RetainedMesh>,
    /// Next mesh ID to assign
    next_mesh_id: u32,
    /// Instance buffer for quad rendering (rewritten each frame)
    quad_instance_buffer: GrowableBuffer,
}

impl BufferManager {
    /// Create a new buffer manager with pre-allocated buffers for all formats
    pub fn new(device: &wgpu::Device) -> Self {
        // Create immediate mode vertex buffers for each format
        let vertex_buffers = std::array::from_fn(|i| {
            GrowableBuffer::new(
                device,
                wgpu::BufferUsages::VERTEX,
                &format!("Immediate Vertex Buffer Format {}", i),
            )
        });

        // Create immediate mode index buffers for each format
        let index_buffers = std::array::from_fn(|i| {
            GrowableBuffer::new(
                device,
                wgpu::BufferUsages::INDEX,
                &format!("Immediate Index Buffer Format {}", i),
            )
        });

        // Create retained mesh vertex buffers for each format
        let retained_vertex_buffers = std::array::from_fn(|i| {
            GrowableBuffer::new(
                device,
                wgpu::BufferUsages::VERTEX,
                &format!("Retained Vertex Buffer Format {}", i),
            )
        });

        // Create retained mesh index buffers for each format
        let retained_index_buffers = std::array::from_fn(|i| {
            GrowableBuffer::new(
                device,
                wgpu::BufferUsages::INDEX,
                &format!("Retained Index Buffer Format {}", i),
            )
        });

        // Create instance buffer for quad rendering (storage buffer for GPU lookup)
        let quad_instance_buffer = GrowableBuffer::new(
            device,
            wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            "Quad Instance Storage Buffer",
        );

        Self {
            vertex_buffers,
            index_buffers,
            retained_vertex_buffers,
            retained_index_buffers,
            retained_meshes: HashMap::new(),
            next_mesh_id: 1, // 0 is reserved for INVALID
            quad_instance_buffer,
        }
    }

    /// Load a non-indexed mesh (retained mode)
    ///
    /// Convenience wrapper that accepts unpacked f32 vertex data and packs it internally.
    /// For procedural meshes or power users with pre-packed data, use load_mesh_packed() instead.
    ///
    /// The mesh is stored in the appropriate vertex buffer based on format.
    /// Returns a MeshHandle for use with draw_mesh().
    pub fn load_mesh(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        data: &[f32],
        format: u8,
    ) -> Result<MeshHandle> {
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
        self.retained_vertex_buffers[format_idx].ensure_capacity(device, packed_data.len() as u64);

        // Write packed data to retained buffer
        let vertex_offset = self.retained_vertex_buffers[format_idx].write(queue, &packed_data);

        // Create mesh handle
        let handle = MeshHandle(self.next_mesh_id);
        self.next_mesh_id += 1;

        self.retained_meshes.insert(
            handle.0,
            RetainedMesh {
                format,
                vertex_count: vertex_count as u32,
                index_count: 0,
                vertex_offset,
                index_offset: 0,
            },
        );

        tracing::debug!(
            "Loaded mesh {}: {} vertices, format {} (f32→packed)",
            handle.0,
            vertex_count,
            VertexFormatInfo::for_format(format).name
        );

        Ok(handle)
    }

    /// Load an indexed mesh (retained mode)
    ///
    /// Convenience wrapper that accepts unpacked f32 vertex data and packs it internally.
    /// For procedural meshes or power users with pre-packed data, use load_mesh_indexed_packed() instead.
    ///
    /// The mesh is stored in the appropriate vertex and index buffers based on format.
    /// Returns a MeshHandle for use with draw_mesh().
    pub fn load_mesh_indexed(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        data: &[f32],
        indices: &[u16],
        format: u8,
    ) -> Result<MeshHandle> {
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
        self.retained_vertex_buffers[format_idx].ensure_capacity(device, packed_data.len() as u64);

        // Ensure retained index buffer has capacity
        // Pad index data to 4-byte alignment for wgpu COPY_BUFFER_ALIGNMENT (only if needed)
        let index_byte_data: &[u8] = bytemuck::cast_slice(indices);
        let index_data_to_write: Cow<[u8]> = if index_byte_data.len() % 4 == 0 {
            Cow::Borrowed(index_byte_data)
        } else {
            let padded_len = (index_byte_data.len() + 3) & !3;
            let mut padded = index_byte_data.to_vec();
            padded.resize(padded_len, 0);
            Cow::Owned(padded)
        };

        self.retained_index_buffers[format_idx]
            .ensure_capacity(device, index_data_to_write.len() as u64);

        // Write to retained buffers (packed vertex data + indices)
        let vertex_offset = self.retained_vertex_buffers[format_idx].write(queue, &packed_data);
        let index_offset = self.retained_index_buffers[format_idx].write(queue, &index_data_to_write);

        // Create mesh handle
        let handle = MeshHandle(self.next_mesh_id);
        self.next_mesh_id += 1;

        self.retained_meshes.insert(
            handle.0,
            RetainedMesh {
                format,
                vertex_count: vertex_count as u32,
                index_count: indices.len() as u32,
                vertex_offset,
                index_offset,
            },
        );

        tracing::debug!(
            "Loaded indexed mesh {}: {} vertices, {} indices, format {} (f32→packed)",
            handle.0,
            vertex_count,
            indices.len(),
            VertexFormatInfo::for_format(format).name
        );

        Ok(handle)
    }

    /// Load a packed mesh (retained mode)
    ///
    /// The mesh data is already packed (f16/snorm16/unorm8 format).
    /// The mesh is stored in the appropriate vertex buffer based on format.
    /// Returns a MeshHandle for use with draw_mesh().
    pub fn load_mesh_packed(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        data: &[u8],
        format: u8,
    ) -> Result<MeshHandle> {
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
        self.retained_vertex_buffers[format_idx].ensure_capacity(device, data.len() as u64);

        // Write to retained buffer
        let vertex_offset = self.retained_vertex_buffers[format_idx].write(queue, data);

        // Create mesh handle
        let handle = MeshHandle(self.next_mesh_id);
        self.next_mesh_id += 1;

        self.retained_meshes.insert(
            handle.0,
            RetainedMesh {
                format,
                vertex_count: vertex_count as u32,
                index_count: 0,
                vertex_offset,
                index_offset: 0,
            },
        );

        tracing::debug!(
            "Loaded PACKED mesh {}: {} vertices, format {}",
            handle.0,
            vertex_count,
            VertexFormatInfo::for_format(format).name
        );

        Ok(handle)
    }

    /// Load a packed indexed mesh (retained mode)
    ///
    /// The mesh data is already packed (f16/snorm16/unorm8 format).
    /// The mesh is stored in the appropriate vertex and index buffers based on format.
    /// Returns a MeshHandle for use with draw_mesh().
    pub fn load_mesh_indexed_packed(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        data: &[u8],
        indices: &[u16],
        format: u8,
    ) -> Result<MeshHandle> {
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
        self.retained_vertex_buffers[format_idx].ensure_capacity(device, data.len() as u64);

        // Ensure retained index buffer has capacity
        // Pad index data to 4-byte alignment for wgpu COPY_BUFFER_ALIGNMENT (only if needed)
        let index_byte_data: &[u8] = bytemuck::cast_slice(indices);
        let index_data_to_write: Cow<[u8]> = if index_byte_data.len() % 4 == 0 {
            Cow::Borrowed(index_byte_data)
        } else {
            let padded_len = (index_byte_data.len() + 3) & !3;
            let mut padded = index_byte_data.to_vec();
            padded.resize(padded_len, 0);
            Cow::Owned(padded)
        };

        self.retained_index_buffers[format_idx]
            .ensure_capacity(device, index_data_to_write.len() as u64);

        // Write to retained buffers
        let vertex_offset = self.retained_vertex_buffers[format_idx].write(queue, data);
        let index_offset = self.retained_index_buffers[format_idx].write(queue, &index_data_to_write);

        // Create mesh handle
        let handle = MeshHandle(self.next_mesh_id);
        self.next_mesh_id += 1;

        self.retained_meshes.insert(
            handle.0,
            RetainedMesh {
                format,
                vertex_count: vertex_count as u32,
                index_count: indices.len() as u32,
                vertex_offset,
                index_offset,
            },
        );

        tracing::debug!(
            "Loaded PACKED indexed mesh {}: {} vertices, {} indices, format {}",
            handle.0,
            vertex_count,
            indices.len(),
            VertexFormatInfo::for_format(format).name
        );

        Ok(handle)
    }

    /// Get mesh info by handle
    pub fn get_mesh(&self, handle: MeshHandle) -> Option<&RetainedMesh> {
        self.retained_meshes.get(&handle.0)
    }

    /// Get vertex buffer for a format
    pub fn vertex_buffer(&self, format: u8) -> &GrowableBuffer {
        &self.vertex_buffers[format as usize]
    }

    /// Get mutable vertex buffer for a format
    pub fn vertex_buffer_mut(&mut self, format: u8) -> &mut GrowableBuffer {
        &mut self.vertex_buffers[format as usize]
    }

    /// Get index buffer for a format
    pub fn index_buffer(&self, format: u8) -> &GrowableBuffer {
        &self.index_buffers[format as usize]
    }

    /// Get mutable index buffer for a format
    pub fn index_buffer_mut(&mut self, format: u8) -> &mut GrowableBuffer {
        &mut self.index_buffers[format as usize]
    }

    /// Get retained vertex buffer for a format
    pub fn retained_vertex_buffer(&self, format: u8) -> &GrowableBuffer {
        &self.retained_vertex_buffers[format as usize]
    }

    /// Get retained vertex buffer for a format (mutable)
    pub fn retained_vertex_buffer_mut(&mut self, format: u8) -> &mut GrowableBuffer {
        &mut self.retained_vertex_buffers[format as usize]
    }

    /// Get retained index buffer for a format
    pub fn retained_index_buffer(&self, format: u8) -> &GrowableBuffer {
        &self.retained_index_buffers[format as usize]
    }

    /// Get retained index buffer for a format (mutable)
    pub fn retained_index_buffer_mut(&mut self, format: u8) -> &mut GrowableBuffer {
        &mut self.retained_index_buffers[format as usize]
    }

    /// Upload quad instances to the instance buffer
    ///
    /// Returns Ok(()) on success.
    pub fn upload_quad_instances(
        &mut self,
        device: &wgpu::Device,
        queue: &wgpu::Queue,
        instances: &[crate::graphics::QuadInstance],
    ) -> Result<()> {
        let byte_data = bytemuck::cast_slice(instances);
        self.quad_instance_buffer
            .ensure_capacity(device, byte_data.len() as u64);
        // Reset used before writing new frame data
        self.quad_instance_buffer.reset();
        self.quad_instance_buffer.write(queue, byte_data);
        Ok(())
    }

    /// Get the quad instance buffer
    pub fn quad_instance_buffer(&self) -> &wgpu::Buffer {
        self.quad_instance_buffer
            .buffer()
            .expect("Quad instance buffer should always exist")
    }

    /// Get the quad instance buffer capacity (for bind group cache hash)
    pub fn quad_instance_capacity(&self) -> u64 {
        self.quad_instance_buffer.capacity()
    }

    /// Clear all retained meshes (call when switching games)
    ///
    /// This implements the "clear-on-init" pattern - clearing at the start of
    /// loading a new game rather than when exiting. This handles crashes/failed
    /// init gracefully since the next game load will clear stale state.
    pub fn clear_game_meshes(&mut self) {
        self.retained_meshes.clear();
        self.next_mesh_id = 1;
        for buf in &mut self.retained_vertex_buffers {
            buf.reset();
        }
        for buf in &mut self.retained_index_buffers {
            buf.reset();
        }
        tracing::debug!("Cleared retained meshes for new game");
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graphics::vertex::{FORMAT_COLOR, FORMAT_NORMAL, FORMAT_UV};

    #[test]
    fn test_retained_mesh_non_indexed() {
        let mesh = RetainedMesh {
            format: FORMAT_UV | FORMAT_NORMAL,
            vertex_count: 36,
            index_count: 0,
            vertex_offset: 1024,
            index_offset: 0,
        };
        assert_eq!(mesh.format, FORMAT_UV | FORMAT_NORMAL);
        assert_eq!(mesh.index_count, 0);
    }

    #[test]
    fn test_retained_mesh_indexed() {
        let mesh = RetainedMesh {
            format: FORMAT_COLOR,
            vertex_count: 8,
            index_count: 36,
            vertex_offset: 0,
            index_offset: 512,
        };
        assert_eq!(mesh.vertex_count, 8);
        assert_eq!(mesh.index_count, 36);
    }

    /// Test that index data alignment padding works correctly for wgpu COPY_BUFFER_ALIGNMENT
    #[test]
    fn test_index_data_alignment_padding() {
        // Helper to compute padded index data (same logic as load_mesh_indexed*)
        fn pad_index_data(indices: &[u16]) -> Cow<'_, [u8]> {
            let index_byte_data: &[u8] = bytemuck::cast_slice(indices);
            if index_byte_data.len() % 4 == 0 {
                Cow::Borrowed(index_byte_data)
            } else {
                let padded_len = (index_byte_data.len() + 3) & !3;
                let mut padded = index_byte_data.to_vec();
                padded.resize(padded_len, 0);
                Cow::Owned(padded)
            }
        }

        // Even number of indices (e.g., 200) = 400 bytes = already 4-byte aligned
        let even_indices: Vec<u16> = (0..200).collect();
        let padded = pad_index_data(&even_indices);
        assert_eq!(padded.len(), 400);
        assert_eq!(padded.len() % 4, 0);
        assert!(matches!(padded, Cow::Borrowed(_)), "Should borrow when already aligned");

        // Odd number of indices (e.g., 201) = 402 bytes = needs padding to 404
        let odd_indices: Vec<u16> = (0..201).collect();
        let padded = pad_index_data(&odd_indices);
        assert_eq!(padded.len(), 404);
        assert_eq!(padded.len() % 4, 0);
        assert!(matches!(padded, Cow::Owned(_)), "Should allocate when padding needed");

        // Edge case: 1 index = 2 bytes = needs padding to 4
        let one_index: Vec<u16> = vec![42];
        let padded = pad_index_data(&one_index);
        assert_eq!(padded.len(), 4);
        assert_eq!(padded.len() % 4, 0);

        // Edge case: 3 indices = 6 bytes = needs padding to 8
        let three_indices: Vec<u16> = vec![1, 2, 3];
        let padded = pad_index_data(&three_indices);
        assert_eq!(padded.len(), 8);
        assert_eq!(padded.len() % 4, 0);

        // Edge case: empty = 0 bytes = already aligned
        let empty: Vec<u16> = vec![];
        let padded = pad_index_data(&empty);
        assert_eq!(padded.len(), 0);
        assert!(matches!(padded, Cow::Borrowed(_)));
    }
}
