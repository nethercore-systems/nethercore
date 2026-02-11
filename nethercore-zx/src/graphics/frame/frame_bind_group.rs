//! Frame bind group creation and caching
//!
//! This module handles creating and caching the frame bind group that is shared
//! across all draw calls. The bind group only needs recreation when:
//! - Buffer capacities change (buffers are recreated)
//! - Render mode changes (different bind group layout)
//! - EPU resources are recreated

use super::super::ZXGraphics;
use super::super::command_buffer::VRPCommand;
use super::super::render_state::{CullMode, PassConfig, RenderState};
use super::bind_group_cache::BindGroupKey;
use crate::state::ZXFFIState;

impl ZXGraphics {
    /// Create or retrieve cached frame bind group.
    /// Returns None if there are no commands to render.
    pub(super) fn get_or_create_frame_bind_group(
        &mut self,
        z_state: &ZXFFIState,
    ) -> Option<wgpu::BindGroup> {
        let first_cmd = self.command_buffer.commands().first()?;

        // The bind group layout is shared across pipelines. However, in render modes 1-3 the
        // mesh pipeline requires normals; the first sorted command is often a Quad or
        // EpuEnvironment which uses a non-normal vertex format. Use a safe mesh format when
        // creating a pipeline solely to obtain the bind group layout.
        let bind_group_format = if self.current_render_mode > 0 {
            crate::graphics::FORMAT_NORMAL
        } else {
            self.unit_quad_format
        };

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
            } => (bind_group_format, *cull_mode, *pass_id),
            VRPCommand::EpuEnvironment { pass_id, .. } => {
                // EPU environment uses its own pipeline
                (bind_group_format, CullMode::None, *pass_id)
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
            epu_resource_version: self.epu_runtime.resource_version(),
        }
        .hash_value();

        // Check if cached bind group is still valid
        if let Some(ref cached) = self.cached_frame_bind_group {
            if self.cached_frame_bind_group_hash == bind_group_hash {
                // Reuse cached bind group
                return Some(cached.clone());
            }
        }

        // Need to create new bind group (hash changed or no cached bind group)
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
        // 5: Quad rendering (quad_instances)
        // 6-7: EPU textures (env_radiance, sampler)
        // 8-9: EPU state + frame uniforms
        // 11: EPU SH9 (diffuse irradiance)
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
                    binding: 5,
                    resource: self
                        .buffer_manager
                        .quad_instance_buffer()
                        .as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 6,
                    resource: wgpu::BindingResource::TextureView(
                        self.epu_runtime.env_radiance_view(),
                    ),
                },
                wgpu::BindGroupEntry {
                    binding: 7,
                    resource: wgpu::BindingResource::Sampler(&self.epu_sampler),
                },
                wgpu::BindGroupEntry {
                    binding: 8,
                    resource: self.epu_runtime.env_states_buffer().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 9,
                    resource: self.epu_runtime.frame_uniforms_buffer().as_entire_binding(),
                },
                wgpu::BindGroupEntry {
                    binding: 11,
                    resource: self.epu_runtime.sh9_buffer().as_entire_binding(),
                },
            ],
        });
        self.cached_frame_bind_group = Some(bind_group.clone());
        self.cached_frame_bind_group_hash = bind_group_hash;
        Some(bind_group)
    }
}
