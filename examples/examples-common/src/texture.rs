//! Texture utilities for examples
//!
//! Provides common texture generation functions.

/// Generate an 8x8 checkerboard texture pattern
/// Returns 256 bytes (8x8 RGBA pixels)
///
/// # Arguments
/// * `color_a` - First color (RGBA, 4 bytes)
/// * `color_b` - Second color (RGBA, 4 bytes)
/// * `buffer` - Output buffer (must be at least 256 bytes)
pub fn generate_checkerboard_8x8(color_a: [u8; 4], color_b: [u8; 4], buffer: &mut [u8; 256]) {
    for y in 0..8 {
        for x in 0..8 {
            let idx = (y * 8 + x) * 4;
            let is_a = ((x + y) % 2) == 0;
            let color = if is_a { &color_a } else { &color_b };
            buffer[idx] = color[0];
            buffer[idx + 1] = color[1];
            buffer[idx + 2] = color[2];
            buffer[idx + 3] = color[3];
        }
    }
}
