//! Blit operations for rendering the offscreen render target to window
//!
//! Handles viewport scaling modes (Stretch, Fit, PixelPerfect) and
//! final presentation to the window surface.

use super::super::ZXGraphics;

impl ZXGraphics {
    /// Blit the render target to the window surface
    /// Call this every frame to display the last rendered content
    pub fn blit_to_window(&self, encoder: &mut wgpu::CommandEncoder, view: &wgpu::TextureView) {
        // Calculate viewport based on scale mode
        let (viewport_x, viewport_y, viewport_width, viewport_height) = match self.scale_mode {
            nethercore_core::app::config::ScaleMode::Stretch => {
                // Stretch to fill window (may distort aspect ratio)
                (
                    0.0,
                    0.0,
                    self.config.width as f32,
                    self.config.height as f32,
                )
            }
            nethercore_core::app::config::ScaleMode::Fit => {
                // Maintain aspect ratio, scale to fill as much as possible
                let render_width = self.render_target.width as f32;
                let render_height = self.render_target.height as f32;
                let window_width = self.config.width as f32;
                let window_height = self.config.height as f32;

                // Calculate scale factor that fits within window while maintaining aspect ratio
                let scale_x = window_width / render_width;
                let scale_y = window_height / render_height;
                let scale = scale_x.min(scale_y);

                // Calculate scaled dimensions
                let scaled_width = render_width * scale;
                let scaled_height = render_height * scale;

                // Center the viewport (letterbox/pillarbox)
                let x = (window_width - scaled_width) / 2.0;
                let y = (window_height - scaled_height) / 2.0;

                (x, y, scaled_width, scaled_height)
            }
            nethercore_core::app::config::ScaleMode::PixelPerfect => {
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
                    depth_slice: None,
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
}
