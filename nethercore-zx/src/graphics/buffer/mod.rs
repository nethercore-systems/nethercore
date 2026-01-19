//! GPU buffer management for vertex and index data
//!
//! Provides auto-growing buffers for efficient GPU memory management
//! and retained mesh storage.

mod growable_buffer;
mod retained_mesh;

#[cfg(test)]
mod tests;

use anyhow::Result;
use hashbrown::HashMap;

pub use growable_buffer::GrowableBuffer;
use retained_mesh::MeshLoader;
pub use retained_mesh::{MeshHandle, RetainedMesh};

use super::vertex::VERTEX_FORMAT_COUNT;

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
        let mut loader = MeshLoader::new(
            &mut self.retained_vertex_buffers,
            &mut self.retained_index_buffers,
            &mut self.next_mesh_id,
        );
        let (handle, mesh) = loader.load_mesh(device, queue, data, format)?;
        self.retained_meshes.insert(handle.0, mesh);
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
        let mut loader = MeshLoader::new(
            &mut self.retained_vertex_buffers,
            &mut self.retained_index_buffers,
            &mut self.next_mesh_id,
        );
        let (handle, mesh) = loader.load_mesh_indexed(device, queue, data, indices, format)?;
        self.retained_meshes.insert(handle.0, mesh);
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
        let mut loader = MeshLoader::new(
            &mut self.retained_vertex_buffers,
            &mut self.retained_index_buffers,
            &mut self.next_mesh_id,
        );
        let (handle, mesh) = loader.load_mesh_packed(device, queue, data, format)?;
        self.retained_meshes.insert(handle.0, mesh);
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
        let mut loader = MeshLoader::new(
            &mut self.retained_vertex_buffers,
            &mut self.retained_index_buffers,
            &mut self.next_mesh_id,
        );
        let (handle, mesh) =
            loader.load_mesh_indexed_packed(device, queue, data, indices, format)?;
        self.retained_meshes.insert(handle.0, mesh);
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
            .ensure_capacity(device, queue, byte_data.len() as u64);
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
