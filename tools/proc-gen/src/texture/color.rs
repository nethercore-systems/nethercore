//! Color manipulation utilities for procedural textures
//!
//! Provides HSV/HSL color space conversion and color variation systems
//! that go beyond simple brightness changes to include hue shifts,
//! saturation adjustments, and color temperature changes.

use super::TextureBuffer;
use super::modifiers::TextureModifier;

/// RGB to HSV conversion
#[inline]
pub fn rgb_to_hsv(r: u8, g: u8, b: u8) -> (f32, f32, f32) {
    let r = r as f32 / 255.0;
    let g = g as f32 / 255.0;
    let b = b as f32 / 255.0;

    let max = r.max(g).max(b);
    let min = r.min(g).min(b);
    let delta = max - min;

    let h = if delta == 0.0 {
        0.0
    } else if max == r {
        60.0 * (((g - b) / delta) % 6.0)
    } else if max == g {
        60.0 * (((b - r) / delta) + 2.0)
    } else {
        60.0 * (((r - g) / delta) + 4.0)
    };

    let h = if h < 0.0 { h + 360.0 } else { h };

    let s = if max == 0.0 { 0.0 } else { delta / max };
    let v = max;

    (h, s, v)
}

/// HSV to RGB conversion
#[inline]
pub fn hsv_to_rgb(h: f32, s: f32, v: f32) -> (u8, u8, u8) {
    let h = h % 360.0;
    let h = if h < 0.0 { h + 360.0 } else { h };

    let c = v * s;
    let x = c * (1.0 - ((h / 60.0) % 2.0 - 1.0).abs());
    let m = v - c;

    let (r, g, b) = match (h / 60.0) as u32 {
        0 => (c, x, 0.0),
        1 => (x, c, 0.0),
        2 => (0.0, c, x),
        3 => (0.0, x, c),
        4 => (x, 0.0, c),
        _ => (c, 0.0, x),
    };

    (
        ((r + m) * 255.0).clamp(0.0, 255.0) as u8,
        ((g + m) * 255.0).clamp(0.0, 255.0) as u8,
        ((b + m) * 255.0).clamp(0.0, 255.0) as u8,
    )
}

/// Color temperature adjustment (warm/cool shift)
///
/// Negative values shift toward blue (cool), positive toward orange (warm)
#[inline]
pub fn apply_temperature(r: u8, g: u8, b: u8, temperature: f32) -> (u8, u8, u8) {
    let temp = temperature.clamp(-1.0, 1.0);

    if temp > 0.0 {
        // Warm: boost red, reduce blue
        let r_adj = r as f32 + (255.0 - r as f32) * temp * 0.3;
        let b_adj = b as f32 * (1.0 - temp * 0.3);
        (
            r_adj.clamp(0.0, 255.0) as u8,
            g,
            b_adj.clamp(0.0, 255.0) as u8,
        )
    } else {
        // Cool: boost blue, reduce red
        let temp = temp.abs();
        let r_adj = r as f32 * (1.0 - temp * 0.3);
        let b_adj = b as f32 + (255.0 - b as f32) * temp * 0.3;
        (
            r_adj.clamp(0.0, 255.0) as u8,
            g,
            b_adj.clamp(0.0, 255.0) as u8,
        )
    }
}

/// Color variation configuration
///
/// Provides sophisticated color manipulation that goes beyond simple
/// brightness changes to create realistic material variation.
#[derive(Clone)]
pub struct ColorVariation {
    /// Hue shift range in degrees (-180 to 180)
    pub hue_shift: (f32, f32),
    /// Saturation multiplier range (0.0 to 2.0)
    pub saturation_scale: (f32, f32),
    /// Value/brightness multiplier range (0.0 to 2.0)
    pub value_scale: (f32, f32),
    /// Color temperature shift (-1.0 cool to 1.0 warm)
    pub temperature_shift: (f32, f32),
}

impl Default for ColorVariation {
    fn default() -> Self {
        Self {
            hue_shift: (0.0, 0.0),
            saturation_scale: (1.0, 1.0),
            value_scale: (1.0, 1.0),
            temperature_shift: (0.0, 0.0),
        }
    }
}

impl ColorVariation {
    /// Create a subtle natural variation (good for organic materials)
    pub fn subtle() -> Self {
        Self {
            hue_shift: (-5.0, 5.0),
            saturation_scale: (0.9, 1.1),
            value_scale: (0.95, 1.05),
            temperature_shift: (-0.1, 0.1),
        }
    }

    /// Create moderate variation (good for worn/weathered materials)
    pub fn moderate() -> Self {
        Self {
            hue_shift: (-15.0, 15.0),
            saturation_scale: (0.7, 1.2),
            value_scale: (0.8, 1.1),
            temperature_shift: (-0.2, 0.2),
        }
    }

    /// Create strong variation (good for rust, decay, etc.)
    pub fn strong() -> Self {
        Self {
            hue_shift: (-30.0, 30.0),
            saturation_scale: (0.5, 1.3),
            value_scale: (0.6, 1.2),
            temperature_shift: (-0.4, 0.4),
        }
    }

    /// Create rusty metal variation
    pub fn rusty() -> Self {
        Self {
            hue_shift: (-10.0, 25.0),  // Shift toward orange/brown
            saturation_scale: (0.6, 1.3),
            value_scale: (0.7, 1.0),
            temperature_shift: (0.2, 0.6),  // Warmer
        }
    }

    /// Sample a random variation using a noise value (0.0 to 1.0)
    #[inline]
    pub fn sample(&self, noise: f32) -> ColorVariationSample {
        let lerp = |range: (f32, f32), t: f32| range.0 + (range.1 - range.0) * t;
        ColorVariationSample {
            hue_shift: lerp(self.hue_shift, noise),
            saturation_scale: lerp(self.saturation_scale, noise),
            value_scale: lerp(self.value_scale, noise),
            temperature_shift: lerp(self.temperature_shift, noise),
        }
    }

    /// Sample using multiple noise values for independent variation
    #[inline]
    pub fn sample_multi(&self, h_noise: f32, s_noise: f32, v_noise: f32, t_noise: f32) -> ColorVariationSample {
        let lerp = |range: (f32, f32), t: f32| range.0 + (range.1 - range.0) * t;
        ColorVariationSample {
            hue_shift: lerp(self.hue_shift, h_noise),
            saturation_scale: lerp(self.saturation_scale, s_noise),
            value_scale: lerp(self.value_scale, v_noise),
            temperature_shift: lerp(self.temperature_shift, t_noise),
        }
    }
}

/// A sampled color variation to apply to a pixel
#[derive(Clone, Copy)]
pub struct ColorVariationSample {
    pub hue_shift: f32,
    pub saturation_scale: f32,
    pub value_scale: f32,
    pub temperature_shift: f32,
}

impl ColorVariationSample {
    /// Apply this variation to an RGB color
    #[inline]
    pub fn apply(&self, r: u8, g: u8, b: u8) -> (u8, u8, u8) {
        // Convert to HSV
        let (mut h, mut s, mut v) = rgb_to_hsv(r, g, b);

        // Apply HSV modifications
        h = (h + self.hue_shift) % 360.0;
        if h < 0.0 { h += 360.0; }
        s = (s * self.saturation_scale).clamp(0.0, 1.0);
        v = (v * self.value_scale).clamp(0.0, 1.0);

        // Convert back to RGB
        let (r, g, b) = hsv_to_rgb(h, s, v);

        // Apply temperature shift
        apply_temperature(r, g, b, self.temperature_shift)
    }
}

/// Apply noise-driven color variation to a texture
pub struct ApplyColorVariation {
    /// Variation parameters
    pub variation: ColorVariation,
    /// Noise scale for variation sampling
    pub noise_scale: f64,
    /// Random seed
    pub seed: u32,
    /// Use independent noise for each channel (more varied) or single noise (coherent)
    pub independent_channels: bool,
}

impl Default for ApplyColorVariation {
    fn default() -> Self {
        Self {
            variation: ColorVariation::subtle(),
            noise_scale: 0.05,
            seed: 0,
            independent_channels: true,
        }
    }
}

impl TextureModifier for ApplyColorVariation {
    fn apply(&self, buffer: &mut TextureBuffer) {
        use noise::{NoiseFn, Perlin};

        let perlin = Perlin::new(self.seed);

        for y in 0..buffer.height {
            for x in 0..buffer.width {
                let nx = x as f64 * self.noise_scale;
                let ny = y as f64 * self.noise_scale;

                let sample = if self.independent_channels {
                    let h_noise = ((perlin.get([nx, ny]) + 1.0) / 2.0) as f32;
                    let s_noise = ((perlin.get([nx + 100.0, ny + 100.0]) + 1.0) / 2.0) as f32;
                    let v_noise = ((perlin.get([nx + 200.0, ny + 200.0]) + 1.0) / 2.0) as f32;
                    let t_noise = ((perlin.get([nx + 300.0, ny + 300.0]) + 1.0) / 2.0) as f32;
                    self.variation.sample_multi(h_noise, s_noise, v_noise, t_noise)
                } else {
                    let noise = ((perlin.get([nx, ny]) + 1.0) / 2.0) as f32;
                    self.variation.sample(noise)
                };

                let pixel = buffer.get_pixel(x, y);
                let (r, g, b) = sample.apply(pixel[0], pixel[1], pixel[2]);
                buffer.set_pixel(x, y, [r, g, b, pixel[3]]);
            }
        }
    }
}

/// Adjust hue of entire texture
pub struct HueShift {
    /// Hue shift in degrees (-180 to 180)
    pub degrees: f32,
}

impl TextureModifier for HueShift {
    fn apply(&self, buffer: &mut TextureBuffer) {
        for y in 0..buffer.height {
            for x in 0..buffer.width {
                let pixel = buffer.get_pixel(x, y);
                let (mut h, s, v) = rgb_to_hsv(pixel[0], pixel[1], pixel[2]);
                h = (h + self.degrees) % 360.0;
                if h < 0.0 { h += 360.0; }
                let (r, g, b) = hsv_to_rgb(h, s, v);
                buffer.set_pixel(x, y, [r, g, b, pixel[3]]);
            }
        }
    }
}

/// Adjust saturation of entire texture
pub struct Saturation {
    /// Saturation multiplier (0.0 = grayscale, 1.0 = unchanged, 2.0 = double)
    pub factor: f32,
}

impl TextureModifier for Saturation {
    fn apply(&self, buffer: &mut TextureBuffer) {
        for y in 0..buffer.height {
            for x in 0..buffer.width {
                let pixel = buffer.get_pixel(x, y);
                let (h, s, v) = rgb_to_hsv(pixel[0], pixel[1], pixel[2]);
                let new_s = (s * self.factor).clamp(0.0, 1.0);
                let (r, g, b) = hsv_to_rgb(h, new_s, v);
                buffer.set_pixel(x, y, [r, g, b, pixel[3]]);
            }
        }
    }
}

/// Apply color temperature shift to entire texture
pub struct Temperature {
    /// Temperature shift (-1.0 cool to 1.0 warm)
    pub shift: f32,
}

impl TextureModifier for Temperature {
    fn apply(&self, buffer: &mut TextureBuffer) {
        for y in 0..buffer.height {
            for x in 0..buffer.width {
                let pixel = buffer.get_pixel(x, y);
                let (r, g, b) = apply_temperature(pixel[0], pixel[1], pixel[2], self.shift);
                buffer.set_pixel(x, y, [r, g, b, pixel[3]]);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rgb_hsv_roundtrip() {
        let colors = [
            (255, 0, 0),     // Red
            (0, 255, 0),     // Green
            (0, 0, 255),     // Blue
            (255, 255, 0),   // Yellow
            (128, 128, 128), // Gray
            (200, 100, 50),  // Orange-ish
        ];

        for (r, g, b) in colors {
            let (h, s, v) = rgb_to_hsv(r, g, b);
            let (r2, g2, b2) = hsv_to_rgb(h, s, v);
            assert!((r as i16 - r2 as i16).abs() <= 1, "Red mismatch: {} vs {}", r, r2);
            assert!((g as i16 - g2 as i16).abs() <= 1, "Green mismatch: {} vs {}", g, g2);
            assert!((b as i16 - b2 as i16).abs() <= 1, "Blue mismatch: {} vs {}", b, b2);
        }
    }

    #[test]
    fn test_color_variation_sample() {
        let variation = ColorVariation::rusty();
        let sample = variation.sample(0.5);

        // Apply to a gray color
        let (r, g, b) = sample.apply(128, 128, 128);

        // Should be different from original due to variation
        assert!(r != 128 || g != 128 || b != 128);
    }

    #[test]
    fn test_temperature_warm() {
        let (r, g, b) = apply_temperature(128, 128, 128, 0.5);
        assert!(r > 128); // Red should increase
        assert!(b < 128); // Blue should decrease
    }

    #[test]
    fn test_temperature_cool() {
        let (r, g, b) = apply_temperature(128, 128, 128, -0.5);
        assert!(r < 128); // Red should decrease
        assert!(b > 128); // Blue should increase
    }
}
