//! Virtual Render Pass for batching draw calls
//!
//! Accumulates draw commands during the frame and provides vertex/index data
//! for flushing to the GPU at frame end. This serves as an intermediate
//! representation between FFI commands and GPU execution.

use super::render_state::{CullMode, RenderState, TextureHandle};
use super::vertex::{vertex_stride, VERTEX_FORMAT_COUNT};

/// Specifies which buffer the geometry data comes from
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BufferSource {
    /// Dynamic geometry uploaded each frame (draw_triangles)
    /// Contains buffer_index into mvp_shading_states
    Immediate(u32),
    /// Static geometry uploaded once (load_mesh_indexed, draw_mesh)
    /// Contains buffer_index into mvp_shading_states
    Retained(u32),
    /// GPU-instanced quads (billboards, sprites, draw_rect)
    /// No buffer index - quads store shading state in instance data
    Quad,
}

/// Virtual Render Pass Command
///
/// Represents a single draw call with all necessary state captured.
/// Named VRPCommand to clearly indicate it's part of the Virtual Render Pass system.
///
/// Variants correspond to different rendering paths:
/// - Mesh: Non-indexed draws (draw_triangles)
/// - IndexedMesh: Indexed draws (draw_mesh, load_mesh_indexed)
/// - Quad: GPU-instanced quads (billboards, sprites, text)
#[derive(Debug, Clone)]
pub enum VRPCommand {
    /// Non-indexed mesh draw (draw_triangles, immediate geometry)
    Mesh {
        format: u8,
        vertex_count: u32,
        base_vertex: u32,
        buffer_index: u32,
        texture_slots: [TextureHandle; 4],
        depth_test: bool,
        cull_mode: CullMode,
    },
    /// Indexed mesh draw (draw_mesh, load_mesh_indexed)
    IndexedMesh {
        format: u8,
        index_count: u32,
        base_vertex: u32,
        first_index: u32,
        buffer_index: u32,
        texture_slots: [TextureHandle; 4],
        depth_test: bool,
        cull_mode: CullMode,
    },
    /// GPU-instanced quad draw (billboards, sprites, text, rects)
    /// All quads share a single unit quad mesh (4 vertices, 6 indices)
    Quad {
        base_vertex: u32,    // Unit quad base vertex in buffer
        first_index: u32,    // Unit quad first index in buffer
        instance_count: u32, // Number of quad instances to draw
        texture_slots: [TextureHandle; 4],
        depth_test: bool,
        cull_mode: CullMode,
    },
}

/// Virtual Render Pass for batching immediate-mode draws
///
/// Accumulates draw commands and vertex/index data during the frame,
/// providing everything needed for GPU execution at frame end.
#[derive(Debug)]
pub struct VirtualRenderPass {
    /// Draw commands accumulated this frame
    commands: Vec<VRPCommand>,
    /// Per-format immediate vertex data (CPU side)
    vertex_data: [Vec<u8>; VERTEX_FORMAT_COUNT],
    /// Per-format immediate index data (CPU side, u16 for memory efficiency)
    index_data: [Vec<u16>; VERTEX_FORMAT_COUNT],
    /// Per-format vertex counts for base_vertex calculation
    vertex_counts: [u32; VERTEX_FORMAT_COUNT],
    /// Per-format index counts
    index_counts: [u32; VERTEX_FORMAT_COUNT],
}

impl VirtualRenderPass {
    /// Create a new command buffer
    pub fn new() -> Self {
        Self {
            commands: Vec::with_capacity(1024),
            vertex_data: std::array::from_fn(|_| Vec::with_capacity(64 * 1024)),
            index_data: std::array::from_fn(|_| Vec::with_capacity(16 * 1024)),
            vertex_counts: [0; VERTEX_FORMAT_COUNT],
            index_counts: [0; VERTEX_FORMAT_COUNT],
        }
    }

    /// Add vertices for immediate drawing (non-indexed)
    ///
    /// Returns the base vertex index for this batch.
    pub fn add_vertices(
        &mut self,
        format: u8,
        vertices: &[f32],
        buffer_index: u32,
        texture_slots: [TextureHandle; 4],
        state: &RenderState,
    ) -> u32 {
        let format_idx = format as usize;
        let stride = vertex_stride(format) as usize;
        let vertex_count = (vertices.len() * 4) / stride;
        let base_vertex = self.vertex_counts[format_idx];

        // Append vertex data
        let byte_data = bytemuck::cast_slice(vertices);
        self.vertex_data[format_idx].extend_from_slice(byte_data);
        self.vertex_counts[format_idx] += vertex_count as u32;

        // Record draw command
        self.commands.push(VRPCommand::Mesh {
            format,
            vertex_count: vertex_count as u32,
            base_vertex,
            buffer_index,
            texture_slots,
            depth_test: state.depth_test,
            cull_mode: state.cull_mode,
        });

        base_vertex
    }

    /// Add indexed vertices for immediate drawing
    ///
    /// Returns (base_vertex, first_index) for this batch.
    pub fn add_vertices_indexed(
        &mut self,
        format: u8,
        vertices: &[f32],
        indices: &[u16],
        buffer_index: u32,
        texture_slots: [TextureHandle; 4],
        state: &RenderState,
    ) -> (u32, u32) {
        let format_idx = format as usize;
        let stride = vertex_stride(format) as usize;
        let vertex_count = (vertices.len() * 4) / stride;
        let base_vertex = self.vertex_counts[format_idx];
        let first_index = self.index_counts[format_idx];

        // Append vertex data
        let byte_data = bytemuck::cast_slice(vertices);
        self.vertex_data[format_idx].extend_from_slice(byte_data);
        self.vertex_counts[format_idx] += vertex_count as u32;

        // Append index data
        self.index_data[format_idx].extend_from_slice(indices);
        self.index_counts[format_idx] += indices.len() as u32;

        // Record draw command
        self.commands.push(VRPCommand::IndexedMesh {
            format,
            index_count: indices.len() as u32,
            base_vertex,
            first_index,
            buffer_index,
            texture_slots,
            depth_test: state.depth_test,
            cull_mode: state.cull_mode,
        });

        (base_vertex, first_index)
    }

    /// Get accumulated commands
    pub fn commands(&self) -> &[VRPCommand] {
        &self.commands
    }

    /// Get mutable access to accumulated commands (for in-place sorting)
    pub fn commands_mut(&mut self) -> &mut [VRPCommand] {
        &mut self.commands
    }

    /// Append vertex data to buffer and return base_vertex index
    ///
    /// Used for direct conversion from ZVRPCommand without state mutation.
    pub fn append_vertex_data(&mut self, format: u8, data: &[f32]) -> u32 {
        let format_idx = format as usize;
        let stride = vertex_stride(format) as usize;
        let vertex_count = (data.len() * 4) / stride;
        let base_vertex = self.vertex_counts[format_idx];

        // Append vertex data
        let byte_data = bytemuck::cast_slice(data);
        self.vertex_data[format_idx].extend_from_slice(byte_data);
        self.vertex_counts[format_idx] += vertex_count as u32;

        base_vertex
    }

    /// Append index data to buffer and return first_index
    ///
    /// Used for direct conversion from ZVRPCommand without state mutation.
    pub fn append_index_data(&mut self, format: u8, data: &[u16]) -> u32 {
        let format_idx = format as usize;
        let first_index = self.index_counts[format_idx];

        // Append index data
        self.index_data[format_idx].extend_from_slice(data);
        self.index_counts[format_idx] += data.len() as u32;

        first_index
    }

    /// Add a draw command directly
    ///
    /// Used for direct conversion from ZVRPCommand without state mutation.
    pub fn add_command(&mut self, cmd: VRPCommand) {
        self.commands.push(cmd);
    }

    /// Record a non-indexed triangle draw (called from FFI)
    pub fn record_triangles(
        &mut self,
        format: u8,
        vertex_data: &[f32],
        buffer_index: u32,
        texture_slots: [TextureHandle; 4],
        depth_test: bool,
        cull_mode: CullMode,
    ) {
        let format_idx = format as usize;
        let stride = vertex_stride(format) as usize;
        let vertex_count = (vertex_data.len() * 4) / stride;
        let base_vertex = self.vertex_counts[format_idx];

        // Write directly to buffer (no intermediate Vec)
        let byte_data = bytemuck::cast_slice(vertex_data);
        self.vertex_data[format_idx].extend_from_slice(byte_data);
        self.vertex_counts[format_idx] += vertex_count as u32;

        self.commands.push(VRPCommand::Mesh {
            format,
            vertex_count: vertex_count as u32,
            base_vertex,
            buffer_index,
            texture_slots,
            depth_test,
            cull_mode,
        });
    }

    /// Record an indexed triangle draw (called from FFI)
    pub fn record_triangles_indexed(
        &mut self,
        format: u8,
        vertex_data: &[f32],
        index_data: &[u16],
        buffer_index: u32,
        texture_slots: [TextureHandle; 4],
        depth_test: bool,
        cull_mode: CullMode,
    ) {
        let format_idx = format as usize;
        let stride = vertex_stride(format) as usize;
        let vertex_count = (vertex_data.len() * 4) / stride;
        let base_vertex = self.vertex_counts[format_idx];
        let first_index = self.index_counts[format_idx];

        // Write directly to buffers
        let byte_data = bytemuck::cast_slice(vertex_data);
        self.vertex_data[format_idx].extend_from_slice(byte_data);
        self.vertex_counts[format_idx] += vertex_count as u32;

        self.index_data[format_idx].extend_from_slice(index_data);
        self.index_counts[format_idx] += index_data.len() as u32;

        self.commands.push(VRPCommand::IndexedMesh {
            format,
            index_count: index_data.len() as u32,
            base_vertex,
            first_index,
            buffer_index,
            texture_slots,
            depth_test,
            cull_mode,
        });
    }

    /// Record a mesh draw (called from FFI)
    pub fn record_mesh(
        &mut self,
        mesh_format: u8,
        mesh_vertex_count: u32,
        mesh_index_count: u32,
        mesh_vertex_offset: u64,
        mesh_index_offset: u64,
        buffer_index: u32,
        texture_slots: [TextureHandle; 4],
        depth_test: bool,
        cull_mode: CullMode,
    ) {
        let stride = vertex_stride(mesh_format) as u64;
        let base_vertex = (mesh_vertex_offset / stride) as u32;

        // Choose variant based on whether mesh is indexed
        if mesh_index_count > 0 {
            let first_index = (mesh_index_offset / 2) as u32; // u16 indices are 2 bytes each
            self.commands.push(VRPCommand::IndexedMesh {
                format: mesh_format,
                index_count: mesh_index_count,
                base_vertex,
                first_index,
                buffer_index,
                texture_slots,
                depth_test,
                cull_mode,
            });
        } else {
            self.commands.push(VRPCommand::Mesh {
                format: mesh_format,
                vertex_count: mesh_vertex_count,
                base_vertex,
                buffer_index,
                texture_slots,
                depth_test,
                cull_mode,
            });
        }
    }

    /// Get vertex data for a format
    pub fn vertex_data(&self, format: u8) -> &[u8] {
        &self.vertex_data[format as usize]
    }

    /// Get index data for a format
    pub fn index_data(&self, format: u8) -> &[u16] {
        &self.index_data[format as usize]
    }

    /// Reset the command buffer for the next frame
    pub fn reset(&mut self) {
        self.commands.clear();
        for data in &mut self.vertex_data {
            data.clear();
        }
        for data in &mut self.index_data {
            data.clear();
        }
        self.vertex_counts = [0; VERTEX_FORMAT_COUNT];
        self.index_counts = [0; VERTEX_FORMAT_COUNT];
    }
}

impl Default for VirtualRenderPass {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graphics::vertex::{FORMAT_COLOR, FORMAT_SKINNED, FORMAT_UV};

    #[test]
    fn test_command_buffer_new() {
        let cb = VirtualRenderPass::new();
        assert!(cb.commands().is_empty());
        for i in 0..VERTEX_FORMAT_COUNT {
            assert!(cb.vertex_data(i as u8).is_empty());
            assert!(cb.index_data(i as u8).is_empty());
        }
    }

    #[test]
    fn test_command_buffer_add_vertices() {
        let mut cb = VirtualRenderPass::new();
        let state = RenderState::default();

        let vertices = [
            0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.5, 1.0, 0.0, 0.0, 0.0,
            1.0,
        ];

        let base = cb.add_vertices(
            FORMAT_COLOR,
            &vertices,
            0,  // buffer_index
            [TextureHandle::INVALID; 4],
            &state,
        );

        assert_eq!(base, 0);
        assert_eq!(cb.commands().len(), 1);
        match &cb.commands()[0] {
            VRPCommand::Mesh { vertex_count, format, .. } => {
                assert_eq!(*vertex_count, 3);
                assert_eq!(*format, FORMAT_COLOR);
            }
            _ => panic!("Expected Mesh variant"),
        }
    }

    #[test]
    fn test_command_buffer_add_vertices_indexed() {
        let mut cb = VirtualRenderPass::new();
        let state = RenderState::default();

        let vertices = [0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 0.0];
        let indices = [0u16, 1, 2, 0, 2, 3];

        let (base_vertex, first_index) = cb.add_vertices_indexed(
            0,
            &vertices,
            &indices,
            0,  // buffer_index
            [TextureHandle::INVALID; 4],
            &state,
        );

        assert_eq!(base_vertex, 0);
        assert_eq!(first_index, 0);
        assert_eq!(cb.commands().len(), 1);
        match &cb.commands()[0] {
            VRPCommand::IndexedMesh { index_count, .. } => {
                assert_eq!(*index_count, 6);
            }
            _ => panic!("Expected IndexedMesh variant"),
        }
    }

    #[test]
    fn test_command_buffer_reset() {
        let mut cb = VirtualRenderPass::new();
        let state = RenderState::default();

        let vertices = [0.0f32, 0.0, 0.0, 1.0, 0.0, 0.0, 0.5, 1.0, 0.0];
        cb.add_vertices(
            0,
            &vertices,
            0,  // buffer_index
            [TextureHandle::INVALID; 4],
            &state,
        );

        assert!(!cb.commands().is_empty());

        cb.reset();

        assert!(cb.commands().is_empty());
        assert!(cb.vertex_data(0).is_empty());
    }

    // Command buffer tests removed - implementation details changed with unified shading state
    // TODO: Add new tests for unified shading state command buffer if needed
    // #[test]
    // fn test_draw_command_captures_texture_slots() { ... }
    // #[test]
    // fn test_draw_commands_capture_render_state() { ... }

    #[test]
    fn test_command_buffer_different_formats() {
        let mut cb = VirtualRenderPass::new();
        let state = RenderState::default();

        let v_pos = [0.0f32, 0.0, 0.0, 1.0, 0.0, 0.0, 0.5, 1.0, 0.0];
        let v_pos_uv = [
            0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.5, 1.0, 0.0, 0.5, 1.0,
        ];

        cb.add_vertices(
            0,
            &v_pos,
            0,  // buffer_index
            [TextureHandle::INVALID; 4],
            &state,
        );
        cb.add_vertices(
            FORMAT_UV,
            &v_pos_uv,
            0,  // buffer_index
            [TextureHandle::INVALID; 4],
            &state,
        );

        assert_eq!(cb.commands().len(), 2);
        match &cb.commands()[0] {
            VRPCommand::Mesh { format, .. } => assert_eq!(*format, 0),
            _ => panic!("Expected Mesh variant"),
        }
        match &cb.commands()[1] {
            VRPCommand::Mesh { format, .. } => assert_eq!(*format, FORMAT_UV),
            _ => panic!("Expected Mesh variant"),
        }
    }

    #[test]
    fn test_command_buffer_mvp_index_capture() {
        let mut cb = VirtualRenderPass::new();
        let state = RenderState::default();
        let vertices = [0.0f32, 0.0, 0.0, 1.0, 0.0, 0.0, 0.5, 1.0, 0.0];

        let buffer_index1 = 0;
        let buffer_index2 = 1;

        cb.add_vertices(
            0,
            &vertices,
            buffer_index1,
            [TextureHandle::INVALID; 4],
            &state,
        );
        cb.add_vertices(
            0,
            &vertices,
            buffer_index2,
            [TextureHandle::INVALID; 4],
            &state,
        );

        match &cb.commands()[0] {
            VRPCommand::Mesh { buffer_index, .. } => assert_eq!(*buffer_index, buffer_index1),
            _ => panic!("Expected Mesh variant"),
        }
        match &cb.commands()[1] {
            VRPCommand::Mesh { buffer_index, .. } => assert_eq!(*buffer_index, buffer_index2),
            _ => panic!("Expected Mesh variant"),
        }
    }

    #[test]
    fn test_command_buffer_large_batch() {
        let mut cb = VirtualRenderPass::new();
        let state = RenderState::default();

        let triangle = [0.0f32, 0.0, 0.0, 1.0, 0.0, 0.0, 0.5, 1.0, 0.0];
        let mut large_data = Vec::with_capacity(9000);
        for _ in 0..1000 {
            large_data.extend_from_slice(&triangle);
        }

        let base = cb.add_vertices(
            0,
            &large_data,
            0,  // buffer_index
            [TextureHandle::INVALID; 4],
            &state,
        );
        assert_eq!(base, 0);
        match &cb.commands()[0] {
            VRPCommand::Mesh { vertex_count, .. } => assert_eq!(*vertex_count, 3000),
            _ => panic!("Expected Mesh variant"),
        }
    }

    #[test]
    fn test_command_buffer_skinned_vertices() {
        let mut cb = VirtualRenderPass::new();
        let state = RenderState::default();

        let vertices = [
            0.0,
            0.0,
            0.0,
            f32::from_bits(0x03020100),
            1.0,
            0.0,
            0.0,
            0.0,
            1.0,
            0.0,
            0.0,
            f32::from_bits(0x03020100),
            1.0,
            0.0,
            0.0,
            0.0,
            0.5,
            1.0,
            0.0,
            f32::from_bits(0x03020100),
            1.0,
            0.0,
            0.0,
            0.0,
        ];

        let base = cb.add_vertices(
            FORMAT_SKINNED,
            &vertices,
            0,  // buffer_index
            [TextureHandle::INVALID; 4],
            &state,
        );

        assert_eq!(base, 0);
        assert_eq!(cb.commands().len(), 1);
        match &cb.commands()[0] {
            VRPCommand::Mesh { vertex_count, format, .. } => {
                assert_eq!(*vertex_count, 3);
                assert_eq!(*format, FORMAT_SKINNED);
            }
            _ => panic!("Expected Mesh variant"),
        }
    }
}
