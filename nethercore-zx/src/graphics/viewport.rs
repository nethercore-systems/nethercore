//! Viewport management for split-screen rendering

use crate::console::RESOLUTION;

/// Rectangular viewport for split-screen rendering.
///
/// Defines the screen region where rendering occurs. Each viewport can have
/// its own camera, and 2D coordinates are relative to the viewport origin.
///
/// # Example
/// ```ignore
/// // 2-player horizontal split
/// viewport(0, 0, 480, 540);     // Player 1: left half
/// viewport(480, 0, 480, 540);   // Player 2: right half
///
/// // 4-player quad split
/// viewport(0, 0, 480, 270);     // Player 1: top-left
/// viewport(480, 0, 480, 270);   // Player 2: top-right
/// viewport(0, 270, 480, 270);   // Player 3: bottom-left
/// viewport(480, 270, 480, 270); // Player 4: bottom-right
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default)]
pub struct Viewport {
    /// X coordinate of top-left corner (pixels from left edge)
    pub x: u32,
    /// Y coordinate of top-left corner (pixels from top edge)
    pub y: u32,
    /// Width of viewport in pixels
    pub width: u32,
    /// Height of viewport in pixels
    pub height: u32,
}

impl Viewport {
    /// Full-screen viewport (ZX native resolution)
    pub const FULLSCREEN: Viewport = Viewport {
        x: 0,
        y: 0,
        width: RESOLUTION.0,
        height: RESOLUTION.1,
    };

    /// Calculate aspect ratio (width / height)
    ///
    /// Used by camera functions to create correct perspective projection.
    #[inline]
    pub fn aspect_ratio(&self) -> f32 {
        if self.height == 0 {
            1.0 // Avoid division by zero
        } else {
            self.width as f32 / self.height as f32
        }
    }

    /// Check if viewport is valid (non-zero dimensions)
    #[inline]
    pub fn is_valid(&self) -> bool {
        self.width > 0 && self.height > 0
    }
}
