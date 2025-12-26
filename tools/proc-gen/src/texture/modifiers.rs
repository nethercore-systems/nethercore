//! Texture modifiers for post-processing
//!
//! Provides a trait-based system for applying modifications to textures,
//! similar to the mesh modifier pattern in `proc_gen::mesh::modifiers`.

use super::TextureBuffer;

/// Trait for texture modifiers
pub trait TextureModifier {
    /// Apply the modification to the texture buffer
    fn apply(&self, buffer: &mut TextureBuffer);
}

/// Extension trait for fluent modifier application
pub trait TextureApply {
    /// Apply a modifier and return self for chaining
    fn apply<M: TextureModifier>(&mut self, modifier: M) -> &mut Self;
}

impl TextureApply for TextureBuffer {
    fn apply<M: TextureModifier>(&mut self, modifier: M) -> &mut Self {
        modifier.apply(self);
        self
    }
}

/// Blend mode for combining textures
#[derive(Clone, Copy, Default)]
pub enum BlendMode {
    /// Simple alpha-based blending (source over destination)
    #[default]
    Normal,
    /// Multiply colors (darkens)
    Multiply,
    /// Screen colors (lightens)
    Screen,
    /// Overlay (combines multiply and screen)
    Overlay,
    /// Additive blending
    Add,
}

/// Blend another texture onto the target
pub struct Blend {
    /// Source texture to blend
    pub source: TextureBuffer,
    /// Blend mode to use
    pub mode: BlendMode,
    /// Opacity (0.0 = invisible, 1.0 = fully opaque)
    pub opacity: f32,
}

impl TextureModifier for Blend {
    fn apply(&self, buffer: &mut TextureBuffer) {
        for y in 0..buffer.height.min(self.source.height) {
            for x in 0..buffer.width.min(self.source.width) {
                let base = buffer.get_pixel(x, y);
                let blend = self.source.get_pixel(x, y);
                let result = blend_pixels(base, blend, self.mode, self.opacity);
                buffer.set_pixel(x, y, result);
            }
        }
    }
}

fn blend_pixels(base: [u8; 4], blend: [u8; 4], mode: BlendMode, opacity: f32) -> [u8; 4] {
    let bf = |v: u8| v as f32 / 255.0;
    let tb = |v: f32| (v.clamp(0.0, 1.0) * 255.0) as u8;

    let (br, bg, bb) = (bf(base[0]), bf(base[1]), bf(base[2]));
    let (sr, sg, sb) = (bf(blend[0]), bf(blend[1]), bf(blend[2]));

    let (rr, rg, rb) = match mode {
        BlendMode::Normal => (sr, sg, sb),
        BlendMode::Multiply => (br * sr, bg * sg, bb * sb),
        BlendMode::Screen => (
            1.0 - (1.0 - br) * (1.0 - sr),
            1.0 - (1.0 - bg) * (1.0 - sg),
            1.0 - (1.0 - bb) * (1.0 - sb),
        ),
        BlendMode::Overlay => {
            let overlay =
                |b: f32, s: f32| {
                    if b < 0.5 {
                        2.0 * b * s
                    } else {
                        1.0 - 2.0 * (1.0 - b) * (1.0 - s)
                    }
                };
            (overlay(br, sr), overlay(bg, sg), overlay(bb, sb))
        }
        BlendMode::Add => ((br + sr).min(1.0), (bg + sg).min(1.0), (bb + sb).min(1.0)),
    };

    // Lerp between base and blended based on opacity
    let lerp = |b: f32, r: f32| b + (r - b) * opacity;
    [
        tb(lerp(br, rr)),
        tb(lerp(bg, rg)),
        tb(lerp(bb, rb)),
        base[3].max(blend[3]), // Preserve max alpha
    ]
}

/// Adjust contrast of the texture
pub struct Contrast {
    /// Contrast factor (1.0 = no change, >1 = more contrast, <1 = less)
    pub factor: f32,
}

impl TextureModifier for Contrast {
    fn apply(&self, buffer: &mut TextureBuffer) {
        for pixel in buffer.pixels.chunks_exact_mut(4) {
            for i in 0..3 {
                let v = pixel[i] as f32 / 255.0;
                let adjusted = ((v - 0.5) * self.factor + 0.5).clamp(0.0, 1.0);
                pixel[i] = (adjusted * 255.0) as u8;
            }
        }
    }
}

/// Invert colors (RGB only, alpha unchanged)
pub struct Invert;

impl TextureModifier for Invert {
    fn apply(&self, buffer: &mut TextureBuffer) {
        for pixel in buffer.pixels.chunks_exact_mut(4) {
            pixel[0] = 255 - pixel[0];
            pixel[1] = 255 - pixel[1];
            pixel[2] = 255 - pixel[2];
            // Alpha unchanged
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::texture::solid;

    #[test]
    fn test_texture_apply_trait() {
        let mut tex = solid(8, 8, [128, 128, 128, 255]);
        tex.apply(Contrast { factor: 1.5 }).apply(Invert);

        // After invert, the contrast-adjusted gray should be inverted
        let p = tex.get_pixel(0, 0);
        assert_ne!(p[0], 128); // Should be different from original
    }

    #[test]
    fn test_contrast_neutral() {
        let original = [128, 64, 192, 255];
        let mut tex = solid(4, 4, original);
        tex.apply(Contrast { factor: 1.0 });

        // Factor 1.0 should not change anything
        let p = tex.get_pixel(0, 0);
        assert_eq!(p, original);
    }

    #[test]
    fn test_contrast_increase() {
        let mut tex = solid(4, 4, [128, 128, 128, 255]);
        tex.apply(Contrast { factor: 2.0 });

        // Gray (0.5) at factor 2.0: (0.5 - 0.5) * 2 + 0.5 = 0.5, still 128
        let p = tex.get_pixel(0, 0);
        assert_eq!(p[0], 128);

        // Try with a different value
        let mut tex2 = solid(4, 4, [192, 192, 192, 255]); // 0.75
        tex2.apply(Contrast { factor: 2.0 });
        // (0.75 - 0.5) * 2 + 0.5 = 1.0 -> 255
        let p2 = tex2.get_pixel(0, 0);
        assert_eq!(p2[0], 255);
    }

    #[test]
    fn test_invert() {
        let original = [100, 150, 200, 255];
        let mut tex = solid(4, 4, original);
        tex.apply(Invert);

        let p = tex.get_pixel(0, 0);
        assert_eq!(p[0], 155); // 255 - 100
        assert_eq!(p[1], 105); // 255 - 150
        assert_eq!(p[2], 55); // 255 - 200
        assert_eq!(p[3], 255); // Alpha unchanged
    }

    #[test]
    fn test_blend_normal() {
        let base = solid(4, 4, [100, 100, 100, 255]);
        let overlay = solid(4, 4, [200, 200, 200, 255]);

        let mut tex = base;
        tex.apply(Blend {
            source: overlay,
            mode: BlendMode::Normal,
            opacity: 0.5,
        });

        let p = tex.get_pixel(0, 0);
        // At 50% opacity, should be halfway between 100 and 200
        assert_eq!(p[0], 150);
    }

    #[test]
    fn test_blend_multiply() {
        let base = solid(4, 4, [255, 255, 255, 255]);
        let overlay = solid(4, 4, [128, 128, 128, 255]);

        let mut tex = base;
        tex.apply(Blend {
            source: overlay,
            mode: BlendMode::Multiply,
            opacity: 1.0,
        });

        let p = tex.get_pixel(0, 0);
        // White * 0.5 gray = 0.5 gray = 128
        assert_eq!(p[0], 128);
    }

    #[test]
    fn test_blend_add() {
        let base = solid(4, 4, [100, 100, 100, 255]);
        let overlay = solid(4, 4, [100, 100, 100, 255]);

        let mut tex = base;
        tex.apply(Blend {
            source: overlay,
            mode: BlendMode::Add,
            opacity: 1.0,
        });

        let p = tex.get_pixel(0, 0);
        // 100/255 + 100/255 = 200/255 -> 200
        assert_eq!(p[0], 200);
    }

    #[test]
    fn test_blend_different_sizes() {
        let mut base = solid(8, 8, [100, 100, 100, 255]);
        let overlay = solid(4, 4, [200, 200, 200, 255]);

        base.apply(Blend {
            source: overlay,
            mode: BlendMode::Normal,
            opacity: 1.0,
        });

        // Top-left 4x4 should be blended
        let p1 = base.get_pixel(0, 0);
        assert_eq!(p1[0], 200);

        // Outside overlay area should be unchanged
        let p2 = base.get_pixel(6, 6);
        assert_eq!(p2[0], 100);
    }
}
