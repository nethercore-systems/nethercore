//! Simple wgpu graphics context for egui-only rendering
//!
//! This provides the minimal graphics infrastructure needed to render the
//! library UI. Games are rendered in separate player processes.

use std::sync::Arc;

use anyhow::{Context, Result};
use winit::window::Window;

/// Simple graphics context for the library UI.
///
/// This only contains the wgpu state needed to render egui.
/// Game rendering happens in separate player processes.
pub struct LibraryGraphics {
    /// wgpu device
    device: wgpu::Device,
    /// wgpu queue
    queue: wgpu::Queue,
    /// Window surface
    surface: wgpu::Surface<'static>,
    /// Surface configuration
    surface_config: wgpu::SurfaceConfiguration,
}

impl LibraryGraphics {
    /// Create graphics context for the given window.
    pub fn new(window: Arc<Window>) -> Result<Self> {
        let instance = wgpu::Instance::new(&wgpu::InstanceDescriptor {
            backends: wgpu::Backends::all(),
            ..Default::default()
        });

        let surface = instance
            .create_surface(window.clone())
            .context("Failed to create surface")?;

        let adapter = pollster::block_on(instance.request_adapter(&wgpu::RequestAdapterOptions {
            power_preference: wgpu::PowerPreference::LowPower,
            compatible_surface: Some(&surface),
            force_fallback_adapter: false,
        }))
        .context("Failed to find suitable GPU adapter")?;

        let (device, queue) = pollster::block_on(adapter.request_device(&wgpu::DeviceDescriptor {
            label: Some("Library Device"),
            required_features: wgpu::Features::empty(),
            required_limits: wgpu::Limits::default(),
            memory_hints: wgpu::MemoryHints::Performance,
            experimental_features: Default::default(),
            trace: wgpu::Trace::Off,
        }))
        .context("Failed to create GPU device")?;

        let size = window.inner_size();
        let surface_caps = surface.get_capabilities(&adapter);
        let surface_format = surface_caps
            .formats
            .iter()
            .find(|f| f.is_srgb())
            .copied()
            .unwrap_or(surface_caps.formats[0]);

        let surface_config = wgpu::SurfaceConfiguration {
            usage: wgpu::TextureUsages::RENDER_ATTACHMENT,
            format: surface_format,
            width: size.width.max(1),
            height: size.height.max(1),
            present_mode: wgpu::PresentMode::AutoVsync,
            alpha_mode: surface_caps.alpha_modes[0],
            view_formats: vec![],
            desired_maximum_frame_latency: 2,
        };

        surface.configure(&device, &surface_config);

        tracing::info!(
            "Library graphics initialized: {}x{}, format: {:?}",
            surface_config.width,
            surface_config.height,
            surface_format
        );

        Ok(Self {
            device,
            queue,
            surface,
            surface_config,
        })
    }

    /// Get the wgpu device.
    pub fn device(&self) -> &wgpu::Device {
        &self.device
    }

    /// Get the wgpu queue.
    pub fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    /// Get the surface format.
    pub fn surface_format(&self) -> wgpu::TextureFormat {
        self.surface_config.format
    }

    /// Get the current width.
    pub fn width(&self) -> u32 {
        self.surface_config.width
    }

    /// Get the current height.
    pub fn height(&self) -> u32 {
        self.surface_config.height
    }

    /// Get the current surface texture for rendering.
    pub fn get_current_texture(&self) -> Result<wgpu::SurfaceTexture, wgpu::SurfaceError> {
        self.surface.get_current_texture()
    }

    /// Resize the surface.
    pub fn resize(&mut self, width: u32, height: u32) {
        if width > 0 && height > 0 {
            self.surface_config.width = width;
            self.surface_config.height = height;
            self.surface.configure(&self.device, &self.surface_config);
            tracing::debug!("Library surface resized to {}x{}", width, height);
        }
    }
}
