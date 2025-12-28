//! Texture feature generators
//!
//! Generates actual visual features (scratches, cracks, grain, rivets, etc.)
//! rather than just noise. These create the details that distinguish
//! professional art from programmer art.

use super::TextureBuffer;
use super::modifiers::{TextureModifier, TextureApply, BlendMode, Blend};
use noise::{NoiseFn, Perlin};

/// Direction pattern for linear features like scratches
#[derive(Clone, Copy, Default)]
pub enum ScratchDirection {
    /// Random direction per scratch
    #[default]
    Random,
    /// Horizontal scratches
    Horizontal,
    /// Vertical scratches
    Vertical,
    /// Diagonal (45 degrees)
    Diagonal,
    /// Custom angle in degrees
    Angle(f32),
}

/// Generate scratch marks on a texture
#[derive(Clone)]
pub struct Scratches {
    /// Scratch density (0.0 to 1.0)
    pub density: f32,
    /// Scratch length range (in texture percentage)
    pub length: (f32, f32),
    /// Scratch width in pixels
    pub width: f32,
    /// Scratch depth/intensity (0.0 to 1.0)
    pub depth: f32,
    /// Direction pattern
    pub direction: ScratchDirection,
    /// Random seed
    pub seed: u32,
    /// Color of scratches (usually darker or lighter than base)
    pub color: [u8; 4],
}

impl Default for Scratches {
    fn default() -> Self {
        Self {
            density: 0.1,
            length: (0.1, 0.3),
            width: 1.0,
            depth: 0.5,
            direction: ScratchDirection::Random,
            seed: 0,
            color: [180, 180, 180, 255],
        }
    }
}

impl Scratches {
    /// Create light surface scratches
    pub fn light() -> Self {
        Self {
            density: 0.05,
            length: (0.05, 0.15),
            width: 1.0,
            depth: 0.3,
            color: [200, 200, 200, 255],
            ..Default::default()
        }
    }

    /// Create heavy wear scratches
    pub fn heavy() -> Self {
        Self {
            density: 0.2,
            length: (0.1, 0.4),
            width: 2.0,
            depth: 0.7,
            color: [160, 160, 160, 255],
            ..Default::default()
        }
    }
}

impl TextureModifier for Scratches {
    fn apply(&self, buffer: &mut TextureBuffer) {
        use std::f32::consts::PI;

        let mut scratch_buffer = TextureBuffer::new(buffer.width, buffer.height);
        let perlin = Perlin::new(self.seed);

        // Number of scratches based on density and texture size
        let area = buffer.width * buffer.height;
        let num_scratches = ((area as f32 * self.density * 0.01) as u32).max(1);

        for i in 0..num_scratches {
            // Pseudo-random position using noise
            let nx = (i as f64 * 0.1) + self.seed as f64;
            let ny = (i as f64 * 0.2) + self.seed as f64;

            let start_x = ((perlin.get([nx, 0.0]) + 1.0) / 2.0 * buffer.width as f64) as i32;
            let start_y = ((perlin.get([0.0, ny]) + 1.0) / 2.0 * buffer.height as f64) as i32;

            // Length
            let len_t = ((perlin.get([nx + 10.0, ny + 10.0]) + 1.0) / 2.0) as f32;
            let length = self.length.0 + (self.length.1 - self.length.0) * len_t;
            let length_px = (length * buffer.width.min(buffer.height) as f32) as i32;

            // Direction
            let angle = match self.direction {
                ScratchDirection::Random => {
                    perlin.get([nx + 20.0, ny + 20.0]) as f32 * PI
                }
                ScratchDirection::Horizontal => 0.0,
                ScratchDirection::Vertical => PI / 2.0,
                ScratchDirection::Diagonal => PI / 4.0,
                ScratchDirection::Angle(deg) => deg.to_radians(),
            };

            let dx = angle.cos();
            let dy = angle.sin();

            // Draw scratch line
            for step in 0..length_px {
                let x = start_x + (step as f32 * dx) as i32;
                let y = start_y + (step as f32 * dy) as i32;

                // Draw with width
                let half_width = (self.width / 2.0).ceil() as i32;
                for wy in -half_width..=half_width {
                    for wx in -half_width..=half_width {
                        let px = x + wx;
                        let py = y + wy;

                        if px >= 0 && px < buffer.width as i32 && py >= 0 && py < buffer.height as i32 {
                            // Distance from center line affects intensity
                            let dist = ((wx * wx + wy * wy) as f32).sqrt() / self.width;
                            if dist <= 1.0 {
                                let intensity = 1.0 - dist;
                                let alpha = (self.color[3] as f32 * intensity * self.depth) as u8;
                                let current = scratch_buffer.get_pixel(px as u32, py as u32);
                                let new_alpha = current[3].max(alpha);
                                scratch_buffer.set_pixel(
                                    px as u32, py as u32,
                                    [self.color[0], self.color[1], self.color[2], new_alpha]
                                );
                            }
                        }
                    }
                }
            }
        }

        // Blend scratch buffer onto main buffer
        buffer.apply(Blend {
            source: scratch_buffer,
            mode: BlendMode::Normal,
            opacity: self.depth,
        });
    }
}

/// Crack pattern type
#[derive(Clone, Copy, Default)]
pub enum CrackPattern {
    /// Organic cracks (like dried mud)
    #[default]
    Organic,
    /// Radial cracks from impact point
    Radial,
    /// Branching tree-like cracks
    Branching,
    /// Grid-like cracks (like old paint)
    Grid,
}

/// Generate crack patterns on a texture
#[derive(Clone)]
pub struct Cracks {
    /// Crack density (0.0 to 1.0)
    pub density: f32,
    /// Crack pattern type
    pub pattern: CrackPattern,
    /// Crack width in pixels
    pub width: f32,
    /// Crack depth/darkness (0.0 to 1.0)
    pub depth: f32,
    /// Branching factor (for Branching pattern)
    pub branching: f32,
    /// Random seed
    pub seed: u32,
    /// Crack color
    pub color: [u8; 4],
}

impl Default for Cracks {
    fn default() -> Self {
        Self {
            density: 0.1,
            pattern: CrackPattern::Organic,
            width: 1.5,
            depth: 0.6,
            branching: 0.3,
            seed: 0,
            color: [40, 35, 30, 255],
        }
    }
}

impl TextureModifier for Cracks {
    fn apply(&self, buffer: &mut TextureBuffer) {
        let perlin = Perlin::new(self.seed);
        let scale = (0.02 + self.density * 0.05) as f64;

        for y in 0..buffer.height {
            for x in 0..buffer.width {
                let nx = x as f64 * scale;
                let ny = y as f64 * scale;

                // Use Voronoi-like distance field for cracks
                let v1 = perlin.get([nx, ny]);
                let v2 = perlin.get([nx + 0.5, ny + 0.5]);

                // Crack detection based on gradient
                let edge = ((v1 - v2).abs() as f32 * 2.0).min(1.0);

                // Threshold to create sharp cracks
                let threshold = 1.0 - self.density * 0.5;
                if edge > threshold {
                    let intensity = ((edge - threshold) / (1.0 - threshold)).min(1.0);

                    // Distance-based width
                    let width_factor = if intensity > 0.8 { 1.0 } else { intensity };

                    if width_factor > 0.0 {
                        let pixel = buffer.get_pixel(x, y);
                        let blend_factor = intensity * self.depth;

                        let r = (pixel[0] as f32 * (1.0 - blend_factor) + self.color[0] as f32 * blend_factor) as u8;
                        let g = (pixel[1] as f32 * (1.0 - blend_factor) + self.color[1] as f32 * blend_factor) as u8;
                        let b = (pixel[2] as f32 * (1.0 - blend_factor) + self.color[2] as f32 * blend_factor) as u8;

                        buffer.set_pixel(x, y, [r, g, b, pixel[3]]);
                    }
                }
            }
        }
    }
}

/// Wood/material grain direction
#[derive(Clone, Copy, Default)]
pub enum GrainDirection {
    #[default]
    Horizontal,
    Vertical,
    /// Angle in degrees
    Angled(f32),
}

/// Generate wood-like or material grain patterns
#[derive(Clone)]
pub struct Grain {
    /// Grain scale (larger = wider grain)
    pub scale: f32,
    /// Grain intensity (0.0 to 1.0)
    pub intensity: f32,
    /// Grain direction
    pub direction: GrainDirection,
    /// Color variation in grain
    pub color_variation: f32,
    /// Random seed
    pub seed: u32,
}

impl Default for Grain {
    fn default() -> Self {
        Self {
            scale: 0.1,
            intensity: 0.4,
            direction: GrainDirection::Horizontal,
            color_variation: 0.1,
            seed: 0,
        }
    }
}

impl Grain {
    /// Wood grain preset
    pub fn wood() -> Self {
        Self {
            scale: 0.08,
            intensity: 0.5,
            direction: GrainDirection::Vertical,
            color_variation: 0.15,
            seed: 0,
        }
    }

    /// Brushed metal grain preset
    pub fn brushed_metal() -> Self {
        Self {
            scale: 0.02,
            intensity: 0.3,
            direction: GrainDirection::Horizontal,
            color_variation: 0.05,
            seed: 0,
        }
    }
}

impl TextureModifier for Grain {
    fn apply(&self, buffer: &mut TextureBuffer) {
        use super::color::{rgb_to_hsv, hsv_to_rgb};

        let perlin = Perlin::new(self.seed);

        let (cos_a, sin_a) = match self.direction {
            GrainDirection::Horizontal => (1.0, 0.0),
            GrainDirection::Vertical => (0.0, 1.0),
            GrainDirection::Angled(deg) => (deg.to_radians().cos(), deg.to_radians().sin()),
        };

        for y in 0..buffer.height {
            for x in 0..buffer.width {
                // Rotate coordinates for direction
                let rx = x as f64 * cos_a as f64 + y as f64 * sin_a as f64;
                let ry = -(x as f64) * sin_a as f64 + y as f64 * cos_a as f64;

                // Stretched noise for grain effect
                let grain_value = perlin.get([rx * self.scale as f64 * 0.1, ry * self.scale as f64]);

                // Add fine detail
                let detail = perlin.get([rx * self.scale as f64 * 0.5, ry * self.scale as f64 * 0.1]);
                let combined = (grain_value + detail * 0.3) / 1.3;

                let pixel = buffer.get_pixel(x, y);
                let (h, s, v) = rgb_to_hsv(pixel[0], pixel[1], pixel[2]);

                // Apply value modulation
                let v_mod = v * (1.0 + combined as f32 * self.intensity);
                let v_new = v_mod.clamp(0.0, 1.0);

                // Apply slight hue variation
                let h_mod = h + combined as f32 * self.color_variation * 20.0;
                let h_new = if h_mod < 0.0 { h_mod + 360.0 } else { h_mod % 360.0 };

                let (r, g, b) = hsv_to_rgb(h_new, s, v_new);
                buffer.set_pixel(x, y, [r, g, b, pixel[3]]);
            }
        }
    }
}

/// Generate pores/pitting on a surface
#[derive(Clone)]
pub struct Pores {
    /// Pore density (0.0 to 1.0)
    pub density: f32,
    /// Pore size range in pixels
    pub size: (f32, f32),
    /// Pore depth/darkness (0.0 to 1.0)
    pub depth: f32,
    /// Random seed
    pub seed: u32,
}

impl Default for Pores {
    fn default() -> Self {
        Self {
            density: 0.3,
            size: (1.0, 3.0),
            depth: 0.4,
            seed: 0,
        }
    }
}

impl TextureModifier for Pores {
    fn apply(&self, buffer: &mut TextureBuffer) {
        use noise::Worley;

        let worley = Worley::new(self.seed);
        let scale = (0.05 + self.density * 0.1) as f64;

        for y in 0..buffer.height {
            for x in 0..buffer.width {
                let nx = x as f64 * scale;
                let ny = y as f64 * scale;

                let cell_value = worley.get([nx, ny]);

                // Create pore at cell centers
                let pore_intensity = (1.0 - cell_value.abs() as f32).max(0.0);
                let pore_intensity = if pore_intensity > 0.7 {
                    (pore_intensity - 0.7) / 0.3 * self.depth
                } else {
                    0.0
                };

                if pore_intensity > 0.0 {
                    let pixel = buffer.get_pixel(x, y);
                    let darken = 1.0 - pore_intensity * 0.5;
                    buffer.set_pixel(x, y, [
                        (pixel[0] as f32 * darken) as u8,
                        (pixel[1] as f32 * darken) as u8,
                        (pixel[2] as f32 * darken) as u8,
                        pixel[3],
                    ]);
                }
            }
        }
    }
}

/// Rust pattern type
#[derive(Clone, Copy, Default)]
pub enum RustPattern {
    /// Uniform rust spots
    #[default]
    Spots,
    /// Rust accumulating at edges
    EdgeRust,
    /// Heavy oxidation
    HeavyOxide,
    /// Streaky rust (from water running)
    Streaky,
}

/// Rust color preset
#[derive(Clone, Copy, Default)]
pub enum RustColor {
    /// Classic orange rust
    #[default]
    Orange,
    /// Dark brown rust
    Brown,
    /// Greenish patina (for copper/bronze)
    Patina,
}

impl RustColor {
    fn to_rgb(&self) -> [u8; 4] {
        match self {
            RustColor::Orange => [180, 90, 40, 255],
            RustColor::Brown => [100, 60, 35, 255],
            RustColor::Patina => [100, 140, 120, 255],
        }
    }
}

/// Generate rust/oxidation on a texture
#[derive(Clone)]
pub struct Rust {
    /// Rust amount (0.0 to 1.0)
    pub amount: f32,
    /// Rust color preset
    pub color: RustColor,
    /// Rust pattern
    pub pattern: RustPattern,
    /// Random seed
    pub seed: u32,
}

impl Default for Rust {
    fn default() -> Self {
        Self {
            amount: 0.3,
            color: RustColor::Orange,
            pattern: RustPattern::Spots,
            seed: 0,
        }
    }
}

impl TextureModifier for Rust {
    fn apply(&self, buffer: &mut TextureBuffer) {
        use super::color::{rgb_to_hsv, hsv_to_rgb};

        let perlin = Perlin::new(self.seed);
        let rust_rgb = self.color.to_rgb();

        for y in 0..buffer.height {
            for x in 0..buffer.width {
                let nx = x as f64 * 0.03;
                let ny = y as f64 * 0.03;

                // Multi-octave noise for rust distribution
                let n1 = perlin.get([nx, ny]);
                let n2 = perlin.get([nx * 2.0, ny * 2.0]) * 0.5;
                let n3 = perlin.get([nx * 4.0, ny * 4.0]) * 0.25;
                let noise = (n1 + n2 + n3) / 1.75;

                // Convert to rust intensity
                let threshold = 1.0 - self.amount;
                let rust_intensity = if noise as f32 > threshold {
                    ((noise as f32 - threshold) / (1.0 - threshold)).min(1.0)
                } else {
                    0.0
                };

                if rust_intensity > 0.0 {
                    let pixel = buffer.get_pixel(x, y);

                    // Blend toward rust color
                    let blend = rust_intensity * 0.8;
                    let r = (pixel[0] as f32 * (1.0 - blend) + rust_rgb[0] as f32 * blend) as u8;
                    let g = (pixel[1] as f32 * (1.0 - blend) + rust_rgb[1] as f32 * blend) as u8;
                    let b = (pixel[2] as f32 * (1.0 - blend) + rust_rgb[2] as f32 * blend) as u8;

                    // Add some roughness variation
                    let (h, s, v) = rgb_to_hsv(r, g, b);
                    let v_rough = v * (0.9 + noise as f32 * 0.2).min(1.0);
                    let (r, g, b) = hsv_to_rgb(h, s.min(1.0), v_rough.min(1.0));

                    buffer.set_pixel(x, y, [r, g, b, pixel[3]]);
                }
            }
        }
    }
}

/// Generate water stains/mineral deposits
#[derive(Clone)]
pub struct WaterStains {
    /// Stain intensity (0.0 to 1.0)
    pub intensity: f32,
    /// Stain color (usually white/gray for mineral deposits)
    pub color: [u8; 4],
    /// Random seed
    pub seed: u32,
}

impl Default for WaterStains {
    fn default() -> Self {
        Self {
            intensity: 0.3,
            color: [220, 220, 215, 255],
            seed: 0,
        }
    }
}

impl TextureModifier for WaterStains {
    fn apply(&self, buffer: &mut TextureBuffer) {
        let perlin = Perlin::new(self.seed);

        for y in 0..buffer.height {
            for x in 0..buffer.width {
                // Vertical bias for drip patterns
                let nx = x as f64 * 0.02;
                let ny = y as f64 * 0.08;

                let stain_value = perlin.get([nx, ny]);
                let detail = perlin.get([nx * 3.0, ny * 3.0]) * 0.3;
                let combined = (stain_value + detail) / 1.3;

                let threshold = 1.0 - self.intensity * 0.5;
                if combined as f32 > threshold {
                    let stain_intensity = ((combined as f32 - threshold) / (1.0 - threshold)).min(1.0) * self.intensity;

                    let pixel = buffer.get_pixel(x, y);
                    let blend = stain_intensity * 0.4;
                    let r = (pixel[0] as f32 * (1.0 - blend) + self.color[0] as f32 * blend) as u8;
                    let g = (pixel[1] as f32 * (1.0 - blend) + self.color[1] as f32 * blend) as u8;
                    let b = (pixel[2] as f32 * (1.0 - blend) + self.color[2] as f32 * blend) as u8;

                    buffer.set_pixel(x, y, [r, g, b, pixel[3]]);
                }
            }
        }
    }
}

/// Generate dust accumulation on a surface
#[derive(Clone)]
pub struct Dust {
    /// Dust density (0.0 to 1.0)
    pub density: f32,
    /// Dust color
    pub color: [u8; 4],
    /// Random seed
    pub seed: u32,
}

impl Default for Dust {
    fn default() -> Self {
        Self {
            density: 0.2,
            color: [180, 175, 165, 255],
            seed: 0,
        }
    }
}

impl TextureModifier for Dust {
    fn apply(&self, buffer: &mut TextureBuffer) {
        let perlin = Perlin::new(self.seed);

        for y in 0..buffer.height {
            for x in 0..buffer.width {
                let nx = x as f64 * 0.05;
                let ny = y as f64 * 0.05;

                let dust_value = (perlin.get([nx, ny]) + 1.0) / 2.0;
                let dust_intensity = (dust_value as f32 * self.density).min(1.0);

                let pixel = buffer.get_pixel(x, y);
                let blend = dust_intensity * 0.3;
                let r = (pixel[0] as f32 * (1.0 - blend) + self.color[0] as f32 * blend) as u8;
                let g = (pixel[1] as f32 * (1.0 - blend) + self.color[1] as f32 * blend) as u8;
                let b = (pixel[2] as f32 * (1.0 - blend) + self.color[2] as f32 * blend) as u8;

                buffer.set_pixel(x, y, [r, g, b, pixel[3]]);
            }
        }
    }
}

/// Apply edge highlighting (makes edges appear worn/bright)
#[derive(Clone)]
pub struct EdgeHighlight {
    /// Edge detection threshold
    pub threshold: f32,
    /// Highlight intensity
    pub intensity: f32,
    /// Highlight color (usually lighter than base)
    pub color: [u8; 4],
}

impl Default for EdgeHighlight {
    fn default() -> Self {
        Self {
            threshold: 0.1,
            intensity: 0.5,
            color: [255, 250, 240, 255],
        }
    }
}

impl TextureModifier for EdgeHighlight {
    fn apply(&self, buffer: &mut TextureBuffer) {
        let original = buffer.clone();

        for y in 1..buffer.height.saturating_sub(1) {
            for x in 1..buffer.width.saturating_sub(1) {
                // Sobel edge detection
                let get_lum = |px: u32, py: u32| {
                    let p = original.get_pixel(px, py);
                    (p[0] as f32 * 0.299 + p[1] as f32 * 0.587 + p[2] as f32 * 0.114) / 255.0
                };

                let gx = get_lum(x + 1, y - 1) + 2.0 * get_lum(x + 1, y) + get_lum(x + 1, y + 1)
                    - get_lum(x - 1, y - 1) - 2.0 * get_lum(x - 1, y) - get_lum(x - 1, y + 1);

                let gy = get_lum(x - 1, y + 1) + 2.0 * get_lum(x, y + 1) + get_lum(x + 1, y + 1)
                    - get_lum(x - 1, y - 1) - 2.0 * get_lum(x, y - 1) - get_lum(x + 1, y - 1);

                let edge = (gx * gx + gy * gy).sqrt();

                if edge > self.threshold {
                    let edge_intensity = ((edge - self.threshold) / (1.0 - self.threshold)).min(1.0) * self.intensity;

                    let pixel = buffer.get_pixel(x, y);
                    let r = (pixel[0] as f32 * (1.0 - edge_intensity) + self.color[0] as f32 * edge_intensity) as u8;
                    let g = (pixel[1] as f32 * (1.0 - edge_intensity) + self.color[1] as f32 * edge_intensity) as u8;
                    let b = (pixel[2] as f32 * (1.0 - edge_intensity) + self.color[2] as f32 * edge_intensity) as u8;

                    buffer.set_pixel(x, y, [r, g, b, pixel[3]]);
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::texture::solid;

    #[test]
    fn test_scratches_apply() {
        let mut tex = solid(64, 64, [128, 128, 128, 255]);
        Scratches::light().apply(&mut tex);
        // Should have some variation
        let mut has_variation = false;
        for y in 0..64 {
            for x in 0..64 {
                let p = tex.get_pixel(x, y);
                if p != [128, 128, 128, 255] {
                    has_variation = true;
                    break;
                }
            }
        }
        assert!(has_variation);
    }

    #[test]
    fn test_grain_apply() {
        let mut tex = solid(64, 64, [128, 100, 80, 255]);
        Grain::wood().apply(&mut tex);
        // Grain should modify the texture
        let center = tex.get_pixel(32, 32);
        assert!(center != [128, 100, 80, 255] || tex.get_pixel(16, 16) != [128, 100, 80, 255]);
    }

    #[test]
    fn test_rust_apply() {
        let mut tex = solid(64, 64, [150, 150, 150, 255]);
        Rust { amount: 0.5, ..Default::default() }.apply(&mut tex);
        // Should have some rust coloring
        let mut has_rust = false;
        for y in 0..64 {
            for x in 0..64 {
                let p = tex.get_pixel(x, y);
                // Check for orange-ish tint
                if p[0] > p[1] && p[0] > p[2] {
                    has_rust = true;
                    break;
                }
            }
        }
        assert!(has_rust);
    }
}
