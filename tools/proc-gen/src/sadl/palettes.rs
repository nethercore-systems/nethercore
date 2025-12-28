//! Color palettes for harmonious color schemes
//!
//! Each palette defines HSL ranges and sampling functions for
//! generating consistent, harmonious colors.

use crate::texture::{hsv_to_rgb, apply_temperature};

/// Color palette for coordinated color schemes
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ColorPalette {
    // Warm palettes
    WarmEarthy,
    Autumn,
    Sunset,
    Fire,

    // Cool palettes
    #[default]
    CoolMetal,
    Ocean,
    Arctic,
    Night,

    // Vibrant palettes
    Neon,
    Vibrant,
    Tropical,
    Rainbow,

    // Muted palettes
    Muted,
    Grayscale,
    Sepia,
    Dusty,

    // Soft palettes
    Pastel,
    Cotton,
    Dawn,
    Lavender,

    // Monochromatic
    Monochrome,
    BloodRed,
    ForestGreen,
    RoyalBlue,
    Gold,
    Violet,
    Copper,
    Jade,
}

impl ColorPalette {
    /// Get the palette specification
    pub fn spec(&self) -> PaletteSpec {
        match self {
            // Warm palettes
            ColorPalette::WarmEarthy => PaletteSpec {
                hue_ranges: vec![(20.0, 50.0), (30.0, 40.0)],
                saturation_range: (0.3, 0.6),
                lightness_range: (0.3, 0.6),
                accent_hue_offset: 180.0,
                primary_weight: 0.8,
                contrast_preference: 0.5,
            },
            ColorPalette::Autumn => PaletteSpec {
                hue_ranges: vec![(10.0, 45.0), (45.0, 60.0)],
                saturation_range: (0.5, 0.8),
                lightness_range: (0.3, 0.55),
                accent_hue_offset: 60.0,
                primary_weight: 0.7,
                contrast_preference: 0.6,
            },
            ColorPalette::Sunset => PaletteSpec {
                hue_ranges: vec![(0.0, 40.0), (270.0, 300.0)],
                saturation_range: (0.6, 0.9),
                lightness_range: (0.4, 0.65),
                accent_hue_offset: 40.0,
                primary_weight: 0.6,
                contrast_preference: 0.7,
            },
            ColorPalette::Fire => PaletteSpec {
                hue_ranges: vec![(0.0, 30.0), (40.0, 60.0)],
                saturation_range: (0.8, 1.0),
                lightness_range: (0.4, 0.7),
                accent_hue_offset: 20.0,
                primary_weight: 0.5,
                contrast_preference: 0.8,
            },

            // Cool palettes
            ColorPalette::CoolMetal => PaletteSpec {
                hue_ranges: vec![(200.0, 230.0), (0.0, 0.0)],
                saturation_range: (0.05, 0.2),
                lightness_range: (0.4, 0.7),
                accent_hue_offset: 30.0,
                primary_weight: 0.9,
                contrast_preference: 0.4,
            },
            ColorPalette::Ocean => PaletteSpec {
                hue_ranges: vec![(180.0, 220.0), (200.0, 240.0)],
                saturation_range: (0.4, 0.7),
                lightness_range: (0.3, 0.6),
                accent_hue_offset: 40.0,
                primary_weight: 0.7,
                contrast_preference: 0.5,
            },
            ColorPalette::Arctic => PaletteSpec {
                hue_ranges: vec![(190.0, 210.0), (170.0, 200.0)],
                saturation_range: (0.1, 0.3),
                lightness_range: (0.7, 0.95),
                accent_hue_offset: 20.0,
                primary_weight: 0.8,
                contrast_preference: 0.3,
            },
            ColorPalette::Night => PaletteSpec {
                hue_ranges: vec![(220.0, 270.0), (180.0, 220.0)],
                saturation_range: (0.3, 0.5),
                lightness_range: (0.1, 0.35),
                accent_hue_offset: 60.0,
                primary_weight: 0.7,
                contrast_preference: 0.4,
            },

            // Vibrant palettes
            ColorPalette::Neon => PaletteSpec {
                hue_ranges: vec![(280.0, 340.0), (160.0, 200.0), (40.0, 80.0)],
                saturation_range: (0.9, 1.0),
                lightness_range: (0.5, 0.7),
                accent_hue_offset: 120.0,
                primary_weight: 0.4,
                contrast_preference: 0.9,
            },
            ColorPalette::Vibrant => PaletteSpec {
                hue_ranges: vec![(0.0, 360.0)],
                saturation_range: (0.7, 1.0),
                lightness_range: (0.45, 0.6),
                accent_hue_offset: 180.0,
                primary_weight: 0.5,
                contrast_preference: 0.8,
            },
            ColorPalette::Tropical => PaletteSpec {
                hue_ranges: vec![(80.0, 180.0), (320.0, 360.0)],
                saturation_range: (0.6, 0.9),
                lightness_range: (0.4, 0.6),
                accent_hue_offset: 180.0,
                primary_weight: 0.6,
                contrast_preference: 0.7,
            },
            ColorPalette::Rainbow => PaletteSpec {
                hue_ranges: vec![(0.0, 360.0)],
                saturation_range: (0.8, 1.0),
                lightness_range: (0.5, 0.6),
                accent_hue_offset: 60.0,
                primary_weight: 0.3,
                contrast_preference: 0.9,
            },

            // Muted palettes
            ColorPalette::Muted => PaletteSpec {
                hue_ranges: vec![(0.0, 360.0)],
                saturation_range: (0.1, 0.3),
                lightness_range: (0.4, 0.6),
                accent_hue_offset: 30.0,
                primary_weight: 0.7,
                contrast_preference: 0.3,
            },
            ColorPalette::Grayscale => PaletteSpec {
                hue_ranges: vec![(0.0, 0.0)],
                saturation_range: (0.0, 0.05),
                lightness_range: (0.2, 0.8),
                accent_hue_offset: 0.0,
                primary_weight: 1.0,
                contrast_preference: 0.5,
            },
            ColorPalette::Sepia => PaletteSpec {
                hue_ranges: vec![(30.0, 45.0)],
                saturation_range: (0.2, 0.4),
                lightness_range: (0.3, 0.7),
                accent_hue_offset: 10.0,
                primary_weight: 0.9,
                contrast_preference: 0.4,
            },
            ColorPalette::Dusty => PaletteSpec {
                hue_ranges: vec![(20.0, 60.0), (180.0, 220.0)],
                saturation_range: (0.15, 0.35),
                lightness_range: (0.4, 0.65),
                accent_hue_offset: 180.0,
                primary_weight: 0.8,
                contrast_preference: 0.3,
            },

            // Soft palettes
            ColorPalette::Pastel => PaletteSpec {
                hue_ranges: vec![(0.0, 360.0)],
                saturation_range: (0.3, 0.5),
                lightness_range: (0.7, 0.85),
                accent_hue_offset: 60.0,
                primary_weight: 0.5,
                contrast_preference: 0.3,
            },
            ColorPalette::Cotton => PaletteSpec {
                hue_ranges: vec![(200.0, 250.0), (330.0, 360.0)],
                saturation_range: (0.2, 0.4),
                lightness_range: (0.75, 0.9),
                accent_hue_offset: 30.0,
                primary_weight: 0.7,
                contrast_preference: 0.2,
            },
            ColorPalette::Dawn => PaletteSpec {
                hue_ranges: vec![(330.0, 360.0), (0.0, 30.0), (200.0, 230.0)],
                saturation_range: (0.3, 0.5),
                lightness_range: (0.6, 0.8),
                accent_hue_offset: 40.0,
                primary_weight: 0.6,
                contrast_preference: 0.4,
            },
            ColorPalette::Lavender => PaletteSpec {
                hue_ranges: vec![(260.0, 290.0)],
                saturation_range: (0.25, 0.5),
                lightness_range: (0.6, 0.8),
                accent_hue_offset: 30.0,
                primary_weight: 0.8,
                contrast_preference: 0.3,
            },

            // Monochromatic
            ColorPalette::Monochrome => PaletteSpec {
                hue_ranges: vec![(0.0, 0.0)],
                saturation_range: (0.0, 0.0),
                lightness_range: (0.0, 1.0),
                accent_hue_offset: 0.0,
                primary_weight: 1.0,
                contrast_preference: 0.5,
            },
            ColorPalette::BloodRed => PaletteSpec {
                hue_ranges: vec![(350.0, 360.0), (0.0, 10.0)],
                saturation_range: (0.6, 0.9),
                lightness_range: (0.2, 0.5),
                accent_hue_offset: 15.0,
                primary_weight: 0.9,
                contrast_preference: 0.6,
            },
            ColorPalette::ForestGreen => PaletteSpec {
                hue_ranges: vec![(100.0, 150.0)],
                saturation_range: (0.4, 0.7),
                lightness_range: (0.2, 0.45),
                accent_hue_offset: 30.0,
                primary_weight: 0.85,
                contrast_preference: 0.4,
            },
            ColorPalette::RoyalBlue => PaletteSpec {
                hue_ranges: vec![(220.0, 250.0)],
                saturation_range: (0.5, 0.8),
                lightness_range: (0.3, 0.55),
                accent_hue_offset: 30.0,
                primary_weight: 0.85,
                contrast_preference: 0.5,
            },
            ColorPalette::Gold => PaletteSpec {
                hue_ranges: vec![(40.0, 55.0)],
                saturation_range: (0.7, 0.95),
                lightness_range: (0.4, 0.6),
                accent_hue_offset: 20.0,
                primary_weight: 0.9,
                contrast_preference: 0.5,
            },
            ColorPalette::Violet => PaletteSpec {
                hue_ranges: vec![(270.0, 300.0)],
                saturation_range: (0.5, 0.8),
                lightness_range: (0.3, 0.55),
                accent_hue_offset: 30.0,
                primary_weight: 0.85,
                contrast_preference: 0.5,
            },
            ColorPalette::Copper => PaletteSpec {
                hue_ranges: vec![(15.0, 35.0)],
                saturation_range: (0.5, 0.8),
                lightness_range: (0.35, 0.55),
                accent_hue_offset: 20.0,
                primary_weight: 0.9,
                contrast_preference: 0.5,
            },
            ColorPalette::Jade => PaletteSpec {
                hue_ranges: vec![(150.0, 180.0)],
                saturation_range: (0.4, 0.7),
                lightness_range: (0.35, 0.55),
                accent_hue_offset: 30.0,
                primary_weight: 0.85,
                contrast_preference: 0.4,
            },
        }
    }

    /// Sample a random color from this palette
    pub fn sample(&self, rng: &mut SimpleRng) -> [u8; 4] {
        self.spec().sample(rng)
    }

    /// Sample primary and accent colors
    pub fn sample_pair(&self, rng: &mut SimpleRng) -> ([u8; 4], [u8; 4]) {
        self.spec().sample_pair(rng)
    }

    /// Sample full color set (primary, secondary, accent, dark, light)
    pub fn sample_full(&self, rng: &mut SimpleRng) -> ColorSet {
        self.spec().sample_full(rng)
    }

    /// Parse palette from string
    pub fn from_str(s: &str) -> Option<Self> {
        match s.to_lowercase().replace(['-', '_'], "").as_str() {
            "warmearthy" | "earthy" => Some(ColorPalette::WarmEarthy),
            "autumn" | "fall" => Some(ColorPalette::Autumn),
            "sunset" => Some(ColorPalette::Sunset),
            "fire" | "flame" => Some(ColorPalette::Fire),
            "coolmetal" | "metal" => Some(ColorPalette::CoolMetal),
            "ocean" | "sea" => Some(ColorPalette::Ocean),
            "arctic" | "ice" | "frozen" => Some(ColorPalette::Arctic),
            "night" | "dark" => Some(ColorPalette::Night),
            "neon" => Some(ColorPalette::Neon),
            "vibrant" => Some(ColorPalette::Vibrant),
            "tropical" => Some(ColorPalette::Tropical),
            "rainbow" => Some(ColorPalette::Rainbow),
            "muted" => Some(ColorPalette::Muted),
            "grayscale" | "gray" | "grey" => Some(ColorPalette::Grayscale),
            "sepia" => Some(ColorPalette::Sepia),
            "dusty" => Some(ColorPalette::Dusty),
            "pastel" => Some(ColorPalette::Pastel),
            "cotton" | "soft" => Some(ColorPalette::Cotton),
            "dawn" => Some(ColorPalette::Dawn),
            "lavender" => Some(ColorPalette::Lavender),
            "monochrome" | "bw" => Some(ColorPalette::Monochrome),
            "bloodred" | "red" => Some(ColorPalette::BloodRed),
            "forestgreen" | "green" => Some(ColorPalette::ForestGreen),
            "royalblue" | "blue" => Some(ColorPalette::RoyalBlue),
            "gold" | "golden" => Some(ColorPalette::Gold),
            "violet" | "purple" => Some(ColorPalette::Violet),
            "copper" | "bronze" => Some(ColorPalette::Copper),
            "jade" | "teal" => Some(ColorPalette::Jade),
            _ => None,
        }
    }
}

/// Palette specification with sampling parameters
#[derive(Clone, Debug)]
pub struct PaletteSpec {
    /// Allowed hue ranges (0-360 degrees)
    pub hue_ranges: Vec<(f32, f32)>,
    /// Saturation range (0.0-1.0)
    pub saturation_range: (f32, f32),
    /// Lightness range (0.0-1.0)
    pub lightness_range: (f32, f32),
    /// Accent hue offset from primary
    pub accent_hue_offset: f32,
    /// Weight toward primary color (0.0-1.0)
    pub primary_weight: f32,
    /// Contrast preference (0.0 low to 1.0 high)
    pub contrast_preference: f32,
}

impl PaletteSpec {
    /// Sample a random color
    pub fn sample(&self, rng: &mut SimpleRng) -> [u8; 4] {
        let hue = self.sample_hue(rng);
        let sat = lerp(self.saturation_range.0, self.saturation_range.1, rng.next_f32());
        let light = lerp(self.lightness_range.0, self.lightness_range.1, rng.next_f32());

        // Convert HSL to RGB (using our HSV function with lightness adjustment)
        let value = light + sat * (1.0 - (2.0 * light - 1.0).abs()) / 2.0;
        let rgb = hsv_to_rgb(hue, sat, value);

        [rgb.0, rgb.1, rgb.2, 255]
    }

    /// Sample primary and accent colors
    pub fn sample_pair(&self, rng: &mut SimpleRng) -> ([u8; 4], [u8; 4]) {
        let primary_hue = self.sample_hue(rng);
        let accent_hue = (primary_hue + self.accent_hue_offset) % 360.0;

        let sat = lerp(self.saturation_range.0, self.saturation_range.1, rng.next_f32());
        let light = lerp(self.lightness_range.0, self.lightness_range.1, rng.next_f32());

        let value = light + sat * (1.0 - (2.0 * light - 1.0).abs()) / 2.0;

        let primary_rgb = hsv_to_rgb(primary_hue, sat, value);
        let accent_rgb = hsv_to_rgb(accent_hue, sat.min(0.8), value);

        (
            [primary_rgb.0, primary_rgb.1, primary_rgb.2, 255],
            [accent_rgb.0, accent_rgb.1, accent_rgb.2, 255],
        )
    }

    /// Sample full color set
    pub fn sample_full(&self, rng: &mut SimpleRng) -> ColorSet {
        let primary_hue = self.sample_hue(rng);
        let secondary_hue = (primary_hue + 30.0) % 360.0;
        let accent_hue = (primary_hue + self.accent_hue_offset) % 360.0;

        let sat = lerp(self.saturation_range.0, self.saturation_range.1, rng.next_f32());

        let mid_light = lerp(self.lightness_range.0, self.lightness_range.1, 0.5);
        let dark_light = self.lightness_range.0 * 0.7;
        let light_light = (self.lightness_range.1 + 0.2).min(0.95);

        let primary_v = mid_light + sat * (1.0 - (2.0 * mid_light - 1.0).abs()) / 2.0;
        let dark_v = dark_light + sat * 0.3 * (1.0 - (2.0 * dark_light - 1.0).abs()) / 2.0;
        let light_v = light_light + sat * 0.5 * (1.0 - (2.0 * light_light - 1.0).abs()) / 2.0;

        let primary_rgb = hsv_to_rgb(primary_hue, sat, primary_v);
        let secondary_rgb = hsv_to_rgb(secondary_hue, sat * 0.8, primary_v);
        let accent_rgb = hsv_to_rgb(accent_hue, sat.min(0.8), primary_v * 0.9);
        let dark_rgb = hsv_to_rgb(primary_hue, sat * 0.4, dark_v);
        let light_rgb = hsv_to_rgb(primary_hue, sat * 0.3, light_v);

        ColorSet {
            primary: [primary_rgb.0, primary_rgb.1, primary_rgb.2, 255],
            secondary: [secondary_rgb.0, secondary_rgb.1, secondary_rgb.2, 255],
            accent: [accent_rgb.0, accent_rgb.1, accent_rgb.2, 255],
            dark: [dark_rgb.0, dark_rgb.1, dark_rgb.2, 255],
            light: [light_rgb.0, light_rgb.1, light_rgb.2, 255],
        }
    }

    fn sample_hue(&self, rng: &mut SimpleRng) -> f32 {
        if self.hue_ranges.is_empty() {
            return rng.next_f32() * 360.0;
        }

        let range_idx = (rng.next() as usize) % self.hue_ranges.len();
        let (min_hue, max_hue) = self.hue_ranges[range_idx];

        lerp(min_hue, max_hue, rng.next_f32())
    }
}

/// A set of coordinated colors
#[derive(Clone, Copy, Debug)]
pub struct ColorSet {
    pub primary: [u8; 4],
    pub secondary: [u8; 4],
    pub accent: [u8; 4],
    pub dark: [u8; 4],
    pub light: [u8; 4],
}

impl ColorSet {
    /// Apply temperature shift to all colors
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.primary = apply_color_temp(self.primary, temperature);
        self.secondary = apply_color_temp(self.secondary, temperature);
        self.accent = apply_color_temp(self.accent, temperature);
        self.dark = apply_color_temp(self.dark, temperature);
        self.light = apply_color_temp(self.light, temperature);
        self
    }
}

fn apply_color_temp(color: [u8; 4], temperature: f32) -> [u8; 4] {
    let rgb = apply_temperature(color[0], color[1], color[2], temperature);
    [rgb.0, rgb.1, rgb.2, color[3]]
}

/// Simple deterministic RNG
pub struct SimpleRng {
    state: u32,
}

impl SimpleRng {
    pub fn new(seed: u32) -> Self {
        Self { state: seed.wrapping_add(1) }
    }

    pub fn next(&mut self) -> u32 {
        self.state = self.state.wrapping_mul(1103515245).wrapping_add(12345);
        self.state
    }

    pub fn next_f32(&mut self) -> f32 {
        (self.next() & 0x7FFFFFFF) as f32 / 0x7FFFFFFF as f32
    }
}

fn lerp(a: f32, b: f32, t: f32) -> f32 {
    a + (b - a) * t
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_palette_sample() {
        let mut rng = SimpleRng::new(42);
        let color = ColorPalette::CoolMetal.sample(&mut rng);

        assert_eq!(color[3], 255); // Alpha should be 255
    }

    #[test]
    fn test_palette_sample_pair() {
        let mut rng = SimpleRng::new(42);
        let (primary, accent) = ColorPalette::Fire.sample_pair(&mut rng);

        // Both should be valid colors
        assert_eq!(primary[3], 255);
        assert_eq!(accent[3], 255);
    }

    #[test]
    fn test_palette_from_str() {
        assert_eq!(ColorPalette::from_str("metal"), Some(ColorPalette::CoolMetal));
        assert_eq!(ColorPalette::from_str("NEON"), Some(ColorPalette::Neon));
        assert_eq!(ColorPalette::from_str("unknown"), None);
    }

    #[test]
    fn test_color_set() {
        let mut rng = SimpleRng::new(42);
        let set = ColorPalette::Ocean.sample_full(&mut rng);

        // Should have distinct colors
        assert_ne!(set.primary, set.dark);
        assert_ne!(set.primary, set.light);
    }
}
