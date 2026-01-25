//! Virtual Render Pass for batching draw calls
//!
//! Accumulates draw commands during the frame and provides vertex/index data
//! for flushing to the GPU at frame end. This serves as an intermediate
//! representation between FFI commands and GPU execution.

use super::Viewport;
use super::render_state::{CullMode, TextureHandle};
use super::vertex::{VERTEX_FORMAT_COUNT, vertex_stride, vertex_stride_packed};
use zx_common::pack_vertex_data_into;
use std::sync::OnceLock;
use std::time::Instant;

/// Z-index value for commands that don't participate in 2D ordering
///
/// Environment and mesh commands use z_index 0 as they don't use the 2D ordering system.
/// They're sorted by render_type instead.
const NO_Z_INDEX: u32 = 0;

/// Render type for command sorting and pipeline selection
///
/// Determines rendering order and which pipeline to use:
/// - Quad: Screen-space 2D UI (renders first for early-z optimization)
/// - Mesh: 3D geometry (renders second, culled behind UI)
/// - Environment: Procedural background (renders last, fills gaps)
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

    /// Procedural environment background
    ///
    /// Renders last with depth test enabled (LessEqual). Only fragments where
    /// depth == 1.0 (clear value) pass, avoiding expensive environment shader invocations
    /// for pixels already covered by geometry.
    Environment = 2,
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
        /// Resolved to TextureHandle at render time via texture_table.
        textures: [u32; 4],
        cull_mode: CullMode,
        /// Viewport for split-screen rendering (captured at command creation)
        viewport: Viewport,
        /// Pass ID for render pass ordering (execution barrier)
        pass_id: u32,
        /// Cached sort key computed at command creation time
        sort_key: CommandSortKey,
    },
    /// Indexed mesh draw (draw_mesh, load_mesh_indexed)
    IndexedMesh {
        format: u8,
        index_count: u32,
        base_vertex: u32,
        first_index: u32,
        buffer_index: u32,
        /// FFI texture handles captured at command creation time.
        /// Resolved to TextureHandle at render time via texture_table.
        textures: [u32; 4],
        cull_mode: CullMode,
        /// Viewport for split-screen rendering (captured at command creation)
        viewport: Viewport,
        /// Pass ID for render pass ordering (execution barrier)
        pass_id: u32,
        /// Cached sort key computed at command creation time
        sort_key: CommandSortKey,
    },
    /// GPU-instanced quad draw (billboards, sprites, text, rects)
    /// All quads share a single unit quad mesh (4 vertices, 6 indices)
    Quad {
        base_vertex: u32,    // Unit quad base vertex in buffer
        first_index: u32,    // Unit quad first index in buffer
        base_instance: u32,  // Starting instance index in instance buffer
        instance_count: u32, // Number of quad instances to draw
        texture_slots: [TextureHandle; 4],
        cull_mode: CullMode,
        /// Viewport for split-screen rendering (captured at command creation)
        viewport: Viewport,
        /// Pass ID for render pass ordering (execution barrier)
        pass_id: u32,
        /// Z-index for 2D ordering within a pass (higher = closer to camera)
        z_index: u32,
        /// True if screen-space quad (always writes depth), false if billboard (uses PassConfig depth)
        is_screen_space: bool,
        /// Cached sort key computed at command creation time
        sort_key: CommandSortKey,
    },
    /// EPU environment draw (fullscreen procedural background)
    EpuEnvironment {
        /// Index into `mvp_shading_indices` (instance_index) so the environment shader uses the
        /// correct view/proj + shading state.
        mvp_index: u32,
        /// Viewport for split-screen rendering (captured at command creation)
        viewport: Viewport,
        /// Pass ID for render pass ordering (execution barrier)
        pass_id: u32,
        /// Cached sort key computed at command creation time
        sort_key: CommandSortKey,
    },
}

impl VRPCommand {
    #[inline]
    pub fn sort_key(&self) -> CommandSortKey {
        match self {
            VRPCommand::Mesh { sort_key, .. }
            | VRPCommand::IndexedMesh { sort_key, .. }
            | VRPCommand::Quad { sort_key, .. }
            | VRPCommand::EpuEnvironment { sort_key, .. } => *sort_key,
        }
    }
}

/// Sort key for draw command ordering
///
/// Commands are sorted to minimize GPU state changes:
/// 1. Pass ID (preserves render pass ordering - execution barriers)
/// 2. Viewport (split-screen regions)
/// 3. Z-index (2D ordering for quads - higher values render on top)
/// 4. Render type (Quad → Mesh → Environment for optimal early-z)
/// 5. Render state (cull mode)
/// 6. Textures (minimize bind calls)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct CommandSortKey {
    /// Pass ID (highest priority - preserves render pass sequence)
    /// Increments on each begin_pass_*() call
    pub pass_id: u32,
    /// Viewport region (grouped to minimize GPU viewport/scissor changes)
    pub viewport: Viewport,
    /// Z-index for 2D ordering (only used for quads, 0 for other commands)
    pub z_index: u32,
    /// Render type (Quad=0, Mesh=1, Environment=2)
    pub render_type: RenderType,
    /// Vertex format (for regular pipelines)
    pub vertex_format: u8,
    /// Cull mode (none=0, back=1, front=2)
    pub cull_mode: u8,
    /// Texture slots (for grouping by bound textures)
    pub textures: [u32; 4],
}

impl CommandSortKey {
    /// Create sort key for an environment command
    pub fn environment(pass_id: u32, viewport: Viewport) -> Self {
        Self {
            pass_id,
            viewport,
            z_index: NO_Z_INDEX,
            render_type: RenderType::Environment,
            vertex_format: 0,
            cull_mode: 0,
            textures: [0; 4],
        }
    }

    /// Create sort key for a mesh command
    pub fn mesh(
        pass_id: u32,
        viewport: Viewport,
        vertex_format: u8,
        cull_mode: CullMode,
        textures: [u32; 4],
    ) -> Self {
        Self {
            pass_id,
            viewport,
            z_index: NO_Z_INDEX,
            render_type: RenderType::Mesh,
            vertex_format,
            cull_mode: cull_mode as u8,
            textures,
        }
    }

    /// Create sort key for a quad command
    pub fn quad(pass_id: u32, viewport: Viewport, z_index: u32, textures: [u32; 4]) -> Self {
        Self {
            pass_id,
            viewport,
            z_index,
            render_type: RenderType::Quad,
            vertex_format: 0,
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
    /// Per-format immediate vertex data in packed GPU format (CPU side)
    vertex_data: [Vec<u8>; VERTEX_FORMAT_COUNT],
    /// Per-format immediate index data (CPU side, u16 for memory efficiency)
    index_data: [Vec<u16>; VERTEX_FORMAT_COUNT],
    /// Per-format vertex counts for base_vertex calculation
    vertex_counts: [u32; VERTEX_FORMAT_COUNT],
    /// Per-format index counts
    index_counts: [u32; VERTEX_FORMAT_COUNT],

    // Perf counters (only used when NETHERCORE_ZX_PERF is enabled)
    pack_immediate_ns: u64,
}

impl VirtualRenderPass {
    #[inline]
    fn perf_enabled() -> bool {
        static ENABLED: OnceLock<bool> = OnceLock::new();
        *ENABLED.get_or_init(|| std::env::var("NETHERCORE_ZX_PERF").map_or(false, |v| v != "0"))
    }

    /// Create a new command buffer
    pub fn new() -> Self {
        Self {
            commands: Vec::with_capacity(1024),
            vertex_data: std::array::from_fn(|_| Vec::with_capacity(64 * 1024)),
            index_data: std::array::from_fn(|_| Vec::with_capacity(16 * 1024)),
            vertex_counts: [0; VERTEX_FORMAT_COUNT],
            index_counts: [0; VERTEX_FORMAT_COUNT],
            pack_immediate_ns: 0,
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

    #[inline]
    pub fn pack_immediate_ns(&self) -> u64 {
        self.pack_immediate_ns
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
    /// They are resolved to TextureHandle at render time via texture_table.
    #[allow(clippy::too_many_arguments)]
    pub fn record_triangles(
        &mut self,
        format: u8,
        vertex_data: &[f32],
        buffer_index: u32,
        textures: [u32; 4],
        cull_mode: CullMode,
        viewport: Viewport,
        pass_id: u32,
    ) {
        let format_idx = format as usize;
        let stride = vertex_stride(format) as usize;
        let vertex_count = (vertex_data.len() * 4) / stride;
        let base_vertex = self.vertex_counts[format_idx];

        // Pack once at record time and store GPU-ready bytes (avoids per-frame repacking).
        if Self::perf_enabled() {
            let t0 = Instant::now();
            pack_vertex_data_into(vertex_data, format, &mut self.vertex_data[format_idx]);
            self.pack_immediate_ns = self
                .pack_immediate_ns
                .wrapping_add(t0.elapsed().as_nanos() as u64);
        } else {
            pack_vertex_data_into(vertex_data, format, &mut self.vertex_data[format_idx]);
        }
        self.vertex_counts[format_idx] += vertex_count as u32;

        self.commands.push(VRPCommand::Mesh {
            format,
            vertex_count: vertex_count as u32,
            base_vertex,
            buffer_index,
            textures,
            cull_mode,
            viewport,
            pass_id,
            sort_key: CommandSortKey::mesh(pass_id, viewport, format, cull_mode, textures),
        });
    }

    /// Record an indexed triangle draw (called from FFI)
    ///
    /// `textures` contains FFI texture handles captured at command creation time.
    /// They are resolved to TextureHandle at render time via texture_table.
    #[allow(clippy::too_many_arguments)]
    pub fn record_triangles_indexed(
        &mut self,
        format: u8,
        vertex_data: &[f32],
        index_data: &[u16],
        buffer_index: u32,
        textures: [u32; 4],
        cull_mode: CullMode,
        viewport: Viewport,
        pass_id: u32,
    ) {
        let format_idx = format as usize;
        let stride = vertex_stride(format) as usize;
        let vertex_count = (vertex_data.len() * 4) / stride;
        let base_vertex = self.vertex_counts[format_idx];
        let first_index = self.index_counts[format_idx];

        // Pack once at record time and store GPU-ready bytes (avoids per-frame repacking).
        if Self::perf_enabled() {
            let t0 = Instant::now();
            pack_vertex_data_into(vertex_data, format, &mut self.vertex_data[format_idx]);
            self.pack_immediate_ns = self
                .pack_immediate_ns
                .wrapping_add(t0.elapsed().as_nanos() as u64);
        } else {
            pack_vertex_data_into(vertex_data, format, &mut self.vertex_data[format_idx]);
        }
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
            cull_mode,
            viewport,
            pass_id,
            sort_key: CommandSortKey::mesh(pass_id, viewport, format, cull_mode, textures),
        });
    }

    /// Record a mesh draw (called from FFI)
    ///
    /// `textures` contains FFI texture handles captured at command creation time.
    /// They are resolved to TextureHandle at render time via texture_table.
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
        cull_mode: CullMode,
        viewport: Viewport,
        pass_id: u32,
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
                cull_mode,
                viewport,
                pass_id,
                sort_key: CommandSortKey::mesh(
                    pass_id,
                    viewport,
                    mesh_format,
                    cull_mode,
                    textures,
                ),
            });
        } else {
            self.commands.push(VRPCommand::Mesh {
                format: mesh_format,
                vertex_count: mesh_vertex_count,
                base_vertex,
                buffer_index,
                textures,
                cull_mode,
                viewport,
                pass_id,
                sort_key: CommandSortKey::mesh(
                    pass_id,
                    viewport,
                    mesh_format,
                    cull_mode,
                    textures,
                ),
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

    /// Get vertex count for a format (for base_vertex and perf reporting)
    pub fn vertex_count(&self, format: u8) -> u32 {
        self.vertex_counts[format as usize]
    }

    /// Get index count for a format (for perf reporting)
    pub fn index_count(&self, format: u8) -> u32 {
        self.index_counts[format as usize]
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
        self.pack_immediate_ns = 0;
    }
}

impl Default for VirtualRenderPass {
    fn default() -> Self {
        Self::new()
    }
}
