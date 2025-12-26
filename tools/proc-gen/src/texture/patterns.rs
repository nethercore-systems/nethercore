//! Basic texture patterns
//!
//! This module provides simple texture generation functions for common patterns
//! like solid colors, checkerboards, and gradients.

use super::TextureBuffer;

/// Generate a solid color texture
pub fn solid(width: u32, height: u32, color: [u8; 4]) -> TextureBuffer {
    TextureBuffer::filled(width, height, color)
}

/// Generate a checkerboard pattern texture
pub fn checker(
    width: u32,
    height: u32,
    tile_size: u32,
    color_a: [u8; 4],
    color_b: [u8; 4],
) -> TextureBuffer {
    let mut buffer = TextureBuffer::new(width, height);

    for y in 0..height {
        for x in 0..width {
            let checker = ((x / tile_size) + (y / tile_size)) % 2 == 0;
            let color = if checker { color_a } else { color_b };
            buffer.set_pixel(x, y, color);
        }
    }
    buffer
}

/// Generate a vertical gradient (top to bottom)
pub fn gradient_v(
    width: u32,
    height: u32,
    color_top: [u8; 4],
    color_bottom: [u8; 4],
) -> TextureBuffer {
    let mut buffer = TextureBuffer::new(width, height);
    let max_y = (height - 1).max(1) as f32;

    for y in 0..height {
        let t = y as f32 / max_y;
        let color = lerp_color(color_top, color_bottom, t);
        for x in 0..width {
            buffer.set_pixel(x, y, color);
        }
    }
    buffer
}

/// Generate a horizontal gradient (left to right)
pub fn gradient_h(
    width: u32,
    height: u32,
    color_left: [u8; 4],
    color_right: [u8; 4],
) -> TextureBuffer {
    let mut buffer = TextureBuffer::new(width, height);
    let max_x = (width - 1).max(1) as f32;

    for x in 0..width {
        let t = x as f32 / max_x;
        let color = lerp_color(color_left, color_right, t);
        for y in 0..height {
            buffer.set_pixel(x, y, color);
        }
    }
    buffer
}

/// Generate a radial gradient (center outward)
pub fn gradient_radial(
    width: u32,
    height: u32,
    color_center: [u8; 4],
    color_edge: [u8; 4],
) -> TextureBuffer {
    let mut buffer = TextureBuffer::new(width, height);
    let cx = width as f32 / 2.0;
    let cy = height as f32 / 2.0;
    let max_dist = (cx * cx + cy * cy).sqrt();

    for y in 0..height {
        for x in 0..width {
            let dx = x as f32 - cx;
            let dy = y as f32 - cy;
            let dist = (dx * dx + dy * dy).sqrt();
            let t = (dist / max_dist).clamp(0.0, 1.0);
            buffer.set_pixel(x, y, lerp_color(color_center, color_edge, t));
        }
    }
    buffer
}

/// Linear interpolation between two colors
pub(crate) fn lerp_color(a: [u8; 4], b: [u8; 4], t: f32) -> [u8; 4] {
    [
        lerp_u8(a[0], b[0], t),
        lerp_u8(a[1], b[1], t),
        lerp_u8(a[2], b[2], t),
        lerp_u8(a[3], b[3], t),
    ]
}

/// Linear interpolation for u8 values
pub(crate) fn lerp_u8(a: u8, b: u8, t: f32) -> u8 {
    (a as f32 + (b as f32 - a as f32) * t).clamp(0.0, 255.0) as u8
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_solid() {
        let color = [100, 150, 200, 255];
        let tex = solid(16, 16, color);
        assert_eq!(tex.width, 16);
        assert_eq!(tex.height, 16);
        for y in 0..16 {
            for x in 0..16 {
                assert_eq!(tex.get_pixel(x, y), color);
            }
        }
    }

    #[test]
    fn test_checker() {
        let white = [255, 255, 255, 255];
        let black = [0, 0, 0, 255];
        let tex = checker(8, 8, 4, white, black);

        // Top-left 4x4 should be white
        assert_eq!(tex.get_pixel(0, 0), white);
        assert_eq!(tex.get_pixel(3, 3), white);

        // Top-right 4x4 should be black
        assert_eq!(tex.get_pixel(4, 0), black);
        assert_eq!(tex.get_pixel(7, 3), black);

        // Bottom-left 4x4 should be black
        assert_eq!(tex.get_pixel(0, 4), black);

        // Bottom-right 4x4 should be white
        assert_eq!(tex.get_pixel(4, 4), white);
    }

    #[test]
    fn test_gradient_v() {
        let top = [255, 0, 0, 255];
        let bottom = [0, 0, 255, 255];
        let tex = gradient_v(8, 8, top, bottom);

        // Top row should be red
        assert_eq!(tex.get_pixel(0, 0), top);
        // Bottom row should be blue
        assert_eq!(tex.get_pixel(0, 7), bottom);
        // Middle should be interpolated
        let mid = tex.get_pixel(0, 4);
        assert!(mid[0] > 0 && mid[0] < 255);
        assert!(mid[2] > 0 && mid[2] < 255);
    }

    #[test]
    fn test_gradient_h() {
        let left = [255, 0, 0, 255];
        let right = [0, 255, 0, 255];
        let tex = gradient_h(8, 8, left, right);

        assert_eq!(tex.get_pixel(0, 0), left);
        assert_eq!(tex.get_pixel(7, 0), right);
    }

    #[test]
    fn test_gradient_radial() {
        let center = [255, 255, 255, 255];
        let edge = [0, 0, 0, 255];
        let tex = gradient_radial(16, 16, center, edge);

        // Center should be close to white
        let c = tex.get_pixel(8, 8);
        assert!(c[0] > 200);

        // Corners should be close to black
        let corner = tex.get_pixel(0, 0);
        assert!(corner[0] < 50);
    }

    #[test]
    fn test_lerp_u8() {
        assert_eq!(lerp_u8(0, 100, 0.0), 0);
        assert_eq!(lerp_u8(0, 100, 1.0), 100);
        assert_eq!(lerp_u8(0, 100, 0.5), 50);
        assert_eq!(lerp_u8(100, 200, 0.5), 150);
    }
}
