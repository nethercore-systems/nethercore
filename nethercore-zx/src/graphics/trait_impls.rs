//! Trait implementations for ZXGraphics

use anyhow::Result;

use nethercore_core::console::Graphics;

use super::zx_graphics::ZXGraphics;

impl Graphics for ZXGraphics {
    fn resize(&mut self, width: u32, height: u32) {
        if width == 0 || height == 0 {
            return;
        }

        self.config.width = width;
        self.config.height = height;
        self.surface.configure(&self.device, &self.config);

        let (depth_texture, depth_view) = Self::create_depth_texture(&self.device, width, height);
        self.depth_texture = depth_texture;
        self.depth_view = depth_view;

        tracing::debug!("Resized graphics to {}x{}", width, height);
    }

    fn begin_frame(&mut self) {
        self.command_buffer.reset();

        let frame = match self.surface.get_current_texture() {
            Ok(frame) => frame,
            Err(wgpu::SurfaceError::Lost | wgpu::SurfaceError::Outdated) => {
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
        if let Some(frame) = self.current_frame.take() {
            frame.present();
        }
        self.current_view = None;
    }
}

impl nethercore_core::capture::CaptureSupport for ZXGraphics {
    fn device(&self) -> &wgpu::Device {
        &self.device
    }

    fn queue(&self) -> &wgpu::Queue {
        &self.queue
    }

    fn render_target_texture(&self) -> &wgpu::Texture {
        &self.render_target.color_texture
    }

    fn render_target_dimensions(&self) -> (u32, u32) {
        (self.render_target.width, self.render_target.height)
    }
}

impl nethercore_core::app::StandaloneGraphicsSupport for ZXGraphics {
    fn surface_format(&self) -> wgpu::TextureFormat {
        self.config.format
    }

    fn width(&self) -> u32 {
        self.config.width
    }

    fn height(&self) -> u32 {
        self.config.height
    }

    fn get_current_texture(&mut self) -> Result<wgpu::SurfaceTexture, wgpu::SurfaceError> {
        self.surface.get_current_texture()
    }

    fn blit_to_window(&self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) {
        // Delegate to existing method in frame.rs
        ZXGraphics::blit_to_window(self, encoder, view)
    }

    fn set_scale_mode(&mut self, mode: nethercore_core::app::config::ScaleMode) {
        self.scale_mode = mode;
    }
}
