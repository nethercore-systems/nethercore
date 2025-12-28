//! Normal map generation from height/bump data
//!
//! Generates tangent-space normal maps from height maps or directly
//! from procedural noise. Essential for adding depth to low-poly assets.

use super::TextureBuffer;
use super::modifiers::TextureModifier;
use noise::{NoiseFn, Perlin};

/// Generate a normal map from a height map using Sobel operators
///
/// # Arguments
/// * `height` - Grayscale height map (uses R channel)
/// * `strength` - Normal map strength (1.0 = standard, higher = more pronounced)
///
/// # Returns
/// RGB normal map in tangent space (B channel = up)
pub fn normal_from_height(height: &TextureBuffer, strength: f32) -> TextureBuffer {
    let mut normal = TextureBuffer::new(height.width, height.height);

    for y in 0..height.height {
        for x in 0..height.width {
            // Sample heights with wrapping for tileability
            let get_h = |dx: i32, dy: i32| {
                let px = ((x as i32 + dx).rem_euclid(height.width as i32)) as u32;
                let py = ((y as i32 + dy).rem_euclid(height.height as i32)) as u32;
                height.get_pixel(px, py)[0] as f32 / 255.0
            };

            // Sobel operator for gradient
            let gx = (get_h(1, -1) - get_h(-1, -1))
                + 2.0 * (get_h(1, 0) - get_h(-1, 0))
                + (get_h(1, 1) - get_h(-1, 1));

            let gy = (get_h(-1, 1) - get_h(-1, -1))
                + 2.0 * (get_h(0, 1) - get_h(0, -1))
                + (get_h(1, 1) - get_h(1, -1));

            // Scale by strength
            let gx = gx * strength;
            let gy = gy * strength;

            // Calculate normal vector (pointing up in tangent space)
            let len = (gx * gx + gy * gy + 1.0).sqrt();
            let nx = -gx / len;
            let ny = -gy / len;
            let nz = 1.0 / len;

            // Convert from [-1, 1] to [0, 255]
            let r = ((nx * 0.5 + 0.5) * 255.0) as u8;
            let g = ((ny * 0.5 + 0.5) * 255.0) as u8;
            let b = ((nz * 0.5 + 0.5) * 255.0) as u8;

            normal.set_pixel(x, y, [r, g, b, 255]);
        }
    }

    normal
}

/// Generate a procedural normal map using noise
pub struct ProceduralNormalMap {
    /// Noise scale (larger = more detail)
    pub scale: f64,
    /// Normal strength
    pub strength: f32,
    /// Number of octaves for detail
    pub octaves: u32,
    /// Random seed
    pub seed: u32,
}

impl Default for ProceduralNormalMap {
    fn default() -> Self {
        Self {
            scale: 0.05,
            strength: 1.0,
            octaves: 4,
            seed: 0,
        }
    }
}

impl ProceduralNormalMap {
    /// Generate a normal map
    pub fn generate(&self, width: u32, height: u32) -> TextureBuffer {
        // First generate a height map
        let height_map = self.generate_height(width, height);
        // Then convert to normals
        normal_from_height(&height_map, self.strength)
    }

    /// Generate just the height map
    pub fn generate_height(&self, width: u32, height: u32) -> TextureBuffer {
        let perlin = Perlin::new(self.seed);
        let mut buffer = TextureBuffer::new(width, height);

        for y in 0..height {
            for x in 0..width {
                let h = self.sample_fbm(&perlin, x as f64, y as f64);
                let value = ((h + 1.0) / 2.0 * 255.0).clamp(0.0, 255.0) as u8;
                buffer.set_pixel(x, y, [value, value, value, 255]);
            }
        }

        buffer
    }

    fn sample_fbm(&self, noise: &Perlin, x: f64, y: f64) -> f64 {
        let mut total = 0.0;
        let mut amplitude = 1.0;
        let mut frequency = self.scale;
        let mut max_value = 0.0;

        for _ in 0..self.octaves {
            total += noise.get([x * frequency, y * frequency]) * amplitude;
            max_value += amplitude;
            amplitude *= 0.5;
            frequency *= 2.0;
        }

        total / max_value
    }
}

/// Generate a detail normal map for fine surface detail
pub struct DetailNormalMap {
    /// Fine detail scale
    pub detail_scale: f64,
    /// Medium detail scale
    pub medium_scale: f64,
    /// Detail strength
    pub detail_strength: f32,
    /// Medium strength
    pub medium_strength: f32,
    /// Random seed
    pub seed: u32,
}

impl Default for DetailNormalMap {
    fn default() -> Self {
        Self {
            detail_scale: 0.2,
            medium_scale: 0.05,
            detail_strength: 0.5,
            medium_strength: 1.0,
            seed: 0,
        }
    }
}

impl DetailNormalMap {
    /// Generate a multi-frequency detail normal map
    pub fn generate(&self, width: u32, height: u32) -> TextureBuffer {
        let perlin = Perlin::new(self.seed);
        let mut buffer = TextureBuffer::new(width, height);

        for y in 0..height {
            for x in 0..width {
                let nx = x as f64;
                let ny = y as f64;

                // Sample at two frequencies
                let medium_gx = (perlin.get([(nx + 1.0) * self.medium_scale, ny * self.medium_scale])
                    - perlin.get([(nx - 1.0) * self.medium_scale, ny * self.medium_scale])) as f32;
                let medium_gy = (perlin.get([nx * self.medium_scale, (ny + 1.0) * self.medium_scale])
                    - perlin.get([nx * self.medium_scale, (ny - 1.0) * self.medium_scale])) as f32;

                let detail_gx = (perlin.get([(nx + 1.0) * self.detail_scale, ny * self.detail_scale])
                    - perlin.get([(nx - 1.0) * self.detail_scale, ny * self.detail_scale])) as f32;
                let detail_gy = (perlin.get([nx * self.detail_scale, (ny + 1.0) * self.detail_scale])
                    - perlin.get([nx * self.detail_scale, (ny - 1.0) * self.detail_scale])) as f32;

                // Combine gradients
                let gx = medium_gx * self.medium_strength + detail_gx * self.detail_strength;
                let gy = medium_gy * self.medium_strength + detail_gy * self.detail_strength;

                // Calculate normal
                let len = (gx * gx + gy * gy + 1.0).sqrt();
                let nx = -gx / len;
                let ny = -gy / len;
                let nz = 1.0 / len;

                let r = ((nx * 0.5 + 0.5) * 255.0) as u8;
                let g = ((ny * 0.5 + 0.5) * 255.0) as u8;
                let b = ((nz * 0.5 + 0.5) * 255.0) as u8;

                buffer.set_pixel(x, y, [r, g, b, 255]);
            }
        }

        buffer
    }
}

/// Blend two normal maps together
pub fn blend_normals(base: &TextureBuffer, detail: &TextureBuffer, strength: f32) -> TextureBuffer {
    let mut result = TextureBuffer::new(base.width, base.height);

    for y in 0..base.height.min(detail.height) {
        for x in 0..base.width.min(detail.width) {
            let b = base.get_pixel(x, y);
            let d = detail.get_pixel(x, y);

            // Convert from [0,255] to [-1,1]
            let bn = [
                (b[0] as f32 / 255.0) * 2.0 - 1.0,
                (b[1] as f32 / 255.0) * 2.0 - 1.0,
                (b[2] as f32 / 255.0) * 2.0 - 1.0,
            ];
            let dn = [
                (d[0] as f32 / 255.0) * 2.0 - 1.0,
                (d[1] as f32 / 255.0) * 2.0 - 1.0,
                (d[2] as f32 / 255.0) * 2.0 - 1.0,
            ];

            // UDN blending (Unreal Development Network method)
            // This properly combines normals in tangent space
            let combined = [
                bn[0] + dn[0] * strength,
                bn[1] + dn[1] * strength,
                bn[2] * dn[2],
            ];

            // Normalize
            let len = (combined[0] * combined[0] + combined[1] * combined[1] + combined[2] * combined[2]).sqrt();
            let normalized = [
                combined[0] / len,
                combined[1] / len,
                combined[2] / len,
            ];

            // Convert back to [0,255]
            let r = ((normalized[0] * 0.5 + 0.5) * 255.0).clamp(0.0, 255.0) as u8;
            let g = ((normalized[1] * 0.5 + 0.5) * 255.0).clamp(0.0, 255.0) as u8;
            let b_val = ((normalized[2] * 0.5 + 0.5) * 255.0).clamp(0.0, 255.0) as u8;

            result.set_pixel(x, y, [r, g, b_val, 255]);
        }
    }

    result
}

/// Generate a flat (neutral) normal map
pub fn flat_normal(width: u32, height: u32) -> TextureBuffer {
    // Neutral normal: pointing straight up (0, 0, 1) encoded as (128, 128, 255)
    TextureBuffer::filled(width, height, [128, 128, 255, 255])
}

/// Modifier to add normal map detail from noise
pub struct AddNormalDetail {
    /// Detail strength
    pub strength: f32,
    /// Noise scale
    pub scale: f64,
    /// Random seed
    pub seed: u32,
}

impl Default for AddNormalDetail {
    fn default() -> Self {
        Self {
            strength: 0.3,
            scale: 0.1,
            seed: 0,
        }
    }
}

impl TextureModifier for AddNormalDetail {
    fn apply(&self, buffer: &mut TextureBuffer) {
        let detail = DetailNormalMap {
            detail_scale: self.scale,
            medium_scale: self.scale * 0.25,
            detail_strength: self.strength,
            medium_strength: self.strength * 0.5,
            seed: self.seed,
        };
        let detail_map = detail.generate(buffer.width, buffer.height);

        // Blend with existing normal map
        let blended = blend_normals(buffer, &detail_map, 1.0);

        // Copy result back
        buffer.pixels = blended.pixels;
    }
}

/// Complete texture output with all maps
pub struct TextureOutput {
    /// Albedo/diffuse map
    pub albedo: TextureBuffer,
    /// Normal map (optional)
    pub normal: Option<TextureBuffer>,
    /// Height map (optional)
    pub height: Option<TextureBuffer>,
    /// Roughness map (optional, for MRE)
    pub roughness: Option<TextureBuffer>,
    /// Metallic map (optional, for MRE)
    pub metallic: Option<TextureBuffer>,
    /// Ambient occlusion map (optional)
    pub ao: Option<TextureBuffer>,
}

impl TextureOutput {
    /// Create output with just albedo
    pub fn albedo_only(albedo: TextureBuffer) -> Self {
        Self {
            albedo,
            normal: None,
            height: None,
            roughness: None,
            metallic: None,
            ao: None,
        }
    }

    /// Create output with albedo and generated normal
    pub fn with_generated_normal(albedo: TextureBuffer, height: TextureBuffer, normal_strength: f32) -> Self {
        let normal = normal_from_height(&height, normal_strength);
        Self {
            albedo,
            normal: Some(normal),
            height: Some(height),
            roughness: None,
            metallic: None,
            ao: None,
        }
    }

    /// Generate a roughness map from the albedo (darker = rougher)
    pub fn generate_roughness_from_albedo(&mut self, invert: bool, base_roughness: f32) {
        let mut roughness = TextureBuffer::new(self.albedo.width, self.albedo.height);

        for y in 0..self.albedo.height {
            for x in 0..self.albedo.width {
                let p = self.albedo.get_pixel(x, y);
                // Luminance
                let lum = (p[0] as f32 * 0.299 + p[1] as f32 * 0.587 + p[2] as f32 * 0.114) / 255.0;

                let rough = if invert {
                    base_roughness + (1.0 - lum) * (1.0 - base_roughness)
                } else {
                    base_roughness + lum * (1.0 - base_roughness)
                };

                let value = (rough.clamp(0.0, 1.0) * 255.0) as u8;
                roughness.set_pixel(x, y, [value, value, value, 255]);
            }
        }

        self.roughness = Some(roughness);
    }

    /// Pack into MRE (Metallic-Roughness-Emissive) texture for ZX render mode 2
    pub fn pack_mre(&self, metallic_value: f32, emissive_value: f32) -> TextureBuffer {
        let mut mre = TextureBuffer::new(self.albedo.width, self.albedo.height);

        let metallic_byte = (metallic_value.clamp(0.0, 1.0) * 255.0) as u8;
        let emissive_byte = (emissive_value.clamp(0.0, 1.0) * 255.0) as u8;

        for y in 0..self.albedo.height {
            for x in 0..self.albedo.width {
                let roughness = self.roughness.as_ref()
                    .map(|r| r.get_pixel(x, y)[0])
                    .unwrap_or(128);

                mre.set_pixel(x, y, [metallic_byte, roughness, emissive_byte, 255]);
            }
        }

        mre
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_flat_normal() {
        let normal = flat_normal(16, 16);
        let center = normal.get_pixel(8, 8);
        // Should be neutral blue (128, 128, 255)
        assert_eq!(center[0], 128);
        assert_eq!(center[1], 128);
        assert_eq!(center[2], 255);
    }

    #[test]
    fn test_normal_from_flat_height() {
        // Flat height map should produce flat normals
        let height = TextureBuffer::filled(32, 32, [128, 128, 128, 255]);
        let normal = normal_from_height(&height, 1.0);

        let center = normal.get_pixel(16, 16);
        // Should be close to neutral (128, 128, ~255)
        assert!((center[0] as i32 - 128).abs() <= 1);
        assert!((center[1] as i32 - 128).abs() <= 1);
        assert!(center[2] > 250);
    }

    #[test]
    fn test_procedural_normal_map() {
        let generator = ProceduralNormalMap {
            scale: 0.1,
            strength: 1.0,
            octaves: 2,
            seed: 42,
        };
        let normal = generator.generate(64, 64);

        assert_eq!(normal.width, 64);
        assert_eq!(normal.height, 64);

        // Should have some variation
        let mut has_variation = false;
        let first = normal.get_pixel(0, 0);
        for y in 0..64 {
            for x in 0..64 {
                if normal.get_pixel(x, y) != first {
                    has_variation = true;
                    break;
                }
            }
        }
        assert!(has_variation);
    }

    #[test]
    fn test_blend_normals() {
        let base = flat_normal(32, 32);
        let detail = ProceduralNormalMap::default().generate(32, 32);

        let blended = blend_normals(&base, &detail, 0.5);

        assert_eq!(blended.width, 32);
        assert_eq!(blended.height, 32);
    }

    #[test]
    fn test_texture_output_mre_pack() {
        let albedo = TextureBuffer::filled(16, 16, [200, 150, 100, 255]);
        let mut output = TextureOutput::albedo_only(albedo);
        output.generate_roughness_from_albedo(true, 0.3);

        let mre = output.pack_mre(0.0, 0.0);
        let p = mre.get_pixel(8, 8);

        // Metallic should be 0
        assert_eq!(p[0], 0);
        // Roughness should be somewhere between 0 and 255
        assert!(p[1] > 0 && p[1] < 255);
        // Emissive should be 0
        assert_eq!(p[2], 0);
    }
}
