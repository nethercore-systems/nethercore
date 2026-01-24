//! Pipeline cache management
//!
//! Manages caching of compiled render pipelines and shader modules.

use hashbrown::HashMap;

use super::super::render_state::{PassConfig, RenderState};
use super::pipeline_creation::{
    PipelineEntry, create_environment_pipeline, create_pipeline, create_quad_pipeline,
};
use super::pipeline_key::PipelineKey;

/// Cache for render pipelines
///
/// Stores compiled pipelines keyed by their render state configuration.
/// Pipelines are created on-demand and reused across frames.
/// Shader modules are created on-demand and cached by (mode, format).
pub struct PipelineCache {
    pipelines: HashMap<PipelineKey, PipelineEntry>,
    /// Cached shader modules for main mesh pipelines, keyed by (render_mode, format).
    shader_modules: HashMap<(u8, u8), wgpu::ShaderModule>,
    quad_shader_module: Option<wgpu::ShaderModule>,
    environment_shader_module: Option<wgpu::ShaderModule>,
    precompiled_render_modes: u8,
}

impl PipelineCache {
    /// Create an empty pipeline cache
    pub fn new() -> Self {
        Self {
            pipelines: HashMap::new(),
            shader_modules: HashMap::new(),
            quad_shader_module: None,
            environment_shader_module: None,
            precompiled_render_modes: 0,
        }
    }

    /// Whether shader precompilation should run during graphics initialization.
    ///
    /// Defaults:
    /// - Debug builds: off (faster startup; shaders compile on-demand)
    /// - Release builds: on (avoids first-use stutters)
    ///
    /// Override with `NETHERCORE_PRECOMPILE_SHADERS=0|1`.
    pub fn should_precompile_shaders() -> bool {
        match std::env::var("NETHERCORE_PRECOMPILE_SHADERS") {
            Ok(v) => parse_env_bool(&v).unwrap_or_else(|| {
                tracing::warn!(
                    "Invalid NETHERCORE_PRECOMPILE_SHADERS value '{}'; expected 0/1/true/false",
                    v
                );
                !cfg!(debug_assertions)
            }),
            Err(_) => !cfg!(debug_assertions),
        }
    }

    /// Precompile the shader modules needed for a given render mode.
    ///
    /// This should be called during graphics initialization to ensure all shaders
    /// compile successfully. Panics on any shader compilation failure, indicating
    /// a bug in shader generation.
    pub fn precompile_shaders_for_render_mode(&mut self, device: &wgpu::Device, render_mode: u8) {
        let render_mode = render_mode.min(3);
        let mask = 1u8 << render_mode;
        if self.precompiled_render_modes & mask != 0 {
            return;
        }

        tracing::info!(
            "Precompiling shaders for render mode {} ({})...",
            render_mode,
            crate::shader_gen::mode_name(render_mode)
        );

        // Always warm up shared shaders that are used by 2D/UI or default backgrounds.
        let _ = self.get_or_create_quad_shader_module(device);
        let _ = self.get_or_create_environment_shader_module(device);

        // Compile mesh shaders for all valid vertex formats in this mode.
        let formats = crate::shader_gen::valid_formats_for_mode(render_mode);

        for format in formats {
            let _ = self.get_or_create_mesh_shader_module(device, render_mode, format);
        }

        self.precompiled_render_modes |= mask;
        tracing::info!("Shader precompile complete for mode {}", render_mode);
    }

    fn get_or_create_mesh_shader_module(
        &mut self,
        device: &wgpu::Device,
        render_mode: u8,
        format: u8,
    ) -> &wgpu::ShaderModule {
        use crate::graphics::FORMAT_NORMAL;
        use crate::shader_gen::generate_shader;

        let render_mode = render_mode.min(3);
        let key = (render_mode, format);
        if self.shader_modules.contains_key(&key) {
            return &self.shader_modules[&key];
        }

        if render_mode > 0 && (format & FORMAT_NORMAL) == 0 {
            panic!(
                "Vertex format {} missing normals for render mode {} ({})",
                format,
                render_mode,
                crate::shader_gen::mode_name(render_mode)
            );
        }

        let shader_source = generate_shader(render_mode, format).unwrap_or_else(|e| {
            panic!(
                "Shader generation failed for mode {} format {}: {}",
                render_mode, format, e
            )
        });

        let label = format!("Mode{}_Format{}", render_mode, format);

        let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
            label: Some(&label),
            source: wgpu::ShaderSource::Wgsl(shader_source.into()),
        });

        self.shader_modules.insert(key, module);
        &self.shader_modules[&key]
    }

    fn get_or_create_quad_shader_module(&mut self, device: &wgpu::Device) -> &wgpu::ShaderModule {
        if self.quad_shader_module.is_none() {
            self.quad_shader_module =
                Some(device.create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("Quad Shader"),
                    source: wgpu::ShaderSource::Wgsl(crate::shader_gen::QUAD_SHADER.into()),
                }));
        }

        self.quad_shader_module
            .as_ref()
            .expect("Quad shader module just inserted")
    }

    fn get_or_create_environment_shader_module(
        &mut self,
        device: &wgpu::Device,
    ) -> &wgpu::ShaderModule {
        if self.environment_shader_module.is_none() {
            self.environment_shader_module =
                Some(device.create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some("Environment Shader"),
                    source: wgpu::ShaderSource::Wgsl(crate::shader_gen::ENVIRONMENT_SHADER.into()),
                }));
        }

        self.environment_shader_module
            .as_ref()
            .expect("Environment shader module just inserted")
    }

    /// Get or create a pipeline for the given state
    ///
    /// Returns a reference to the cached pipeline, creating it if necessary.
    pub fn get_or_create(
        &mut self,
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        render_mode: u8,
        format: u8,
        state: &RenderState,
        pass_config: &PassConfig,
    ) -> &PipelineEntry {
        let key = PipelineKey::new(render_mode, format, state, pass_config);

        // Return existing pipeline if cached
        if self.pipelines.contains_key(&key) {
            return &self.pipelines[&key];
        }

        // Otherwise, create a new pipeline
        tracing::debug!(
            "Creating pipeline: mode={}, format={}, depth={}, cull={:?}, pass_config={:?}",
            render_mode,
            format,
            state.depth_test,
            state.cull_mode,
            pass_config
        );

        let shader_module = self.get_or_create_mesh_shader_module(device, render_mode, format);
        let entry = create_pipeline(
            device,
            surface_format,
            render_mode,
            format,
            shader_module,
            state,
            pass_config,
        );
        self.pipelines.insert(key, entry);
        &self.pipelines[&key]
    }

    /// Check if a pipeline exists in the cache
    pub fn contains(
        &self,
        render_mode: u8,
        format: u8,
        state: &RenderState,
        pass_config: &PassConfig,
    ) -> bool {
        let key = PipelineKey::new(render_mode, format, state, pass_config);
        self.pipelines.contains_key(&key)
    }

    /// Get or create a quad pipeline
    ///
    /// Returns a reference to the cached quad pipeline, creating it if necessary.
    /// `is_screen_space` determines depth behavior:
    /// - true (screen-space): always writes depth at 0 for early-z optimization
    /// - false (billboard): uses PassConfig depth settings
    pub fn get_or_create_quad(
        &mut self,
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        pass_config: &PassConfig,
        is_screen_space: bool,
    ) -> &PipelineEntry {
        let key = PipelineKey::quad(pass_config, is_screen_space);

        // Return existing pipeline if cached
        if self.pipelines.contains_key(&key) {
            return &self.pipelines[&key];
        }

        // Otherwise, create a new quad pipeline
        tracing::debug!(
            "Creating quad pipeline: is_screen_space={}, pass_config={:?}",
            is_screen_space,
            pass_config
        );

        let shader_module = self.get_or_create_quad_shader_module(device);
        let entry = create_quad_pipeline(
            device,
            surface_format,
            shader_module,
            pass_config,
            is_screen_space,
        );
        self.pipelines.insert(key, entry);
        &self.pipelines[&key]
    }

    /// Get or create an environment pipeline
    ///
    /// Returns a reference to the cached environment pipeline, creating it if necessary.
    pub fn get_or_create_environment(
        &mut self,
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        pass_config: &PassConfig,
    ) -> &PipelineEntry {
        let key = PipelineKey::environment(pass_config);

        // Return existing pipeline if cached
        if self.pipelines.contains_key(&key) {
            return &self.pipelines[&key];
        }

        // Otherwise, create a new environment pipeline
        tracing::debug!(
            "Creating environment pipeline: pass_config={:?}",
            pass_config
        );

        let shader_module = self.get_or_create_environment_shader_module(device);
        let entry = create_environment_pipeline(device, surface_format, shader_module, pass_config);
        self.pipelines.insert(key, entry);
        &self.pipelines[&key]
    }

    /// Get a pipeline by key (works for both Regular and Quad)
    pub fn get_by_key(&self, key: &PipelineKey) -> Option<&PipelineEntry> {
        self.pipelines.get(key)
    }
}

impl Default for PipelineCache {
    fn default() -> Self {
        Self::new()
    }
}

fn parse_env_bool(value: &str) -> Option<bool> {
    match value.trim().to_ascii_lowercase().as_str() {
        "1" | "true" | "yes" | "on" => Some(true),
        "0" | "false" | "no" | "off" => Some(false),
        _ => None,
    }
}
