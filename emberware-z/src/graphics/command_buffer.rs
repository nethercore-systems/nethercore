//! Command buffer for batching draw calls
//!
//! Accumulates draw commands during the frame and provides vertex/index data
//! for flushing to the GPU at frame end.

use glam::Mat4;

use super::render_state::{BlendMode, CullMode, RenderState, TextureHandle};
use super::vertex::{vertex_stride, VERTEX_FORMAT_COUNT};

/// A draw command for batching
#[derive(Debug, Clone)]
pub struct DrawCommand {
    /// Vertex format
    pub format: u8,
    /// Model transform matrix
    pub transform: Mat4,
    /// Number of vertices to draw
    pub vertex_count: u32,
    /// Number of indices (0 for non-indexed)
    pub index_count: u32,
    /// Base vertex index in the immediate buffer
    pub base_vertex: u32,
    /// First index in the immediate index buffer
    pub first_index: u32,
    /// Texture slots bound for this draw
    pub texture_slots: [TextureHandle; 4],
    /// Uniform color
    pub color: u32,
    /// Render state at time of draw
    pub depth_test: bool,
    pub cull_mode: CullMode,
    pub blend_mode: BlendMode,
}

/// Command buffer for batching immediate-mode draws
pub struct CommandBuffer {
    /// Draw commands accumulated this frame
    commands: Vec<DrawCommand>,
    /// Per-format immediate vertex data (CPU side)
    vertex_data: [Vec<u8>; VERTEX_FORMAT_COUNT],
    /// Per-format immediate index data (CPU side)
    index_data: [Vec<u32>; VERTEX_FORMAT_COUNT],
    /// Per-format vertex counts for base_vertex calculation
    vertex_counts: [u32; VERTEX_FORMAT_COUNT],
    /// Per-format index counts
    index_counts: [u32; VERTEX_FORMAT_COUNT],
}

impl CommandBuffer {
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
    pub fn add_vertices(&mut self, format: u8, vertices: &[f32], transform: Mat4, state: &RenderState) -> u32 {
        let format_idx = format as usize;
        let stride = vertex_stride(format) as usize;
        let vertex_count = (vertices.len() * 4) / stride;
        let base_vertex = self.vertex_counts[format_idx];

        // Append vertex data
        let byte_data = bytemuck::cast_slice(vertices);
        self.vertex_data[format_idx].extend_from_slice(byte_data);
        self.vertex_counts[format_idx] += vertex_count as u32;

        // Record draw command
        self.commands.push(DrawCommand {
            format,
            transform,
            vertex_count: vertex_count as u32,
            index_count: 0,
            base_vertex,
            first_index: 0,
            texture_slots: state.texture_slots,
            color: state.color,
            depth_test: state.depth_test,
            cull_mode: state.cull_mode,
            blend_mode: state.blend_mode,
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
        indices: &[u32],
        transform: Mat4,
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
        self.commands.push(DrawCommand {
            format,
            transform,
            vertex_count: vertex_count as u32,
            index_count: indices.len() as u32,
            base_vertex,
            first_index,
            texture_slots: state.texture_slots,
            color: state.color,
            depth_test: state.depth_test,
            cull_mode: state.cull_mode,
            blend_mode: state.blend_mode,
        });

        (base_vertex, first_index)
    }

    /// Get accumulated commands
    pub fn commands(&self) -> &[DrawCommand] {
        &self.commands
    }

    /// Get vertex data for a format
    pub fn vertex_data(&self, format: u8) -> &[u8] {
        &self.vertex_data[format as usize]
    }

    /// Get index data for a format
    pub fn index_data(&self, format: u8) -> &[u32] {
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

impl Default for CommandBuffer {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graphics::vertex::{FORMAT_COLOR, FORMAT_SKINNED, FORMAT_UV};
    use glam::Vec3;

    #[test]
    fn test_command_buffer_new() {
        let cb = CommandBuffer::new();
        assert!(cb.commands().is_empty());
        for i in 0..VERTEX_FORMAT_COUNT {
            assert!(cb.vertex_data(i as u8).is_empty());
            assert!(cb.index_data(i as u8).is_empty());
        }
    }

    #[test]
    fn test_command_buffer_add_vertices() {
        let mut cb = CommandBuffer::new();
        let state = RenderState::default();

        let vertices = [
            0.0, 0.0, 0.0, 1.0, 0.0, 0.0,
            1.0, 0.0, 0.0, 0.0, 1.0, 0.0,
            0.5, 1.0, 0.0, 0.0, 0.0, 1.0,
        ];

        let base = cb.add_vertices(FORMAT_COLOR, &vertices, Mat4::IDENTITY, &state);

        assert_eq!(base, 0);
        assert_eq!(cb.commands().len(), 1);
        assert_eq!(cb.commands()[0].vertex_count, 3);
        assert_eq!(cb.commands()[0].format, FORMAT_COLOR);
    }

    #[test]
    fn test_command_buffer_add_vertices_indexed() {
        let mut cb = CommandBuffer::new();
        let state = RenderState::default();

        let vertices = [
            0.0, 0.0, 0.0,
            1.0, 0.0, 0.0,
            1.0, 1.0, 0.0,
            0.0, 1.0, 0.0,
        ];
        let indices = [0u32, 1, 2, 0, 2, 3];

        let (base_vertex, first_index) = cb.add_vertices_indexed(0, &vertices, &indices, Mat4::IDENTITY, &state);

        assert_eq!(base_vertex, 0);
        assert_eq!(first_index, 0);
        assert_eq!(cb.commands().len(), 1);
        assert_eq!(cb.commands()[0].vertex_count, 4);
        assert_eq!(cb.commands()[0].index_count, 6);
    }

    #[test]
    fn test_command_buffer_reset() {
        let mut cb = CommandBuffer::new();
        let state = RenderState::default();

        let vertices = [0.0f32, 0.0, 0.0, 1.0, 0.0, 0.0, 0.5, 1.0, 0.0];
        cb.add_vertices(0, &vertices, Mat4::IDENTITY, &state);

        assert!(!cb.commands().is_empty());

        cb.reset();

        assert!(cb.commands().is_empty());
        assert!(cb.vertex_data(0).is_empty());
    }

    #[test]
    fn test_command_buffer_multiple_batches() {
        let mut cb = CommandBuffer::new();
        let state = RenderState::default();

        let v1 = [0.0f32, 0.0, 0.0, 1.0, 0.0, 0.0, 0.5, 1.0, 0.0];
        let base1 = cb.add_vertices(0, &v1, Mat4::IDENTITY, &state);

        let v2 = [2.0f32, 0.0, 0.0, 3.0, 0.0, 0.0, 2.5, 1.0, 0.0];
        let base2 = cb.add_vertices(0, &v2, Mat4::IDENTITY, &state);

        assert_eq!(base1, 0);
        assert_eq!(base2, 3);
        assert_eq!(cb.commands().len(), 2);
    }

    #[test]
    fn test_draw_command_creation() {
        let cmd = DrawCommand {
            format: FORMAT_UV,
            transform: Mat4::IDENTITY,
            vertex_count: 3,
            index_count: 0,
            base_vertex: 0,
            first_index: 0,
            texture_slots: [TextureHandle::INVALID; 4],
            color: 0xFFFFFFFF,
            depth_test: true,
            cull_mode: CullMode::Back,
            blend_mode: BlendMode::None,
        };
        assert_eq!(cmd.format, FORMAT_UV);
        assert_eq!(cmd.vertex_count, 3);
        assert_eq!(cmd.color, 0xFFFFFFFF);
    }

    #[test]
    fn test_draw_command_clone() {
        let cmd = DrawCommand {
            format: FORMAT_COLOR,
            transform: Mat4::from_translation(Vec3::new(1.0, 2.0, 3.0)),
            vertex_count: 100,
            index_count: 150,
            base_vertex: 50,
            first_index: 75,
            texture_slots: [TextureHandle(1), TextureHandle(2), TextureHandle::INVALID, TextureHandle::INVALID],
            color: 0xFF0000FF,
            depth_test: false,
            cull_mode: CullMode::None,
            blend_mode: BlendMode::Alpha,
        };
        let cloned = cmd.clone();
        assert_eq!(cloned.format, cmd.format);
        assert_eq!(cloned.vertex_count, cmd.vertex_count);
        assert_eq!(cloned.texture_slots, cmd.texture_slots);
    }

    #[test]
    fn test_draw_command_captures_texture_slots() {
        let mut cb = CommandBuffer::new();
        let mut state = RenderState::default();

        state.texture_slots[0] = TextureHandle(10);
        state.texture_slots[1] = TextureHandle(20);

        let vertices = [0.0f32, 0.0, 0.0, 1.0, 0.0, 0.0, 0.5, 1.0, 0.0];
        cb.add_vertices(0, &vertices, Mat4::IDENTITY, &state);

        assert_eq!(cb.commands()[0].texture_slots[0], TextureHandle(10));
        assert_eq!(cb.commands()[0].texture_slots[1], TextureHandle(20));
        assert_eq!(cb.commands()[0].texture_slots[2], TextureHandle::INVALID);
    }

    #[test]
    fn test_draw_commands_capture_render_state() {
        let mut cb = CommandBuffer::new();
        let vertices = [0.0f32, 0.0, 0.0, 1.0, 0.0, 0.0, 0.5, 1.0, 0.0];

        let state1 = RenderState::default();
        cb.add_vertices(0, &vertices, Mat4::IDENTITY, &state1);

        let state2 = RenderState {
            color: 0xFF0000FF,
            depth_test: false,
            cull_mode: CullMode::None,
            blend_mode: BlendMode::Alpha,
            ..Default::default()
        };
        cb.add_vertices(0, &vertices, Mat4::IDENTITY, &state2);

        assert_eq!(cb.commands()[0].color, 0xFFFFFFFF);
        assert!(cb.commands()[0].depth_test);
        assert_eq!(cb.commands()[0].cull_mode, CullMode::Back);
        assert_eq!(cb.commands()[0].blend_mode, BlendMode::None);

        assert_eq!(cb.commands()[1].color, 0xFF0000FF);
        assert!(!cb.commands()[1].depth_test);
        assert_eq!(cb.commands()[1].cull_mode, CullMode::None);
        assert_eq!(cb.commands()[1].blend_mode, BlendMode::Alpha);
    }

    #[test]
    fn test_command_buffer_different_formats() {
        let mut cb = CommandBuffer::new();
        let state = RenderState::default();

        let v_pos = [0.0f32, 0.0, 0.0, 1.0, 0.0, 0.0, 0.5, 1.0, 0.0];
        let v_pos_uv = [
            0.0, 0.0, 0.0, 0.0, 0.0,
            1.0, 0.0, 0.0, 1.0, 0.0,
            0.5, 1.0, 0.0, 0.5, 1.0,
        ];

        cb.add_vertices(0, &v_pos, Mat4::IDENTITY, &state);
        cb.add_vertices(FORMAT_UV, &v_pos_uv, Mat4::IDENTITY, &state);

        assert_eq!(cb.commands().len(), 2);
        assert_eq!(cb.commands()[0].format, 0);
        assert_eq!(cb.commands()[1].format, FORMAT_UV);
    }

    #[test]
    fn test_command_buffer_transform_capture() {
        let mut cb = CommandBuffer::new();
        let state = RenderState::default();
        let vertices = [0.0f32, 0.0, 0.0, 1.0, 0.0, 0.0, 0.5, 1.0, 0.0];

        let transform1 = Mat4::IDENTITY;
        let transform2 = Mat4::from_translation(Vec3::new(10.0, 20.0, 30.0));

        cb.add_vertices(0, &vertices, transform1, &state);
        cb.add_vertices(0, &vertices, transform2, &state);

        assert_eq!(cb.commands()[0].transform, transform1);
        assert_eq!(cb.commands()[1].transform, transform2);
    }

    #[test]
    fn test_command_buffer_large_batch() {
        let mut cb = CommandBuffer::new();
        let state = RenderState::default();

        let triangle = [0.0f32, 0.0, 0.0, 1.0, 0.0, 0.0, 0.5, 1.0, 0.0];
        let mut large_data = Vec::with_capacity(9000);
        for _ in 0..1000 {
            large_data.extend_from_slice(&triangle);
        }

        let base = cb.add_vertices(0, &large_data, Mat4::IDENTITY, &state);
        assert_eq!(base, 0);
        assert_eq!(cb.commands()[0].vertex_count, 3000);
    }

    #[test]
    fn test_command_buffer_skinned_vertices() {
        let mut cb = CommandBuffer::new();
        let state = RenderState::default();

        let vertices = [
            0.0, 0.0, 0.0, f32::from_bits(0x03020100), 1.0, 0.0, 0.0, 0.0,
            1.0, 0.0, 0.0, f32::from_bits(0x03020100), 1.0, 0.0, 0.0, 0.0,
            0.5, 1.0, 0.0, f32::from_bits(0x03020100), 1.0, 0.0, 0.0, 0.0,
        ];

        let base = cb.add_vertices(FORMAT_SKINNED, &vertices, Mat4::IDENTITY, &state);

        assert_eq!(base, 0);
        assert_eq!(cb.commands().len(), 1);
        assert_eq!(cb.commands()[0].vertex_count, 3);
        assert_eq!(cb.commands()[0].format, FORMAT_SKINNED);
    }
}
