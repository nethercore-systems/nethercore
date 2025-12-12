//! Frame rendering and presentation
//!
//! This module handles the main rendering loop, including:
//! - Blitting render target to window
//! - Processing and executing draw commands
//! - Managing render passes and GPU state

use glam::Mat4;

use super::command_buffer::{BufferSource, VRPCommand};
use super::pipeline::PipelineKey;
use super::render_state::{BlendMode, CullMode, RenderState, TextureHandle};
use super::vertex::VERTEX_FORMAT_COUNT;
use super::ZGraphics;
use z_common::pack_vertex_data;

impl ZGraphics {
    /// Blit the render target to the window surface
    /// Call this every frame to display the last rendered content
    pub fn blit_to_window(&self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) {
        // Calculate viewport based on scale mode
        let (viewport_x, viewport_y, viewport_width, viewport_height) = match self.scale_mode {
            emberware_core::app::config::ScaleMode::Stretch => {
                // Stretch to fill window (may distort aspect ratio)
                (
                    0.0,
                    0.0,
                    self.config.width as f32,
                    self.config.height as f32,
                )
            }
            emberware_core::app::config::ScaleMode::PixelPerfect => {
                // Integer scaling with letterboxing (pixel-perfect)
                let render_width = self.render_target.width as f32;
                let render_height = self.render_target.height as f32;
                let window_width = self.config.width as f32;
                let window_height = self.config.height as f32;

                // Calculate largest integer scale that fits BOTH dimensions
                let scale_x = (window_width / render_width).floor();
                let scale_y = (window_height / render_height).floor();
                let scale = scale_x.min(scale_y).max(1.0); // At least 1x

                // Calculate scaled dimensions
                let scaled_width = render_width * scale;
                let scaled_height = render_height * scale;

                // Center the viewport
                let x = (window_width - scaled_width) / 2.0;
                let y = (window_height - scaled_height) / 2.0;

                (x, y, scaled_width, scaled_height)
            }
        };

        // Blit to window
        {
            let mut blit_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Blit Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
                    resolve_target: None,
                    ops: wgpu::Operations {
                        load: wgpu::LoadOp::Clear(wgpu::Color::BLACK),
                        store: wgpu::StoreOp::Store,
                    },
                })],
                depth_stencil_attachment: None,
                timestamp_writes: None,
                occlusion_query_set: None,
            });
            blit_pass.set_pipeline(&self.blit_pipeline);
            blit_pass.set_bind_group(0, &self.blit_bind_group, &[]);

            // Set viewport for scaling mode
            blit_pass.set_viewport(
                viewport_x,
                viewport_y,
                viewport_width,
                viewport_height,
                0.0,
                1.0,
            );

            blit_pass.draw(0..3, 0..1);
        }
    }

    pub fn render_frame(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        z_state: &crate::state::ZFFIState,
        texture_map: &hashbrown::HashMap<u32, TextureHandle>,
        clear_color: [f32; 4],
    ) {
        // If no commands, just clear render target
        // (blit is handled separately via blit_to_window())
        if self.command_buffer.commands().is_empty() {
            {
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
                    })],
                    depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                        view: &self.render_target.depth_view,
                        depth_ops: Some(wgpu::Operations {
                            load: wgpu::LoadOp::Clear(1.0),
                            store: wgpu::StoreOp::Store,
                        }),
                        stencil_ops: None,
                    }),
                    timestamp_writes: None,
                    occlusion_query_set: None,
                });
            }
            return;
        }

        // Upload vertex/index data from command buffer to GPU buffers
        for format in 0..VERTEX_FORMAT_COUNT as u8 {
            let vertex_data = self.command_buffer.vertex_data(format);
            if !vertex_data.is_empty() {
                // Convert f32 bytes → f32 slice → packed bytes for GPU
                let floats: &[f32] = bytemuck::cast_slice(vertex_data);
                let packed_data = pack_vertex_data(floats, format);

                self.buffer_manager
                    .vertex_buffer_mut(format)
                    .ensure_capacity(&self.device, packed_data.len() as u64);
                self.buffer_manager
                    .vertex_buffer(format)
                    .write_at(&self.queue, 0, &packed_data);
            }

            let index_data = self.command_buffer.index_data(format);
            if !index_data.is_empty() {
                let index_bytes: &[u8] = bytemuck::cast_slice(index_data);
                self.buffer_manager
                    .index_buffer_mut(format)
                    .ensure_capacity(&self.device, index_bytes.len() as u64);
                self.buffer_manager
                    .index_buffer(format)
                    .write_at(&self.queue, 0, index_bytes);
            }
        }

        // OPTIMIZATION 3: Sort draw commands IN-PLACE by (pipeline_key, texture_slots) to minimize state changes
        // Commands are reset at the start of next frame, so no need to preserve original order or clone
        self.command_buffer
            .commands_mut()
            .sort_unstable_by_key(|cmd| {
                // Extract fields from command variant
                // For sorting, we use raw u32 texture values (consistent ordering is all we need)
                let (format, depth_test, cull_mode, texture_ids, _buffer_index, is_quad, is_sky) =
                    match cmd {
                        VRPCommand::Mesh {
                            format,
                            depth_test,
                            cull_mode,
                            textures,
                            buffer_index,
                            ..
                        } => (
                            *format,
                            *depth_test,
                            *cull_mode,
                            *textures, // FFI handles as [u32; 4]
                            Some(*buffer_index),
                            false,
                            false,
                        ),
                        VRPCommand::IndexedMesh {
                            format,
                            depth_test,
                            cull_mode,
                            textures,
                            buffer_index,
                            ..
                        } => (
                            *format,
                            *depth_test,
                            *cull_mode,
                            *textures, // FFI handles as [u32; 4]
                            Some(*buffer_index),
                            false,
                            false,
                        ),
                        VRPCommand::Quad {
                            depth_test,
                            cull_mode,
                            texture_slots,
                            ..
                        } => (
                            self.unit_quad_format,
                            *depth_test,
                            *cull_mode,
                            // Extract inner u32 from TextureHandle for consistent sort key type
                            [
                                texture_slots[0].0,
                                texture_slots[1].0,
                                texture_slots[2].0,
                                texture_slots[3].0,
                            ],
                            None,
                            true,
                            false,
                        ),
                        VRPCommand::Sky { depth_test, .. } => {
                            // Sky uses unique sort key to render first (before all geometry)
                            (
                                0,
                                *depth_test,
                                super::render_state::CullMode::None,
                                [0u32; 4], // Sky doesn't use textures
                                None,
                                false,
                                true,
                            )
                        }
                    };

                // Extract blend mode from command directly (now stored per-command)
                let blend_mode = match cmd {
                    VRPCommand::Mesh { blend_mode, .. } => *blend_mode,
                    VRPCommand::IndexedMesh { blend_mode, .. } => *blend_mode,
                    VRPCommand::Quad { blend_mode, .. } => *blend_mode,
                    VRPCommand::Sky { .. } => BlendMode::None,
                };

                // Sort key: (render_mode, format, blend_mode, depth_test, cull_mode, texture_slots)
                // This groups commands by pipeline first, then by textures
                // Note: texture_filter is no longer part of pipeline state - it's in
                // PackedUnifiedShadingState.flags for per-draw shader selection
                let state = RenderState {
                    depth_test,
                    cull_mode,
                    blend_mode,
                };

                // Create sort key based on pipeline type (Regular vs Quad vs Sky)
                let (render_mode, vertex_format, blend_mode_u8, depth_test_u8, cull_mode_u8) =
                    if is_sky {
                        // Sky pipeline: Use lowest sort key to render first (before all geometry)
                        (0u8, 0u8, 0u8, 0u8, 0u8)
                    } else if is_quad {
                        // Quad pipeline: Use special values to group separately
                        let pipeline_key = PipelineKey::quad(&state);
                        match pipeline_key {
                            PipelineKey::Quad {
                                blend_mode,
                                depth_test,
                            } => (255u8, 255u8, blend_mode, depth_test as u8, 0u8),
                            _ => unreachable!(),
                        }
                    } else {
                        // Regular pipeline: Use actual values
                        let pipeline_key =
                            PipelineKey::new(self.current_render_mode, format, &state);
                        match pipeline_key {
                            PipelineKey::Regular {
                                render_mode,
                                vertex_format,
                                blend_mode,
                                depth_test,
                                cull_mode,
                            } => (
                                render_mode,
                                vertex_format,
                                blend_mode,
                                depth_test as u8,
                                cull_mode,
                            ),
                            _ => unreachable!(),
                        }
                    };

                (
                    render_mode,
                    vertex_format,
                    blend_mode_u8,
                    depth_test_u8,
                    cull_mode_u8,
                    texture_ids[0],
                    texture_ids[1],
                    texture_ids[2],
                    texture_ids[3],
                )
            });

        // Upload matrices from z_state to GPU storage buffers
        // 1. Upload model matrices
        if !z_state.model_matrices.is_empty() {
            self.ensure_model_buffer_capacity(z_state.model_matrices.len());
            let data = bytemuck::cast_slice(&z_state.model_matrices);
            self.queue.write_buffer(&self.model_matrix_buffer, 0, data);
        }

        // 2. Upload view matrices
        if !z_state.view_matrices.is_empty() {
            self.ensure_view_buffer_capacity(z_state.view_matrices.len());
            let data = bytemuck::cast_slice(&z_state.view_matrices);
            self.queue.write_buffer(&self.view_matrix_buffer, 0, data);
        }

        // 3. Upload projection matrices
        if !z_state.proj_matrices.is_empty() {
            self.ensure_proj_buffer_capacity(z_state.proj_matrices.len());
            let data = bytemuck::cast_slice(&z_state.proj_matrices);
            self.queue.write_buffer(&self.proj_matrix_buffer, 0, data);
        }

        // 4. Upload shading states (NEW - Phase 5)
        if !z_state.shading_states.is_empty() {
            self.ensure_shading_state_buffer_capacity(z_state.shading_states.len());
            let data = bytemuck::cast_slice(&z_state.shading_states);
            self.queue.write_buffer(&self.shading_state_buffer, 0, data);
        }

        // 5. Upload MVP + shading state indices (already deduplicated by add_mvp_shading_state)
        // WGSL: array<vec4<u32>> - unpacked indices use all 4 fields naturally (no bit-packing!)
        // Each entry is 4 × u32: [model_idx, view_idx, proj_idx, shading_idx]
        let state_count = z_state.mvp_shading_states.len();
        if state_count > 0 {
            self.ensure_mvp_indices_buffer_capacity(state_count);
            let data = bytemuck::cast_slice(&z_state.mvp_shading_states);
            self.queue.write_buffer(&self.mvp_indices_buffer, 0, data);
        }

        // 6. Upload bone matrices (3x4 format, 12 floats per bone)
        if !z_state.bone_matrices.is_empty() {
            let bone_count = z_state.bone_matrices.len().min(256);
            let mut bone_data: Vec<f32> = Vec::with_capacity(bone_count * 12);
            for matrix in &z_state.bone_matrices[..bone_count] {
                bone_data.extend_from_slice(&matrix.to_array());
            }
            self.queue
                .write_buffer(&self.bone_buffer, 0, bytemuck::cast_slice(&bone_data));
        }

        // 7. Upload inverse bind matrices when a skeleton is bound
        if let Some(skeleton) = z_state.get_bound_skeleton() {
            let bone_count = skeleton.bone_count as usize;
            let mut inverse_bind_data: Vec<f32> = Vec::with_capacity(bone_count * 12);
            for matrix in &skeleton.inverse_bind[..bone_count] {
                inverse_bind_data.extend_from_slice(&matrix.to_array());
            }
            self.queue.write_buffer(
                &self.inverse_bind_buffer,
                0,
                bytemuck::cast_slice(&inverse_bind_data),
            );
        }

        // Take texture cache out temporarily to avoid nested mutable borrows during render pass.
        // Cache is persistent across frames - entries are reused when keys match.
        let mut texture_bind_groups = std::mem::take(&mut self.texture_bind_groups);

        // Create frame bind group once per frame (same for all draws)
        // Get bind group layout from first pipeline (all pipelines have same frame layout)
        let frame_bind_group = if let Some(first_cmd) = self.command_buffer.commands().first() {
            // Extract fields from first command variant
            let (format, depth_test, cull_mode) = match first_cmd {
                VRPCommand::Mesh {
                    format,
                    depth_test,
                    cull_mode,
                    ..
                } => (*format, *depth_test, *cull_mode),
                VRPCommand::IndexedMesh {
                    format,
                    depth_test,
                    cull_mode,
                    ..
                } => (*format, *depth_test, *cull_mode),
                VRPCommand::Quad {
                    depth_test,
                    cull_mode,
                    ..
                } => (self.unit_quad_format, *depth_test, *cull_mode),
                VRPCommand::Sky { depth_test, .. } => {
                    // Sky uses its own pipeline, but we need values for bind group layout
                    (0, *depth_test, CullMode::None)
                }
            };

            let first_state = RenderState {
                depth_test,
                cull_mode,
                blend_mode: BlendMode::None, // Doesn't matter for layout
            };
            let pipeline_entry = self.pipeline_cache.get_or_create(
                &self.device,
                self.config.format,
                self.current_render_mode,
                format,
                &first_state,
            );

            self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                label: Some("Frame Bind Group (Unified)"),
                layout: &pipeline_entry.bind_group_layout_frame,
                entries: &[
                    wgpu::BindGroupEntry {
                        binding: 0,
                        resource: self.model_matrix_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 1,
                        resource: self.view_matrix_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 2,
                        resource: self.proj_matrix_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 3,
                        resource: self.shading_state_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 4,
                        resource: self.mvp_indices_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 5,
                        resource: self.bone_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 6,
                        resource: self.inverse_bind_buffer.as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 7,
                        resource: self
                            .buffer_manager
                            .quad_instance_buffer()
                            .as_entire_binding(),
                    },
                    wgpu::BindGroupEntry {
                        binding: 8,
                        resource: self.screen_dims_buffer.as_entire_binding(),
                    },
                ],
            })
        } else {
            // No commands to render, nothing to do
            return;
        };

        // Render pass - render game content to offscreen target
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Game Render Pass"),
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
                })],
                depth_stencil_attachment: Some(wgpu::RenderPassDepthStencilAttachment {
                    view: &self.render_target.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            // State tracking to skip redundant GPU calls (commands are sorted by pipeline/texture)
            let mut bound_pipeline: Option<PipelineKey> = None;
            let mut bound_texture_slots: Option<[TextureHandle; 4]> = None;
            let mut bound_vertex_format: Option<(u8, BufferSource)> = None;
            let mut frame_bind_group_set = false;

            // Helper closure to resolve FFI texture handles to TextureHandle
            let resolve_textures = |textures: &[u32; 4]| -> [TextureHandle; 4] {
                [
                    texture_map
                        .get(&textures[0])
                        .copied()
                        .unwrap_or(TextureHandle::INVALID),
                    texture_map
                        .get(&textures[1])
                        .copied()
                        .unwrap_or(TextureHandle::INVALID),
                    texture_map
                        .get(&textures[2])
                        .copied()
                        .unwrap_or(TextureHandle::INVALID),
                    texture_map
                        .get(&textures[3])
                        .copied()
                        .unwrap_or(TextureHandle::INVALID),
                ]
            };

            for cmd in self.command_buffer.commands() {
                // Destructure command variant to extract common fields
                // For Mesh/IndexedMesh: resolve FFI texture handles to TextureHandle
                // For Quad: use texture_slots directly (already TextureHandle)
                let (
                    format,
                    depth_test,
                    cull_mode,
                    texture_slots,
                    buffer_source,
                    is_quad,
                    is_sky,
                    cmd_blend_mode,
                ) = match cmd {
                    VRPCommand::Mesh {
                        format,
                        depth_test,
                        cull_mode,
                        textures,
                        buffer_index,
                        blend_mode,
                        ..
                    } => (
                        *format,
                        *depth_test,
                        *cull_mode,
                        resolve_textures(textures), // Resolve FFI handles at render time
                        BufferSource::Immediate(*buffer_index),
                        false,
                        false,
                        Some(*blend_mode),
                    ),
                    VRPCommand::IndexedMesh {
                        format,
                        depth_test,
                        cull_mode,
                        textures,
                        buffer_index,
                        blend_mode,
                        ..
                    } => (
                        *format,
                        *depth_test,
                        *cull_mode,
                        resolve_textures(textures), // Resolve FFI handles at render time
                        BufferSource::Retained(*buffer_index),
                        false,
                        false,
                        Some(*blend_mode),
                    ),
                    VRPCommand::Quad {
                        depth_test,
                        cull_mode,
                        blend_mode,
                        texture_slots,
                        ..
                    } => (
                        self.unit_quad_format,
                        *depth_test,
                        *cull_mode,
                        *texture_slots, // Already TextureHandle
                        BufferSource::Quad,
                        true,
                        false,
                        Some(*blend_mode),
                    ),
                    VRPCommand::Sky { depth_test, .. } => (
                        self.unit_quad_format, // Sky uses unit quad mesh
                        *depth_test,
                        super::render_state::CullMode::None,
                        [TextureHandle::INVALID; 4], // Default textures (unused)
                        BufferSource::Quad,          // Sky renders as a fullscreen quad
                        false,
                        true,
                        None,
                    ),
                };

                // Blend mode is now stored per-command for all variants
                let blend_mode = if is_sky {
                    // Sky uses default alpha blending (sky gradient blends with nothing behind)
                    BlendMode::Alpha
                } else {
                    cmd_blend_mode.expect("All non-sky commands should have blend_mode")
                };

                // Create render state from command + blend mode
                let state = RenderState {
                    depth_test,
                    cull_mode,
                    blend_mode,
                };

                // Get/create pipeline - use sky/quad/regular pipeline based on command type
                if is_sky {
                    // Sky rendering: Ensure sky pipeline exists
                    self.pipeline_cache
                        .get_or_create_sky(&self.device, self.config.format);
                } else if is_quad {
                    // Quad rendering: Ensure quad pipeline exists
                    self.pipeline_cache.get_or_create_quad(
                        &self.device,
                        self.config.format,
                        &state,
                    );
                } else {
                    // Regular mesh rendering: Ensure format-specific pipeline exists
                    if !self
                        .pipeline_cache
                        .contains(self.current_render_mode, format, &state)
                    {
                        self.pipeline_cache.get_or_create(
                            &self.device,
                            self.config.format,
                            self.current_render_mode,
                            format,
                            &state,
                        );
                    }
                }

                // Now get immutable reference to pipeline entry (avoiding borrow issues)
                let pipeline_key = if is_sky {
                    PipelineKey::Sky
                } else if is_quad {
                    PipelineKey::quad(&state)
                } else {
                    PipelineKey::new(self.current_render_mode, format, &state)
                };

                let pipeline_entry = self
                    .pipeline_cache
                    .get_by_key(&pipeline_key)
                    .expect("Pipeline should exist after get_or_create");

                // Get or create texture bind group (cached by texture slots)
                let texture_bind_group =
                    texture_bind_groups.entry(texture_slots).or_insert_with(|| {
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
                    });

                // Set pipeline (only if changed)
                if bound_pipeline != Some(pipeline_key) {
                    render_pass.set_pipeline(&pipeline_entry.pipeline);
                    bound_pipeline = Some(pipeline_key);
                }

                // Set frame bind group once (unified across all draws)
                if !frame_bind_group_set {
                    render_pass.set_bind_group(0, &frame_bind_group, &[]);
                    frame_bind_group_set = true;
                }

                // Set texture bind group (only if changed)
                if bound_texture_slots != Some(texture_slots) {
                    render_pass.set_bind_group(1, &*texture_bind_group, &[]);
                    bound_texture_slots = Some(texture_slots);
                }

                // Set vertex buffer (only if format or buffer source changed)
                if bound_vertex_format != Some((format, buffer_source)) {
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
                    bound_vertex_format = Some((format, buffer_source));
                }

                // Handle different rendering paths based on command variant
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

                        tracing::info!(
                            "Drawing {} quad instances at base_instance {} (indices {}..{}, base_vertex {}, textures: {:?})",
                            instance_count,
                            base_instance,
                            first_index,
                            first_index + UNIT_QUAD_INDEX_COUNT,
                            base_vertex,
                            texture_slots
                        );

                        // Indexed draw with GPU instancing (quads always use indices)
                        let index_buffer = self
                            .buffer_manager
                            .retained_index_buffer(self.unit_quad_format);
                        if let Some(buffer) = index_buffer.buffer() {
                            render_pass
                                .set_index_buffer(buffer.slice(..), wgpu::IndexFormat::Uint16);
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
                            BufferSource::Retained(_) => {
                                self.buffer_manager.retained_index_buffer(format)
                            }
                            BufferSource::Quad => unreachable!(),
                        };
                        if let Some(buffer) = index_buffer.buffer() {
                            render_pass
                                .set_index_buffer(buffer.slice(..), wgpu::IndexFormat::Uint16);
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
                    VRPCommand::Sky {
                        shading_state_index,
                        ..
                    } => {
                        // Sky rendering: Fullscreen triangle with procedural gradient
                        tracing::info!(
                            "Drawing sky with shading_state_index {} (vertices 0..3)",
                            shading_state_index
                        );

                        // Draw fullscreen triangle (3 vertices, no vertex buffer)
                        // Uses shading_state_index as instance range to pass index to shader
                        render_pass.draw(0..3, *shading_state_index..*shading_state_index + 1);
                    }
                }
            }
        }

        // Move texture cache back into self (preserving allocations for next frame)
        self.texture_bind_groups = texture_bind_groups;

        // NOTE: Blit is now handled separately via blit_to_window()
        // This allows us to re-blit the last rendered frame on high refresh rate monitors
        // without re-rendering the game content
    }

    /// Ensure model matrix buffer has sufficient capacity
    pub(super) fn ensure_model_buffer_capacity(&mut self, count: usize) {
        if count <= self.model_matrix_capacity {
            return;
        }

        let new_capacity = (count * 2).next_power_of_two();
        tracing::debug!(
            "Growing model matrix buffer: {} → {}",
            self.model_matrix_capacity,
            new_capacity
        );

        self.model_matrix_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Model Matrices"),
            size: (new_capacity * std::mem::size_of::<Mat4>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.model_matrix_capacity = new_capacity;
    }

    /// Ensure view matrix buffer has sufficient capacity
    pub(super) fn ensure_view_buffer_capacity(&mut self, count: usize) {
        if count <= self.view_matrix_capacity {
            return;
        }

        let new_capacity = (count * 2).next_power_of_two();
        tracing::debug!(
            "Growing view matrix buffer: {} → {}",
            self.view_matrix_capacity,
            new_capacity
        );

        self.view_matrix_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("View Matrices"),
            size: (new_capacity * std::mem::size_of::<Mat4>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.view_matrix_capacity = new_capacity;
    }

    /// Ensure projection matrix buffer has sufficient capacity
    pub(super) fn ensure_proj_buffer_capacity(&mut self, count: usize) {
        if count <= self.proj_matrix_capacity {
            return;
        }

        let new_capacity = (count * 2).next_power_of_two();
        tracing::debug!(
            "Growing projection matrix buffer: {} → {}",
            self.proj_matrix_capacity,
            new_capacity
        );

        self.proj_matrix_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Projection Matrices"),
            size: (new_capacity * std::mem::size_of::<Mat4>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.proj_matrix_capacity = new_capacity;
    }

    /// Ensure MVP indices buffer has sufficient capacity
    pub(super) fn ensure_mvp_indices_buffer_capacity(&mut self, count: usize) {
        if count <= self.mvp_indices_capacity {
            return;
        }

        let new_capacity = (count * 2).next_power_of_two();
        tracing::debug!(
            "Growing MVP indices buffer: {} → {}",
            self.mvp_indices_capacity,
            new_capacity
        );

        self.mvp_indices_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("MVP Indices"),
            size: (new_capacity * 2 * std::mem::size_of::<u32>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.mvp_indices_capacity = new_capacity;
    }

    /// Ensure shading state buffer has sufficient capacity
    pub(super) fn ensure_shading_state_buffer_capacity(&mut self, count: usize) {
        if count <= self.shading_state_capacity {
            return;
        }

        let new_capacity = (count * 2).next_power_of_two();
        tracing::debug!(
            "Growing shading state buffer: {} → {}",
            self.shading_state_capacity,
            new_capacity
        );

        self.shading_state_buffer = self.device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Shading States"),
            size: (new_capacity * std::mem::size_of::<super::PackedUnifiedShadingState>()) as u64,
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });
        self.shading_state_capacity = new_capacity;
    }
}
