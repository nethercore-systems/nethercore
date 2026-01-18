//! Main frame rendering logic
//!
//! This module handles:
//! - Processing and sorting draw commands
//! - Uploading per-frame data to GPU buffers
//! - Managing render passes and GPU state
//! - Executing draw commands with state tracking

use super::bind_group_cache::BindGroupKey;
use super::super::ZXGraphics;
use super::super::command_buffer::{BufferSource, CommandSortKey, VRPCommand};
use super::super::pipeline::PipelineKey;
use super::super::render_state::{CullMode, PassConfig, RenderState, TextureHandle};
use super::super::vertex::VERTEX_FORMAT_COUNT;
use glam::Mat4;
use zx_common::pack_vertex_data;

impl ZXGraphics {
    pub fn render_frame(
        &mut self,
        encoder: &mut wgpu::CommandEncoder,
        z_state: &crate::state::ZXFFIState,
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
                    .ensure_capacity(&self.device, &self.queue, packed_data.len() as u64);
                self.buffer_manager
                    .vertex_buffer(format)
                    .write_at(&self.queue, 0, &packed_data);
            }

            let index_data = self.command_buffer.index_data(format);
            if !index_data.is_empty() {
                let index_bytes: &[u8] = bytemuck::cast_slice(index_data);
                self.buffer_manager
                    .index_buffer_mut(format)
                    .ensure_capacity(&self.device, &self.queue, index_bytes.len() as u64);
                self.buffer_manager
                    .index_buffer(format)
                    .write_at(&self.queue, 0, index_bytes);
            }
        }

        // OPTIMIZATION 3: Sort draw commands IN-PLACE by CommandSortKey to minimize state changes
        // Commands are reset at the start of next frame, so no need to preserve original order or clone
        // Sort order: pass_id → viewport → z_index → render_type → cull → textures
        self.command_buffer
            .commands_mut()
            .sort_unstable_by_key(|cmd| match cmd {
                VRPCommand::Mesh {
                    format,
                    cull_mode,
                    textures,
                    viewport,
                    pass_id,
                    ..
                } => CommandSortKey::mesh(*pass_id, *viewport, *format, *cull_mode, *textures),
                VRPCommand::IndexedMesh {
                    format,
                    cull_mode,
                    textures,
                    viewport,
                    pass_id,
                    ..
                } => CommandSortKey::mesh(*pass_id, *viewport, *format, *cull_mode, *textures),
                VRPCommand::Quad {
                    texture_slots,
                    viewport,
                    pass_id,
                    z_index,
                    ..
                } => CommandSortKey::quad(
                    *pass_id,
                    *viewport,
                    *z_index,
                    [
                        texture_slots[0].0,
                        texture_slots[1].0,
                        texture_slots[2].0,
                        texture_slots[3].0,
                    ],
                ),
                VRPCommand::Environment {
                    viewport, pass_id, ..
                } => CommandSortKey::environment(*pass_id, *viewport),
            });

        // =================================================================
        // UNIFIED BUFFER UPLOADS
        // =================================================================

        // 1. Upload unified transforms: [models | views | projs]
        let model_count = z_state.model_matrices.len();
        let view_count = z_state.view_matrices.len();
        let proj_count = z_state.proj_matrices.len();
        let total_transforms = model_count + view_count + proj_count;

        if total_transforms > 0 {
            self.ensure_unified_transforms_capacity(total_transforms);

            // Build contiguous data: models, then views, then projs
            let mut transform_data =
                Vec::with_capacity(total_transforms * std::mem::size_of::<Mat4>());
            transform_data.extend_from_slice(bytemuck::cast_slice(&z_state.model_matrices));
            transform_data.extend_from_slice(bytemuck::cast_slice(&z_state.view_matrices));
            transform_data.extend_from_slice(bytemuck::cast_slice(&z_state.proj_matrices));

            self.queue
                .write_buffer(&self.unified_transforms_buffer, 0, &transform_data);
        }

        // 2. Upload shading states
        if !z_state.shading_pool.is_empty() {
            self.ensure_shading_state_buffer_capacity(z_state.shading_pool.len());
            let data = bytemuck::cast_slice(z_state.shading_pool.as_slice());
            self.queue.write_buffer(&self.shading_state_buffer, 0, data);
        }

        // 2b. Upload environment states (Multi-Environment v4)
        if !z_state.environment_pool.is_empty() {
            self.ensure_environment_states_buffer_capacity(z_state.environment_pool.len());
            let data = bytemuck::cast_slice(z_state.environment_pool.as_slice());
            self.queue
                .write_buffer(&self.environment_states_buffer, 0, data);
        }

        // 3. Upload MVP + shading indices with ABSOLUTE offsets into unified_transforms
        // CPU pre-computes absolute indices so shader does direct lookup without offset arithmetic
        // view_idx → view_idx + model_count
        // proj_idx → proj_idx + model_count + view_count
        let state_count = z_state.mvp_shading_states.len();
        if state_count > 0 {
            self.ensure_mvp_indices_buffer_capacity(state_count);

            // Transform relative indices to absolute indices
            let view_offset = model_count as u32;
            let proj_offset = (model_count + view_count) as u32;

            let absolute_indices: Vec<super::super::MvpShadingIndices> = z_state
                .mvp_shading_states
                .iter()
                .map(|idx| super::super::MvpShadingIndices {
                    model_idx: idx.model_idx,
                    view_idx: idx.view_idx + view_offset,
                    proj_idx: idx.proj_idx + proj_offset,
                    shading_idx: idx.shading_idx,
                })
                .collect();

            let data = bytemuck::cast_slice(&absolute_indices);
            self.queue.write_buffer(&self.mvp_indices_buffer, 0, data);
        }

        // 6. Upload immediate bone matrices to unified_animation (dynamic section)
        // Bones are appended after static data (inverse_bind + keyframes)
        if !z_state.bone_matrices.is_empty() {
            let bone_count = z_state.bone_matrices.len().min(256);
            let mut bone_data: Vec<f32> = Vec::with_capacity(bone_count * 12);
            for matrix in &z_state.bone_matrices[..bone_count] {
                bone_data.extend_from_slice(&matrix.to_array());
            }
            // Write after static sections (inverse_bind + keyframes)
            let byte_offset = (self.animation_static_end * 48) as u64;
            self.queue.write_buffer(
                &self.unified_animation_buffer,
                byte_offset,
                bytemuck::cast_slice(&bone_data),
            );
        }

        // NOTE: Inverse bind matrices are uploaded once during init via upload_static_inverse_bind()
        // They live in unified_animation[0..inverse_bind_end]

        // Take texture cache out temporarily to avoid nested mutable borrows during render pass.
        // Cache is persistent across frames - entries are reused when keys match.
        let mut texture_bind_groups = std::mem::take(&mut self.texture_bind_groups);

        // Create or reuse cached frame bind group (same for all draws)
        // Get bind group layout from first pipeline (all pipelines have same frame layout)
        //
        // Bind group caching: The frame bind group only needs recreation when:
        // 1. Buffer capacities change (buffers are recreated)
        // 2. Render mode changes (different bind group layout)
        // This saves ~0.1ms/frame on typical hardware by avoiding descriptor set churn.
        let frame_bind_group = if let Some(first_cmd) = self.command_buffer.commands().first() {
            // Extract fields from first command variant
            // Note: depth_test is per-pass via PassConfig, but we use defaults for bind group layout
            let (format, cull_mode, pass_id) = match first_cmd {
                VRPCommand::Mesh {
                    format,
                    cull_mode,
                    pass_id,
                    ..
                } => (*format, *cull_mode, *pass_id),
                VRPCommand::IndexedMesh {
                    format,
                    cull_mode,
                    pass_id,
                    ..
                } => (*format, *cull_mode, *pass_id),
                VRPCommand::Quad {
                    cull_mode, pass_id, ..
                } => (self.unit_quad_format, *cull_mode, *pass_id),
                VRPCommand::Environment { pass_id, .. } => {
                    // Environment uses its own pipeline, but we need values for bind group layout
                    (0, CullMode::None, *pass_id)
                }
            };

            // Get PassConfig for the first command's pass to determine depth state
            let pass_config = z_state
                .pass_configs
                .get(pass_id as usize)
                .copied()
                .unwrap_or_default();

            // Compute hash based on buffer capacities and render mode
            // When any capacity changes, buffer is recreated and bind group must be recreated
            let bind_group_hash = BindGroupKey {
                unified_transforms_capacity: self.unified_transforms_capacity,
                unified_animation_capacity: self.unified_animation_capacity,
                shading_state_capacity: self.shading_state_capacity,
                mvp_indices_capacity: self.mvp_indices_capacity,
                render_mode: self.current_render_mode,
                quad_instance_capacity: self.buffer_manager.quad_instance_capacity(),
            }
            .hash_value();

            // Check if cached bind group is still valid
            if let Some(ref cached) = self.cached_frame_bind_group {
                if self.cached_frame_bind_group_hash == bind_group_hash {
                    // Reuse cached bind group
                    cached.clone()
                } else {
                    // Hash changed, need to recreate
                    // Derive depth state from PassConfig for this pass
                    let first_state = RenderState {
                        depth_test: pass_config.depth_write,
                        cull_mode,
                    };
                    // Use default PassConfig for bind group layout - all pipelines share the same layout
                    let pipeline_entry = self.pipeline_cache.get_or_create(
                        &self.device,
                        self.config.format,
                        self.current_render_mode,
                        format,
                        &first_state,
                        &PassConfig::default(), // Bind group layout is same for all pass configs
                    );

                    // Bind group layout (grouped by purpose):
                    // 0-1: Transforms (unified_transforms, mvp_indices)
                    // 2: Shading (shading_states)
                    // 3: Animation (unified_animation)
                    // 4: Environment (environment_states) - Multi-Environment v4
                    // 5: Quad rendering (quad_instances)
                    let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                        label: Some("Frame Bind Group (Unified)"),
                        layout: &pipeline_entry.bind_group_layout_frame,
                        entries: &[
                            wgpu::BindGroupEntry {
                                binding: 0,
                                resource: self.unified_transforms_buffer.as_entire_binding(),
                            },
                            wgpu::BindGroupEntry {
                                binding: 1,
                                resource: self.mvp_indices_buffer.as_entire_binding(),
                            },
                            wgpu::BindGroupEntry {
                                binding: 2,
                                resource: self.shading_state_buffer.as_entire_binding(),
                            },
                            wgpu::BindGroupEntry {
                                binding: 3,
                                resource: self.unified_animation_buffer.as_entire_binding(),
                            },
                            wgpu::BindGroupEntry {
                                binding: 4,
                                resource: self.environment_states_buffer.as_entire_binding(),
                            },
                            wgpu::BindGroupEntry {
                                binding: 5,
                                resource: self
                                    .buffer_manager
                                    .quad_instance_buffer()
                                    .as_entire_binding(),
                            },
                        ],
                    });
                    self.cached_frame_bind_group = Some(bind_group.clone());
                    self.cached_frame_bind_group_hash = bind_group_hash;
                    bind_group
                }
            } else {
                // No cached bind group, create new one
                // Derive depth state from PassConfig for this pass
                let first_state = RenderState {
                    depth_test: pass_config.depth_write,
                    cull_mode,
                };
                // Use default PassConfig for bind group layout - all pipelines share the same layout
                let pipeline_entry = self.pipeline_cache.get_or_create(
                    &self.device,
                    self.config.format,
                    self.current_render_mode,
                    format,
                    &first_state,
                    &PassConfig::default(), // Bind group layout is same for all pass configs
                );

                // Bind group layout (grouped by purpose):
                // 0-1: Transforms (unified_transforms, mvp_indices)
                // 2: Shading (shading_states)
                // 3: Animation (unified_animation)
                // 4: Environment (environment_states) - Multi-Environment v4
                // 5: Quad rendering (quad_instances)
                let bind_group = self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                    label: Some("Frame Bind Group (Unified)"),
                    layout: &pipeline_entry.bind_group_layout_frame,
                    entries: &[
                        wgpu::BindGroupEntry {
                            binding: 0,
                            resource: self.unified_transforms_buffer.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 1,
                            resource: self.mvp_indices_buffer.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 2,
                            resource: self.shading_state_buffer.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 3,
                            resource: self.unified_animation_buffer.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 4,
                            resource: self.environment_states_buffer.as_entire_binding(),
                        },
                        wgpu::BindGroupEntry {
                            binding: 5,
                            resource: self
                                .buffer_manager
                                .quad_instance_buffer()
                                .as_entire_binding(),
                        },
                    ],
                });
                self.cached_frame_bind_group = Some(bind_group.clone());
                self.cached_frame_bind_group_hash = bind_group_hash;
                bind_group
            }
        } else {
            // No commands to render, nothing to do
            return;
        };

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

        // Process commands in segments, restarting render pass when depth_clear is needed
        // Commands are sorted by pass_id, so all commands from the same pass are contiguous
        let commands = self.command_buffer.commands();
        let mut cmd_idx = 0;

        // First render pass: clear color, depth, and stencil
        let mut is_first_pass = true;

        while cmd_idx < commands.len() {
            // Determine what load ops we need for this render pass segment
            let first_cmd = &commands[cmd_idx];
            let first_pass_id = match first_cmd {
                VRPCommand::Mesh { pass_id, .. }
                | VRPCommand::IndexedMesh { pass_id, .. }
                | VRPCommand::Quad { pass_id, .. }
                | VRPCommand::Environment { pass_id, .. } => *pass_id,
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
            let mut current_viewport: Option<super::super::Viewport> = None;
            let mut current_pass_id: Option<u32> = None;
            let mut bound_pipeline: Option<PipelineKey> = None;
            let mut bound_texture_slots: Option<[TextureHandle; 4]> = None;
            let mut bound_vertex_format: Option<(u8, BufferSource)> = None;
            let mut frame_bind_group_set = false;

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
                    VRPCommand::Environment {
                        viewport, pass_id, ..
                    } => (
                        *viewport,
                        *pass_id,
                        self.unit_quad_format, // Environment uses unit quad mesh
                        super::super::render_state::CullMode::None,
                        [TextureHandle::INVALID; 4], // Default textures (unused)
                        BufferSource::Quad,          // Environment renders as a fullscreen quad
                        false,
                        true,
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
                if current_pass_id.is_some()
                    && current_pass_id != Some(cmd_pass_id)
                    && cmd_pass_config.depth_clear
                {
                    // Don't increment cmd_idx - we'll process this command in the next render pass
                    break;
                }

                // Set viewport and scissor rect if changed (split-screen support)
                if current_viewport != Some(cmd_viewport) {
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
                    current_viewport = Some(cmd_viewport);
                }

                // Set stencil reference if pass changed
                if current_pass_id != Some(cmd_pass_id) {
                    // Set stencil reference from PassConfig
                    if cmd_pass_config.is_stencil_active() {
                        render_pass.set_stencil_reference(cmd_pass_config.stencil_ref as u32);
                    }
                    current_pass_id = Some(cmd_pass_id);
                }

                // Create render state from command (depth_test derived from PassConfig)
                let state = RenderState {
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
                        &state,
                        &cmd_pass_config,
                    ) {
                        self.pipeline_cache.get_or_create(
                            &self.device,
                            self.config.format,
                            self.current_render_mode,
                            format,
                            &state,
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
                    PipelineKey::new(self.current_render_mode, format, &state, &cmd_pass_config)
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
                    VRPCommand::Environment {
                        shading_state_index,
                        ..
                    } => {
                        // Environment rendering: Fullscreen triangle with procedural background

                        // Draw fullscreen triangle (3 vertices, no vertex buffer)
                        // Uses shading_state_index as instance range to pass index to shader
                        render_pass.draw(0..3, *shading_state_index..*shading_state_index + 1);
                    }
                }

                // Move to next command
                cmd_idx += 1;
            }
            // Inner while loop ends - render_pass is dropped here, ending the GPU pass

            // No longer the first pass - subsequent passes preserve color
            is_first_pass = false;
        }
        // Outer while loop ends

        // Move texture cache back into self (preserving allocations for next frame)
        self.texture_bind_groups = texture_bind_groups;

        // NOTE: Blit is handled separately via blit_to_window()
        // This allows us to re-blit the last rendered frame on high refresh rate monitors
        // without re-rendering the game content
    }
}
