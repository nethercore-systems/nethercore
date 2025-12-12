//! Texture utilities for examples
//!
//! Provides common texture generation functions.

/// Generate an 8x8 checkerboard texture pattern at compile time
///
/// Returns 64 pixels as u32 values (0xRRGGBBAA format). Can be used in const context.
///
/// # Example
/// ```
/// const CHECKERBOARD: [u32; 64] = checkerboard_8x8(0x00C8C8FF, 0xC800C8FF);
/// ```
pub const fn checkerboard_8x8(color_a: u32, color_b: u32) -> [u32; 64] {
    let mut pixels = [0u32; 64];
    let mut y = 0;
    while y < 8 {
        let mut x = 0;
        while x < 8 {
            let idx = y * 8 + x;
            pixels[idx] = if (x + y) % 2 == 0 { color_a } else { color_b };
            x += 1;
        }
        y += 1;
    }
    pixels
}
