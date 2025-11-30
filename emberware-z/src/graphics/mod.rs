//! Emberware Z graphics backend (wgpu)
//!
//! Implements the `Graphics` trait from emberware-core with a wgpu-based
//! renderer featuring PS1/N64 aesthetic (vertex jitter, affine textures).
//!
//! # Architecture
//!
//! **ZFFIState** (staging) → **ZGraphics** (GPU execution)
//!
//! - FFI functions write draw commands, transforms, and render state to ZFFIState
//! - App.rs passes ZFFIState to ZGraphics each frame
//! - ZGraphics consumes commands and executes them on the GPU
//! - ZGraphics owns all actual GPU resources (textures, meshes, buffers, pipelines)
//!
//! Note: Many public APIs here are designed for game rendering and are not yet
//! fully wired up. Dead code warnings are suppressed at module level.
//!
//! # Vertex Buffer Architecture
//!
//! Each vertex format gets its own buffer to avoid padding waste:
//! - FORMAT_UV (1): Has UV coordinates
//! - FORMAT_COLOR (2): Has per-vertex color (RGB, 3 floats)
//! - FORMAT_NORMAL (4): Has normals
//! - FORMAT_SKINNED (8): Has bone indices/weights
//!
//! All 16 combinations are supported (8 base + 8 skinned variants).
//!
//! # Command Buffer Pattern
//!
//! Immediate-mode draws are buffered on the CPU side and flushed once per frame
//! to minimize draw calls. Retained meshes are stored separately.
//!
//! # Resource Cleanup Strategy
//!
//! Graphics resources are automatically cleaned up when the owning structures are dropped:
//!
//! - **Textures** (`TextureEntry`): Stored in `ZGraphics::textures` HashMap. When the
//!   HashMap entry is removed or ZGraphics is dropped, wgpu::Texture/TextureView implement
//!   Drop and release GPU resources automatically.
//!
//! - **Retained Meshes** (`RetainedMesh`): Stored in `ZGraphics::retained_meshes` HashMap.
//!   Mesh data lives in shared vertex/index buffers per format, so individual mesh removal
//!   doesn't free GPU memory (buffer compaction not implemented). Full cleanup occurs when
//!   ZGraphics is dropped.
//!
//! - **Vertex/Index Buffers** (`GrowableBuffer`): One per vertex format (16 total).
//!   Automatically dropped when ZGraphics is dropped; wgpu::Buffer implements Drop.
//!
//! - **Pipelines**: Cached in `ZGraphics::pipelines` HashMap. Dropped when ZGraphics drops.
//!
//! - **Per-Frame Resources**: Command buffer resets each frame via `reset_command_buffer()`.
//!   No GPU allocations for immediate-mode draws between frames.
//!
//! **Game Lifecycle**: When a game exits (mode changes from Playing to Library), the game's
//! `GameInstance` is dropped, which clears pending textures/meshes in `GameState`. However,
//! ZGraphics remains alive across game switches to avoid expensive GPU reinitialization.
//! This means textures/meshes loaded by a previous game persist in GPU memory until
//! explicitly removed or the application exits. For the intended use case (single game per
//! session), this is acceptable. Future versions may add explicit resource invalidation on
//! game switch if memory pressure becomes a concern.

// Many public APIs are designed for game rendering but not yet fully wired up
#![allow(dead_code)]

mod buffer;
mod command_buffer;
mod pipeline;
mod render_state;
mod vertex;

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{Context, Result};
use glam::{Mat4, Vec3};
use wgpu::util::DeviceExt;
use winit::window::Window;

use emberware_core::console::Graphics;

use crate::console::VRAM_LIMIT;

// Re-export public types from submodules
pub use buffer::{GrowableBuffer, MeshHandle, RetainedMesh};
pub use command_buffer::CommandBuffer;
pub use render_state::{
    BlendMode, CameraUniforms, CullMode, LightUniform, LightsUniforms, MatcapBlendMode,
    MaterialUniforms, RenderState, SkyUniforms, TextureFilter, TextureHandle,
};
pub use vertex::{
    vertex_stride, VertexFormatInfo, FORMAT_COLOR, FORMAT_NORMAL, FORMAT_SKINNED, FORMAT_UV,
    VERTEX_FORMAT_COUNT,
};

// Re-export for crate-internal use
pub(crate) use pipeline::PipelineEntry;

use pipeline::PipelineKey;

/// Internal texture data
///
/// Fields tracked for debugging and VRAM accounting.
/// Will be read when render_frame() processes draw commands.
#[allow(dead_code)]
struct TextureEntry {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    width: u32,
    height: u32,
    /// Size in bytes (for VRAM tracking)
    size_bytes: usize,
}

/// Emberware Z graphics backend
///
/// Manages wgpu device, textures, render state, and frame presentation.
/// Implements the vertex buffer architecture with one buffer per stride
/// and command buffer pattern for draw batching.
pub struct ZGraphics {
    // Core wgpu objects
    surface: wgpu::Surface<'static>,
    device: wgpu::Device,
    queue: wgpu::Queue,
    config: wgpu::SurfaceConfiguration,

    // Depth buffer
    depth_texture: wgpu::Texture,
    depth_view: wgpu::TextureView,

    // Texture management
    textures: HashMap<u32, TextureEntry>,
    next_texture_id: u32,
    vram_used: usize,

    // Fallback textures
    fallback_checkerboard: TextureHandle,
    fallback_white: TextureHandle,

    // Built-in font texture
    font_texture: TextureHandle,

    // Samplers
    sampler_nearest: wgpu::Sampler,
    sampler_linear: wgpu::Sampler,

    // Current render state
    render_state: RenderState,

    // Sky system
    sky_uniforms: SkyUniforms,
    sky_buffer: wgpu::Buffer,

    // Camera system (view/projection + position for specular)
    camera_uniforms: CameraUniforms,
    camera_buffer: wgpu::Buffer,

    // Lighting system (4 directional lights for PBR)
    lights_uniforms: LightsUniforms,
    lights_buffer: wgpu::Buffer,

    // Material system (global metallic/roughness/emissive)
    material_uniforms: MaterialUniforms,
    material_buffer: wgpu::Buffer,

    // Bone system (GPU skinning)
    bone_buffer: wgpu::Buffer,

    // Frame state
    current_frame: Option<wgpu::SurfaceTexture>,
    current_view: Option<wgpu::TextureView>,

    // Vertex buffer architecture
    // Per-format vertex buffers (one for each of 16 vertex formats)
    vertex_buffers: [GrowableBuffer; VERTEX_FORMAT_COUNT],
    // Per-format index buffers
    index_buffers: [GrowableBuffer; VERTEX_FORMAT_COUNT],

    // Retained mesh storage
    retained_meshes: HashMap<u32, RetainedMesh>,
    next_mesh_id: u32,

    // Command buffer for immediate mode draws
    command_buffer: CommandBuffer,

    // Current transform matrix (model transform)
    current_transform: Mat4,
    // Transform stack for push/pop
    transform_stack: Vec<Mat4>,

    // Shader and pipeline cache
    pipelines: HashMap<PipelineKey, PipelineEntry>,
    current_render_mode: u8,
}

impl ZGraphics {
    /// Create a new ZGraphics instance
    ///
    /// This initializes wgpu with the given window and sets up all core resources.
    pub async fn new(window: Arc<Window>) -> Result<Self> {
        let size = window.inner_size();
        let width = size.width.max(1);
        let height = size.height.max(1);

        // Create wgpu instance
        let instance = wgpu::Instance::new(wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        // Create surface
        let surface = instance
            .create_surface(window)
            .context("Failed to create surface")?;

        // Request adapter
        let adapter = instance
            .request_adapter(&wgpu::RequestAdapterOptions {
                power_preference: wgpu::PowerPreference::HighPerformance,
                compatible_surface: Some(&surface),
                force_fallback_adapter: false,
            })
            .await
            .context("Failed to find suitable GPU adapter")?;

        tracing::info!("Using GPU adapter: {:?}", adapter.get_info().name);

        // Request device and queue
        let (device, queue) = adapter
            .request_device(
                &wgpu::DeviceDescriptor {
                    label: Some("Emberware Z Device"),
                    required_features: wgpu::Features::empty(),
                    required_limits: wgpu::Limits::default(),
                    memory_hints: wgpu::MemoryHints::Performance,
                },
                None,
            )
            .await
            .context("Failed to create GPU device")?;

        // Configure surface
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width,
            height,
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };
        surface.configure(&device, &config);

        // Create depth buffer
        let (depth_texture, depth_view) = Self::create_depth_texture(&device, width, height);

        // Create samplers
        let sampler_nearest = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Sampler Nearest"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Nearest,
            min_filter: wgpu::FilterMode::Nearest,
            mipmap_filter: wgpu::FilterMode::Nearest,
            ..Default::default()
        });

        let sampler_linear = device.create_sampler(&wgpu::SamplerDescriptor {
            label: Some("Sampler Linear"),
            address_mode_u: wgpu::AddressMode::Repeat,
            address_mode_v: wgpu::AddressMode::Repeat,
            address_mode_w: wgpu::AddressMode::Repeat,
            mag_filter: wgpu::FilterMode::Linear,
            min_filter: wgpu::FilterMode::Linear,
            mipmap_filter: wgpu::FilterMode::Linear,
            ..Default::default()
        });

        // Create per-format vertex and index buffers
        let vertex_buffers = std::array::from_fn(|i| {
            let info = VertexFormatInfo::for_format(i as u8);
            GrowableBuffer::new(
                &device,
                wgpu::BufferUsages::VERTEX,
                &format!("Vertex Buffer {}", info.name),
            )
        });

        let index_buffers = std::array::from_fn(|i| {
            let info = VertexFormatInfo::for_format(i as u8);
            GrowableBuffer::new(
                &device,
                wgpu::BufferUsages::INDEX,
                &format!("Index Buffer {}", info.name),
            )
        });

        // Create sky uniform buffer
        let sky_uniforms = SkyUniforms::default();
        let sky_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Sky Uniform Buffer"),
            contents: bytemuck::cast_slice(&[sky_uniforms]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Create camera uniform buffer (view/projection + position)
        let camera_uniforms = CameraUniforms::default();
        let camera_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Camera Uniform Buffer"),
            contents: bytemuck::cast_slice(&[camera_uniforms]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Create lights uniform buffer (4 directional lights)
        let lights_uniforms = LightsUniforms::default();
        let lights_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Lights Uniform Buffer"),
            contents: bytemuck::cast_slice(&[lights_uniforms]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Create material uniform buffer (metallic/roughness/emissive)
        let material_uniforms = MaterialUniforms::default();
        let material_buffer = device.create_buffer_init(&wgpu::util::BufferInitDescriptor {
            label: Some("Material Uniform Buffer"),
            contents: bytemuck::cast_slice(&[material_uniforms]),
            usage: wgpu::BufferUsages::UNIFORM | wgpu::BufferUsages::COPY_DST,
        });

        // Create bone storage buffer for GPU skinning (256 bones × 64 bytes = 16KB)
        let bone_buffer = device.create_buffer(&wgpu::BufferDescriptor {
            label: Some("Bone Storage Buffer"),
            size: 256 * 64, // 256 matrices × 64 bytes per matrix
            usage: wgpu::BufferUsages::STORAGE | wgpu::BufferUsages::COPY_DST,
            mapped_at_creation: false,
        });

        let mut graphics = Self {
            surface,
            device,
            queue,
            config,
            depth_texture,
            depth_view,
            textures: HashMap::new(),
            next_texture_id: 1, // 0 is reserved for INVALID
            vram_used: 0,
            fallback_checkerboard: TextureHandle::INVALID,
            fallback_white: TextureHandle::INVALID,
            font_texture: TextureHandle::INVALID,
            sampler_nearest,
            sampler_linear,
            render_state: RenderState::default(),
            sky_uniforms,
            sky_buffer,
            camera_uniforms,
            camera_buffer,
            lights_uniforms,
            lights_buffer,
            material_uniforms,
            material_buffer,
            bone_buffer,
            current_frame: None,
            current_view: None,
            vertex_buffers,
            index_buffers,
            retained_meshes: HashMap::new(),
            next_mesh_id: 1, // 0 is reserved for INVALID
            command_buffer: CommandBuffer::new(),
            current_transform: Mat4::IDENTITY,
            transform_stack: Vec::with_capacity(16),
            pipelines: HashMap::new(),
            current_render_mode: 0, // Default to Mode 0 (Unlit)
        };

        // Create fallback textures
        graphics.create_fallback_textures()?;

        Ok(graphics)
    }

    /// Create a new ZGraphics instance (blocking version for sync contexts)
    pub fn new_blocking(window: Arc<Window>) -> Result<Self> {
        pollster::block_on(Self::new(window))
    }

    /// Create depth texture and view
    fn create_depth_texture(
        device: &wgpu::Device,
        width: u32,
        height: u32,
    ) -> (wgpu::Texture, wgpu::TextureView) {
        let texture = device.create_texture(&wgpu::TextureDescriptor {
            label: Some("Depth Texture"),
            size: wgpu::Extent3d {
                width,
                height,
                depth_or_array_layers: 1,
            },
            mip_level_count: 1,
            sample_count: 1,
            dimension: wgpu::TextureDimension::D2,
            format: wgpu::TextureFormat::Depth32Float,
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT | wgpu::TextureUsages::TEXTURE_BINDING,
            view_formats: &[],
        });
        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());
        (texture, view)
    }

    /// Create fallback textures (checkerboard, white, and font)
    ///
    /// These textures are essential for rendering - errors are propagated
    /// to allow graceful initialization failure.
    fn create_fallback_textures(&mut self) -> Result<()> {
        // 8x8 magenta/black checkerboard for missing textures
        let mut checkerboard_data = vec![0u8; 8 * 8 * 4];
        for y in 0..8 {
            for x in 0..8 {
                let idx = (y * 8 + x) * 4;
                let is_magenta = (x + y) % 2 == 0;
                if is_magenta {
                    checkerboard_data[idx] = 255; // R
                    checkerboard_data[idx + 1] = 0; // G
                    checkerboard_data[idx + 2] = 255; // B
                    checkerboard_data[idx + 3] = 255; // A
                } else {
                    checkerboard_data[idx] = 0; // R
                    checkerboard_data[idx + 1] = 0; // G
                    checkerboard_data[idx + 2] = 0; // B
                    checkerboard_data[idx + 3] = 255; // A
                }
            }
        }
        self.fallback_checkerboard = self
            .load_texture_internal(8, 8, &checkerboard_data, false)
            .context("Failed to create checkerboard fallback texture")?;

        // 1x1 white texture for untextured draws
        let white_data = [255u8, 255, 255, 255];
        self.fallback_white = self
            .load_texture_internal(1, 1, &white_data, false)
            .context("Failed to create white fallback texture")?;

        // Load built-in font texture
        use crate::font;
        let font_atlas = font::generate_font_atlas();
        self.font_texture = self
            .load_texture_internal(font::ATLAS_WIDTH, font::ATLAS_HEIGHT, &font_atlas, false)
            .context("Failed to create font texture")?;

        tracing::debug!(
            "Created font texture: {}x{}",
            font::ATLAS_WIDTH,
            font::ATLAS_HEIGHT
        );
        Ok(())
    }

    // ========================================================================
    // Texture Management
    // ========================================================================

    /// Load a texture from RGBA8 pixel data
    ///
    /// Returns a TextureHandle or an error if VRAM budget is exceeded.
    pub fn load_texture(
        &mut self,
        width: u32,
        height: u32,
        pixels: &[u8],
    ) -> Result<TextureHandle> {
        self.load_texture_internal(width, height, pixels, true)
    }

    /// Internal texture loading (optionally tracks VRAM)
    fn load_texture_internal(
        &mut self,
        width: u32,
        height: u32,
        pixels: &[u8],
        track_vram: bool,
    ) -> Result<TextureHandle> {
        let expected_size = (width * height * 4) as usize;
        if pixels.len() != expected_size {
            anyhow::bail!(
                "Pixel data size mismatch: expected {} bytes, got {}",
                expected_size,
                pixels.len()
            );
        }

        let size_bytes = expected_size;

        // Check VRAM budget
        if track_vram && self.vram_used + size_bytes > VRAM_LIMIT {
            anyhow::bail!(
                "VRAM budget exceeded: {} + {} > {} bytes",
                self.vram_used,
                size_bytes,
                VRAM_LIMIT
            );
        }

        // Create texture
        let texture = self.device.create_texture_with_data(
            &self.queue,
            &wgpu::TextureDescriptor {
                label: Some("Game Texture"),
                size: wgpu::Extent3d {
                    width,
                    height,
                    depth_or_array_layers: 1,
                },
                mip_level_count: 1,
                sample_count: 1,
                dimension: wgpu::TextureDimension::D2,
                format: wgpu::TextureFormat::Rgba8UnormSrgb,
                usage: wgpu::TextureUsages::TEXTURE_BINDING | wgpu::TextureUsages::COPY_DST,
                view_formats: &[],
            },
            wgpu::util::TextureDataOrder::LayerMajor,
            pixels,
        );

        let view = texture.create_view(&wgpu::TextureViewDescriptor::default());

        let handle = TextureHandle(self.next_texture_id);
        self.next_texture_id += 1;

        self.textures.insert(
            handle.0,
            TextureEntry {
                texture,
                view,
                width,
                height,
                size_bytes,
            },
        );

        if track_vram {
            self.vram_used += size_bytes;
        }

        tracing::debug!(
            "Loaded texture {}: {}x{}, {} bytes (VRAM: {}/{})",
            handle.0,
            width,
            height,
            size_bytes,
            self.vram_used,
            VRAM_LIMIT
        );

        Ok(handle)
    }

    /// Get texture view by handle
    pub fn get_texture_view(&self, handle: TextureHandle) -> Option<&wgpu::TextureView> {
        self.textures.get(&handle.0).map(|t| &t.view)
    }

    /// Get fallback checkerboard texture view
    pub fn get_fallback_checkerboard_view(&self) -> &wgpu::TextureView {
        &self.textures[&self.fallback_checkerboard.0].view
    }

    /// Get fallback white texture view
    pub fn get_fallback_white_view(&self) -> &wgpu::TextureView {
        &self.textures[&self.fallback_white.0].view
    }

    /// Get font texture handle
    pub fn font_texture(&self) -> TextureHandle {
        self.font_texture
    }

    /// Get font texture view
    pub fn get_font_texture_view(&self) -> &wgpu::TextureView {
        &self.textures[&self.font_texture.0].view
    }

    /// Get texture view for a slot, returning fallback if unbound
    pub fn get_slot_texture_view(&self, slot: usize) -> &wgpu::TextureView {
        let handle = self
            .render_state
            .texture_slots
            .get(slot)
            .copied()
            .unwrap_or(TextureHandle::INVALID);
        if handle == TextureHandle::INVALID {
            self.get_fallback_white_view()
        } else {
            self.get_texture_view(handle)
                .unwrap_or_else(|| self.get_fallback_checkerboard_view())
        }
    }

    /// Get VRAM usage in bytes
    pub fn vram_used(&self) -> usize {
        self.vram_used
    }

    /// Get VRAM limit in bytes
    pub fn vram_limit(&self) -> usize {
        VRAM_LIMIT
    }

    // ========================================================================
    // Render State
    // ========================================================================

    /// Set uniform tint color (0xRRGGBBAA)
    pub fn set_color(&mut self, color: u32) {
        self.render_state.color = color;
    }

    /// Enable or disable depth testing
    pub fn set_depth_test(&mut self, enabled: bool) {
        self.render_state.depth_test = enabled;
    }

    /// Set face culling mode
    pub fn set_cull_mode(&mut self, mode: CullMode) {
        self.render_state.cull_mode = mode;
    }

    /// Set blend mode
    pub fn set_blend_mode(&mut self, mode: BlendMode) {
        self.render_state.blend_mode = mode;
    }

    /// Set texture filter mode
    pub fn set_texture_filter(&mut self, filter: TextureFilter) {
        self.render_state.texture_filter = filter;
    }

    /// Bind texture to slot 0 (albedo)
    pub fn bind_texture(&mut self, handle: TextureHandle) {
        self.bind_texture_slot(handle, 0);
    }

    /// Bind texture to a specific slot (0-3)
    pub fn bind_texture_slot(&mut self, handle: TextureHandle, slot: usize) {
        if slot < 4 {
            self.render_state.texture_slots[slot] = handle;
        }
    }

    /// Get current render state
    pub fn render_state(&self) -> &RenderState {
        &self.render_state
    }

    /// Get current sampler based on texture filter setting
    pub fn current_sampler(&self) -> &wgpu::Sampler {
        match self.render_state.texture_filter {
            TextureFilter::Nearest => &self.sampler_nearest,
            TextureFilter::Linear => &self.sampler_linear,
        }
    }

    // ========================================================================
    // Sky System
    // ========================================================================

    /// Set sky parameters for procedural sky rendering
    ///
    /// Parameters:
    /// - horizon_rgb: Horizon color (RGB, linear, 3 floats)
    /// - zenith_rgb: Zenith (top) color (RGB, linear, 3 floats)
    /// - sun_dir: Sun direction (normalized, XYZ, 3 floats)
    /// - sun_rgb: Sun color (RGB, linear, 3 floats)
    /// - sun_sharpness: Sun sharpness (higher = sharper sun, typically 32-256)
    pub fn set_sky(
        &mut self,
        horizon_rgb: [f32; 3],
        zenith_rgb: [f32; 3],
        sun_dir: [f32; 3],
        sun_rgb: [f32; 3],
        sun_sharpness: f32,
    ) {
        // Normalize sun direction
        let sun_vec = Vec3::from_array(sun_dir);
        let sun_normalized = if sun_vec.length() > 0.0001 {
            sun_vec.normalize()
        } else {
            Vec3::Y // Default to up if zero vector
        };

        self.sky_uniforms = SkyUniforms {
            horizon_color: [horizon_rgb[0], horizon_rgb[1], horizon_rgb[2], 0.0],
            zenith_color: [zenith_rgb[0], zenith_rgb[1], zenith_rgb[2], 0.0],
            sun_direction: [sun_normalized.x, sun_normalized.y, sun_normalized.z, 0.0],
            sun_color_and_sharpness: [sun_rgb[0], sun_rgb[1], sun_rgb[2], sun_sharpness],
        };

        // Upload to GPU
        self.queue.write_buffer(
            &self.sky_buffer,
            0,
            bytemuck::cast_slice(&[self.sky_uniforms]),
        );

        tracing::debug!(
            "Set sky: horizon={:?}, zenith={:?}, sun_dir={:?}, sun_color={:?}, sharpness={}",
            horizon_rgb,
            zenith_rgb,
            sun_normalized.to_array(),
            sun_rgb,
            sun_sharpness
        );
    }

    /// Get current sky uniforms
    pub fn sky_uniforms(&self) -> &SkyUniforms {
        &self.sky_uniforms
    }

    /// Get sky uniform buffer for binding
    pub fn sky_buffer(&self) -> &wgpu::Buffer {
        &self.sky_buffer
    }

    // ========================================================================
    // Scene Uniforms (Camera, Lights, Materials)
    // ========================================================================

    /// Update scene uniforms (camera, lights, materials) and upload to GPU
    ///
    /// This should be called once per frame before rendering to ensure PBR
    /// shaders have up-to-date camera position (for specular), lights, and
    /// material properties.
    ///
    /// # Arguments
    /// * `camera` - Camera state from ZFFIState
    /// * `lights` - Array of 4 light states from ZFFIState
    /// * `aspect_ratio` - Screen aspect ratio (width/height)
    /// * `metallic` - Global metallic value (0.0 = non-metallic, 1.0 = fully metallic)
    /// * `roughness` - Global roughness value (0.0 = smooth, 1.0 = rough)
    /// * `emissive` - Global emissive intensity (0.0 = no emission, 1.0+ = glowing)
    pub fn update_scene_uniforms(
        &mut self,
        camera: &crate::state::CameraState,
        lights: &[crate::state::LightState; 4],
        aspect_ratio: f32,
        metallic: f32,
        roughness: f32,
        emissive: f32,
    ) {
        // Update camera uniforms
        let view = camera.view_matrix();
        let proj = camera.projection_matrix(aspect_ratio);

        self.camera_uniforms = CameraUniforms {
            view: view.to_cols_array_2d(),
            projection: proj.to_cols_array_2d(),
            position: [camera.position.x, camera.position.y, camera.position.z, 0.0],
        };

        self.queue.write_buffer(
            &self.camera_buffer,
            0,
            bytemuck::cast_slice(&[self.camera_uniforms]),
        );

        // Update lights uniforms
        for (i, light) in lights.iter().enumerate() {
            // Normalize light direction
            let dir = Vec3::from_array(light.direction);
            let dir_normalized = if dir.length() > 0.0001 {
                dir.normalize()
            } else {
                Vec3::new(0.0, -1.0, 0.0) // Default: downward
            };

            self.lights_uniforms.lights[i] = LightUniform {
                direction_and_enabled: [
                    dir_normalized.x,
                    dir_normalized.y,
                    dir_normalized.z,
                    if light.enabled { 1.0 } else { 0.0 },
                ],
                color_and_intensity: [
                    light.color[0],
                    light.color[1],
                    light.color[2],
                    light.intensity,
                ],
            };
        }

        self.queue.write_buffer(
            &self.lights_buffer,
            0,
            bytemuck::cast_slice(&[self.lights_uniforms]),
        );

        // Update material uniforms
        self.material_uniforms = MaterialUniforms {
            properties: [
                metallic.clamp(0.0, 1.0),
                roughness.clamp(0.0, 1.0),
                emissive.max(0.0),
                0.0, // .w unused
            ],
        };

        self.queue.write_buffer(
            &self.material_buffer,
            0,
            bytemuck::cast_slice(&[self.material_uniforms]),
        );

        tracing::trace!(
            "Updated scene uniforms: camera_pos={:?}, lights_enabled={}/{}/{}/{}, material=M{:.2}/R{:.2}/E{:.2}",
            camera.position,
            lights[0].enabled,
            lights[1].enabled,
            lights[2].enabled,
            lights[3].enabled,
            metallic,
            roughness,
            emissive
        );
    }

    /// Get camera uniform buffer for binding
    pub fn camera_buffer(&self) -> &wgpu::Buffer {
        &self.camera_buffer
    }

    /// Get lights uniform buffer for binding
    pub fn lights_buffer(&self) -> &wgpu::Buffer {
        &self.lights_buffer
    }

    /// Get material uniform buffer for binding
    pub fn material_buffer(&self) -> &wgpu::Buffer {
        &self.material_buffer
    }

    // ========================================================================
    // Transform Stack
    // ========================================================================

    /// Reset transform to identity matrix
    pub fn transform_identity(&mut self) {
        self.current_transform = Mat4::IDENTITY;
    }

    /// Translate the current transform
    pub fn transform_translate(&mut self, x: f32, y: f32, z: f32) {
        self.current_transform *= Mat4::from_translation(glam::vec3(x, y, z));
    }

    /// Rotate the current transform around an axis (angle in degrees)
    pub fn transform_rotate(&mut self, angle_deg: f32, x: f32, y: f32, z: f32) {
        let axis = glam::vec3(x, y, z).normalize();
        let angle_rad = angle_deg.to_radians();
        self.current_transform *= Mat4::from_axis_angle(axis, angle_rad);
    }

    /// Scale the current transform
    pub fn transform_scale(&mut self, x: f32, y: f32, z: f32) {
        self.current_transform *= Mat4::from_scale(glam::vec3(x, y, z));
    }

    /// Push the current transform onto the stack
    ///
    /// Returns false if the stack is full (max 16 entries)
    pub fn transform_push(&mut self) -> bool {
        if self.transform_stack.len() >= 16 {
            tracing::warn!("Transform stack overflow (max 16)");
            return false;
        }
        self.transform_stack.push(self.current_transform);
        true
    }

    /// Pop the transform from the stack
    ///
    /// Returns false if the stack is empty
    pub fn transform_pop(&mut self) -> bool {
        if let Some(transform) = self.transform_stack.pop() {
            self.current_transform = transform;
            true
        } else {
            tracing::warn!("Transform stack underflow");
            false
        }
    }

    /// Set the current transform from a 4x4 matrix (16 floats, column-major)
    pub fn transform_set(&mut self, matrix: &[f32; 16]) {
        self.current_transform = Mat4::from_cols_array(matrix);
    }

    /// Get the current transform matrix
    pub fn current_transform(&self) -> Mat4 {
        self.current_transform
    }

    // ========================================================================
    // Retained Mesh Loading
    // ========================================================================

    /// Load a non-indexed mesh (retained mode)
    ///
    /// The mesh is stored in the appropriate vertex buffer based on format.
    /// Returns a MeshHandle for use with draw_mesh().
    pub fn load_mesh(&mut self, data: &[f32], format: u8) -> Result<MeshHandle> {
        let format_idx = format as usize;
        if format_idx >= VERTEX_FORMAT_COUNT {
            anyhow::bail!("Invalid vertex format: {}", format);
        }

        let stride = vertex_stride(format) as usize;
        let byte_data = bytemuck::cast_slice(data);
        let vertex_count = byte_data.len() / stride;

        if byte_data.len() % stride != 0 {
            anyhow::bail!(
                "Vertex data size {} is not a multiple of stride {}",
                byte_data.len(),
                stride
            );
        }

        // Ensure buffer has capacity
        self.vertex_buffers[format_idx].ensure_capacity(&self.device, byte_data.len() as u64);

        // Write to buffer
        let vertex_offset = self.vertex_buffers[format_idx].write(&self.queue, byte_data);

        // Create mesh handle
        let handle = MeshHandle(self.next_mesh_id);
        self.next_mesh_id += 1;

        self.retained_meshes.insert(
            handle.0,
            RetainedMesh {
                format,
                vertex_count: vertex_count as u32,
                index_count: 0,
                vertex_offset,
                index_offset: 0,
            },
        );

        tracing::debug!(
            "Loaded mesh {}: {} vertices, format {}",
            handle.0,
            vertex_count,
            VertexFormatInfo::for_format(format).name
        );

        Ok(handle)
    }

    /// Load an indexed mesh (retained mode)
    ///
    /// The mesh is stored in the appropriate vertex and index buffers based on format.
    /// Returns a MeshHandle for use with draw_mesh().
    pub fn load_mesh_indexed(
        &mut self,
        data: &[f32],
        indices: &[u32],
        format: u8,
    ) -> Result<MeshHandle> {
        let format_idx = format as usize;
        if format_idx >= VERTEX_FORMAT_COUNT {
            anyhow::bail!("Invalid vertex format: {}", format);
        }

        let stride = vertex_stride(format) as usize;
        let byte_data = bytemuck::cast_slice(data);
        let vertex_count = byte_data.len() / stride;

        if byte_data.len() % stride != 0 {
            anyhow::bail!(
                "Vertex data size {} is not a multiple of stride {}",
                byte_data.len(),
                stride
            );
        }

        // Ensure vertex buffer has capacity
        self.vertex_buffers[format_idx].ensure_capacity(&self.device, byte_data.len() as u64);

        // Ensure index buffer has capacity
        let index_byte_data: &[u8] = bytemuck::cast_slice(indices);
        self.index_buffers[format_idx].ensure_capacity(&self.device, index_byte_data.len() as u64);

        // Write to buffers
        let vertex_offset = self.vertex_buffers[format_idx].write(&self.queue, byte_data);
        let index_offset = self.index_buffers[format_idx].write(&self.queue, index_byte_data);

        // Create mesh handle
        let handle = MeshHandle(self.next_mesh_id);
        self.next_mesh_id += 1;

        self.retained_meshes.insert(
            handle.0,
            RetainedMesh {
                format,
                vertex_count: vertex_count as u32,
                index_count: indices.len() as u32,
                vertex_offset,
                index_offset,
            },
        );

        tracing::debug!(
            "Loaded indexed mesh {}: {} vertices, {} indices, format {}",
            handle.0,
            vertex_count,
            indices.len(),
            VertexFormatInfo::for_format(format).name
        );

        Ok(handle)
    }

    /// Get mesh info by handle
    pub fn get_mesh(&self, handle: MeshHandle) -> Option<&RetainedMesh> {
        self.retained_meshes.get(&handle.0)
    }

    // ========================================================================
    // Command Buffer Access
    // ========================================================================

    /// Get the command buffer (for flush/rendering)
    pub fn command_buffer(&self) -> &CommandBuffer {
        &self.command_buffer
    }

    /// Get mutable command buffer
    pub fn command_buffer_mut(&mut self) -> &mut CommandBuffer {
        &mut self.command_buffer
    }

    /// Reset the command buffer for the next frame
    ///
    /// Called automatically at the start of begin_frame, but can be called
    /// manually if needed.
    pub fn reset_command_buffer(&mut self) {
        self.command_buffer.reset();
    }

    // ========================================================================
    // Draw Command Processing
    // ========================================================================

    /// Process all draw commands from ZFFIState and execute them
    ///
    /// This method consumes draw commands from the ZFFIState and executes them
    /// on the GPU, directly translating FFI state into graphics calls without
    /// an intermediate unpacking/repacking step.
    ///
    /// This replaces the previous execute_draw_commands() function in app.rs,
    /// eliminating redundant data translation and simplifying the architecture.
    pub fn process_draw_commands(
        &mut self,
        z_state: &mut crate::state::ZFFIState,
        texture_map: &std::collections::HashMap<u32, TextureHandle>,
        mesh_map: &std::collections::HashMap<u32, MeshHandle>,
    ) {
        use crate::state::ZDrawCommand;

        // Apply init config to graphics (render mode, etc.)
        self.set_render_mode(z_state.init_config.render_mode);

        // Update scene uniforms (camera, lights, materials) for PBR rendering
        // Note: aspect ratio is computed in app.rs and passed via update_scene_uniforms
        // This is kept separate as it needs window size which ZGraphics doesn't have

        // Convert matcap blend modes from u8 to MatcapBlendMode
        let matcap_blend_modes = [
            Self::convert_matcap_blend_mode(z_state.matcap_blend_modes[0]),
            Self::convert_matcap_blend_mode(z_state.matcap_blend_modes[1]),
            Self::convert_matcap_blend_mode(z_state.matcap_blend_modes[2]),
            Self::convert_matcap_blend_mode(z_state.matcap_blend_modes[3]),
        ];

        // Process all draw commands by directly converting to internal DrawCommand
        for cmd in z_state.draw_commands.drain(..) {
            match cmd {
                ZDrawCommand::DrawTriangles {
                    format,
                    vertex_data,
                    transform,
                    color,
                    depth_test,
                    cull_mode,
                    blend_mode,
                    bound_textures,
                } => {
                    // Validate format
                    if format as usize >= VERTEX_FORMAT_COUNT {
                        tracing::warn!("Invalid vertex format for draw_triangles: {}", format);
                        continue;
                    }

                    // Map game texture handles to graphics texture handles
                    let texture_slots = Self::map_texture_handles(texture_map, &bound_textures);

                    // Calculate vertex count
                    let stride = vertex_stride(format) as usize;
                    let vertex_count = (vertex_data.len() * 4) / stride;

                    // Append vertex data and get base_vertex
                    let base_vertex = self.command_buffer.append_vertex_data(format, &vertex_data);

                    // Add draw command directly
                    self.command_buffer.add_command(command_buffer::DrawCommand {
                        format,
                        transform,
                        vertex_count: vertex_count as u32,
                        index_count: 0,
                        base_vertex,
                        first_index: 0,
                        texture_slots,
                        color,
                        depth_test,
                        cull_mode: Self::convert_cull_mode(cull_mode),
                        blend_mode: Self::convert_blend_mode(blend_mode),
                        matcap_blend_modes,
                    });
                }
                ZDrawCommand::DrawTrianglesIndexed {
                    format,
                    vertex_data,
                    index_data,
                    transform,
                    color,
                    depth_test,
                    cull_mode,
                    blend_mode,
                    bound_textures,
                } => {
                    // Validate format
                    if format as usize >= VERTEX_FORMAT_COUNT {
                        tracing::warn!(
                            "Invalid vertex format for draw_triangles_indexed: {}",
                            format
                        );
                        continue;
                    }

                    // Map game texture handles to graphics texture handles
                    let texture_slots = Self::map_texture_handles(texture_map, &bound_textures);

                    // Calculate vertex count
                    let stride = vertex_stride(format) as usize;
                    let vertex_count = (vertex_data.len() * 4) / stride;

                    // Append vertex and index data
                    let base_vertex = self.command_buffer.append_vertex_data(format, &vertex_data);
                    let first_index = self.command_buffer.append_index_data(format, &index_data);

                    // Add draw command directly
                    self.command_buffer.add_command(command_buffer::DrawCommand {
                        format,
                        transform,
                        vertex_count: vertex_count as u32,
                        index_count: index_data.len() as u32,
                        base_vertex,
                        first_index,
                        texture_slots,
                        color,
                        depth_test,
                        cull_mode: Self::convert_cull_mode(cull_mode),
                        blend_mode: Self::convert_blend_mode(blend_mode),
                        matcap_blend_modes,
                    });
                }
                ZDrawCommand::DrawMesh {
                    handle,
                    transform,
                    color,
                    depth_test,
                    cull_mode,
                    blend_mode,
                    bound_textures,
                } => {
                    // Look up mesh handle
                    if let Some(&mesh_handle) = mesh_map.get(&handle) {
                        if let Some(mesh) = self.get_mesh(mesh_handle) {
                            // Draw the retained mesh
                            tracing::trace!(
                                "Drawing mesh handle {} ({} vertices)",
                                handle,
                                mesh.vertex_count
                            );

                            // Validate format
                            if mesh.format as usize >= VERTEX_FORMAT_COUNT {
                                tracing::warn!("Invalid vertex format for mesh: {}", mesh.format);
                                continue;
                            }

                            // Map game texture handles to graphics texture handles
                            let texture_slots =
                                Self::map_texture_handles(texture_map, &bound_textures);

                            // Convert byte offsets to vertex/index counts
                            let stride = vertex_stride(mesh.format) as u64;
                            let base_vertex = (mesh.vertex_offset / stride) as u32;
                            let first_index = if mesh.index_count > 0 {
                                (mesh.index_offset / 4) as u32 // u32 indices are 4 bytes each
                            } else {
                                0
                            };

                            // Add draw command for the retained mesh
                            self.command_buffer.add_command(command_buffer::DrawCommand {
                                format: mesh.format,
                                transform,
                                vertex_count: mesh.vertex_count,
                                index_count: mesh.index_count,
                                base_vertex,
                                first_index,
                                texture_slots,
                                color,
                                depth_test,
                                cull_mode: Self::convert_cull_mode(cull_mode),
                                blend_mode: Self::convert_blend_mode(blend_mode),
                                matcap_blend_modes,
                            });
                        }
                    } else {
                        tracing::warn!("Mesh handle {} not found", handle);
                    }
                }
                ZDrawCommand::DrawBillboard {
                    width,
                    height,
                    mode,
                    uv_rect,
                    transform,
                    color,
                    depth_test,
                    cull_mode,
                    blend_mode,
                    bound_textures,
                } => {
                    // Generate billboard quad geometry
                    tracing::trace!(
                        "Billboard: {}x{} mode={} at {:?} color={:08x}",
                        width,
                        height,
                        mode,
                        transform,
                        color
                    );

                    // Validate mode
                    if mode < 1 || mode > 4 {
                        tracing::warn!("Invalid billboard mode: {} (must be 1-4)", mode);
                        continue;
                    }

                    // Map game texture handles to graphics texture handles
                    let texture_slots = Self::map_texture_handles(texture_map, &bound_textures);

                    // Extract position from transform (last column)
                    let position = transform.w_axis.truncate();

                    // Calculate camera direction
                    let camera_pos = z_state.camera.position;
                    let to_camera = (camera_pos - position).normalize();

                    // Generate billboard orientation based on mode
                    let (right, up) = match mode {
                        1 => {
                            // Spherical: fully face camera
                            let view_matrix = z_state.camera.view_matrix();
                            let right = view_matrix.x_axis.truncate();
                            let up = view_matrix.y_axis.truncate();
                            (right, up)
                        }
                        2 => {
                            // Cylindrical Y-axis: rotate around Y to face camera
                            let right = glam::Vec3::Y.cross(to_camera).normalize();
                            let up = glam::Vec3::Y;
                            (right, up)
                        }
                        3 => {
                            // Cylindrical X-axis: rotate around X to face camera
                            let up = to_camera.cross(glam::Vec3::X).normalize();
                            let right = glam::Vec3::X;
                            (right, up)
                        }
                        4 => {
                            // Cylindrical Z-axis: rotate around Z to face camera
                            let right = glam::Vec3::Z.cross(to_camera).normalize();
                            let up = to_camera.cross(right).normalize();
                            (right, up)
                        }
                        _ => unreachable!(),
                    };

                    // Calculate UV coordinates
                    let (u0, v0, u1, v1) = uv_rect.unwrap_or((0.0, 0.0, 1.0, 1.0));

                    // Generate quad vertices (POS_UV_COLOR format = 3)
                    // Position (3) + UV (2) + Color (3) = 8 floats per vertex
                    let half_w = width * 0.5;
                    let half_h = height * 0.5;

                    // Extract color components (RGBA)
                    let r = ((color >> 24) & 0xFF) as f32 / 255.0;
                    let g = ((color >> 16) & 0xFF) as f32 / 255.0;
                    let b = ((color >> 8) & 0xFF) as f32 / 255.0;

                    // Four corners of the billboard
                    let v0_pos = position - right * half_w - up * half_h;
                    let v1_pos = position + right * half_w - up * half_h;
                    let v2_pos = position + right * half_w + up * half_h;
                    let v3_pos = position - right * half_w + up * half_h;

                    #[rustfmt::skip]
                    let vertices = vec![
                        // Vertex 0 (bottom-left)
                        v0_pos.x, v0_pos.y, v0_pos.z, u0, v1, r, g, b,
                        // Vertex 1 (bottom-right)
                        v1_pos.x, v1_pos.y, v1_pos.z, u1, v1, r, g, b,
                        // Vertex 2 (top-right)
                        v2_pos.x, v2_pos.y, v2_pos.z, u1, v0, r, g, b,
                        // Vertex 3 (top-left)
                        v3_pos.x, v3_pos.y, v3_pos.z, u0, v0, r, g, b,
                    ];

                    // Two triangles (CCW winding)
                    let indices = vec![0, 1, 2, 0, 2, 3];

                    // POS_UV_COLOR format
                    let format = 3;

                    // Append vertex and index data
                    let base_vertex = self.command_buffer.append_vertex_data(format, &vertices);
                    let first_index = self.command_buffer.append_index_data(format, &indices);

                    // Add draw command with identity transform (positions are in world space)
                    self.command_buffer.add_command(command_buffer::DrawCommand {
                        format,
                        transform: glam::Mat4::IDENTITY,
                        vertex_count: 4,
                        index_count: 6,
                        base_vertex,
                        first_index,
                        texture_slots,
                        color: 0xFFFFFFFF, // White (color already in vertices)
                        depth_test,
                        cull_mode: Self::convert_cull_mode(cull_mode),
                        blend_mode: Self::convert_blend_mode(blend_mode),
                        matcap_blend_modes,
                    });
                }
                ZDrawCommand::DrawSprite {
                    x,
                    y,
                    width,
                    height,
                    uv_rect: _,
                    origin: _,
                    rotation: _,
                    color,
                    blend_mode: _,
                    bound_textures: _,
                } => {
                    // TODO: Implement 2D sprite rendering
                    tracing::trace!(
                        "Sprite: {}x{} at ({},{}) color={:08x}",
                        width,
                        height,
                        x,
                        y,
                        color
                    );
                }
                ZDrawCommand::DrawRect {
                    x,
                    y,
                    width,
                    height,
                    color,
                    blend_mode: _,
                } => {
                    // TODO: Implement 2D rectangle rendering
                    tracing::trace!(
                        "Rect: {}x{} at ({},{}) color={:08x}",
                        width,
                        height,
                        x,
                        y,
                        color
                    );
                }
                ZDrawCommand::DrawText {
                    text,
                    x,
                    y,
                    size,
                    color,
                    blend_mode,
                    font,
                } => {
                    // Render text using built-in or custom font
                    let text_str = std::str::from_utf8(&text).unwrap_or("");

                    // Skip empty text
                    if text_str.is_empty() {
                        continue;
                    }

                    // Look up custom font if font handle != 0
                    let font_opt = if font == 0 {
                        None
                    } else {
                        let font_index = (font - 1) as usize;
                        z_state.fonts.get(font_index)
                    };

                    // Generate text quads (POS_UV_COLOR format = 3)
                    let (vertices, indices) =
                        Self::generate_text_quads(text_str, x, y, size, color, font_opt);

                    // Skip if no vertices generated
                    if vertices.is_empty() || indices.is_empty() {
                        continue;
                    }

                    // Determine which texture to use
                    let font_texture = if let Some(custom_font) = font_opt {
                        // Use custom font's texture
                        if let Some(&graphics_handle) = texture_map.get(&custom_font.texture) {
                            graphics_handle
                        } else {
                            tracing::warn!(
                                "Custom font texture handle {} not found, using built-in font",
                                custom_font.texture
                            );
                            self.font_texture
                        }
                    } else {
                        // Use built-in font texture
                        self.font_texture
                    };

                    // Text uses POS_UV_COLOR format (format 3)
                    const TEXT_FORMAT: u8 = 3; // FORMAT_UV | FORMAT_COLOR

                    // Append vertex and index data
                    let base_vertex = self.command_buffer.append_vertex_data(TEXT_FORMAT, &vertices);
                    let first_index = self.command_buffer.append_index_data(TEXT_FORMAT, &indices);

                    // Create texture slots with font texture in slot 0
                    let mut texture_slots = [TextureHandle::INVALID; 4];
                    texture_slots[0] = font_texture;

                    // Add draw command for text rendering
                    // Text is always rendered in 2D screen space with identity transform
                    self.command_buffer.add_command(command_buffer::DrawCommand {
                        format: TEXT_FORMAT,
                        transform: Mat4::IDENTITY,
                        vertex_count: (vertices.len() / 8) as u32, // 8 floats per vertex
                        index_count: indices.len() as u32,
                        base_vertex,
                        first_index,
                        texture_slots,
                        color: 0xFFFFFFFF, // Color already baked into vertices
                        depth_test: false, // 2D text doesn't use depth test
                        cull_mode: CullMode::None,
                        blend_mode: Self::convert_blend_mode(blend_mode),
                        matcap_blend_modes,
                    });
                }
                ZDrawCommand::SetSky {
                    horizon_color,
                    zenith_color,
                    sun_direction,
                    sun_color,
                    sun_sharpness,
                } => {
                    self.set_sky(
                        horizon_color,
                        zenith_color,
                        sun_direction,
                        sun_color,
                        sun_sharpness,
                    );
                }
            }
        }

        // Clear FFI staging state for next frame
        z_state.clear_frame();
    }

    /// Convert game matcap blend mode to graphics matcap blend mode
    fn convert_matcap_blend_mode(mode: u8) -> MatcapBlendMode {
        match mode {
            0 => MatcapBlendMode::Multiply,
            1 => MatcapBlendMode::Add,
            2 => MatcapBlendMode::HsvModulate,
            _ => MatcapBlendMode::Multiply,
        }
    }

    /// Map game texture handles to graphics texture handles
    fn map_texture_handles(
        texture_map: &std::collections::HashMap<u32, TextureHandle>,
        bound_textures: &[u32; 4],
    ) -> [TextureHandle; 4] {
        let mut texture_slots = [TextureHandle::INVALID; 4];
        for (slot, &game_handle) in bound_textures.iter().enumerate() {
            if game_handle != 0 {
                if let Some(&graphics_handle) = texture_map.get(&game_handle) {
                    texture_slots[slot] = graphics_handle;
                }
            }
        }
        texture_slots
    }

    /// Convert game cull mode to graphics cull mode
    fn convert_cull_mode(mode: u8) -> CullMode {
        match mode {
            0 => CullMode::None,
            1 => CullMode::Back,
            2 => CullMode::Front,
            _ => CullMode::None,
        }
    }

    /// Convert game blend mode to graphics blend mode
    fn convert_blend_mode(mode: u8) -> BlendMode {
        match mode {
            0 => BlendMode::None,
            1 => BlendMode::Alpha,
            2 => BlendMode::Additive,
            3 => BlendMode::Multiply,
            _ => BlendMode::None,
        }
    }

    // ========================================================================
    // Buffer Access (for rendering)
    // ========================================================================

    /// Get vertex buffer for a format
    pub fn vertex_buffer(&self, format: u8) -> &GrowableBuffer {
        &self.vertex_buffers[format as usize]
    }

    /// Get index buffer for a format
    pub fn index_buffer(&self, format: u8) -> &GrowableBuffer {
        &self.index_buffers[format as usize]
    }

    // ========================================================================
    // Device Access
    // ========================================================================

    /// Get wgpu device reference
    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    /// Get wgpu queue reference
    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    /// Get surface format
    pub fn surface_format(&self) -> wgpu::TextureFormat {
        self.config.format
    }

    /// Get depth format
    pub fn depth_format(&self) -> wgpu::TextureFormat {
        wgpu::TextureFormat::Depth32Float
    }

    /// Get depth texture view
    pub fn depth_view(&self) -> &wgpu::TextureView {
        &self.depth_view
    }

    /// Get current surface dimensions
    pub fn dimensions(&self) -> (u32, u32) {
        (self.config.width, self.config.height)
    }

    /// Get current surface width
    pub fn width(&self) -> u32 {
        self.config.width
    }

    /// Get current surface height
    pub fn height(&self) -> u32 {
        self.config.height
    }

    /// Get the current surface texture for rendering
    pub fn get_current_texture(&mut self) -> Result<wgpu::SurfaceTexture> {
        self.surface
            .get_current_texture()
            .context("Failed to acquire next surface texture")
    }

    // ========================================================================
    // Shader Compilation and Pipeline Management
    // ========================================================================

    /// Set the current render mode (0-3)
    ///
    /// This determines which shader templates are used for rendering.
    /// Must be called in init() before any rendering.
    pub fn set_render_mode(&mut self, mode: u8) {
        if mode > 3 {
            tracing::warn!("Invalid render mode: {}, clamping to 3", mode);
            self.current_render_mode = 3;
        } else {
            self.current_render_mode = mode;
            tracing::info!(
                "Set render mode to {} ({})",
                mode,
                crate::shader_gen::mode_name(mode)
            );
        }
    }

    /// Get the current render mode
    pub fn render_mode(&self) -> u8 {
        self.current_render_mode
    }

    /// Get or create a pipeline for the given state
    ///
    /// This caches pipelines to avoid recompilation.
    pub fn get_pipeline(&mut self, format: u8, state: &RenderState) -> &PipelineEntry {
        let key = PipelineKey::new(self.current_render_mode, format, state);

        // Return existing pipeline if cached
        if self.pipelines.contains_key(&key) {
            return &self.pipelines[&key];
        }

        // Otherwise, create a new pipeline
        tracing::debug!(
            "Creating pipeline: mode={}, format={}, blend={:?}, depth={}, cull={:?}",
            self.current_render_mode,
            format,
            state.blend_mode,
            state.depth_test,
            state.cull_mode
        );

        let entry = pipeline::create_pipeline(
            &self.device,
            self.config.format,
            self.current_render_mode,
            format,
            state,
        );
        self.pipelines.insert(key, entry);
        &self.pipelines[&key]
    }

    // ========================================================================
    // Text Rendering
    // ========================================================================

    /// Generate vertex data for rendering text
    ///
    /// This method generates quads for each character in the text string.
    /// The text is rendered in screen space (2D), left-aligned.
    ///
    /// # Arguments
    /// * `text` - UTF-8 text string to render
    /// * `x` - Screen X coordinate in pixels (0 = left edge)
    /// * `y` - Screen Y coordinate in pixels (0 = top edge)
    /// * `size` - Font size in pixels (base size is 8x8, this scales it)
    /// * `color` - Text color as 0xRRGGBBAA
    ///
    /// # Returns
    /// A tuple of (vertices, indices) for POS_UV_COLOR format (format 3)
    ///
    /// # Notes
    /// - Characters outside ASCII 32-126 are rendered as spaces
    /// - This is a simple left-to-right, single-line renderer (no word wrap)
    pub fn generate_text_quads(
        text: &str,
        x: f32,
        y: f32,
        size: f32,
        color: u32,
        font_opt: Option<&crate::state::Font>,
    ) -> (Vec<f32>, Vec<u32>) {
        use crate::font;

        // Extract color components (0xRRGGBBAA)
        let r = ((color >> 24) & 0xFF) as f32 / 255.0;
        let g = ((color >> 16) & 0xFF) as f32 / 255.0;
        let b = ((color >> 8) & 0xFF) as f32 / 255.0;

        let char_count = text.chars().count();
        let mut vertices = Vec::with_capacity(char_count * 4 * 8); // 4 verts × 8 floats
        let mut indices = Vec::with_capacity(char_count * 6); // 6 indices per quad

        let mut cursor_x = x;
        let mut vertex_index = 0u32;

        if let Some(custom_font) = font_opt {
            // Custom font rendering
            let scale = size / custom_font.char_height as f32;
            let glyph_height = custom_font.char_height as f32 * scale;

            // Calculate texture atlas grid dimensions
            // Glyphs are arranged left-to-right, top-to-bottom in a 16-column grid
            let glyphs_per_row = 16;
            let max_glyph_width = custom_font
                .char_widths
                .as_ref()
                .map(|widths| *widths.iter().max().unwrap_or(&custom_font.char_width))
                .unwrap_or(custom_font.char_width);

            let atlas_width = glyphs_per_row * max_glyph_width as usize;
            let atlas_height = ((custom_font.char_count as usize + glyphs_per_row - 1)
                / glyphs_per_row)
                * custom_font.char_height as usize;

            for ch in text.chars() {
                let char_code = ch as u32;

                // Map character to glyph index
                let glyph_index = if char_code >= custom_font.first_codepoint
                    && char_code < custom_font.first_codepoint + custom_font.char_count
                {
                    (char_code - custom_font.first_codepoint) as usize
                } else {
                    0 // Fallback to first character
                };

                // Get glyph width
                let glyph_width_px = custom_font
                    .char_widths
                    .as_ref()
                    .and_then(|widths| widths.get(glyph_index).copied())
                    .unwrap_or(custom_font.char_width);
                let glyph_width = glyph_width_px as f32 * scale;

                // Calculate UV coordinates
                let col = glyph_index % glyphs_per_row;
                let row = glyph_index / glyphs_per_row;

                let u0 = (col * max_glyph_width as usize) as f32 / atlas_width as f32;
                let v0 = (row * custom_font.char_height as usize) as f32 / atlas_height as f32;
                let u1 =
                    ((col * max_glyph_width as usize) + glyph_width_px as usize) as f32
                        / atlas_width as f32;
                let v1 = ((row + 1) * custom_font.char_height as usize) as f32
                    / atlas_height as f32;

                // Screen-space quad vertices (2D)
                // Format: POS_UV_COLOR (format 3)
                // Each vertex: [x, y, z, u, v, r, g, b]

                // Top-left
                vertices.extend_from_slice(&[cursor_x, y, 0.0, u0, v0, r, g, b]);
                // Top-right
                vertices.extend_from_slice(&[cursor_x + glyph_width, y, 0.0, u1, v0, r, g, b]);
                // Bottom-right
                vertices.extend_from_slice(&[
                    cursor_x + glyph_width,
                    y + glyph_height,
                    0.0,
                    u1,
                    v1,
                    r,
                    g,
                    b,
                ]);
                // Bottom-left
                vertices.extend_from_slice(&[cursor_x, y + glyph_height, 0.0, u0, v1, r, g, b]);

                // Indices for two triangles (quad)
                indices.extend_from_slice(&[
                    vertex_index,
                    vertex_index + 1,
                    vertex_index + 2,
                    vertex_index,
                    vertex_index + 2,
                    vertex_index + 3,
                ]);

                cursor_x += glyph_width;
                vertex_index += 4;
            }
        } else {
            // Built-in font rendering (8x8 monospace)
            let scale = size / font::GLYPH_HEIGHT as f32;
            let glyph_width = font::GLYPH_WIDTH as f32 * scale;
            let glyph_height = font::GLYPH_HEIGHT as f32 * scale;

            for ch in text.chars() {
                let char_code = ch as u32;

                // Get UV coordinates for this character
                let (u0, v0, u1, v1) = font::get_glyph_uv(char_code);

                // Screen-space quad vertices (2D)
                // Format: POS_UV_COLOR (format 3)
                // Each vertex: [x, y, z, u, v, r, g, b]

                // Top-left
                vertices.extend_from_slice(&[cursor_x, y, 0.0, u0, v0, r, g, b]);
                // Top-right
                vertices.extend_from_slice(&[cursor_x + glyph_width, y, 0.0, u1, v0, r, g, b]);
                // Bottom-right
                vertices.extend_from_slice(&[
                    cursor_x + glyph_width,
                    y + glyph_height,
                    0.0,
                    u1,
                    v1,
                    r,
                    g,
                    b,
                ]);
                // Bottom-left
                vertices.extend_from_slice(&[cursor_x, y + glyph_height, 0.0, u0, v1, r, g, b]);

                // Indices for two triangles (quad)
                indices.extend_from_slice(&[
                    vertex_index,
                    vertex_index + 1,
                    vertex_index + 2,
                    vertex_index,
                    vertex_index + 2,
                    vertex_index + 3,
                ]);

                cursor_x += glyph_width;
                vertex_index += 4;
            }
        }

        (vertices, indices)
    }

    /// Render the command buffer contents to a texture view
    ///
    /// This is the core rendering function that takes buffered draw commands
    /// and issues GPU draw calls.
    ///
    /// # Arguments
    /// * `view` - The texture view to render to
    /// * `view_matrix` - Camera view matrix
    /// * `projection_matrix` - Camera projection matrix
    /// * `clear_color` - Background clear color (RGBA 0-1)
    pub fn render_frame(
        &mut self,
        view: &wgpu::TextureView,
        view_matrix: Mat4,
        projection_matrix: Mat4,
        clear_color: [f32; 4],
    ) {
        // Create command encoder
        let mut encoder = self
            .device
            .create_command_encoder(&wgpu::CommandEncoderDescriptor {
                label: Some("Game Render Encoder"),
            });

        // If no commands, just clear and return
        if self.command_buffer.commands().is_empty() {
            {
                let _render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                    label: Some("Clear Pass"),
                    color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                        view,
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
                        view: &self.depth_view,
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
            self.queue.submit(std::iter::once(encoder.finish()));
            return;
        }

        // OPTIMIZATION 1: Create frame-level uniform buffers ONCE (not per-command)
        let view_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("View Matrix Buffer"),
                contents: bytemuck::cast_slice(&view_matrix.to_cols_array()),
                usage: wgpu::BufferUsages::UNIFORM,
            });

        let proj_buffer = self
            .device
            .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                label: Some("Projection Matrix Buffer"),
                contents: bytemuck::cast_slice(&projection_matrix.to_cols_array()),
                usage: wgpu::BufferUsages::UNIFORM,
            });

        // Upload vertex/index data from command buffer to GPU buffers
        for format in 0..VERTEX_FORMAT_COUNT as u8 {
            let vertex_data = self.command_buffer.vertex_data(format);
            if !vertex_data.is_empty() {
                self.vertex_buffers[format as usize]
                    .ensure_capacity(&self.device, vertex_data.len() as u64);
                self.vertex_buffers[format as usize].write_at(&self.queue, 0, vertex_data);
            }

            let index_data = self.command_buffer.index_data(format);
            if !index_data.is_empty() {
                let index_bytes: &[u8] = bytemuck::cast_slice(index_data);
                self.index_buffers[format as usize]
                    .ensure_capacity(&self.device, index_bytes.len() as u64);
                self.index_buffers[format as usize].write_at(&self.queue, 0, index_bytes);
            }
        }

        // OPTIMIZATION 3: Sort draw commands IN-PLACE by (pipeline_key, texture_slots) to minimize state changes
        // Commands are reset at the start of next frame, so no need to preserve original order or clone
        self.command_buffer
            .commands_mut()
            .sort_unstable_by_key(|cmd| {
                // Sort key: (render_mode, format, blend_mode, depth_test, cull_mode, texture_slots)
                // This groups commands by pipeline first, then by textures
                let state = RenderState {
                    color: cmd.color,
                    depth_test: cmd.depth_test,
                    cull_mode: cmd.cull_mode,
                    blend_mode: cmd.blend_mode,
                    texture_filter: self.render_state.texture_filter,
                    texture_slots: cmd.texture_slots,
                    matcap_blend_modes: cmd.matcap_blend_modes,
                };
                let pipeline_key = PipelineKey::new(self.current_render_mode, cmd.format, &state);
                (
                    pipeline_key.render_mode,
                    pipeline_key.vertex_format,
                    pipeline_key.blend_mode,
                    pipeline_key.depth_test as u8,
                    pipeline_key.cull_mode,
                    cmd.texture_slots[0].0,
                    cmd.texture_slots[1].0,
                    cmd.texture_slots[2].0,
                    cmd.texture_slots[3].0,
                )
            });

        // OPTIMIZATION 2: Cache bind groups to avoid creating duplicates
        // Material bind group cache: color -> uniform buffer
        // Cache material buffers by (color, metallic_bits, roughness_bits, emissive_bits, matcap_modes)
        let mut material_buffers: HashMap<(u32, u32, u32, u32, (u32, u32, u32, u32)), wgpu::Buffer> = HashMap::new();
        // Texture bind group cache: texture_slots -> bind group
        let mut texture_bind_groups: HashMap<[TextureHandle; 4], wgpu::BindGroup> = HashMap::new();

        // Render pass
        {
            let mut render_pass = encoder.begin_render_pass(&wgpu::RenderPassDescriptor {
                label: Some("Game Render Pass"),
                color_attachments: &[Some(wgpu::RenderPassColorAttachment {
                    view,
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
                    view: &self.depth_view,
                    depth_ops: Some(wgpu::Operations {
                        load: wgpu::LoadOp::Clear(1.0),
                        store: wgpu::StoreOp::Store,
                    }),
                    stencil_ops: None,
                }),
                timestamp_writes: None,
                occlusion_query_set: None,
            });

            for cmd in self.command_buffer.commands() {
                // Create render state from command
                let state = RenderState {
                    color: cmd.color,
                    depth_test: cmd.depth_test,
                    cull_mode: cmd.cull_mode,
                    blend_mode: cmd.blend_mode,
                    texture_filter: self.render_state.texture_filter,
                    texture_slots: cmd.texture_slots,
                    matcap_blend_modes: cmd.matcap_blend_modes,
                };

                // Get/create pipeline
                let pipeline_key = PipelineKey::new(self.current_render_mode, cmd.format, &state);
                if !self.pipelines.contains_key(&pipeline_key) {
                    let entry = pipeline::create_pipeline(
                        &self.device,
                        self.config.format,
                        self.current_render_mode,
                        cmd.format,
                        &state,
                    );
                    self.pipelines.insert(pipeline_key, entry);
                }
                let pipeline_entry = &self.pipelines[&pipeline_key];

                // Get or create material uniform buffer (cached by color + properties + blend modes)
                // Cache key combines color with material properties and matcap blend modes
                let matcap_key = (
                    state.matcap_blend_modes[0] as u32,
                    state.matcap_blend_modes[1] as u32,
                    state.matcap_blend_modes[2] as u32,
                    state.matcap_blend_modes[3] as u32,
                );
                let material_key = (
                    cmd.color,
                    self.material_uniforms.properties[0].to_bits(),
                    self.material_uniforms.properties[1].to_bits(),
                    self.material_uniforms.properties[2].to_bits(),
                    matcap_key,
                );
                let material_buffer = material_buffers.entry(material_key).or_insert_with(|| {
                    let color_vec = state.color_vec4();
                    #[repr(C)]
                    #[derive(Copy, Clone, bytemuck::Pod, bytemuck::Zeroable)]
                    struct MaterialUniforms {
                        color: [f32; 4],
                        matcap_blend_modes: [u32; 4],
                    }
                    let material = MaterialUniforms {
                        color: [color_vec.x, color_vec.y, color_vec.z, color_vec.w],
                        matcap_blend_modes: [
                            state.matcap_blend_modes[0] as u32,
                            state.matcap_blend_modes[1] as u32,
                            state.matcap_blend_modes[2] as u32,
                            state.matcap_blend_modes[3] as u32,
                        ],
                    };
                    self.device
                        .create_buffer_init(&wgpu::util::BufferInitDescriptor {
                            label: Some("Material Buffer"),
                            contents: bytemuck::cast_slice(&[material]),
                            usage: wgpu::BufferUsages::UNIFORM,
                        })
                });

                // Create frame bind group (group 0)
                // Note: This contains the material buffer which varies per-command,
                // so we can't easily cache it. However, we've eliminated redundant
                // material buffer creation via the cache above.
                let frame_bind_group = match self.current_render_mode {
                    0 | 1 => {
                        // Mode 0 (Unlit) and Mode 1 (Matcap): Basic bindings
                        self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                            label: Some("Frame Bind Group"),
                            layout: &pipeline_entry.bind_group_layout_frame,
                            entries: &[
                                wgpu::BindGroupEntry {
                                    binding: 0,
                                    resource: view_buffer.as_entire_binding(),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 1,
                                    resource: proj_buffer.as_entire_binding(),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 2,
                                    resource: self.sky_buffer.as_entire_binding(),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 3,
                                    resource: material_buffer.as_entire_binding(),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 4,
                                    resource: self.bone_buffer.as_entire_binding(),
                                },
                            ],
                        })
                    }
                    2 | 3 => {
                        // Mode 2 (PBR) and Mode 3 (Hybrid): Additional lighting uniforms
                        self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                            label: Some("Frame Bind Group"),
                            layout: &pipeline_entry.bind_group_layout_frame,
                            entries: &[
                                wgpu::BindGroupEntry {
                                    binding: 0,
                                    resource: view_buffer.as_entire_binding(),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 1,
                                    resource: proj_buffer.as_entire_binding(),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 2,
                                    resource: self.sky_buffer.as_entire_binding(),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 3,
                                    resource: material_buffer.as_entire_binding(),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 4,
                                    resource: self.lights_buffer.as_entire_binding(),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 5,
                                    resource: self.camera_buffer.as_entire_binding(),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 6,
                                    resource: self.bone_buffer.as_entire_binding(),
                                },
                            ],
                        })
                    }
                    _ => {
                        // Fallback - same as mode 0/1
                        self.device.create_bind_group(&wgpu::BindGroupDescriptor {
                            label: Some("Frame Bind Group"),
                            layout: &pipeline_entry.bind_group_layout_frame,
                            entries: &[
                                wgpu::BindGroupEntry {
                                    binding: 0,
                                    resource: view_buffer.as_entire_binding(),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 1,
                                    resource: proj_buffer.as_entire_binding(),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 2,
                                    resource: self.sky_buffer.as_entire_binding(),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 3,
                                    resource: material_buffer.as_entire_binding(),
                                },
                                wgpu::BindGroupEntry {
                                    binding: 4,
                                    resource: self.bone_buffer.as_entire_binding(),
                                },
                            ],
                        })
                    }
                };

                // Get or create texture bind group (cached by texture slots)
                let texture_bind_group = texture_bind_groups
                    .entry(cmd.texture_slots)
                    .or_insert_with(|| {
                        // Get texture views for this command's bound textures
                        let tex_view_0 = if cmd.texture_slots[0] == TextureHandle::INVALID {
                            self.get_fallback_white_view()
                        } else {
                            self.get_texture_view(cmd.texture_slots[0])
                                .unwrap_or_else(|| self.get_fallback_checkerboard_view())
                        };
                        let tex_view_1 = if cmd.texture_slots[1] == TextureHandle::INVALID {
                            self.get_fallback_white_view()
                        } else {
                            self.get_texture_view(cmd.texture_slots[1])
                                .unwrap_or_else(|| self.get_fallback_checkerboard_view())
                        };
                        let tex_view_2 = if cmd.texture_slots[2] == TextureHandle::INVALID {
                            self.get_fallback_white_view()
                        } else {
                            self.get_texture_view(cmd.texture_slots[2])
                                .unwrap_or_else(|| self.get_fallback_checkerboard_view())
                        };
                        let tex_view_3 = if cmd.texture_slots[3] == TextureHandle::INVALID {
                            self.get_fallback_white_view()
                        } else {
                            self.get_texture_view(cmd.texture_slots[3])
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
                                wgpu::BindGroupEntry {
                                    binding: 4,
                                    resource: wgpu::BindingResource::Sampler(
                                        self.current_sampler(),
                                    ),
                                },
                            ],
                        })
                    });

                // Set pipeline and bind groups
                render_pass.set_pipeline(&pipeline_entry.pipeline);
                render_pass.set_bind_group(0, &frame_bind_group, &[]);
                render_pass.set_bind_group(1, &*texture_bind_group, &[]);

                // Set vertex buffer
                if let Some(buffer) = self.vertex_buffers[cmd.format as usize].buffer() {
                    render_pass.set_vertex_buffer(0, buffer.slice(..));
                }

                // Draw
                if cmd.index_count > 0 {
                    // Indexed draw
                    if let Some(buffer) = self.index_buffers[cmd.format as usize].buffer() {
                        render_pass.set_index_buffer(buffer.slice(..), wgpu::IndexFormat::Uint32);
                        render_pass.draw_indexed(
                            cmd.first_index..cmd.first_index + cmd.index_count,
                            cmd.base_vertex as i32,
                            0..1,
                        );
                    }
                } else {
                    // Non-indexed draw
                    render_pass.draw(cmd.base_vertex..cmd.base_vertex + cmd.vertex_count, 0..1);
                }
            }
        }

        // Submit commands
        self.queue.submit(std::iter::once(encoder.finish()));
    }
}

impl Graphics for ZGraphics {
    fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }

        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);

        // Recreate depth buffer
        let (depth_texture, depth_view) = Self::create_depth_texture(&self.device, width, height);
        self.depth_texture = depth_texture;
        self.depth_view = depth_view;

        tracing::debug!("Resized graphics to {}x{}", width, height);
    }

    fn begin_frame(&mut self) {
        // Reset command buffer for new frame
        self.command_buffer.reset();

        // Reset transform to identity
        self.current_transform = Mat4::IDENTITY;
        self.transform_stack.clear();

        // Acquire next frame
        let frame = match self.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
                // Reconfigure surface and try again
                self.surface.configure(&self.device, &self.config);
                match self.surface.get_current_texture() {
                    Ok(frame) => frame,
                    Err(e) => {
                        tracing::error!("Failed to acquire frame after reconfigure: {:?}", e);
                        return;
                    }
                }
            }
            Err(e) => {
                tracing::error!("Failed to acquire frame: {:?}", e);
                return;
            }
        };

        let view = frame
            .texture
            .create_view(&wgpu::TextureViewDescriptor::default());
        self.current_frame = Some(frame);
        self.current_view = Some(view);
    }

    fn end_frame(&mut self) {
        // Present frame
        if let Some(frame) = self.current_frame.take() {
            frame.present();
        }
        self.current_view = None;
    }

    fn set_bones(&mut self, bones: &[Mat4]) {
        if bones.is_empty() {
            return;
        }

        // Clamp to maximum 256 bones
        let bone_count = bones.len().min(256);

        // Convert Mat4 matrices to flat f32 array for GPU upload
        // Each Mat4 is 16 floats (64 bytes)
        let mut bone_data: Vec<f32> = Vec::with_capacity(bone_count * 16);
        for matrix in &bones[..bone_count] {
            bone_data.extend_from_slice(&matrix.to_cols_array());
        }

        // Upload to GPU bone storage buffer
        self.queue
            .write_buffer(&self.bone_buffer, 0, bytemuck::cast_slice(&bone_data));

        tracing::trace!("Uploaded {} bone matrices to GPU", bone_count);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_generate_text_quads_empty() {
        let (vertices, indices) = ZGraphics::generate_text_quads("", 0.0, 0.0, 16.0, 0xFFFFFFFF, None);
        assert!(vertices.is_empty());
        assert!(indices.is_empty());
    }

    #[test]
    fn test_generate_text_quads_single_char() {
        let (vertices, indices) = ZGraphics::generate_text_quads("A", 0.0, 0.0, 16.0, 0xFFFFFFFF, None);
        assert_eq!(vertices.len(), 32);
        assert_eq!(indices.len(), 6);
    }

    #[test]
    fn test_generate_text_quads_multiple_chars() {
        let (vertices, indices) =
            ZGraphics::generate_text_quads("Hello", 0.0, 0.0, 8.0, 0xFFFFFFFF, None);
        assert_eq!(vertices.len(), 160);
        assert_eq!(indices.len(), 30);
    }

    #[test]
    fn test_generate_text_quads_color() {
        let (vertices, _) = ZGraphics::generate_text_quads("X", 0.0, 0.0, 8.0, 0xFF0000FF, None);
        assert!((vertices[5] - 1.0).abs() < 0.01);
        assert!((vertices[6] - 0.0).abs() < 0.01);
        assert!((vertices[7] - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_generate_text_quads_position() {
        let (vertices, _) = ZGraphics::generate_text_quads("A", 100.0, 50.0, 16.0, 0xFFFFFFFF, None);
        assert!((vertices[0] - 100.0).abs() < 0.01);
        assert!((vertices[1] - 50.0).abs() < 0.01);
        assert!((vertices[2] - 0.0).abs() < 0.01);
    }

    #[test]
    fn test_generate_text_quads_indices_valid() {
        let (_, indices) = ZGraphics::generate_text_quads("AB", 0.0, 0.0, 8.0, 0xFFFFFFFF, None);
        assert_eq!(indices[0..6], [0, 1, 2, 0, 2, 3]);
        assert_eq!(indices[6..12], [4, 5, 6, 4, 6, 7]);
    }
}
