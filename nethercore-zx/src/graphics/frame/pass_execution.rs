//! Render pass creation and command execution
//!
//! This module handles:
//! - Creating render passes with appropriate load/store operations
//! - Executing draw commands with state tracking
//! - Managing pipeline, bind group, and vertex buffer state

use super::super::command_buffer::{BufferSource, VRPCommand};
use super::super::pipeline::{PipelineEntry, PipelineKey};
use super::super::render_state::{RenderState, TextureHandle};
use super::super::TextureHandleTable;
use super::super::ZXGraphics;
use crate::state::ZXFFIState;
use hashbrown::HashMap;

/// State tracked during render pass execution to minimize redundant GPU state changes.
struct RenderPassState {
    current_viewport: Option<super::super::Viewport>,
    current_pass_id: Option<u32>,
    bound_pipeline: Option<PipelineKey>,
    bound_texture_slots: Option<[TextureHandle; 4]>,
    bound_vertex_format: Option<(u8, BufferSource)>,
    frame_bind_group_set: bool,
}

impl RenderPassState {
    fn new() -> Self {
        Self {
            current_viewport: None,
            current_pass_id: None,
            bound_pipeline: None,
            bound_texture_slots: None,
            bound_vertex_format: None,
            frame_bind_group_set: false,
        }
    }
}

impl ZXGraphics {
    /// Execute the clear pass when there are no draw commands.
    pub(super) fn execute_clear_pass(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        clear_color: [f32; 4],
    ) {
        let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
            label: Some("Clear Pass"),
            color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                view: &self.render_target.color_view,
                resolve_target: None,
                ops: wgpu::Operations {
                    load: wgpu::LoadOp::Clear(wgpu::Color {
                        r: clear_color[0] as f64,
                        g: clear_color[1] as f64,
                        b: clear_color[2] as f64,
                        a: clear_color[3] as f64,
                    }),
                    store: wgpu::StoreOp::Store,
                },
                depth_slice: None,
            })],
            depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                view: &self.render_target.depth_view,
                depth_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(1.0),
                    store: wgpu::StoreOp::Store,
                }),
                stencil_ops: Some(wgpu::Operations {
                    load: wgpu::LoadOp::Clear(0),
                    store: wgpu::StoreOp::Store,
                }),
            }),
            timestamp_writes: None,
            occlusion_query_set: None,
        });
    }

    /// Execute all render passes for the frame.
    /// Commands are processed in segments, restarting render pass when depth_clear is needed.
    pub(super) fn execute_render_passes(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        z_state: &ZXFFIState,
        texture_table: &TextureHandleTable,
        clear_color: [f32; 4],
        frame_bind_group: &wgpu::BindGroup,
        texture_bind_groups: &mut HashMap<[TextureHandle; 4], wgpu::BindGroup>,
        perf_enabled: bool,
    ) {
        // Helper closure to resolve FFI texture handles to TextureHandle
        let resolve_textures =
            |textures: &[u32; 4]| -> [TextureHandle; 4] { texture_table.resolve4(textures) };

        // Process commands in segments, restarting render pass when depth_clear is needed
        // Commands are sorted by pass_id, so all commands from the same pass are contiguous
        let commands = self.command_buffer.commands();
        let mut cmd_idx = 0;

        // First render pass: clear color, depth, and stencil
        let mut is_first_pass = true;

        while cmd_idx < commands.len() {
            if perf_enabled {
                self.perf.render_pass_segments = self.perf.render_pass_segments.wrapping_add(1);
            }

            // Determine what load ops we need for this render pass segment
            let first_cmd = &commands[cmd_idx];
            let first_pass_id = match first_cmd {
                VRPCommand::Mesh { pass_id, .. }
                | VRPCommand::IndexedMesh { pass_id, .. }
                | VRPCommand::Quad { pass_id, .. }
                | VRPCommand::EpuEnvironment { pass_id, .. } => *pass_id,
            };
            let first_pass_config = z_state
                .pass_configs
                .get(first_pass_id as usize)
                .copied()
                .unwrap_or_default();

            // Determine load ops based on whether this is the first pass and depth_clear flag
            let (color_load, depth_load, stencil_load) = if is_first_pass {
                // First pass: always clear color/depth/stencil
                (
                    wgpu::LoadOp::Clear(wgpu::Color {
                        r: clear_color[0] as f64,
                        g: clear_color[1] as f64,
                        b: clear_color[2] as f64,
                        a: clear_color[3] as f64,
                    }),
                    wgpu::LoadOp::Clear(1.0),
                    wgpu::LoadOp::Clear(0),
                )
            } else if first_pass_config.depth_clear {
                // Mid-frame depth clear: preserve color, clear depth, preserve stencil
                (
                    wgpu::LoadOp::Load,
                    wgpu::LoadOp::Clear(1.0),
                    wgpu::LoadOp::Load,
                )
            } else {
                // No clear needed: preserve everything
                (wgpu::LoadOp::Load, wgpu::LoadOp::Load, wgpu::LoadOp::Load)
            };

            // Create render pass for this segment
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Game Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view: &self.render_target.color_view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: color_load,
                        store: wgpu::StoreOp::Store,
                    },
                    depth_slice: None,
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.render_target.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: depth_load,
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: Some(wgpu::Operations {
                        load: stencil_load,
                        store: wgpu::StoreOp::Store,
                    }),
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            // State tracking (reset for each render pass segment)
            let mut state = RenderPassState::new();

            // Process commands until we hit a pass that needs depth clear
            while cmd_idx < commands.len() {
                let cmd = &commands[cmd_idx];

                // Destructure command variant to extract common fields
                // For Mesh/IndexedMesh: resolve FFI texture handles to TextureHandle
                // For Quad: use texture_slots directly (already TextureHandle)
                let (
                    cmd_viewport,
                    cmd_pass_id,
                    format,
                    cull_mode,
                    texture_slots,
                    buffer_source,
                    is_quad,
                    is_environment,
                    is_screen_space_quad,
                ) = match cmd {
                    VRPCommand::Mesh {
                        format,
                        cull_mode,
                        textures,
                        buffer_index,
                        viewport,
                        pass_id,
                        ..
                    } => (
                        *viewport,
                        *pass_id,
                        *format,
                        *cull_mode,
                        resolve_textures(textures), // Resolve FFI handles at render time
                        BufferSource::Immediate(*buffer_index),
                        false,
                        false,
                        false,
                    ),
                    VRPCommand::IndexedMesh {
                        format,
                        cull_mode,
                        textures,
                        buffer_index,
                        viewport,
                        pass_id,
                        ..
                    } => (
                        *viewport,
                        *pass_id,
                        *format,
                        *cull_mode,
                        resolve_textures(textures), // Resolve FFI handles at render time
                        BufferSource::Retained(*buffer_index),
                        false,
                        false,
                        false,
                    ),
                    VRPCommand::Quad {
                        cull_mode,
                        texture_slots,
                        viewport,
                        pass_id,
                        is_screen_space,
                        ..
                    } => (
                        *viewport,
                        *pass_id,
                        self.unit_quad_format,
                        *cull_mode,
                        *texture_slots, // Already TextureHandle
                        BufferSource::Quad,
                        true,
                        false,
                        *is_screen_space,
                    ),
                    VRPCommand::EpuEnvironment {
                        viewport, pass_id, ..
                    } => (
                        *viewport,
                        *pass_id,
                        self.unit_quad_format, // EPU environment uses unit quad mesh
                        super::super::render_state::CullMode::None,
                        [TextureHandle::INVALID; 4], // Default textures (unused)
                        BufferSource::Quad,          // Environment renders as a fullscreen quad
                        false,
                        true, // is_environment = true for EPU
                        false,
                    ),
                };

                // Get PassConfig for this command's pass
                let cmd_pass_config = z_state
                    .pass_configs
                    .get(cmd_pass_id as usize)
                    .copied()
                    .unwrap_or_default();

                // Check if this pass needs depth clear and we're not already in a fresh pass
                // If so, break out to restart the render pass with the correct load ops
                if state.current_pass_id.is_some()
                    && state.current_pass_id != Some(cmd_pass_id)
                    && cmd_pass_config.depth_clear
                {
                    // Don't increment cmd_idx - we'll process this command in the next render pass
                    break;
                }

                // Set viewport and scissor rect if changed (split-screen support)
                if state.current_viewport != Some(cmd_viewport) {
                    render_pass.set_viewport(
                        cmd_viewport.x as f32,
                        cmd_viewport.y as f32,
                        cmd_viewport.width as f32,
                        cmd_viewport.height as f32,
                        0.0,
                        1.0,
                    );
                    render_pass.set_scissor_rect(
                        cmd_viewport.x,
                        cmd_viewport.y,
                        cmd_viewport.width,
                        cmd_viewport.height,
                    );
                    state.current_viewport = Some(cmd_viewport);
                }

                // Set stencil reference if pass changed
                if state.current_pass_id != Some(cmd_pass_id) {
                    // Set stencil reference from PassConfig
                    if cmd_pass_config.is_stencil_active() {
                        render_pass.set_stencil_reference(cmd_pass_config.stencil_ref as u32);
                    }
                    state.current_pass_id = Some(cmd_pass_id);
                }

                // Create render state from command (depth_test derived from PassConfig)
                let render_state = RenderState {
                    depth_test: cmd_pass_config.depth_write,
                    cull_mode,
                };

                // Get/create pipeline - use environment/quad/regular pipeline based on command type
                if is_environment {
                    // Environment rendering: Ensure environment pipeline exists
                    self.pipeline_cache.get_or_create_environment(
                        &self.device,
                        self.config.format,
                        &cmd_pass_config,
                    );
                } else if is_quad {
                    // Quad rendering: Ensure quad pipeline exists
                    // Screen-space quads always write depth (early-z optimization)
                    // Billboards use PassConfig depth settings (they're 3D positioned)
                    self.pipeline_cache.get_or_create_quad(
                        &self.device,
                        self.config.format,
                        &cmd_pass_config,
                        is_screen_space_quad,
                    );
                } else {
                    // Regular mesh rendering: Ensure format-specific pipeline exists
                    if !self.pipeline_cache.contains(
                        self.current_render_mode,
                        format,
                        &render_state,
                        &cmd_pass_config,
                    ) {
                        self.pipeline_cache.get_or_create(
                            &self.device,
                            self.config.format,
                            self.current_render_mode,
                            format,
                            &render_state,
                            &cmd_pass_config,
                        );
                    }
                }

                // Now get immutable reference to pipeline entry (avoiding borrow issues)
                let pipeline_key = if is_environment {
                    PipelineKey::environment(&cmd_pass_config)
                } else if is_quad {
                    PipelineKey::quad(&cmd_pass_config, is_screen_space_quad)
                } else {
                    PipelineKey::new(self.current_render_mode, format, &render_state, &cmd_pass_config)
                };

                let pipeline_entry = self
                    .pipeline_cache
                    .get_by_key(&pipeline_key)
                    .expect("Pipeline should exist after get_or_create");

                // Get or create texture bind group (cached by texture slots)
                let texture_bind_group =
                    texture_bind_groups.entry(texture_slots).or_insert_with(|| {
                        self.create_texture_bind_group(texture_slots, pipeline_entry)
                    });

                // Set pipeline (only if changed)
                if state.bound_pipeline != Some(pipeline_key) {
                    render_pass.set_pipeline(&pipeline_entry.pipeline);
                    state.bound_pipeline = Some(pipeline_key);
                    if perf_enabled {
                        self.perf.pipeline_switches = self.perf.pipeline_switches.wrapping_add(1);
                    }
                }

                // Set frame bind group once (unified across all draws)
                if !state.frame_bind_group_set {
                    render_pass.set_bind_group(0, frame_bind_group, &[]);
                    state.frame_bind_group_set = true;
                }

                // Set texture bind group (only if changed)
                if state.bound_texture_slots != Some(texture_slots) {
                    render_pass.set_bind_group(1, &*texture_bind_group, &[]);
                    state.bound_texture_slots = Some(texture_slots);
                }

                // Set vertex buffer (only if format or buffer source changed)
                if state.bound_vertex_format != Some((format, buffer_source)) {
                    let vertex_buffer = match buffer_source {
                        BufferSource::Immediate(_) => self.buffer_manager.vertex_buffer(format),
                        BufferSource::Retained(_) => {
                            self.buffer_manager.retained_vertex_buffer(format)
                        }
                        BufferSource::Quad => {
                            // Quad instancing uses unit quad mesh (format: POS_UV_COLOR)
                            self.buffer_manager
                                .retained_vertex_buffer(self.unit_quad_format)
                        }
                    };
                    if let Some(buffer) = vertex_buffer.buffer() {
                        render_pass.set_vertex_buffer(0, buffer.slice(..));
                    }
                    state.bound_vertex_format = Some((format, buffer_source));
                }

                // Execute the draw command
                self.execute_draw_command(cmd, &mut render_pass, buffer_source, format);

                // Move to next command
                cmd_idx += 1;
            }
            // Inner while loop ends - render_pass is dropped here, ending the GPU pass

            // No longer the first pass - subsequent passes preserve color
            is_first_pass = false;
        }
        // Outer while loop ends
    }

    /// Create a texture bind group for the given texture slots.
    fn create_texture_bind_group(
        &self,
        texture_slots: [TextureHandle; 4],
        pipeline_entry: &PipelineEntry,
    ) -> wgpu::BindGroup {
        // Get texture views for this command's bound textures
        let tex_view_0 = if texture_slots[0] == TextureHandle::INVALID {
            self.get_fallback_white_view()
        } else {
            self.get_texture_view(texture_slots[0])
                .unwrap_or_else(|| self.get_fallback_checkerboard_view())
        };
        let tex_view_1 = if texture_slots[1] == TextureHandle::INVALID {
            self.get_fallback_white_view()
        } else {
            self.get_texture_view(texture_slots[1])
                .unwrap_or_else(|| self.get_fallback_checkerboard_view())
        };
        let tex_view_2 = if texture_slots[2] == TextureHandle::INVALID {
            self.get_fallback_white_view()
        } else {
            self.get_texture_view(texture_slots[2])
                .unwrap_or_else(|| self.get_fallback_checkerboard_view())
        };
        let tex_view_3 = if texture_slots[3] == TextureHandle::INVALID {
            self.get_fallback_white_view()
        } else {
            self.get_texture_view(texture_slots[3])
                .unwrap_or_else(|| self.get_fallback_checkerboard_view())
        };

        self.device.create_bind_group(&wgpu::BindGroupDescriptor {
            label: Some("Texture Bind Group"),
            layout: &pipeline_entry.bind_group_layout_textures,
            entries: &[
                wgpu::BindGroupEntry {
                    binding: 0,
                    resource: wgpu::BindingResource::TextureView(tex_view_0),
                },
                wgpu::BindGroupEntry {
                    binding: 1,
                    resource: wgpu::BindingResource::TextureView(tex_view_1),
                },
                wgpu::BindGroupEntry {
                    binding: 2,
                    resource: wgpu::BindingResource::TextureView(tex_view_2),
                },
                wgpu::BindGroupEntry {
                    binding: 3,
                    resource: wgpu::BindingResource::TextureView(tex_view_3),
                },
                // Both samplers bound - shader selects via shading state flag
                wgpu::BindGroupEntry {
                    binding: 4,
                    resource: wgpu::BindingResource::Sampler(&self.sampler_nearest),
                },
                wgpu::BindGroupEntry {
                    binding: 5,
                    resource: wgpu::BindingResource::Sampler(&self.sampler_linear),
                },
            ],
        })
    }

    /// Execute a single draw command.
    fn execute_draw_command(
        &self,
        cmd: &VRPCommand,
        render_pass: &mut wgpu::RenderPass<'_>,
        buffer_source: BufferSource,
        format: u8,
    ) {
        match cmd {
            VRPCommand::Quad {
                instance_count,
                base_instance,
                base_vertex,
                first_index,
                ..
            } => {
                // Quad rendering: Instance data comes from storage buffer binding(6)
                // The quad shader reads QuadInstance data via @builtin(instance_index)
                // No per-instance vertex attributes needed (unlike old approach)
                // Unit quad: 4 vertices, 6 indices (2 triangles)

                const UNIT_QUAD_INDEX_COUNT: u32 = 6;

                // Indexed draw with GPU instancing (quads always use indices)
                let index_buffer = self
                    .buffer_manager
                    .retained_index_buffer(self.unit_quad_format);
                if let Some(buffer) = index_buffer.buffer() {
                    render_pass.set_index_buffer(buffer.slice(..), wgpu::IndexFormat::Uint16);
                    render_pass.draw_indexed(
                        *first_index..*first_index + UNIT_QUAD_INDEX_COUNT,
                        *base_vertex as i32,
                        *base_instance..*base_instance + *instance_count,
                    );
                } else {
                    tracing::error!("Quad index buffer is None!");
                }
            }
            VRPCommand::IndexedMesh {
                index_count,
                base_vertex,
                first_index,
                buffer_index,
                ..
            } => {
                // Indexed mesh: MVP instancing with storage buffer lookup
                let index_buffer = match buffer_source {
                    BufferSource::Immediate(_) => self.buffer_manager.index_buffer(format),
                    BufferSource::Retained(_) => self.buffer_manager.retained_index_buffer(format),
                    BufferSource::Quad => unreachable!(),
                };
                if let Some(buffer) = index_buffer.buffer() {
                    render_pass.set_index_buffer(buffer.slice(..), wgpu::IndexFormat::Uint16);
                    render_pass.draw_indexed(
                        *first_index..*first_index + *index_count,
                        *base_vertex as i32,
                        *buffer_index..*buffer_index + 1,
                    );
                }
            }
            VRPCommand::Mesh {
                vertex_count,
                base_vertex,
                buffer_index,
                ..
            } => {
                // Non-indexed mesh: MVP instancing with storage buffer lookup
                render_pass.draw(
                    *base_vertex..*base_vertex + *vertex_count,
                    *buffer_index..*buffer_index + 1,
                );
            }
            VRPCommand::EpuEnvironment { mvp_index, .. } => {
                // EPU environment rendering: Fullscreen triangle with procedural background
                // Uses the new instruction-based EPU compute pipeline

                // Draw fullscreen triangle (3 vertices, no vertex buffer)
                // Uses mvp_index as instance range (indexes mvp_shading_indices)
                render_pass.draw(0..3, *mvp_index..*mvp_index + 1);
            }
        }
    }
}
