//! Viewport Functions (Split-Screen)

#[cfg(target_arch = "wasm32")]
#[link(wasm_import_module = "env")]
extern "C" {
    /// Set the viewport for subsequent draw calls.
    ///
    /// All 3D and 2D rendering will be clipped to this region.
    /// Camera aspect ratio automatically adjusts to viewport dimensions.
    /// 2D coordinates (draw_sprite, draw_text, etc.) become viewport-relative.
    ///
    /// # Arguments
    /// * `x` — Left edge in pixels (0-959)
    /// * `y` — Top edge in pixels (0-539)
    /// * `width` — Width in pixels (1-960)
    /// * `height` — Height in pixels (1-540)
    ///
    /// # Example (2-player horizontal split)
    /// ```rust,ignore
    /// // Player 1: left half
    /// viewport(0, 0, 480, 540);
    /// camera_set(p1_x, p1_y, p1_z, p1_tx, p1_ty, p1_tz);
    /// epu_set(env_config_ptr);
    /// draw_mesh(scene);
    /// draw_epu();
    ///
    /// // Player 2: right half
    /// viewport(480, 0, 480, 540);
    /// camera_set(p2_x, p2_y, p2_z, p2_tx, p2_ty, p2_tz);
    /// epu_set(env_config_ptr);
    /// draw_mesh(scene);
    /// draw_epu();
    ///
    /// // Reset for HUD
    /// viewport_clear();
    /// set_color(0xFFFFFFFF);
    /// draw_text_str("PAUSED", 400.0, 270.0, 32.0);
    /// ```
    pub fn viewport(x: u32, y: u32, width: u32, height: u32);

    /// Reset viewport to fullscreen (960×540).
    ///
    /// Call this at the end of split-screen rendering to restore full-screen
    /// coordinates for HUD elements or between frames.
    pub fn viewport_clear();
}
