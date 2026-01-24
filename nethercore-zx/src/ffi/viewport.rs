//! Viewport FFI functions for split-screen rendering
//!
//! Provides functions to set the viewport rectangle for subsequent draw calls.
//! Each viewport can have its own camera and all 2D coordinates become viewport-relative.

use anyhow::Result;
use tracing::warn;
use wasmtime::{Caller, Linker};

use super::ZXGameContext;
use crate::console::RESOLUTION;

/// Register viewport FFI functions
pub fn register(linker: &mut Linker<ZXGameContext>) -> Result<()> {
    linker.func_wrap("env", "viewport", viewport)?;
    linker.func_wrap("env", "viewport_clear", viewport_clear)?;
    Ok(())
}

/// Set the viewport for subsequent draw calls.
///
/// All 3D and 2D rendering will be clipped to this region.
/// Camera aspect ratio automatically adjusts to viewport dimensions.
/// 2D coordinates (draw_sprite, draw_text, etc.) become viewport-relative.
///
/// # Arguments
/// * `x` - Left edge in pixels (0-959)
/// * `y` - Top edge in pixels (0-539)
/// * `width` - Width in pixels (1-960)
/// * `height` - Height in pixels (1-540)
///
/// # Example (2-player horizontal split)
/// ```ignore
/// viewport(0, 0, 480, 540);     // Player 1: left half
/// camera_set(p1_cam...);
/// epu_draw(env_config_ptr);
/// draw_scene();
///
/// viewport(480, 0, 480, 540);   // Player 2: right half
/// camera_set(p2_cam...);
/// epu_draw(env_config_ptr);
/// draw_scene();
///
/// viewport_clear();  // Reset for HUD or next frame
/// ```
fn viewport(mut caller: Caller<'_, ZXGameContext>, x: u32, y: u32, width: u32, height: u32) {
    let (res_w, res_h) = RESOLUTION;

    // Validate origin is within screen bounds
    if x >= res_w || y >= res_h {
        warn!(
            "viewport: origin ({}, {}) out of bounds (screen is {}x{})",
            x, y, res_w, res_h
        );
        return;
    }

    // Validate dimensions are non-zero
    if width == 0 || height == 0 {
        warn!(
            "viewport: dimensions must be > 0 (got {}x{})",
            width, height
        );
        return;
    }

    // Clamp dimensions to fit within screen bounds
    let clamped_width = width.min(res_w - x);
    let clamped_height = height.min(res_h - y);

    if clamped_width != width || clamped_height != height {
        warn!(
            "viewport: dimensions clamped from {}x{} to {}x{} to fit screen",
            width, height, clamped_width, clamped_height
        );
    }

    let state = &mut caller.data_mut().ffi;
    state.current_viewport = crate::graphics::Viewport {
        x,
        y,
        width: clamped_width,
        height: clamped_height,
    };
}

/// Reset viewport to fullscreen (ZX native resolution).
///
/// Call this at the end of split-screen rendering to restore full-screen
/// coordinates for HUD elements or between frames.
fn viewport_clear(mut caller: Caller<'_, ZXGameContext>) {
    let state = &mut caller.data_mut().ffi;
    state.current_viewport = crate::graphics::Viewport::FULLSCREEN;
}

#[cfg(test)]
mod tests {
    #[test]
    fn test_viewport_fullscreen_constant() {
        let vp = crate::graphics::Viewport::FULLSCREEN;
        assert_eq!(vp.x, 0);
        assert_eq!(vp.y, 0);
        let (width, height) = crate::console::RESOLUTION;
        assert_eq!(vp.width, width);
        assert_eq!(vp.height, height);
    }

    #[test]
    fn test_viewport_aspect_ratio() {
        // Full screen: 16:9
        let vp = crate::graphics::Viewport::FULLSCREEN;
        assert!((vp.aspect_ratio() - 16.0 / 9.0).abs() < 0.001);

        // Half width: 8:9
        let vp_half = crate::graphics::Viewport {
            x: 0,
            y: 0,
            width: 480,
            height: 540,
        };
        assert!((vp_half.aspect_ratio() - 8.0 / 9.0).abs() < 0.001);

        // Quarter screen: 8:9
        let vp_quarter = crate::graphics::Viewport {
            x: 0,
            y: 0,
            width: 480,
            height: 270,
        };
        assert!((vp_quarter.aspect_ratio() - 16.0 / 9.0).abs() < 0.001);
    }
}
