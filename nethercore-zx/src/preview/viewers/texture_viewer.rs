//! Texture viewer controls and state

use super::ZXAssetViewer;

impl ZXAssetViewer {
    /// Zoom in on texture preview
    pub fn texture_zoom_in(&mut self) {
        self.texture_zoom = (self.texture_zoom * 1.5).min(10.0);
    }

    /// Zoom out on texture preview
    pub fn texture_zoom_out(&mut self) {
        self.texture_zoom = (self.texture_zoom / 1.5).max(0.1);
    }

    /// Reset texture zoom to 1:1
    pub fn texture_reset_zoom(&mut self) {
        self.texture_zoom = 1.0;
        self.texture_pan = (0.0, 0.0);
    }

    /// Pan texture preview
    pub fn texture_pan(&mut self, dx: f32, dy: f32) {
        self.texture_pan.0 += dx;
        self.texture_pan.1 += dy;
    }

    /// Get current texture zoom level
    pub fn texture_zoom(&self) -> f32 {
        self.texture_zoom
    }

    /// Get current texture pan offset
    pub fn texture_pan_offset(&self) -> (f32, f32) {
        self.texture_pan
    }

    /// Update texture cache if needed and return the cached handle.
    ///
    /// This is a helper method to reduce duplication between texture and font preview.
    /// Creates or updates the cached egui texture handle based on the cache_id.
    pub(super) fn update_texture_cache(
        &mut self,
        ctx: &egui::Context,
        cache_id: &str,
        texture_name: &str,
        width: u32,
        height: u32,
        rgba_data: &[u8],
    ) {
        if self.cached_texture_id.as_ref() != Some(&cache_id.to_string()) {
            let color_image = egui::ColorImage::from_rgba_unmultiplied(
                [width as usize, height as usize],
                rgba_data,
            );

            let filter = if self.texture_linear_filter {
                egui::TextureFilter::Linear
            } else {
                egui::TextureFilter::Nearest
            };

            let options = egui::TextureOptions {
                magnification: filter,
                minification: filter,
                ..Default::default()
            };

            self.cached_texture = Some(ctx.load_texture(texture_name, color_image, options));
            self.cached_texture_id = Some(cache_id.to_string());
        }
    }
}
