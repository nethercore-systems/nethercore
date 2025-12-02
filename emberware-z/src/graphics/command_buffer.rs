//! Virtual Render Pass for batching draw calls
//!
//! Accumulates draw commands during the frame and provides vertex/index data
//! for flushing to the GPU at frame end. This serves as an intermediate
//! representation between FFI commands and GPU execution.

use super::matrix_packing::MvpIndex;
use super::render_state::{BlendMode, CullMode, MatcapBlendMode, RenderState, TextureHandle};
use super::vertex::{vertex_stride, VERTEX_FORMAT_COUNT};

/// Specifies which buffer the geometry data comes from
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BufferSource {
    /// Dynamic geometry uploaded each frame (sprites, billboards, draw_triangles)
    Immediate,
    /// Static geometry uploaded once (load_mesh_indexed, draw_mesh)
    Retained,
}

/// Virtual Render Pass Command
///
/// Represents a single draw call with all necessary state captured.
/// Named VRPCommand to clearly indicate it's part of the Virtual Render Pass system.
#[derive(Debug, Clone)]
pub struct VRPCommand {
    /// Vertex format
    pub format: u8,
    /// Packed MVP matrix indices (model: 16 bits, view: 8 bits, proj: 8 bits)
    pub mvp_index: MvpIndex,
    /// Number of vertices to draw
    pub vertex_count: u32,
    /// Number of indices (0 for non-indexed)
    pub index_count: u32,
    /// Base vertex index in the buffer
    pub base_vertex: u32,
    /// First index in the index buffer
    pub first_index: u32,
    /// Which buffer contains this geometry's data
    pub buffer_source: BufferSource,
    /// Texture slots bound for this draw
    pub texture_slots: [TextureHandle; 4],
    /// Uniform color
    pub color: u32,
    /// Render state at time of draw
    pub depth_test: bool,
    pub cull_mode: CullMode,
    pub blend_mode: BlendMode,
    /// Matcap blend modes for slots 1-3 (Mode 1 only)
    pub matcap_blend_modes: [MatcapBlendMode; 4],
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
        mvp_index: MvpIndex,
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
        self.commands.push(VRPCommand {
            format,
            mvp_index,
            vertex_count: vertex_count as u32,
            index_count: 0,
            base_vertex,
            first_index: 0,
            buffer_source: BufferSource::Immediate,
            texture_slots: state.texture_slots,
            color: state.color,
            depth_test: state.depth_test,
            cull_mode: state.cull_mode,
            blend_mode: state.blend_mode,
            matcap_blend_modes: state.matcap_blend_modes,
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
        mvp_index: MvpIndex,
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
        self.commands.push(VRPCommand {
            format,
            mvp_index,
            vertex_count: vertex_count as u32,
            index_count: indices.len() as u32,
            base_vertex,
            first_index,
            buffer_source: BufferSource::Immediate,
            texture_slots: state.texture_slots,
            color: state.color,
            depth_test: state.depth_test,
            cull_mode: state.cull_mode,
            blend_mode: state.blend_mode,
            matcap_blend_modes: state.matcap_blend_modes,
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
        mvp_index: MvpIndex,
        color: u32,
        depth_test: bool,
        cull_mode: CullMode,
        blend_mode: BlendMode,
        texture_slots: [TextureHandle; 4],
        matcap_blend_modes: [MatcapBlendMode; 4],
    ) {
        let format_idx = format as usize;
        let stride = vertex_stride(format) as usize;
        let vertex_count = (vertex_data.len() * 4) / stride;
        let base_vertex = self.vertex_counts[format_idx];

        // Write directly to buffer (no intermediate Vec)
        let byte_data = bytemuck::cast_slice(vertex_data);
        self.vertex_data[format_idx].extend_from_slice(byte_data);
        self.vertex_counts[format_idx] += vertex_count as u32;

        self.commands.push(VRPCommand {
            format,
            mvp_index,
            vertex_count: vertex_count as u32,
            index_count: 0,
            base_vertex,
            first_index: 0,
            buffer_source: BufferSource::Immediate,
            texture_slots,
            color,
            depth_test,
            cull_mode,
            blend_mode,
            matcap_blend_modes,
        });
    }

    /// Record an indexed triangle draw (called from FFI)
    pub fn record_triangles_indexed(
        &mut self,
        format: u8,
        vertex_data: &[f32],
        index_data: &[u16],
        mvp_index: MvpIndex,
        color: u32,
        depth_test: bool,
        cull_mode: CullMode,
        blend_mode: BlendMode,
        texture_slots: [TextureHandle; 4],
        matcap_blend_modes: [MatcapBlendMode; 4],
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

        self.commands.push(VRPCommand {
            format,
            mvp_index,
            vertex_count: vertex_count as u32,
            index_count: index_data.len() as u32,
            base_vertex,
            first_index,
            buffer_source: BufferSource::Immediate,
            texture_slots,
            color,
            depth_test,
            cull_mode,
            blend_mode,
            matcap_blend_modes,
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
        mvp_index: MvpIndex,
        color: u32,
        depth_test: bool,
        cull_mode: CullMode,
        blend_mode: BlendMode,
        texture_slots: [TextureHandle; 4],
        matcap_blend_modes: [MatcapBlendMode; 4],
    ) {
        let stride = vertex_stride(mesh_format) as u64;
        let base_vertex = (mesh_vertex_offset / stride) as u32;
        let first_index = if mesh_index_count > 0 {
            (mesh_index_offset / 2) as u32 // u16 indices are 2 bytes each
        } else {
            0
        };

        self.commands.push(VRPCommand {
            format: mesh_format,
            mvp_index,
            vertex_count: mesh_vertex_count,
            index_count: mesh_index_count,
            base_vertex,
            first_index,
            buffer_source: BufferSource::Retained,
            texture_slots,
            color,
            depth_test,
            cull_mode,
            blend_mode,
            matcap_blend_modes,
        });
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

        let mvp_index = MvpIndex::new(0, 0, 0);
        let base = cb.add_vertices(FORMAT_COLOR, &vertices, mvp_index, &state);

        assert_eq!(base, 0);
        assert_eq!(cb.commands().len(), 1);
        assert_eq!(cb.commands()[0].vertex_count, 3);
        assert_eq!(cb.commands()[0].format, FORMAT_COLOR);
    }

    #[test]
    fn test_command_buffer_add_vertices_indexed() {
        let mut cb = VirtualRenderPass::new();
        let state = RenderState::default();

        let vertices = [0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 1.0, 0.0, 0.0, 1.0, 0.0];
        let indices = [0u16, 1, 2, 0, 2, 3];

        let mvp_index = MvpIndex::new(0, 0, 0);
        let (base_vertex, first_index) =
            cb.add_vertices_indexed(0, &vertices, &indices, mvp_index, &state);

        assert_eq!(base_vertex, 0);
        assert_eq!(first_index, 0);
        assert_eq!(cb.commands().len(), 1);
        assert_eq!(cb.commands()[0].vertex_count, 4);
        assert_eq!(cb.commands()[0].index_count, 6);
    }

    #[test]
    fn test_command_buffer_reset() {
        let mut cb = VirtualRenderPass::new();
        let state = RenderState::default();

        let vertices = [0.0f32, 0.0, 0.0, 1.0, 0.0, 0.0, 0.5, 1.0, 0.0];
        let mvp_index = MvpIndex::new(0, 0, 0);
        cb.add_vertices(0, &vertices, mvp_index, &state);

        assert!(!cb.commands().is_empty());

        cb.reset();

        assert!(cb.commands().is_empty());
        assert!(cb.vertex_data(0).is_empty());
    }

    #[test]
    fn test_command_buffer_multiple_batches() {
        let mut cb = VirtualRenderPass::new();
        let state = RenderState::default();

        let v1 = [0.0f32, 0.0, 0.0, 1.0, 0.0, 0.0, 0.5, 1.0, 0.0];
        let mvp_index = MvpIndex::new(0, 0, 0);
        let base1 = cb.add_vertices(0, &v1, mvp_index, &state);

        let v2 = [2.0f32, 0.0, 0.0, 3.0, 0.0, 0.0, 2.5, 1.0, 0.0];
        let base2 = cb.add_vertices(0, &v2, mvp_index, &state);

        assert_eq!(base1, 0);
        assert_eq!(base2, 3);
        assert_eq!(cb.commands().len(), 2);
    }

    #[test]
    fn test_draw_command_creation() {
        let cmd = VRPCommand {
            format: FORMAT_UV,
            mvp_index: MvpIndex::new(0, 0, 0),
            vertex_count: 3,
            index_count: 0,
            base_vertex: 0,
            first_index: 0,
            buffer_source: BufferSource::Immediate,
            texture_slots: [TextureHandle::INVALID; 4],
            color: 0xFFFFFFFF,
            depth_test: true,
            cull_mode: CullMode::Back,
            blend_mode: BlendMode::None,
            matcap_blend_modes: [MatcapBlendMode::Multiply; 4],
        };
        assert_eq!(cmd.format, FORMAT_UV);
        assert_eq!(cmd.vertex_count, 3);
        assert_eq!(cmd.color, 0xFFFFFFFF);
    }

    #[test]
    fn test_draw_command_clone() {
        let cmd = VRPCommand {
            format: FORMAT_COLOR,
            mvp_index: MvpIndex::new(10, 1, 1),
            vertex_count: 100,
            index_count: 150,
            base_vertex: 50,
            first_index: 75,
            buffer_source: BufferSource::Retained,
            texture_slots: [
                TextureHandle(1),
                TextureHandle(2),
                TextureHandle::INVALID,
                TextureHandle::INVALID,
            ],
            color: 0xFF0000FF,
            depth_test: false,
            cull_mode: CullMode::None,
            blend_mode: BlendMode::Alpha,
            matcap_blend_modes: [MatcapBlendMode::Multiply; 4],
        };
        let cloned = cmd.clone();
        assert_eq!(cloned.format, cmd.format);
        assert_eq!(cloned.vertex_count, cmd.vertex_count);
        assert_eq!(cloned.texture_slots, cmd.texture_slots);
    }

    #[test]
    fn test_draw_command_captures_texture_slots() {
        let mut cb = VirtualRenderPass::new();
        let mut state = RenderState::default();

        state.texture_slots[0] = TextureHandle(10);
        state.texture_slots[1] = TextureHandle(20);

        let vertices = [0.0f32, 0.0, 0.0, 1.0, 0.0, 0.0, 0.5, 1.0, 0.0];
        let mvp_index = MvpIndex::new(0, 0, 0);
        cb.add_vertices(0, &vertices, mvp_index, &state);

        assert_eq!(cb.commands()[0].texture_slots[0], TextureHandle(10));
        assert_eq!(cb.commands()[0].texture_slots[1], TextureHandle(20));
        assert_eq!(cb.commands()[0].texture_slots[2], TextureHandle::INVALID);
    }

    #[test]
    fn test_draw_commands_capture_render_state() {
        let mut cb = VirtualRenderPass::new();
        let vertices = [0.0f32, 0.0, 0.0, 1.0, 0.0, 0.0, 0.5, 1.0, 0.0];

        let state1 = RenderState::default();
        let mvp_index = MvpIndex::new(0, 0, 0);
        cb.add_vertices(0, &vertices, mvp_index, &state1);

        let state2 = RenderState {
            color: 0xFF0000FF,
            depth_test: false,
            cull_mode: CullMode::None,
            blend_mode: BlendMode::Alpha,
            ..Default::default()
        };
        cb.add_vertices(0, &vertices, mvp_index, &state2);

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
        let mut cb = VirtualRenderPass::new();
        let state = RenderState::default();

        let v_pos = [0.0f32, 0.0, 0.0, 1.0, 0.0, 0.0, 0.5, 1.0, 0.0];
        let v_pos_uv = [
            0.0, 0.0, 0.0, 0.0, 0.0, 1.0, 0.0, 0.0, 1.0, 0.0, 0.5, 1.0, 0.0, 0.5, 1.0,
        ];

        let mvp_index = MvpIndex::new(0, 0, 0);
        cb.add_vertices(0, &v_pos, mvp_index, &state);
        cb.add_vertices(FORMAT_UV, &v_pos_uv, mvp_index, &state);

        assert_eq!(cb.commands().len(), 2);
        assert_eq!(cb.commands()[0].format, 0);
        assert_eq!(cb.commands()[1].format, FORMAT_UV);
    }

    #[test]
    fn test_command_buffer_mvp_index_capture() {
        let mut cb = VirtualRenderPass::new();
        let state = RenderState::default();
        let vertices = [0.0f32, 0.0, 0.0, 1.0, 0.0, 0.0, 0.5, 1.0, 0.0];

        let mvp_index1 = MvpIndex::new(0, 0, 0);
        let mvp_index2 = MvpIndex::new(10, 1, 1);

        cb.add_vertices(0, &vertices, mvp_index1, &state);
        cb.add_vertices(0, &vertices, mvp_index2, &state);

        assert_eq!(cb.commands()[0].mvp_index, mvp_index1);
        assert_eq!(cb.commands()[1].mvp_index, mvp_index2);
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

        let mvp_index = MvpIndex::new(0, 0, 0);
        let base = cb.add_vertices(0, &large_data, mvp_index, &state);
        assert_eq!(base, 0);
        assert_eq!(cb.commands()[0].vertex_count, 3000);
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

        let mvp_index = MvpIndex::new(0, 0, 0);
        let base = cb.add_vertices(FORMAT_SKINNED, &vertices, mvp_index, &state);

        assert_eq!(base, 0);
        assert_eq!(cb.commands().len(), 1);
        assert_eq!(cb.commands()[0].vertex_count, 3);
        assert_eq!(cb.commands()[0].format, FORMAT_SKINNED);
    }
}
