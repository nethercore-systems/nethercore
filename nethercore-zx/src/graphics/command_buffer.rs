//! Virtual Render Pass for batching draw calls
//!
//! Accumulates draw commands during the frame and provides vertex/index data
//! for flushing to the GPU at frame end. This serves as an intermediate
//! representation between FFI commands and GPU execution.

use super::render_state::{CullMode, StencilMode, TextureHandle};
use super::vertex::{VERTEX_FORMAT_COUNT, vertex_stride, vertex_stride_packed};
use super::Viewport;

/// Layer value for commands that don't participate in 2D layer ordering
///
/// Sky and mesh commands use layer 0 as they don't use the 2D layering system.
/// They're sorted by render_type instead.
const NO_LAYER: u32 = 0;

/// Render type for command sorting and pipeline selection
///
/// Determines rendering order and which pipeline to use:
/// - Quad: Screen-space 2D UI (renders first for early-z optimization)
/// - Mesh: 3D geometry (renders second, culled behind UI)
/// - Sky: Procedural background (renders last, fills gaps)
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum RenderType {
    /// Screen-space quads (2D UI, sprites, text)
    ///
    /// Renders first with depth writes enabled at depth=0.0. This allows 3D meshes
    /// behind opaque UI elements to be culled via early-z rejection, saving fragment
    /// shader cost.
    Quad = 0,

    /// 3D meshes and geometry
    ///
    /// Renders second, after 2D UI. Fragments behind opaque UI are culled by
    /// early depth testing.
    Mesh = 1,

    /// Procedural sky background
    ///
    /// Renders last with depth test enabled (LessEqual). Only fragments where
    /// depth == 1.0 (clear value) pass, avoiding expensive sky shader invocations
    /// for pixels already covered by geometry.
    Sky = 2,
}

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
        /// FFI texture handles captured at command creation time.
        /// Resolved to TextureHandle at render time via texture_map.
        textures: [u32; 4],
        depth_test: bool,
        cull_mode: CullMode,
        /// Viewport for split-screen rendering (captured at command creation)
        viewport: Viewport,
        /// Stencil mode for masked rendering
        stencil_mode: StencilMode,
    },
    /// Indexed mesh draw (draw_mesh, load_mesh_indexed)
    IndexedMesh {
        format: u8,
        index_count: u32,
        base_vertex: u32,
        first_index: u32,
        buffer_index: u32,
        /// FFI texture handles captured at command creation time.
        /// Resolved to TextureHandle at render time via texture_map.
        textures: [u32; 4],
        depth_test: bool,
        cull_mode: CullMode,
        /// Viewport for split-screen rendering (captured at command creation)
        viewport: Viewport,
        /// Stencil mode for masked rendering
        stencil_mode: StencilMode,
    },
    /// GPU-instanced quad draw (billboards, sprites, text, rects)
    /// All quads share a single unit quad mesh (4 vertices, 6 indices)
    Quad {
        base_vertex: u32,    // Unit quad base vertex in buffer
        first_index: u32,    // Unit quad first index in buffer
        base_instance: u32,  // Starting instance index in instance buffer
        instance_count: u32, // Number of quad instances to draw
        texture_slots: [TextureHandle; 4],
        depth_test: bool,
        cull_mode: CullMode,
        /// Viewport for split-screen rendering (captured at command creation)
        viewport: Viewport,
        /// Stencil mode for masked rendering
        stencil_mode: StencilMode,
        /// Layer for 2D ordering (higher layers render on top)
        layer: u32,
    },
    /// Sky draw (fullscreen gradient + sun)
    Sky {
        shading_state_index: u32, // Index into shading_states for sky data
        depth_test: bool,         // Should be false (always behind)
        /// Viewport for split-screen rendering (captured at command creation)
        viewport: Viewport,
        /// Stencil mode for masked rendering
        stencil_mode: StencilMode,
    },
}

/// Sort key for draw command ordering
///
/// Commands are sorted to minimize GPU state changes:
/// 1. Viewport (split-screen regions)
/// 2. Layer (2D ordering for quads - higher layers render on top)
/// 3. Stencil mode (masked rendering groups)
/// 4. Render type (Quad → Mesh → Sky for optimal early-z)
/// 5. Render state (depth test, cull mode)
/// 6. Textures (minimize bind calls)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct CommandSortKey {
    /// Viewport region (grouped first to minimize GPU viewport/scissor changes)
    pub viewport: Viewport,
    /// Layer for 2D ordering (only used for quads, 0 for other commands)
    pub layer: u32,
    /// Stencil mode for masked rendering
    pub stencil_mode: StencilMode,
    /// Render type (Quad=0, Mesh=1, Sky=2)
    pub render_type: RenderType,
    /// Vertex format (for regular pipelines)
    pub vertex_format: u8,
    /// Depth test (false=0, true=1)
    pub depth_test: u8,
    /// Cull mode (none=0, back=1, front=2)
    pub cull_mode: u8,
    /// Texture slots (for grouping by bound textures)
    pub textures: [u32; 4],
}

impl CommandSortKey {
    /// Create sort key for a sky command
    pub fn sky(viewport: Viewport, stencil_mode: StencilMode) -> Self {
        Self {
            viewport,
            layer: NO_LAYER,
            stencil_mode,
            render_type: RenderType::Sky,
            vertex_format: 0,
            depth_test: 0,
            cull_mode: 0,
            textures: [0; 4],
        }
    }

    /// Create sort key for a mesh command
    pub fn mesh(
        viewport: Viewport,
        stencil_mode: StencilMode,
        vertex_format: u8,
        depth_test: bool,
        cull_mode: CullMode,
        textures: [u32; 4],
    ) -> Self {
        Self {
            viewport,
            layer: NO_LAYER,
            stencil_mode,
            render_type: RenderType::Mesh,
            vertex_format,
            depth_test: depth_test as u8,
            cull_mode: cull_mode as u8,
            textures,
        }
    }

    /// Create sort key for a quad command
    pub fn quad(
        viewport: Viewport,
        layer: u32,
        stencil_mode: StencilMode,
        depth_test: bool,
        textures: [u32; 4],
    ) -> Self {
        Self {
            viewport,
            layer,
            stencil_mode,
            render_type: RenderType::Quad,
            vertex_format: 0,
            depth_test: depth_test as u8,
            cull_mode: 0,
            textures,
        }
    }
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

    /// Get accumulated commands
    pub fn commands(&self) -> &[VRPCommand] {
        &self.commands
    }

    /// Get mutable access to accumulated commands (for in-place sorting)
    pub fn commands_mut(&mut self) -> &mut [VRPCommand] {
        &mut self.commands
    }

    /// Add a draw command directly
    ///
    /// Used for direct conversion from ZVRPCommand without state mutation.
    pub fn add_command(&mut self, cmd: VRPCommand) {
        self.commands.push(cmd);
    }

    /// Record a non-indexed triangle draw (called from FFI)
    ///
    /// `textures` contains FFI texture handles captured at command creation time.
    /// They are resolved to TextureHandle at render time via texture_map.
    #[allow(clippy::too_many_arguments)]
    pub fn record_triangles(
        &mut self,
        format: u8,
        vertex_data: &[f32],
        buffer_index: u32,
        textures: [u32; 4],
        depth_test: bool,
        cull_mode: CullMode,
        viewport: Viewport,
        stencil_mode: StencilMode,
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
            textures,
            depth_test,
            cull_mode,
            viewport,
            stencil_mode,
        });
    }

    /// Record an indexed triangle draw (called from FFI)
    ///
    /// `textures` contains FFI texture handles captured at command creation time.
    /// They are resolved to TextureHandle at render time via texture_map.
    #[allow(clippy::too_many_arguments)]
    pub fn record_triangles_indexed(
        &mut self,
        format: u8,
        vertex_data: &[f32],
        index_data: &[u16],
        buffer_index: u32,
        textures: [u32; 4],
        depth_test: bool,
        cull_mode: CullMode,
        viewport: Viewport,
        stencil_mode: StencilMode,
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
            textures,
            depth_test,
            cull_mode,
            viewport,
            stencil_mode,
        });
    }

    /// Record a mesh draw (called from FFI)
    ///
    /// `textures` contains FFI texture handles captured at command creation time.
    /// They are resolved to TextureHandle at render time via texture_map.
    #[allow(clippy::too_many_arguments)]
    pub fn record_mesh(
        &mut self,
        mesh_format: u8,
        mesh_vertex_count: u32,
        mesh_index_count: u32,
        mesh_vertex_offset: u64,
        mesh_index_offset: u64,
        buffer_index: u32,
        textures: [u32; 4],
        depth_test: bool,
        cull_mode: CullMode,
        viewport: Viewport,
        stencil_mode: StencilMode,
    ) {
        // Use packed stride since retained meshes are stored in packed format
        let stride = vertex_stride_packed(mesh_format) as u64;
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
                textures,
                depth_test,
                cull_mode,
                viewport,
                stencil_mode,
            });
        } else {
            self.commands.push(VRPCommand::Mesh {
                format: mesh_format,
                vertex_count: mesh_vertex_count,
                base_vertex,
                buffer_index,
                textures,
                depth_test,
                cull_mode,
                viewport,
                stencil_mode,
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
