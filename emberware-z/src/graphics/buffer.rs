//! GPU buffer management for vertex and index data
//!
//! Provides auto-growing buffers for efficient GPU memory management
//! and retained mesh storage.

use hashbrown::HashMap;

use anyhow::Result;

use super::vertex::{vertex_stride, VertexFormatInfo, VERTEX_FORMAT_COUNT};

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

    /// Get a buffer slice for the used portion
    pub fn slice(&self) -> wgpu::BufferSlice<'_> {
        self.buffer.slice(0..self.used)
    }
}

/// Handle to a retained mesh
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct MeshHandle(pub u32);

impl MeshHandle {
    /// Invalid/null mesh handle
    pub const INVALID: MeshHandle = MeshHandle(0);
}

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

        Self {
            vertex_buffers,
            index_buffers,
            retained_vertex_buffers,
            retained_index_buffers,
            retained_meshes: HashMap::new(),
            next_mesh_id: 1, // 0 is reserved for INVALID
        }
    }

    /// Load a non-indexed mesh (retained mode)
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
        let format_idx = format as usize;
        if format_idx >= VERTEX_FORMAT_COUNT {
            anyhow::bail!("Invalid vertex format: {}", format);
        }

        let stride = vertex_stride(format) as usize;
        let byte_data = bytemuck::cast_slice(data);
        let vertex_count = byte_data.len() / stride;

        if byte_data.len() % stride != 0 {
            anyhow::bail!(
                "Vertex data size {} is not a multiple of stride {}",
                byte_data.len(),
                stride
            );
        }

        // Ensure retained buffer has capacity
        self.retained_vertex_buffers[format_idx].ensure_capacity(device, byte_data.len() as u64);

        // Write to retained buffer
        let vertex_offset = self.retained_vertex_buffers[format_idx].write(queue, byte_data);

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
            "Loaded mesh {}: {} vertices, format {}",
            handle.0,
            vertex_count,
            VertexFormatInfo::for_format(format).name
        );

        Ok(handle)
    }

    /// Load an indexed mesh (retained mode)
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
        let format_idx = format as usize;
        if format_idx >= VERTEX_FORMAT_COUNT {
            anyhow::bail!("Invalid vertex format: {}", format);
        }

        let stride = vertex_stride(format) as usize;
        let byte_data = bytemuck::cast_slice(data);
        let vertex_count = byte_data.len() / stride;

        if byte_data.len() % stride != 0 {
            anyhow::bail!(
                "Vertex data size {} is not a multiple of stride {}",
                byte_data.len(),
                stride
            );
        }

        // Ensure retained vertex buffer has capacity
        self.retained_vertex_buffers[format_idx].ensure_capacity(device, byte_data.len() as u64);

        // Ensure retained index buffer has capacity
        let index_byte_data: &[u8] = bytemuck::cast_slice(indices);
        self.retained_index_buffers[format_idx]
            .ensure_capacity(device, index_byte_data.len() as u64);

        // Write to retained buffers
        let vertex_offset = self.retained_vertex_buffers[format_idx].write(queue, byte_data);
        let index_offset = self.retained_index_buffers[format_idx].write(queue, index_byte_data);

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
            "Loaded indexed mesh {}: {} vertices, {} indices, format {}",
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

    /// Get retained index buffer for a format
    pub fn retained_index_buffer(&self, format: u8) -> &GrowableBuffer {
        &self.retained_index_buffers[format as usize]
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graphics::vertex::{FORMAT_COLOR, FORMAT_NORMAL, FORMAT_UV};

    #[test]
    fn test_mesh_handle_invalid() {
        assert_eq!(MeshHandle::INVALID, MeshHandle(0));
    }

    #[test]
    fn test_retained_mesh_default_values() {
        let mesh = RetainedMesh {
            format: 0,
            vertex_count: 100,
            index_count: 150,
            vertex_offset: 0,
            index_offset: 0,
        };
        assert_eq!(mesh.format, 0);
        assert_eq!(mesh.vertex_count, 100);
        assert_eq!(mesh.index_count, 150);
        assert_eq!(mesh.vertex_offset, 0);
        assert_eq!(mesh.index_offset, 0);
    }

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
}
