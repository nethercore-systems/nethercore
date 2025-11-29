//! Emberware Z graphics backend (wgpu)
//!
//! Implements the `Graphics` trait from emberware-core with a wgpu-based
//! renderer featuring PS1/N64 aesthetic (vertex jitter, affine textures).

use std::collections::HashMap;
use std::sync::Arc;

use anyhow::{Context, Result};
use glam::Vec4;
use wgpu::util::DeviceExt;
use winit::window::Window;

use emberware_core::console::Graphics;

use crate::console::VRAM_LIMIT;

// ============================================================================
// Texture Handle
// ============================================================================

/// Handle to a loaded texture
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct TextureHandle(pub u32);

impl TextureHandle {
    /// Invalid/null texture handle
    pub const INVALID: TextureHandle = TextureHandle(0);
}

// ============================================================================
// Render State Enums
// ============================================================================

/// Cull mode for face culling
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u32)]
pub enum CullMode {
    /// No face culling
    #[default]
    None = 0,
    /// Cull back faces
    Back = 1,
    /// Cull front faces
    Front = 2,
}

impl CullMode {
    pub fn from_u32(value: u32) -> Self {
        match value {
            0 => CullMode::None,
            1 => CullMode::Back,
            2 => CullMode::Front,
            _ => CullMode::None,
        }
    }

    pub fn to_wgpu(self) -> Option<wgpu::Face> {
        match self {
            CullMode::None => None,
            CullMode::Back => Some(wgpu::Face::Back),
            CullMode::Front => Some(wgpu::Face::Front),
        }
    }
}

/// Blend mode for alpha blending
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u32)]
pub enum BlendMode {
    /// No blending (opaque)
    #[default]
    None = 0,
    /// Standard alpha blending
    Alpha = 1,
    /// Additive blending
    Additive = 2,
    /// Multiply blending
    Multiply = 3,
}

impl BlendMode {
    pub fn from_u32(value: u32) -> Self {
        match value {
            0 => BlendMode::None,
            1 => BlendMode::Alpha,
            2 => BlendMode::Additive,
            3 => BlendMode::Multiply,
            _ => BlendMode::None,
        }
    }

    pub fn to_wgpu(self) -> Option<wgpu::BlendState> {
        match self {
            BlendMode::None => None,
            BlendMode::Alpha => Some(wgpu::BlendState::ALPHA_BLENDING),
            BlendMode::Additive => Some(wgpu::BlendState {
                color: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::SrcAlpha,
                    dst_factor: wgpu::BlendFactor::One,
                    operation: wgpu::BlendOperation::Add,
                },
                alpha: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::One,
                    dst_factor: wgpu::BlendFactor::One,
                    operation: wgpu::BlendOperation::Add,
                },
            }),
            BlendMode::Multiply => Some(wgpu::BlendState {
                color: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::Dst,
                    dst_factor: wgpu::BlendFactor::Zero,
                    operation: wgpu::BlendOperation::Add,
                },
                alpha: wgpu::BlendComponent {
                    src_factor: wgpu::BlendFactor::DstAlpha,
                    dst_factor: wgpu::BlendFactor::Zero,
                    operation: wgpu::BlendOperation::Add,
                },
            }),
        }
    }
}

/// Texture filter mode
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default)]
#[repr(u32)]
pub enum TextureFilter {
    /// Nearest neighbor (pixelated)
    #[default]
    Nearest = 0,
    /// Linear interpolation (smooth)
    Linear = 1,
}

impl TextureFilter {
    pub fn from_u32(value: u32) -> Self {
        match value {
            0 => TextureFilter::Nearest,
            1 => TextureFilter::Linear,
            _ => TextureFilter::Nearest,
        }
    }

    pub fn to_wgpu(self) -> wgpu::FilterMode {
        match self {
            TextureFilter::Nearest => wgpu::FilterMode::Nearest,
            TextureFilter::Linear => wgpu::FilterMode::Linear,
        }
    }
}

// ============================================================================
// Render State
// ============================================================================

/// Current render state (tracks what needs pipeline changes)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct RenderState {
    /// Uniform tint color (0xRRGGBBAA)
    pub color: u32,
    /// Depth test enabled
    pub depth_test: bool,
    /// Face culling mode
    pub cull_mode: CullMode,
    /// Blending mode
    pub blend_mode: BlendMode,
    /// Texture filter mode
    pub texture_filter: TextureFilter,
    /// Bound textures per slot (0-3)
    pub texture_slots: [TextureHandle; 4],
}

impl Default for RenderState {
    fn default() -> Self {
        Self {
            color: 0xFFFFFFFF, // White, fully opaque
            depth_test: true,
            cull_mode: CullMode::Back,
            blend_mode: BlendMode::None,
            texture_filter: TextureFilter::Nearest,
            texture_slots: [TextureHandle::INVALID; 4],
        }
    }
}

impl RenderState {
    /// Get color as Vec4 (RGBA, 0.0-1.0)
    pub fn color_vec4(&self) -> Vec4 {
        Vec4::new(
            ((self.color >> 24) & 0xFF) as f32 / 255.0,
            ((self.color >> 16) & 0xFF) as f32 / 255.0,
            ((self.color >> 8) & 0xFF) as f32 / 255.0,
            (self.color & 0xFF) as f32 / 255.0,
        )
    }
}

// ============================================================================
// Texture Entry
// ============================================================================

/// Internal texture data
struct TextureEntry {
    texture: wgpu::Texture,
    view: wgpu::TextureView,
    width: u32,
    height: u32,
    /// Size in bytes (for VRAM tracking)
    size_bytes: usize,
}

// ============================================================================
// ZGraphics
// ============================================================================

/// Emberware Z graphics backend
///
/// Manages wgpu device, textures, render state, and frame presentation.
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

    // Samplers
    sampler_nearest: wgpu::Sampler,
    sampler_linear: wgpu::Sampler,

    // Current render state
    render_state: RenderState,

    // Frame state
    current_frame: Option<wgpu::SurfaceTexture>,
    current_view: Option<wgpu::TextureView>,
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
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
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
            sampler_nearest,
            sampler_linear,
            render_state: RenderState::default(),
            current_frame: None,
            current_view: None,
        };

        // Create fallback textures
        graphics.create_fallback_textures();

        Ok(graphics)
    }

    /// Create a new ZGraphics instance (blocking version for sync contexts)
    pub fn new_blocking(window: Arc<Window>) -> Result<Self> {
        pollster::block_on(Self::new(window))
    }

    /// Create depth texture and view
    fn create_depth_texture(device: &wgpu::Device, width: u32, height: u32) -> (wgpu::Texture, wgpu::TextureView) {
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

    /// Create fallback textures (checkerboard and white)
    fn create_fallback_textures(&mut self) {
        // 8x8 magenta/black checkerboard for missing textures
        let mut checkerboard_data = vec![0u8; 8 * 8 * 4];
        for y in 0..8 {
            for x in 0..8 {
                let idx = (y * 8 + x) * 4;
                let is_magenta = (x + y) % 2 == 0;
                if is_magenta {
                    checkerboard_data[idx] = 255;     // R
                    checkerboard_data[idx + 1] = 0;   // G
                    checkerboard_data[idx + 2] = 255; // B
                    checkerboard_data[idx + 3] = 255; // A
                } else {
                    checkerboard_data[idx] = 0;       // R
                    checkerboard_data[idx + 1] = 0;   // G
                    checkerboard_data[idx + 2] = 0;   // B
                    checkerboard_data[idx + 3] = 255; // A
                }
            }
        }
        self.fallback_checkerboard = self
            .load_texture_internal(8, 8, &checkerboard_data, false)
            .expect("Failed to create checkerboard fallback texture");

        // 1x1 white texture for untextured draws
        let white_data = [255u8, 255, 255, 255];
        self.fallback_white = self
            .load_texture_internal(1, 1, &white_data, false)
            .expect("Failed to create white fallback texture");
    }

    // ========================================================================
    // Texture Management
    // ========================================================================

    /// Load a texture from RGBA8 pixel data
    ///
    /// Returns a TextureHandle or an error if VRAM budget is exceeded.
    pub fn load_texture(&mut self, width: u32, height: u32, pixels: &[u8]) -> Result<TextureHandle> {
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

    /// Get texture view for a slot, returning fallback if unbound
    pub fn get_slot_texture_view(&self, slot: usize) -> &wgpu::TextureView {
        let handle = self.render_state.texture_slots.get(slot).copied().unwrap_or(TextureHandle::INVALID);
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

        let view = frame.texture.create_view(&wgpu::TextureViewDescriptor::default());
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
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_render_state_default() {
        let state = RenderState::default();
        assert_eq!(state.color, 0xFFFFFFFF);
        assert!(state.depth_test);
        assert_eq!(state.cull_mode, CullMode::Back);
        assert_eq!(state.blend_mode, BlendMode::None);
        assert_eq!(state.texture_filter, TextureFilter::Nearest);
        assert_eq!(state.texture_slots, [TextureHandle::INVALID; 4]);
    }

    #[test]
    fn test_render_state_color_vec4() {
        let state = RenderState {
            color: 0xFF8040C0,
            ..Default::default()
        };
        let v = state.color_vec4();
        assert!((v.x - 1.0).abs() < 0.01);       // R = 0xFF
        assert!((v.y - 0.502).abs() < 0.01);    // G = 0x80
        assert!((v.z - 0.251).abs() < 0.01);    // B = 0x40
        assert!((v.w - 0.753).abs() < 0.01);    // A = 0xC0
    }

    #[test]
    fn test_cull_mode_conversion() {
        assert_eq!(CullMode::from_u32(0), CullMode::None);
        assert_eq!(CullMode::from_u32(1), CullMode::Back);
        assert_eq!(CullMode::from_u32(2), CullMode::Front);
        assert_eq!(CullMode::from_u32(99), CullMode::None);

        assert!(CullMode::None.to_wgpu().is_none());
        assert_eq!(CullMode::Back.to_wgpu(), Some(wgpu::Face::Back));
        assert_eq!(CullMode::Front.to_wgpu(), Some(wgpu::Face::Front));
    }

    #[test]
    fn test_blend_mode_conversion() {
        assert_eq!(BlendMode::from_u32(0), BlendMode::None);
        assert_eq!(BlendMode::from_u32(1), BlendMode::Alpha);
        assert_eq!(BlendMode::from_u32(2), BlendMode::Additive);
        assert_eq!(BlendMode::from_u32(3), BlendMode::Multiply);
        assert_eq!(BlendMode::from_u32(99), BlendMode::None);

        assert!(BlendMode::None.to_wgpu().is_none());
        assert!(BlendMode::Alpha.to_wgpu().is_some());
        assert!(BlendMode::Additive.to_wgpu().is_some());
        assert!(BlendMode::Multiply.to_wgpu().is_some());
    }

    #[test]
    fn test_texture_filter_conversion() {
        assert_eq!(TextureFilter::from_u32(0), TextureFilter::Nearest);
        assert_eq!(TextureFilter::from_u32(1), TextureFilter::Linear);
        assert_eq!(TextureFilter::from_u32(99), TextureFilter::Nearest);

        assert_eq!(TextureFilter::Nearest.to_wgpu(), wgpu::FilterMode::Nearest);
        assert_eq!(TextureFilter::Linear.to_wgpu(), wgpu::FilterMode::Linear);
    }

    #[test]
    fn test_texture_handle_invalid() {
        assert_eq!(TextureHandle::INVALID, TextureHandle(0));
    }
}
