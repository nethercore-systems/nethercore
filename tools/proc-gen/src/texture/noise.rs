//! Noise-based texture generation using the `noise` crate
//!
//! Provides Perlin, Simplex, and Voronoi (cellular) noise generators
//! with fractal Brownian motion (fBm) support for multi-octave noise.

use super::patterns::lerp_color;
use super::TextureBuffer;
use noise::{NoiseFn, Perlin, Simplex, Worley};

/// Configuration for Perlin noise generation
#[derive(Clone)]
pub struct PerlinConfig {
    /// Scale of the noise (larger = more zoomed out)
    pub scale: f64,
    /// Number of octaves for fractal noise
    pub octaves: u32,
    /// Persistence (amplitude multiplier per octave)
    pub persistence: f64,
    /// Lacunarity (frequency multiplier per octave)
    pub lacunarity: f64,
    /// Random seed
    pub seed: u32,
}

impl Default for PerlinConfig {
    fn default() -> Self {
        Self {
            scale: 0.05,
            octaves: 4,
            persistence: 0.5,
            lacunarity: 2.0,
            seed: 0,
        }
    }
}

impl PerlinConfig {
    /// Create a new Perlin config with the given seed
    pub fn with_seed(seed: u32) -> Self {
        Self {
            seed,
            ..Default::default()
        }
    }

    /// Generate a texture using Perlin noise
    pub fn generate(&self, width: u32, height: u32, low: [u8; 4], high: [u8; 4]) -> TextureBuffer {
        let perlin = Perlin::new(self.seed);
        let mut buffer = TextureBuffer::new(width, height);

        for y in 0..height {
            for x in 0..width {
                let value = self.sample_fbm(&perlin, x as f64, y as f64);
                let t = ((value + 1.0) / 2.0).clamp(0.0, 1.0) as f32;
                buffer.set_pixel(x, y, lerp_color(low, high, t));
            }
        }
        buffer
    }

    /// Sample fractal Brownian motion (multi-octave noise)
    fn sample_fbm<N: NoiseFn<f64, 2>>(&self, noise: &N, x: f64, y: f64) -> f64 {
        let mut total = 0.0;
        let mut amplitude = 1.0;
        let mut frequency = self.scale;
        let mut max_value = 0.0;

        for _ in 0..self.octaves {
            total += noise.get([x * frequency, y * frequency]) * amplitude;
            max_value += amplitude;
            amplitude *= self.persistence;
            frequency *= self.lacunarity;
        }

        total / max_value
    }
}

/// Configuration for Simplex noise (faster alternative to Perlin)
#[derive(Clone)]
pub struct SimplexConfig {
    /// Scale of the noise (larger = more zoomed out)
    pub scale: f64,
    /// Number of octaves for fractal noise
    pub octaves: u32,
    /// Persistence (amplitude multiplier per octave)
    pub persistence: f64,
    /// Lacunarity (frequency multiplier per octave)
    pub lacunarity: f64,
    /// Random seed
    pub seed: u32,
}

impl Default for SimplexConfig {
    fn default() -> Self {
        Self {
            scale: 0.05,
            octaves: 4,
            persistence: 0.5,
            lacunarity: 2.0,
            seed: 0,
        }
    }
}

impl SimplexConfig {
    /// Create a new Simplex config with the given seed
    pub fn with_seed(seed: u32) -> Self {
        Self {
            seed,
            ..Default::default()
        }
    }

    /// Generate a texture using Simplex noise
    pub fn generate(&self, width: u32, height: u32, low: [u8; 4], high: [u8; 4]) -> TextureBuffer {
        let simplex = Simplex::new(self.seed);
        let mut buffer = TextureBuffer::new(width, height);

        for y in 0..height {
            for x in 0..width {
                let value = self.sample_fbm(&simplex, x as f64, y as f64);
                let t = ((value + 1.0) / 2.0).clamp(0.0, 1.0) as f32;
                buffer.set_pixel(x, y, lerp_color(low, high, t));
            }
        }
        buffer
    }

    /// Sample fractal Brownian motion (multi-octave noise)
    fn sample_fbm<N: NoiseFn<f64, 2>>(&self, noise: &N, x: f64, y: f64) -> f64 {
        let mut total = 0.0;
        let mut amplitude = 1.0;
        let mut frequency = self.scale;
        let mut max_value = 0.0;

        for _ in 0..self.octaves {
            total += noise.get([x * frequency, y * frequency]) * amplitude;
            max_value += amplitude;
            amplitude *= self.persistence;
            frequency *= self.lacunarity;
        }

        total / max_value
    }
}

/// Voronoi pattern mode
#[derive(Clone, Copy, Default)]
pub enum VoronoiMode {
    /// Distance to nearest cell center (standard cellular look)
    #[default]
    Distance,
    /// Second nearest minus nearest (highlights cell edges)
    Edge,
}

/// Configuration for Voronoi (cellular) noise
#[derive(Clone)]
pub struct VoronoiConfig {
    /// Cell density (higher = more cells)
    pub density: f64,
    /// Random seed
    pub seed: u32,
    /// Voronoi mode
    pub mode: VoronoiMode,
}

impl Default for VoronoiConfig {
    fn default() -> Self {
        Self {
            density: 0.03,
            seed: 0,
            mode: VoronoiMode::Distance,
        }
    }
}

impl VoronoiConfig {
    /// Create a new Voronoi config with the given seed
    pub fn with_seed(seed: u32) -> Self {
        Self {
            seed,
            ..Default::default()
        }
    }

    /// Generate a texture using Voronoi/cellular noise
    pub fn generate(&self, width: u32, height: u32, low: [u8; 4], high: [u8; 4]) -> TextureBuffer {
        let worley = Worley::new(self.seed);
        let mut buffer = TextureBuffer::new(width, height);

        for y in 0..height {
            for x in 0..width {
                let nx = x as f64 * self.density;
                let ny = y as f64 * self.density;

                let value = match self.mode {
                    VoronoiMode::Distance => worley.get([nx, ny]),
                    VoronoiMode::Edge => {
                        // Edge mode uses the difference between distances
                        // Worley returns distance to nearest by default
                        let v = worley.get([nx, ny]);
                        // Transform to highlight edges (values closer to 1 are edges)
                        1.0 - v.abs()
                    }
                };

                let t = ((value + 1.0) / 2.0).clamp(0.0, 1.0) as f32;
                buffer.set_pixel(x, y, lerp_color(low, high, t));
            }
        }
        buffer
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_perlin_default() {
        let config = PerlinConfig::default();
        assert_eq!(config.octaves, 4);
        assert_eq!(config.seed, 0);
    }

    #[test]
    fn test_perlin_generate() {
        let config = PerlinConfig::with_seed(42);
        let low = [0, 0, 0, 255];
        let high = [255, 255, 255, 255];
        let tex = config.generate(64, 64, low, high);

        assert_eq!(tex.width, 64);
        assert_eq!(tex.height, 64);

        // Check that we have some variation (not all pixels the same)
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
        assert!(has_variation, "Perlin noise should have variation");
    }

    #[test]
    fn test_perlin_deterministic() {
        let config = PerlinConfig::with_seed(123);
        let low = [0, 0, 0, 255];
        let high = [255, 255, 255, 255];

        let tex1 = config.generate(32, 32, low, high);
        let tex2 = config.generate(32, 32, low, high);

        // Same seed should produce same output
        assert_eq!(tex1.pixels, tex2.pixels);
    }

    #[test]
    fn test_simplex_generate() {
        let config = SimplexConfig::with_seed(42);
        let low = [50, 50, 50, 255];
        let high = [200, 200, 200, 255];
        let tex = config.generate(64, 64, low, high);

        assert_eq!(tex.width, 64);
        assert_eq!(tex.height, 64);

        // All pixels should be within the color range
        for y in 0..64 {
            for x in 0..64 {
                let p = tex.get_pixel(x, y);
                assert!(p[0] >= 50 && p[0] <= 200);
            }
        }
    }

    #[test]
    fn test_voronoi_generate() {
        let config = VoronoiConfig::with_seed(42);
        let low = [0, 0, 0, 255];
        let high = [255, 255, 255, 255];
        let tex = config.generate(64, 64, low, high);

        assert_eq!(tex.width, 64);
        assert_eq!(tex.height, 64);
    }

    #[test]
    fn test_voronoi_edge_mode() {
        let config = VoronoiConfig {
            mode: VoronoiMode::Edge,
            ..Default::default()
        };
        let low = [0, 0, 0, 255];
        let high = [255, 255, 255, 255];
        let tex = config.generate(64, 64, low, high);

        assert_eq!(tex.width, 64);
        assert_eq!(tex.height, 64);
    }
}
