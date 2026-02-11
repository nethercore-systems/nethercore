//! Performance metrics collection for frame rendering
//!
//! This module handles tracking performance metrics like:
//! - Command counts by type
//! - Unique texture set combinations
//! - Per-format vertex/index byte counts
//! - Pack immediate timing

use super::super::ZXGraphics;
use super::super::command_buffer::VRPCommand;
use super::super::vertex::{VERTEX_FORMAT_COUNT, vertex_stride};

impl ZXGraphics {
    /// Collect performance metrics for the current frame's commands.
    /// This should be called at the start of render_frame when perf is enabled.
    pub(super) fn collect_frame_perf_metrics(&mut self) {
        self.perf.frames = self.perf.frames.wrapping_add(1);
        self.perf.pack_immediate_ns = self
            .perf
            .pack_immediate_ns
            .wrapping_add(self.command_buffer.pack_immediate_ns());

        // Per-format immediate byte counts (pre-pack f32 vs packed GPU bytes).
        for format in 0..VERTEX_FORMAT_COUNT as u8 {
            let format_idx = format as usize;
            let vertex_count = self.command_buffer.vertex_count(format) as u64;
            let prepack_bytes = vertex_count * vertex_stride(format) as u64;
            let packed_bytes = self.command_buffer.vertex_data(format).len() as u64;
            let index_bytes = (self.command_buffer.index_count(format) as u64) * 2;

            self.perf.immediate_vertex_prepack_bytes[format_idx] =
                self.perf.immediate_vertex_prepack_bytes[format_idx].wrapping_add(prepack_bytes);
            self.perf.immediate_vertex_packed_bytes[format_idx] =
                self.perf.immediate_vertex_packed_bytes[format_idx].wrapping_add(packed_bytes);
            self.perf.immediate_index_bytes[format_idx] =
                self.perf.immediate_index_bytes[format_idx].wrapping_add(index_bytes);
        }

        // Command counts + unique texture-slot combinations (FFI handles for meshes, TextureHandle IDs for quads).
        self.perf.texture_set_scratch.clear();
        for cmd in self.command_buffer.commands() {
            match cmd {
                VRPCommand::Mesh { textures, .. } => {
                    self.perf.cmd_mesh = self.perf.cmd_mesh.wrapping_add(1);
                    self.perf.texture_set_scratch.insert(*textures);
                }
                VRPCommand::IndexedMesh { textures, .. } => {
                    self.perf.cmd_indexed_mesh = self.perf.cmd_indexed_mesh.wrapping_add(1);
                    self.perf.texture_set_scratch.insert(*textures);
                }
                VRPCommand::Quad { texture_slots, .. } => {
                    self.perf.cmd_quad = self.perf.cmd_quad.wrapping_add(1);
                    self.perf.texture_set_scratch.insert([
                        texture_slots[0].0,
                        texture_slots[1].0,
                        texture_slots[2].0,
                        texture_slots[3].0,
                    ]);
                }
                VRPCommand::EpuEnvironment { .. } => {
                    self.perf.cmd_environment = self.perf.cmd_environment.wrapping_add(1);
                }
            }
        }
        self.perf.unique_texture_sets = self
            .perf
            .unique_texture_sets
            .wrapping_add(self.perf.texture_set_scratch.len() as u64);
    }
}
