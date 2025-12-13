//! Texture utilities for examples
//!
//! Provides common texture generation functions.

/// Generate an 8x8 checkerboard texture pattern at compile time
///
/// Takes colors in 0xRRGGBBAA format and returns 256 bytes (8x8 RGBA pixels)
/// in proper byte order for GPU consumption.
///
/// # Example
/// ```
/// const CHECKERBOARD: [u8; 256] = checkerboard_8x8(0x00C8C8FF, 0xC800C8FF);
/// ```
pub const fn checkerboard_8x8(color_a: u32, color_b: u32) -> [u8; 256] {
    let mut pixels = [0u8; 256];
    let mut y = 0;
    while y < 8 {
        let mut x = 0;
        while x < 8 {
            let idx = (y * 8 + x) * 4;
            let color = if (x + y) % 2 == 0 { color_a } else { color_b };
            // Extract from 0xRRGGBBAA format, write as RGBA bytes
            pixels[idx] = ((color >> 24) & 0xFF) as u8; // R
            pixels[idx + 1] = ((color >> 16) & 0xFF) as u8; // G
            pixels[idx + 2] = ((color >> 8) & 0xFF) as u8; // B
            pixels[idx + 3] = (color & 0xFF) as u8; // A
            x += 1;
        }
        y += 1;
    }
    pixels
}
