//! Pipeline cache management
//!
//! Manages caching of compiled render pipelines and shader modules.

use hashbrown::HashMap;

use super::super::render_state::{PassConfig, RenderState};
use super::pipeline_creation::{
    create_pipeline, create_quad_pipeline, create_sky_pipeline, PipelineEntry,
};
use super::pipeline_key::PipelineKey;

/// Cache for render pipelines
///
/// Stores compiled pipelines keyed by their render state configuration.
/// Pipelines are created on-demand and reused across frames.
/// Shader modules are precompiled at startup for all 40 permutations.
pub struct PipelineCache {
    pipelines: HashMap<PipelineKey, PipelineEntry>,
    /// Precompiled shader modules for all 40 mode/format combinations
    /// Index = mode * 16 + format for mode 0, or calculated index for modes 1-3
    shader_modules: Option<Vec<wgpu::ShaderModule>>,
}

impl PipelineCache {
    /// Create an empty pipeline cache
    pub fn new() -> Self {
        Self {
            pipelines: HashMap::new(),
            shader_modules: None,
        }
    }

    /// Precompile all 40 shader modules at startup
    ///
    /// This should be called during graphics initialization to ensure all shaders
    /// compile successfully. Panics on any shader compilation failure, indicating
    /// a bug in shader generation.
    pub fn precompile_all_shaders(&mut self, device: &wgpu::Device) {
        use crate::graphics::FORMAT_NORMAL;
        use crate::shader_gen::generate_shader;

        tracing::info!("Precompiling all 40 shader modules...");

        let mut modules = Vec::with_capacity(40);

        // Mode 0: 16 shaders (all formats)
        for format in 0u8..16 {
            let source = generate_shader(0, format)
                .expect("Mode 0 shader generation should succeed for all formats");
            let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                label: Some(&format!("Mode0_Format{}", format)),
                source: wgpu::ShaderSource::Wgsl(source.into()),
            });
            modules.push(module);
        }

        // Modes 1-3: 8 shaders each (only formats with NORMAL)
        for mode in 1u8..=3 {
            for format in (0u8..16).filter(|f| f & FORMAT_NORMAL != 0) {
                let source = generate_shader(mode, format).unwrap_or_else(|e| {
                    panic!(
                        "Mode {} format {} shader generation failed: {}",
                        mode, format, e
                    )
                });
                let module = device.create_shader_module(wgpu::ShaderModuleDescriptor {
                    label: Some(&format!("Mode{}_Format{}", mode, format)),
                    source: wgpu::ShaderSource::Wgsl(source.into()),
                });
                modules.push(module);
            }
        }

        assert_eq!(
            modules.len(),
            40,
            "Expected 40 shader modules, got {}",
            modules.len()
        );
        tracing::info!("Successfully precompiled all 40 shader modules");

        self.shader_modules = Some(modules);
    }

    /// Check if shaders have been precompiled
    #[allow(dead_code)] // Useful for testing/debugging
    pub fn shaders_precompiled(&self) -> bool {
        self.shader_modules.is_some()
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

        let entry = create_pipeline(
            device,
            surface_format,
            render_mode,
            format,
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
        state: &RenderState,
        pass_config: &PassConfig,
        is_screen_space: bool,
    ) -> &PipelineEntry {
        let key = PipelineKey::quad(state, pass_config, is_screen_space);

        // Return existing pipeline if cached
        if self.pipelines.contains_key(&key) {
            return &self.pipelines[&key];
        }

        // Otherwise, create a new quad pipeline
        tracing::debug!(
            "Creating quad pipeline: depth={}, is_screen_space={}, pass_config={:?}",
            state.depth_test,
            is_screen_space,
            pass_config
        );

        let entry = create_quad_pipeline(device, surface_format, pass_config, is_screen_space);
        self.pipelines.insert(key, entry);
        &self.pipelines[&key]
    }

    /// Get or create a sky pipeline
    ///
    /// Returns a reference to the cached sky pipeline, creating it if necessary.
    pub fn get_or_create_sky(
        &mut self,
        device: &wgpu::Device,
        surface_format: wgpu::TextureFormat,
        pass_config: &PassConfig,
    ) -> &PipelineEntry {
        let key = PipelineKey::sky(pass_config);

        // Return existing pipeline if cached
        if self.pipelines.contains_key(&key) {
            return &self.pipelines[&key];
        }

        // Otherwise, create a new sky pipeline
        tracing::debug!("Creating sky pipeline: pass_config={:?}", pass_config);

        let entry = create_sky_pipeline(device, surface_format, pass_config);
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
