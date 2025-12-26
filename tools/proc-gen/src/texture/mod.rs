//! Procedural texture generation
//!
//! This module provides tools for generating textures procedurally,
//! including patterns, noise functions, and material presets.
//!
//! # Example
//! ```no_run
//! use proc_gen::texture::*;
//!
//! // Basic patterns
//! let checker = checker(128, 128, 16, [255,255,255,255], [0,0,0,255]);
//!
//! // Noise texture
//! let perlin = PerlinConfig::default()
//!     .generate(256, 256, [50,50,50,255], [200,200,200,255]);
//!
//! // Material preset
//! let metal_tex = metal(256, 256, [180,180,200,255], 42);
//!
//! // Export
//! write_png(&metal_tex, std::path::Path::new("output.png")).unwrap();
//! ```

mod patterns;
mod noise;
mod modifiers;
mod materials;
mod export;

// Core type
pub use self::buffer::TextureBuffer;

// Basic patterns
pub use patterns::{checker, gradient_h, gradient_radial, gradient_v, solid};

// Noise generators
pub use noise::{PerlinConfig, SimplexConfig, VoronoiConfig, VoronoiMode};

// Modifiers
pub use modifiers::{Blend, BlendMode, Contrast, Invert, TextureApply, TextureModifier};

// Material presets
pub use materials::{crystal, metal, stone, MaterialBuilder};

// Export
pub use export::write_png;

mod buffer {
    /// RGBA texture buffer for procedural texture generation
    #[derive(Clone)]
    pub struct TextureBuffer {
        /// Width in pixels
        pub width: u32,
        /// Height in pixels
        pub height: u32,
        /// RGBA pixel data (4 bytes per pixel, row-major order)
        pub pixels: Vec<u8>,
    }

    impl TextureBuffer {
        /// Create a new texture buffer initialized to transparent black
        pub fn new(width: u32, height: u32) -> Self {
            Self {
                width,
                height,
                pixels: vec![0u8; (width * height * 4) as usize],
            }
        }

        /// Create a texture buffer filled with a solid color
        pub fn filled(width: u32, height: u32, color: [u8; 4]) -> Self {
            let mut buffer = Self::new(width, height);
            for chunk in buffer.pixels.chunks_exact_mut(4) {
                chunk.copy_from_slice(&color);
            }
            buffer
        }

        /// Get pixel at (x, y)
        #[inline]
        pub fn get_pixel(&self, x: u32, y: u32) -> [u8; 4] {
            let idx = ((y * self.width + x) * 4) as usize;
            [
                self.pixels[idx],
                self.pixels[idx + 1],
                self.pixels[idx + 2],
                self.pixels[idx + 3],
            ]
        }

        /// Set pixel at (x, y)
        #[inline]
        pub fn set_pixel(&mut self, x: u32, y: u32, color: [u8; 4]) {
            let idx = ((y * self.width + x) * 4) as usize;
            self.pixels[idx] = color[0];
            self.pixels[idx + 1] = color[1];
            self.pixels[idx + 2] = color[2];
            self.pixels[idx + 3] = color[3];
        }

        /// Get mutable slice of pixel data at (x, y)
        #[inline]
        pub fn pixel_mut(&mut self, x: u32, y: u32) -> &mut [u8] {
            let idx = ((y * self.width + x) * 4) as usize;
            &mut self.pixels[idx..idx + 4]
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_texture_buffer_new() {
        let buf = TextureBuffer::new(64, 64);
        assert_eq!(buf.width, 64);
        assert_eq!(buf.height, 64);
        assert_eq!(buf.pixels.len(), 64 * 64 * 4);
        // All pixels should be zero (transparent black)
        assert!(buf.pixels.iter().all(|&p| p == 0));
    }

    #[test]
    fn test_texture_buffer_filled() {
        let color = [255, 128, 64, 255];
        let buf = TextureBuffer::filled(8, 8, color);
        for y in 0..8 {
            for x in 0..8 {
                assert_eq!(buf.get_pixel(x, y), color);
            }
        }
    }

    #[test]
    fn test_texture_buffer_set_get_pixel() {
        let mut buf = TextureBuffer::new(4, 4);
        let color = [100, 150, 200, 255];
        buf.set_pixel(2, 3, color);
        assert_eq!(buf.get_pixel(2, 3), color);
        assert_eq!(buf.get_pixel(0, 0), [0, 0, 0, 0]);
    }
}
