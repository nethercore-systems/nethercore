//! GPU buffer management for vertex and index data
//!
//! Provides auto-growing buffers for efficient GPU memory management
//! and retained mesh storage.

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

    /// Reset the used counter (for per-frame immediate mode buffers)
    pub fn reset(&mut self) {
        self.used = 0;
    }

    /// Get the underlying wgpu buffer
    pub fn buffer(&self) -> &wgpu::Buffer {
        &self.buffer
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
#[derive(Debug)]
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
