//! Material preset textures
//!
//! Provides ready-to-use material textures for common surfaces like metal,
//! stone, and crystal. Also includes a builder for creating custom materials.

use super::modifiers::{Blend, BlendMode, Contrast, TextureApply};
use super::noise::{PerlinConfig, VoronoiConfig, VoronoiMode};
use super::TextureBuffer;
use noise::{NoiseFn, Perlin};

/// Generate a brushed metal texture
///
/// Creates a metallic surface with subtle horizontal streaks,
/// simulating brushed or machined metal.
///
/// # Arguments
/// * `width` - Texture width in pixels
/// * `height` - Texture height in pixels
/// * `base_color` - Base metal color (e.g., silver, gold, copper)
/// * `seed` - Random seed for deterministic generation
pub fn metal(width: u32, height: u32, base_color: [u8; 4], seed: u32) -> TextureBuffer {
    let mut buffer = TextureBuffer::filled(width, height, base_color);

    // Generate horizontal streak pattern (stretched X for brushed look)
    let perlin = Perlin::new(seed);
    let streak_buffer = {
        let mut b = TextureBuffer::new(width, height);
        for y in 0..height {
            for x in 0..width {
                // Stretch X to create horizontal streaks
                let value = perlin.get([x as f64 * 0.01, y as f64 * 0.3]);
                let v = ((value + 1.0) / 2.0 * 40.0) as i16; // Subtle variation
                let c = (base_color[0] as i16 + v - 20).clamp(0, 255) as u8;
                b.set_pixel(x, y, [c, c, c, 255]);
            }
        }
        b
    };

    buffer.apply(Blend {
        source: streak_buffer,
        mode: BlendMode::Overlay,
        opacity: 0.3,
    });

    buffer
}

/// Generate a stone/rock texture
///
/// Creates a rocky surface with multi-octave noise for natural variation.
///
/// # Arguments
/// * `width` - Texture width in pixels
/// * `height` - Texture height in pixels
/// * `base_color` - Base stone color
/// * `seed` - Random seed for deterministic generation
pub fn stone(width: u32, height: u32, base_color: [u8; 4], seed: u32) -> TextureBuffer {
    // Create dark and light variants of the base color
    let dark = [
        (base_color[0] as f32 * 0.6) as u8,
        (base_color[1] as f32 * 0.6) as u8,
        (base_color[2] as f32 * 0.6) as u8,
        255,
    ];
    let light = [
        (base_color[0] as f32 * 1.3).min(255.0) as u8,
        (base_color[1] as f32 * 1.3).min(255.0) as u8,
        (base_color[2] as f32 * 1.3).min(255.0) as u8,
        255,
    ];

    // Multi-octave noise for rocky appearance
    let noise_config = PerlinConfig {
        scale: 0.04,
        octaves: 6,
        persistence: 0.6,
        lacunarity: 2.0,
        seed,
    };

    let mut buffer = noise_config.generate(width, height, dark, light);

    // Add some contrast for more definition
    buffer.apply(Contrast { factor: 1.2 });

    buffer
}

/// Generate a crystal/gem texture
///
/// Creates a crystalline surface with Voronoi cells for facets
/// and subtle internal glow/refraction effect.
///
/// # Arguments
/// * `width` - Texture width in pixels
/// * `height` - Texture height in pixels
/// * `base_color` - Base crystal color (inner color)
/// * `edge_color` - Edge/facet color
/// * `seed` - Random seed for deterministic generation
pub fn crystal(
    width: u32,
    height: u32,
    base_color: [u8; 4],
    edge_color: [u8; 4],
    seed: u32,
) -> TextureBuffer {
    // Voronoi for crystalline facets
    let voronoi = VoronoiConfig {
        density: 0.025,
        seed,
        mode: VoronoiMode::Distance,
    };

    let mut buffer = voronoi.generate(width, height, base_color, edge_color);

    // Add subtle internal glow/refraction
    let glow = PerlinConfig {
        scale: 0.08,
        octaves: 2,
        persistence: 0.5,
        lacunarity: 2.0,
        seed: seed.wrapping_add(1),
    };

    let glow_texture = glow.generate(width, height, [0, 0, 0, 255], [255, 255, 255, 255]);

    buffer.apply(Blend {
        source: glow_texture,
        mode: BlendMode::Screen,
        opacity: 0.15,
    });

    buffer
}

/// Builder for custom material textures
///
/// Allows layering multiple noise functions and modifiers
/// to create complex material textures.
///
/// # Example
/// ```
/// use proc_gen::texture::*;
///
/// let custom = MaterialBuilder::new(256, 256, [100, 80, 60, 255])
///     .layer_perlin(
///         PerlinConfig { scale: 0.1, ..Default::default() },
///         [0, 0, 0, 255],
///         [50, 50, 50, 255],
///         BlendMode::Overlay,
///         0.5,
///     )
///     .contrast(1.3)
///     .build();
/// ```
pub struct MaterialBuilder {
    buffer: TextureBuffer,
}

impl MaterialBuilder {
    /// Create a new material builder with a solid base color
    pub fn new(width: u32, height: u32, base_color: [u8; 4]) -> Self {
        Self {
            buffer: TextureBuffer::filled(width, height, base_color),
        }
    }

    /// Layer Perlin noise onto the material
    pub fn layer_perlin(
        mut self,
        config: PerlinConfig,
        low: [u8; 4],
        high: [u8; 4],
        blend: BlendMode,
        opacity: f32,
    ) -> Self {
        let noise = config.generate(self.buffer.width, self.buffer.height, low, high);
        self.buffer.apply(Blend {
            source: noise,
            mode: blend,
            opacity,
        });
        self
    }

    /// Layer Voronoi noise onto the material
    pub fn layer_voronoi(
        mut self,
        config: VoronoiConfig,
        low: [u8; 4],
        high: [u8; 4],
        blend: BlendMode,
        opacity: f32,
    ) -> Self {
        let noise = config.generate(self.buffer.width, self.buffer.height, low, high);
        self.buffer.apply(Blend {
            source: noise,
            mode: blend,
            opacity,
        });
        self
    }

    /// Apply contrast adjustment
    pub fn contrast(mut self, factor: f32) -> Self {
        self.buffer.apply(Contrast { factor });
        self
    }

    /// Build the final texture
    pub fn build(self) -> TextureBuffer {
        self.buffer
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_metal() {
        let tex = metal(64, 64, [180, 180, 200, 255], 42);
        assert_eq!(tex.width, 64);
        assert_eq!(tex.height, 64);

        // Should have some variation (not all same color)
        let first = tex.get_pixel(0, 0);
        let mut has_variation = false;
        for y in 0..64 {
            for x in 0..64 {
                if tex.get_pixel(x, y) != first {
                    has_variation = true;
                    break;
                }
            }
        }
        assert!(has_variation);
    }

    #[test]
    fn test_stone() {
        let tex = stone(64, 64, [120, 100, 80, 255], 123);
        assert_eq!(tex.width, 64);
        assert_eq!(tex.height, 64);
    }

    #[test]
    fn test_crystal() {
        let tex = crystal(64, 64, [100, 150, 255, 255], [50, 80, 200, 255], 777);
        assert_eq!(tex.width, 64);
        assert_eq!(tex.height, 64);
    }

    #[test]
    fn test_material_deterministic() {
        let tex1 = metal(32, 32, [200, 200, 200, 255], 42);
        let tex2 = metal(32, 32, [200, 200, 200, 255], 42);

        // Same parameters should produce identical output
        assert_eq!(tex1.pixels, tex2.pixels);
    }

    #[test]
    fn test_material_builder() {
        let tex = MaterialBuilder::new(64, 64, [100, 80, 60, 255])
            .layer_perlin(
                PerlinConfig::with_seed(42),
                [0, 0, 0, 255],
                [50, 50, 50, 255],
                BlendMode::Overlay,
                0.5,
            )
            .contrast(1.2)
            .build();

        assert_eq!(tex.width, 64);
        assert_eq!(tex.height, 64);
    }

    #[test]
    fn test_material_builder_voronoi() {
        let tex = MaterialBuilder::new(64, 64, [100, 100, 100, 255])
            .layer_voronoi(
                VoronoiConfig::with_seed(42),
                [50, 50, 50, 255],
                [150, 150, 150, 255],
                BlendMode::Normal,
                0.7,
            )
            .build();

        assert_eq!(tex.width, 64);
        assert_eq!(tex.height, 64);
    }
}
